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

fn time_manager_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_TimeManager")
}

fn time_manager_toc() -> PathBuf {
    time_manager_dir().join("Blizzard_TimeManager_Mainline.toc")
}

const ALL_FOUR_SCREENS: &[ScreenKind] = &[
    ScreenKind::Game,
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TIME_MANAGER_GLOBAL_FUNCTIONS: &[&str] = &[
    "TimeManager_Toggle",
    "TimeManager_LoadUI",
    "TimeManager_Update",
    "TimeManager_UpdateAlarmTime",
    "TimeManager_UpdateTimeTicker",
    "TimeManager_ToggleTimeFormat",
    "TimeManager_ToggleLocalTime",
    "TimeManager_ShouldCheckAlarm",
    "TimeManager_StartCheckingAlarm",
    "TimeManager_CheckAlarm",
    "TimeManager_FireAlarmWarning",
    "TimeManager_FireAlarm",
    "TimeManager_TurnOffAlarm",
    "TimeManager_IsAlarmFiring",
];

const STOPWATCH_GLOBAL_FUNCTIONS: &[&str] = &[
    "Stopwatch_Toggle",
    "Stopwatch_StartCountdown",
    "Stopwatch_Play",
    "Stopwatch_Pause",
    "Stopwatch_IsPlaying",
    "Stopwatch_Clear",
    "Stopwatch_FinishCountdown",
];

const NAMED_TOPLEVEL_FRAMES: &[&str] = &[
    "TimeManagerFrame",
    "TimeManagerClockButton",
    "StopwatchFrame",
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
fn find_toc_file_resolves_mainline_flavor_toc() {
    let resolved = find_toc_file(&time_manager_dir()).expect("TimeManager TOC resolves");
    assert_eq!(
        resolved,
        time_manager_toc(),
        "Mainline flavor TOC — addon ships ONLY \
         `Blizzard_TimeManager_Mainline.toc` (no bare TOC). \
         find_toc_file at src/loader/mod.rs:65-95 tries \
         `_Mainline.toc` first and resolves it directly"
    );
}

#[test]
fn toc_is_load_on_demand_with_current_dependencies() {
    let toc = TocFile::from_file(&time_manager_toc()).expect("TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "`## LoadOnDemand: 1` — clock + stopwatch panel only loads \
         when the user clicks the minimap clock (TimeManager_LoadUI \
         calls UIParentLoadAddOn) or runs /tm /timer /stopwatch slash \
         commands"
    );
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_FrameXMLUtil".to_string(),
            "Blizzard_RaidWarning".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "Retail 12.1.0.69497 declares TimeManager dependencies in published order"
    );
    assert_eq!(
        toc.optional_deps(),
        vec!["Blizzard_Plunderstorm".to_string()],
        "Retail 12.1.0.69497 declares Blizzard_Plunderstorm as the optional dependency"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: mainline` — toc.rs:294-302 permits \
         `mainline`/`standard` so is_game_type_restricted MUST return \
         false"
    );
    assert!(toc.default_enabled());
}

#[test]
fn saved_variables_per_character_published_for_stopwatch_options() {
    let toc = TocFile::from_file(&time_manager_toc()).expect("TOC parses");

    assert!(
        toc.saved_variables().is_empty(),
        "`## SavedVariablesPerCharacter` is NOT read by \
         saved_variables() — toc.rs:316-328 only reads SavedVariables \
         + SavedVariablesMachine. Got: {:?}",
        toc.saved_variables()
    );
    assert_eq!(
        toc.saved_variables_per_character(),
        vec!["BlizzardStopwatchOptions".to_string()],
        "`## SavedVariablesPerCharacter: BlizzardStopwatchOptions` — \
         per-character SV (alarm time, alarm message, military/local \
         time toggle) since each character can have different \
         clock/alarm preferences. Got: {:?}",
        toc.saved_variables_per_character()
    );
}

#[test]
fn allow_load_game_restricts_to_game_screen_only() {
    let toc = TocFile::from_file(&time_manager_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (case-insensitive at toc.rs:308) → \
         Game-screen only — clock/stopwatch tied to GetGameTime + \
         minimap presence which only exist in-world"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must be excluded — minimap clock + \
             alarm system are in-world only"
        );
    }
}

