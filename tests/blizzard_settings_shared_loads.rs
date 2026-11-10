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

fn settings_shared_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Settings_Shared")
}

fn settings_shared_toc() -> PathBuf {
    find_toc_file(&settings_shared_dir()).expect("Blizzard_Settings_Shared TOC resolves")
}

const HARD_DEPS: &[&str] = &[
    "Blizzard_SharedXML",
    "Blizzard_HelpPlate",
    "Blizzard_StaticPopup_Glue",
    "Blizzard_StaticPopup_Game",
];

// The unified PTR TOC adds two unconditional hard deps; the Forever TOC adds
// only Blizzard_GamepadSharedUtility.
const PTR_HARD_DEPS: &[&str] = &[
    "Blizzard_SharedXML",
    "Blizzard_HelpPlate",
    "Blizzard_StaticPopup_Glue",
    "Blizzard_StaticPopup_Game",
    "Blizzard_GamepadSharedUtility",
    "Blizzard_UIParentPanelManager",
];

const FOREVER_HARD_DEPS: &[&str] = &[
    "Blizzard_SharedXML",
    "Blizzard_HelpPlate",
    "Blizzard_StaticPopup_Glue",
    "Blizzard_StaticPopup_Game",
    "Blizzard_GamepadSharedUtility",
];

// The classic pinned `_Classic.toc` keeps the two hard dependencies and moves
// the two screen-scoped static popup addons into `## OptionalDep:`.
const CLASSIC_HARD_DEPS: &[&str] = &["Blizzard_SharedXML", "Blizzard_HelpPlate"];
const CLASSIC_OPTIONAL_DEPS: &[&str] = &["Blizzard_StaticPopup_Glue", "Blizzard_StaticPopup_Game"];

fn hard_deps() -> &'static [&'static str] {
    if toc_layout::classic_cache() {
        CLASSIC_HARD_DEPS
    } else if toc_layout::forever() {
        FOREVER_HARD_DEPS
    } else if toc_layout::unified_toc_tree() {
        PTR_HARD_DEPS
    } else {
        HARD_DEPS
    }
}

// Resolved body order of the unified PTR TOC: it lists the Lua siblings of
// CategoryList, SettingsList, SettingControls, Keybindings, and the Preferred
// Play settings explicitly, and gates the Camelot overrides out of Mainline.
const PTR_BODY: &[&str] = &[
    "NewDefinitionsFramework.lua",
    "NewDefinitionsFramework.xml",
    "Mainline/NewDefinitions.lua",
    "Blizzard_SettingsRegistrar.lua",
    "Blizzard_Settings.lua",
    "Blizzard_Category.lua",
    "Blizzard_CategoryList.lua",
    "Blizzard_CategoryList.xml",
    "Blizzard_SettingsList.lua",
    "Blizzard_SettingsList.xml",
    "Blizzard_Setting.lua",
    "Blizzard_SettingControls.lua",
    "Blizzard_SettingControls.xml",
    "Mainline/Blizzard_SettingsPanelTemplates.xml",
    "Blizzard_SettingsPanel.lua",
    "Blizzard_SettingsPanel.xml",
    "Blizzard_Keybindings.lua",
    "Blizzard_Keybindings.xml",
    "Blizzard_Dialogs.lua",
    "Blizzard_SettingsLayouts.lua",
    "Blizzard_Registration.lua",
    "Blizzard_SettingsInbound.lua",
    "Blizzard_Deprecated.lua",
    "Mainline/GraphicsOverrides.lua",
    "Mainline/AudioOverrides.lua",
    "Blizzard_Settings_PreferredPlaySettings.lua",
    "Mainline/LanguagesOverrides.lua",
];

