#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn garrison_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GarrisonUI")
}

fn garrison_ui_toc() -> PathBuf {
    garrison_ui_dir().join("Blizzard_GarrisonUI_Mainline.toc")
}

fn lod_addon_toc(folder: &str) -> PathBuf {
    let dir = blizzard_ui_dir().join(folder);
    find_toc_file(&dir).unwrap_or_else(|| panic!("{folder} TOC must resolve"))
}

fn assert_directory_omits_symbols(directory: &std::path::Path, symbols: &[&str]) {
    for entry in std::fs::read_dir(directory).expect("Blizzard source directory should read") {
        let path = entry.expect("Blizzard source entry should read").path();
        if path.is_dir() {
            assert_directory_omits_symbols(&path, symbols);
            continue;
        }

        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        if !matches!(extension, "lua" | "xml" | "toc") {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("Blizzard source file should read");
        for symbol in symbols {
            assert!(
                !source.contains(symbol),
                "{symbol} unexpectedly exists in {}",
                path.display()
            );
        }
    }
}

fn load_full_game_ui_with_lod_deps() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    let lod_deps = [
        "Blizzard_MapCanvas",
        "Blizzard_SharedMapDataProviders",
        "Blizzard_GarrisonTemplates",
        "Blizzard_AdventureMap",
    ];
    for folder in lod_deps {
        let toc = lod_addon_toc(folder);
        load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|err| panic!("[load {folder}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_garrison_ui_resolves_mainline_toc() {
    let resolved = find_toc_file(&garrison_ui_dir())
        .expect("Blizzard_GarrisonUI directory must contain a discoverable TOC");
    let resolved_name = resolved
        .file_name()
        .expect("resolved TOC must have a filename")
        .to_str()
        .expect("resolved TOC filename must be utf-8");

    assert_eq!(
        resolved_name, "Blizzard_GarrisonUI_Mainline.toc",
        "Blizzard_GarrisonUI ships only a `_Mainline.toc` variant (no bare \
         Blizzard_GarrisonUI.toc); src/loader/mod.rs:65's `find_toc_file` prefers the \
         `_Mainline.toc` suffix and resolves it directly"
    );
}

#[test]
fn blizzard_garrison_ui_toc_declares_lod_required_deps_game_only_mainline() {
    let toc = TocFile::from_file(&garrison_ui_toc()).expect("Blizzard_GarrisonUI TOC parse");

    assert!(
        toc.is_load_on_demand(),
        "Blizzard_GarrisonUI declares `## LoadOnDemand: 1` — the full mission/landing/board \
         UI is loaded on demand by the player opening a garrison/mission window via \
         GarrisonLandingPage_Toggle / ShowGarrisonLandingPage / OrderHall_LoadUI etc., \
         NOT eagerly at startup"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GarrisonUI does not declare `## UseSecureEnvironment` — the mission UI \
         drives only public garrison data + UI state, no protected-action surface"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_GarrisonUI declares no `## SavedVariables` — the addon owns no persistent \
         user state (mission cooldowns + completion are world-server state)"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_GarrisonUI declares `## AllowLoadGameType: mainline` — \
         is_game_type_restricted() must return false because src/toc.rs:299 treats \
         `mainline`/`standard` as the unrestricted retail game type"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_GarrisonBase".to_string(),
            "Blizzard_GarrisonTemplates".to_string(),
            "Blizzard_AdventureMap".to_string(),
            "Blizzard_Colors".to_string(),
            "Blizzard_HelpPlate".to_string(),
            "Blizzard_FrameXMLUtil".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "Retail 12.1.0.69497 declares seven RequiredDep entries in published order. Got: {:?}",
        deps
    );
}

#[test]
fn blizzard_garrison_ui_toc_declares_allow_load_game_and_mainline() {
    let toc_text =
        std::fs::read_to_string(garrison_ui_toc()).expect("Blizzard_GarrisonUI TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: game"),
        "Blizzard_GarrisonUI must declare `## AllowLoad: game` — the mission/landing \
         UI is meaningful only on the in-world Game screen, never on Login/CharacterSelect"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: mainline"),
        "Blizzard_GarrisonUI must declare `## AllowLoadGameType: mainline` — the \
         garrison/order-hall/covenant systems are retail-only, classic flavors lack \
         the data tables this addon depends on"
    );
}

#[test]
fn blizzard_garrison_ui_toc_lists_thirty_two_mainline_files_plus_localization() {
    let toc_text =
        std::fs::read_to_string(garrison_ui_toc()).expect("Blizzard_GarrisonUI TOC should read");
    let mainline_count = toc_text.matches("Mainline\\").count();
    assert_eq!(
        mainline_count, 32,
        "Retail 12.1.0.69497 enumerates 32 `Mainline\\\\*` entries: the bootstrap Lua, \
         15 Lua/XML pairs, and Localization.lua. Got: {mainline_count}"
    );
    assert!(
        toc_text.contains("Mainline\\Blizzard_GarrisonUI_Bootstrap.lua [Bootstrap]"),
        "Retail 12.1.0.69497 loads Blizzard_GarrisonUI_Bootstrap.lua before the main UI files"
    );
    assert!(
        toc_text.contains("Mainline\\Localization.lua"),
        "TOC must list Mainline\\Localization.lua — the per-locale l10nTable holds \
         localize() overrides for mission/follower display strings"
    );
    assert!(
        toc_text.contains("Mainline\\Blizzard_GarrisonLandingPage.lua")
            && toc_text.contains("Mainline\\Blizzard_GarrisonLandingPage.xml"),
        "TOC must list both LandingPage .lua + .xml — the entry point that publishes \
         GarrisonLandingPage and GarrisonLandingPageMixin"
    );
    assert!(
        toc_text.contains("Mainline\\Blizzard_CovenantMissionUI.lua")
            && toc_text.contains("Mainline\\Blizzard_CovenantMissionUI.xml"),
        "TOC must list both CovenantMissionUI .lua + .xml — the 9.0 adventure-board \
         entry point that publishes CovenantMissionFrame and CovenantMission mixin"
    );
}

#[test]
fn blizzard_garrison_ui_excluded_from_game_auto_discovery_due_to_lod() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GarrisonUI");
    assert!(
        !in_game,
        "Blizzard_GarrisonUI declares `## LoadOnDemand: 1` — it must NOT appear in \
         Game-screen auto-discovery (callers like the Garrison MicroMenu button or the \
         GarrisonLandingPage_Toggle entry invoke `LoadAddOn(\"Blizzard_GarrisonUI\")` \
         lazily when the player opens a garrison UI)"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GarrisonUI");
    assert!(
        !in_login,
        "Blizzard_GarrisonUI declares `## AllowLoad: game` — combined with LoadOnDemand \
         it must also be absent from Login auto-discovery"
    );
}

/// Permanent 3D-rendering gaps documented in CLAUDE.md: Model / ModelScene /
/// PlayerModel / DressUpModel are stub-only because the simulator renders 2D
/// UI exclusively. Errors caused by mission UI driving these APIs (the WoD
/// follower/shipyard model panels, the Shadowlands adventure-board ModelScene,
/// the BFA/Order Hall mission-complete model viewport) are accepted gaps and
/// must NOT cause this test to fail.
fn is_three_d_model_gap(message: &str) -> bool {
    message.contains("SetTargetDistance")
        || message.contains("SetFacingLeft")
        || message.contains("SetFacingRight")
        || message.contains("ModelScene")
        || message.contains("expected number, got nil at argument 1")
}

#[test]
fn blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors() {
    let env = load_full_game_ui_with_lod_deps();

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &garrison_ui_toc())
        .expect("Blizzard_GarrisonUI should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let garrison_ui_errors: Vec<String> = load_errors
        .iter()
        .filter(|message| {
            let mentions_garrison = message.contains("Garrison")
                || message.contains("OrderHall")
                || message.contains("BFA")
                || message.contains("Covenant")
                || message.contains("Adventures");
            mentions_garrison && !is_three_d_model_gap(message)
        })
        .cloned()
        .collect();
    assert!(
        garrison_ui_errors.is_empty(),
        "Blizzard_GarrisonUI emitted non-3D-model Lua errors during explicit load:\n  {}",
        garrison_ui_errors.join("\n  ")
    );
}

#[test]
fn blizzard_garrison_ui_is_addon_loaded_returns_true_after_explicit_load() {
    let env = load_full_game_ui_with_lod_deps();

    let before: bool = env
        .eval("return C_AddOns and C_AddOns.IsAddOnLoaded('Blizzard_GarrisonUI') or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        !before,
        "IsAddOnLoaded should return false BEFORE explicit LoadAddOn — LoadOnDemand \
         keeps Blizzard_GarrisonUI out of auto-discovery"
    );

    load_addon(&env.loader_env(), &garrison_ui_toc())
        .expect("Blizzard_GarrisonUI should load via Rust loader");

    let after: bool = env
        .eval("return C_AddOns and C_AddOns.IsAddOnLoaded('Blizzard_GarrisonUI') or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        after,
        "C_AddOns.IsAddOnLoaded('Blizzard_GarrisonUI') must return true AFTER explicit \
         LoadAddOn (the loader must register the addon's name + state in the addon-info \
         table that backs IsAddOnLoaded)"
    );
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_garrison_ui_does_not_publish_snapshot_only_hide_wrappers() {
    let symbols = ["HideGarrisonMissionFrames", "HideGarrisonShipyardFrame"];
    assert_directory_omits_symbols(&garrison_ui_dir(), &symbols);

    let env = load_full_game_ui_with_lod_deps();
    load_addon(&env.loader_env(), &garrison_ui_toc()).expect("Blizzard_GarrisonUI should load");

    let wrappers_are_absent: (bool, bool) = env
        .eval("return HideGarrisonMissionFrames == nil, HideGarrisonShipyardFrame == nil")
        .expect("garrison hide wrapper visibility should be queryable");
    assert_eq!(wrappers_are_absent, (true, true));
}

#[test]
fn blizzard_garrison_ui_publishes_mission_frame_globals() {
    let env = load_full_game_ui_with_lod_deps();
    load_addon(&env.loader_env(), &garrison_ui_toc()).expect("Blizzard_GarrisonUI should load");

    let mission_frames: (String, String, String, String, String) = env
        .eval(
            "return type(GarrisonMissionFrame), \
                    type(OrderHallMissionFrame), \
                    type(BFAMissionFrame), \
                    type(CovenantMissionFrame), \
                    type(GarrisonShipyardFrame)",
        )
        .expect("Mission frame probe should succeed");
    assert_eq!(
        mission_frames,
        (
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
        ),
        "Blizzard_GarrisonUI publishes 5 top-level mission-frame globals: \
         GarrisonMissionFrame (WoD garrison-mission UI), OrderHallMissionFrame (Legion \
         class-order-hall mission UI), BFAMissionFrame (BfA War Campaign mission UI), \
         CovenantMissionFrame (Shadowlands adventure-board UI inheriting CallbackRegistryMixin), \
         GarrisonShipyardFrame (WoD 6.2 Shipyard mission UI)"
    );
}

#[test]
fn blizzard_garrison_ui_publishes_landing_page_and_support_frames() {
    let env = load_full_game_ui_with_lod_deps();
    load_addon(&env.loader_env(), &garrison_ui_toc()).expect("Blizzard_GarrisonUI should load");

    let frames: (String, String, String, String, String) = env
        .eval(
            "return type(GarrisonLandingPage), \
                    type(GarrisonBuildingFrame), \
                    type(GarrisonMonumentFrame), \
                    type(GarrisonCapacitiveDisplayFrame), \
                    type(GarrisonRecruiterFrame)",
        )
        .expect("Landing/support frame probe should succeed");
    assert_eq!(
        frames,
        (
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
        ),
        "Blizzard_GarrisonUI publishes 5 support frames: GarrisonLandingPage (the report \
         summary frame opened from the MicroMenu Garrison button), GarrisonBuildingFrame \
         (WoD garrison building/plot UI), GarrisonMonumentFrame (WoD trophy display, \
         frameStrata=HIGH), GarrisonCapacitiveDisplayFrame (work-order shipment terminal), \
         GarrisonRecruiterFrame (WoD follower-recruit selection popup)"
    );
}

#[test]
fn blizzard_garrison_ui_publishes_mission_mixins() {
    let env = load_full_game_ui_with_lod_deps();
    load_addon(&env.loader_env(), &garrison_ui_toc()).expect("Blizzard_GarrisonUI should load");

    let mixins: (String, String, String, String, String) = env
        .eval(
            "return type(GarrisonFollowerMission), \
                    type(OrderHallMission), \
                    type(CovenantMission), \
                    type(GarrisonShipyardMission), \
                    type(GarrisonLandingPageMixin)",
        )
        .expect("Mission mixin probe should succeed");
    assert_eq!(
        mixins,
        (
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
        ),
        "Blizzard_GarrisonUI publishes 5 canonical mission/landing mixins as global \
         tables: GarrisonFollowerMission (the WoD garrison + general follower-mission \
         layout mixin chained into all derived frame mixins), OrderHallMission (Legion \
         class-order-hall mission mixin overriding tabs + follower-list visibility), \
         CovenantMission (Shadowlands adventure-board mixin built via \
         CreateFromMixins(CallbackRegistryMixin)), GarrisonShipyardMission (Shipyard \
         mission mixin), GarrisonLandingPageMixin (the report-summary tab/section layout \
         mixin attached to GarrisonLandingPage via mixin=... XML attribute)"
    );
}

#[test]
fn blizzard_garrison_ui_publishes_adventures_board_mixins() {
    let env = load_full_game_ui_with_lod_deps();
    load_addon(&env.loader_env(), &garrison_ui_toc()).expect("Blizzard_GarrisonUI should load");

    let board_mixins: (String, String, String, String, String) = env
        .eval(
            "return type(AdventuresBoardMixin), \
                    type(AdventuresBoardCombatMixin), \
                    type(AdventuresSocketMixin), \
                    type(AdventuresBoardAuraIconMixin), \
                    type(AdventuresBoardAuraContainerMixin)",
        )
        .expect("Adventures board mixin probe should succeed");
    assert_eq!(
        board_mixins,
        (
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
        ),
        "Blizzard_AdventuresBoard.lua publishes 5 mixin tables driving the 9.0 \
         adventure-board layout: AdventuresBoardMixin (base board layout), \
         AdventuresBoardCombatMixin (CreateFromMixins(AdventuresBoardMixin) — combat \
         overlay during mission execution), AdventuresSocketMixin (per-socket follower \
         placement), AdventuresBoardAuraIconMixin (per-aura icon on a follower puck), \
         AdventuresBoardAuraContainerMixin (the aura-icon row container)"
    );
}

#[test]
fn blizzard_garrison_ui_publishes_landing_page_helpers() {
    let env = load_full_game_ui_with_lod_deps();
    load_addon(&env.loader_env(), &garrison_ui_toc()).expect("Blizzard_GarrisonUI should load");

    let helpers: (bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GarrisonLandingPageTab_OnClick) == 'function', \
                    type(GarrisonLandingPageReport_OnLoad) == 'function', \
                    type(GarrisonLandingPageReport_OnShow) == 'function', \
                    type(GarrisonLandingPageReport_OnEvent) == 'function', \
                    type(GarrisonLandingPageReportShipment_OnEnter) == 'function'",
        )
        .expect("Landing page helper probe should succeed");
    assert_eq!(
        helpers,
        (true, true, true, true, true),
        "Blizzard_GarrisonLandingPage.lua publishes 5 canonical helper functions wired \
         from Blizzard_GarrisonLandingPage.xml Scripts blocks: \
         GarrisonLandingPageTab_OnClick (top of report-tab strip), \
         GarrisonLandingPageReport_OnLoad / OnShow / OnEvent (the report-tab lifecycle \
         drivers), GarrisonLandingPageReportShipment_OnEnter (per-shipment-row tooltip \
         entry point)"
    );
}

#[test]
fn blizzard_garrison_ui_publishes_building_and_recruiter_helpers() {
    let env = load_full_game_ui_with_lod_deps();
    load_addon(&env.loader_env(), &garrison_ui_toc()).expect("Blizzard_GarrisonUI should load");

    let helpers: (bool, bool, bool, bool) = env
        .eval(
            "return type(GarrisonBuildingUI_ToggleFrame) == 'function', \
                    type(GarrisonBuildingFrame_OnLoad) == 'function', \
                    type(GarrisonRecruiterFrame_OnLoad) == 'function', \
                    type(GarrisonRecruiterFrame_HireRecruit) == 'function'",
        )
        .expect("Building/recruiter helper probe should succeed");
    assert_eq!(
        helpers,
        (true, true, true, true),
        "Blizzard_GarrisonBuildingUI.lua publishes GarrisonBuildingUI_ToggleFrame (the \
         canonical show/hide entry point invoked by the WoD garrison architect) + \
         GarrisonBuildingFrame_OnLoad (XML lifecycle wired to the building UI root); \
         Blizzard_GarrisonRecruiterUI.lua publishes GarrisonRecruiterFrame_OnLoad + \
         GarrisonRecruiterFrame_HireRecruit (the recruit-confirm action that calls \
         C_Garrison.RecruitFollower)"
    );
}

#[test]
fn blizzard_garrison_ui_landing_page_parents_uiparent_and_is_hidden_by_default() {
    let env = load_full_game_ui_with_lod_deps();
    load_addon(&env.loader_env(), &garrison_ui_toc()).expect("Blizzard_GarrisonUI should load");

    let landing: (String, bool) = env
        .eval(
            "return GarrisonLandingPage:GetParent():GetName(), \
                    GarrisonLandingPage:IsShown() == false",
        )
        .expect("Landing page probe should succeed");
    assert_eq!(
        landing,
        ("UIParent".to_string(), true),
        "GarrisonLandingPage declares `parent=\"UIParent\" toplevel=\"true\" \
         hidden=\"true\"` in Blizzard_GarrisonLandingPage.xml — the report-summary \
         frame attaches to UIParent so it floats above the world frame, and starts \
         hidden until the player clicks the Garrison MicroMenu button"
    );
}

#[test]
fn blizzard_garrison_ui_monument_frame_uses_high_strata_on_uiparent() {
    let env = load_full_game_ui_with_lod_deps();
    load_addon(&env.loader_env(), &garrison_ui_toc()).expect("Blizzard_GarrisonUI should load");

    let monument: (String, String) = env
        .eval(
            "return GarrisonMonumentFrame:GetFrameStrata(), \
                    GarrisonMonumentFrame:GetParent():GetName()",
        )
        .expect("Monument frame probe should succeed");
    assert_eq!(
        monument,
        ("HIGH".to_string(), "UIParent".to_string()),
        "GarrisonMonumentFrame declares `parent=\"UIParent\" frameStrata=\"HIGH\"` in \
         Blizzard_GarrisonMonumentUI.xml — the trophy-display popup rides above the \
         standard mission-UI strata so it overlays the open garrison interior"
    );
}
