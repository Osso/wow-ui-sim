#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
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
fn blizzard_combat_log_processor_toc_declares_secure_environment() {
    let toc_path =
        blizzard_ui_dir().join("Blizzard_CombatLogProcessor/Blizzard_CombatLogProcessor.toc");
    let toc = TocFile::from_file(&toc_path).expect("CombatLogProcessor TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_CombatLogProcessor is a non-LOD addon (it should auto-load on Game screen)"
    );
    assert!(
        toc.is_secure_env(),
        "Blizzard_CombatLogProcessor declares `## UseSecureEnvironment: 1` so its top-level \
         globals (CombatLogProcessor, CombatLogOutbound) live in __secureenv unless the file \
         calls SwapToGlobalEnvironment() first"
    );
    let deps = toc.dependencies();
    assert!(
        deps.contains(&"Blizzard_CombatLogBase".to_string()),
        "Blizzard_CombatLogProcessor should declare `## Dependencies: Blizzard_CombatLogBase` \
         (so its `Enum.CombatLogObject`/CombatLogUtil references resolve), got {deps:?}"
    );
}

#[test]
fn blizzard_combat_log_processor_appears_in_game_discovery() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);

    let discovered = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_CombatLogProcessor");
    assert!(
        discovered,
        "Blizzard_CombatLogProcessor should appear in Game-screen discovery \
         (`## AllowLoadGameType: mainline`, non-LOD)"
    );
}

prefork_full_ui_case! {
fn blizzard_combat_log_processor_loads_without_errors(env: &WowLuaEnv) {

    let processor_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("Blizzard_CombatLogProcessor"))
        .cloned()
        .collect();
    assert!(
        processor_errors.is_empty(),
        "Blizzard_CombatLogProcessor emitted Lua errors during load:\n  {}",
        processor_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_combat_log_base_replays_util_into_secure_environment(env: &WowLuaEnv) {

    let util_replayed: bool = env
        .eval(
            "local secureUtil = rawget(__secureenv, 'CombatLogUtil'); \
             return type(_G.CombatLogUtil) == 'table' \
                and type(secureUtil) == 'table' \
                and type(secureUtil.GenerateDamageResultString) == 'function'",
        )
        .expect("CombatLogUtil secure replay query should succeed");
    assert!(
        util_replayed,
        "Blizzard_CombatLogBase should publish CombatLogUtil in both _G and __secureenv"
    );
}
}

prefork_full_ui_case! {
fn blizzard_combat_log_processor_table_and_methods_are_defined(env: &WowLuaEnv) {

    let processor_present: bool = env
        .eval(
            "local function lookup(name) \
               return _G[name] or (__secureenv and rawget(__secureenv, name)) \
             end; \
             local p = lookup('CombatLogProcessor'); \
             return type(p) == 'table' \
                and type(p.OnLoad) == 'function' \
                and type(p.GetFilterSettings) == 'function' \
                and type(p.SetFilterSettings) == 'function' \
                and type(p.StartRefiltering) == 'function' \
                and type(p.StopRefiltering) == 'function' \
                and type(p.IsRefiltering) == 'function' \
                and type(p.RefilterEntries) == 'function' \
                and type(p.ProcessCurrentCombatEvent) == 'function' \
                and type(p.ProcessCurrentEntry) == 'function' \
                and type(p.ApplyFilterSettings) == 'function' \
                and type(p.GenerateMessage) == 'function'",
        )
        .expect("CombatLogProcessor query should succeed");
    assert!(
        processor_present,
        "CombatLogProcessor (defined in the secure environment) and its 11 methods \
         (OnLoad/GetFilterSettings/SetFilterSettings/StartRefiltering/StopRefiltering/\
         IsRefiltering/RefilterEntries/ProcessCurrentCombatEvent/ProcessCurrentEntry/\
         ApplyFilterSettings/GenerateMessage) should be defined after load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_combat_log_processor_inbound_namespace_is_in_global_env(env: &WowLuaEnv) {

    let inbound_present: bool = env
        .eval(
            "return type(_G.CombatLogInbound) == 'table' \
                and type(_G.CombatLogInbound.GenerateMessage) == 'function'",
        )
        .expect("CombatLogInbound query should succeed");
    assert!(
        inbound_present,
        "CombatLogInbound and CombatLogInbound.GenerateMessage should be visible directly in \
         _G — Blizzard_CombatLogProcessorInbound.lua calls SwapToGlobalEnvironment() before \
         declaring the namespace, so addons (insecure code) can call it without going through \
         the secure environment"
    );
}
}

prefork_full_ui_case! {
fn blizzard_combat_log_processor_outbound_namespace_is_securecall_wrapped(env: &WowLuaEnv) {

    let outbound_present: bool = env
        .eval(
            "local function lookup(name) \
               return _G[name] or (__secureenv and rawget(__secureenv, name)) \
             end; \
             local o = lookup('CombatLogOutbound'); \
             return type(o) == 'table' \
                and type(o.SignalRefilterStarted) == 'function' \
                and type(o.SignalRefilterUpdate) == 'function' \
                and type(o.SignalRefilterFinished) == 'function'",
        )
        .expect("CombatLogOutbound query should succeed");
    assert!(
        outbound_present,
        "CombatLogOutbound (registered into the secure environment via RegisterOutboundNamespace, \
         which re-wraps each function in securecallfunction) should expose its three \
         SignalRefilter* helpers after load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_combat_log_processor_outbound_functions_run_without_error(env: &WowLuaEnv) {

    let invocation_ok: bool = env
        .eval(
            "local function lookup(name) \
               return _G[name] or (__secureenv and rawget(__secureenv, name)) \
             end; \
             local o = lookup('CombatLogOutbound'); \
             local ok_started = pcall(o.SignalRefilterStarted); \
             local ok_progress = pcall(o.SignalRefilterUpdate, 0.5); \
             local ok_finished = pcall(o.SignalRefilterFinished); \
             return ok_started and ok_progress and ok_finished",
        )
        .expect("Outbound invocation query should succeed");
    assert!(
        invocation_ok,
        "Calling CombatLogOutbound.SignalRefilter{{Started,Update,Finished}} should run without \
         error — each is a securecallfunction wrapper around an EventRegistry:TriggerEvent call \
         on an event that EventRegistry has accepted via SetUndefinedEventsAllowed(true)"
    );
}
}
