#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;
fn quest_timer_env() -> (WowLuaEnv, Vec<String>) {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Forever Blizzard UI cache should be available");
    crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_QuestTimer"],
        &[],
    )
}

#[test]
fn forever_quest_timer_frame_renders_supplied_countdowns() {
    let (env, loaded) = quest_timer_env();
    assert!(loaded.iter().any(|name| name == "Blizzard_QuestTimer"));

    env.exec(
        r#"
        A_Admin.SetManualClock(0)
        A_Admin.SetQuestTimers({
            { questID = 41001, questTimer = 90 },
            { questID = 41002, questTimer = 150 },
        })
        QuestTimerFrame:Update()
        "#,
    )
    .expect("supply timers to the original QuestTimerFrame");
    let (count, first, second, first_id, second_id, first_expected, second_expected): (
        i32,
        String,
        String,
        i32,
        i32,
        String,
        String,
    ) = env
        .eval(
            "local first, second = QuestTimerFrame.activeElements[1], QuestTimerFrame.activeElements[2]; return QuestTimerFrame.numTimers, first and first.Name:GetText() or '', second and second.Name:GetText() or '', first and first.questID or 0, second and second.questID or 0, SecondsToTime(90), SecondsToTime(150)",
        )
        .expect("read rendered timer rows");
    assert_eq!(count, 2, "both supplied timers must render as rows");
    assert_eq!((first_id, second_id), (41001, 41002));
    assert_eq!((first, second), (first_expected, second_expected));

    env.exec("A_Admin.AdvanceTime(10)")
        .expect("advance the simulator clock");
    let (first_after, second_after, first_expected, second_expected): (
        String,
        String,
        String,
        String,
    ) = env
        .eval(
            "return QuestTimerFrame.activeElements[1].Name:GetText(), QuestTimerFrame.activeElements[2].Name:GetText(), SecondsToTime(80), SecondsToTime(140)",
        )
        .expect("read timer rows after clock update");
    assert_eq!((first_after, second_after), (first_expected, second_expected));
}
