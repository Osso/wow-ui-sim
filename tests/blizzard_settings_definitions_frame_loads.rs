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

fn definitions_frame_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SettingsDefinitions_Frame")
}

fn definitions_frame_toc() -> PathBuf {
    definitions_frame_dir().join("Blizzard_SettingsDefinitions_Frame.toc")
}

const HARD_DEPS: &[&str] = &[
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_Colors",
    "Blizzard_GameMenuEsc",
];

const PUBLIC_GLOBAL_NAMESPACES: &[&str] = &[
    "ActionBarsOverrides",
    "AccessibilityOverrides",
    "ColorblindOverrides",
    "CombatOverrides",
    "ControlsOverrides",
    "InterfaceOverrides",
    "KeybindingsOverrides",
    "SocialOverrides",
    "CombatAudioAlertUtil",
];

const PUBLIC_GLOBAL_MIXINS: &[&str] = &[
    "ArachnophobiaMixin",
    "AutoLootDropdownControlMixin",
    "ItemQualityColorOverrideMixin",
    "NamePlatePreviewMixin",
    "NamePlatesTutorialMixin",
    "PingSystemMixin",
    "PingSystemTutorialMixin",
    "RaidFramePreviewMixin",
    "RTTSMixin",
    "SettingsKeybindingPrefaceMixin",
    "SettingsKeybindingSectionMixin",
    "SubtitlesPreviewMixin",
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

#[test]
fn retail_12_1_housing_settings_global_strings_match_pinned_export() {
    const EXPECTED: &[(&str, &str)] = &[
        ("HOUSING_SETTINGS_LABEL", "Housing"),
        (
            "DECOR_LIGHT_RADIUS_INDICATOR_ENABLED",
            "Light Radius Indicators",
        ),
        (
            "OPTION_TOOLTIP_DECOR_LIGHT_RADIUS_INDICATOR_ENABLED",
            "Enables Light Radius Indicators to show when placing Decor",
        ),
        ("DECOR_LIGHT_RADIUS_INDICATOR_TYPE_ALWAYS", "Always"),
        (
            "OPTION_TOOLTIP_DECOR_LIGHT_RADIUS_INDICATOR_TYPE_ALWAYS",
            "The Indicator is always visible while moving a light.",
        ),
        ("DECOR_LIGHT_RADIUS_INDICATOR_TYPE_OVERLAP", "Overlap"),
        (
            "OPTION_TOOLTIP_DECOR_LIGHT_RADIUS_INDICATOR_TYPE_OVERLAP",
            "When two lights are overlapping the indicator will appear.",
        ),
        ("DECOR_LIGHT_RADIUS_INDICATOR_TYPE_NEVER", "Never"),
        (
            "OPTION_TOOLTIP_DECOR_LIGHT_RADIUS_INDICATOR_TYPE_NEVER",
            "No indicator will show.",
        ),
        (
            "SELECTED_DECOR_LIGHT_RADIUS_INDICATOR_TYPE",
            "Selected Decor",
        ),
        (
            "OPTION_TOOLTIP_SELECTED_DECOR_LIGHT_RADIUS_INDICATOR_TYPE",
            "Controls how the Light Radius Indicator is displayed on selected Decor while decorating.",
        ),
        ("OTHER_DECOR_LIGHT_RADIUS_INDICATOR_TYPE", "Other Decor"),
        (
            "OPTION_TOOLTIP_OTHER_DECOR_LIGHT_RADIUS_INDICATOR_TYPE",
            "Controls how the Light Radius Indicator is displayed on non-selected Decor while decorating.",
        ),
    ];

    let env = WowLuaEnv::new().expect("failed to create Lua environment");
    for &(name, expected) in EXPECTED {
        let (actual_type, actual_value): (String, String) = env
            .eval(&format!(
                r#"
                local value = rawget(_G, {name:?})
                return type(value), type(value) == "string" and value or tostring(value)
                "#
            ))
            .unwrap_or_else(|error| panic!("failed to read global {name}: {error}"));

        assert_eq!(
            actual_type, "string",
            "global {name}: expected string {expected:?}, actual type {actual_type:?} value {actual_value:?}"
        );
        assert_eq!(
            actual_value, expected,
            "global {name}: expected {expected:?}, actual {actual_value:?}"
        );
    }
}

#[test]
fn find_toc_file_resolves_bare_variant() {
    assert_eq!(
        find_toc_file(&definitions_frame_dir())
            .expect("Blizzard_SettingsDefinitions_Frame TOC resolves"),
        definitions_frame_toc(),
        "Retail ships the bare Blizzard_SettingsDefinitions_Frame.toc."
    );
}

#[test]
fn toc_declares_eager_game_only_with_three_hard_deps() {
    let toc = TocFile::from_file(&definitions_frame_toc())
        .expect("Blizzard_SettingsDefinitions_Frame TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare LoadOnDemand — eager Game-screen load required so \
         every settings category (Controls, Interface, Combat, ActionBars, \
         Social, PingSystem, Keybindings, Accessibility, Colorblind, AudioAssist, \
         Mounts, Subtitles, AdvancedOptions, Nameplates) registers with the \
         Settings panel before the player opens it"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` must enable in-game loading — settings categories \
         are an in-game concern; the glue-screen options dialog uses a separate \
         pipeline"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must NOT be allowed — `## AllowLoad: Game` \
             explicitly excludes glue screens"
        );
    }

    assert!(
        !toc.is_game_type_restricted(),
        "AllowLoadGameType=mainline must NOT be classified as restricted — \
         is_game_type_restricted() returns false when the value contains \
         'mainline' or 'standard' (src/toc.rs:294-302). Only non-mainline \
         values like 'plunderstorm' or 'classic' are restricted"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        HARD_DEPS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "TOC must declare exactly 3 hard `## Dependencies:` in this order: \
         Blizzard_SharedXML (UIPanelDialogTemplate / FrameUtil / dropdown \
         controls), Blizzard_SettingsDefinitions_Shared (Setting / Category / \
         Layout primitives shared across category definitions), Blizzard_Colors \
         (NORMAL_FONT_COLOR / HIGHLIGHT_FONT_COLOR / RAID class color tables \
         used by ItemQualityColorOverrideMixin and NamePlatePreviewMixin). \
         Order matters: SharedXML must publish dropdown control mixins before \
         Controls.lua's `AutoLootDropdownControlMixin = \
         CreateFromMixins(SettingsDropdownControlMixin)` parse-time inheritance \
         resolves"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
}

#[test]
fn toc_raw_bytes_pin_metadata_with_default_state_enabled() {
    let raw = std::fs::read_to_string(definitions_frame_toc())
        .expect("Blizzard_SettingsDefinitions_Frame TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_SettingsDefinitions_Frame"));
    assert!(raw.contains("## AllowLoad: Game"));
    assert!(raw.contains(
        "## Dependencies: Blizzard_SettingsDefinitions_Shared, Blizzard_Colors, Blizzard_GameMenuEsc"
    ));
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare SavedVariables — categories register their own \
         CVars via the Setting primitive; the addon itself stores no \
         per-account state"
    );
}

