use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn wow_survey_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_WowSurveyUI")
}

fn wow_survey_toc() -> PathBuf {
    wow_survey_dir().join("Blizzard_WowSurveyUI.toc")
}

fn status_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_StatusUI")
}

fn status_ui_toc() -> PathBuf {
    status_ui_dir().join("Blizzard_StatusUI.toc")
}

const REQUIRED_DEPS: &[&str] = &["Blizzard_StatusUI"];

const BODY_FILES: &[&str] = &[
    "Blizzard_WowSurveyUI_Bootstrap.lua",
    "Blizzard_WowSurveyUI.lua",
    "Blizzard_WowSurveyUI.xml",
];

const SURVEY_MIXIN_METHODS: &[&str] = &["OnLoad", "OnEvent", "OnClick"];

const STATUS_UI_MIXIN_METHODS: &[&str] = &["OnLoad", "OnShow", "OnHide"];

fn load_wow_survey_ui_with_dependency(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &status_ui_toc())
        .expect("Blizzard_StatusUI dep should load via explicit Rust loader call");
    load_addon(&env.loader_env(), &wow_survey_toc())
        .expect("Blizzard_WowSurveyUI should load via explicit Rust loader call");
}

#[test]
fn find_toc_file_resolves_bare_variant() {
    let resolved =
        find_toc_file(&wow_survey_dir()).expect("Blizzard_WowSurveyUI TOC should resolve");
    assert_eq!(
        resolved,
        wow_survey_toc(),
        "Blizzard_WowSurveyUI ships exactly one bare TOC — find_toc_file probes the \
         `_Mainline.toc` variant first (miss) and falls through to the bare TOC name (hit). \
         The user-survey UI is a thin universal addon shared across every retail flavor"
    );
}

#[test]
fn toc_declares_lod_with_status_ui_dep() {
    let toc = TocFile::from_file(&wow_survey_toc()).expect("Blizzard_WowSurveyUI TOC should parse");

    assert!(
        toc.is_load_on_demand(),
        "Blizzard_WowSurveyUI declares `## LoadOnDemand: 1` — pulled in by \
         `WowSurvey_LoadUI()` in Blizzard_UIParent/Shared/UIParent.lua:319 via \
         `UIParentLoadAddOn(\"Blizzard_WowSurveyUI\")`. The activation path is \
         event-driven: `UIParent_Shared_OnEvent` at line 35 watches `SURVEY_DELIVERED` \
         and on first delivery, lazy-loads this addon and re-dispatches the event to \
         `WowSurveyStatusFrame:OnEvent` so the status pulse appears the same frame the \
         server delivers the survey trigger"
    );

    let deps: Vec<String> = toc.dependencies().to_vec();
    assert_eq!(
        deps,
        REQUIRED_DEPS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "Blizzard_WowSurveyUI declares `## Dependencies: Blizzard_StatusUI` (singular plural \
         `Dependencies` key, single-dep value). The dep is itself a LoD addon that ships only \
         the `StatusUIFrame` virtual template + `StatusUIMixin`, both consumed at file scope by \
         this addon (the named button inherits StatusUIFrame, and the mixin's OnLoad is invoked \
         from WowSurveyStatusMixin:OnLoad as `StatusUIMixin.OnLoad(self)`)"
    );

    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
}

#[test]
fn toc_omits_allow_load_directives() {
    let toc = TocFile::from_file(&wow_survey_toc()).expect("Blizzard_WowSurveyUI TOC should parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without `## AllowLoad`, allows_screen falls through to the default Game-only branch"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Without `## AllowLoad`, allows_screen rejects glue screen {screen:?}"
        );
    }

    assert!(
        !toc.is_game_type_restricted(),
        "Without `## AllowLoadGameType`, is_game_type_restricted returns false. The survey \
         UI is gametype-unrestricted — surveys can be triggered on any retail flavor"
    );
}

#[test]
fn toc_lists_bootstrap_lua_and_xml_body_files() {
    let toc = TocFile::from_file(&wow_survey_toc()).expect("Blizzard_WowSurveyUI TOC should parse");

    let body_files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    assert_eq!(
        body_files,
        BODY_FILES.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "Retail 12.1.0.69497 TOC lists the bootstrap, then `Blizzard_WowSurveyUI.lua`, then \
         `Blizzard_WowSurveyUI.xml`. The lua file declares `WowSurveyStatusMixin = {{}}` and \
         attaches OnLoad/OnEvent/OnClick to it BEFORE \
         the XML element `<Button name=\"WowSurveyStatusFrame\" mixin=\"WowSurveyStatusMixin\">` \
         resolves the mixin name. If the XML loaded first, the mixin reference would resolve \
         to `nil` and the Button would inherit nothing"
    );
}

