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

fn guild_rename_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GuildRename")
}

fn guild_rename_toc() -> PathBuf {
    guild_rename_dir().join("Blizzard_GuildRename.toc")
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
fn blizzard_guild_rename_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&guild_rename_dir()).expect("Blizzard_GuildRename TOC should resolve");
    assert_eq!(
        resolved,
        guild_rename_toc(),
        "Blizzard_GuildRename ships exactly one bare TOC (`Blizzard_GuildRename.toc`) — no \
         flavor variants. `find_toc_file` (src/loader/mod.rs:65) falls through to the bare \
         `.toc` suffix after the flavor-specific lookups miss"
    );
}

#[test]
fn blizzard_guild_rename_toc_declares_non_lod_with_uipanels_required_dep() {
    let toc = TocFile::from_file(&guild_rename_toc()).expect("GuildRename TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_GuildRename declares `## LoadOnDemand: 0` — the rename dialog is part of the \
         core Game-screen surface and auto-loads on the Game-screen discovery pass"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_GuildRename does not declare `## LoadFirst: 1` — it consumes templates \
         published by Blizzard_UIPanels_Game (ButtonFrameTemplate, GossipTitleButtonTemplate, \
         InputBoxTemplate, MoneyFrameTemplate, etc.) and must load AFTER it"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GuildRename does not declare `## UseSecureEnvironment` — runs in the \
         standard Lua environment"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_UIPanels_Game".to_string()],
        "Blizzard_GuildRename declares exactly one `## RequiredDep: Blizzard_UIPanels_Game` — \
         the rename dialog inherits ButtonFrameTemplate, embeds MoneyFrameTemplate / \
         SmallMoneyFrameTemplate, and consumes RegisterUIPanel / HideUIPanel / ShowUIPanel from \
         the UIPanels_Game addon"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_GuildRename declares NO `## SavedVariables*` — rename state, refund timer, \
         and reservation lifecycle are server-authoritative (queried via \
         C_GuildInfo.RequestRenameStatus / RequestRenameNameCheck), no per-installation \
         persistence is needed"
    );
}

#[test]
fn blizzard_guild_rename_toc_declares_game_screen_standard_only() {
    let toc = TocFile::from_file(&guild_rename_toc()).expect("GuildRename TOC should parse");
    let toc_text =
        std::fs::read_to_string(guild_rename_toc()).expect("GuildRename TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: Game"),
        "Blizzard_GuildRename declares `## AllowLoad: Game` (capital G — `allows_screen()` \
         (src/toc.rs:305) lowercases before matching). The rename NPC interaction is exclusive \
         to in-game NPC contact, so the addon never loads on glue screens"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: standard"),
        "Blizzard_GuildRename declares `## AllowLoadGameType: standard` — the guild rename \
         feature is retail-only. `is_game_type_restricted()` (src/toc.rs:294) treats \
         `standard` and `mainline` as the unrestricted retail flavor"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_GuildRename must NOT be game-type restricted — `standard` is the retail \
         alias the simulator targets"
    );
}

#[test]
fn blizzard_guild_rename_toc_lists_lua_then_xml() {
    let toc = TocFile::from_file(&guild_rename_toc()).expect("GuildRename TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_GuildRename.lua".to_string(),
            "Blizzard_GuildRename.xml".to_string(),
        ],
        "Blizzard_GuildRename TOC body lists exactly 2 files in order: Lua first (publishes the \
         7 mixins — SimpleTooltipRegionMixin, GuildRenameFrameMixin, GuildRenameManagedFlowMixin, \
         GuildRenameFlowMixin, GuildRenameTitleFlowMixin, GuildRenameContextButtonMixin, \
         GuildIconDisplayMixin — and the 2 StaticPopupDialogs entries which the XML's `mixin=` \
         attributes must resolve at frame-instantiation time), then XML"
    );
}

