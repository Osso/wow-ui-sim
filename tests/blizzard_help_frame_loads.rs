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

fn help_frame_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HelpFrame")
}

fn help_frame_toc() -> PathBuf {
    help_frame_dir().join("Blizzard_HelpFrame.toc")
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
fn blizzard_help_frame_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&help_frame_dir()).expect("Blizzard_HelpFrame TOC should resolve");
    assert_eq!(
        resolved,
        help_frame_toc(),
        "Blizzard_HelpFrame ships exactly one bare TOC (`Blizzard_HelpFrame.toc`) — no flavor \
         variants. The help/ticket UI is shared across mainline and classic flavors so a single \
         TOC suffices and `find_toc_file` (src/loader/mod.rs:65) falls through to the bare \
         `.toc` suffix"
    );
}

#[test]
fn blizzard_help_frame_toc_declares_non_lod_with_four_dependencies() {
    let toc = TocFile::from_file(&help_frame_toc()).expect("HelpFrame TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_HelpFrame omits `## LoadOnDemand` — the F1 / customer-support help surface \
         must be available throughout the Game-screen session and auto-loads on the Game-screen \
         discovery pass"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_HelpFrame does not declare `## LoadFirst: 1` — it depends on \
         Blizzard_ActionBar / Blizzard_StaticPopup_Game / Blizzard_UIParent and must load \
         AFTER all three"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_HelpFrame does not declare `## UseSecureEnvironment` — runs in the standard \
         Lua environment"
    );
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_MicroMenu".to_string(),
            "Blizzard_ActionBar".to_string(),
            "Blizzard_StaticPopup_Game".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "Blizzard_HelpFrame declares four `## Dependencies:` in published order: \
         Blizzard_MicroMenu, Blizzard_ActionBar, Blizzard_StaticPopup_Game, and \
         Blizzard_GameMenuEsc."
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_HelpFrame declares NO `## SavedVariables*` — ticket state, GM responses, and \
         survey acknowledgement are server-authoritative (queried via GetWebTicket / \
         AcknowledgeSurvey), no per-installation persistence is needed"
    );
}

#[test]
fn blizzard_help_frame_toc_declares_default_state_enabled_and_game_screen() {
    let toc = TocFile::from_file(&help_frame_toc()).expect("HelpFrame TOC should parse");
    let toc_text = std::fs::read_to_string(help_frame_toc()).expect("HelpFrame TOC should read");
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_HelpFrame declares `## DefaultState: enabled` — the help surface is \
         unconditionally enabled, never an opt-in addon"
    );
    assert!(
        toc_text.contains("## AllowLoad: Game"),
        "Blizzard_HelpFrame declares `## AllowLoad: Game` (capital G — `allows_screen()` \
         (src/toc.rs:305) lowercases before matching). The customer-support ticket / browser \
         surface is unreachable from glue screens, which have their own GlueXML help paths"
    );
    assert!(
        !toc_text.contains("## AllowLoadGameType:"),
        "Blizzard_HelpFrame omits `## AllowLoadGameType` — the help/ticket UI is shared across \
         mainline (retail) and classic flavors and is not game-type-restricted"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_HelpFrame must NOT be game-type restricted — absent AllowLoadGameType means \
         all flavors qualify"
    );
}

#[test]
fn blizzard_help_frame_toc_lists_lua_then_xml() {
    let toc = TocFile::from_file(&help_frame_toc()).expect("HelpFrame TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec!["HelpFrame.lua".to_string(), "HelpFrame.xml".to_string()],
        "Blizzard_HelpFrame TOC body lists exactly 2 files in order: HelpFrame.lua first \
         (publishes HelpFrameMixin + StaticPopupDialogs['EXTERNAL_LINK'] + 8 free helper \
         functions referenced by XML's `<Scripts><OnLoad function=\"...\"/>` bindings), then \
         HelpFrame.xml. Note the unprefixed file names — TOC body uses bare `HelpFrame.lua` / \
         `HelpFrame.xml` rather than the `Blizzard_HelpFrame.lua` / `.xml` pattern that most \
         other addons follow"
    );
}

#[test]
fn blizzard_help_frame_directory_ships_three_entries() {
    let dir = help_frame_dir();
    let mut entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_HelpFrame directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_HelpFrame.toc".to_string(),
            "HelpFrame.lua".to_string(),
            "HelpFrame.xml".to_string(),
        ],
        "Blizzard_HelpFrame directory ships exactly 3 entries (TOC + Lua + XML), no flavor \
         subdirectory and no Localization.lua — strings are pulled from the global locale \
         table (HELP_FRAME_TITLE, HELPFRAME_TICKET_CLICK_HELP, BROWSER_*, etc.)"
    );
}

#[test]
fn blizzard_help_frame_appears_exactly_once_in_game_screen_auto_discovery() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let count = addons
        .iter()
        .filter(|(name, _)| name == "Blizzard_HelpFrame")
        .count();
    assert_eq!(
        count, 1,
        "Blizzard_HelpFrame must auto-discover EXACTLY ONCE on the Game screen — non-LoD + \
         `## AllowLoad: Game` qualify it for the Game-screen discovery pass, and the single \
         bare TOC means there's no flavor-variant duplication"
    );
}

