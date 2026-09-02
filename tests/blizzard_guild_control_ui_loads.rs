#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn guild_control_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GuildControlUI")
}

fn guild_control_toc() -> PathBuf {
    guild_control_dir().join("Blizzard_GuildControlUI.toc")
}

fn load_guild_control_ui(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &guild_control_toc())
        .expect("Blizzard_GuildControlUI should load via explicit Rust loader call");
}

#[test]
fn blizzard_guild_control_ui_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&guild_control_dir()).expect("Blizzard_GuildControlUI TOC should resolve");
    assert_eq!(
        resolved,
        guild_control_toc(),
        "Blizzard_GuildControlUI ships exactly one bare TOC (`Blizzard_GuildControlUI.toc`) — \
         no flavor variants. `find_toc_file` (src/loader/mod.rs:65) falls through to the bare \
         `.toc` suffix after the flavor-specific lookups miss"
    );
}

#[test]
fn blizzard_guild_control_ui_toc_declares_lod_with_no_dependencies() {
    let toc = TocFile::from_file(&guild_control_toc()).expect("GuildControlUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_GuildControlUI declares `## LoadOnDemand: 1` — the rank-permission editor \
         is loaded lazily the first time the player (with appropriate guild rank) opens the \
         guild-control panel from the GuildFrame"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_GuildControlUI does not declare `## LoadFirst: 1` — LoadOnDemand precludes \
         any load-order priority"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GuildControlUI does not declare `## UseSecureEnvironment` — runs in the \
         standard Lua environment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_GuildControlUI declares NO `## Dependencies:` line — the addon consumes \
         only globally-available templates (TranslucentFrameTemplate, ScrollFrame templates) \
         and the C_GuildInfo / GuildControl* API surface, all loaded by core addons that \
         auto-load before any LoD addon"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_GuildControlUI declares NO `## SavedVariables*` — guild-control changes \
         are committed via C_GuildInfo / GuildControlSetRank etc. and round-trip server-side, \
         so no per-installation persistence is needed"
    );
}

#[test]
fn blizzard_guild_control_ui_toc_declares_game_screen_mainline_plus_classic() {
    let toc = TocFile::from_file(&guild_control_toc()).expect("GuildControlUI TOC should parse");
    let toc_text =
        std::fs::read_to_string(guild_control_toc()).expect("GuildControlUI TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: Game"),
        "Blizzard_GuildControlUI declares `## AllowLoad: Game` (capital G — `allows_screen()` \
         (src/toc.rs:305) lowercases before matching). LoadOnDemand keeps the addon out of \
         auto-discovery anyway, but the AllowLoad directive documents intent"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: mainline, classic"),
        "Blizzard_GuildControlUI declares `## AllowLoadGameType: mainline, classic` — a \
         comma-separated list of valid game types. `is_game_type_restricted()` (src/toc.rs:294) \
         splits on `,`, trims each entry, and only flags the addon as restricted if NONE of \
         the entries are `mainline` or `standard`. Since `mainline` is in the list, the addon \
         is NOT considered game-type-restricted"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_GuildControlUI must NOT be game-type restricted — its AllowLoadGameType \
         list contains `mainline`, so the retail simulator considers it unrestricted"
    );
}

#[test]
fn blizzard_guild_control_ui_toc_lists_bootstrap_lua_xml_and_localization() {
    let toc = TocFile::from_file(&guild_control_toc()).expect("GuildControlUI TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_GuildControlUI_Bootstrap.lua".to_string(),
            "Blizzard_GuildControlUI.lua".to_string(),
            "Blizzard_GuildControlUI.xml".to_string(),
            "Localization.lua".to_string(),
        ],
        "Blizzard_GuildControlUI TOC lists its bootstrap, Lua, XML, and Localization in \
         published order."
    );
}

#[test]
fn blizzard_guild_control_ui_directory_ships_five_entries() {
    let dir = guild_control_dir();
    let mut entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_GuildControlUI directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_GuildControlUI.lua".to_string(),
            "Blizzard_GuildControlUI.toc".to_string(),
            "Blizzard_GuildControlUI.xml".to_string(),
            "Blizzard_GuildControlUI_Bootstrap.lua".to_string(),
            "Localization.lua".to_string(),
        ],
        "Blizzard_GuildControlUI directory ships its bootstrap, TOC, Lua, XML, and Localization."
    );
}

