//! Progression UI lane analysis: Blizzard_TalentUI, Blizzard_SharedTalentUI,
//! Blizzard_AzeriteUI, Blizzard_AzeriteEssenceUI, Blizzard_AzeriteRespecUI,
//! Blizzard_Soulbinds, Blizzard_LandingSoulbinds.
//!
//! This file is a LANE audit — separate from the per-addon loadability suites
//! (`blizzard_talent_ui_loads.rs`, `blizzard_shared_talent_ui_loads.rs`,
//! `blizzard_azerite_respec_ui_global_strings.rs`, `blizzard_soulbinds_loads.rs`,
//! `blizzard_landing_soulbinds_loads.rs`). The PLAN.md task explicitly asks to
//! "split out per-addon test plans only after identifying which screens require
//! smoke vs startup coverage" — this file produces that classification and pins
//! it as a runnable contract.
//!
//! COVERAGE CLASSIFICATION (per addon):
//!     - STARTUP coverage: addon is non-LoD and appears in the eager Game sweep.
//!       It loads on every Game session and its body must be clean at sim init.
//!       → Only Blizzard_SharedTalentUI fits this profile.
//!
//!     - SMOKE coverage: addon is `## LoadOnDemand: 1`, never appears in the
//!       eager sweep, and only loads when triggered (UIParentLoadAddOn /
//!       C_AddOns.LoadAddOn / show-panel verbs). Test by manually invoking
//!       load_addon and asserting clean parse + global publication.
//!       → Blizzard_AzeriteUI, Blizzard_AzeriteEssenceUI, Blizzard_AzeriteRespecUI,
//!         Blizzard_Soulbinds, Blizzard_LandingSoulbinds fit this profile.
//!
//!     - NEITHER: addon ships a TOC variant the simulator cannot resolve. Direct
//!       parse of the source bytes is the only available probe; eager + LOD
//!       paths both fail at find_toc_file.
//!       → Blizzard_TalentUI fits this profile (Mists-only TOC; the variant
//!         resolver explicitly excludes `_Mists.toc` filenames).

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

const LANE_ADDONS: &[&str] = &[
    "Blizzard_SharedTalentUI",
    "Blizzard_TalentUI",
    "Blizzard_AzeriteUI",
    "Blizzard_AzeriteEssenceUI",
    "Blizzard_AzeriteRespecUI",
    "Blizzard_Soulbinds",
    "Blizzard_LandingSoulbinds",
];

const STARTUP_LANE_ADDONS: &[&str] = &["Blizzard_SharedTalentUI"];

const SMOKE_LANE_ADDONS: &[&str] = &[
    "Blizzard_AzeriteUI",
    "Blizzard_AzeriteEssenceUI",
    "Blizzard_AzeriteRespecUI",
    "Blizzard_Soulbinds",
    "Blizzard_LandingSoulbinds",
];

fn parse_lane_toc(name: &str) -> TocFile {
    let dir = blizzard_ui_dir().join(name);
    let toc = find_toc_file(&dir).unwrap_or_else(|| {
        panic!("`{name}` TOC must resolve via find_toc_file (use parse_unresolvable_toc instead for Mists-only addons)");
    });
    TocFile::from_file(&toc).unwrap_or_else(|err| panic!("`{name}` TOC parses: {err}"))
}