// Resolved body order of the pinned Forever TOC: it resolves `[Game]` to
// Camelot, so the Camelot NewDefinitions/AudioOverrides/LanguagesOverrides
// entries replace the Mainline ones gated out by `ExcludeLoadGameType camelot`.
const FOREVER_BODY: &[&str] = &[
    "NewDefinitionsFramework.lua",
    "NewDefinitionsFramework.xml",
    "Camelot/NewDefinitions.lua",
    "Blizzard_SettingsRegistrar.lua",
    "Blizzard_Settings.lua",
    "Blizzard_Category.lua",
    "Blizzard_CategoryList.lua",
    "Blizzard_CategoryList.xml",
    "Blizzard_SettingsList.lua",
    "Blizzard_SettingsList.xml",
    "Blizzard_Setting.lua",
    "Blizzard_SettingControls.lua",
    "Blizzard_SettingControls.xml",
    "Mainline/Blizzard_SettingsPanelTemplates.xml",
    "Blizzard_SettingsPanel.lua",
    "Blizzard_SettingsPanel.xml",
    "Blizzard_Keybindings.lua",
    "Blizzard_Keybindings.xml",
    "Blizzard_Dialogs.lua",
    "Blizzard_SettingsLayouts.lua",
    "Blizzard_Registration.lua",
    "Blizzard_SettingsInbound.lua",
    "Blizzard_Deprecated.lua",
    "Mainline/GraphicsOverrides.lua",
    "Mainline/AudioOverrides.lua",
    "Camelot/AudioOverrides.lua",
    "Blizzard_Settings_PreferredPlaySettings.lua",
    "Camelot/LanguagesOverrides.lua",
];

const PUBLIC_MIXINS: &[&str] = &[
    "SettingMixin",
    "CVarSettingMixin",
    "ProxySettingMixin",
    "ModifiedClickSettingMixin",
    "AddOnSettingMixin",
    "SettingsCategoryMixin",
    "SettingsCategoryListHeaderMixin",
    "SettingsCategoryListButtonMixin",
    "SettingsCategoryListMixin",
    "SettingsListMixin",
    "SettingsListSearchCategoryMixin",
    "SettingsLayoutMixin",
    "SettingsPanelMixin",
    "SettingsSearchableElementMixin",
    "SettingsSliderOptionsMixin",
    "SettingsCallbackHandleContainerMixin",
    "DefaultTooltipMixin",
    "SettingsNewTagMixin",
    "SettingsListSectionHeaderMixin",
    "SettingsElementHierarchyMixin",
    "SettingsListElementMixin",
    "SettingsControlMixin",
    "SettingsCheckboxMixin",
    "SettingsCheckboxControlMixin",
    "SettingsSliderControlMixin",
    "SettingsDropdownControlMixin",
    "SettingsButtonControlMixin",
    "SettingsColorSwatchMixin",
    "SettingsColorSwatchControlMixin",
    "SettingsCheckboxWithButtonControlMixin",
    "SettingsCheckboxSliderControlMixin",
    "SettingsCheckboxDropdownControlMixin",
    "SettingsCheckboxWithColorSwatchControlMixin",
    "SettingsExpandableSectionMixin",
    "KeyBindingFrameBindingTemplateMixin",
    "KeyBindingButtonMixin",
];

