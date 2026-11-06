use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn secure_transfer_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SecureTransferUI")
}

fn secure_transfer_toc() -> PathBuf {
    secure_transfer_dir().join("Blizzard_SecureTransferUI.toc")
}

const TOC_FILES: &[&str] = &["Blizzard_SecureTransferUI.xml"];

const PUBLIC_GLOBAL_FUNCTIONS: &[&str] = &[
    "GetSecureMoneyString",
    "GetSecureTradeWarningString",
    "SecureTransferDialog_DelayedAccept",
    "SecureTransferDialog_TimerOnAccept",
    "SecureTransferDialog_Show",
    "SecureTransferDialog_OnLoad",
    "SecureTransferDialog_OnEvent",
    "SecureTransferDialog_OnShow",
    "SecureTransferDialog_OnHide",
    "SecureTransferDialogButton_OnClick",
];

const REGISTERED_EVENTS: &[&str] = &[
    "SECURE_TRANSFER_CONFIRM_TRADE_ACCEPT",
    "SECURE_TRANSFER_CONFIRM_SEND_MAIL",
    "SECURE_TRANSFER_CONFIRM_HOUSING_PURCHASE",
    "SECURE_TRANSFER_HOUSING_CURRENCY_PURCHASE_CONFIRMATION",
    "SECURE_TRANSFER_CANCEL",
    "BULK_PURCHASE_RESULT_RECEIVED",
];

