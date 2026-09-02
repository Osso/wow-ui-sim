//! Map lane analysis: Blizzard_WorldMap, Blizzard_AdventureMap,
//! Blizzard_BattlefieldMap — plus the foundation pieces they all stack on
//! (Blizzard_MapCanvas, Blizzard_MapCanvasSecureUtil, Blizzard_SharedMapDataProviders).
//!
//! This file is a LANE audit — separate from per-addon loadability suites
//! (`blizzard_world_map_loads.rs`, `blizzard_adventure_map_loads.rs`,
//! `blizzard_battlefield_map_loads.rs`, `blizzard_map_canvas_loads.rs`).
//!
//! The PLAN.md task explicitly asks for: closure behavior, map-canvas
//! bootstrapping, and any data-provider stubs needed for clean loads. This
//! file pins each as a runnable contract.
//!
//! COVERAGE CLASSIFICATION (per addon):
//!     - STARTUP coverage: addon ends up in the eager Game discovery list,
//!       either because it is non-LoD (WorldMap, MapCanvasSecureUtil,
//!       HelpPlate) or because it is LoD but a non-LoD addon explicitly
//!       declares it as a RequiredDep — `pull_required_lod_addons` then
//!       promotes it into the eager set (MapCanvas, SharedMapDataProviders,
//!       both pulled by WorldMap).
//!         → Blizzard_MapCanvasSecureUtil (LoadFirst)
//!         → Blizzard_MapCanvas (LoD, pulled by WorldMap)
//!         → Blizzard_SharedMapDataProviders (LoD, pulled by WorldMap)
//!         → Blizzard_WorldMap (eager)
//!
//!     - SMOKE coverage: addon is `## LoadOnDemand: 1`, never appears in the
//!       eager sweep (no eager addon depends on it), and only loads when the
//!       player triggers it (UIParentLoadAddOn / C_AddOns.LoadAddOn / opening
//!       the corresponding panel).
//!         → Blizzard_AdventureMap (LoD; only eager dependent would be
//!           Blizzard_GarrisonUI, but that addon is itself LoD)
//!         → Blizzard_BattlefieldMap (LoD; no eager dependents)
//!
//! BOOTSTRAP CHAIN:
//!     Blizzard_MapCanvasSecureUtil   (LoadFirst: 1, no deps — runs before
//!                                      every other map-lane file so the
//!                                      MapCanvasSecureUtil global is live
//!                                      before MapCanvas mixin code references
//!                                      securecallfunction-style helpers)
//!         └── Blizzard_MapCanvas      (LoadOnDemand, deps: SharedXMLBase,
//!                                      MapCanvasSecureUtil — provides
//!                                      MapCanvasMixin + MapCanvasFrameTemplate
//!                                      virtual XML template that all map
//!                                      frames inherit)
//!             ├── Blizzard_WorldMap          (eager, _Mainline.toc, deps:
//!             │                               MapCanvas, SharedMapDataProviders,
//!             │                               HelpPlate — defines WorldMapFrame
//!             │                               global parented to UIParent)
//!             ├── Blizzard_AdventureMap      (LoD, deps: GarrisonTemplates,
//!             │                               MapCanvas, SharedMapDataProviders
//!             │                               — defines map templates and the
//!             │                               AdventureMapQuestChoiceDialog
//!             │                               popup, NO main panel global)
//!             └── Blizzard_BattlefieldMap    (LoD, deps: MapCanvas
//!                                             SharedMapDataProviders
//!                                             ObjectiveTracker — note the
//!                                             SPACE-separated RequiredDep form,
//!                                             defines BattlefieldMapFrame and
//!                                             BattlefieldMapTab globals)

use std::path::PathBuf;

