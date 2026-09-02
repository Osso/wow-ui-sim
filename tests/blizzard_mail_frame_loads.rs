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

fn mail_frame_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_MailFrame")
}

fn mail_frame_toc() -> PathBuf {
    mail_frame_dir().join("Blizzard_MailFrame.toc")
}

const MAIL_FRAME_TOC_FILES: &[&str] = &["MailFrame.lua", "MailFrame.xml", "Localization.lua"];

const MAIL_FRAME_DEPENDENCIES: &[&str] = &["Blizzard_FriendsFrame"];

const MAIL_LAYOUT_CONSTANTS: &[(&str, i64)] = &[
    ("INBOXITEMS_TO_DISPLAY", 7),
    ("PACKAGEITEMS_TO_DISPLAY", 4),
    ("ATTACHMENTS_MAX", 16),
    ("ATTACHMENTS_MAX_SEND", 12),
    ("ATTACHMENTS_PER_ROW_SEND", 7),
    ("ATTACHMENTS_MAX_ROWS_SEND", 2),
    ("ATTACHMENTS_MAX_RECEIVE", 16),
    ("ATTACHMENTS_PER_ROW_RECEIVE", 7),
    ("ATTACHMENTS_MAX_ROWS_RECEIVE", 3),
    ("MAX_COD_AMOUNT", 10000),
];

const SEND_MAIL_TAB_LIST_ENTRIES: &[(i64, &str)] = &[
    (1, "SendMailNameEditBox"),
    (2, "SendMailSubjectEditBox"),
    (3, "SendMailBodyEditBox"),
    (4, "SendMailMoneyGold"),
    (5, "SendMailMoneyCopper"),
];

const MAIL_FREE_FUNCTIONS: &[&str] = &[
    "MailFrame_OnLoad",
    "MailFrame_UpdateTrialState",
    "MailFrame_Show",
    "MailFrame_Hide",
    "MailFrame_OnEvent",
    "MailFrame_OnMouseWheel",
    "MailFrameTab_OnClick",
    "MailFrame_RefreshInbox",
    "InboxFrame_Update",
    "InboxFrame_OnClick",
    "InboxFrame_OnModifiedClick",
    "InboxFrameItem_OnEnter",
    "InboxNextPage",
    "InboxPrevPage",
    "InboxGetMoreMail",
    "OpenMailFrame_OnHide",
    "OpenMailFrame_IsValidMailID",
    "OpenMailFrame_UpdateButtonPositions",
    "OpenMail_Update",
    "OpenMail_GetItemCounts",
    "OpenMail_Reply",
    "OpenMail_Delete",
    "OpenMail_ReportSpam",
    "OpenMailAttachment_OnEnter",
    "OpenMailAttachment_OnClick",
    "SendMailMailButton_OnClick",
    "SendMailFrame_SendMail",
    "SendMailFrame_EnableSendMailButton",
    "SendMailFrame_Update",
    "SendMailFrame_Reset",
    "SendMailFrame_CanSend",
    "SendMailRadioButton_OnClick",
    "SendMailMoneyButton_OnClick",
    "SendMailAttachmentButton_OnClick",
    "SendMailAttachmentButton_OnDropAny",
    "SendMailAttachment_OnEnter",
];

const OPEN_ALL_MAIL_MIXIN_METHODS: &[&str] = &[
    "Reset",
    "StartOpening",
    "StopOpening",
    "ShouldSkipCurrentMail",
    "ShouldSkipCurrentAttachment",
    "AdvanceToNextMail",
    "AdvanceToNextItem",
    "AdvanceAndProcessNextItem",
    "ProcessNextItem",
    "OnLoad",
    "OnEvent",
    "OnUpdate",
    "OnClick",
    "OnHide",
    "AddFailedItem",
    "IsItemFailed",
];

