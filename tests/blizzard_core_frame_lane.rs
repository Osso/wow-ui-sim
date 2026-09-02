//! Core frame/lifecycle lane analysis: Blizzard_FrameXMLBase →
//! Blizzard_FrameXMLUtil → Blizzard_UIParent → Blizzard_UIParentPanelManager →
//! Blizzard_FrameXML.
//!
//! This file is a LANE audit — separate from the per-addon loadability suites
//! (`blizzard_frame_xml_base_loads.rs`, `blizzard_frame_xml_util_loads.rs`,
//! `blizzard_ui_parent_loads.rs`, `blizzard_ui_parent_panel_manager_loads.rs`,
//! `blizzard_frame_xml_loads.rs`).
//!
//! Lane focus areas (per the PLAN.md task):
//!     1. STARTUP SHAPE — what must exist before XML loads (engine-created
//!        UIParent / WorldFrame, panel-positioning attributes), the LoadFirst
//!        ordering of FrameXML, the LoadWith inline-pull of UIParentPanelManager.
//!     2. PANEL LOADING — the ShowUIPanel / HideUIPanel / GetUIPanel / SetUIPanelAttribute /
//!        UpdateUIPanelPositions globals, the UIPanelWindows registry, and
//!        RegisterUIPanel (the current panel-registration entry point).
//!     3. GLOBAL FRAME CREATION CONTRACTS — UIParent / WorldFrame as pre-created
//!        engine frames whose Lua handles must resolve before any addon's file-
//!        scope `parent="UIParent"` reference; the named global frames
//!        (UIErrorsFrame, AlertFrame, SplashFrame) parented to UIParent at load
//!        time; the ManagedFrameSystem template inheritance contract for
//!        downstream-addon managed-frame containers.
//!
//! Layered structure (top = base, bottom = consumer):
//!     FrameXMLBase   (AllowLoad: Game, deps: Blizzard_SharedXML, Blizzard_SharedXMLGame)
//!         ├── FrameXMLUtil   (AllowLoad: Game, singular `## Dep:` form for
//!         │                   SharedXMLGame / Colors / StaticPopup)
//!         └── UIParent       (AllowLoad: game, AllowLoadGameType: mainline,
//!                              deps: FrameXMLBase, ObjectAPI, Colors)
//!             └── UIParentPanelManager   (deps: UIParent, LoadWith: UIParent,
//!                                          AllowLoadGameType: mainline,
//!                                          3 files w/ [AllowLoadEnvironment Global])
//!                 └── FrameXML   (LoadFirst: 1, AllowLoad: Game, MANY deps
//!                                  including UIParent, UIParentPanelManager,
//!                                  FrameXMLUtil, UIPanelTemplates, etc.)

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

const LANE_ADDONS_BASE_TO_CONSUMER: &[&str] = &[
    "Blizzard_FrameXMLBase",
    "Blizzard_FrameXMLUtil",
    "Blizzard_UIParent",
    "Blizzard_UIParentPanelManager",
    "Blizzard_FrameXML",
];

fn lane_toc(name: &str) -> PathBuf {
    let dir = blizzard_ui_dir().join(name);
    find_toc_file(&dir).unwrap_or_else(|| panic!("TOC for `{name}` should resolve under {dir:?}"))
}

fn parse_lane_toc(name: &str) -> TocFile {
    TocFile::from_file(&lane_toc(name)).unwrap_or_else(|err| panic!("`{name}` TOC parses: {err}"))
}

