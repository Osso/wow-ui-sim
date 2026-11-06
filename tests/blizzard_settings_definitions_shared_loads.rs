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

fn definitions_shared_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SettingsDefinitions_Shared")
}

fn definitions_shared_toc() -> PathBuf {
    definitions_shared_dir().join("Blizzard_SettingsDefinitions_Shared.toc")
}

const HARD_DEPS: &[&str] = &[
    "Blizzard_Settings_Shared",
    "Blizzard_TextStatusBar",
    "Blizzard_Colors",
    "Blizzard_AccessibilityTemplates",
];

// The unified PTR TOC places Gamepad.lua between Network.lua and Mac.lua.
const PTR_FIRST_SEVEN: &[&str] = &[
    "Groups.lua",
    "Graphics.xml",
    "Audio.xml",
    "Languages.xml",
    "Network.lua",
    "Gamepad.lua",
    "Mac.lua",
];

const RETAIL_FIRST_SEVEN: &[&str] = &[
    "Groups.lua",
    "Graphics.xml",
    "Audio.xml",
    "Languages.xml",
    "Network.lua",
    "Mac.lua",
    "AccessibilitySettingsPreview.lua",
];

// The pinned Forever TOC lists the camelot `Preset.lua` between Graphics.xml
// and Audio.xml.
const FOREVER_FIRST_SEVEN: &[&str] = &[
    "Groups.lua",
    "Graphics.xml",
    "Preset.lua",
    "Audio.xml",
    "Languages.xml",
    "Network.lua",
    "Gamepad.lua",
];

fn first_seven_entries() -> &'static [&'static str] {
    if toc_layout::forever() {
        FOREVER_FIRST_SEVEN
    } else if toc_layout::unified_toc_tree() {
        PTR_FIRST_SEVEN
    } else {
        RETAIL_FIRST_SEVEN
    }
}

const PUBLIC_GLOBAL_MIXINS: &[&str] = &[
    "AccessibilityFontPreviewMixin",
    "AccessibilitySettingsPreviewMixin",
    "LanguageRestartNeededMixin",
    "MacMicrophoneAccessWarningMixin",
    "QuestTextPreviewMixin",
    "SettingsAdvancedCheckboxSliderMixin",
    "SettingsAdvancedDropdownMixin",
    "SettingsAdvancedQualityControlsMixin",
    "SettingsAdvancedQualitySectionMixin",
    "SettingsAdvancedSliderMixin",
    "SettingsAudioLocaleDropdownMixin",
    "SettingsLanguageDropdownControlMixin",
    "SettingsLanguageDropdownMixin",
    "VoicePushToTalkMixin",
    "VoiceTestMicrophoneMixin",
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

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
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
    let resolved = find_toc_file(&definitions_shared_dir())
        .expect("Blizzard_SettingsDefinitions_Shared TOC resolves");
    assert_eq!(
        resolved,
        definitions_shared_toc(),
        "Blizzard_SettingsDefinitions_Shared ships exactly one bare \
         `Blizzard_SettingsDefinitions_Shared.toc` — no flavor variants. The \
         flavor split happens at the per-file level via `[Family]` placeholder \
         (resolves to `Mainline/` per src/toc.rs:145) and per-line \
         `[AllowLoadGameType mainline]` annotations on the 5 mainline-only \
         text-preview entries"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_declares_eager_both_with_four_hard_deps_and_account_saved_vars() {
    let toc = TocFile::from_file(&definitions_shared_toc())
        .expect("Blizzard_SettingsDefinitions_Shared TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare LoadOnDemand — eager Both-screen load required so \
         the cross-flavor settings categories (Graphics / Audio / Languages / \
         Network / Mac / Accessibility) register on every screen, in-game and \
         glue. Groups.lua loads first and runs `if C_Glue.IsOnGlueScreen()` to \
         pick the SETTING_GROUP_SYSTEM-first ordering on glue vs the \
         SETTING_GROUP_GAMEPLAY-first ordering in-game"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "`## AllowLoad: Both` must enable {screen:?} — cross-flavor settings \
             categories must register on every screen since the player can open \
             the settings panel on glue screens (login/character-select) too"
        );
    }

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        HARD_DEPS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "TOC must declare exactly 4 hard `## Dependencies:` in this order: \
         Blizzard_Settings_Shared (Settings panel infrastructure — \
         Settings.GetOrCreateSettingsGroup / RegisterVerticalLayoutCategory / \
         CreateSettingsListSectionHeaderInitializer); Blizzard_TextStatusBar \
         (UnitFrame status-bar template family used by audio-level meters and \
         voice-activity preview widgets); Blizzard_Colors (NORMAL_FONT_COLOR / \
         HIGHLIGHT_FONT_COLOR / class-color tables for preview widgets); \
         Blizzard_AccessibilityTemplates (AccessibilitySettingsPreviewTemplate \
         that QuestTextPreviewMixin / AccessibilityFontPreviewMixin extend via \
         CreateFromMixins). Order matters: Settings_Shared must publish the \
         Settings global before Groups.lua's first body line runs"
    );
    assert!(toc.optional_deps().is_empty());

    assert_eq!(
        toc.saved_variables(),
        vec!["NewSettingsSeen".to_string()],
        "Retail 12.1.5.70575 declares exactly one account-wide \
         `## SavedVariables: NewSettingsSeen` — tracks which Setting elements \
         have been seen, used by the settings panel to badge unseen entries \
         with a 'NEW' indicator"
    );
    assert!(
        toc.saved_variables_per_character().is_empty(),
        "TOC declares no per-character SavedVariables"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_raw_bytes_pin_load_saved_variables_first_for_pre_lua_sv_loading() {
    let raw = std::fs::read_to_string(definitions_shared_toc())
        .expect("Blizzard_SettingsDefinitions_Shared TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_SettingsDefinitions_Shared"));
    assert!(raw.contains("## AllowLoad: Both"));
    assert!(raw.contains("## SavedVariables: NewSettingsSeen"));

    assert!(
        raw.contains("## LoadSavedVariablesFirst: 1"),
        "TOC must declare `## LoadSavedVariablesFirst: 1` — instructs the addon \
         loader to populate `_G.NewSettingsSeen` from the SavedVariables file \
         BEFORE executing the Lua body, instead of the default post-execution \
         restoration. Required because Groups.lua and the body files reference \
         the SV state at module top scope (e.g., to check whether a category \
         should be flagged as new). The simulator's TocFile parser preserves \
         this metadata key in the raw bytes; runtime SV-load ordering is not \
         currently differentiated"
    );
    assert!(raw.contains(
        "## Dependencies: Blizzard_Settings_Shared, Blizzard_TextStatusBar, \
         Blizzard_Colors, Blizzard_AccessibilityTemplates"
    ));
}
}

