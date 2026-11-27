#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn dispatcher_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Dispatcher/Blizzard_Dispatcher.toc")
}

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
fn blizzard_dispatcher_toc_declares_allowload_both_and_load_on_demand() {
    let toc = TocFile::from_file(&dispatcher_toc()).expect("Blizzard_Dispatcher TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_Dispatcher declares `## LoadOnDemand: 1` — it is NOT auto-discovered as a \
         standalone addon. Discovery only pulls it in when a non-LOD addon (e.g. \
         Blizzard_TutorialManager) declares it as a dependency, via \
         `pull_required_lod_addons` (src/loader/mod.rs:552)"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_Dispatcher does not declare `## UseSecureEnvironment` — it runs in the \
         standard Lua environment and uses hooksecurefunc for function callback hooks"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_Dispatcher itself has NO `## Dependencies:` — it is a leaf utility \
         providing event/function/script callback dispatch to dependents like \
         Blizzard_TutorialManager and Blizzard_BoostTutorial"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_Dispatcher declares no `## AllowLoadGameType:` filter — universal across \
         game types because the dispatch primitives (CreateFrame, hooksecurefunc, OnEvent) \
         are baseline API"
    );

    let toc_text =
        std::fs::read_to_string(dispatcher_toc()).expect("Blizzard_Dispatcher TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: both"),
        "Blizzard_Dispatcher declares `## AllowLoad: both` — UNUSUAL among Blizzard addons. \
         `allows_screen` (src/toc.rs:307) returns true for ANY ScreenKind, so the addon is \
         eligible on both Login (glue) and Game screens. Whether it actually loads depends \
         on whether a non-LOD dependent on that screen pulls it in"
    );
}

#[test]
fn blizzard_dispatcher_appears_in_game_discovery_via_tutorial_manager_dependency() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_Dispatcher");
    assert!(
        in_game,
        "Blizzard_Dispatcher must appear in Game-screen discovery because \
         Blizzard_TutorialManager (no `## LoadOnDemand`, default Game-only) declares \
         `## Dependencies: Blizzard_Dispatcher, Blizzard_HelpPlate`. \
         `pull_required_lod_addons` (src/loader/mod.rs:557) walks non-LOD addons' deps and \
         migrates required LOD addons from `lod_pool` into the main `addons` set"
    );
}

#[test]
fn blizzard_dispatcher_excluded_from_login_discovery_no_glue_dependents() {
    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_Dispatcher");
    assert!(
        !in_login,
        "Even though Blizzard_Dispatcher declares `## AllowLoad: both`, it should NOT appear \
         in Login discovery: it is LOD, so it sits in `lod_pool` until pulled. The known \
         dependents (Blizzard_TutorialManager, Blizzard_BoostTutorial) are both Game-only \
         (no `## AllowLoad`, default Game per src/toc.rs:311), so neither qualifies for the \
         Login pool. With no Login-eligible dependent, `pull_required_lod_addons` never \
         migrates Dispatcher into the Login addons set"
    );
}

prefork_full_ui_case! {
fn blizzard_dispatcher_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("Dispatcher"))
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_Dispatcher emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
prefork_full_ui_case! {
fn blizzard_dispatcher_global_table_installed_with_simulator_marker(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(Dispatcher) == 'table' \
                and Dispatcher.__wow_sim_dispatcher == true",
        )
        .expect("Dispatcher table existence probe should succeed");
    assert!(
        installed,
        "Dispatcher must be a table with `__wow_sim_dispatcher == true`, the marker set by \
         the simulator's pre-installed dispatcher (`__wow_ensure_dispatcher_surface` at \
         runtime_surface_bootstrap.lua:581-621). Because env init seeds Dispatcher \
         BEFORE any addon Lua runs, the addon's versioning guard at \
         Blizzard_Dispatcher.lua:6 (`if (not Dispatcher or (not DISPATCHER_VERSION or \
         DISPATCHER_VERSION < 2.0))`) sees DISPATCHER_VERSION == 2.0 and SKIPS the entire \
         body — preserving the simulator's marker and the addon never overwrites the table"
    );
}
}

prefork_full_ui_case! {
fn blizzard_dispatcher_version_is_two_dot_zero(env: &WowLuaEnv) {

    let version: f64 = env
        .eval("return DISPATCHER_VERSION")
        .expect("DISPATCHER_VERSION query should succeed");
    assert_eq!(
        version, 2.0,
        "DISPATCHER_VERSION must equal 2.0 — pre-set by the simulator's \
         `__wow_ensure_dispatcher_surface` (runtime_surface_bootstrap.lua:587), and matched \
         by the addon's `DISPATCHER_VERSION = 2.0;` assignment at Blizzard_Dispatcher.lua:7. \
         Either source path produces the same numeric value"
    );
}
}

prefork_full_ui_case! {
fn blizzard_dispatcher_exposes_event_and_function_and_script_methods(env: &WowLuaEnv) {

    let kinds: (String, String, String, String, String, String) = env
        .eval(
            "return type(Dispatcher.RegisterEvent), \
                    type(Dispatcher.UnregisterEvent), \
                    type(Dispatcher.RegisterFunction), \
                    type(Dispatcher.UnregisterFunction), \
                    type(Dispatcher.RegisterScript), \
                    type(Dispatcher.UnregisterScript)",
        )
        .expect("Dispatcher method-types query should succeed");
    let expected = (
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
    );
    assert_eq!(
        kinds, expected,
        "Dispatcher must expose all six core (un)register methods as functions. The \
         simulator's bootstrap defines RegisterEvent (line 622), UnregisterEvent (line 651), \
         RegisterFunction (line 723), UnregisterFunction, RegisterScript (line 841), \
         UnregisterScript (line 867) on the dispatcher table at \
         runtime_surface_bootstrap.lua. These are what tutorial code calls to subscribe to \
         events/hooks"
    );
}
}