#[test]
fn toc_raw_bytes_pin_directives() {
    let raw =
        std::fs::read_to_string(wow_survey_toc()).expect("Blizzard_WowSurveyUI TOC should read");

    for directive in [
        "## Title: Blizzard_WowSurveyUI",
        "## LoadOnDemand: 1",
        "## Dependencies: Blizzard_StatusUI",
        "Blizzard_WowSurveyUI_Bootstrap.lua [Bootstrap]",
        "Blizzard_WowSurveyUI.lua",
        "Blizzard_WowSurveyUI.xml",
    ] {
        assert!(
            raw.contains(directive),
            "TOC raw bytes must contain `{directive}` — load-on-demand survey UI"
        );
    }

    for absent_directive in [
        "## Author:",
        "## Version:",
        "## Notes:",
        "## DefaultState:",
        "## RequiredDep:",
        "## RequiredDeps:",
        "## OptionalDeps:",
        "## LoadFirst:",
        "## LoadWith:",
        "## SavedVariables:",
        "## AllowLoad:",
        "## AllowLoadGameType:",
    ] {
        assert!(
            !raw.contains(absent_directive),
            "TOC raw bytes must NOT contain `{absent_directive}`"
        );
    }
}

#[test]
fn directory_holds_four_entries_with_bootstrap() {
    let entries: Vec<String> = std::fs::read_dir(wow_survey_dir())
        .expect("Blizzard_WowSurveyUI directory should exist")
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        4,
        "Retail 12.1.0.69497 directory holds the TOC, bootstrap Lua, main Lua, and XML. \
         Got: {entries:?}"
    );
}

#[test]
fn dep_directory_exists_on_disk() {
    assert!(
        status_ui_dir().is_dir(),
        "The `Blizzard_StatusUI` dep directory must exist on disk at \
         `Interface/BlizzardUI/Blizzard_StatusUI/` — the addon ships only StatusUIFrame + \
         StatusUIMixin and is itself a LoD addon"
    );
    assert!(
        status_ui_toc().is_file(),
        "Blizzard_StatusUI ships a bare TOC (no flavor variants)"
    );
}

#[test]
fn absent_from_all_screen_auto_discovery() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let names: Vec<&str> = addons.iter().map(|(name, _)| name.as_str()).collect();

        assert!(
            !names.contains(&"Blizzard_WowSurveyUI"),
            "Blizzard_WowSurveyUI must NOT appear in {screen:?} eager discovery — \
             `## LoadOnDemand: 1` excludes the addon from every screen's auto-discovery sweep. \
             Activation is exclusively through `WowSurvey_LoadUI()` triggered by the \
             SURVEY_DELIVERED event handler in eagerly-loaded UIParent.lua"
        );
        assert!(
            !names.contains(&"Blizzard_StatusUI"),
            "Blizzard_StatusUI dep must also be absent from {screen:?} eager discovery — \
             both addons are LoD; the dep is brought in only when an LoD consumer is loaded"
        );
    }
}

prefork_full_ui_case! {
fn loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_wow_survey_ui_with_dependency(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("WowSurveyUI")
                || message.contains("StatusUI")
                || message.contains("WowSurveyStatusFrame")
                || message.contains("WowSurveyStatusMixin")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_WowSurveyUI / Blizzard_StatusUI emitted addon-specific Lua errors during \
         load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_explicit_lod_load(env: &WowLuaEnv) {
    load_wow_survey_ui_with_dependency(env);

    let survey_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_WowSurveyUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        survey_loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_WowSurveyUI') must return true after the explicit \
         load_addon call — proves the LoD bring-up succeeded"
    );

    let dep_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_StatusUI')")
        .expect("IsAddOnLoaded dep probe should succeed");
    assert!(
        dep_loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_StatusUI') must also return true — the dep was \
         loaded explicitly before the consumer in load_wow_survey_ui_with_dependency"
    );
}
}

prefork_full_ui_case! {
fn wow_survey_load_ui_publishes_as_function(env: &WowLuaEnv) {
    load_wow_survey_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(WowSurvey_LoadUI)")
        .expect("WowSurvey_LoadUI probe should succeed");
    assert_eq!(
        kind, "function",
        "WowSurvey_LoadUI must publish as a free function in `_G` — declared at \
         Blizzard_UIParent/Shared/UIParent.lua:319 in eagerly-loaded core code, so it exists \
         before the LoD addon itself loads. The body wraps `UIParentLoadAddOn(\"Blizzard_WowSurveyUI\")` \
         which calls `C_AddOns.LoadAddOn` and surfaces the failure dialog on error"
    );
}
}

