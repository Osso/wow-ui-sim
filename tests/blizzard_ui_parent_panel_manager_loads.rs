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

fn ui_parent_panel_manager_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_UIParentPanelManager")
}

fn ui_parent_panel_manager_toc() -> PathBuf {
    ui_parent_panel_manager_dir().join("Blizzard_UIParentPanelManager_Mainline.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &["Blizzard_ManagedFrameSystem", "Blizzard_GameMenuEsc"];

const MODULE_LOAD_TABLES: &[&str] = &[
    "UIPanelWindows",
    "UIChildWindows",
    "UISpecialFrames",
    "UIMenus",
];

const MODULE_LOAD_BOOL_CONSTANTS: &[(&str, bool)] = &[
    ("UIPANEL_SKIP_SET_POINT", true),
    ("UIPANEL_VALIDATE_CURRENT_FRAME", true),
];

const MODULE_LOAD_NUMBER_CONSTANTS: &[(&str, f64)] = &[("BOTTOM_FRAME_CONTAINER_MARGIN", 15.0)];

const FREE_FUNCTIONS: &[&str] = &[
    "UIPanelWindows_Initialize",
    "RegisterUIPanel",
    "SetUIPanelAttribute",
    "ShowUIPanel",
    "HideUIPanel",
    "SetUIPanelShown",
    "ToggleFrame",
    "ToggleUIPanel",
    "GetUIPanel",
    "GetUIPanelWidthUnscaled",
    "GetUIPanelHeightUnscaled",
    "GetUIPanelWidth",
    "GetUIPanelHeight",
    "UIPanelUpdateScaleForFit",
    "GetMaxUIPanelsWidth",
    "ClampUIPanelY",
    "CanShowRightUIPanel",
    "CanShowCenterUIPanel",
    "CanShowUIPanels",
    "CanOpenPanels",
    "AreAllPanelsDisallowed",
    "CloseChildWindows",
    "CloseSpecialWindows",
    "CloseWindows",
    "CloseAllWindows",
    "CloseAllWindows_WithExceptions",
    "CloseMenus",
    "UpdateUIPanelPositions",
    "UpdateScaleForFitForOpenPanels",
    "MaximizeUIPanel",
    "RestoreUIPanelArea",
    "IsOptionFrameOpen",
    "LowerFrameLevel",
    "RaiseFrameLevel",
    "PassClickToParent",
    "ValidateFramePosition",
    "UIParent_ManageFramePositions",
];

const REPRESENTATIVE_PANEL_WINDOWS: &[(&str, &str)] = &[
    ("GameMenuFrame", "center"),
    ("HelpFrame", "center"),
    ("EditModeManagerFrame", "center"),
    ("CharacterFrame", "left"),
    ("EncounterJournal", "left"),
    ("CollectionsJournal", "left"),
    ("MailFrame", "left"),
    ("BankFrame", "left"),
    ("CinematicFrame", "full"),
    ("BarberShopFrame", "full"),
    ("PerksProgramFrame", "full"),
    ("DeathRecapFrame", "center"),
    ("PVPMatchScoreboard", "center"),
    ("GarrisonBuildingFrame", "center"),
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
fn find_toc_file_resolves_mainline_variant() {
    let resolved =
        find_toc_file(&ui_parent_panel_manager_dir()).expect("UIParentPanelManager TOC resolves");
    assert_eq!(
        resolved,
        ui_parent_panel_manager_toc(),
        "find_toc_file at src/loader/mod.rs:65-95 prefers \
         `<addon>_Mainline.toc`. Blizzard_UIParentPanelManager ships ONLY \
         the Mainline variant — no bare TOC, no Classic/Mists companion. \
         The panel-stack management algorithm differs enough between \
         flavors that Classic/Mists ship their own panel manager via a \
         different addon (the legacy in-FrameXML implementation)"
    );
}

#[test]
fn toc_is_eager_with_two_dependencies() {
    let toc = TocFile::from_file(&ui_parent_panel_manager_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` directive → eagerly loaded. The panel \
         manager publishes the global UIPanelWindows registry and the \
         ShowUIPanel / HideUIPanel API that EVERY in-world panel \
         consumes; it must load before any CharacterFrame / MailFrame \
         / etc. tries to show"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps, TOC_DEPENDENCIES,
        "Retail 12.1.0.69497 declares ManagedFrameSystem and GameMenuEsc before the panel \
         manager installs its layout and panel-close hooks. Got: {deps:?}"
    );
    assert!(toc.load_with().is_empty());

    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_game_restricts_to_in_world() {
    let toc = TocFile::from_file(&ui_parent_panel_manager_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase) hits toc.rs:308 → Game-only. \
         The panel-stack only exists in-world; glue screens have no \
         comparable concept"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — `AllowLoad: game` \
             matches only the Game variant via toc.rs:308"
        );
    }
}

#[test]
fn allow_load_game_type_mainline_is_not_restricted() {
    let toc = TocFile::from_file(&ui_parent_panel_manager_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: mainline` hits the non-restricting \
         branch at toc.rs:294-302 (standard|mainline accepted)"
    );
}

#[test]
fn toc_raw_bytes_pin_directives_and_body_files_with_inline_annotations() {
    let raw = std::fs::read_to_string(ui_parent_panel_manager_toc()).expect("TOC reads utf-8");

    let expected_lines = [
        "## Title: Blizzard_UIParentPanelManager",
        "## Author: Blizzard Entertainment",
        "## DefaultState: enabled",
        "## Dependencies: Blizzard_ManagedFrameSystem, Blizzard_GameMenuEsc",
        "## AllowLoad: game",
        "## AllowLoadGameType: mainline",
        "Mainline\\UIPanelWindows.lua",
        "Shared\\UIPanelLayoutFrame.lua [AllowLoadEnvironment Global]",
        "Shared\\UIParentPanelManager.lua [AllowLoadEnvironment Global]",
        "Mainline\\UIParentPanelManagerOverrides.lua [AllowLoadEnvironment Global]",
        "Shared\\UpdateUIPanelPositions.lua [AllowLoadEnvironment Global]",
    ];

    for line in expected_lines {
        assert!(
            raw.contains(line),
            "Raw TOC must pin `{line}`. Body order is deliberate: \
             UIPanelWindows.lua is followed by the layout frame, panel manager, \
             Mainline overrides, and position updater. The four \
             `[AllowLoadEnvironment Global]` annotations are stripped from file paths and \
             mapped to global-env file overrides"
        );
    }

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## OptionalDeps"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
}

#[test]
fn allow_load_environment_annotations_set_global_pass_filters() {
    let toc = TocFile::from_file(&ui_parent_panel_manager_toc()).expect("TOC parses");

    assert_eq!(toc.file_use_secure_env(0), None);
    for index in 1..5 {
        assert_eq!(
            toc.file_use_secure_env(index),
            None,
            "File index {index} carries `[AllowLoadEnvironment Global]`; it must not map to a \
             per-file fenv override"
        );
        assert_eq!(
            toc.file_allow_load_environment(index),
            Some(false),
            "File index {index} carries `[AllowLoadEnvironment Global]`, which must map to a \
             global-pass filter"
        );
    }
}

#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_UIParentPanelManager");
    assert!(
        found,
        "Blizzard_UIParentPanelManager must appear in Game eager \
         discovery — it's a hard dep of every panel that calls \
         ShowUIPanel/HideUIPanel"
    );
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_UIParentPanelManager");
        assert!(
            !found,
            "Blizzard_UIParentPanelManager must NOT appear on \
             {screen:?} — AllowLoad:game restricts to in-world via \
             toc.rs:308, checked at loader/mod.rs:527 BEFORE pool \
             partitioning"
        );
    }
}

#[test]
fn dep_directories_exist_on_disk() {
    for dep in TOC_DEPENDENCIES {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Hard-dep directory `{dep}` must exist on disk"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "{dep} must have a discoverable TOC"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_publishes_module_load_tables(env: &WowLuaEnv) {

    for table in MODULE_LOAD_TABLES {
        let kind: String = env
            .eval(&format!("return type({table})"))
            .unwrap_or_else(|err| panic!("{table} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{table} must be a global table after load. UIPanelWindows \
             is the panel-attribute registry populated by \
             UIPanelWindows_Initialize() and consulted by every \
             ShowUIPanel call to read area / pushable / whileDead / \
             allowOtherPanels / xoffset / yoffset / etc. The \
             UIChildWindows / UISpecialFrames / UIMenus tables \
             redeclared here OVERRIDE the smaller versions published \
             earlier by Blizzard_UIParent (load order guarantees \
             panel-manager wins)"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_module_load_bool_constants(env: &WowLuaEnv) {

    for (name, expected) in MODULE_LOAD_BOOL_CONSTANTS {
        let value: bool = env
            .eval(&format!("return {name}"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            value, *expected,
            "{name} must equal {expected}. UIPANEL_SKIP_SET_POINT is \
             passed as `skipSetPoint=true` to suppress the SetPoint call \
             during HideUIPanel/MoveUIPanel reorderings; \
             UIPANEL_VALIDATE_CURRENT_FRAME is passed when MoveUIPanel \
             should bail if the current frame is invalid"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_module_load_number_constants(env: &WowLuaEnv) {

    for (name, expected) in MODULE_LOAD_NUMBER_CONSTANTS {
        let value: f64 = env
            .eval(&format!("return {name}"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert!(
            (value - expected).abs() < 1e-9,
            "{name} must equal {expected} (got {value}). \
             BOTTOM_FRAME_CONTAINER_MARGIN comes from \
             Mainline/UIParentPanelManagerOverrides.lua:3 and is the \
             gap between bottom-managed frames and the bottom edge"
        );
    }
}
}

prefork_full_ui_case! {
fn ui_panel_skip_set_point_paired_with_nil_do_set_point(env: &WowLuaEnv) {

    let do_set_point_is_nil: bool = env
        .eval("return UIPANEL_DO_SET_POINT == nil")
        .expect("UIPANEL_DO_SET_POINT probe");
    assert!(
        do_set_point_is_nil,
        "UIPANEL_DO_SET_POINT is intentionally nil at \
         UIParentPanelManager.lua:5 — it pairs with UIPANEL_SKIP_SET_POINT=true \
         as the call-site readability helper for the inverse boolean. \
         Passing UIPANEL_DO_SET_POINT explicitly to MoveUIPanel makes \
         the intent (\"do set point\") clearer than passing literal nil"
    );
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_free_functions(env: &WowLuaEnv) {

    for func in FREE_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({func})"))
            .unwrap_or_else(|err| panic!("{func} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{func} must be a global function. The Show/Hide/Toggle \
             trio + RegisterUIPanel + SetUIPanelAttribute is the public \
             API addons hit. The CanShow* / CanOpen* / \
             AreAllPanelsDisallowed predicates gate the pre-show checks. \
             Close{{Child,Special,Windows,All,Menus}} are the bulk-close \
             helpers wired into ESC and on-take-control. The \
             {{Maximize,RestoreUIPanelArea}} pair powers the WorldMap \
             one-off maximize behaviour (all other consumers leave it \
             alone). UpdateUIPanelPositions is the public entry that \
             Shared/UpdateUIPanelPositions.lua hooks onto UIParent's \
             OnAttributeChanged"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_populates_ui_panel_windows_with_expected_areas(env: &WowLuaEnv) {

    let count: i64 = env
        .eval("local n=0 for _ in pairs(UIPanelWindows) do n=n+1 end return n")
        .expect("UIPanelWindows count probe");
    assert!(
        count >= 40,
        "UIPanelWindows must contain at least 40 entries after \
         UIPanelWindows_Initialize() (UIPanelWindows.lua:2-66 declares \
         ~50, plus RegisterUIPanel calls from sibling addons add more). \
         Got: {count}"
    );

    for (panel, expected_area) in REPRESENTATIVE_PANEL_WINDOWS {
        let area: String = env
            .eval(&format!(
                "return tostring(UIPanelWindows[{panel:?}] and UIPanelWindows[{panel:?}].area)"
            ))
            .unwrap_or_else(|err| panic!("{panel} probe failed: {err}"));
        assert_eq!(
            area, *expected_area,
            "UIPanelWindows[{panel}].area must be `{expected_area}`. \
             The center entries (GameMenu/Help/EditModeManager/DeathRecap/\
             PVPMatchScoreboard/GarrisonBuildingFrame) take the center \
             slot with neverAllowOtherPanels or allowOtherPanels nuance; \
             the left entries (CharacterFrame/EncounterJournal/etc.) live \
             in the left-pushable stack with pushable=N priority; the \
             full entries (CinematicFrame/BarberShopFrame/PerksProgram) \
             take the entire screen and don't co-exist with anything"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_UIParentPanelManager/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load must emit zero \
         UIParentPanelManager-body errors. The 4 body files (~1300 \
         lines total) include the 1205-line FramePositionDelegate-driven \
         panel manager and the single-line \
         `UIParent:SetScript(\"OnAttributeChanged\", \
         UpdateUIPanelPositions)` wiring. Found: {addon_specific:?}"
    );
}
}
