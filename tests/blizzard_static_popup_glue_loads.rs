use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn static_popup_glue_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_StaticPopup_Glue")
}

fn static_popup_glue_toc() -> PathBuf {
    static_popup_glue_dir().join("Blizzard_StaticPopup_Glue.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const GLUE_DIALOG_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "Init",
    "SetBackground",
    "GetEditBox",
    "GetButton1",
    "GetButton2",
    "GetButton3",
    "GetButton",
    "GetTextFontString",
    "Resize",
    "SetText",
    "SetFormattedText",
    "ClearHtmlText",
    "SetHtmlText",
    "GetText",
    "GetHtmlText",
    "OnUpdate",
    "OnShow",
    "OnHide",
    "OnHyperlinkClick",
    "OnHyperlinkEnter",
    "OnHyperlinkLeave",
    "SetupStartDelay",
];

const SHARED_DIALOG_KEYS: &[&str] = &[
    "OKAY",
    "PAID_SERVICE_IN_PROGRESS",
    "OKAY_HTML_MUST_ACCEPT",
    "OKAY_MUST_ACCEPT",
    "CANCEL",
    "OKAY_HTML",
    "OKAY_WITH_URL",
    "OKAY_WITH_URL_INDEX",
    "OKAY_WITH_GENERIC_URL",
    "ERROR_CINEMATIC",
    "CLIENT_RESTART_ALERT",
    "RETRIEVING_CHARACTER_LIST",
    "REALM_LIST_IN_PROGRESS",
    "REALM_IS_FULL",
    "CONFIRM_PAID_SERVICE",
    "CONFIRM_VAS_FACTION_CHANGE",
];

const MAINLINE_DIALOG_KEYS: &[&str] = &[
    "ERROR_CONNECT_TO_EVENT_REALM_FAILED",
    "RPE_BOOST_ALLIED_RACE_HERITAGE_ARMOR_WARNING",
    "EVOKER_NEW_PLAYER_WARNING",
    "EVOKER_NEW_PLAYER_CONFIRMATION",
    "ADD_FRIEND",
    "CONFIRM_REMOVE_BN_FRIEND",
    "SWAPPING_ENVIRONMENT",
    "ACCOUNT_CONVERSION_DISPLAY",
    "CREATE_CHARACTER_REALM_CONFIRMATION",
    "ACCOUNT_STORE_BEGIN_PURCHASE_OR_REFUND",
    "CONFIRM_DELETE_CHARACTER_GROUP",
];

fn fresh_env(screen: ScreenKind) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(screen);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_ui_for(screen: ScreenKind) -> WowLuaEnv {
    let env = fresh_env(screen);

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    env
}

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&static_popup_glue_dir()).expect("StaticPopup_Glue TOC resolves");
    assert_eq!(
        resolved,
        static_popup_glue_toc(),
        "Bare TOC: per-flavor selection inline via [Family] placeholder + \
         [AllowLoadGameType] annotations (toc.rs:144-146)"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn dependencies_chain_pulls_three_blizzard_addons() {
    let toc = TocFile::from_file(&static_popup_glue_toc()).expect("TOC parses");

    let expected_deps = vec![
        "Blizzard_StaticPopup".to_string(),
        "Blizzard_AutoComplete".to_string(),
        "Blizzard_AccessibilityTemplates".to_string(),
    ];

    assert_eq!(
        toc.dependencies(),
        expected_deps,
        "Plural `## Dependencies:` 3-dep chain: StaticPopup (dispatcher), \
         AutoComplete (EditBox template), AccessibilityTemplates \
         (UserScaledFrameTemplate). Got: {:?}",
        toc.dependencies()
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_glue_resolves_to_three_glue_screens() {
    let toc = TocFile::from_file(&static_popup_glue_toc()).expect("TOC parses");

    assert!(
        !toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: glue` excludes Game — game-side dialogs live in \
         Blizzard_StaticPopup_Game"
    );
    for screen in GLUE_SCREENS {
        assert!(
            toc.allows_screen(*screen),
            "`## AllowLoad: glue` must allow {screen:?} (toc.rs:309-313)"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn no_addon_level_game_type_restriction() {
    let toc = TocFile::from_file(&static_popup_glue_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "No addon-level `## AllowLoadGameType` — body uses inline \
         `[AllowLoadGameType mainline]` for the Announcement files only"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_is_eager_with_no_secure_env_or_saved_vars() {
    let toc = TocFile::from_file(&static_popup_glue_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "Eager: glue dialog surface must be ready before login flow runs"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.optional_deps().is_empty());
    assert!(toc.default_enabled());
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_raw_bytes_pin_three_metadata_directives() {
    let raw = std::fs::read_to_string(static_popup_glue_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_StaticPopup_Glue",
        "## Dependencies: Blizzard_StaticPopup, Blizzard_AutoComplete, Blizzard_AccessibilityTemplates",
        "## AllowLoad: glue",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin `{directive}` — 3 metadata lines + 9 body \
             entries (some filtered by AllowLoadGameType); each is load-bearing"
        );
    }

    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## AllowLoadGameType:"));
    assert!(!raw.contains("## UseSecureEnvironment"));
}
}

post_wrath_ui_fixture! {
#[test]
fn body_selects_family_and_game_entries_for_active_profile() {
    let toc = TocFile::from_file(&static_popup_glue_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected: &[&str] = if cfg!(feature = "client-wowforever") {
        // Forever's TOC includes [Game] Camelot/GlueDialogDefs.lua.
        &[
            "GlueDialogDefs.lua",
            "Mainline/GlueDialogDefs.lua",
            "Camelot/GlueDialogDefs.lua",
            "GlueDialog.lua",
            "Mainline/GlueDialog.lua",
            "GlueDialogUserScaledTemplates.lua",
            "GlueDialog.xml",
            "Mainline/GlueAnnouncementDialog.lua",
            "Mainline/GlueAnnouncementDialog.xml",
        ]
    } else {
        &[
            "GlueDialogDefs.lua",
            "Mainline/GlueDialogDefs.lua",
            "GlueDialog.lua",
            "Mainline/GlueDialog.lua",
            "GlueDialogUserScaledTemplates.lua",
            "GlueDialog.xml",
            "Mainline/GlueAnnouncementDialog.lua",
            "Mainline/GlueAnnouncementDialog.xml",
        ]
    };

    assert_eq!(
        body.len(),
        expected.len(),
        "selected StaticPopup_Glue TOC entries: {body:?}"
    );

    for (i, want) in expected.iter().enumerate() {
        assert_eq!(
            &body[i], want,
            "Body entry {i}: expected {want}, got {}",
            body[i]
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn body_orders_defs_before_logic_before_xml() {
    let toc = TocFile::from_file(&static_popup_glue_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let defs_idx = body
        .iter()
        .position(|f| f == "GlueDialogDefs.lua")
        .expect("Defs.lua present");
    let dialog_idx = body
        .iter()
        .position(|f| f == "GlueDialog.lua")
        .expect("GlueDialog.lua present");
    let xml_idx = body
        .iter()
        .position(|f| f == "GlueDialog.xml")
        .expect("GlueDialog.xml present");

    assert!(
        defs_idx < dialog_idx,
        "Defs (data) before GlueDialog.lua (mixin logic)"
    );
    assert!(
        dialog_idx < xml_idx,
        "GlueDialog.lua before GlueDialog.xml — XML mixin=\"GlueDialogMixin\" \
         resolves at template-registration time"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn appears_in_eager_discovery_on_glue_screens_only() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_StaticPopup_Glue");
    assert!(
        !game_found,
        "`## AllowLoad: glue` must exclude StaticPopup_Glue from Game sweep \
         — game dialogs are provided by Blizzard_StaticPopup_Game"
    );

    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&ui, *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_StaticPopup_Glue");
        assert!(
            found,
            "`## AllowLoad: glue` must surface StaticPopup_Glue on \
             {screen:?} eager discovery"
        );
    }
}
}

#[test]
fn full_login_load_emits_no_addon_specific_lua_errors() {
    let env = load_full_ui_for(ScreenKind::Login);

    let errors = env.state().borrow().lua_errors.clone();
    let needles = [
        "GlueDialog.lua",
        "GlueDialogDefs.lua",
        "GlueDialog.xml",
        "GlueDialogMixin",
        "GlueDialogButtonMixin",
        "GlueAnnouncementDialogMixin",
        "Blizzard_StaticPopup_Glue",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Full Login-screen load must emit zero StaticPopup_Glue-specific \
         Lua errors. Found {} matching errors: {:#?}",
        matched.len(),
        matched
    );
}

post_wrath_ui_fixture! {
#[test]
fn is_addon_loaded_reports_true_on_each_glue_screen() {
    for screen in GLUE_SCREENS {
        let env = load_full_ui_for(*screen);
        let loaded: bool = env
            .eval("return C_AddOns.IsAddOnLoaded('Blizzard_StaticPopup_Glue')")
            .expect("IsAddOnLoaded query");
        assert!(
            loaded,
            "After {screen:?} eager sweep, IsAddOnLoaded must report true"
        );
    }
}
}

#[test]
fn is_addon_loaded_reports_false_on_game_screen() {
    let env = load_full_ui_for(ScreenKind::Game);
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_StaticPopup_Glue')")
        .expect("IsAddOnLoaded query");
    assert!(
        !loaded,
        "Game-screen sweep must NOT load StaticPopup_Glue (AllowLoad: glue)"
    );
}

post_wrath_ui_fixture! {
#[test]
fn glue_dialog_mixin_publishes_with_full_method_surface() {
    let env = load_full_ui_for(ScreenKind::Login);

    let kind: String = env
        .eval("return type(GlueDialogMixin)")
        .expect("GlueDialogMixin probe");
    assert_eq!(
        kind, "table",
        "GlueDialogMixin = table — published at line 1 of GlueDialog.lua"
    );

    for method in GLUE_DIALOG_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(GlueDialogMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("GlueDialogMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GlueDialogMixin.{method} = function — full surface covers \
             OnLoad/Init/SetBackground + 5 button accessors + Resize + \
             4 text setters/getters + lifecycle scripts + SetupStartDelay"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn glue_dialog_button_mixin_publishes_on_text_scale_updated() {
    let env = load_full_ui_for(ScreenKind::Login);

    let kind: String = env
        .eval("return type(GlueDialogButtonMixin)")
        .expect("GlueDialogButtonMixin probe");
    assert_eq!(
        kind, "table",
        "GlueDialogButtonMixin = table — GlueDialogUserScaledTemplates.lua"
    );

    let method_kind: String = env
        .eval("return type(GlueDialogButtonMixin['OnTextScaleUpdated'])")
        .expect("OnTextScaleUpdated probe");
    assert_eq!(
        method_kind, "function",
        "GlueDialogButtonMixin.OnTextScaleUpdated = function — width/height \
         scale callback consumed by TextSizeManager"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn glue_announcement_dialog_mixin_publishes_with_two_methods() {
    let env = load_full_ui_for(ScreenKind::Login);

    let kind: String = env
        .eval("return type(GlueAnnouncementDialogMixin)")
        .expect("GlueAnnouncementDialogMixin probe");
    assert_eq!(
        kind, "table",
        "GlueAnnouncementDialogMixin = table — Mainline/\
         GlueAnnouncementDialog.lua mainline-gated via inline annotation"
    );

    for method in ["OnShow", "OnCloseClick"] {
        let kind: String = env
            .eval(&format!(
                "return type(GlueAnnouncementDialogMixin['{method}'])"
            ))
            .unwrap_or_else(|err| panic!("GlueAnnouncementDialogMixin.{method}: {err}"));
        assert_eq!(
            kind, "function",
            "GlueAnnouncementDialogMixin.{method} = function — delegates \
             to BaseNineSliceDialogMixin.{method}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn glue_dialog_background_top_atlas_global_publishes() {
    let env = load_full_ui_for(ScreenKind::Login);

    let kind: String = env
        .eval("return type(GlueDialogBackgroundTop)")
        .expect("GlueDialogBackgroundTop probe");
    assert_eq!(
        kind, "string",
        "GlueDialogBackgroundTop = string — Mainline/GlueDialog.lua \
         publishes the atlas name consumed by GlueDialogMixin:OnLoad"
    );

    let value: String = env
        .eval("return GlueDialogBackgroundTop")
        .expect("GlueDialogBackgroundTop value");
    assert_eq!(
        value, "UI-DiamondDialogBox-Border",
        "Mainline atlas name pinned"
    );
}
}

#[test]
fn shared_glue_dialog_definitions_seed_into_dispatcher() {
    let env = load_full_ui_for(ScreenKind::Login);

    for key in SHARED_DIALOG_KEYS {
        let kind: String = env
            .eval(&format!("return type(StaticPopupDialogs['{key}'])"))
            .unwrap_or_else(|err| panic!("StaticPopupDialogs[{key}] probe: {err}"));
        assert_eq!(
            kind, "table",
            "StaticPopupDialogs['{key}'] seeded — GlueDialogDefs.lua \
             injects ~28 shared dialog defs covering OKAY/CANCEL family, \
             URL helpers, realm-list flow, paid services, character boost"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn mainline_glue_dialog_definitions_seed_into_dispatcher() {
    let env = load_full_ui_for(ScreenKind::Login);

    for key in MAINLINE_DIALOG_KEYS {
        let kind: String = env
            .eval(&format!("return type(StaticPopupDialogs['{key}'])"))
            .unwrap_or_else(|err| panic!("StaticPopupDialogs[{key}] probe: {err}"));
        assert_eq!(
            kind, "table",
            "StaticPopupDialogs['{key}'] seeded by Mainline/\
             GlueDialogDefs.lua — covers Plunderstorm error, evoker \
             warnings, BN friend mgmt, swapping environment, account \
             store, character group deletion"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn mainline_realm_is_full_on_cancel_overrides_in_place() {
    let env = load_full_ui_for(ScreenKind::Login);

    let kind: String = env
        .eval("return type(StaticPopupDialogs['REALM_IS_FULL'].OnCancel)")
        .expect("REALM_IS_FULL.OnCancel probe");
    assert_eq!(
        kind, "function",
        "REALM_IS_FULL.OnCancel must be present after Mainline overrides \
         load — shared GlueDialogDefs.lua leaves OnCancel as a comment \
         placeholder (`--OnCancel OVERRIDEN`), Mainline/GlueDialogDefs.lua \
         line 7-10 attaches the C_RealmList.ClearRealmList + \
         CharacterSelectUtil.ChangeRealm callback"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn glue_dialog_named_frame_materializes() {
    let env = load_full_ui_for(ScreenKind::Login);

    let kind: String = env
        .eval("return type(GlueDialog)")
        .expect("GlueDialog probe");
    assert_eq!(
        kind, "table",
        "GlueDialog = frame — GlueDialog.xml line 15 declares the named \
         frame mixin=GlueDialogMixin toplevel=true hidden=true"
    );
}
}

#[test]
fn glue_dialog_button_template_materializes_via_create_frame() {
    let env = load_full_ui_for(ScreenKind::Login);

    let probe = "local ok, frame = pcall(function() \
                    return CreateFrame('Button', nil, UIParent, 'GlueDialogButtonTemplate') \
                  end) \
                  return ok and frame ~= nil";

    let result: bool = env.eval(probe).expect("template probe");
    assert!(
        result,
        "GlueDialogButtonTemplate must materialize via CreateFrame as a \
         Button — virtual=true, mixin=GlueDialogButtonMixin,StaticPopupElementMixin"
    );
}

post_wrath_ui_fixture! {
#[test]
fn glue_announcement_dialog_named_frame_materializes_on_glue() {
    let env = load_full_ui_for(ScreenKind::Login);

    let kind: String = env
        .eval("return type(GlueAnnouncementDialog)")
        .expect("GlueAnnouncementDialog probe");
    assert_eq!(
        kind, "table",
        "GlueAnnouncementDialog = frame — Mainline/\
         GlueAnnouncementDialog.xml inherits BaseNineSliceDialog with \
         GlueAnnouncementDialogMixin OnShow handler. Mainline-gated via \
         inline `[AllowLoadGameType mainline]` annotation"
    );
}
}