#[test]
fn blizzard_guild_rename_directory_ships_three_entries() {
    let dir = guild_rename_dir();
    let mut entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_GuildRename directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_GuildRename.lua".to_string(),
            "Blizzard_GuildRename.toc".to_string(),
            "Blizzard_GuildRename.xml".to_string(),
        ],
        "Blizzard_GuildRename directory ships exactly 3 entries (TOC + Lua + XML), no flavor \
         subdirectory and no Localization.lua — strings are pulled from the global locale \
         table (GUILD_RENAME_*, GOODBYE)"
    );
}

#[test]
fn blizzard_guild_rename_appears_exactly_once_in_game_screen_auto_discovery() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let count = addons
        .iter()
        .filter(|(name, _)| name == "Blizzard_GuildRename")
        .count();
    assert_eq!(
        count, 1,
        "Blizzard_GuildRename must auto-discover EXACTLY ONCE on the Game screen — non-LoD + \
         `## AllowLoad: Game` + `## AllowLoadGameType: standard` qualify it for the Game-screen \
         discovery pass, and the single bare TOC means there's no flavor-variant duplication"
    );
}

#[test]
fn blizzard_guild_rename_excluded_from_all_glue_screen_auto_discovery_passes() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_GuildRename");
        assert!(
            !discovered,
            "Blizzard_GuildRename MUST NOT appear in {screen:?} auto-discovery — \
             `## AllowLoad: Game` is Game-screen only and the rename NPC interaction is \
             unreachable from any glue screen"
        );
    }
}

prefork_full_ui_case! {
fn npc_interaction_query_tracks_open_mailbox_for_blizzard_guild_rename_consumer(env: &WowLuaEnv) {
    env.exec("A_Admin.OpenMailbox()").unwrap();

    let interacting_with_mailbox: bool = env
        .eval("return C_PlayerInteractionManager.IsInteractingWithNpcOfType(Enum.PlayerInteractionType.MailInfo)")
        .unwrap();
    let interacting_with_merchant: bool = env
        .eval("return C_PlayerInteractionManager.IsInteractingWithNpcOfType(Enum.PlayerInteractionType.Merchant)")
        .unwrap();
    assert!(interacting_with_mailbox);
    assert!(!interacting_with_merchant);

    let was_shown: bool = env.eval("return GuildRenameFrame:IsShown()").unwrap();
    env.exec(
        "GuildRenameFrameMixin.OnGuildRenameStatusUpdate(GuildRenameFrame, Enum.GuildErrorType.Success)",
    )
    .unwrap();
    let shown_after_foreign_interaction: bool =
        env.eval("return GuildRenameFrame:IsShown()").unwrap();
    assert!(!was_shown);
    assert!(!shown_after_foreign_interaction);
}
}

prefork_full_ui_case! {
fn blizzard_guild_rename_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_GuildRename/")
                || e.contains("Blizzard_GuildRename\\")
                || e.contains("GuildRenameFrame")
                || e.contains("GuildRenameFrameMixin")
                || e.contains("GuildRenameFlowMixin")
                || e.contains("GuildRenameTitleFlowMixin")
                || e.contains("GuildRenameContextButtonMixin")
                || e.contains("SimpleTooltipRegionMixin")
                || e.contains("GuildIconDisplayMixin")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_GuildRename emitted addon-specific Lua errors during Game-screen auto-load:\n  \
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
fn blizzard_guild_rename_is_addon_loaded_returns_true_after_game_screen_load(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_GuildRename')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "After Game-screen auto-discovery, \
         `C_AddOns.IsAddOnLoaded('Blizzard_GuildRename')` should return true"
    );
}
}

prefork_full_ui_case! {
fn blizzard_guild_rename_publishes_simple_tooltip_region_mixin(env: &WowLuaEnv) {

    let exists: bool = env
        .eval(
            "return type(SimpleTooltipRegionMixin) == 'table' \
             and type(SimpleTooltipRegionMixin.OnEnter) == 'function' \
             and type(SimpleTooltipRegionMixin.OnLeave) == 'function' \
             and type(SimpleTooltipRegionMixin.SetTooltip) == 'function'",
        )
        .expect("SimpleTooltipRegionMixin lookup should succeed");
    assert!(
        exists,
        "Blizzard_GuildRename.lua publishes SimpleTooltipRegionMixin as a `_G` table with \
         OnEnter / OnLeave / SetTooltip methods — a generic mixin reused by the rename frame's \
         MoneyFrame, GuildIcon, and ContextButton tooltip-on-hover surfaces"
    );
}
}