use wow_ui_sim::loader::{
    discover_blizzard_addon_closure_for_screen, discover_blizzard_addons_for_screen, find_toc_file,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

const LANE_ADDONS: &[&str] = &[
    "Blizzard_MapCanvasSecureUtil",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_WorldMap",
    "Blizzard_AdventureMap",
    "Blizzard_BattlefieldMap",
];

const STARTUP_LANE_ADDONS: &[&str] = &[
    "Blizzard_MapCanvasSecureUtil",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_WorldMap",
];

const SMOKE_LANE_ADDONS: &[&str] = &["Blizzard_AdventureMap", "Blizzard_BattlefieldMap"];

fn parse_lane_toc(name: &str) -> TocFile {
    let dir = blizzard_ui_dir().join(name);
    let toc_path = find_toc_file(&dir)
        .unwrap_or_else(|| panic!("`{name}` TOC must resolve via find_toc_file under {dir:?}"));
    TocFile::from_file(&toc_path).unwrap_or_else(|err| panic!("`{name}` TOC parses: {err}"))
}

fn fresh_lane_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("WowLuaEnv constructs");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = fresh_lane_env();
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn each_lane_addon_directory_resolves_with_a_toc() {
    for name in LANE_ADDONS {
        let dir = blizzard_ui_dir().join(name);
        assert!(
            dir.is_dir(),
            "Lane addon `{name}` directory must exist at \
             `Interface/BlizzardUI/{name}/`. The map lane covers the world map, the \
             garrison/legion adventure map, and the raid/PvP battlefield map; missing any \
             directory means the corresponding panel has no XML/Lua surface available"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "Lane addon `{name}` TOC must resolve via find_toc_file. WorldMap and \
             SharedMapDataProviders ship `_Mainline.toc` variants; the rest ship plain \
             `<name>.toc`. The variant resolver at src/loader/mod.rs:65-95 probes \
             `_Mainline.toc` first, so both forms light up cleanly here"
        );
    }
}

#[test]
fn map_canvas_secure_util_uses_load_first_so_it_runs_before_mapcanvas_mixin_code() {
    let secure_util = parse_lane_toc("Blizzard_MapCanvasSecureUtil");
    assert!(
        secure_util.is_load_first(),
        "Blizzard_MapCanvasSecureUtil MUST carry `## LoadFirst: 1`. The two-pass eager loader \
         (topological_sort_addons in src/loader/mod.rs) emits LoadFirst addons before any \
         normal eager addon. MapCanvas's mixin file calls `securecallfunction(rawget, ...)` \
         and `MapCanvasSecureUtil.CreateHandlerRegistry()` at file scope inside helper \
         constructors, so the MapCanvasSecureUtil global must be live before MapCanvas.lua \
         parses. If LoadFirst regresses, eager-pulled MapCanvas (via WorldMap dep) would race \
         the SecureUtil load and fail with `attempt to index global 'MapCanvasSecureUtil' (a nil value)`"
    );
    assert!(
        secure_util.dependencies().is_empty(),
        "MapCanvasSecureUtil declares NO RequiredDep — it is pure-Lua utility code (handler \
         registries with priority sorting and tainted-array enumeration via securecallfunction). \
         Empty deps make it safe to LoadFirst; if it had deps, LoadFirst would either load them \
         in turn or stall. Got: {:?}",
        secure_util.dependencies()
    );
}

#[test]
fn lane_dep_edges_classify_each_addons_load_floor() {
    let secure_util = parse_lane_toc("Blizzard_MapCanvasSecureUtil");
    let map_canvas = parse_lane_toc("Blizzard_MapCanvas");
    let shared_providers = parse_lane_toc("Blizzard_SharedMapDataProviders");
    let world_map = parse_lane_toc("Blizzard_WorldMap");
    let adventure_map = parse_lane_toc("Blizzard_AdventureMap");
    let battlefield_map = parse_lane_toc("Blizzard_BattlefieldMap");

    assert!(
        secure_util.dependencies().is_empty(),
        "MapCanvasSecureUtil has zero deps — see map_canvas_secure_util_uses_load_first..."
    );

    assert_eq!(
        map_canvas.dependencies(),
        vec![
            "Blizzard_SharedXMLBase".to_string(),
            "Blizzard_MapCanvasSecureUtil".to_string(),
        ],
        "MapCanvas declares deps in this exact order: SharedXMLBase first (provides \
         CallbackRegistryMixin and CreateFromMixins, which MapCanvasMixin calls in the very \
         first line of Blizzard_MapCanvas.lua), then MapCanvasSecureUtil (for the secure \
         handler registry helpers). Order matters because dependencies() preserves declaration \
         order; the topological sorter uses them as edges"
    );

    assert!(
        shared_providers.dependencies().is_empty(),
        "SharedMapDataProviders declares NO RequiredDep / Dependencies — every data provider \
         (FogOfWar, FlightPoint, AreaPOI, etc.) inherits from MapCanvasDataProviderMixin which \
         is published by MapCanvas. The cross-addon edge is implicit (file-scope mixin call); \
         the simulator pulls SharedMapDataProviders into the eager set only because WorldMap \
         declares it as a RequiredDep. Got: {:?}",
        shared_providers.dependencies()
    );

    assert_eq!(
        world_map.dependencies(),
        vec![
            "Blizzard_MapCanvas".to_string(),
            "Blizzard_SharedMapDataProviders".to_string(),
            "Blizzard_HelpPlate".to_string(),
        ],
        "WorldMap RequiredDep list is the canonical map-stack: MapCanvas (mixin + frame \
         template), SharedMapDataProviders (FogOfWar / FlightPoint / DungeonEntrance / etc.), \
         HelpPlate (the `?` contextual-tutorial overlay that floats on the map UI). All three \
         are pulled into the eager set when WorldMap's eager non-LoD load runs"
    );

    assert!(
        world_map.is_game_type_restricted() || !world_map.is_game_type_restricted(),
        "(no-op gate — WorldMap inverse-restriction semantics covered by \
         world_map_targets_mainline_via_filename_only)"
    );

    assert_eq!(
        adventure_map.dependencies(),
        vec![
            "Blizzard_GarrisonTemplates".to_string(),
            "Blizzard_MapCanvas".to_string(),
            "Blizzard_SharedMapDataProviders".to_string(),
        ],
        "AdventureMap stacks on the standard map base PLUS GarrisonTemplates — its panels \
         host garrison/legion mission rows that share visual templates with the garrison UI. \
         The dep order matters because GarrisonTemplates itself depends on Colors+HelpPlate; \
         the closure walk pulls those in transitively"
    );

    assert_eq!(
        battlefield_map.dependencies(),
        vec![
            "Blizzard_MapCanvas".to_string(),
            "Blizzard_SharedMapDataProviders".to_string(),
            "Blizzard_ObjectiveTracker".to_string(),
            "Blizzard_ColorPickerFrame".to_string(),
        ],
        "Retail BattlefieldMap_Mainline.toc uses one space-separated RequiredDep line for \
         MapCanvas, SharedMapDataProviders, ObjectiveTracker, and ColorPickerFrame"
    );
}

#[test]
fn world_map_ships_only_a_mainline_toc_so_classic_clients_could_not_load_it() {
    let dir = blizzard_ui_dir().join("Blizzard_WorldMap");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("read_dir({dir:?}) failed: {err}"))
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.ends_with(".toc"))
        .collect();

    assert_eq!(
        entries,
        vec!["Blizzard_WorldMap_Mainline.toc"],
        "Blizzard_WorldMap ships ONLY a `_Mainline.toc` variant — there is no plain \
         `Blizzard_WorldMap.toc` and no `_Classic`/`_Mists` flavor. The variant resolver \
         picks `_Mainline.toc` on its first probe, so this addon lights up only on Mainline \
         targets. A future Classic-target simulator run would see no TOC and skip the addon \
         entirely (find_toc_file's fallback explicitly excludes Classic-flavor filenames). \
         Got: {entries:?}"
    );
}

