#![cfg(feature = "client-retail")]
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

fn group_finder_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GroupFinder")
}

fn group_finder_mainline_toc() -> PathBuf {
    group_finder_dir().join("Blizzard_GroupFinder_Mainline.toc")
}

fn group_finder_tbc_toc() -> PathBuf {
    group_finder_dir().join("Blizzard_GroupFinder_TBC.toc")
}

fn group_finder_vanilla_toc() -> PathBuf {
    group_finder_dir().join("Blizzard_GroupFinder_Vanilla.toc")
}

fn load_full_game_ui() -> WowLuaEnv {
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

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_group_finder_find_toc_resolves_mainline_variant() {
    let resolved =
        find_toc_file(&group_finder_dir()).expect("Blizzard_GroupFinder TOC should resolve");
    assert_eq!(
        resolved,
        group_finder_mainline_toc(),
        "Blizzard_GroupFinder ships three flavor variants — `_Mainline.toc`, `_TBC.toc`, \
         `_Vanilla.toc`. `find_toc_file` (src/loader/mod.rs:65) prefers the `_Mainline.toc` \
         variant on retail, so the simulator picks the Mainline TOC when it asks for the \
         addon's primary descriptor"
    );
}

#[test]
fn blizzard_group_finder_mainline_toc_declares_six_deps_and_game_screen_gating() {
    let toc = TocFile::from_file(&group_finder_mainline_toc()).expect("Mainline TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_GroupFinder Mainline omits `## LoadOnDemand` — the PVEFrame / GroupFinder \
         tab UI is part of the core Game-screen surface and auto-loads on the Game-screen \
         discovery pass"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_GroupFinder does not declare `## LoadFirst: 1` — the addon consumes \
         templates published by Blizzard_EditMode / Blizzard_GameTooltip / \
         Blizzard_UIPanelTemplates and must load AFTER them, never before"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GroupFinder does not declare `## UseSecureEnvironment` — runs in the \
         standard Lua environment"
    );

    let deps: Vec<String> = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_EditMode".to_string(),
            "Blizzard_GameTooltip".to_string(),
            "Blizzard_UIPanelTemplates".to_string(),
            "Blizzard_RaidWarning".to_string(),
            "Blizzard_LFGUtil".to_string(),
            "Blizzard_FrameXMLUtil".to_string(),
        ],
        "Blizzard_GroupFinder Mainline declares six `## Dependencies:` in published order: \
         Blizzard_EditMode, Blizzard_GameTooltip, Blizzard_UIPanelTemplates, \
         Blizzard_RaidWarning, Blizzard_LFGUtil, and Blizzard_FrameXMLUtil. Got: {deps:?}"
    );

    let saved_vars: Vec<String> = toc.saved_variables();
    assert_eq!(
        saved_vars,
        vec!["PLUNDERSTORM_QUEUE_FROM_MAINLINE_TUTORIAL_STATE".to_string()],
        "Blizzard_GroupFinder declares exactly one `## SavedVariables:` global — \
         PLUNDERSTORM_QUEUE_FROM_MAINLINE_TUTORIAL_STATE — used by PVEFrame.lua to track \
         which Plunderstorm tutorial steps the player has acknowledged across sessions"
    );
}

#[test]
fn blizzard_group_finder_mainline_toc_declares_game_screen_mainline_only() {
    let toc_text =
        std::fs::read_to_string(group_finder_mainline_toc()).expect("Mainline TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: game"),
        "Blizzard_GroupFinder Mainline declares `## AllowLoad: game` — Game-screen-only. \
         `allows_screen()` (src/toc.rs:305) lowercases before matching, so `game` routes to \
         the Game-screen branch and the addon is excluded from glue-screen auto-discovery"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: mainline"),
        "Blizzard_GroupFinder Mainline declares `## AllowLoadGameType: mainline` — retail \
         only. Classic flavors load the TBC / Vanilla TOC variants instead"
    );
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_GroupFinder Mainline declares `## DefaultState: enabled` — the LFG / PVE \
         finder UI must always be active on the Game screen"
    );
}

