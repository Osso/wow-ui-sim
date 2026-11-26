use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn objective_tracker_renders_two_watched_seed_quests() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_ObjectiveTracker", "Blizzard_SplashFrame"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_SplashFrame"));
            supply_watched_quests(env);
            advance_tracker(env);
            assert_seeded_quest_blocks(env);
        },
    );
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn supply_watched_quests(env: &WowLuaEnv) {
    env.exec(
        r#"
        C_QuestLog.AddQuestWatch(80000)
        C_QuestLog.AddQuestWatch(80001)
        A_Admin.FireEvent("QUEST_WATCH_LIST_CHANGED", 80000, true)
        A_Admin.FireEvent("QUEST_WATCH_LIST_CHANGED", 80001, true)
        local watched = {}
        for index = 1, C_QuestLog.GetNumQuestWatches() do
            watched[C_QuestLog.GetQuestIDForQuestWatchIndex(index)] = true
        end
        assert(watched[80000] and watched[80001], "both supplied quests must be watched")
        "#,
    )
    .expect("client APIs and admin events should supply both watched quests");
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn advance_tracker(env: &WowLuaEnv) {
    wow_ui_sim::startup::process_pending_timers(env);
    for _ in 0..3 {
        wow_ui_sim::startup::fire_one_on_update_tick(env);
        wow_ui_sim::startup::process_pending_timers(env);
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_seeded_quest_blocks(env: &WowLuaEnv) {
    env.exec(
        r#"
        local blocks = QuestObjectiveTracker.usedBlocks[QuestObjectiveTracker.blockTemplate]
        assert(blocks[80000] and blocks[80000].HeaderText:GetText() == "The Lost Expedition")
        assert(blocks[80001] and blocks[80001].HeaderText:GetText() == "Defending the Gates")
        "#,
    )
    .expect("the original tracker should render both seeded quest titles");
}