const NAMED_MAIL_FRAMES: &[&str] = &[
    "MailFrame",
    "OpenMailFrame",
    "InboxFrame",
    "SendMailFrame",
    "MailFrameTab1",
    "MailFrameTab2",
    "OpenAllMail",
    "SendMailNameEditBox",
    "SendMailSubjectEditBox",
    "SendMailBodyEditBox",
    "SendMailMailButton",
    "SendMailCancelButton",
    "OpenMailReplyButton",
    "OpenMailDeleteButton",
    "OpenMailCancelButton",
    "OpenMailReportSpamButton",
];

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
fn blizzard_mail_frame_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&mail_frame_dir()).expect("Blizzard_MailFrame TOC should resolve");
    assert_eq!(
        resolved,
        mail_frame_toc(),
        "Blizzard_MailFrame ships exactly one bare TOC. The mail UI is mainline-retail-only \
         (Classic ships its own classic-flavored mail addon under a different bucket), so the \
         retail tree carries one Blizzard_MailFrame.toc with no flavor suffix variants — \
         `find_toc_file` resolves to the bare file after the `_Mainline.toc` lookup misses"
    );
}

#[test]
fn blizzard_mail_frame_toc_declares_default_state_enabled_with_allow_load_game() {
    let toc = TocFile::from_file(&mail_frame_toc()).expect("Blizzard_MailFrame TOC parses");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_MailFrame omits `## LoadOnDemand:` — `## DefaultState: enabled` makes it an \
         eager-load Game-screen addon. The mail UI must register its `MAIL_INBOX_UPDATE` / \
         `MAIL_SEND_INFO_UPDATE` / etc handlers at boot so it can respond the moment the player \
         interacts with a mailbox; deferred-load would miss the first `MAIL_INBOX_UPDATE` event"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        MAIL_FRAME_DEPENDENCIES,
        "Retail TOC declares `## Dependencies: Blizzard_FriendsFrame`"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — mail state is server-authoritative (inbox / outbox lives on \
         the server, the client only renders queries) so no per-character or account-wide \
         persistent state is needed"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: mainline` — `is_game_type_restricted` returns \
         false because mainline is one of the non-restricted values at src/toc.rs:294-302. \
         The mainline-only marker excludes this addon from Classic-flavor builds, where the \
         mail system uses different in-game mechanics"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_MailFrame declares `## AllowLoad: Game` — must auto-discover on the Game \
         screen only (mailboxes only exist in-world, never on glue screens)"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_MailFrame must NOT auto-discover on glue screen {screen:?} — \
             `## AllowLoad: Game` routes through src/toc.rs:308 which only matches \
             ScreenKind::Game"
        );
    }
}

#[test]
fn blizzard_mail_frame_toc_lists_three_files() {
    let toc = TocFile::from_file(&mail_frame_toc()).expect("Blizzard_MailFrame TOC parses");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        MAIL_FRAME_TOC_FILES,
        "TOC body lists exactly 3 files in load order — MailFrame.lua publishes globals + \
         mixins, MailFrame.xml defines the MailFrame / OpenMailFrame trees and inline scripts \
         that REFERENCE the globals from line 1, Localization.lua runs last to apply per-locale \
         frame anchor / width adjustments"
    );
}

#[test]
fn blizzard_mail_frame_directory_holds_four_entries() {
    let entries = std::fs::read_dir(mail_frame_dir())
        .expect("Blizzard_MailFrame directory reads")
        .count();
    assert_eq!(
        entries, 4,
        "Directory holds exactly 4 entries — Blizzard_MailFrame.toc + the 3 source files \
         (MailFrame.lua, MailFrame.xml, Localization.lua). No XSD copies, no per-locale \
         `_<locale>.toc` variants — every locale ships through the single Localization.lua \
         table dispatch"
    );
}