#[test]
fn blizzard_group_finder_mainline_toc_lists_expected_top_level_files() {
    let toc = TocFile::from_file(&group_finder_mainline_toc()).expect("Mainline TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Mainline/QueueUpdater.lua".to_string(),
            "Mainline/PVEFrame.lua".to_string(),
            "Mainline/PVEFrame.xml".to_string(),
            "Shared/LFGFrame.lua".to_string(),
            "Mainline/LFGFrame.xml".to_string(),
            "Mainline/LFDFrame.lua".to_string(),
            "Mainline/LFDFrame.xml".to_string(),
            "Shared/LFGReadyCheck.lua".to_string(),
            "Shared/LFGReadyCheck.xml".to_string(),
            "Shared/RaidFinder.lua".to_string(),
            "Shared/RaidFinder.xml".to_string(),
            "Mainline/LFGList.lua".to_string(),
            "Mainline/LFGList.xml".to_string(),
            "Mainline/PvpPopup.lua".to_string(),
            "Mainline/PvpPopup.xml".to_string(),
            "Mainline/PVPHelper.lua".to_string(),
            "Mainline/PVPHelper.xml".to_string(),
            "Shared/ScenarioFinder_Shared.lua".to_string(),
            "Mainline/ScenarioFinder.lua".to_string(),
            "Shared/ScenarioFinder.xml".to_string(),
        ],
        "Blizzard_GroupFinder Mainline TOC enumerates 20 source files in this exact order: \
         QueueUpdater first (publishes the LFG queue-update timer the rest of the addon \
         consumes), then PVEFrame Lua+XML (the umbrella PVEFrameMixin frame that owns all \
         finder tabs), then the LFGFrame / LFDFrame / LFGReadyCheck / RaidFinder / LFGList / \
         PvpPopup / PVPHelper / ScenarioFinder modules. Shared/ files (LFGFrame, \
         LFGReadyCheck, RaidFinder, ScenarioFinder_Shared) are reused across Mainline + \
         Mists; Mainline/ files override or extend with retail-only behavior"
    );
}

#[test]
fn blizzard_group_finder_directory_ships_three_tocs_plus_three_subdirectories() {
    let dir = group_finder_dir();
    let mut entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_GroupFinder directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_GroupFinder_Mainline.toc".to_string(),
            "Blizzard_GroupFinder_TBC.toc".to_string(),
            "Blizzard_GroupFinder_Vanilla.toc".to_string(),
            "Mainline".to_string(),
            "Mists".to_string(),
            "Shared".to_string(),
        ],
        "Blizzard_GroupFinder directory ships exactly 6 entries: 3 flavor TOCs (Mainline / \
         TBC / Vanilla) + 3 source subdirectories (Mainline / Mists / Shared). The Classic \
         subdirectory is named Mists (not Classic) — Mists of Pandaria classic — and the TBC \
         / Vanilla TOCs reference Classic\\\\... paths that resolve into the Mists directory"
    );
}

#[test]
fn blizzard_group_finder_tbc_toc_declares_three_deps_and_tbc_game_type() {
    let toc = TocFile::from_file(&group_finder_tbc_toc()).expect("TBC TOC should parse");
    assert!(
        toc.is_game_type_restricted(),
        "Blizzard_GroupFinder TBC declares `## AllowLoadGameType: tbc` — `tbc` is not \
         `mainline` or `standard`, so `is_game_type_restricted()` (src/toc.rs:294) returns \
         true. The retail simulator filters this TOC out of auto-discovery"
    );
    let toc_text = std::fs::read_to_string(group_finder_tbc_toc()).expect("TBC TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: tbc"),
        "TBC TOC raw text must contain `## AllowLoadGameType: tbc`"
    );
    assert!(
        toc_text.contains("## AllowLoad: game"),
        "TBC TOC raw text must contain `## AllowLoad: game` — same Game-screen gating as \
         Mainline, only the game-type filter differs"
    );

    let deps: Vec<String> = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_GameTooltip".to_string(),
            "Blizzard_UIPanelTemplates".to_string(),
            "Blizzard_LFGUtil".to_string(),
        ],
        "Blizzard_GroupFinder TBC declares GameTooltip, UIPanelTemplates, and LFGUtil; \
         the Mainline-only Blizzard_EditMode dependency remains absent on TBC"
    );
}

#[test]
fn blizzard_group_finder_vanilla_toc_declares_vanilla_game_type() {
    let toc = TocFile::from_file(&group_finder_vanilla_toc()).expect("Vanilla TOC should parse");
    assert!(
        toc.is_game_type_restricted(),
        "Blizzard_GroupFinder Vanilla declares `## AllowLoadGameType: vanilla` — also \
         filtered out of the retail simulator's auto-discovery"
    );
    let toc_text =
        std::fs::read_to_string(group_finder_vanilla_toc()).expect("Vanilla TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: vanilla"),
        "Vanilla TOC raw text must contain `## AllowLoadGameType: vanilla`"
    );
}

#[test]
fn blizzard_group_finder_appears_exactly_once_in_game_screen_discovery() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let occurrences: Vec<&PathBuf> = addons
        .iter()
        .filter(|(name, _)| name == "Blizzard_GroupFinder")
        .map(|(_, p)| p)
        .collect();
    assert_eq!(
        occurrences.len(),
        1,
        "Blizzard_GroupFinder must appear EXACTLY ONCE in Game-screen auto-discovery — the \
         TBC and Vanilla TOCs are filtered out by `is_game_type_restricted()` so only the \
         Mainline variant is picked up. Discovered {} occurrences",
        occurrences.len()
    );
    assert_eq!(
        occurrences[0],
        &group_finder_mainline_toc(),
        "The single discovered Blizzard_GroupFinder TOC must be the `_Mainline.toc` variant"
    );
}