#[test]
fn blizzard_help_frame_excluded_from_all_glue_screen_auto_discovery_passes() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons.iter().any(|(name, _)| name == "Blizzard_HelpFrame");
        assert!(
            !discovered,
            "Blizzard_HelpFrame MUST NOT appear in {screen:?} auto-discovery — \
             `## AllowLoad: Game` is Game-screen only and the help/ticket NPC interaction is \
             unreachable from any glue screen"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_help_frame_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_HelpFrame/")
                || e.contains("Blizzard_HelpFrame\\")
                || e.contains("HelpFrame.lua")
                || e.contains("HelpFrame.xml")
                || e.contains("HelpFrameMixin")
                || e.contains("HelpOpenWebTicketButton")
                || e.contains("TicketStatusFrame")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_HelpFrame emitted addon-specific Lua errors during Game-screen auto-load:\n  \
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
fn blizzard_help_frame_is_addon_loaded_returns_true_after_game_screen_load(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HelpFrame')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "After Game-screen auto-discovery, \
         `C_AddOns.IsAddOnLoaded('Blizzard_HelpFrame')` should return true"
    );
}
}

prefork_full_ui_case! {
fn blizzard_help_frame_publishes_three_module_constants(env: &WowLuaEnv) {

    let interval: f64 = env
        .eval("return GMTICKET_CHECK_INTERVAL")
        .expect("GMTICKET_CHECK_INTERVAL lookup should succeed");
    assert_eq!(
        interval, 600.0,
        "GMTICKET_CHECK_INTERVAL should be 600 (10 minutes in seconds) — HelpFrame.lua line 21 \
         declares `GMTICKET_CHECK_INTERVAL = 600`, used by HelpOpenWebTicketButton_OnUpdate as \
         the polling interval to call GetWebTicket()"
    );

    let kb: f64 = env
        .eval("return HELPFRAME_KNOWLEDGE_BASE")
        .expect("HELPFRAME_KNOWLEDGE_BASE lookup should succeed");
    assert_eq!(
        kb, 1.0,
        "HELPFRAME_KNOWLEDGE_BASE should be 1 — HelpFrame.lua line 23 declares the \
         knowledge-base ShowFrame key constant"
    );

    let submit: f64 = env
        .eval("return HELPFRAME_SUBMIT_TICKET")
        .expect("HELPFRAME_SUBMIT_TICKET lookup should succeed");
    assert_eq!(
        submit, 14.0,
        "HELPFRAME_SUBMIT_TICKET should be 14 — HelpFrame.lua line 24 declares the \
         submit-ticket ShowFrame key constant, branched on by HelpFrameMixin:ShowFrame to \
         decide whether to call HelpBrowser:NavigateHome on next show"
    );
}
}