#[test]
fn blizzard_mail_frame_auto_discovered_on_game_screen_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_MailFrame");
    assert!(
        game_found,
        "Blizzard_MailFrame must be auto-discovered on the Game screen — the eager-load combo \
         (`## DefaultState: enabled` + `## AllowLoad: Game` + `## AllowLoadGameType: mainline`) \
         routes it into the eager `addons` set during the Game-screen discovery sweep, NOT the \
         lod_pool (no LoadOnDemand)"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_MailFrame");
        assert!(
            !found,
            "Blizzard_MailFrame must NOT be auto-discovered on glue screen {screen:?} — \
             `## AllowLoad: Game` excludes it from glue discovery sweeps. Glue screens have no \
             world / mailbox concept, so loading the mail UI there would waste memory and \
             register events that can never fire"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_mail_frame_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_MailFrame")
                || message.contains("MailFrame.lua")
                || message.contains("MailFrame.xml")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_MailFrame emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_is_addon_loaded_after_auto_discovery(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MailFrame')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_MailFrame') must return true after the eager \
         auto-discovery sweep — proves the mail addon registers with the loaded-set during \
         the standard Game-screen boot pipeline, no explicit load_addon call required"
    );
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_publishes_layout_constants(env: &WowLuaEnv) {

    for (name, expected) in MAIL_LAYOUT_CONSTANTS {
        let actual: i64 = env
            .eval(&format!("return {name}"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            actual, *expected,
            "{name} must equal {expected} — published at MailFrame.lua lines 1-10 as a \
             top-level numeric global. These constants drive grid layout (rows / columns / max \
             counts) for the inbox + send + open mail panels and feed both Lua dispatch loops \
             AND XML `id=\"N\"` declarations. Drifting any one value fragments the grid"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_publishes_send_mail_tab_list(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(SEND_MAIL_TAB_LIST)")
        .expect("SEND_MAIL_TAB_LIST type probe succeeds");
    assert_eq!(
        kind, "table",
        "SEND_MAIL_TAB_LIST must publish at `_G` as a table — declared at MailFrame.lua line \
         11 as the master tab-traversal index. Drives the EditBox-to-MoneyInputFrame focus \
         chain when the player presses Tab inside the SendMail panel"
    );

    let length: i64 = env
        .eval("return #SEND_MAIL_TAB_LIST")
        .expect("SEND_MAIL_TAB_LIST length probe succeeds");
    assert_eq!(
        length, 5,
        "SEND_MAIL_TAB_LIST must hold exactly 5 entries — recipient name, subject, body, gold \
         field, copper field. Index 4 + 5 are MoneyInputFrame children, the rest are top-level \
         EditBoxes"
    );

    for (index, expected) in SEND_MAIL_TAB_LIST_ENTRIES {
        let actual: String = env
            .eval(&format!("return SEND_MAIL_TAB_LIST[{index}]"))
            .unwrap_or_else(|err| panic!("SEND_MAIL_TAB_LIST[{index}] probe failed: {err}"));
        assert_eq!(
            actual, *expected,
            "SEND_MAIL_TAB_LIST[{index}] must equal {expected:?} — global frame-name string \
             that the focus-traversal helper resolves via `_G[name]` at runtime. Reordering \
             would break Tab-key flow inside the send panel"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_publishes_free_functions(env: &WowLuaEnv) {

    for name in MAIL_FREE_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} type probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{name} must publish at `_G` as a function — XML inline scripts in MailFrame.xml \
             reference these by bare name (e.g. `<OnClick function=\"OpenMail_Reply\"/>`), so \
             they MUST resolve at the point the parser binds the handler. Missing any one \
             would crash the addon at first interaction with the relevant button"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_publishes_open_all_mail_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(OpenAllMailMixin)")
        .expect("OpenAllMailMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "OpenAllMailMixin must publish at `_G` as a table — declared at MailFrame.lua line \
         1188. Consumed by the XML `mixin=\"OpenAllMailMixin\"` attribute on the OpenAllMail \
         button (MailFrame.xml line 431), which copies its methods onto the button instance \
         via the mixin contract"
    );

    for method in OPEN_ALL_MAIL_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(OpenAllMailMixin.{method})"))
            .unwrap_or_else(|err| panic!("OpenAllMailMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "OpenAllMailMixin.{method} must match the current event-driven inbox sweep \
             implementation, including mail/attachment skipping and failed-item tracking"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_named_frames_resolve_globally(env: &WowLuaEnv) {

    for name in NAMED_MAIL_FRAMES {
        let exists: bool = env
            .eval(&format!("return _G[{name:?}] ~= nil"))
            .unwrap_or_else(|err| panic!("{name} existence probe failed: {err}"));
        assert!(
            exists,
            "{name} must publish at `_G` after addon load — declared with `name=\"...\"` in \
             MailFrame.xml. Missing implies the XML parser dropped the element or the named \
             frame creation pipeline didn't register the global"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_inherits_button_frame_template(env: &WowLuaEnv) {

    let parent_name: String = env
        .eval("return MailFrame:GetParent():GetName()")
        .expect("MailFrame:GetParent():GetName() probe succeeds");
    assert_eq!(
        parent_name, "UIParent",
        "MailFrame must parent to UIParent — XML declares `parent=\"UIParent\"` on the \
         top-level Frame at MailFrame.xml:274. UIParent is the canonical in-game UI root that \
         scales / hides with the rest of the player UI"
    );

    let is_hidden: bool = env
        .eval("return not MailFrame:IsShown()")
        .expect("MailFrame:IsShown() probe succeeds");
    assert!(
        is_hidden,
        "MailFrame must be hidden by default — XML declares `hidden=\"true\"` at \
         MailFrame.xml:274. Mail UI only opens when the player interacts with a mailbox; a \
         visible-by-default mail frame would clutter the screen at every login"
    );

    let toplevel: bool = env
        .eval("return MailFrame:IsToplevel()")
        .expect("MailFrame:IsToplevel() probe succeeds");
    assert!(
        toplevel,
        "MailFrame must be toplevel — XML declares `toplevel=\"true\"` at MailFrame.xml:274. \
         Toplevel raises the frame above sibling UIParent panels on click, so clicking the \
         inbox surfaces it past adjacent panels (bag, character sheet, etc.)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_open_mail_anchored_to_inbox_frame(env: &WowLuaEnv) {

    let parent_name: String = env
        .eval("return OpenMailFrame:GetParent():GetName()")
        .expect("OpenMailFrame:GetParent():GetName() probe succeeds");
    assert_eq!(
        parent_name, "UIParent",
        "OpenMailFrame must parent to UIParent — XML declares `parent=\"UIParent\"` at \
         MailFrame.xml:877. OpenMailFrame is anchored RELATIVE to InboxFrame at MailFrame.xml \
         lines 878-880 (TOPLEFT to InboxFrame.TOPRIGHT) but is itself a UIParent child for \
         strata / scaling parity with MailFrame"
    );

    let is_hidden: bool = env
        .eval("return not OpenMailFrame:IsShown()")
        .expect("OpenMailFrame:IsShown() probe succeeds");
    assert!(
        is_hidden,
        "OpenMailFrame must be hidden by default — only opens when InboxFrame_OnClick fires \
         on a specific mail item. Visible-by-default would dock empty alongside the inbox"
    );

    let toplevel: bool = env
        .eval("return OpenMailFrame:IsToplevel()")
        .expect("OpenMailFrame:IsToplevel() probe succeeds");
    assert!(
        toplevel,
        "OpenMailFrame must be toplevel — clicking an opened letter must raise the panel \
         above siblings"
    );
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_tabs_register_with_panel_template(env: &WowLuaEnv) {

    let num_tabs: i64 = env
        .eval("return MailFrame.numTabs or -1")
        .expect("MailFrame.numTabs probe succeeds");
    assert_eq!(
        num_tabs, 2,
        "MailFrame.numTabs must equal 2 after MailFrame_OnLoad — set via \
         `PanelTemplates_SetNumTabs(self, 2)` at MailFrame.lua:26. Tab 1 is the Inbox, Tab 2 \
         is SendMail (hidden when the player is on a trial / starter-edition account, gated \
         by GameLimitedMode_IsActive at MailFrame.lua:46-49)"
    );

    let tab1_id: i64 = env
        .eval("return MailFrameTab1:GetID()")
        .expect("MailFrameTab1:GetID() probe succeeds");
    assert_eq!(
        tab1_id, 1,
        "MailFrameTab1:GetID() must return 1 — XML declares `id=\"1\"` at MailFrame.xml:840. \
         The id flows into PanelTemplates_SetTab(self, GetID()) in the OnClick handler"
    );

    let tab2_id: i64 = env
        .eval("return MailFrameTab2:GetID()")
        .expect("MailFrameTab2:GetID() probe succeeds");
    assert_eq!(
        tab2_id, 2,
        "MailFrameTab2:GetID() must return 2 — XML declares `id=\"2\"` at MailFrame.xml:850"
    );
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_inbox_paging_starts_at_one(env: &WowLuaEnv) {

    let page_num: i64 = env
        .eval("return InboxFrame.pageNum or -1")
        .expect("InboxFrame.pageNum probe succeeds");
    assert_eq!(
        page_num, 1,
        "InboxFrame.pageNum must equal 1 after MailFrame_OnLoad — set at MailFrame.lua:23 as \
         the initial page index. InboxNextPage / InboxPrevPage mutate this counter and \
         InboxFrame_Update re-reads it to render the correct 7-mail slice"
    );
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_send_mail_attachments_array_resolves(env: &WowLuaEnv) {

    let array_present: bool = env
        .eval("return type(SendMailFrame.SendMailAttachments) == 'table'")
        .expect("SendMailFrame.SendMailAttachments probe succeeds");
    assert!(
        array_present,
        "SendMailFrame.SendMailAttachments must publish as a table — XML declares 16 buttons \
         (SendMailAttachment1..SendMailAttachment16 at MailFrame.xml:697-712) inheriting the \
         virtual `SendMailAttachment` template (line 173) which carries \
         `parentArray=\"SendMailAttachments\"`. The parentArray contract appends each child \
         into the parent's table for index-based dispatch in SendMailFrame_Update"
    );

    let count: i64 = env
        .eval("return #SendMailFrame.SendMailAttachments")
        .expect("SendMailAttachments count probe succeeds");
    assert_eq!(
        count, 16,
        "SendMailFrame.SendMailAttachments must hold exactly 16 entries — one per \
         SendMailAttachmentN button declared in XML. SendMailFrame_Update iterates indices \
         1..ATTACHMENTS_MAX_SEND (12) so the 4 trailing slots are inert padding"
    );
}
}

prefork_full_ui_case! {
fn blizzard_mail_frame_inbox_max_size_local_constant_unexposed(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MAX_INBOX_SIZE)")
        .expect("MAX_INBOX_SIZE probe succeeds");
    assert_eq!(
        kind, "nil",
        "MAX_INBOX_SIZE must remain unexposed — declared at MailFrame.lua:18 as a `local` \
         (file-scoped) constant, NOT a global. Other addons that probe `_G.MAX_INBOX_SIZE` \
         would see nil; the 100-mail cap is private to InboxFrame_Update"
    );
}
}

#[test]
fn blizzard_mail_frame_localization_table_holds_thirteen_locales() {
    let raw = std::fs::read_to_string(mail_frame_dir().join("Localization.lua"))
        .expect("Blizzard_MailFrame Localization.lua reads");
    for code in [
        "deDE", "enGB", "enUS", "esES", "esMX", "frFR", "itIT", "koKR", "ptBR", "ptPT", "ruRU",
        "zhCN", "zhTW",
    ] {
        assert!(
            raw.contains(&format!("{code} = {{")),
            "Localization.lua must declare a key for `{code}` — the 13-locale table covers \
             every shipping retail UI locale so SetupLocalization can dispatch \
             localizeFrames(<currentLocale>) without nil-key panics. Locales without \
             callbacks (deDE / enUS / etc) ship empty tables, kept as explicit keys for the \
             dispatch contract"
        );
    }
}