fn parse_unresolvable_toc(name: &str, explicit_filename: &str) -> TocFile {
    let path = blizzard_ui_dir().join(name).join(explicit_filename);
    TocFile::from_file(&path)
        .unwrap_or_else(|err| panic!("`{name}/{explicit_filename}` parses: {err}"))
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
fn each_lane_addon_directory_exists_on_disk() {
    for name in LANE_ADDONS {
        let dir = blizzard_ui_dir().join(name);
        assert!(
            dir.is_dir(),
            "Lane addon `{name}` directory must exist at \
             `Interface/BlizzardUI/{name}/`. The progression UI lane covers per-character \
             power systems (talents / azerite / soulbinds); missing any directory means the \
             corresponding gameplay system has no UI surface available for SMOKE-load tests"
        );
    }
}

#[test]
fn shared_talent_ui_is_the_only_eager_load_in_the_lane() {
    let shared = parse_lane_toc("Blizzard_SharedTalentUI");
    assert!(
        !shared.is_load_on_demand(),
        "Blizzard_SharedTalentUI carries `## LoadOnDemand: 0` (eager). It is the SHARED utility \
         layer for talent-tree UIs (button templates, edge templates, anim mixins) and must be \
         live before any consumer addon (TalentUI, ClassTalentUI, PlayerSpells) instantiates a \
         talent button. The shared layer publishes virtual templates like \
         SharedTalentButtonTemplate that consumers inherit via XML, so it cannot be lazy-loaded"
    );

    for smoke in SMOKE_LANE_ADDONS {
        let toc = parse_lane_toc(smoke);
        assert!(
            toc.is_load_on_demand(),
            "Lane SMOKE addon `{smoke}` MUST carry `## LoadOnDemand: 1` — the progression UI \
             surfaces are panel-style modals (AzeriteEmpoweredItemUI / AzeriteEssenceUI / \
             AzeriteRespecFrame / SoulbindViewer / LandingPageSoulbindPanelTemplate) that only \
             open in response to a player action. Eager-loading them on every session would \
             waste startup time"
        );
    }
}

#[test]
fn talent_ui_ships_only_a_mists_toc_variant_unresolvable_by_find_toc_file() {
    let talent_ui_dir = blizzard_ui_dir().join("Blizzard_TalentUI");

    let resolved = find_toc_file(&talent_ui_dir);
    assert!(
        resolved.is_none(),
        "SIMULATOR/SOURCE GAP — find_toc_file returns None for Blizzard_TalentUI because the \
         addon ships ONLY `Blizzard_TalentUI_Mists.toc`. The variant resolver at \
         src/loader/mod.rs:65-95 probes (1) `<name>_Mainline.toc`, (2) `<name>.toc`, then \
         (3) any non-Classic/Wrath/TBC/Vanilla/Mists `.toc` file in the directory — and the \
         Mists exclusion at the fallback explicitly skips `_Mists.toc` filenames. So \
         Blizzard_TalentUI is invisible to the standard load path on a Mainline target. \
         Real WoW selects flavor variants based on the running build; the simulator hard-codes \
         Mainline preference. Got: {resolved:?}"
    );

    let mists_toc = talent_ui_dir.join("Blizzard_TalentUI_Mists.toc");
    assert!(
        mists_toc.exists(),
        "Mists-flavor TOC must exist on disk for direct-parse tests, even though the variant \
         resolver skips it on Mainline runs"
    );

    let toc = parse_unresolvable_toc("Blizzard_TalentUI", "Blizzard_TalentUI_Mists.toc");
    assert!(
        toc.is_load_on_demand(),
        "TalentUI's Mists TOC carries `## LoadOnDemand: 1` — even if a future Mists-aware \
         variant resolver lit this addon up, it would only load when the player opens the \
         legacy talent panel"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_HelpPlate".to_string()],
        "TalentUI Mists TOC depends only on Blizzard_HelpPlate — the contextual-tutorial \
         tooltips that float beside the talent grid"
    );
}

#[test]
fn lane_dep_edges_classify_each_addons_load_floor() {
    let shared = parse_lane_toc("Blizzard_SharedTalentUI");
    let azerite_main = parse_lane_toc("Blizzard_AzeriteUI");
    let azerite_essence = parse_lane_toc("Blizzard_AzeriteEssenceUI");
    let azerite_respec = parse_lane_toc("Blizzard_AzeriteRespecUI");
    let soulbinds = parse_lane_toc("Blizzard_Soulbinds");
    let landing_soulbinds = parse_lane_toc("Blizzard_LandingSoulbinds");

    assert_eq!(
        shared.dependencies(),
        vec!["Blizzard_SpellSearch".to_string()],
        "SharedTalentUI depends on Blizzard_SpellSearch — the addon's filter-by-spell-name \
         feature (talent-tree search bar) calls into SpellSearch's index. Without this dep, \
         the search input is dead at file scope"
    );

    assert_eq!(
        azerite_main.dependencies(),
        vec!["Blizzard_TransformTree".to_string()],
        "AzeriteUI depends on Blizzard_TransformTree — the empowered-item visualization uses \
         transform-tree primitives to position the concentric power rings. The dep is unique \
         in this lane (no other progression addon needs TransformTree)"
    );

    assert_eq!(
        azerite_essence.dependencies(),
        vec!["Blizzard_Colors".to_string()],
        "AzeriteEssenceUI depends on Blizzard_Colors — the major/minor essence slots use \
         themed border colors per essence rank. Note that this is the ONE dep declared even \
         though the addon also visually depends on PortraitFrameTemplate (from FrameXMLBase \
         via UIPanelTemplates); the missing edge is implicit because PortraitFrameTemplate \
         is published by an eager addon"
    );

    assert!(
        azerite_respec.dependencies().is_empty(),
        "AzeriteRespecUI declares NO dependencies — the only progression-lane addon with an \
         empty dep list. The respec dialog is structurally a thin wrapper over \
         EtherealFrameTemplate (from FrameXMLBase) and StaticPopup confirmation; both come \
         from eager addons. Got: {:?}",
        azerite_respec.dependencies()
    );

    assert_eq!(
        soulbinds.dependencies(),
        vec![
            "Blizzard_Colors".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "Retail 12.1.0.69497 Soulbinds depends on Blizzard_Colors and Blizzard_GameMenuEsc"
    );

    assert!(
        landing_soulbinds.dependencies().is_empty(),
        "LandingSoulbinds declares NO dependencies — minimal panel for the covenant landing \
         page. All template inheritance points (ResizeLayoutFrame) come from eager foundation \
         lane addons. Got: {:?}",
        landing_soulbinds.dependencies()
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
            "Lane SMOKE addon `{smoke}` MUST NOT appear in eager Game discovery — \
             `## LoadOnDemand: 1` excludes it from the eager sweep at \
             `discover_blizzard_addons_for_screen`. The LOD pool entry only joins the active \
             load list if a non-LOD addon explicitly depends on it (see \
             `pull_required_lod_addons`); none of the eager addons depend on these progression \
             panels at file scope, so they remain unloaded until UIParentLoadAddOn or \
             C_AddOns.LoadAddOn triggers them. If this regresses, eager startup time inflates \
             with panel-shaped XML that won't be needed for most sessions"
        );
    }

    assert!(
        !names.contains(&"Blizzard_TalentUI".to_string()),
        "Blizzard_TalentUI MUST NOT appear in eager Game discovery — its Mists-only TOC is \
         invisible to find_toc_file (see talent_ui_ships_only_a_mists_toc_variant_unresolvable_by_find_toc_file)"
    );
}

#[test]
fn shared_talent_ui_appears_in_eager_game_discovery_as_the_lane_startup_anchor() {
    let ui = blizzard_ui_dir();
    let names: Vec<String> = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
        .into_iter()
        .map(|(n, _)| n)
        .collect();

    for startup in STARTUP_LANE_ADDONS {
        assert!(
            names.contains(&startup.to_string()),
            "Lane STARTUP addon `{startup}` MUST appear in eager Game discovery — it carries \
             `## LoadOnDemand: 0` (or no LoadOnDemand directive at all, which means eager). \
             The shared talent-button / edge / anim layer is the load-floor for every \
             talent-tree-shaped consumer addon (TalentUI, ClassTalentUI from Blizzard_PlayerSpells, \
             hero-talent overlays). If this regresses, every consumer panel fails its \
             template-inherit lookup at parse time"
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
                "Progression lane addon `{lane_addon}` MUST NOT appear in {screen:?} eager \
                 discovery. None of the lane TOCs declare `## AllowLoad: Both` or \
                 `## AllowLoad: Glue` — the implicit default is Game-only. Per-character \
                 power systems are meaningful only after the player has selected a character"
            );
        }
    }
}