#[test]
fn toc_body_lists_nine_mainline_overrides_then_current_main_files() {
    let toc = TocFile::from_file(&definitions_frame_toc())
        .expect("Blizzard_SettingsDefinitions_Frame TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    let first_nine: Vec<String> = listed.iter().take(9).cloned().collect();
    let expected_overrides = [
        "Mainline/GameplaySettingsGroup.lua",
        "Mainline/InterfaceOverrides.lua",
        "Mainline/ControlsOverrides.lua",
        "Mainline/CombatOverrides.lua",
        "Mainline/SocialOverrides.lua",
        "Mainline/ColorblindOverrides.lua",
        "Mainline/KeybindingsOverrides.lua",
        "Mainline/AccessibilityOverrides.lua",
        "Mainline/NameplatesOverrides.lua",
    ];
    assert_eq!(
        first_nine,
        expected_overrides
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "First 8 body entries must be the 8 Mainline\\*.lua override files in \
         declared order (GameplaySettingsGroup → InterfaceOverrides → \
         ControlsOverrides → CombatOverrides → SocialOverrides → \
         ColorblindOverrides → KeybindingsOverrides → AccessibilityOverrides). \
         GameplaySettingsGroup.lua MUST run first because it sets \
         `CUSTOM_GAMEPLAY_SETTINGS_ORDER` (a top-level global table mapping \
         CONTROLS_LABEL/INTERFACE_LABEL/etc. → numeric position) that downstream \
         Register() functions read at parse time to set their category position"
    );

    for required in [
        "Controls.xml",
        "Interface.lua",
        "Interface.xml",
        "Combat.lua",
        "ActionBars.lua",
        "Social.lua",
        "PingSystem.xml",
        "Keybindings.xml",
        "Accessibility.lua",
        "Accessibility.xml",
        "Mainline/Colorblind.xml",
        "CombatAudioAlertConstants.lua",
        "CombatAudioAlertUtil.lua",
        "Mounts.lua",
        "AdvancedOptions.lua",
        "Nameplates.lua",
        "Nameplates.xml",
    ] {
        assert!(
            listed.iter().any(|p| p == required),
            "TOC body must list {required:?}. Got: {listed:?}"
        );
    }

    let raw = std::fs::read_to_string(definitions_frame_toc())
        .expect("Blizzard_SettingsDefinitions_Frame TOC reads utf-8");
    assert!(
        raw.contains(
            "# NOTE: Accessibility.lua should be the only file loaded in classic, only \
             mainline needs the xml, but classic may need an older version of the lua file."
        ),
        "TOC must preserve the inline `# NOTE: Accessibility.lua...` comment — \
         documents the cross-flavor split that justifies keeping Accessibility.lua \
         in the bare-name body and Accessibility.xml separately listed for \
         mainline. Stripping this comment loses the rationale for why classic \
         flavors have a different load shape"
    );
}

#[test]
fn appears_only_on_game_screen_eager_discovery() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let found_in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_SettingsDefinitions_Frame");
    assert!(
        found_in_game,
        "Blizzard_SettingsDefinitions_Frame must auto-discover on Game screen — \
         eager (no LoadOnDemand) AND `## AllowLoad: Game` puts it in the \
         in-game eager set"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_SettingsDefinitions_Frame");
        assert!(
            !found,
            "Blizzard_SettingsDefinitions_Frame must be excluded from glue \
             screen {screen:?} — `## AllowLoad: Game` explicitly restricts to \
             in-game"
        );
    }
}