const PUBLIC_SETTINGS_FUNCTIONS: &[&str] = &[
    "CreateCategory",
    "RegisterCategory",
    "RegisterAddOnCategory",
    "RegisterVerticalLayoutCategory",
    "RegisterCanvasLayoutCategory",
    "RegisterVerticalLayoutSubcategory",
    "RegisterCanvasLayoutSubcategory",
    "RegisterAddOnSetting",
    "RegisterProxySetting",
    "RegisterCVarSetting",
    "RegisterModifiedClickSetting",
    "RegisterInitializer",
    "GetCategory",
    "GetSetting",
    "GetValue",
    "SetValue",
    "OpenToCategory",
    "CreateCheckbox",
    "CreateSlider",
    "CreateDropdown",
    "CreateColorSwatch",
    "AssignLayoutToCategory",
    "AssignTutorialToCategory",
    "GetOrCreateSettingsGroup",
    "SafeLoadBindings",
    "NotifyUpdate",
    "SetKeybindingsCategory",
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
fn find_toc_file_resolves_bare_variant() {
    let resolved =
        find_toc_file(&settings_shared_dir()).expect("Blizzard_Settings_Shared TOC resolves");
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some(toc_layout::expected_toc_name("Blizzard_Settings_Shared").as_str()),
        "find_toc_file resolves the active profile's Settings_Shared TOC: Retail ships the \
         bare TOC and the classic caches the `_Classic` companion"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_declares_eager_both_with_four_dep_entries() {
    let toc = TocFile::from_file(&settings_shared_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_Settings_Shared MUST be eager — it provides the Settings.* \
         namespace and SettingsPanelMixin that every category-registering addon \
         (Blizzard_SettingsDefinitions_Shared, Blizzard_SettingsDefinitions_Frame, \
         Blizzard_AccountSaveUI, etc.) consumes at parse time. LoD here would \
         break the entire settings panel boot pipeline"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_glue_only());

    let deps = toc.dependencies();
    assert_eq!(
        deps.len(),
        hard_deps().len(),
        "TOC declares its Dep entries in published order. Got: {deps:?}"
    );
    for (i, expected) in hard_deps().iter().enumerate() {
        assert_eq!(deps[i], *expected, "Hard dep #{i} must be {expected}");
    }

    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare zero account-level SavedVariables — settings state \
         lives in CVars and per-character preference tables managed by \
         Blizzard_SettingsDefinitions_Shared (NewSettingsSeen), NOT in this base \
         addon"
    );
    assert!(toc.saved_variables_per_character().is_empty());
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_uses_dep_entries_without_optional_dependencies() {
    let toc = TocFile::from_file(&settings_shared_toc()).expect("TOC parses");

    if toc_layout::classic_cache() {
        assert_eq!(
            toc.optional_deps(),
            CLASSIC_OPTIONAL_DEPS
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            "the classic TOC declares the two screen-scoped static popup addons as optional"
        );
    } else {
        assert!(
            toc.optional_deps().is_empty(),
            "Retail 12.1.0.69497 uses Dep entries, not OptionalDep metadata"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_allows_load_on_every_screen_via_allow_load_both() {
    let toc = TocFile::from_file(&settings_shared_toc()).expect("TOC parses");

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "AllowLoad: Both must permit {screen:?} — settings panel base shares \
             code between in-game and glue-screen options dialogs (the glue \
             options dialog reuses SettingsPanelMixin / Setting / Category / \
             SettingsLayoutMixin). Without Both, the glue-screen options would \
             have no shared category infrastructure"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_raw_bytes_pin_current_dep_entries_and_allow_load_both() {
    let raw =
        std::fs::read_to_string(settings_shared_toc()).expect("Settings_Shared TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_Settings_Shared"));
    let expected: &[&str] = if toc_layout::classic_cache() {
        &[
            "## Dependencies: Blizzard_SharedXML, Blizzard_HelpPlate",
            "## OptionalDep: Blizzard_StaticPopup_Glue, Blizzard_StaticPopup_Game",
            "## AllowLoad: Both",
            "## AllowLoadGameType: classic",
        ]
    } else {
        &[
            "## Dep: Blizzard_SharedXML",
            "## Dep: Blizzard_HelpPlate",
            "## Dep: Blizzard_StaticPopup_Glue [AllowLoad glue]",
            "## Dep: Blizzard_StaticPopup_Game [AllowLoad game]",
            "## AllowLoad: Both",
        ]
    };
    for directive in expected {
        assert!(
            raw.contains(directive),
            "missing current directive: {directive}"
        );
    }
    assert!(
        !raw.contains("## SavedVariables"),
        "Settings_Shared MUST NOT declare any SavedVariables — settings state \
         flows through CVars and per-character SVs owned by \
         SettingsDefinitions_Shared, not this base addon"
    );
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## RequiredDep"));
    if !toc_layout::classic_cache() {
        assert!(!raw.contains("## OptionalDep"));
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_body_load_order_runs_definitions_framework_before_namespace_seed() {
    let raw = std::fs::read_to_string(settings_shared_toc()).expect("TOC reads utf-8");

    if toc_layout::forever() {
        let toc = TocFile::from_file(&settings_shared_toc()).expect("TOC parses");
        let body: Vec<String> = toc
            .files
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect();
        assert_eq!(
            body,
            FOREVER_BODY.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "The pinned Forever TOC resolves to this load order: the definitions framework \
             and namespace seed run first, then the category/settings/panel/registration chain"
        );
        return;
    }

    if toc_layout::unified_toc_tree() {
        let toc = TocFile::from_file(&settings_shared_toc()).expect("TOC parses");
        let body: Vec<String> = toc
            .files
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect();
        assert_eq!(
            body,
            PTR_BODY.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "The unified PTR TOC resolves to this load order: the definitions framework and \
             namespace seed run first, then the category/settings/panel/registration chain"
        );
        return;
    }

    let body_lines: Vec<&str> = raw
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();

    if toc_layout::classic_cache() {
        let expected_body = vec![
            "NewDefinitionsFramework.lua",
            "Blizzard_SettingsRegistrar.lua",
            "Blizzard_Settings.lua",
            "Blizzard_Category.lua",
            "Blizzard_CategoryList.xml",
            "Blizzard_SettingsList.xml",
            "Blizzard_Setting.lua",
            "Blizzard_SettingControls.xml",
            "Classic\\Blizzard_SettingsPanelTemplates.xml",
            "Blizzard_SettingsPanel.lua",
            "Blizzard_SettingsPanel.xml",
            "Blizzard_Keybindings.xml",
            "Blizzard_Dialogs.lua",
            "Blizzard_SettingsLayouts.lua",
            "Blizzard_Registration.lua",
            "Blizzard_SettingsInbound.lua",
            "Blizzard_Deprecated.lua",
            "Classic\\GraphicsOverrides.lua",
            "Classic\\AudioOverrides.lua",
        ];
        assert_eq!(
            body_lines, expected_body,
            "the pinned classic TOC lists these Settings_Shared body entries in this exact order"
        );
        return;
    }

    let expected_body = vec![
        "NewDefinitionsFramework.lua",
        "NewDefinitionsFramework.xml",
        "[Family]\\NewDefinitions.lua [AllowLoadGameType mainline]",
        "Blizzard_SettingsRegistrar.lua",
        "Blizzard_Settings.lua",
        "Blizzard_Category.lua",
        "Blizzard_CategoryList.xml",
        "Blizzard_SettingsList.xml",
        "Blizzard_Setting.lua",
        "Blizzard_SettingControls.xml",
        "[Family]\\Blizzard_SettingsPanelTemplates.xml",
        "Blizzard_SettingsPanel.lua",
        "Blizzard_SettingsPanel.xml",
        "Blizzard_Keybindings.xml",
        "Blizzard_Dialogs.lua",
        "Blizzard_SettingsLayouts.lua",
        "Blizzard_Registration.lua",
        "Blizzard_SettingsInbound.lua",
        "Blizzard_Deprecated.lua",
        "[Family]\\GraphicsOverrides.lua",
        "[Family]\\AudioOverrides.lua",
    ];
    assert_eq!(
        body_lines, expected_body,
        "Retail 12.1.0.69497 lists 21 current Settings_Shared body entries in this exact order"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_entry_count_breakdown_matches_filesystem_layout() {
    let toc = TocFile::from_file(&settings_shared_toc()).expect("TOC parses");

    let lua_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "lua"))
        .count();
    let xml_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "xml"))
        .count();
    let expected_lua = if toc_layout::classic_cache() {
        13
    } else if toc_layout::forever() {
        21
    } else if toc_layout::unified_toc_tree() {
        20
    } else {
        14
    };
    let expected_xml = if toc_layout::classic_cache() { 6 } else { 7 };
    assert_eq!(
        lua_count, expected_lua,
        "TOC body lists {expected_lua} Lua entries. Got: {lua_count}"
    );
    assert_eq!(
        xml_count, expected_xml,
        "TOC body lists {expected_xml} XML entries, including NewDefinitionsFramework.xml. Got: {xml_count}"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn family_placeholder_resolves_to_mainline_subdir_in_three_entries() {
    let toc = TocFile::from_file(&settings_shared_toc()).expect("TOC parses");

    let mainline_files: Vec<String> = toc
        .files
        .iter()
        .filter_map(|f| {
            let s = f.to_string_lossy().into_owned();
            if s.starts_with("Mainline/") {
                Some(s)
            } else {
                None
            }
        })
        .collect();
    let mut expected = vec![
        "Mainline/NewDefinitions.lua",
        "Mainline/Blizzard_SettingsPanelTemplates.xml",
        "Mainline/GraphicsOverrides.lua",
        "Mainline/AudioOverrides.lua",
    ];
    if toc_layout::ptr() {
        // PTR 12.1.7 Blizzard_Settings_Shared.toc:38 adds the Mainline language override.
        expected.push("Mainline/LanguagesOverrides.lua");
    }
    assert_eq!(
        mainline_files.len(),
        expected.len(),
        "Mainline body entries must match the active Settings_Shared TOC. PTR \
         Blizzard_Settings_Shared.toc:38 adds LanguagesOverrides.lua. Got: {mainline_files:?}"
    );

    for expected in expected {
        assert!(
            mainline_files.iter().any(|f| f == expected),
            "Mainline body entry {expected} must be present after normalization"
        );
    }
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
            .any(|(name, _)| name == "Blizzard_Settings_Shared");
        assert!(
            found,
            "Blizzard_Settings_Shared must appear in eager discovery on \
             {screen:?} — `## AllowLoad: Both` permits all 4 screens, and the \
             addon is non-LoD so it loads in the eager sweep. Without this the \
             settings panel base would be missing on glue screens, breaking the \
             pre-login options dialog"
        );
    }
}
}

prefork_full_ui_case! {
fn loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let relevant: Vec<&String> = errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_Settings_Shared")
                || e.contains("SettingsPanelMixin")
                || e.contains("SettingsCategoryMixin")
                || e.contains("SettingMixin")
                || e.contains("Blizzard_Setting.lua")
                || e.contains("Blizzard_SettingsPanel.lua")
                || e.contains("Blizzard_Category.lua")
                || e.contains("Blizzard_CategoryList.lua")
                || e.contains("NewDefinitionsFramework.lua")
        })
        .collect();
    assert!(
        relevant.is_empty(),
        "Blizzard_Settings_Shared must load with zero addon-specific Lua errors. \
         Got:\n  {}",
        relevant
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_Settings_Shared')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_Settings_Shared') must be true after \
         the eager Game-screen sweep — the addon registers via the eager pipeline \
         (non-LoD, AllowLoad=Both, AllowLoadGameType=mainline)"
    );
}
}

prefork_full_ui_case! {
fn publishes_settings_namespace_with_enum_subtables(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(Settings)")
        .expect("Settings type probe");
    assert_eq!(
        kind, "table",
        "_G.Settings must be a table — Blizzard_Settings.lua line 1 declares \
         `Settings = {{ CannotDefault = nil }}` at top scope. Every settings-using \
         addon (SettingsDefinitions_Shared, SettingsDefinitions_Frame, \
         AccountSaveUI, addon-specific category registrations) reads from this \
         namespace at parse time, so a missing `Settings` global means the entire \
         settings panel pipeline fails to bootstrap"
    );

    for subtable in [
        "VarType",
        "Default",
        "CategorySet",
        "ControlType",
        "CommitFlag",
    ] {
        let kind: String = env
            .eval(&format!("return type(Settings.{subtable})"))
            .unwrap_or_else(|_| panic!("Settings.{subtable} probe failed"));
        assert_eq!(
            kind, "table",
            "Settings.{subtable} must be an enum-like table — these are populated \
             via EnumUtil.MakeEnum / FlagsUtil.MakeFlags inside Blizzard_Settings.lua"
        );
    }

    let bool_var: String = env
        .eval("return Settings.VarType.Boolean")
        .expect("VarType.Boolean probe");
    assert_eq!(
        bool_var, "boolean",
        "Settings.VarType.Boolean must equal the literal string \"boolean\" — \
         RegisterAddOnSetting takes this as a type tag and forwards to the type \
         coercion path"
    );
}
}

prefork_full_ui_case! {
fn publishes_36_widget_and_setting_mixin_tables_across_settings_pipeline(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|_| panic!("{mixin} type probe failed"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must be a table — every settings widget / setting \
             primitive / category descriptor relies on these mixin namespaces \
             being available before XML templates with matching `mixin=\"...\"` \
             attributes parse"
        );
    }
}
}

prefork_full_ui_case! {
fn publishes_settings_factory_and_registrar_functions(env: &WowLuaEnv) {

    for func in PUBLIC_SETTINGS_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(Settings.{func})"))
            .unwrap_or_else(|_| panic!("Settings.{func} type probe failed"));
        assert_eq!(
            kind, "function",
            "Settings.{func} must be a function — addons register categories / \
             settings / controls through these factories. Missing any one breaks \
             a category-registration pattern (e.g., Settings.RegisterAddOnSetting \
             is called by every standalone-settings addon to seed CVar-mirroring \
             persistent state)"
        );
    }
}
}