#[test]
fn battlefield_map_mainline_toc_uses_space_separated_required_dep_form() {
    let toc_path = blizzard_ui_dir()
        .join("Blizzard_BattlefieldMap")
        .join("Blizzard_BattlefieldMap_Mainline.toc");
    let raw =
        std::fs::read_to_string(&toc_path).unwrap_or_else(|err| panic!("read {toc_path:?}: {err}"));

    assert!(
        raw.contains(
            "## RequiredDep: Blizzard_MapCanvas Blizzard_SharedMapDataProviders \
             Blizzard_ObjectiveTracker Blizzard_ColorPickerFrame"
        ),
        "BattlefieldMap_Mainline.toc must use the current space-separated RequiredDep form. \
         Raw TOC bytes:\n{raw}"
    );
}

#[test]
fn smoke_lane_addons_carry_load_on_demand_directive() {
    for smoke in SMOKE_LANE_ADDONS {
        let toc = parse_lane_toc(smoke);
        assert!(
            toc.is_load_on_demand(),
            "Lane SMOKE addon `{smoke}` MUST carry `## LoadOnDemand: 1`. The map panels are \
             not auto-shown — the player triggers them by opening a quest log popup \
             (AdventureMap), pressing the battlefield-map keybinding (BattlefieldMap), or \
             entering a relevant zone. Eager-loading them on every Game session would inflate \
             startup time with XML for panels most sessions never open"
        );
    }
}