const DIALOG_CHILD_KEYS: &[&str] = &[
    "Text",
    "MoneyLabel",
    "WarningText",
    "CoverFrame",
    "Border",
    "Button1",
    "Button2",
    "DarkOverlay",
    "Spinner",
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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved =
        find_toc_file(&secure_transfer_dir()).expect("Blizzard_SecureTransferUI TOC resolves");
    assert_eq!(
        resolved,
        secure_transfer_toc(),
        "Blizzard_SecureTransferUI ships exactly one bare TOC. The trade/mail/housing \
         confirmation pipeline must be flavor-agnostic — same secure-env contract on \
         every flavor"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_declares_eager_both_with_secure_env_and_required_dep_singular() {
    let toc =
        TocFile::from_file(&secure_transfer_toc()).expect("Blizzard_SecureTransferUI TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare LoadOnDemand — eager load required so the dialog \
         pre-registers SECURE_TRANSFER_* and BULK_PURCHASE_RESULT_RECEIVED events \
         at OnLoad before the player initiates any trade or mail send"
    );
    assert!(!toc.is_load_first());

    assert!(
        toc.is_secure_env(),
        "TOC must declare `## UseSecureEnvironment: 1` — every Lua chunk in this \
         addon loads into the secure Lua environment. Critical because \
         `SecureTransferDialog` displays the gold/item/Hearthsteel-currency transfer \
         confirmation: addon code MUST NOT be able to monkey-patch the dialog's \
         Show flow, the C_SecureTransfer C-API call sites, or the Button1/Button2 \
         OnClick handlers — that would let a malicious addon silently approve \
         money transfers on the player's behalf"
    );

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "`## AllowLoad: Both` must enable {screen:?} — `SecureTransferDialog_OnHide` \
             explicitly checks `if not C_Glue.IsOnGlueScreen() then \
             SecureTransferOutbound.UpdateSendMailButton() end` so the same dialog \
             code path runs on glue screens (where SendMail integration is skipped) \
             and in-game"
        );
    }

    let deps = toc.dependencies();
    let expected_deps: Vec<String> = if toc_layout::unified_toc_tree() {
        vec![
            "Blizzard_SharedXML".to_string(),
            "Blizzard_CatalogShopSharedUtil".to_string(),
        ]
    } else {
        vec!["Blizzard_SharedXML".to_string()]
    };
    assert_eq!(
        deps, expected_deps,
        "TOC must declare its `## RequiredDep:` list — note the SINGULAR \
         `RequiredDep` key (not `RequiredDeps` and not `Dependencies`); the \
         simulator's `dependencies()` accessor falls through `RequiredDep` -> \
         `Dependencies` -> `RequiredDeps` so all three resolve to the same list. \
         SharedXML provides SecureDialogBorderTemplate (used by the Border child) \
         and the FrameUtil / CreateAtlasMarkup helpers; the unified PTR TOC also \
         requires Blizzard_CatalogShopSharedUtil"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_declares_metadata_in_raw_bytes_with_underscored_title_form() {
    let raw = std::fs::read_to_string(secure_transfer_toc())
        .expect("Blizzard_SecureTransferUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_SecureTransferUI"),
        "TOC must declare `## Title: Blizzard_SecureTransferUI` — note the \
         underscored form matching the folder name verbatim, NOT a pretty-printed \
         space-separated form like 'Blizzard Secure Transfer UI'. Carried over from \
         Blizzard source"
    );
    assert!(raw.contains("## AllowLoad: Both"));
    assert!(raw.contains("## RequiredDep: Blizzard_SharedXML"));
    assert!(raw.contains("## UseSecureEnvironment: 1"));
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:` — the secure-env trade/mail \
         confirmation MUST always be enabled; there is no user-facing toggle"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — relies on implicit Blizzard ownership"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_lists_only_xml_with_two_lua_files_loaded_via_script_directives() {
    let toc =
        TocFile::from_file(&secure_transfer_toc()).expect("Blizzard_SecureTransferUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, TOC_FILES,
        "TOC body must list exactly 1 file: Blizzard_SecureTransferUI.xml. The two \
         Lua files (Blizzard_SecureTransferUIOutbound.lua + \
         Blizzard_SecureTransferUI.lua) load via `<Script file=\"...\"/>` directives \
         embedded at the top of the XML root, dispatched through \
         `process_include` / `process_script` at src/loader/xml_file.rs. The XML-driven \
         load order is significant: Outbound loads first, captures __secureenv, swaps \
         its chunk to the global environment for outbound method definitions, then \
         exports the namespace through the saved secure-env reference so the secure \
         main file can call SecureTransferOutbound.*; the main file loads second"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn appears_on_every_screen_eager_discovery() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_SecureTransferUI");
        assert!(
            found,
            "Blizzard_SecureTransferUI must auto-discover on screen {screen:?} — \
             eager (no LoadOnDemand) AND `## AllowLoad: Both` makes the addon \
             part of every screen's eager set"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn root_directory_holds_two_lua_one_xml_one_toc() {
    let dir = secure_transfer_dir();
    assert!(dir.join("Blizzard_SecureTransferUI.lua").is_file());
    assert!(dir.join("Blizzard_SecureTransferUIOutbound.lua").is_file());
    assert!(dir.join("Blizzard_SecureTransferUI.xml").is_file());
    assert!(dir.join("Blizzard_SecureTransferUI.toc").is_file());

    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("read addon dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        4,
        "Blizzard_SecureTransferUI directory must contain exactly 4 entries \
         (2 lua + 1 xml + 1 toc), got {entries:?}. The Outbound.lua file is the \
         secure-env exit door for calls into the global Lua environment \
         (SendMailFrame_EnableSendMailButton, GetAppropriateTopLevelParent, etc.)"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn xml_wraps_frames_in_scoped_modifier_with_forbidden_attribute() {
    let xml_path = secure_transfer_dir().join("Blizzard_SecureTransferUI.xml");
    let raw = std::fs::read_to_string(&xml_path).expect("XML reads utf-8");
    assert!(
        raw.contains("<ScopedModifier forbidden=\"true\">"),
        "XML must wrap both the SecureTransferButtonTemplate and the \
         SecureTransferDialog frame in `<ScopedModifier forbidden=\"true\">` — \
         this directive (parsed by xml_file.rs:103-121 `process_scoped_modifier`) \
         flips `loading_forbidden = true` for every child frame, applying the \
         `IsForbidden` flag at frame creation time. Forbidden frames cannot be \
         enumerated or interacted with by addon code via standard frame iteration \
         APIs, isolating the secure transfer dialog from addon-driven UI \
         rearrangement"
    );
    assert!(
        raw.contains("<Script file=\"Blizzard_SecureTransferUIOutbound.lua\"/>"),
        "XML must declare the Outbound.lua Script directive FIRST so the global-env \
         exit door is established before the secure-env main file calls it"
    );
    assert!(raw.contains("<Script file=\"Blizzard_SecureTransferUI.lua\"/>"));
}
}

prefork_full_ui_case! {
fn loads_without_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_SecureTransferUI")
                || message.contains("SecureTransferDialog")
                || message.contains("SecureTransferOutbound")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_SecureTransferUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SecureTransferUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_SecureTransferUI') must return true after \
         the eager Game-screen sweep"
    );
}
}

prefork_full_ui_case! {
fn publishes_ten_dialog_global_functions_into_secure_environment(env: &WowLuaEnv) {

    for func in PUBLIC_GLOBAL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(__secureenv.{func})"))
            .unwrap_or_else(|err| panic!("type(__secureenv.{func}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "__secureenv.{func} must be a function — the addon publishes into the \
             SECURE environment, not `_G`. `## UseSecureEnvironment: 1` swaps each \
             addon-loaded function's fenv to the registry-stored `__secureenv` \
             table (see `mark_secure_state` at \
             src/lua_api/globals/security/secure_env.rs:82), so a top-level \
             `function GetSecureMoneyString(...)` writes to \
             `__secureenv.GetSecureMoneyString` rather than `_G.GetSecureMoneyString`. \
             The addon publishes 10 functions: \
             GetSecureMoneyString (formats copper amount with optional thousand \
             separators using GOLD/SILVER/COPPER_AMOUNT_TEXTURE format strings, \
             colorblind-mode aware via GetCVar('colorblindMode')); \
             GetSecureTradeWarningString (formats TRADE_WARNING_CHANGED_OFFER \
             when C_SecureTransfer.ShouldShowTradeOfferWarning() returns true); \
             SecureTransferDialog_DelayedAccept (1-second Button1 disable); \
             SecureTransferDialog_TimerOnAccept (3-second countdown ticker on \
             Button1 — visible delay before accept becomes clickable, anti-fatigue \
             safeguard for trade confirmations); SecureTransferDialog_Show (the \
             central entry point that takes a registered key, looks up the dialog \
             entry from SECURE_TRANSFER_DIALOGS, formats text with vararg args, \
             positions over focused frame or center, configures Button1/Button2, \
             SetParentMaintainRenderLayering to GetAppropriateTopLevelParent, \
             ShowFrame); 5 script handlers (SecureTransferDialog_OnLoad / OnEvent / \
             OnShow / OnHide / SecureTransferDialogButton_OnClick) wired by the \
             XML's <Scripts> bindings"
        );
    }
}
}

prefork_full_ui_case! {
fn secure_transfer_dialog_publishes_as_named_global_hidden_by_default(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(_G.SecureTransferDialog)")
        .expect("SecureTransferDialog global probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.SecureTransferDialog must publish as a table — XML declares the named \
         non-virtual frame at root scope inside the forbidden ScopedModifier"
    );

    let shown: bool = env
        .eval("return SecureTransferDialog:IsShown()")
        .expect("IsShown probe succeeds");
    assert!(
        !shown,
        "SecureTransferDialog must start hidden — XML declares `hidden=\"true\"`. \
         The dialog only shows when SecureTransferDialog_Show is called from \
         OnEvent in response to SECURE_TRANSFER_CONFIRM_* server events"
    );

    let strata: String = env
        .eval("return SecureTransferDialog:GetFrameStrata()")
        .expect("GetFrameStrata probe succeeds");
    assert_eq!(
        strata, "DIALOG",
        "SecureTransferDialog must default to frameStrata=DIALOG — XML attribute. \
         Some entries override at Show time (CONFIRM_HOUSING_PURCHASE / \
         SLOW_HOUSING_PURCHASE / HOUSING_PURCHASE_FAILURE* use FULLSCREEN_DIALOG; \
         START_HOUSING_VC_PURCHASE uses TOOLTIP to layer above the \
         CatalogShop TopUpFrame which itself uses FULLSCREEN_DIALOG)"
    );
}
}

prefork_full_ui_case! {
fn secure_transfer_dialog_publishes_nine_named_child_keys(env: &WowLuaEnv) {

    for child_key in DIALOG_CHILD_KEYS {
        let child_kind: String = env
            .eval(&format!("return type(SecureTransferDialog.{child_key})"))
            .unwrap_or_else(|err| {
                panic!("type(SecureTransferDialog.{child_key}) probe failed: {err}")
            });
        assert_eq!(
            child_kind, "table",
            "SecureTransferDialog.{child_key} must publish as a table — the XML \
             body declares 9 parentKey children: 3 FontStrings (Text=GameFontHighlight \
             dialog body, MoneyLabel=NumberFontNormal hidden by default for money \
             dialogs, WarningText=GameFontRed for trade-changed-offer warnings), \
             CoverFrame (full-screen black 50% overlay with nop OnKeyDown/OnKeyUp \
             to capture keypresses during fullScreenCover dialogs), Border (inherits \
             SecureDialogBorderTemplate), Button1/Button2 (SecureTransferButtonTemplate \
             with id=1/id=2 used by SecureTransferDialogButton_OnClick to dispatch \
             accept vs cancel), DarkOverlay (frameLevel=500 black 80% overlay \
             behind the spinner during waitForEvent dialogs), Spinner (frameLevel=1000 \
             SpinnerTemplate shown while waiting for BULK_PURCHASE_RESULT_RECEIVED)"
        );
    }
}
}

prefork_full_ui_case! {
fn secure_transfer_dialog_registers_six_events_at_onload(env: &WowLuaEnv) {

    for event in REGISTERED_EVENTS {
        let registered: bool = env
            .eval(&format!(
                "return SecureTransferDialog:IsEventRegistered('{event}')"
            ))
            .unwrap_or_else(|err| panic!("IsEventRegistered('{event}') probe failed: {err}"));
        assert!(
            registered,
            "SecureTransferDialog must register {event} after OnLoad — \
             `SecureTransferDialog_OnLoad` registers exactly 6 events: 4 server-side \
             SECURE_TRANSFER_CONFIRM_* push events (TRADE_ACCEPT, SEND_MAIL, \
             HOUSING_PURCHASE, HOUSING_CURRENCY_PURCHASE_CONFIRMATION), \
             SECURE_TRANSFER_CANCEL (server cancellation broadcast), and \
             BULK_PURCHASE_RESULT_RECEIVED (housing-purchase result). Missing any \
             of these would orphan the dialog: the corresponding flow would never \
             trigger the confirmation popup or never resolve from waiting state"
        );
    }
}
}

prefork_full_ui_case! {
fn secure_transfer_outbound_publishes_only_in_secure_env_with_five_methods(env: &WowLuaEnv) {

    let global_kind: String = env
        .eval("return type(rawget(_G, 'SecureTransferOutbound'))")
        .expect("global SecureTransferOutbound probe succeeds");
    assert_eq!(
        global_kind, "nil",
        "rawget(_G, 'SecureTransferOutbound') must be nil — the outbound namespace is \
         deliberately exported into the secure environment, not the public global table"
    );

    let secure_kind: String = env
        .eval("return type(rawget(__secureenv, 'SecureTransferOutbound'))")
        .expect("secure SecureTransferOutbound probe succeeds");
    assert_eq!(
        secure_kind, "table",
        "rawget(__secureenv, 'SecureTransferOutbound') must be a table — Outbound.lua \
         saves the current secure environment before SwapToGlobalEnvironment, then \
         exports its local namespace through that saved secure-env reference"
    );

    for method in [
        "UpdateSendMailButton",
        "GetAppropriateTopLevelParent",
        "GetCatalogShopTopUpFrame",
        "GetHearthsteelVirtualCurrencyCode",
        "HideCatalogShopTopUpFrame",
    ] {
        let method_kind: String = env
            .eval(&format!(
                "return type(__secureenv.SecureTransferOutbound.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("type(__secureenv.SecureTransferOutbound.{method}) probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "__secureenv.SecureTransferOutbound.{method} must be a function — the \
             namespace exposes 5 callbacks secure transfer code uses to reach global \
             UI behavior: UpdateSendMailButton; GetAppropriateTopLevelParent; \
             GetCatalogShopTopUpFrame; GetHearthsteelVirtualCurrencyCode; and \
             HideCatalogShopTopUpFrame"
        );
    }
}
}

prefork_full_ui_case! {
fn get_secure_money_string_formats_copper_into_gold_silver_copper_segments(env: &WowLuaEnv) {

    let formatted: String = env
        .eval("return __secureenv.GetSecureMoneyString(123456)")
        .expect("GetSecureMoneyString probe succeeds");
    assert!(
        !formatted.is_empty(),
        "__secureenv.GetSecureMoneyString(123456) must return a non-empty string — \
         123456 copper = 12 gold 34 silver 56 copper, so the result must contain \
         three segments. Exact content depends on whether colorblind mode is \
         active (texture-format vs symbol-format), but length must be > 0. \
         `## UseSecureEnvironment: 1` puts this addon's globals in `__secureenv`, \
         not `_G` (see secure_env.rs:82)"
    );

    let zero_money: String = env
        .eval("return __secureenv.GetSecureMoneyString(0)")
        .expect("GetSecureMoneyString(0) probe succeeds");
    assert!(
        !zero_money.is_empty(),
        "__secureenv.GetSecureMoneyString(0) must return a non-empty string — the \
         function's `if copper > 0 or moneyString == \"\"` clause guarantees that \
         even zero-money inputs produce a copper segment. Without this fallback, \
         calling code would have to special-case the zero amount"
    );
}
}

prefork_full_ui_case! {
fn secure_transfer_dialogs_registry_includes_eight_keys_after_copytable_extension(env: &WowLuaEnv) {

    let exists: bool = env
        .eval(
            "return SecureTransferDialog ~= nil and \
             type(__secureenv.SecureTransferDialog_Show) == 'function'",
        )
        .expect("SecureTransferDialog_Show probe succeeds");
    assert!(
        exists,
        "__secureenv.SecureTransferDialog_Show must be the live entry point — the \
         module-local SECURE_TRANSFER_DIALOGS registry holds 8 keys after the line \
         190-191 `CopyTable` extension: CONFIRM_TRADE / SEND_MONEY_TO_STRANGER / \
         SEND_ITEMS_TO_STRANGER / CONFIRM_HOUSING_PURCHASE / \
         CONFIRM_HOUSING_PURCHASE_SINGLE_ITEM (CopyTable clone of \
         CONFIRM_HOUSING_PURCHASE with overridden text) / SLOW_HOUSING_PURCHASE / \
         HOUSING_PURCHASE_FAILURE / HOUSING_PURCHASE_FAILURE_INSUFFICIENT_FUNDS / \
         START_HOUSING_VC_PURCHASE. Calling Show with an unregistered key is a \
         no-op (line 210: `if not SECURE_TRANSFER_DIALOGS[which] then return end`). \
         The dialog frame itself lives in `_G` because frame registration \
         bypasses the secure-env fenv swap"
    );

    env.exec("__secureenv.SecureTransferDialog_Show('NONEXISTENT_KEY')")
        .expect("Show with bad key probe succeeds");

    let still_hidden: bool = env
        .eval("return SecureTransferDialog:IsShown()")
        .expect("post-bad-key IsShown probe succeeds");
    assert!(
        !still_hidden,
        "SecureTransferDialog must remain hidden after Show('NONEXISTENT_KEY') — \
         the early-return at line 210 protects against typos and keeps the dialog \
         from being shown without a registered configuration"
    );
}
}