#[test]
fn root_directory_holds_nine_mainline_overrides_in_subdir() {
    let dir = definitions_frame_dir();
    let mainline_dir = dir.join("Mainline");
    assert!(
        mainline_dir.is_dir(),
        "Mainline\\ subdirectory must exist — holds the 8 mainline-only override \
         .lua files plus the Colorblind.xml that's mainline-only"
    );

    for filename in [
        "GameplaySettingsGroup.lua",
        "InterfaceOverrides.lua",
        "ControlsOverrides.lua",
        "CombatOverrides.lua",
        "SocialOverrides.lua",
        "ColorblindOverrides.lua",
        "KeybindingsOverrides.lua",
        "AccessibilityOverrides.lua",
        "NameplatesOverrides.lua",
        "Colorblind.xml",
    ] {
        assert!(
            mainline_dir.join(filename).is_file(),
            "Mainline\\{filename} must exist — listed in TOC body and required \
             for mainline category-registration overrides"
        );
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
            message.contains("Blizzard_SettingsDefinitions_Frame")
                || message.contains("CUSTOM_GAMEPLAY_SETTINGS_ORDER")
                || message.contains("RaidFramePreview")
                || message.contains("PingSystemMixin")
                || message.contains("ArachnophobiaMixin")
                || message.contains("NamePlatePreview")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_SettingsDefinitions_Frame emitted addon-specific Lua errors \
         during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SettingsDefinitions_Frame')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_SettingsDefinitions_Frame') must \
         return true after the eager Game-screen sweep"
    );
}
}