prefork_full_ui_case! {
fn shared_talent_ui_is_loaded_after_eager_sweep_for_full_game_ui(env: &WowLuaEnv) {

    for startup in STARTUP_LANE_ADDONS {
        let loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded('{startup}')"))
            .unwrap_or_else(|err| panic!("IsAddOnLoaded({startup}) probe failed: {err}"));
        assert!(
            loaded,
            "C_AddOns.IsAddOnLoaded('{startup}') must return true after the eager Game sweep. \
             SharedTalentUI provides the talent-button utility globals (TalentButtonUtil / \
             TalentButtonAnimUtil / SharedTalentButtonTemplate) that consumer talent UIs \
             reference at file-scope template-inherit time — must be live before any LOD \
             talent panel triggers"
        );
    }
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
            "Lane SMOKE addon `{smoke}` MUST remain unloaded after the eager Game sweep. The \
             classification contract states: SMOKE addons load only when explicitly triggered \
             (UIParentLoadAddOn / C_AddOns.LoadAddOn / panel-show verbs). If this regresses, \
             the eager sweep is mis-pulling LOD addons — likely via a transitive dep edge \
             from a non-LOD addon, which would cascade into longer startup times"
        );
    }
}
}

prefork_full_ui_case! {
fn shared_talent_ui_publishes_talent_button_utility_globals_after_full_load(env: &WowLuaEnv) {

    let utility_globals = [
        ("TalentButtonUtil", "table"),
        ("TalentButtonAnimUtil", "table"),
    ];
    for (global, expected_kind) in utility_globals {
        let actual: String = env
            .eval(&format!("return type({global})"))
            .unwrap_or_else(|err| panic!("type({global}) probe failed: {err}"));
        assert_eq!(
            actual, expected_kind,
            "SharedTalentUI utility global `{global}` must publish as `{expected_kind}` after \
             the eager Game sweep. These are the consumer-facing surfaces that downstream \
             talent UIs inherit from at file scope. If this regresses, every consumer panel's \
             `OnLoad` path crashes on a nil reference. Source: \
             Interface/BlizzardUI/Blizzard_SharedTalentUI/Blizzard_SharedTalentUtil.lua \
             (TalentButtonUtil) and Blizzard_TalentButtonAnimUtil.lua (TalentButtonAnimUtil)"
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
        .filter(|message| LANE_ADDONS.iter().any(|name| message.contains(name)))
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Progression lane MUST load without lane-specific Lua errors during eager Game sweep. \
         Only SharedTalentUI is eagerly loaded; the SMOKE addons are LOD and don't run at \
         this point. Any error here indicates SharedTalentUI's body broke. Got:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn smoke_lane_addons_can_be_loaded_on_demand_after_eager_sweep_completes(env: &WowLuaEnv) {

    for smoke in SMOKE_LANE_ADDONS {
        let toc_path = blizzard_ui_dir().join(smoke);
        let toc = find_toc_file(&toc_path).unwrap_or_else(|| {
            panic!("`{smoke}` TOC must resolve via find_toc_file before LOD-trigger probe");
        });
        let result = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc);
        assert!(
            result.is_ok(),
            "SMOKE addon `{smoke}` must load cleanly when triggered after the eager Game \
             sweep. The eager sweep brings in every dep this addon needs (Colors / \
             TransformTree / FrameXMLBase / etc.); the LOD load itself is just an XML+Lua \
             body sweep with no fresh dep resolution. If this fails, the SMOKE coverage \
             classification is wrong — the addon would need a different harness shape than \
             plain load_addon. Got: {result:?}"
        );

        let loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded('{smoke}')"))
            .unwrap_or_else(|err| panic!("post-LOD IsAddOnLoaded({smoke}) probe failed: {err}"));
        assert!(
            loaded,
            "After explicit load_addon, IsAddOnLoaded('{smoke}') must flip to true. The \
             loader updates the AddonInfo.loaded flag at end-of-load_addon"
        );
    }
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
         Per-addon SMOKE tests for the progression lane should use the same 7-step harness \
         documented in `tests/blizzard_core_frame_lane.rs`: \
         (1) WowLuaEnv::new() (pre-creates UIParent/WorldFrame/panel-attrs); \
         (2) set_screen_size(1024, 768) + set_screen_mode(ScreenKind::Game); \
         (3) state.addon_base_paths = vec![blizzard_ui_dir()]; \
         (4) register_intrinsic_templates(); \
         (5) discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) + per-result load_addon \
             (eager sweep — brings in SharedTalentUI for STARTUP coverage); \
         (6) apply_post_load_workarounds(); \
         (7) fire_startup_events_for_screen(&env, ScreenKind::Game). \
         For SMOKE coverage, append step (8): manually call load_addon with the LOD TOC path"
    );
}