#[test]
fn blizzard_guild_control_ui_pulled_into_game_discovery_via_communities_required_dep() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let discovered = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GuildControlUI");
    assert!(
        discovered,
        "Blizzard_GuildControlUI is LoD but Blizzard_Communities_Mainline.toc declares \
         `## RequiredDep: ..., Blizzard_GuildControlUI, ...`. `pull_required_lod_addons` \
         (src/loader/mod.rs:357) reaches into the LoD pool and promotes any LoD addon \
         required by a non-LoD addon into the auto-discovery set, so GuildControlUI loads \
         automatically on the Game screen as part of Communities' dep closure"
    );
}

#[test]
fn blizzard_guild_control_ui_excluded_from_all_glue_screen_auto_discovery_passes() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_GuildControlUI");
        assert!(
            !discovered,
            "Blizzard_GuildControlUI MUST NOT appear in {screen:?} auto-discovery — \
             `## AllowLoad: Game` is Game-screen only, so even though Blizzard_Communities \
             pulls it in via RequiredDep, the glue-screen discovery passes filter both \
             addons out before the LoD-promotion step runs"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_guild_control_ui_loads_explicitly_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_guild_control_ui(env);

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_GuildControlUI/")
                || e.contains("Blizzard_GuildControlUI\\")
                || e.contains("GuildControlUI_")
                || e.contains("GUILD_OFFICER_PERMISSION_STRINGS")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_GuildControlUI emitted addon-specific Lua errors during explicit \
         LoadAddOn:\n  {}",
        related
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_guild_control_ui_is_addon_loaded_returns_true_after_explicit_load(env: &WowLuaEnv) {
    load_guild_control_ui(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_GuildControlUI')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "After explicit LoadAddOn, `C_AddOns.IsAddOnLoaded('Blizzard_GuildControlUI')` should \
         return true"
    );
}
}