prefork_full_ui_case! {
fn publishes_nine_override_namespace_globals_as_tables(env: &WowLuaEnv) {

    for namespace in PUBLIC_GLOBAL_NAMESPACES {
        let kind: String = env
            .eval(&format!("return type(_G.{namespace})"))
            .unwrap_or_else(|err| panic!("type(_G.{namespace}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{namespace} must publish as a table — these 9 namespace tables \
             group the per-category override functions (e.g. \
             ActionBarsOverrides.CreateActionBarVisibilitySettings, \
             ActionBarsOverrides.AdjustActionBarSettings, \
             ActionBarsOverrides.RunSettingsCallback). The Mainline\\ override \
             files declare them as `<Namespace>Overrides = {{}}` at top scope; \
             non-Mainline classic flavors would publish stub or different \
             tables depending on their own override files"
        );
    }
}
}

prefork_full_ui_case! {
fn publishes_twelve_global_mixin_tables_for_settings_widgets(env: &WowLuaEnv) {

    for mixin in PUBLIC_GLOBAL_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — the 12 global mixins drive \
             the widgets shown inside settings categories: ArachnophobiaMixin \
             (Accessibility spider→crab swap preview), AutoLootDropdownControlMixin \
             (Controls auto-loot CVar dropdown extending \
             SettingsDropdownControlMixin), ItemQualityColorOverrideMixin \
             (Interface item-quality color override), NamePlatePreviewMixin / \
             NamePlatesTutorialMixin (Nameplates preview), PingSystemMixin / \
             PingSystemTutorialMixin (PingSystem category), \
             RaidFramePreviewMixin (Interface raid-frame preview), RTTSMixin \
             (Subtitles real-time text-to-speech dropdown extending \
             SettingsDropdownControlMixin), SettingsKeybindingPrefaceMixin / \
             SettingsKeybindingSectionMixin (Keybindings expandable sections), \
             SubtitlesPreviewMixin (Subtitles preview)"
        );
    }
}
}

prefork_full_ui_case! {
fn custom_gameplay_settings_order_publishes_with_label_keys(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(_G.CUSTOM_GAMEPLAY_SETTINGS_ORDER)")
        .expect("CUSTOM_GAMEPLAY_SETTINGS_ORDER probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.CUSTOM_GAMEPLAY_SETTINGS_ORDER must publish as a table — \
         Mainline\\GameplaySettingsGroup.lua line 2 declares \
         `CUSTOM_GAMEPLAY_SETTINGS_ORDER = {{ ... }}` at top scope. This table \
         maps category labels (CONTROLS_LABEL, INTERFACE_LABEL, ACTIONBARS_LABEL, \
         COMBAT_LABEL, SOCIAL_LABEL, PING_SYSTEM_LABEL, SETTINGS_KEYBINDINGS_LABEL, \
         ADVANCED_OPTIONS_LABEL, NAMEPLATE_OPTIONS_LABEL) to numeric positions \
         (7-15), letting per-category Register() functions place themselves in \
         a deterministic order without hardcoding numbers. GameplaySettingsGroup.lua \
         loads FIRST in TOC body order so the global is populated before any \
         category file's Register() function reads it"
    );
}
}

prefork_full_ui_case! {
fn self_cast_setting_values_publishes_four_named_modes_from_combat_lua(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(_G.SELF_CAST_SETTING_VALUES)")
        .expect("SELF_CAST_SETTING_VALUES probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.SELF_CAST_SETTING_VALUES must publish as a table — Combat.lua \
         line 1 declares `SELF_CAST_SETTING_VALUES = {{ NONE=1, AUTO=2, \
         KEY_PRESS=3, AUTO_AND_KEY_PRESS=4 }}` at top scope. The Combat \
         category dropdown reads these enum values to build its self-cast \
         mode picker"
    );

    for (key, expected) in [
        ("NONE", 1),
        ("AUTO", 2),
        ("KEY_PRESS", 3),
        ("AUTO_AND_KEY_PRESS", 4),
    ] {
        let value: i64 = env
            .eval(&format!("return SELF_CAST_SETTING_VALUES.{key}"))
            .unwrap_or_else(|err| panic!("SELF_CAST_SETTING_VALUES.{key} probe failed: {err}"));
        assert_eq!(
            value, expected,
            "SELF_CAST_SETTING_VALUES.{key} must equal {expected} — these \
             literal values are referenced by their integer codes in saved \
             CVar state, so changing the mapping would invalidate every \
             player's saved self-cast preference"
        );
    }
}
}
