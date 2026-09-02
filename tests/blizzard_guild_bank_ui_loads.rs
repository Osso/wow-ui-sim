#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
#[cfg(feature = "client-ptr")]
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn guild_bank_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GuildBankUI")
}

fn guild_bank_toc() -> PathBuf {
    find_toc_file(&guild_bank_dir()).expect("Blizzard_GuildBankUI TOC should resolve")
}

fn load_guild_bank_ui(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &guild_bank_toc())
        .expect("Blizzard_GuildBankUI should load via explicit Rust loader call");
}

#[cfg(feature = "client-ptr")]
fn load_full_game_ui_with_guild_bank_lod() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &guild_bank_toc())
        .expect("Blizzard_GuildBankUI should load via explicit Rust loader call");

    env
}

#[test]
fn blizzard_guild_bank_ui_find_toc_resolves_mainline_variant() {
    let resolved = guild_bank_toc();
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some("Blizzard_GuildBankUI_Mainline.toc"),
        "retail resolves the mainline GuildBankUI TOC through find_toc_file"
    );
}

#[test]
fn blizzard_guild_bank_ui_toc_declares_lod_with_no_dependencies() {
    let toc = TocFile::from_file(&guild_bank_toc()).expect("GuildBankUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_GuildBankUI declares `## LoadOnDemand: 1` — the guild-bank UI is loaded \
         lazily the first time the player interacts with a guild-bank NPC (the GuildFrame \
         opens it via `LoadAddOn(\"Blizzard_GuildBankUI\")`)"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_GuildBankUI does not declare `## LoadFirst: 1` — LoadOnDemand precludes any \
         load-order priority since the addon only loads on explicit demand"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GuildBankUI does not declare `## UseSecureEnvironment` — runs in the \
         standard Lua environment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_GuildBankUI declares NO `## Dependencies:` line — the addon consumes only \
         globally-available templates (BasicFrameTemplate, SmallMoneyFrameTemplate, \
         IconSelectorPopupFrameTemplate, PanelTabButtonTemplate) and the C_GuildBank / \
         C_Container API surfaces, all of which are loaded by core addons that auto-load on \
         the Game screen before any LoD addon runs"
    );
}

#[test]
fn blizzard_guild_bank_ui_mainline_toc_declares_game_screen_mainline_gating() {
    let toc_text = std::fs::read_to_string(guild_bank_toc()).expect("GuildBankUI TOC should read");
    assert!(toc_text.contains("## AllowLoad: game"));
    assert!(toc_text.contains("## AllowLoadGameType: mainline"));
    assert!(toc_text.contains("## LoadOnDemand: 1"));
}

#[test]
fn blizzard_guild_bank_ui_toc_lists_bootstrap_xml_then_localization() {
    let toc = TocFile::from_file(&guild_bank_toc()).expect("GuildBankUI TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_GuildBankUI_Bootstrap.lua".to_string(),
            "Mainline/Blizzard_GuildBankUI.xml".to_string(),
            "Localization.lua".to_string(),
        ],
        "Blizzard_GuildBankUI Mainline TOC lists its bootstrap, Mainline XML, and Localization \
         in published order. The XML loads its Lua sibling through `<Script file=\"...\"/>`."
    );
}

#[test]
fn blizzard_guild_bank_ui_directory_ships_eight_entries() {
    let dir = guild_bank_dir();
    let mut entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_GuildBankUI directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_GuildBankUI_Bootstrap.lua".to_string(),
            "Blizzard_GuildBankUI_Classic.toc".to_string(),
            "Blizzard_GuildBankUI_Mainline.toc".to_string(),
            "Blizzard_GuildBankUI_Mists.toc".to_string(),
            "Localization.lua".to_string(),
            "Mainline".to_string(),
            "Mists".to_string(),
            "TBC".to_string(),
        ],
        "Blizzard_GuildBankUI ships its bootstrap, three flavor TOCs, Localization, and \
         Mainline/Mists/TBC source directories"
    );
}