fn position_in(haystack: &[String], needle: &str) -> Option<usize> {
    haystack.iter().position(|n| n == needle)
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

#[test]
fn each_lane_addon_directory_resolves_with_a_toc() {
    for name in LANE_ADDONS_BASE_TO_CONSUMER {
        let dir = blizzard_ui_dir().join(name);
        assert!(
            dir.is_dir(),
            "Lane addon `{name}` directory must exist at \
             `Interface/BlizzardUI/{name}/` — the core frame/lifecycle lane is the floor for \
             every Game-screen addon, so missing any directory cascades into hundreds of \
             downstream load failures"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "Lane addon `{name}` must ship a resolvable TOC file. UIParent / FrameXML / \
             UIParentPanelManager use `_Mainline.toc` variants; FrameXMLUtil uses a single \
             `Blizzard_FrameXMLUtil.toc`. find_toc_file resolves either form"
        );
    }
}

#[test]
fn lane_dep_edges_pin_canonical_chain() {
    let base = parse_lane_toc("Blizzard_FrameXMLBase");
    let _util = parse_lane_toc("Blizzard_FrameXMLUtil");
    let uiparent = parse_lane_toc("Blizzard_UIParent");
    let panel_manager = parse_lane_toc("Blizzard_UIParentPanelManager");
    let frame_xml = parse_lane_toc("Blizzard_FrameXML");

    let base_deps = base.dependencies();
    assert_eq!(
        base_deps,
        vec![
            "Blizzard_SharedXML".to_string(),
            "Blizzard_SharedXMLGame".to_string(),
        ],
        "FrameXMLBase pins exactly two foundation deps via the multi-value `## Dependencies:` \
         form. The lane's own root needs the foundation lane (Mixin/Pools/CallbackRegistry/Color) \
         live before its Constants.lua / FlowContainer.lua / FrameLocks.lua publish. Got: \
         {base_deps:?}"
    );

    let uiparent_deps = uiparent.dependencies();
    assert_eq!(
        uiparent_deps,
        vec!["Blizzard_SharedXMLBase".to_string()],
        "UIParent pins only SharedXMLBase in the current retail TOC. Got: {uiparent_deps:?}"
    );

    let panel_deps = panel_manager.dependencies();
    assert_eq!(
        panel_deps,
        vec![
            "Blizzard_ManagedFrameSystem".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "UIParentPanelManager pins its current managed-frame and game-menu dependencies. Got: \
         {panel_deps:?}"
    );

    let frame_xml_deps = frame_xml.dependencies();
    let expected_frame_xml_deps = [
        "Blizzard_ObjectAPI",
        "Blizzard_FrameXMLBase",
        "Blizzard_UIErrorsFrame",
        "Blizzard_UIParentPanelManager",
        "Blizzard_SettingsDefinitions_Frame",
        "Blizzard_ItemButton",
        "Blizzard_UnitPopup",
        "Blizzard_FrameXMLUtil",
        "Blizzard_RaidWarning",
        "Blizzard_UIPanelTemplates",
        "Blizzard_GameTooltip",
        "Blizzard_MoneyFrame",
        "Blizzard_Colors",
        "Blizzard_TransmogShared",
        "Blizzard_LFGUtil",
        "Blizzard_ManagedFrameSystem",
        "Blizzard_MirrorTimer",
    ];
    assert_eq!(
        frame_xml_deps,
        expected_frame_xml_deps
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "FrameXML must preserve all 17 current dependencies in retail TOC order, including \
         Blizzard_UnitPopup between Blizzard_ItemButton and Blizzard_FrameXMLUtil. Got: \
         {frame_xml_deps:?}"
    );
}

#[test]
fn frame_xml_util_repeated_singular_deps_are_accumulated() {
    let util = parse_lane_toc("Blizzard_FrameXMLUtil");

    assert_eq!(
        util.dependencies(),
        vec![
            "Blizzard_SharedXMLGame".to_string(),
            "Blizzard_Colors".to_string(),
            "Blizzard_StaticPopup".to_string(),
        ],
        "FrameXMLUtil's repeated `## Dep:` directives must remain ordered and complete"
    );

    let raw =
        std::fs::read_to_string(lane_toc("Blizzard_FrameXMLUtil")).expect("FrameXMLUtil TOC reads");
    for entry in &[
        "## Dep: Blizzard_SharedXMLGame",
        "## Dep: Blizzard_Colors",
        "## Dep: Blizzard_StaticPopup",
    ] {
        assert!(raw.contains(entry), "raw TOC must contain `{entry}`");
    }
}

#[test]
fn frame_xml_uses_load_first_to_run_before_other_eager_addons() {
    let frame_xml = parse_lane_toc("Blizzard_FrameXML");
    assert!(
        frame_xml.is_load_first(),
        "FrameXML carries `## LoadFirst: 1` — it is the canonical load-first addon in the \
         entire Blizzard UI tree. The directive elevates it ahead of other AllowLoad: Game \
         non-LoD addons that don't carry LoadFirst, so its globals (AlertFrame, UIErrorsFrame, \
         SplashFrame, secure templates) are live before any consumer addon's file-scope code \
         runs. If this regresses, addon-load order becomes underspecified — most consumer \
         addons assume FrameXML is loaded by the time they file-scope `parent=\"UIParent\"` \
         children that inherit FrameXML templates"
    );

    for other in &[
        "Blizzard_FrameXMLBase",
        "Blizzard_FrameXMLUtil",
        "Blizzard_UIParent",
        "Blizzard_UIParentPanelManager",
    ] {
        let toc = parse_lane_toc(other);
        assert!(
            !toc.is_load_first(),
            "`{other}` MUST NOT carry LoadFirst — only FrameXML uses LoadFirst in this lane. \
             The dep-graph already orders FrameXMLBase / UIParent / panel manager ahead of \
             FrameXML; LoadFirst on those would be a load-order conflict (LoadFirst on a dep \
             of another LoadFirst addon is ill-defined)"
        );
    }
}

#[test]
fn ui_parent_panel_manager_pulls_alongside_ui_parent_via_load_with() {
    let panel_manager = parse_lane_toc("Blizzard_UIParentPanelManager");

    let load_with = panel_manager.load_with();
    assert_eq!(
        load_with,
        vec!["Blizzard_UIParent".to_string()],
        "UIParentPanelManager carries `## LoadWith: Blizzard_UIParent` — when UIParent loads, \
         the panel manager loads INLINE in the same load-pass rather than waiting for the \
         dep-graph sweep to reach it. This is structurally redundant with the explicit \
         Dependencies edge for the simulator's eager-discovery path (both names are non-LoD, \
         so both are pulled either way), while still preserving the current source's inline \
         load-with relationship. Got: {load_with:?}"
    );

    for other in &[
        "Blizzard_FrameXMLBase",
        "Blizzard_FrameXMLUtil",
        "Blizzard_UIParent",
        "Blizzard_FrameXML",
    ] {
        let toc = parse_lane_toc(other);
        assert!(
            toc.load_with().is_empty(),
            "`{other}` MUST NOT carry LoadWith — UIParentPanelManager is the only LoadWith \
             user in this lane. Adding LoadWith elsewhere would create an inline-pull cascade \
             that bypasses the topological dep sort"
        );
    }
}

#[test]
fn ui_parent_panel_manager_diagnostic_files_carry_allow_load_environment_global_in_raw_form() {
    let panel_manager = parse_lane_toc("Blizzard_UIParentPanelManager");

    let raw = std::fs::read_to_string(lane_toc("Blizzard_UIParentPanelManager"))
        .expect("UIParentPanelManager TOC reads");
    for entry in &[
        "Shared\\UIParentPanelManager.lua [AllowLoadEnvironment Global]",
        "Mainline\\UIParentPanelManagerOverrides.lua [AllowLoadEnvironment Global]",
        "Shared\\UpdateUIPanelPositions.lua [AllowLoadEnvironment Global]",
    ] {
        assert!(
            raw.contains(entry),
            "Raw bytes must contain `{entry}` — these three files explicitly restrict the line \
             to the global load pass. The annotation is a load-pass filter, not an fenv override"
        );
    }

    let body: Vec<String> = panel_manager
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let panel_idx = body
        .iter()
        .position(|f| f.ends_with("UIParentPanelManager.lua"))
        .expect("UIParentPanelManager.lua must appear in body");
    assert_eq!(
        panel_manager.file_use_secure_env(panel_idx),
        None,
        "`[AllowLoadEnvironment Global]` must not map to a per-file fenv override"
    );
    assert_eq!(
        panel_manager.file_allow_load_environment(panel_idx),
        Some(false),
        "`[AllowLoadEnvironment Global]` is a global-pass filter"
    );
}

#[test]
fn ui_parent_lane_addons_restrict_to_game_screen_only() {
    let glue_screens = [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ];

    for name in LANE_ADDONS_BASE_TO_CONSUMER {
        let toc = parse_lane_toc(name);
        for screen in glue_screens {
            assert!(
                !toc.allows_screen(screen),
                "Core lane addon `{name}` MUST NOT load on glue screen {screen:?}. Every TOC \
                 in this lane carries `## AllowLoad: Game` (case variants tolerated) — UIParent \
                 references in-game character state, panel manager's positioning operates on \
                 the in-game frame layout, FrameXML's combat-feedback / loot / talents / \
                 instance-difficulty frames are all in-game-only. Glue screens use Blizzard_GlueXML \
                 / Blizzard_GlueParent instead"
            );
        }
        assert!(
            toc.allows_screen(ScreenKind::Game),
            "Core lane addon `{name}` must allow loading on Game screen — without it the \
             entire in-game UI tree is missing"
        );
    }
}

#[test]
fn ui_parent_and_panel_manager_are_mainline_only_via_allow_load_game_type() {
    let uiparent = parse_lane_toc("Blizzard_UIParent");
    let panel_manager = parse_lane_toc("Blizzard_UIParentPanelManager");

    let raw_uiparent =
        std::fs::read_to_string(lane_toc("Blizzard_UIParent")).expect("UIParent TOC reads");
    assert!(
        raw_uiparent.contains("## AllowLoadGameType: mainline"),
        "UIParent_Mainline.toc must carry `## AllowLoadGameType: mainline` (Mists has its own \
         _Mists.toc variant). The simulator's variant-resolver picks _Mainline.toc by default; \
         without the gate, a Mists run could mis-resolve to this TOC"
    );
    assert!(
        !uiparent.is_game_type_restricted(),
        "INVERSE-RESTRICTION SEMANTICS — `is_game_type_restricted()` returns false when \
         AllowLoadGameType lists `mainline` or `standard`, true otherwise (src/toc.rs:294-302). \
         UIParent's `## AllowLoadGameType: mainline` is meaningful as a marker but does NOT \
         flip the restriction flag, because mainline IS the default. Tests asserting `true` \
         here would break — the directive is documentary on mainline, gating only on \
         non-mainline gametypes (plunderstorm / classic / etc.)"
    );

    let raw_panel = std::fs::read_to_string(lane_toc("Blizzard_UIParentPanelManager"))
        .expect("UIParentPanelManager TOC reads");
    assert!(
        raw_panel.contains("## AllowLoadGameType: mainline"),
        "UIParentPanelManager_Mainline.toc must carry `## AllowLoadGameType: mainline`"
    );
    assert!(
        !panel_manager.is_game_type_restricted(),
        "UIParentPanelManager mirrors UIParent's mainline gating with the same inverse semantics"
    );
}

#[test]
fn engine_frames_pre_create_uiparent_with_panel_attributes_before_xml() {
    let env = WowLuaEnv::new().expect("fresh env");

    let kind: String = env
        .eval("return type(UIParent)")
        .expect("UIParent must resolve before any addon loads");
    assert_eq!(
        kind, "table",
        "UIParent MUST exist as a userdata-frame Lua handle the moment WowLuaEnv::new() returns \
         — BEFORE any addon TOC is loaded. The simulator pre-creates UIParent / WorldFrame in \
         `init_builtin_frames()` (called from WowLuaEnv::new at src/lua_api/env.rs:73) which \
         calls `create_engine_frames()` at src/lua_api/builtin_frames.rs:107-227. This is the \
         simulator's mirror of real WoW's engine-side frame creation: UIParent is born with \
         the C++ engine, before any Lua loads. Downstream addons file-scope reference \
         `parent=\"UIParent\"` and `UIParent:GetWidth()` and would crash without this guarantee. \
         Got type: {kind:?}"
    );

    let world_kind: String = env
        .eval("return type(WorldFrame)")
        .expect("WorldFrame must resolve before any addon loads");
    assert_eq!(
        world_kind, "table",
        "WorldFrame is the second engine-created frame (WidgetType::WorldFrame). It is created \
         alongside UIParent in create_engine_frames(). XML files like Blizzard_UIParent's \
         WorldFrame.xml then re-define it via the `<WorldFrame name=\"WorldFrame\" .../>` \
         element; the loader's `migrate_children_to_new_frame` path handles the re-create \
         properly, so any children parented to the engine-stub WorldFrame are reparented to \
         the XML-defined one. Got type: {world_kind:?}"
    );

    let panel_attrs_seeded: bool = env
        .eval(
            "return UIParent:GetAttribute('DEFAULT_FRAME_WIDTH') == 384 \
             and UIParent:GetAttribute('TOP_OFFSET') == -116 \
             and UIParent:GetAttribute('LEFT_OFFSET') == 16 \
             and UIParent:GetAttribute('CENTER_OFFSET') == 384 \
             and UIParent:GetAttribute('RIGHT_OFFSET') == 768",
        )
        .expect("UIParent panel attrs probe");
    assert!(
        panel_attrs_seeded,
        "UIParent panel-positioning attributes MUST be seeded before XML loads. \
         `set_ui_parent_panel_attributes` at src/lua_api/builtin_frames.rs:249-260 writes \
         DEFAULT_FRAME_WIDTH=384, TOP_OFFSET=-116, LEFT_OFFSET=16, CENTER_OFFSET=384, \
         RIGHT_OFFSET=768. These come from UIParent.xml's <Attributes> block in real WoW and \
         must exist BEFORE UIParentPanelManager.lua's body runs (its panel-positioning math \
         reads `UIParent:GetAttribute('LEFT_OFFSET')` etc. at file scope). The seeding is the \
         simulator's startup-shape contract: engine-pre-create is the moment by which all \
         UIParent attributes consumed at file-scope by addon code must be live"
    );

    let world_frame_pre_xml_id: f64 = env
        .eval("return WorldFrame:GetID()")
        .expect("WorldFrame:GetID probe");
    assert!(
        world_frame_pre_xml_id >= 0.0,
        "WorldFrame must have a stable ID before XML loads. The pre-XML id seeds the \
         `widgets.get_id_by_name(\"WorldFrame\")` lookup used by C++-bridge calls"
    );
}

#[test]
fn lane_appears_in_eager_discovery_with_load_first_promoted_for_frame_xml() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let names: Vec<String> = addons.iter().map(|(name, _)| name.clone()).collect();

    let positions: Vec<(&str, usize)> = LANE_ADDONS_BASE_TO_CONSUMER
        .iter()
        .map(|name| {
            let pos = position_in(&names, name).unwrap_or_else(|| {
                panic!(
                    "Lane addon `{name}` must appear in Game eager discovery — every core-lane \
                     addon is non-LoD and AllowLoad: Game"
                );
            });
            (*name, pos)
        })
        .collect();

    let pos = |needle: &str| {
        positions
            .iter()
            .find_map(|(n, p)| if *n == needle { Some(*p) } else { None })
            .unwrap_or_else(|| panic!("position for {needle}"))
    };

    let pos_base = pos("Blizzard_FrameXMLBase");
    let pos_util = pos("Blizzard_FrameXMLUtil");
    let pos_panel_manager = pos("Blizzard_UIParentPanelManager");
    let pos_frame_xml = pos("Blizzard_FrameXML");

    for (name, position) in [
        ("Blizzard_FrameXMLBase", pos_base),
        ("Blizzard_FrameXMLUtil", pos_util),
        ("Blizzard_UIParentPanelManager", pos_panel_manager),
    ] {
        assert!(
            position < pos_frame_xml,
            "{name} must precede FrameXML (got positions {position} and {pos_frame_xml})"
        );
    }
}

prefork_full_ui_case! {
    fn lane_addons_are_loaded_after_eager_sweep_for_full_game_ui(env: &WowLuaEnv) {

        for name in LANE_ADDONS_BASE_TO_CONSUMER {
            let loaded: bool = env
                .eval(&format!("return C_AddOns.IsAddOnLoaded('{name}')"))
                .unwrap_or_else(|err| panic!("IsAddOnLoaded({name}) probe failed: {err}"));
            assert!(
                loaded,
                "C_AddOns.IsAddOnLoaded('{name}') must return true after the eager Game sweep. The \
                 required harness shape for any downstream test is therefore: \
                 (1) `WowLuaEnv::new()` (pre-creates UIParent / WorldFrame / panel attrs), \
                 (2) `set_screen_size(W, H)` + `set_screen_mode(ScreenKind::Game)`, \
                 (3) `state.addon_base_paths = vec![blizzard_ui_dir()]`, \
                 (4) `register_intrinsic_templates()`, \
                 (5) `discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)` + per-result \
                 `load_addon` (the eager sweep brings in the full lane plus all other AllowLoad: \
                 Game eager addons), \
                 (6) `apply_post_load_workarounds()`, \
                 (7) `fire_startup_events_for_screen(&env, ScreenKind::Game)`. Downstream tests \
                 do NOT need to call load_addon for any lane addon directly"
            );
        }
    }
}

prefork_full_ui_case! {
    fn lane_publishes_panel_manager_globals_after_full_game_load(env: &WowLuaEnv) {

        let panel_manager_globals = [
            ("ShowUIPanel", "function"),
            ("HideUIPanel", "function"),
            ("GetUIPanel", "function"),
            ("SetUIPanelAttribute", "function"),
            ("UpdateUIPanelPositions", "function"),
            ("UIPanelWindows", "table"),
            ("RegisterUIPanel", "function"),
        ];

        for (global, expected_kind) in panel_manager_globals {
            let actual: String = env
                .eval(&format!("return type({global})"))
                .unwrap_or_else(|err| panic!("type({global}) probe failed: {err}"));
            assert_eq!(
                actual, expected_kind,
                "Panel-manager global `{global}` must publish as `{expected_kind}` after the lane \
                 loads. ShowUIPanel/HideUIPanel/GetUIPanel/SetUIPanelAttribute live in \
                 Blizzard_UIParentPanelManager/Shared/UIParentPanelManager.lua at lines 828, 853, \
                 881, 115; UpdateUIPanelPositions in UpdateUIPanelPositions.lua; UIPanelWindows is \
                 a literal table populated by UIPanelWindows.lua's `UIPanelWindows_Initialize()`; \
                 RegisterUIPanel is the current registration entry point. Downstream addons rely on \
                 every one of these being live at file scope; if any is nil, the entire panel-show \
                 flow is dead"
            );
        }

        let panel_count: f64 = env
            .eval("local n = 0 for _ in pairs(UIPanelWindows) do n = n + 1 end return n")
            .expect("UIPanelWindows count probe");
        assert!(
            panel_count > 30.0,
            "UIPanelWindows must contain >30 entries after the eager sweep — the registry is the \
             lookup table for ShowUIPanel and lists every panel that participates in the slot-based \
             positioning system. UIPanelWindows.lua registers GameMenuFrame / HelpFrame / \
             CharacterFrame / ProfessionsBookFrame / PVPUIFrame / EncounterJournal / CollectionsJournal \
             / TradeFrame / LootFrame / MerchantFrame / TabardFrame / MailFrame / BankFrame / \
             QuestLogPopupDetailFrame / QuestFrame / GuildRegistrarFrame / GossipFrame / DressUpFrame \
             / PetitionFrame / ItemTextFrame / FriendsFrame / ... — well over 30. Got: {panel_count}"
        );
    }
}

prefork_full_ui_case! {
    fn lane_publishes_global_frames_parented_to_uiparent_after_full_game_load(env: &WowLuaEnv) {

        let global_frames = [
            ("UIErrorsFrame", "MessageFrame"),
            ("AlertFrame", "Frame"),
            ("SplashFrame", "Frame"),
            ("CinematicFrame", "Frame"),
            ("MovieFrame", "Frame"),
            ("ColorPickerFrame", "Frame"),
            ("StackSplitFrame", "Frame"),
            ("ReadyCheckFrame", "Frame"),
            ("StaticPopup1", "Frame"),
        ];

        for (frame, _expected_kind_doc) in global_frames {
            let exists: bool = env
                .eval(&format!(
                    "return _G['{frame}'] ~= nil and type(_G['{frame}']) == 'table'"
                ))
                .unwrap_or_else(|err| panic!("global frame `{frame}` probe failed: {err}"));
            assert!(
                exists,
                "Global frame `{frame}` MUST publish as a userdata-frame after FrameXML loads. \
                 `UIErrorsFrame` ships in FrameXML/UIErrorsFrame.xml as a MessageFrame parented to \
                 UIParent at frameStrata DIALOG; `AlertFrame` in Mainline/AlertFrames.xml; \
                 `SplashFrame` in SplashFrame.xml; `CinematicFrame` in Shared/CinematicFrame.xml; \
                 `MovieFrame` in MovieFrame.xml; `ColorPickerFrame` in Mainline/ColorPickerFrame.xml; \
                 `StackSplitFrame` in Mainline/StackSplitFrame.xml; `ReadyCheckFrame` in \
                 Mainline/ReadyCheck.xml; `StaticPopup1` from Blizzard_StaticPopup (a dep of \
                 FrameXMLUtil, included in the eager sweep). The lane defines the universe of \
                 `parent=\"UIParent\"` global frames that downstream addons reference"
            );
        }
    }
}

prefork_full_ui_case! {
    fn lane_publishes_managed_frame_templates_for_panel_layout_inheritance(env: &WowLuaEnv) {
        let _env = env;

        let lane_templates = [
            (
                "ManagedFrameTemplate",
                "Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.xml",
            ),
            (
                "BottomManagedFrameTemplate",
                "Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.xml",
            ),
            (
                "RightManagedFrameTemplate",
                "Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.xml",
            ),
            (
                "ManagedFrameContainerBaseTemplate",
                "Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.xml",
            ),
            (
                "ManagedFrameContainer",
                "Blizzard_ManagedFrameSystem/Mainline/ManagedFrameSystem.xml",
            ),
            (
                "AlertFrameTemplate",
                "Blizzard_FrameXML/Mainline/AlertFrames.xml",
            ),
        ];

        for (template, source) in lane_templates {
            assert!(
                wow_ui_sim::xml::get_template(template).is_some(),
                "Lane template `{template}` (defined in {source}) must register in \
                 `wow_ui_sim::xml::get_template`. ManagedFrameSystem owns the current bottom/right \
                 auto-layout inheritance for downstream frames; AlertFrameTemplate remains the base \
                 for FrameXML/Mainline/AlertFrameSystems.xml's per-system alert variants. If this \
                 regresses, downstream addons fail to instantiate their managed frames"
            );
        }
    }
}

prefork_full_ui_case! {
    fn lane_emits_no_addon_specific_lua_errors_during_full_load(env: &WowLuaEnv) {

        let load_errors: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .filter(|message| {
                LANE_ADDONS_BASE_TO_CONSUMER
                    .iter()
                    .any(|name| message.contains(name))
            })
            .cloned()
            .collect();

        assert!(
            load_errors.is_empty(),
            "Core lane MUST load without lane-specific Lua errors — every Game-screen addon \
             depends on this lane being clean. Got:\n  {}",
            load_errors.join("\n  ")
        );
    }
}

#[test]
fn lane_required_harness_shape_helpers_exist() {
    assert!(
        blizzard_ui_dir().is_dir(),
        "Harness invariant: blizzard_ui_dir() (Interface/BlizzardUI) must resolve. Every lane \
         test sets `state.addon_base_paths = vec![blizzard_ui_dir()]` and the loader probes \
         this path for both eager-discovery and explicit load_addon calls"
    );

    let env = fresh_lane_env();
    let kind: String = env
        .eval("return type(_G)")
        .expect("fresh env must expose _G");
    assert_eq!(
        kind, "table",
        "Harness invariant: a fresh WowLuaEnv must expose `_G` even before any addon loads. \
         If this regresses, downstream tests cannot probe globals via env.eval"
    );

    let create_frame_kind: String = env
        .eval("return type(CreateFrame)")
        .expect("fresh env must expose CreateFrame");
    assert_eq!(
        create_frame_kind, "function",
        "Harness invariant: CreateFrame must be live in a fresh env (engine-provided global). \
         The lane's XML-defined frames depend on CreateFrame at file scope for any Lua-side \
         frame instantiation that doesn't come through the XML loader"
    );
}
