#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::c_api::combat_log_history::{CombatLogEntry, CombatLogValue};
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

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
    assert_filter_settings_are_rust_backed(&env);
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
        &["Blizzard_CombatLogProcessor"],
        &[],
    );
    assert!(
        loaded
            .iter()
            .any(|name| name == "Blizzard_CombatLogProcessor")
    );
    env
}

fn install_test_message_formatter(env: &WowLuaEnv) {
    env.exec(
        r#"
        local original = assert(rawget(__secureenv, "CombatLogProcessor"))
        local secure = assert(getfenv(original.StartRefiltering).C_CombatLogSecure)
        _G.__combatLogHistoryTestSecure = secure
        _G.__combatLogHistoryOriginalProcessor = original
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
    assert_empty_api_results(env);
    env.exec("__combatLogHistoryTestProcessor:StartRefiltering()")
        .expect("empty history should enter the original processor");
    env.process_timers()
        .expect("zero-delay empty-history ticker should fire");
    assert_empty_refilter_completed(env);
}

fn assert_empty_api_results(env: &WowLuaEnv) {
    env.exec(
        r##"
        assert(C_CombatLog.GetEntryCount() == 0)
        assert(C_CombatLog.GetCurrentEntryInfo() == 0)
        assert(C_CombatLog.GetEntryRetentionTime() == 300)
        assert(C_CombatLog.GetMessageLimit() == 300)
        assert(C_CombatLog.AreFilteredEventsEnabled() == false)
        assert(__combatLogHistoryTestSecure.GetEntryCount() == 0)
        local entryCount = select("#", __combatLogHistoryTestSecure.GetCurrentEntryInfo())
        local entry = __combatLogHistoryTestSecure.GetCurrentEntryInfo()
        assert(entryCount == 1 and entry == nil, "empty current-entry is one nil")
        local eventCount = select("#", __combatLogHistoryTestSecure.GetCurrentEventInfo())
        local event = __combatLogHistoryTestSecure.GetCurrentEventInfo()
        assert(eventCount == 1 and event == nil, "empty current-event is one nil")
        assert(__combatLogHistoryTestSecure.SeekToNewestEntry() == false)
        assert(__combatLogHistoryTestSecure.SeekToPreviousEntry() == false)
        assert(C_CombatLog.ShouldShowCurrentEntry() == false)
        "##,
    )
    .expect("empty history API results should match the previous fixture");
}

fn assert_empty_refilter_completed(env: &WowLuaEnv) {
    env.exec(
        r#"
        local state = tostring(__combatLogHistoryTestProcessor:IsRefiltering())
            .. ":" .. tostring(#C_CombatLog._state.createdMessages)
        assert(state == "false:0", "empty-history output was " .. state)
        "#,
    )
    .expect("empty history should finish without output");
}

fn assert_filter_settings_are_rust_backed(env: &WowLuaEnv) {
    env.exec(
        r#"
        local settings = {
            settings = {},
            filters = {
                {
                    eventList = { SPELL_DAMAGE = true },
                    sourceFlags = { [1] = true },
                    destFlags = { [1] = true },
                },
            },
        }
        C_CombatLog.ApplyFilterSettings(settings)
        assert(C_CombatLog._state.filterSettings == settings)
        assert(__wow_combat_log_get_filter_settings() == settings)
        assert(__combatLogHistoryOriginalProcessor:GetFilterSettings() == settings)
        "#,
    )
    .expect("filter settings should be retained by the Rust history model");
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
    env.process_timers()
        .expect("first populated-history ticker should fire");
    env.process_timers()
        .expect("second populated-history ticker should fire");
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
                and __combatLogHistoryTestProcessor:IsRefiltering()
            "#,
        )
        .expect("processor history output should be readable");
    assert!(
        result,
        "two distinct host events must traverse newest-to-oldest through Blizzard's processor"
    );
    env.exec("__combatLogHistoryTestProcessor:StopRefiltering()")
        .expect("test processor ticker should stop after two observed entries");
}

fn entry(timestamp: f64, event: &str) -> CombatLogEntry {
    CombatLogEntry {
        values: vec![
            CombatLogValue::Number(timestamp),
            CombatLogValue::String(event.to_owned()),
        ],
    }
}