post_wrath_ui_fixture! {
#[test]
fn family_placeholder_in_toc_body_resolves_to_mainline_subdir() {
    let toc = TocFile::from_file(&definitions_shared_toc())
        .expect("Blizzard_SettingsDefinitions_Shared TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    for resolved in [
        "Mainline/QuestTextPreview.lua",
        "Mainline/QuestTextPreview.xml",
        "Mainline/TextSizePreview.lua",
        "Mainline/TextSizePreview.xml",
        "Mainline/Text.lua",
    ] {
        assert!(
            listed.iter().any(|p| p == resolved),
            "TOC body must list `{resolved}` after `[Family]` placeholder \
             resolution — push_file_entry at src/toc.rs:145 replaces \
             `[Family]` with `Mainline` AND backslash with forward slash, so \
             the raw `[Family]\\QuestTextPreview.lua` becomes \
             `Mainline/QuestTextPreview.lua`. Got: {listed:?}"
        );
    }

    for raw_form in [
        "[Family]/QuestTextPreview.lua",
        "[Family]\\QuestTextPreview.lua",
    ] {
        assert!(
            !listed.iter().any(|p| p == raw_form),
            "TOC body must NOT contain unresolved `[Family]` placeholder \
             `{raw_form}` — the parser MUST replace it during load. If this \
             fires, push_file_entry's [Family] substitution regressed"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_body_first_seven_entries_are_cross_flavor_files_in_declared_order() {
    let toc = TocFile::from_file(&definitions_shared_toc())
        .expect("Blizzard_SettingsDefinitions_Shared TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    let first_seven: Vec<String> = listed.iter().take(7).cloned().collect();
    assert_eq!(
        first_seven,
        first_seven_entries()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "First 7 body entries must be the cross-flavor files in declared order. \
         Groups.lua MUST run first — it gates `if C_Glue.IsOnGlueScreen()` and \
         calls `Settings.GetOrCreateSettingsGroup(SETTING_GROUP_SYSTEM, ...)` / \
         `(SETTING_GROUP_GAMEPLAY, ...)` / `(SETTING_GROUP_ACCESSIBILITY, ...)` \
         to seed the panel's group ordering. The XML files \
         (Graphics/Audio/Languages) load via XML→`<Script file=\"...\"/>` \
         pulling Graphics.lua / Audio.lua / Languages.lua respectively. \
         Network.lua then Mac.lua then AccessibilitySettingsPreview.lua \
         declare the remaining cross-flavor mixins"
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
            .any(|(name, _)| name == "Blizzard_SettingsDefinitions_Shared");
        assert!(
            found,
            "Blizzard_SettingsDefinitions_Shared must auto-discover on screen \
             {screen:?} — eager (no LoadOnDemand) AND `## AllowLoad: Both` \
             puts it in every screen's eager set"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn root_directory_holds_eight_top_level_lua_xml_files_plus_mainline_subdir() {
    let dir = definitions_shared_dir();
    assert!(
        dir.join("Blizzard_SettingsDefinitions_Shared.toc")
            .is_file()
    );

    for filename in [
        "Groups.lua",
        "Graphics.lua",
        "Graphics.xml",
        "Audio.lua",
        "Audio.xml",
        "Languages.lua",
        "Languages.xml",
        "Network.lua",
        "Mac.lua",
        "AccessibilitySettingsPreview.lua",
    ] {
        assert!(
            dir.join(filename).is_file(),
            "Blizzard_SettingsDefinitions_Shared/{filename} must exist"
        );
    }

    let mainline_dir = dir.join("Mainline");
    assert!(mainline_dir.is_dir());

    for filename in [
        "QuestTextPreview.lua",
        "QuestTextPreview.xml",
        "TextSizePreview.lua",
        "TextSizePreview.xml",
        "Text.lua",
    ] {
        assert!(
            mainline_dir.join(filename).is_file(),
            "Blizzard_SettingsDefinitions_Shared/Mainline/{filename} must exist \
             — `[Family]\\` placeholder in TOC body resolves to this directory \
             on mainline; classic flavors omit it via the per-line \
             `[AllowLoadGameType mainline]` postfix filter"
        );
    }
}
}

prefork_full_ui_case! {
fn loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_SettingsDefinitions_Shared")
                || message.contains("AccessibilitySettingsPreview")
                || message.contains("SettingsLanguageDropdown")
                || message.contains("VoiceTestMicrophone")
                || message.contains("SettingsAdvancedQuality")
                || message.contains("SETTING_GROUP_")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_SettingsDefinitions_Shared emitted addon-specific Lua errors \
         during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SettingsDefinitions_Shared')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_SettingsDefinitions_Shared') must \
         return true after the eager Game-screen sweep"
    );
}
}

prefork_full_ui_case! {
fn publishes_fifteen_global_mixin_tables_for_settings_widgets(env: &WowLuaEnv) {

    for mixin in PUBLIC_GLOBAL_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — the 15 widget mixins drive \
             the cross-flavor settings panel widgets: \
             AccessibilitySettingsPreviewMixin (base preview class) and 2 \
             `CreateFromMixins` descendants AccessibilityFontPreviewMixin / \
             QuestTextPreviewMixin (the latter mainline-only via \
             [AllowLoadGameType mainline]); LanguageRestartNeededMixin extends \
             SettingsListElementMixin (warning row when locale change requires \
             restart); MacMicrophoneAccessWarningMixin (Mac.lua macOS-specific \
             permission notice); 5 SettingsAdvanced* mixins for graphics-quality \
             section / sliders / dropdowns / checkbox-sliders extending \
             SettingsExpandableSectionMixin and DefaultTooltipMixin; \
             SettingsLanguageDropdownMixin / SettingsLanguageDropdownControlMixin / \
             SettingsAudioLocaleDropdownMixin (locale picker family); \
             VoicePushToTalkMixin / VoiceTestMicrophoneMixin (Audio.lua \
             voice-chat preview rows extending SettingsListElementMixin)"
        );
    }
}
}

prefork_full_ui_case! {
fn debug_setting_group_global_published_only_when_gm_client(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(_G.SETTING_GROUP_DEBUG)")
        .expect("SETTING_GROUP_DEBUG probe succeeds");
    assert!(
        kind == "string" || kind == "nil",
        "_G.SETTING_GROUP_DEBUG must be either nil or a string — Groups.lua \
         line 9 does `SETTING_GROUP_DEBUG = \"Debug\"` ONLY in the in-game \
         (not glue) branch, AND the simulator's `IsGMClient()` stub may or \
         may not return truthy. Got kind={kind}. The variable serves as the \
         label for the in-game-only Debug group registered at position 4 when \
         the player is connected as a GM"
    );
}
}

prefork_full_ui_case! {
fn add_text_option_with_preview_helpers_publish_from_mainline_text_lua(env: &WowLuaEnv) {

    for fn_name in ["AddTextOptionWithPreview", "AddTextOptionsWithPreview"] {
        let kind: String = env
            .eval(&format!("return type(_G.{fn_name})"))
            .unwrap_or_else(|err| panic!("type(_G.{fn_name}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "_G.{fn_name} must publish as a function — Mainline/Text.lua \
             declares both helpers at top scope (lines 3 and 8) for the \
             text-preview section's option-with-preview / options-with-preview \
             registration patterns. These are mainline-only because the file \
             loads via `[Family]\\Text.lua [AllowLoadGameType mainline]` — \
             classic flavors don't get them"
        );
    }
}
}

prefork_full_ui_case! {
fn new_settings_seen_saved_var_publishes_as_table_after_load(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(_G.NewSettingsSeen)")
        .expect("NewSettingsSeen probe succeeds");
    assert!(
        kind == "table" || kind == "nil",
        "_G.NewSettingsSeen must be either a table (when SV file restored \
         contents OR the addon body initialized it) or nil (when SV is absent \
         AND no body line did `NewSettingsSeen = NewSettingsSeen or {{}}`). \
         The TOC declares this as `## SavedVariablesPerCharacter: \
         NewSettingsSeen` so the loader's SV restoration path manages the \
         global; the `## LoadSavedVariablesFirst: 1` annotation says to do \
         this before Lua body runs. Got: {kind}"
    );
}
}
