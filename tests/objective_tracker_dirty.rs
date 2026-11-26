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
    let (first_exists, first_title, second_exists, second_title): (bool, String, bool, String) = env
        .eval(
            r#"
            local blocks = QuestObjectiveTracker.usedBlocks[QuestObjectiveTracker.blockTemplate]
            local first = blocks and blocks[80000]
            local second = blocks and blocks[80001]
            return first ~= nil, first and first.HeaderText:GetText() or "",
                   second ~= nil, second and second.HeaderText:GetText() or ""
            "#,
        )
        .expect("the original tracker block state should be readable");
    let expected_titles = if cfg!(feature = "client-wowforever") {
        ("[80] The Lost Expedition", "[80] Defending the Gates")
    } else {
        ("The Lost Expedition", "Defending the Gates")
    };
    assert!(first_exists, "quest 80000 should have a rendered tracker block");
    assert_eq!(first_title, expected_titles.0);
    assert!(second_exists, "quest 80001 should have a rendered tracker block");
    assert_eq!(second_title, expected_titles.1);
}