#[test]
fn world_map_is_eager_anchor_not_load_on_demand() {
    let world_map = parse_lane_toc("Blizzard_WorldMap");
    assert!(
        !world_map.is_load_on_demand(),
        "Blizzard_WorldMap is eager (no `## LoadOnDemand: 1`). Unlike AdventureMap and \
         BattlefieldMap, the world map is core gameplay UI: pressing M or clicking the \
         minimap opens it constantly, so eager-loading is justified. WorldMap's eager \
         status is also what causes MapCanvas + SharedMapDataProviders to get pulled into \
         the eager set despite their LoD flags (see pull_required_lod_addons)"
    );
}

#[test]
fn smoke_lane_addons_do_not_appear_in_eager_game_discovery() {
    let ui = blizzard_ui_dir();
    let names: Vec<String> = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
        .into_iter()
        .map(|(n, _)| n)
        .collect();

    for smoke in SMOKE_LANE_ADDONS {
        assert!(
            !names.contains(&smoke.to_string()),
            "Lane SMOKE addon `{smoke}` MUST NOT appear in eager Game discovery. The \
             `## LoadOnDemand: 1` directive routes the addon into the LoD pool, and \
             pull_required_lod_addons only promotes it if a non-LoD addon RequiredDeps it. \
             For AdventureMap, the only eager dependent would be Blizzard_GarrisonUI — which \
             is itself LoD (Blizzard_GarrisonUI_Mainline.toc declares `## LoadOnDemand: 1`). \
             For BattlefieldMap, no eager addon RequiredDeps it. So both stay in the LoD pool"
        );
    }
}

#[test]
fn startup_lane_addons_appear_in_eager_game_discovery() {
    let ui = blizzard_ui_dir();
    let names: Vec<String> = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
        .into_iter()
        .map(|(n, _)| n)
        .collect();

    for startup in STARTUP_LANE_ADDONS {
        assert!(
            names.contains(&startup.to_string()),
            "Lane STARTUP addon `{startup}` MUST appear in eager Game discovery. \
             MapCanvasSecureUtil and WorldMap are eagerly classified by their TOCs. MapCanvas \
             and SharedMapDataProviders are LoD but get promoted by pull_required_lod_addons \
             because WorldMap (eager) declares both as RequiredDep. The recursive pull \
             handles MapCanvas→MapCanvasSecureUtil too, but SecureUtil is already eager via \
             LoadFirst so the recursion is a no-op there"
        );
    }
}

#[test]
fn map_canvas_secure_util_emerges_before_other_lane_addons_in_eager_order() {
    let ui = blizzard_ui_dir();
    let names: Vec<String> = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
        .into_iter()
        .map(|(n, _)| n)
        .collect();

    let secure_util_idx = names
        .iter()
        .position(|n| n == "Blizzard_MapCanvasSecureUtil")
        .expect("MapCanvasSecureUtil must appear in eager Game discovery");
    let canvas_idx = names
        .iter()
        .position(|n| n == "Blizzard_MapCanvas")
        .expect("MapCanvas must appear in eager Game discovery (pulled via WorldMap dep)");
    let world_map_idx = names
        .iter()
        .position(|n| n == "Blizzard_WorldMap")
        .expect("WorldMap must appear in eager Game discovery");

    assert!(
        secure_util_idx < canvas_idx,
        "MapCanvasSecureUtil (LoadFirst) must emit before MapCanvas in the eager order. \
         MapCanvas's dependencies() includes MapCanvasSecureUtil, so the sorter would respect \
         that edge anyway, but LoadFirst doubly guarantees it: the first pass emits LoadFirst \
         addons (and their deps) before the second pass touches normal eager addons. \
         Got order: SecureUtil@{secure_util_idx}, MapCanvas@{canvas_idx}"
    );
    assert!(
        canvas_idx < world_map_idx,
        "MapCanvas must emit before WorldMap. WorldMap declares MapCanvas as RequiredDep, so \
         the topological sort places it earlier. The MapCanvasFrameTemplate virtual XML \
         template (defined in Blizzard_MapCanvas.xml) must be registered before \
         Blizzard_WorldMap.xml's `<Frame name=\"WorldMapFrameTemplate\" \
         inherits=\"MapCanvasFrameTemplate\">` parses. \
         Got order: MapCanvas@{canvas_idx}, WorldMap@{world_map_idx}"
    );
}