#[test]
fn toc_raw_bytes_pin_current_metadata_directives_and_bootstrap() {
    let raw = std::fs::read_to_string(time_manager_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard Time Manager",
        "## Author: Blizzard Entertainment",
        "## LoadOnDemand: 1",
        "## Dependencies: Blizzard_FrameXMLUtil, Blizzard_RaidWarning, Blizzard_GameMenuEsc",
        "## OptionalDeps: Blizzard_Plunderstorm",
        "## SavedVariablesPerCharacter: BlizzardStopwatchOptions",
        "## AllowLoad: game",
        "## AllowLoadGameType: mainline",
        "Blizzard_TimeManager_Bootstrap.lua [Bootstrap]",
        "Mainline\\Blizzard_TimeManager.lua",
        "Mainline\\Blizzard_TimeManager.xml",
        "Mainline\\Localization.lua",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin current TimeManager metadata and body entry `{directive}`"
        );
    }

    assert!(!raw.contains("## SavedVariables:"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## DefaultState"));
}

#[test]
fn body_resolves_bootstrap_and_three_entries_with_normalized_slashes() {
    let toc = TocFile::from_file(&time_manager_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = vec![
        "Blizzard_TimeManager_Bootstrap.lua".to_string(),
        "Mainline/Blizzard_TimeManager.lua".to_string(),
        "Mainline/Blizzard_TimeManager.xml".to_string(),
        "Mainline/Localization.lua".to_string(),
    ];

    assert_eq!(
        body, expected,
        "Body must resolve to the bootstrap followed by the three declared Mainline entries. \
         Got: {body:?}"
    );
}

#[test]
fn appears_in_game_discovery_via_blizzard_game_implicit_dependency() {
    for screen in ALL_FOUR_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_TimeManager");
        assert_eq!(
            found,
            *screen == ScreenKind::Game,
            "TimeManager should appear only for {screen:?}: Blizzard_Game pulls this LoD addon \
             through the current implicit startup dependency while AllowLoad: game excludes \
             glue screens"
        );
    }
}

#[test]
fn no_mainline_addon_declares_time_manager_as_dependency() {
    let entries = std::fs::read_dir(blizzard_ui_dir()).expect("BlizzardUI dir reads");
    let mut declarers: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let addon_dir = entry.path();
        if !addon_dir.is_dir() {
            continue;
        }
        let Some(toc_path) = find_toc_file(&addon_dir) else {
            continue;
        };
        let Ok(toc) = TocFile::from_file(&toc_path) else {
            continue;
        };
        if toc
            .dependencies()
            .iter()
            .any(|d| d == "Blizzard_TimeManager")
            || toc
                .optional_deps()
                .iter()
                .any(|d| d == "Blizzard_TimeManager")
        {
            let name = addon_dir.file_name().unwrap().to_string_lossy().to_string();
            declarers.push(name);
        }
    }

    assert!(
        declarers.is_empty(),
        "No mainline-discoverable Blizzard addon may declare \
         Blizzard_TimeManager as a Dependency or OptionalDep — it is \
         strictly UI-triggered via TimeManager_LoadUI's \
         UIParentLoadAddOn at runtime, not at boot. Found declarers: \
         {declarers:?}"
    );
}

prefork_full_ui_case! {
fn time_manager_load_ui_global_published_at_boot(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(TimeManager_LoadUI)")
        .expect("TimeManager_LoadUI probe");
    assert_eq!(
        kind, "function",
        "TimeManager_LoadUI must be published by Blizzard_UIParent at \
         boot (Shared/UIParent.lua line 299) — wraps \
         UIParentLoadAddOn(\"Blizzard_TimeManager\") and is the entry \
         point used by minimap clock OnClick + slash commands"
    );
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_time_manager_globals(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &time_manager_toc())
        .expect("Blizzard_TimeManager must load via Rust loader");

    for func in TIME_MANAGER_GLOBAL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({func})"))
            .unwrap_or_else(|err| panic!("{func} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{func} must be a function after explicit LoD load — \
             covers TimeManager toggle/update/alarm-checking/format \
             toggle public API consumed by UIParent's slash command \
             handlers + minimap clock button"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_stopwatch_globals(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &time_manager_toc())
        .expect("Blizzard_TimeManager must load via Rust loader");

    for func in STOPWATCH_GLOBAL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({func})"))
            .unwrap_or_else(|err| panic!("{func} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{func} must be a function after explicit LoD load — \
             stopwatch public API consumed by ChatFrameBase \
             SlashCommands.lua (/stopwatch slash command + \
             /sw play/pause/clear) and Stopwatch_StartCountdown for \
             /timer NN:NN:NN"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_creates_three_named_toplevel_frames(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &time_manager_toc())
        .expect("Blizzard_TimeManager must load via Rust loader");

    for frame_name in NAMED_TOPLEVEL_FRAMES {
        let exists: bool = env
            .eval(&format!("return _G['{frame_name}'] ~= nil"))
            .unwrap_or_else(|err| panic!("{frame_name} probe failed: {err}"));
        assert!(
            exists,
            "{frame_name} must exist as a named global after LoD load \
             — TimeManagerFrame (alarm config dialog), \
             TimeManagerClockButton (minimap clock — parented to \
             MinimapCluster), StopwatchFrame (movable countdown timer \
             — toplevel DIALOG strata)"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &time_manager_toc())
        .expect("Blizzard_TimeManager must load via Rust loader");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let needles = [
        "Blizzard_TimeManager",
        "TimeManagerFrame",
        "TimeManagerClockButton",
        "StopwatchFrame",
        "TimeManager_",
        "Stopwatch_",
        "BlizzardStopwatchOptions",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Explicit load_addon for TimeManager must emit zero \
         addon-specific Lua errors. Found {} matching: {:#?}",
        matched.len(),
        matched
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_true_after_player_login_auto_load(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_TimeManager')")
        .expect("IsAddOnLoaded probe");
    assert!(
        loaded,
        "Although LoadOnDemand=1 excludes TimeManager from the eager \
         sweep AND no TOC declares it as a Dependency, \
         Blizzard_UIParent's PLAYER_LOGIN handler at UIParent.lua \
         line 1326 unconditionally calls TimeManager_LoadUI() which \
         in turn invokes UIParentLoadAddOn(\"Blizzard_TimeManager\"). \
         load_full_game_ui fires PLAYER_LOGIN, so by the time \
         IsAddOnLoaded probes the state, the addon HAS been loaded \
         via the runtime auto-load path"
    );
}
}