#[test]
fn blizzard_guild_bank_ui_excluded_from_all_screen_auto_discovery_due_to_lod() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_GuildBankUI");
        assert!(
            !discovered,
            "Blizzard_GuildBankUI MUST NOT appear in {screen:?} auto-discovery — \
             `## LoadOnDemand: 1` keeps it out of every auto-discovery pass. The addon \
             loads only via explicit `LoadAddOn(\"Blizzard_GuildBankUI\")` from the \
             GuildFrame's open-bank handler"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_guild_bank_ui_loads_explicitly_without_unexpected_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_guild_bank_ui(env);

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_GuildBankUI/")
                || e.contains("Blizzard_GuildBankUI\\")
                || e.contains("GuildBankFrameMixin")
                || e.contains("GuildBankTabButtonMixin")
                || e.contains("GuildBankPopupFrameMixin")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_GuildBankUI emitted unexpected addon-specific Lua errors during explicit \
         LoadAddOn:\n  {}",
        related
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_guild_bank_does_not_publish_snapshot_only_hide_wrapper() {
    let env = load_full_game_ui_with_guild_bank_lod();

    let wrapper_is_absent: bool = env
        .eval("return HideGuildBankFrame == nil")
        .expect("guild bank wrapper visibility should be queryable");
    assert!(
        wrapper_is_absent,
        "snapshot-only HideGuildBankFrame unexpectedly exists after PTR addon load"
    );
}

prefork_full_ui_case! {
fn blizzard_guild_bank_ui_is_addon_loaded_returns_true_after_explicit_load(env: &WowLuaEnv) {
    load_guild_bank_ui(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_GuildBankUI')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "After explicit LoadAddOn, `C_AddOns.IsAddOnLoaded('Blizzard_GuildBankUI')` should \
         return true — the addon registers itself as loaded once the Rust loader processes \
         its TOC"
    );
}
}

prefork_full_ui_case! {
fn blizzard_guild_bank_ui_publishes_eight_mixins(env: &WowLuaEnv) {
    load_guild_bank_ui(env);

    for mixin in [
        "GuildBankFrameMixin",
        "GuildBankTabButtonMixin",
        "GuildBankFrameTabMixin",
        "GuildBankTabMixin",
        "GuildBankFrameDepositButtonMixin",
        "GuildBankFrameWithdrawButtonMixin",
        "GuildBankItemButtonMixin",
        "GuildBankPopupFrameMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("mixin existence query should succeed");
        assert!(
            exists,
            "After explicit LoadAddOn, `{mixin}` should publish as a `_G` table — \
             Blizzard_GuildBankUI.lua declares 8 top-level mixins (GuildBankFrameMixin / \
             GuildBankTabButtonMixin / GuildBankFrameTabMixin / GuildBankTabMixin / \
             GuildBankFrameDepositButtonMixin / GuildBankFrameWithdrawButtonMixin / \
             GuildBankItemButtonMixin / GuildBankPopupFrameMixin) bound to XML frames via \
             the `mixin=` attribute"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_bank_ui_publishes_guild_bank_frame_mixin_lifecycle_methods(env: &WowLuaEnv) {
    load_guild_bank_ui(env);

    for method in [
        "OnLoad",
        "OnEvent",
        "OnShow",
        "OnHide",
        "Update",
        "UpdateTabs",
        "UpdateTabard",
        "RefreshIconList",
        "SelectAvailableTab",
        "IsTabViewable",
    ] {
        let method_exists: bool = env
            .eval(&format!(
                "return type(_G['GuildBankFrameMixin']['{method}']) == 'function'"
            ))
            .expect("mixin method query should succeed");
        assert!(
            method_exists,
            "GuildBankFrameMixin.{method} should be a function — owned by \
             Blizzard_GuildBankUI.lua. The mixin defines the full GuildBankFrame lifecycle: \
             OnLoad/OnEvent/OnShow/OnHide for visibility, Update/UpdateTabs/UpdateTabard for \
             rendering refresh, RefreshIconList for the popup-frame icon picker, \
             SelectAvailableTab for default-tab selection on open, IsTabViewable for \
             permission gating"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_bank_ui_publishes_guild_bank_frame_global(env: &WowLuaEnv) {
    load_guild_bank_ui(env);

    let exists: bool = env
        .eval(
            "local f = _G['GuildBankFrame']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("GuildBankFrame existence query should succeed");
    assert!(
        exists,
        "After explicit LoadAddOn, `GuildBankFrame` should publish as a global frame \
         instance — Blizzard_GuildBankUI.xml declares `<Frame name=\"GuildBankFrame\" \
         mixin=\"GuildBankFrameMixin\" inherits=\"BasicFrameTemplate\" parent=\"UIParent\" \
         hidden=\"true\" toplevel=\"true\">` so the named non-virtual frame materializes \
         as a runtime frame"
    );
}
}

prefork_full_ui_case! {
fn blizzard_guild_bank_ui_publishes_four_guild_bank_frame_tab_buttons(env: &WowLuaEnv) {
    load_guild_bank_ui(env);

    for tab_id in 1..=4 {
        let exists: bool = env
            .eval(&format!(
                "local f = _G['GuildBankFrameTab{tab_id}']; return type(f) == 'table' and type(f.GetName) == 'function'"
            ))
            .expect("tab button existence query should succeed");
        assert!(
            exists,
            "GuildBankFrameTab{tab_id} should publish as a global Button — \
             Blizzard_GuildBankUI.xml declares 4 named tab buttons (id=1: GUILD_BANK / id=2: \
             GUILD_BANK_LOG / id=3: GUILD_BANK_MONEY_LOG / id=4: GUILD_BANK_TAB_INFO) all \
             inheriting GuildBankFrameTabTemplate (which itself inherits \
             PanelTabButtonTemplate) — these are the top-row tabs that switch between the \
             bank-tab grid, the transaction log, the money log, and the tab-info pane"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_bank_ui_publishes_guild_bank_money_frame_global(env: &WowLuaEnv) {
    load_guild_bank_ui(env);

    let exists: bool = env
        .eval(
            "local f = _G['GuildBankMoneyFrame']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("GuildBankMoneyFrame existence query should succeed");
    assert!(
        exists,
        "After explicit LoadAddOn, `GuildBankMoneyFrame` should publish as a global frame — \
         Blizzard_GuildBankUI.xml declares `<Frame name=\"GuildBankMoneyFrame\" \
         parentKey=\"MoneyFrame\" inherits=\"SmallMoneyFrameTemplate\">` so the named \
         non-virtual frame materializes inside GuildBankFrame, displaying the guild's \
         current money balance"
    );
}
}
