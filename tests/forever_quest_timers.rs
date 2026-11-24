#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_quest_timer_consumer() -> WowLuaEnv {
    let ui_dir = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui_dir);
    let loaded = crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
        &env,
        &ui_dir,
        &["Blizzard_FrameXMLBase", "Blizzard_QuestTimer"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_QuestTimer"));
    env
}

fn assert_timer_consumer_updates(env: &WowLuaEnv) {
    env.exec(
        r#"
        A_Admin.SetManualClock(0)
        A_Admin.SetQuestTimer(101, 125)
        A_Admin.SetQuestTimer(202, 305)
        local rows = QuestTimerFrame.activeElements
        assert(QuestTimerFrame:IsShown(), "timer consumer did not show its frame")
        assert(#rows == 2, "timer consumer did not create two rows")
        assert(rows[1].Name:GetText() == "2:05", "first timer text was not 2:05")
        assert(rows[2].Name:GetText() == "5:05", "second timer text was not 5:05")
        A_Admin.AdvanceTime(61)
        A_Admin.FireEvent("QUEST_LOG_UPDATE")
        assert(rows[1].Name:GetText() == "1:04", "first timer did not decrease")
        assert(rows[2].Name:GetText() == "4:04", "second timer did not decrease")
        A_Admin.SetQuestTimer(101, nil)
        assert(#QuestTimerFrame.activeElements == 1, "clearing a timer did not remove one row")
        assert(QuestTimerFrame.activeElements[1].Name:GetText() == "4:04")
        "#,
    )
    .unwrap();
}

#[test]
fn forever_quest_timer_consumer_shows_countdowns_that_advance_and_clear() {
    let env = load_quest_timer_consumer();
    assert_timer_consumer_updates(&env);
}