prefork_full_ui_case! {
fn blizzard_guild_rename_publishes_seven_mixin_globals(env: &WowLuaEnv) {

    for mixin in [
        "SimpleTooltipRegionMixin",
        "GuildRenameFrameMixin",
        "GuildRenameManagedFlowMixin",
        "GuildRenameFlowMixin",
        "GuildRenameTitleFlowMixin",
        "GuildRenameContextButtonMixin",
        "GuildIconDisplayMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("mixin existence query should succeed");
        assert!(
            exists,
            "After Game-screen auto-load, `{mixin}` should publish as a `_G` mixin table — \
             Blizzard_GuildRename.lua declares 7 mixins: SimpleTooltipRegionMixin (line 1), \
             GuildRenameFrameMixin (line 41), GuildRenameManagedFlowMixin (line 345), \
             GuildRenameFlowMixin (line 355, CreateFromMixins TimedCallbackMixin + Managed), \
             GuildRenameTitleFlowMixin (line 450, CreateFromMixins Managed + timeFormatter), \
             GuildRenameContextButtonMixin (line 549, CreateFromMixins Tooltip), and \
             GuildIconDisplayMixin (line 598, CreateFromMixins Tooltip)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_rename_frame_mixin_exposes_status_query_methods(env: &WowLuaEnv) {

    for method in [
        "OnLoad",
        "OnShow",
        "OnHide",
        "OnEvent",
        "AddModeFrame",
        "SetSpinnerShown",
        "OnGuildRenameStatusUpdate",
        "GetRenamePermissionStatus",
        "GetNameChangeRequestStatus",
        "GetExecuteNameChangeStatus",
        "HasRenamePermission",
        "IsPlayerGuildMaster",
        "GetRenameCost",
        "GetCurrentGuildMoney",
        "IsRenameEnabled",
        "GetRefundAmount",
        "GetRenameCooldownRemaining",
        "IsRenameCooldownActive",
        "GetPreviousGuildName",
        "IsReservedNameValid",
        "GetReservedName",
        "NameMatchesExistingReservation",
        "BeginInteraction",
        "BeginInteractionMode",
        "UpdateInteractionMode",
        "GetMode",
        "UpdateFromMode",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(GuildRenameFrameMixin['{method}']) == 'function'"
            ))
            .expect("method existence query should succeed");
        assert!(
            exists,
            "GuildRenameFrameMixin must expose `:{method}()` — \
             Blizzard_GuildRename.lua declares 27 methods on GuildRenameFrameMixin spanning \
             frame lifecycle (OnLoad/OnShow/OnHide/OnEvent), mode-frame dispatch (Add/Update \
             InteractionMode + UpdateFromMode), and 21 read-only status query helpers that \
             pull from the cached `self.status` payload returned by the GUILD_RENAME_STATUS_UPDATE \
             event"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_rename_flow_mixin_inherits_timed_callback_and_manager(env: &WowLuaEnv) {

    for method in [
        "OnLoad",
        "OnShow",
        "CheckRequestNameChange",
        "UpdateFromStatus",
        "GetDesiredName",
        "UpdateFlowNameStatus",
        "ClearRenameStatus",
        "SetManager",
        "GetManager",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(GuildRenameFlowMixin['{method}']) == 'function'"
            ))
            .expect("method existence query should succeed");
        assert!(
            exists,
            "GuildRenameFlowMixin must expose `:{method}()` — line 355 of \
             Blizzard_GuildRename.lua declares `GuildRenameFlowMixin = CreateFromMixins(\
             TimedCallbackMixin, GuildRenameManagedFlowMixin)`, so the mixin inherits \
             :SetCheckDelaySeconds / :RunCallbackAsync from TimedCallbackMixin and \
             :SetManager / :GetManager from GuildRenameManagedFlowMixin in addition to its \
             own 7 directly-declared methods"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_rename_title_flow_mixin_exposes_rename_and_refund_options(env: &WowLuaEnv) {

    for method in [
        "OnLoad",
        "OnUpdate",
        "UpdateOptions",
        "UpdateFromStatus",
        "FormatTime",
        "SetManager",
        "GetManager",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(GuildRenameTitleFlowMixin['{method}']) == 'function'"
            ))
            .expect("method existence query should succeed");
        assert!(
            exists,
            "GuildRenameTitleFlowMixin must expose `:{method}()` — line 450 declares \
             `GuildRenameTitleFlowMixin = CreateFromMixins(GuildRenameManagedFlowMixin, \
             {{ timeFormatter = timeFormatter }})`, so :SetManager / :GetManager are inherited \
             and the per-mixin :OnLoad attaches OnClick handlers to RenameOption / RefundOption \
             that drive BeginInteractionMode / StaticPopup_Show, while :UpdateOptions toggles \
             the option buttons based on cached permission and cooldown state"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_rename_context_button_mixin_inherits_tooltip_and_overrides_on_enter(env: &WowLuaEnv) {

    let exists: bool = env
        .eval(
            "return type(GuildRenameContextButtonMixin) == 'table' \
             and type(GuildRenameContextButtonMixin.SetToGuildRename) == 'function' \
             and type(GuildRenameContextButtonMixin.SetToGoodbye) == 'function' \
             and type(GuildRenameContextButtonMixin.OnEnter) == 'function' \
             and type(GuildRenameContextButtonMixin.OnLeave) == 'function' \
             and type(GuildRenameContextButtonMixin.SetTooltip) == 'function'",
        )
        .expect("GuildRenameContextButtonMixin lookup should succeed");
    assert!(
        exists,
        "GuildRenameContextButtonMixin must expose its own :SetToGuildRename / :SetToGoodbye / \
         :OnEnter (which overrides the inherited tooltip handler to gate visibility on \
         self.renameStatus) and inherit :OnLeave / :SetTooltip from SimpleTooltipRegionMixin — \
         line 549 declares `GuildRenameContextButtonMixin = \
         CreateFromMixins(SimpleTooltipRegionMixin)`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_guild_rename_publishes_static_popup_dialog_entries(env: &WowLuaEnv) {

    for popup_key in [
        "CONFIRM_PURCHASE_GUILD_RENAME",
        "CONFIRM_GUILD_RENAME_REFUND",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(StaticPopupDialogs['{popup_key}']) == 'table' \
                 and type(StaticPopupDialogs['{popup_key}'].OnAccept) == 'function'"
            ))
            .expect("StaticPopupDialogs lookup should succeed");
        assert!(
            exists,
            "StaticPopupDialogs['{popup_key}'] must register with an OnAccept callback — \
             Blizzard_GuildRename.lua declares 2 popup entries: CONFIRM_PURCHASE_GUILD_RENAME \
             (calls C_GuildInfo.RequestGuildRename with the desired name) and \
             CONFIRM_GUILD_RENAME_REFUND (calls C_GuildInfo.RequestGuildRenameRefund). Both \
             pin button1 / button2 to GUILD_RENAME_DIALOG_CONFIRM_BUTTON / \
             GUILD_RENAME_DIALOG_CANCEL_BUTTON and set hideOnEscape = 1"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_guild_rename_publishes_guild_rename_frame_global(env: &WowLuaEnv) {

    let exists: bool = env
        .eval(
            "local f = _G['GuildRenameFrame']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("GuildRenameFrame existence query should succeed");
    assert!(
        exists,
        "After Game-screen auto-load, `GuildRenameFrame` should publish as a global frame \
         instance — Blizzard_GuildRename.xml declares `<UIThemeContainerFrame \
         name=\"GuildRenameFrame\" toplevel=\"true\" parent=\"UIParent\" movable=\"true\" \
         enableMouse=\"true\" hidden=\"true\" inherits=\"ButtonFrameTemplate\" \
         mixin=\"GuildRenameFrameMixin\">` so the named non-virtual frame materializes as a \
         runtime frame published under its declared name"
    );
}
}