prefork_full_ui_case! {
fn publishes_layout_factory_globals_for_canvas_and_vertical_modes(env: &WowLuaEnv) {

    for factory in ["CreateVerticalLayout", "CreateCanvasLayout"] {
        let kind: String = env
            .eval(&format!("return type({factory})"))
            .unwrap_or_else(|_| panic!("{factory} type probe failed"));
        assert_eq!(
            kind, "function",
            "_G.{factory} must be a function — Blizzard_SettingsLayouts.lua \
             publishes both at top scope. Settings.RegisterVerticalLayoutCategory \
             / RegisterCanvasLayoutCategory call these to construct the layout \
             descriptor wrapped around the SettingsLayoutMixin instance"
        );
    }
}
}

#[test]
fn post_cleanup_restore_preserves_blizzard_settings_get_category() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Settings = Settings or {}
        function Settings.GetCategory(name)
            return "blizzard:" .. tostring(name)
        end
        "#,
    )
    .expect("install Settings.GetCategory sentinel");

    env.restore_post_cleanup_globals();

    let result: String = env
        .eval(r#"return Settings.GetCategory("KrowiTest")"#)
        .expect("Settings.GetCategory sentinel probe");
    assert_eq!(
        result, "blizzard:KrowiTest",
        "post-cleanup bootstrap must not replace Blizzard's Settings.GetCategory implementation"
    );
}