#[test]
fn blizzard_group_finder_excluded_from_all_glue_screen_discovery_passes() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_GroupFinder");
        assert!(
            !discovered,
            "Blizzard_GroupFinder MUST NOT appear in {screen:?} auto-discovery — \
             `## AllowLoad: game` is Game-screen only. The LFG / PVE finder UI is \
             irrelevant on glue screens"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_group_finder_loads_via_full_game_ui_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_GroupFinder/")
                || e.contains("Blizzard_GroupFinder\\")
                || e.contains("PVEFrameMixin")
                || e.contains("GroupFinderFrame_")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_GroupFinder emitted addon-specific Lua errors during Game-screen load:\n  \
         {}",
        related
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_group_finder_publishes_group_finder_frame_lifecycle_helpers(env: &WowLuaEnv) {

    for helper in [
        "GroupFinderFrame_OnLoad",
        "GroupFinderFrame_OnEvent",
        "GroupFinderFrame_OnShow",
        "GroupFinderFrame_GetSelection",
        "GroupFinderFrame_GetSelectedIndex",
        "GroupFinderFrame_Update",
        "GroupFinderFrame_EvaluateButtonVisibility",
        "GroupFinderFrame_EvaluateHelpTips",
        "GroupFinderFrame_ShowGroupFrame",
        "GroupFinderFrame_SelectGroupButton",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{helper}']) == 'function'"))
            .expect("helper existence query should succeed");
        assert!(
            exists,
            "After Game-screen load, `{helper}` should be published as a `_G` function — \
             Mainline/PVEFrame.lua publishes 10 GroupFinderFrame_* lifecycle and helper \
             globals consumed by the GroupFinderFrame XML's `<Scripts><OnLoad function=\
             \"...\"/>` bindings and the inter-tab routing in PVEFrame.lua"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_group_finder_publishes_pve_frame_helpers(env: &WowLuaEnv) {

    for helper in [
        "PVEFrame_ToggleFrame",
        "PVEFrame_ShowFrame",
        "PVEFrame_TabOnClick",
        "PVEFrame_HideLeftInset",
        "PVEFrame_ShowLeftInset",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{helper}']) == 'function'"))
            .expect("helper existence query should succeed");
        assert!(
            exists,
            "After Game-screen load, `{helper}` should be published as a `_G` function — \
             Mainline/PVEFrame.lua publishes 5 PVEFrame_* helpers consumed by the \
             PVEFrameTab XML script bindings and the cross-addon /lfd / /raidbrowser slash \
             handlers that toggle the PVEFrame surface"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_group_finder_publishes_pve_frame_mixin_with_lifecycle_methods(env: &WowLuaEnv) {

    let mixin_exists: bool = env
        .eval("return type(_G['PVEFrameMixin']) == 'table'")
        .expect("mixin existence query should succeed");
    assert!(
        mixin_exists,
        "PVEFrameMixin should publish as a `_G` table — Mainline/PVEFrame.lua declares the \
         mixin with OnLoad / OnShow / OnHide / OnEvent / ScenariosEnabled methods bound by \
         the XML `<Frame name=\"PVEFrame\" mixin=\"PVEFrameMixin\">` declaration"
    );

    for method in ["OnLoad", "OnShow", "OnHide", "OnEvent", "ScenariosEnabled"] {
        let method_exists: bool = env
            .eval(&format!(
                "return type(_G['PVEFrameMixin']['{method}']) == 'function'"
            ))
            .expect("mixin method query should succeed");
        assert!(
            method_exists,
            "PVEFrameMixin.{method} should be a function — owned by Mainline/PVEFrame.lua"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_group_finder_publishes_pve_frame_global(env: &WowLuaEnv) {

    let exists: bool = env
        .eval(
            "local f = _G['PVEFrame']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("PVEFrame existence query should succeed");
    assert!(
        exists,
        "After Game-screen load, `PVEFrame` should publish as a global frame instance — \
         Mainline/PVEFrame.xml declares `<Frame name=\"PVEFrame\" mixin=\"PVEFrameMixin\" \
         inherits=\"PortraitFrameTemplate\" parent=\"UIParent\" hidden=\"true\" \
         toplevel=\"true\">` so the named non-virtual frame materializes as a runtime frame"
    );
}
}

prefork_full_ui_case! {
fn blizzard_group_finder_publishes_lfg_list_authenticator_messaging_mixin(env: &WowLuaEnv) {

    let mixin_exists: bool = env
        .eval("return type(_G['LFGAuthenticatorMessagingMixin']) == 'table'")
        .expect("mixin existence query should succeed");
    assert!(
        mixin_exists,
        "LFGAuthenticatorMessagingMixin should publish as a `_G` table — Mainline/\
         LFGList.lua declares the base mixin from which LFGEditBoxMixin / \
         LFGListLockButtonMixin / LFGListCreateGroupDisabledStateButtonMixin all derive \
         via CreateFromMixins. This is the foundation of the LFG-list authenticator-gated \
         input surfaces"
    );
}
}
