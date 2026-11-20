#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::c_api::combat_log_history::{CombatLogEntry, CombatLogValue};
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_one_on_update_tick;

fn blizzard_ui_dir() -> std::path::PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

#[test]
fn original_processor_refilters_empty_and_host_supplied_history() {
    use crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env;

    let ui = blizzard_ui_dir();
    let discovered = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        discovered
            .iter()
            .any(|(name, _)| name == "Blizzard_CombatLogProcessor"),
        "the mainline CombatLogProcessor TOC must be runtime-selected on the Game screen"
    );

    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &["Blizzard_CombatLog"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_CombatLogProcessor"));
    env.exec(
        r#"
        local original = assert(rawget(__secureenv, "CombatLogProcessor"))
        local processor = CreateFromMixins(original)
        _G.__combatLogHistoryTestProcessor = processor
        processor.GenerateMessage = function(_, _, _, event)
            return event, 1, 1, 1
        end
        assert(C_CombatLogSecure.GetEntryCount() == 0)
        local emptyCount, emptyEntry = select("#", C_CombatLogSecure.GetCurrentEntryInfo()), C_CombatLogSecure.GetCurrentEntryInfo()
        assert(emptyCount == 1 and emptyEntry == nil, "empty current-entry result is one nil")
        local seekResult = C_CombatLogSecure.SeekToNewestEntry()
        assert(seekResult == false, "empty newest seek returns false")
        assert(C_CombatLogSecure.ShouldShowCurrentEntry() == false)
        processor:StartRefiltering()
        "#,
    )
    .expect("original processor should start empty-history refiltering");
    fire_one_on_update_tick(&env);
    env.exec(
        r#"
        assert(not __combatLogHistoryTestProcessor:IsRefiltering(), "empty refilter should finish")
        assert(#C_CombatLog._state.createdMessages == 0, "empty history emits no messages")
        "#,
    )
    .expect("original processor should finish empty-history refiltering");

    {
        let mut sim = env.state().borrow_mut();
        sim.combat_log_history.entries = vec![
            entry(1.0, "COMBATLOG_TEST_FIRST"),
            entry(2.0, "COMBATLOG_TEST_SECOND"),
        ];
    }
    env.exec("__combatLogHistoryTestProcessor:StartRefiltering()")
        .expect("original processor should start populated-history refiltering");
    fire_one_on_update_tick(&env);
    fire_one_on_update_tick(&env);
    fire_one_on_update_tick(&env);
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
        .expect("original processor history result should be readable");
    assert!(
        result,
        "two distinct host events must traverse the original processor newest-to-oldest"
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