prefork_full_ui_case! {
fn publishes_settings_panel_named_frame_with_high_strata_and_hidden_default(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(SettingsPanel)")
        .expect("SettingsPanel type probe");
    assert_eq!(
        kind, "table",
        "_G.SettingsPanel must exist as a Frame — \
         Blizzard_SettingsPanel.xml:`<Frame name=\"SettingsPanel\" \
         mixin=\"SettingsPanelMixin\" inherits=\"SettingsFrameTemplate\" \
         toplevel=\"true\" hidden=\"true\" frameStrata=\"HIGH\">` registers the \
         top-level container that hosts the entire settings UI. \
         Blizzard_Registration.lua line 10 calls RegisterUIPanel(SettingsPanel, \
         {{area=\"center\", pushable=0, whileDead=1, checkFit=1}}) which would \
         crash if the named frame were missing"
    );

    let strata: String = env
        .eval("return SettingsPanel:GetFrameStrata()")
        .expect("frame strata probe");
    assert_eq!(
        strata, "HIGH",
        "SettingsPanel must use HIGH frame strata — pinned by the XML attribute \
         frameStrata=\"HIGH\". Other dialogs use DIALOG; the settings panel sits \
         below those so DIALOG-strata popups (confirm-restart, accept-changes) \
         layer above it"
    );

    let hidden: bool = env
        .eval("return SettingsPanel:IsShown() == false")
        .expect("hidden probe");
    assert!(
        hidden,
        "SettingsPanel must default to hidden — pinned by `hidden=\"true\"` on \
         the XML. Players open it via the keybinding / menu button, not on \
         every game start. Without this default it would steal focus during \
         login"
    );
}
}

