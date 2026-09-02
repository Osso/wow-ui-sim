use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn token_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_TokenUI")
}

fn token_ui_toc() -> PathBuf {
    token_ui_dir().join("Blizzard_TokenUI.toc")
}

const ALL_GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOKEN_UI_MIXINS: &[&str] = &[
    "TokenHeaderMixin",
    "TokenEntryMixin",
    "TokenEntryAccountWideIconMixin",
    "TokenSubHeaderMixin",
    "TokenSubHeaderToggleCollapseButtonMixin",
    "TokenFrameMixin",
    "TokenFramePopupMixin",
    "InactiveCurrencyCheckboxMixin",
    "BackpackCurrencyCheckboxMixin",
    "BackpackTokenFrameMixin",
    "BackpackTokenMixin",
];

const CURRENCY_TRANSFER_MIXINS: &[&str] = &[
    "CurrencyTransferSystemMixin",
    "CurrencyTransferToggleButtonMixin",
    "CurrencyTransferMenuMixin",
    "CurrencyTransferBalancePreviewMixin",
    "CurrencyTransferConfirmButtonMixin",
    "CurrencyTransferCancelButtonMixin",
    "CurrencyTransferAmountSelectorMixin",
    "CurrencyTransferAmountInputBoxMixin",
    "CurrencyTransferCostDisplayMixin",
    "CurrencyTransferSourceSelectorMixin",
    "CurrencyTransferLogMixin",
    "CurrencyTransferLogEntryMixin",
    "CurrencyTransferLogToggleButtonMixin",
];

const NAMED_TOPLEVEL_FRAMES: &[&str] = &[
    "TokenFrame",
    "CurrencyTransferLog",
    "TokenFramePopup",
    "CurrencyTransferMenu",
    "BackpackTokenFrame",
];

fn fresh_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
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
    let env = fresh_game_env();

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&token_ui_dir()).expect("TokenUI TOC resolves");
    assert_eq!(
        resolved,
        token_ui_toc(),
        "Bare TOC — no flavor suffix; in-game character-frame token \
         tab + currency-transfer dialog ships universally on mainline"
    );
}

#[test]
fn toc_has_only_title_directive_with_no_dependencies() {
    let toc = TocFile::from_file(&token_ui_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` — TokenUI is eager but the addon \
         attaches its TokenFrame to CharacterFrame which only exists \
         in-game, so `## AllowLoad` (absent) defaults to Game-only \
         which serves the same gating purpose"
    );
    assert!(
        toc.dependencies().is_empty(),
        "No `## Dependencies` / `## RequiredDep` — TokenUI is a \
         minimum-metadata TOC (Title-only); CurrencyTransferMenu \
         inherits from CallbackRegistryMixin which is provided by \
         FrameXML core, no addon-level dep needed. Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        !toc.is_game_type_restricted(),
        "AllowLoadGameType absent → not restricted (false)"
    );
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_absent_defaults_to_game_only_screen() {
    let toc = TocFile::from_file(&token_ui_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "AllowLoad absent → toc.rs:305-313 None branch defaults to \
         Game-only — TokenFrame parents to CharacterFrame which only \
         exists in-world"
    );

    for screen in ALL_GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "AllowLoad absent must EXCLUDE glue screen {screen:?} — \
             CharacterFrame and the in-game CharacterFrame token tab \
             do not exist on glue screens"
        );
    }
}

#[test]
fn toc_raw_bytes_pin_single_metadata_directive_and_five_body_files() {
    let raw = std::fs::read_to_string(token_ui_toc()).expect("TOC reads utf-8");

    let expected_lines = [
        "## Title: Blizzard_TokenUI",
        "Blizzard_CurrencyTransfer.lua",
        "Blizzard_CurrencyTransfer.xml",
        "Blizzard_TokenUI.lua",
        "Blizzard_TokenUI.xml",
        "Localization.lua",
    ];

    for directive in expected_lines {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin `{directive}` — minimum-viable shape: 1 \
             metadata directive (Title) + 5 body files (CurrencyTransfer \
             lua/xml first, then TokenUI lua/xml, then Localization)"
        );
    }

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## Dependencies"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## AllowLoad"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
}

#[test]
fn body_resolves_to_five_entries_in_declared_order() {
    let toc = TocFile::from_file(&token_ui_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = vec![
        "Blizzard_CurrencyTransfer.lua".to_string(),
        "Blizzard_CurrencyTransfer.xml".to_string(),
        "Blizzard_TokenUI.lua".to_string(),
        "Blizzard_TokenUI.xml".to_string(),
        "Localization.lua".to_string(),
    ];

    assert_eq!(
        body, expected,
        "Body must be 5 entries in declared order. \
         CurrencyTransfer-lua-then-xml is loaded BEFORE TokenUI-lua-then-xml \
         because TokenUI.xml line 179 has `<Frame name=\"CurrencyTransferLog\" \
         inherits=\"CurrencyTransferLogTemplate\">` which requires \
         CurrencyTransfer.xml's virtual template to already be \
         registered. Got: {body:?}"
    );
}

#[test]
fn present_in_game_screen_eager_discovery_only() {
    let game = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game.iter().any(|(name, _)| name == "Blizzard_TokenUI");
    assert!(
        in_game,
        "Blizzard_TokenUI must appear in Game eager discovery — eager \
         (no LoadOnDemand) + AllowLoad-absent defaults to Game-only"
    );

    for screen in ALL_GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_TokenUI");
        assert!(
            !found,
            "Blizzard_TokenUI must NOT appear in {screen:?} eager \
             discovery — AllowLoad-absent excludes glue screens"
        );
    }
}