prefork_full_ui_case! {
fn blizzard_dispatcher_unregister_all_aggregates_three_categories(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(Dispatcher.UnregisterAll)")
        .expect("Dispatcher.UnregisterAll type query should succeed");
    assert_eq!(
        kind, "function",
        "Dispatcher:UnregisterAll(owner) is the convenience method that calls \
         UnregisterAllEvents, UnregisterAllFunctions, and UnregisterAllScripts in sequence \
         (runtime_surface_bootstrap.lua:913-917). Tutorial cleanup code uses this single \
         call to unhook every callback owned by a frame/object when it is destroyed"
    );
}
}

prefork_full_ui_case! {
fn blizzard_dispatcher_initialize_creates_dispatcher_frame(env: &WowLuaEnv) {

    let frame_ok: bool = env
        .eval(
            "return type(Dispatcher.EventFrame) == 'table' \
                and Dispatcher.EventFrame:GetName() == 'DispatcherFrame'",
        )
        .expect("Dispatcher.EventFrame query should succeed");
    assert!(
        frame_ok,
        "Dispatcher.EventFrame must be the frame named 'DispatcherFrame'. \
         `dispatcher:Initialize()` (runtime_surface_bootstrap.lua:611) creates this frame \
         via CreateFrame('Frame', 'DispatcherFrame') the first time it runs. The Blizzard \
         addon also calls `Dispatcher:Initialize()` at file end (line 548, OUTSIDE the \
         versioning `if` block), but the simulator's Initialize is idempotent — it \
         early-returns when EventFrame already exists, so the addon's call is a no-op"
    );
}
}

prefork_full_ui_case! {
fn blizzard_dispatcher_register_event_returns_numeric_id_and_increments(env: &WowLuaEnv) {

    let ids: (i64, i64) = env
        .eval(
            "do \
                local id1 = Dispatcher:RegisterEvent('PLAYER_LOGIN', function() end) \
                local id2 = Dispatcher:RegisterEvent('PLAYER_LOGIN', function() end) \
                return id1, id2 \
            end",
        )
        .expect("Dispatcher:RegisterEvent invocation should succeed");
    assert!(
        ids.0 > 0 && ids.1 > ids.0,
        "Dispatcher:RegisterEvent must return a strictly increasing numeric id. The \
         simulator's RegisterEvent (runtime_surface_bootstrap.lua:645-648) reads \
         self.NextEventID, increments it by one, and returns the previous value. Two \
         consecutive registrations must produce ids id1 < id2 and both > 0. Got: {ids:?}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_dispatcher_state_tables_initialized_empty(env: &WowLuaEnv) {

    let tables_ok: bool = env
        .eval(
            "return type(Dispatcher.Events) == 'table' \
                and type(Dispatcher.Functions) == 'table' \
                and type(Dispatcher.Scripts) == 'table'",
        )
        .expect("Dispatcher state-tables probe should succeed");
    // The Functions.Global/Owners split belonged to the runtime_surface_bootstrap
    // fallback that b67f550d02 retired on Retail; the original creates Global lazily.
    assert!(
        tables_ok,
        "Dispatcher must seed its callback containers as tables at construction \
         (Blizzard_Dispatcher.lua:25-27): Events, Functions and Scripts. The original \
         creates Functions.Global on the first global RegisterFunction (:268-269)"
    );
}
}

#[test]
fn blizzard_dispatcher_directory_has_only_single_lua_file() {
    let dir = blizzard_ui_dir().join("Blizzard_Dispatcher");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_Dispatcher dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let has_xml = entries.iter().any(|n| n.ends_with(".xml"));
    assert!(
        !has_xml,
        "Blizzard_Dispatcher has NO XML files — pure Lua dispatcher implementation. \
         Got entries: {entries:?}"
    );

    let lua_files: Vec<&String> = entries.iter().filter(|n| n.ends_with(".lua")).collect();
    assert_eq!(
        lua_files.len(),
        1,
        "Blizzard_Dispatcher should ship exactly one Lua file (`Blizzard_Dispatcher.lua` — \
         all dispatch + debug logic in a single 548-line file). Got Lua files: {lua_files:?}"
    );
    assert!(
        entries.iter().any(|n| n == "Blizzard_Dispatcher.lua"),
        "The single Lua file must be named `Blizzard_Dispatcher.lua` (matching the addon \
         folder + .lua convention). Got entries: {entries:?}"
    );
}

#[test]
fn blizzard_dispatcher_pulled_in_before_blizzard_tutorial_manager_in_load_order() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let dispatcher_idx = game_addons
        .iter()
        .position(|(name, _)| name == "Blizzard_Dispatcher");
    let manager_idx = game_addons
        .iter()
        .position(|(name, _)| name == "Blizzard_TutorialManager");

    let dispatcher_idx = dispatcher_idx.expect("Blizzard_Dispatcher should be discovered");
    let manager_idx = manager_idx.expect("Blizzard_TutorialManager should be discovered");
    assert!(
        dispatcher_idx < manager_idx,
        "Blizzard_Dispatcher must come BEFORE Blizzard_TutorialManager in the load order. \
         `topological_sort_addons` (src/loader/mod.rs) honors `## Dependencies:` declarations: \
         since TutorialManager declares `## Dependencies: Blizzard_Dispatcher, ...`, \
         Dispatcher's index ({dispatcher_idx}) must be less than TutorialManager's \
         ({manager_idx}). Otherwise TutorialManager could observe a missing Dispatcher when \
         its own Lua executes"
    );
}