prefork_full_ui_case! {
fn blizzard_guild_control_ui_publishes_three_module_constants(env: &WowLuaEnv) {
    load_guild_control_ui(env);

    let bank_tab_offset: f64 = env
        .eval("return BANK_TAB_OFFSET")
        .expect("BANK_TAB_OFFSET lookup should succeed");
    assert_eq!(
        bank_tab_offset, 4.0,
        "BANK_TAB_OFFSET should be 4 — Blizzard_GuildControlUI.lua line 1 declares \
         `BANK_TAB_OFFSET = 4` (the vertical pixel offset between guild-bank tab rows in \
         the rank-permission editor)"
    );

    let bank_tab_height: f64 = env
        .eval("return BANK_TAB_HEIGHT")
        .expect("BANK_TAB_HEIGHT lookup should succeed");
    assert_eq!(
        bank_tab_height, 77.0,
        "BANK_TAB_HEIGHT should be 77 (BANK_TAB_OFFSET + 73) — Blizzard_GuildControlUI.lua \
         line 2 declares `BANK_TAB_HEIGHT = BANK_TAB_OFFSET + 73`"
    );

    let num_rank_flags: f64 = env
        .eval("return NUM_RANK_FLAGS")
        .expect("NUM_RANK_FLAGS lookup should succeed");
    assert_eq!(
        num_rank_flags, 21.0,
        "NUM_RANK_FLAGS should be 21 — Blizzard_GuildControlUI.lua line 3 declares \
         `NUM_RANK_FLAGS = 21` (the number of distinct rank-permission checkboxes the editor \
         displays per rank)"
    );

    let max_guild_ranks: f64 = env
        .eval("return MAX_GUILDRANKS")
        .expect("MAX_GUILDRANKS lookup should succeed");
    assert_eq!(
        max_guild_ranks, 10.0,
        "MAX_GUILDRANKS should be 10 — Blizzard_GuildControlUI.lua line 4 declares \
         `MAX_GUILDRANKS = 10` (the maximum number of guild ranks supported by the rank-list \
         and the AddRank/RemoveRank button gating)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_guild_control_ui_publishes_guild_officer_permission_strings_table(env: &WowLuaEnv) {
    load_guild_control_ui(env);

    let exists: bool = env
        .eval("return type(GUILD_OFFICER_PERMISSION_STRINGS) == 'table'")
        .expect("GUILD_OFFICER_PERMISSION_STRINGS lookup should succeed");
    assert!(
        exists,
        "GUILD_OFFICER_PERMISSION_STRINGS should publish as a `_G` table — \
         Blizzard_GuildControlUI.lua declares this table mapping guild-officer permission \
         flags to their display strings, consumed by the rank-permission editor's checkbox \
         labels"
    );
}
}

prefork_full_ui_case! {
fn blizzard_guild_control_ui_publishes_seventeen_guild_control_helpers(env: &WowLuaEnv) {
    load_guild_control_ui(env);

    for helper in [
        "GuildControlUI_OnLoad",
        "GuildControlUI_OnEvent",
        "GuildControlUI_SubmitChanges",
        "GuildControlUI_SetBankTabChange",
        "GuildControlUI_SetBankTabWithdrawChange",
        "GuildControlUI_BankTabPermissions_Update",
        "GuildControlUI_RankPermissions_Update",
        "GuildControlUI_RankOrder_Update",
        "GuildControlUI_BankFrame_OnLoad",
        "GuildControlUI_CheckClicked",
        "GuildControlUI_RemoveRankButton_OnClick",
        "GuildControlUI_AddRankButton_OnClick",
        "GuildControlUI_ShiftRankDownButton_OnClick",
        "GuildControlUI_ShiftRankUpButton_OnClick",
        "GuildControlUI_DisableRankButtons",
        "GuildControlUIRankDropdown_OnClick",
        "GuildControlUIRankPermissions_HideGuildBankOptions",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{helper}']) == 'function'"))
            .expect("helper existence query should succeed");
        assert!(
            exists,
            "After explicit LoadAddOn, `{helper}` should publish as a `_G` function — \
             Blizzard_GuildControlUI.lua publishes 17 GuildControlUI* helpers (15 \
             GuildControlUI_* + 2 GuildControlUIRank* — naming inconsistency is intentional, \
             the dropdown click and bank-options hide handler use the no-underscore name) \
             consumed by the XML script bindings on the rank-list buttons, the \
             permission-checkbox click routing, the bank-tab toggle handlers, and the \
             rank-shift up/down arrows"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_control_ui_publishes_two_helper_on_load_globals(env: &WowLuaEnv) {
    load_guild_control_ui(env);

    for helper in [
        "GuildControlRankSettings_OnLoad",
        "GuildControlRankBank_OnLoad",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{helper}']) == 'function'"))
            .expect("helper existence query should succeed");
        assert!(
            exists,
            "After explicit LoadAddOn, `{helper}` should publish as a `_G` function — \
             Blizzard_GuildControlUI.lua declares 2 helper OnLoad globals (no UI_ prefix) \
             consumed by the GuildControlRankSettings and GuildControlRankBank XML frames' \
             OnLoad script bindings"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_control_ui_publishes_guild_control_ui_global_frame(env: &WowLuaEnv) {
    load_guild_control_ui(env);

    let exists: bool = env
        .eval(
            "local f = _G['GuildControlUI']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("GuildControlUI existence query should succeed");
    assert!(
        exists,
        "After explicit LoadAddOn, `GuildControlUI` should publish as a global frame \
         instance — Blizzard_GuildControlUI.xml declares `<Frame name=\"GuildControlUI\" \
         inherits=\"TranslucentFrameTemplate\" toplevel=\"true\" frameStrata=\"DIALOG\" \
         parent=\"UIParent\" movable=\"true\" enableMouse=\"true\" hidden=\"true\">` so the \
         named non-virtual frame materializes as a runtime frame published under its declared \
         name"
    );

    let checkbox_text_publication: (bool, bool, bool, bool) = env
        .eval(
            r#"
            return
                _G.GuildControlUIRankSettingsFrameCheckbox2Text ~= nil,
                _G.GuildControlUIRankSettingsFrameCheckbox21Text ~= nil,
                _G.GuildControlUIRankSettingsFrameCheckbox1Text == nil,
                _G.GuildControlUIRankSettingsFrameCheckbox3Text == nil
            "#,
        )
        .expect("GuildControl checkbox publication query should succeed");
    assert_eq!(
        checkbox_text_publication,
        (true, true, true, true),
        "XML declares checkbox text children 2 and 21, while dynamically probed IDs 1 and 3 \
         remain absent"
    );
}
}