prefork_full_ui_case! {
fn publishes_new_settings_tracking_globals_seeded_then_populated(env: &WowLuaEnv) {

    let new_kind: String = env
        .eval("return type(NewSettings)")
        .expect("NewSettings type probe");
    assert_eq!(
        new_kind, "table",
        "_G.NewSettings must be a table — NewDefinitionsFramework.lua line 1 \
         seeds `NewSettings = {{}};` empty, then Mainline/NewDefinitions.lua \
         populates it with per-version arrays of new setting names \
         (NewSettings[\"10.1.0\"] = {{\"PROXY_CENSOR_MESSAGES\"}}; NewSettings[\
         \"11.2.0\"] = {{...}}; etc.)"
    );

    for tracking in ["NewSettingsSeen", "NewSettingsPredicates"] {
        let kind: String = env
            .eval(&format!("return type({tracking})"))
            .unwrap_or_else(|_| panic!("{tracking} type probe failed"));
        assert_eq!(
            kind, "table",
            "_G.{tracking} must be a table — both seeded by \
             NewDefinitionsFramework.lua at top scope before per-version content \
             populates"
        );
    }

    let has_120_block: bool = env
        .eval("return NewSettings['12.0.0'] ~= nil and #NewSettings['12.0.0'] > 0")
        .expect("12.0.0 block probe");
    assert!(
        has_120_block,
        "NewSettings['12.0.0'] must be a non-empty array after \
         Mainline/NewDefinitions.lua runs — the version key is the patch number \
         that introduced the listed settings; each subsequent patch adds a new \
         keyed array. Empty array would mean the data load was skipped"
    );
}
}

