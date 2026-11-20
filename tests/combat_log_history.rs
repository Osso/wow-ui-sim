#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::c_api::combat_log_history::{CombatLogEntry, CombatLogValue};
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_one_on_update_tick;

fn blizzard_ui_dir() -> std::path::PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

#[test]
fn original_processor_refilters_empty_and_host_supplied_history() {
    assert_processor_is_runtime_selected();
    let env = load_combat_log_processor();
    install_test_message_formatter(&env);
    assert_empty_history_contract(&env);
    supply_host_history_and_refilter(&env);
}

fn assert_processor_is_runtime_selected() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    assert!(
        addons
            .iter()
            .any(|(name, _)| name == "Blizzard_CombatLogProcessor"),
        "mainline CombatLogProcessor must be selected on the Game screen"
    );
}

fn load_combat_log_processor() -> WowLuaEnv {
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &blizzard_ui_dir(),
        &["Blizzard_CombatLog"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_CombatLogProcessor"));
    env
}

fn install_test_message_formatter(env: &WowLuaEnv) {
    env.exec(
        r#"
        local original = assert(rawget(__secureenv, "CombatLogProcessor"))
        local processor = CreateFromMixins(original)
        _G.__combatLogHistoryTestProcessor = processor
        processor.GenerateMessage = function(_, _, _, event)
            return event, 1, 1, 1
        end
        "#,
    )
    .expect("original processor methods should be available");
}

fn assert_empty_history_contract(env: &WowLuaEnv) {
    env.exec(
        r##"
        assert(C_CombatLogSecure.GetEntryCount() == 0)
        local emptyCount = select("#", C_CombatLogSecure.GetCurrentEntryInfo())
        local emptyEntry = C_CombatLogSecure.GetCurrentEntryInfo()
        assert(emptyCount == 1 and emptyEntry == nil, "empty current-entry is one nil")
        assert(C_CombatLogSecure.SeekToNewestEntry() == false, "empty seek returns false")
        assert(C_CombatLogSecure.ShouldShowCurrentEntry() == false)
        __combatLogHistoryTestProcessor:StartRefiltering()
        "##,
    )
    .expect("empty history should enter the original processor");
    fire_one_on_update_tick(env);
    env.exec(
        r#"
        assert(not __combatLogHistoryTestProcessor:IsRefiltering())
        assert(#C_CombatLog._state.createdMessages == 0)
        "#,
    )
    .expect("empty history should finish without output");
}

fn supply_host_history_and_refilter(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.combat_log_history.entries = vec![
        entry(1.0, "COMBATLOG_TEST_FIRST"),
        entry(2.0, "COMBATLOG_TEST_SECOND"),
    ];
    drop(sim);
    env.exec("__combatLogHistoryTestProcessor:StartRefiltering()")
        .expect("processor should start with supplied history");
    fire_one_on_update_tick(env);
    fire_one_on_update_tick(env);
    fire_one_on_update_tick(env);
    assert_distinct_history_was_replayed(env);
}

fn assert_distinct_history_was_replayed(env: &WowLuaEnv) {
    let result: bool = env
        .eval(
            r#"
            local messages = C_CombatLog._state.createdMessages
            return #messages == 2
                and messages[1].message == "COMBATLOG_TEST_SECOND"
                and messages[2].message == "COMBATLOG_TEST_FIRST"
                and not __combatLogHistoryTestProcessor:IsRefiltering()
            "#,
        )
        .expect("processor history output should be readable");
    assert!(
        result,
        "two distinct host events must traverse newest-to-oldest through Blizzard's processor"
    );
}

fn entry(timestamp: f64, event: &str) -> CombatLogEntry {
    CombatLogEntry {
        values: vec![
            CombatLogValue::Number(timestamp),
            CombatLogValue::String(event.to_owned()),
        ],
    }
}