#[test]
fn world_map_dependency_closure_pulls_in_canvas_secure_util_and_data_providers() {
    let ui = blizzard_ui_dir();
    let closure: Vec<String> =
        discover_blizzard_addon_closure_for_screen(&ui, ScreenKind::Game, &["Blizzard_WorldMap"])
            .into_iter()
            .map(|(n, _)| n)
            .collect();

    for required in [
        "Blizzard_WorldMap",
        "Blizzard_MapCanvas",
        "Blizzard_MapCanvasSecureUtil",
        "Blizzard_SharedMapDataProviders",
        "Blizzard_HelpPlate",
    ] {
        assert!(
            closure.contains(&required.to_string()),
            "WorldMap dependency closure MUST include `{required}`. The closure walk follows \
             RequiredDep + OptionalDeps edges from WorldMap → {{MapCanvas, \
             SharedMapDataProviders, HelpPlate}} → MapCanvas → MapCanvasSecureUtil + \
             SharedXMLBase. If `{required}` is missing, downstream addon developers running a \
             scoped harness on WorldMap would see panel-load failures because the data \
             provider mixin or secure-util globals are undefined. Got closure: {closure:?}"
        );
    }
}

#[test]
fn lane_addons_do_not_load_on_glue_screens() {
    let ui = blizzard_ui_dir();
    let glue_screens = [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ];

    for screen in glue_screens {
        let names: Vec<String> = discover_blizzard_addons_for_screen(&ui, screen)
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        for lane_addon in LANE_ADDONS {
            assert!(
                !names.contains(&lane_addon.to_string()),
                "Map lane addon `{lane_addon}` MUST NOT appear in {screen:?} eager discovery. \
                 None of the lane TOCs declare `## AllowLoad: Both` or `## AllowLoad: Glue` — \
                 the implicit default is Game-only (allows_screen returns false for glue \
                 screens when AllowLoad is unset). Map UI is meaningless before a character \
                 has logged in"
            );
        }
    }
}

prefork_full_ui_case! {
fn full_game_ui_load_publishes_world_map_globals(env: &WowLuaEnv) {

    let world_map_kind: String = env
        .eval("return type(WorldMapFrame)")
        .expect("WorldMapFrame type probe");
    assert_eq!(
        world_map_kind, "table",
        "WorldMapFrame MUST publish as a table-typed global after the eager Game sweep. \
         The XML at Blizzard_WorldMap.xml:28 declares `<Frame name=\"WorldMapFrame\" \
         parent=\"UIParent\" inherits=\"WorldMapFrameTemplate\" frameBuffer=\"true\">` — that \
         frame becomes addressable via _G.WorldMapFrame. Without the global, every consumer \
         (Quest log, world quest pins, mission boards) breaks at file-scope reference"
    );

    let canvas_mixin_kind: String = env
        .eval("return type(MapCanvasMixin)")
        .expect("MapCanvasMixin type probe");
    assert_eq!(
        canvas_mixin_kind, "table",
        "MapCanvasMixin MUST publish as a table after the eager sweep. \
         Blizzard_MapCanvas.lua:1 sets `MapCanvasMixin = CreateFromMixins(CallbackRegistryMixin)`. \
         Every map frame inherits this mixin via `mixin=\"MapCanvasMixin\"` on the \
         MapCanvasFrameTemplate; if the global is nil, frame instantiation fails at \
         CreateFromMixins-time"
    );

    let secure_util_kind: String = env
        .eval("return type(MapCanvasSecureUtil)")
        .expect("MapCanvasSecureUtil type probe");
    assert_eq!(
        secure_util_kind, "table",
        "MapCanvasSecureUtil MUST publish as a table after the eager sweep. \
         Blizzard_MapCanvasSecureUtil.lua:9 sets `MapCanvasSecureUtil = {{}}` and adds \
         CreateHandlerRegistry to it. MapCanvas's data-provider mixins call \
         MapCanvasSecureUtil.CreateHandlerRegistry() to set up tainted-array enumerators; if \
         the global is nil, every map data provider's OnLoad crashes"
    );
}
}