prefork_full_ui_case! {
fn blizzard_help_frame_publishes_help_frame_mixin_methods(env: &WowLuaEnv) {

    for method in [
        "OnLoad",
        "OnShow",
        "OnHide",
        "OnEvent",
        "OnError",
        "ShowFrame",
        "SetInitialLoading",
        "GetInitialLoading",
        "ShowUnavailable",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HelpFrameMixin['{method}']) == 'function'"
            ))
            .expect("HelpFrameMixin method existence query should succeed");
        assert!(
            exists,
            "HelpFrameMixin must expose `:{method}()` — HelpFrame.lua publishes 9 mixin methods \
             driving the F1 panel lifecycle (OnLoad registers UPDATE_GM_STATUS / \
             QUICK_TICKET_SYSTEM_STATUS / QUICK_TICKET_THROTTLE_CHANGED / \
             SIMPLE_BROWSER_WEB_PROXY_FAILED / SIMPLE_BROWSER_WEB_ERROR), the loading-spinner \
             overlay (Set/GetInitialLoading), the browser-driven error path (OnError routes to \
             ShowUnavailable when initial load failed and to DEFAULT_CHAT_FRAME otherwise), and \
             the cross-feature ShowFrame entry-point that toggles `navigateHomeOnShow` on the \
             HELPFRAME_SUBMIT_TICKET key"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_frame_publishes_help_open_web_ticket_button_helpers(env: &WowLuaEnv) {

    for helper in [
        "HelpOpenWebTicketButton_OnEnter",
        "HelpOpenWebTicketButton_OnUpdate",
        "HelpOpenWebTicketButton_OnEvent",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{helper}']) == 'function'"))
            .expect("HelpOpenWebTicketButton helper existence query should succeed");
        assert!(
            exists,
            "HelpFrame.lua must publish `_G['{helper}']` — these 3 free functions wire the \
             CharacterMicroButton-anchored ticket-status icon: OnEnter renders the ticket / \
             survey / response tooltip, OnUpdate polls GetWebTicket every \
             GMTICKET_CHECK_INTERVAL seconds, OnEvent handles UPDATE_WEB_TICKET payloads to \
             switch between the open / NMI / response / survey states"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_frame_publishes_ticket_status_frame_handlers(env: &WowLuaEnv) {

    for helper in [
        "TicketStatusFrame_OnLoad",
        "TicketStatusFrame_OnEvent",
        "TicketStatusFrame_OnShow",
        "TicketStatusFrame_OnHide",
        "TicketStatusFrameButton_OnLoad",
        "TicketStatusFrameButton_OnClick",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{helper}']) == 'function'"))
            .expect("TicketStatusFrame helper existence query should succeed");
        assert!(
            exists,
            "HelpFrame.lua must publish `_G['{helper}']` — these 6 free functions wire the \
             top-of-screen TicketStatusFrame banner: OnLoad registers UPDATE_WEB_TICKET, \
             OnEvent toggles visibility based on ticket status (NMI / SURVEY / RESPONSE), \
             OnShow/OnHide call UIParent_UpdateTopFramePositions to reflow the top frame \
             strip, the inner Button OnLoad lowers its frame level by 1 so the title text \
             paints above it, and OnClick opens the ticket details in HelpBrowser via \
             HelpFrame:ShowFrame(HELPFRAME_SUBMIT_TICKET)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_frame_publishes_two_free_helpers(env: &WowLuaEnv) {

    for helper in [
        "HelpFrame_IsGMTicketQueueActive",
        "HelpFrame_ShowReportCheatingDialog",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{helper}']) == 'function'"))
            .expect("HelpFrame helper existence query should succeed");
        assert!(
            exists,
            "HelpFrame.lua must publish `_G['{helper}']` — IsGMTicketQueueActive returns the \
             cached UPDATE_GM_STATUS flag (read by other addons gating ticket UI on queue \
             availability), and ShowReportCheatingDialog initializes the ReportCheatingDialog \
             with C_ReportSystem.InitiateReportPlayer and pops it via StaticPopupSpecial_Show \
             (called from cheat-report keybindings and unit-frame context menus)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_frame_publishes_external_link_static_popup(env: &WowLuaEnv) {

    let exists: bool = env
        .eval(
            "return type(StaticPopupDialogs['EXTERNAL_LINK']) == 'table' \
             and type(StaticPopupDialogs['EXTERNAL_LINK'].OnAccept) == 'function' \
             and type(StaticPopupDialogs['EXTERNAL_LINK'].OnAlt) == 'function'",
        )
        .expect("StaticPopupDialogs['EXTERNAL_LINK'] lookup should succeed");
    assert!(
        exists,
        "HelpFrame.lua must register `StaticPopupDialogs['EXTERNAL_LINK']` with both OnAccept \
         and OnAlt callbacks — OnAccept calls `data.browser:OpenExternalLink()` to launch the \
         system browser, OnAlt calls `data.browser:CopyExternalLink()` to copy the URL to the \
         system clipboard. Triggered by the BrowserTemplate's <OnExternalLink> script when \
         the embedded help browser intercepts an external link click"
    );
}
}

prefork_full_ui_case! {
fn blizzard_help_frame_publishes_help_frame_global(env: &WowLuaEnv) {

    let exists: bool = env
        .eval(
            "local f = _G['HelpFrame']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("HelpFrame global lookup should succeed");
    assert!(
        exists,
        "After Game-screen auto-load, `HelpFrame` should publish as a global frame instance — \
         HelpFrame.xml line 37 declares `<Frame name=\"HelpFrame\" toplevel=\"true\" \
         parent=\"UIParent\" inherits=\"DefaultPanelTemplate\" hidden=\"true\" \
         enableMouse=\"true\" mixin=\"HelpFrameMixin\">` so the named non-virtual frame \
         materializes as a runtime frame with the mixin methods bound and the 5 GM/ticket \
         events registered at OnLoad"
    );
}
}

prefork_full_ui_case! {
fn blizzard_help_frame_publishes_ticket_status_frame_and_report_cheating_globals(env: &WowLuaEnv) {

    for frame_name in [
        "TicketStatusFrame",
        "TicketStatusTitleText",
        "TicketStatusFrameIcon",
        "TicketStatusFrameButton",
        "ReportCheatingDialog",
        "BrowserSettingsTooltip",
        "HelpOpenWebTicketButton",
    ] {
        let exists: bool = env
            .eval(&format!(
                "local f = _G['{frame_name}']; return type(f) == 'table' and type(f.GetName) == 'function'"
            ))
            .expect("global frame lookup should succeed");
        assert!(
            exists,
            "After Game-screen auto-load, `{frame_name}` should publish as a global frame \
             instance — HelpFrame.xml declares 7 named non-virtual frames at file scope: \
             HelpFrame (DefaultPanelTemplate F1 panel) + TicketStatusFrame (top-of-screen GM \
             ticket banner with TicketStatusTitleText / TicketStatusFrameIcon child layers \
             and TicketStatusFrameButton child) + ReportCheatingDialog (frameStrata=DIALOG \
             cheat-report popup) + BrowserSettingsTooltip (delete-cookies tooltip) + \
             HelpOpenWebTicketButton (CharacterMicroButton-overlay icon)"
        );
    }
}
}