#[test]
fn token_frame_load_ui_is_absent_and_wow_token_ui_owns_secure_token_flow() {
    let env = fresh_game_env();
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        if name == "Blizzard_TokenUI" {
            continue;
        }
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    let kind: String = env
        .eval("return type(TokenFrame_LoadUI)")
        .expect("TokenFrame_LoadUI probe");
    assert_eq!(
        kind, "nil",
        "Retail 12.1.0.69497 no longer publishes the legacy TokenFrame_LoadUI global"
    );

    let secure_toc_path = blizzard_ui_dir().join("Blizzard_WowTokenUI/Blizzard_WowTokenUI.toc");
    let secure_toc =
        TocFile::from_file(&secure_toc_path).expect("Blizzard_WowTokenUI secure TOC should parse");
    assert!(
        secure_toc.is_secure_env(),
        "Retail token redemption flow belongs to Blizzard_WowTokenUI, which declares \
         UseSecureEnvironment: 1"
    );
    assert_eq!(
        secure_toc.dependencies(),
        vec!["Blizzard_SharedXML".to_string()],
        "Current Blizzard_WowTokenUI declares Blizzard_SharedXML as its required dependency"
    );
    let secure_body: Vec<String> = secure_toc
        .files
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();
    assert_eq!(
        secure_body,
        vec![
            "Blizzard_WowTokenUI.xml".to_string(),
            "Blizzard_WowTokenUIInsecure.xml".to_string(),
        ],
        "Current secure token addon owns the token flow through its secure and insecure XML entrypoints"
    );
}

prefork_full_ui_case! {
fn explicit_load_publishes_eleven_token_ui_mixins(env: &WowLuaEnv) {

    for mixin in TOKEN_UI_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a table after eager Game-screen sweep \
             loads TokenUI — TokenHeaderMixin/EntryMixin/SubHeaderMixin \
             family powers the ScrollBoxList at TokenFrame; \
             FrameMixin/PopupMixin drive the main panel + filter popup; \
             InactiveCurrencyCheckboxMixin/BackpackCurrencyCheckboxMixin \
             drive the currency-show toggles"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_thirteen_currency_transfer_mixins(env: &WowLuaEnv) {

    for mixin in CURRENCY_TRANSFER_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a table after TokenUI loads — \
             CurrencyTransferSystemMixin is the base, ToggleButton + \
             Menu + ConfirmButton + CancelButton + AmountSelector + \
             AmountInputBox + CostDisplay + SourceSelector + \
             BalancePreview compose the transfer dialog; Log + \
             LogEntry + LogToggleButton drive the history panel"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_creates_five_named_toplevel_frames(env: &WowLuaEnv) {

    for frame_name in NAMED_TOPLEVEL_FRAMES {
        let exists: bool = env
            .eval(&format!("return _G['{frame_name}'] ~= nil"))
            .unwrap_or_else(|err| panic!("{frame_name} probe failed: {err}"));
        assert!(
            exists,
            "{frame_name} must exist as a named global after Game-screen \
             load — TokenFrame (parent=CharacterFrame, the currency tab); \
             CurrencyTransferLog (parent=TokenFrame, transfer history); \
             TokenFramePopup (parent=TokenFrame, filter dropdown); \
             CurrencyTransferMenu (toplevel, parent=UIParent, the \
             transfer dialog); BackpackTokenFrame (created by \
             ContainerFrameSettingsManager.OnAddonLoaded ADDON_LOADED \
             callback at ContainerFrame.lua:2173-2178 when \
             addonName==\"Blizzard_TokenUI\")"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_TokenUI")
                || e.contains("Blizzard_CurrencyTransfer")
                || e.contains("TokenFrame")
                || e.contains("CurrencyTransfer")
        })
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Loading Blizzard_TokenUI as part of the eager Game sweep must \
         emit zero addon-specific errors. Found {}: {:#?}",
        addon_specific.len(),
        addon_specific
    );
}
}

prefork_full_ui_case! {
fn token_ui_is_addon_loaded_after_game_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_TokenUI')")
        .expect("IsAddOnLoaded probe");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded must return true after the Game-screen \
         eager sweep — TokenUI has no LoadOnDemand and \
         AllowLoad-absent defaults to Game-only, so the eager \
         discovery loop loads it directly"
    );
}
}