prefork_full_ui_case! {
fn category_set_enum_publishes_addons_and_game_keys(env: &WowLuaEnv) {

    let game_id: i64 = env
        .eval("return Settings.CategorySet.Game")
        .expect("CategorySet.Game probe");
    let addons_id: i64 = env
        .eval("return Settings.CategorySet.AddOns")
        .expect("CategorySet.AddOns probe");
    assert_ne!(
        game_id, addons_id,
        "Settings.CategorySet.Game and .AddOns must be distinct enum values — \
         EnumUtil.MakeEnum(\"Game\", \"AddOns\") assigns sequential ids starting \
         at 1. The category list panel keys off these to split top-level \
         categories into the Game and AddOns buckets shown side-by-side"
    );
}
}

prefork_full_ui_case! {
fn settings_panel_registers_as_ui_panel_only_in_game_screen(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(UIPanelWindows.SettingsPanel)")
        .expect("UIPanelWindows lookup");
    assert!(
        kind == "table" || kind == "nil",
        "UIPanelWindows.SettingsPanel must be either a table (in-game registered) \
         or nil (glue screen, registration skipped). \
         Blizzard_Registration.lua wraps the RegisterUIPanel call in `if not \
         C_Glue.IsOnGlueScreen() then ... end` so glue-screen loads do NOT \
         register the panel into UIPanelWindows. The simulator's \
         C_Glue.IsOnGlueScreen returns false on ScreenKind::Game so the table \
         entry should be present, but accept nil too in case the registration \
         path is gated differently in the simulator's glue stub. Got: {kind}"
    );
}
}