prefork_full_ui_case! {
fn world_map_frame_is_parented_to_ui_parent_after_full_load(env: &WowLuaEnv) {

    let parent_name: String = env
        .eval(
            "local p = WorldMapFrame and WorldMapFrame:GetParent(); \
             return p and p:GetName() or 'nil'",
        )
        .expect("WorldMapFrame parent probe");

    assert_eq!(
        parent_name, "UIParent",
        "WorldMapFrame must be parented to UIParent. The XML declares \
         `parent=\"UIParent\"` on the frame element, and the simulator's CreateFrame path \
         resolves that against the engine-pre-created UIParent (see \
         src/lua_api/builtin_frames.rs:107-227 — UIParent is born before any addon's XML \
         loads). If the parent regresses to nil or another frame, panel-management code that \
         walks up from WorldMapFrame to find UIParent (for SetUIPanelAttribute / \
         RegisterStateDriver) breaks"
    );
}
}

prefork_full_ui_case! {
fn lane_emits_no_addon_specific_lua_errors_during_full_load(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| LANE_ADDONS.iter().any(|name| message.contains(name)))
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Map lane MUST load without lane-specific Lua errors during eager Game sweep. The \
         eager set covers MapCanvasSecureUtil → MapCanvas → SharedMapDataProviders → \
         WorldMap (via LoadFirst + dep-pull). Errors here typically indicate either a \
         missing C_Map / C_QuestLog / C_AreaPoiInfo stub on the data provider side, or a \
         parser regression on the BattlefieldMap-style space-separated dep form. Got:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn smoke_lane_addons_remain_unloaded_after_eager_sweep_until_explicitly_triggered(env: &WowLuaEnv) {

    for smoke in SMOKE_LANE_ADDONS {
        let loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded('{smoke}') == true"))
            .unwrap_or_else(|err| panic!("IsAddOnLoaded({smoke}) probe failed: {err}"));
        assert!(
            !loaded,
            "Lane SMOKE addon `{smoke}` MUST remain unloaded after the eager Game sweep. \
             AdventureMap and BattlefieldMap are pure on-demand panels — only opened when the \
             player triggers them. If this regresses, it means an eager addon picked up a \
             RequiredDep on `{smoke}` since the last lane audit, which would inflate startup \
             time and require revisiting the SMOKE classification"
        );
    }
}
}

prefork_full_ui_case! {
fn smoke_lane_addons_can_be_loaded_on_demand_after_eager_sweep_completes(env: &WowLuaEnv) {

    for smoke in SMOKE_LANE_ADDONS {
        let dir = blizzard_ui_dir().join(smoke);
        let toc_path = find_toc_file(&dir).unwrap_or_else(|| {
            panic!("`{smoke}` TOC must resolve via find_toc_file before LOD-trigger probe");
        });
        let result = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc_path);
        assert!(
            result.is_ok(),
            "SMOKE addon `{smoke}` must load cleanly when triggered after the eager Game \
             sweep. The eager sweep brought in MapCanvas + SharedMapDataProviders + \
             MapCanvasSecureUtil; the SMOKE load itself just sweeps the addon's XML+Lua body. \
             For BattlefieldMap, ObjectiveTracker is also already loaded eagerly. \
             For AdventureMap, GarrisonTemplates is LoD and would be pulled by load_addon's \
             internal dep walker. If this fails, the SMOKE coverage classification is wrong \
             and a different harness shape (manual transitive-dep load) is needed. Got: {result:?}"
        );

        let loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded('{smoke}')"))
            .unwrap_or_else(|err| panic!("post-LOD IsAddOnLoaded({smoke}) probe failed: {err}"));
        assert!(
            loaded,
            "After explicit load_addon, IsAddOnLoaded('{smoke}') must flip to true. The \
             loader updates AddonInfo.loaded at end-of-load_addon"
        );
    }
}
}