prefork_full_ui_case! {
fn wow_survey_status_mixin_publishes_with_methods(env: &WowLuaEnv) {
    load_wow_survey_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(WowSurveyStatusMixin)")
        .expect("WowSurveyStatusMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "WowSurveyStatusMixin must publish as a table at `_G` — declared at file scope on \
         line 33 of Blizzard_WowSurveyUI.lua. The lua loads via the TOC body's first body line \
         (NOT through `<Script file>` — the lua is in the TOC body directly)"
    );

    for method in SURVEY_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(WowSurveyStatusMixin.{method})"))
            .unwrap_or_else(|err| panic!("WowSurveyStatusMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "WowSurveyStatusMixin.{method} must be a function — assigned via \
             `function WowSurveyStatusMixin:Method(...)` syntax in the lua file"
        );
    }
}
}

prefork_full_ui_case! {
fn status_ui_mixin_publishes_with_methods(env: &WowLuaEnv) {
    load_wow_survey_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(StatusUIMixin)")
        .expect("StatusUIMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "StatusUIMixin must publish as a table at `_G` after the dep `Blizzard_StatusUI` \
         loads. The mixin is declared at line 1 of Blizzard_StatusUI/Blizzard_StatusUI.lua \
         and is consumed at file scope by WowSurveyStatusMixin:OnLoad which calls \
         `StatusUIMixin.OnLoad(self)` directly (NOT through CreateFromMixins — explicit \
         delegation, allowing the survey mixin to insert SetText calls before invoking the \
         base class's layout logic)"
    );

    for method in STATUS_UI_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(StatusUIMixin.{method})"))
            .unwrap_or_else(|err| panic!("StatusUIMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "StatusUIMixin.{method} must be a function — the dep contributes 3 methods that \
             form the base status-UI behavior reused by both the survey UI here and the \
             trainer/GM-status UI elsewhere"
        );
    }
}
}

prefork_full_ui_case! {
fn status_ui_frame_template_registers_as_virtual_button(env: &WowLuaEnv) {
    load_wow_survey_ui_with_dependency(env);
    let _env = env;

    assert!(
        wow_ui_sim::xml::get_template("StatusUIFrame").is_some(),
        "StatusUIFrame (`<Button virtual=\"true\">` from Blizzard_StatusUI.xml) must be \
         registered in the template registry after the dep loads. The template is what \
         WowSurveyStatusFrame inherits — it ships the chrome (NineSlice + Pulse animation + \
         TitleText/SubtitleText FontStrings + Icon) and the StatusUIMixin attachment, so the \
         survey button only needs to set the title text + register the SURVEY_DELIVERED event"
    );
}
}

prefork_full_ui_case! {
fn wow_survey_status_frame_publishes_with_inherited_template(env: &WowLuaEnv) {
    load_wow_survey_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(WowSurveyStatusFrame)")
        .expect("WowSurveyStatusFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "WowSurveyStatusFrame must publish at `_G` after addon load — \
         `<Button name=\"WowSurveyStatusFrame\" inherits=\"StatusUIFrame\" \
         mixin=\"WowSurveyStatusMixin\">` resolves through the template registry to a Button \
         frame with the inherited chrome + the survey mixin layered on top"
    );

    let frame_name: String = env
        .eval("return WowSurveyStatusFrame:GetName()")
        .expect("GetName probe should succeed");
    assert_eq!(frame_name, "WowSurveyStatusFrame");

    let object_type: String = env
        .eval("return WowSurveyStatusFrame:GetObjectType()")
        .expect("GetObjectType probe should succeed");
    assert_eq!(
        object_type, "Button",
        "WowSurveyStatusFrame must be a Button (NOT a Frame) — the XML declares it as \
         `<Button>` because the survey-status indicator is clickable; OnClick is mapped to \
         `WowSurveyStatusMixin:OnClick` which calls `StaticPopup_Show(\"WOW_SURVEY\")` to \
         present the YES/LATER/NO dialog"
    );
}
}

prefork_full_ui_case! {
fn static_popup_dialog_registers_with_three_buttons(env: &WowLuaEnv) {
    load_wow_survey_ui_with_dependency(env);

    let dialog_kind: String = env
        .eval("return type(StaticPopupDialogs and StaticPopupDialogs['WOW_SURVEY'])")
        .expect("StaticPopupDialogs[WOW_SURVEY] probe should succeed");
    assert_eq!(
        dialog_kind, "table",
        "StaticPopupDialogs['WOW_SURVEY'] must register as a table at file scope when the \
         lua loads — line 2 assigns `StaticPopupDialogs[\"WOW_SURVEY\"] = {{...}}`"
    );

    let select_callback_by_index: bool = env
        .eval("return StaticPopupDialogs['WOW_SURVEY'].selectCallbackByIndex == true")
        .expect("selectCallbackByIndex probe should succeed");
    assert!(
        select_callback_by_index,
        "selectCallbackByIndex MUST be true — this flag tells StaticPopup_Show to dispatch \
         OnButton1/OnButton2/OnButton3 callbacks indexed by button position rather than the \
         legacy OnAccept/OnCancel pair. The 3-button YES/LATER/NO contract requires this flag \
         because LATER (button 2) needs to differ from NO (button 3): YES opens the survey, \
         LATER suppresses but lets the prompt re-fire on next survey, NO marks the prompt as \
         answered without opening the survey URL"
    );

    for (button_idx, expected_global) in [(1, "YES"), (2, "LATER"), (3, "NO")] {
        let button_text: String = env
            .eval(&format!(
                "return StaticPopupDialogs['WOW_SURVEY'].button{button_idx}"
            ))
            .unwrap_or_else(|err| panic!("button{button_idx} probe failed: {err}"));
        let expected_text: String = env
            .eval(&format!(
                "return type({expected_global}) == 'string' and {expected_global} or ''"
            ))
            .unwrap_or_else(|err| panic!("global {expected_global} probe failed: {err}"));
        assert_eq!(
            button_text, expected_text,
            "button{button_idx} must be the global `{expected_global}` localized string — the \
             dialog reuses the standard YES/LATER/NO localizations from the global string \
             table rather than declaring its own"
        );
    }

    for callback_field in ["OnButton1", "OnButton2", "OnButton3", "OnShow", "OnHide"] {
        let callback_kind: String = env
            .eval(&format!(
                "return type(StaticPopupDialogs['WOW_SURVEY'].{callback_field})"
            ))
            .unwrap_or_else(|err| panic!("{callback_field} probe failed: {err}"));
        assert_eq!(
            callback_kind, "function",
            "StaticPopupDialogs['WOW_SURVEY'].{callback_field} must be a function — the dialog \
             registers explicit OnButton1 (calls C_WowSurvey.OpenSurvey + accepted=true), \
             OnButton2 (no-op, accepted=false), OnButton3 (accepted=true), OnShow (hides \
             WowSurveyStatusFrame on dialog open), and OnHide (re-shows WowSurveyStatusFrame \
             only when not accepted — i.e. on LATER, so the pulse persists for the next \
             prompt cycle)"
        );
    }

    let while_dead: bool = env
        .eval("return StaticPopupDialogs['WOW_SURVEY'].whileDead == 1")
        .expect("whileDead probe should succeed");
    assert!(
        while_dead,
        "whileDead must be 1 — survey prompt remains visible during corpse run"
    );

    let hide_on_escape: bool = env
        .eval("return StaticPopupDialogs['WOW_SURVEY'].hideOnEscape == 1")
        .expect("hideOnEscape probe should succeed");
    assert!(
        hide_on_escape,
        "hideOnEscape must be 1 — Escape closes the dialog without calling any OnButton \
         callback, so OnHide's `not accepted` branch fires and the WowSurveyStatusFrame pulse \
         re-shows. Escape behaves like LATER from the user's perspective"
    );
}
}

prefork_full_ui_case! {
fn wow_survey_status_frame_inherits_status_ui_chrome_children(env: &WowLuaEnv) {
    load_wow_survey_ui_with_dependency(env);

    for child_key in ["TitleText", "SubtitleText", "Icon", "NineSlice", "Pulse"] {
        let child_kind: String = env
            .eval(&format!(
                "return type(WowSurveyStatusFrame and WowSurveyStatusFrame.{child_key})"
            ))
            .unwrap_or_else(|err| panic!("WowSurveyStatusFrame.{child_key} probe failed: {err}"));
        assert!(
            matches!(child_kind.as_str(), "table" | "userdata"),
            "WowSurveyStatusFrame.{child_key} must resolve to a child object via parentKey \
             inheritance from the StatusUIFrame template — got `{child_kind}`. The dep's XML \
             declares 3 ARTWORK-layer regions (TitleText / SubtitleText FontStrings + Icon \
             texture) plus 2 frame children (NineSlice + Pulse animation overlay), and the \
             survey button inherits ALL of them through `inherits=\"StatusUIFrame\"`"
        );
    }
}
}
