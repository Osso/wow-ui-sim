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

fn assert_timer_consumer_updates(env: &WowLuaEnv, selected_quest_id: i32) {
    env.exec(&format!(
        r#"
        A_Admin.SetManualClock(0)
        A_Admin.SetQuestTimer({selected_quest_id}, 125)
        A_Admin.AdvanceTime(10)
        A_Admin.SetQuestTimer(202, 305)
        assert(GetQuestLogTimeLeft() == 114, "legacy timer should subtract one second")
        local rows = QuestTimerFrame.activeElements
        assert(QuestTimerFrame:IsShown(), "timer consumer did not show its frame")
        assert(#rows == 2, "timer consumer did not create two rows")
        assert(rows[1].Name:GetText() == SecondsToTime(115), "first timer text mismatch")
        assert(rows[2].Name:GetText() == SecondsToTime(305), "second timer text mismatch")
        assert(rows[1].Name:GetText() ~= rows[2].Name:GetText(), "timer rows were not distinct")
        A_Admin.AdvanceTime(61)
        A_Admin.FireEvent("QUEST_LOG_UPDATE")
        assert(rows[1].Name:GetText() == SecondsToTime(54), "first timer did not decrease")
        assert(rows[2].Name:GetText() == SecondsToTime(244), "second timer did not decrease")
        assert(GetQuestLogTimeLeft() == 53, "legacy timer did not advance with the clock")
        A_Admin.AdvanceTime(100)
        assert(GetQuestLogTimeLeft() == 0, "expired legacy timer should clamp to zero")
        A_Admin.SetQuestTimer({selected_quest_id}, nil)
        assert(#QuestTimerFrame.activeElements == 1, "clearing a timer did not remove one row")
        assert(QuestTimerFrame.activeElements[1].Name:GetText() == SecondsToTime(144), "second timer continued after the first was cleared")
        "#,
    ))
    .unwrap();
}

#[test]
fn forever_quest_timer_consumer_shows_countdowns_that_advance_and_clear() {
    let env = load_quest_timer_consumer();
    let selected_quest_id = env
        .state()
        .borrow()
        .quest_log_entries
        .entries
        .first()
        .expect("the quest consumer fixture has a selected quest row")
        .quest_id;
    env.state().borrow_mut().selected_quest_log_id = Some(selected_quest_id as u32);
    assert_timer_consumer_updates(&env, selected_quest_id);
    let (count, is_nil): (i32, bool) = env
        .eval("local n = select('#', GetQuestLogTimeLeft()); return n, GetQuestLogTimeLeft() == nil")
        .unwrap();
    assert_eq!(count, 1);
    assert!(is_nil);
}