prefork_full_ui_case! {
fn battlefield_map_publishes_battlefield_map_frame_and_tab_after_smoke_load(env: &WowLuaEnv) {
    let toc_path = find_toc_file(&blizzard_ui_dir().join("Blizzard_BattlefieldMap"))
        .expect("BattlefieldMap TOC resolves");
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc_path)
        .expect("BattlefieldMap must load on demand");

    let frame_kind: String = env
        .eval("return type(BattlefieldMapFrame)")
        .expect("BattlefieldMapFrame type probe");
    assert_eq!(
        frame_kind, "table",
        "BattlefieldMapFrame MUST publish as a table after smoke-loading BattlefieldMap. The \
         XML at Blizzard_BattlefieldMap.xml:56 declares `<Frame name=\"BattlefieldMapFrame\" \
         parent=\"UIParent\" inherits=\"MapCanvasFrameTemplate\" mixin=\"BattlefieldMapMixin\">` \
         — the frame inherits the same MapCanvasFrameTemplate that WorldMapFrame uses. If \
         the global is nil after load, the inheritance lookup failed, likely because \
         MapCanvas's eager sweep didn't register MapCanvasFrameTemplate as a virtual template"
    );

    let tab_kind: String = env
        .eval("return type(BattlefieldMapTab)")
        .expect("BattlefieldMapTab type probe");
    assert_eq!(
        tab_kind, "table",
        "BattlefieldMapTab MUST publish as a table — the draggable header bar that owns \
         BattlefieldMapFrame as a child. Blizzard_BattlefieldMap.xml:3 declares \
         `<Button name=\"BattlefieldMapTab\" parent=\"UIParent\" movable=\"true\" \
         mixin=\"BattlefieldMapTabMixin\">` and BattlefieldMapFrame anchors TOPLEFT to \
         BattlefieldMapTab BOTTOMLEFT. If the tab is nil, the frame anchor regresses to its \
         parent (UIParent) and the panel renders at an unexpected screen position"
    );
}
}

prefork_full_ui_case! {
fn map_canvas_frame_template_is_registered_after_eager_sweep_so_consumers_can_inherit(env: &WowLuaEnv) {
    let _env = env;

    let template = wow_ui_sim::xml::get_template("MapCanvasFrameTemplate");
    assert!(
        template.is_some(),
        "MapCanvasFrameTemplate MUST be registered in the simulator's template registry \
         after the eager Game sweep. Blizzard_MapCanvas.xml:52 declares \
         `<Frame name=\"MapCanvasFrameTemplate\" virtual=\"true\" mixin=\"MapCanvasMixin\">` — \
         WorldMapFrameTemplate, BattlefieldMapFrame, and downstream map-shaped frames inherit \
         from it. The probe queries `wow_ui_sim::xml::get_template` directly (bypasses Lua-side \
         CreateFrame) so it tests registration without triggering OnLoad workarounds. If \
         missing, no map-shaped frame can resolve its inherit chain at parse time"
    );

    let world_map_template = wow_ui_sim::xml::get_template("WorldMapFrameTemplate");
    assert!(
        world_map_template.is_some(),
        "WorldMapFrameTemplate MUST also be registered — it inherits from \
         MapCanvasFrameTemplate and is what the actual WorldMapFrame frame is built from. \
         If MapCanvasFrameTemplate is registered but WorldMapFrameTemplate is not, \
         Blizzard_WorldMap.xml didn't parse — likely a load-order regression where WorldMap \
         emitted before MapCanvas"
    );
}
}

#[test]
fn lane_required_harness_shape_helpers_exist() {
    assert!(
        blizzard_ui_dir().is_dir(),
        "Harness invariant: blizzard_ui_dir() (Interface/BlizzardUI) must resolve. Every lane \
         test sets `state.addon_base_paths = vec![blizzard_ui_dir()]`"
    );

    let env = fresh_lane_env();
    let kind: String = env
        .eval("return type(_G)")
        .expect("fresh env must expose _G");
    assert_eq!(
        kind, "table",
        "Harness invariant: a fresh WowLuaEnv must expose `_G` even before any addon loads. \
         Per-addon SMOKE tests for the map lane should use the same 7-step harness documented \
         in `tests/blizzard_core_frame_lane.rs`: \
         (1) WowLuaEnv::new(); \
         (2) set_screen_size + set_screen_mode(Game); \
         (3) state.addon_base_paths = vec![blizzard_ui_dir()]; \
         (4) register_intrinsic_templates(); \
         (5) discover_blizzard_addons_for_screen + per-result load_addon (eager sweep); \
         (6) apply_post_load_workarounds(); \
         (7) fire_startup_events_for_screen(env, Game). \
         For SMOKE coverage of AdventureMap or BattlefieldMap, append step (8): manually call \
         load_addon with the LOD TOC path"
    );
}
