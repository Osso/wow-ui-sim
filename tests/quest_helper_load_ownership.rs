//! Original-provider preservation through the ObjectiveTracker client lifecycle.
#![cfg(feature = "optional-tests")]

#[test]
fn normal_bootstrap_owns_quest_classification_enums() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("normal environment");
    let values: (i64, i64) = env
        .eval("return Enum.QuestClassification.Campaign, Enum.QuestClassification.Calling")
        .expect("normal enum provider");
    assert_eq!(values, (2, 3));
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn admin_quest_log_entry_setter_adds_two_distinct_rows() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("normal environment");
    env.exec("A_Admin.SetQuestLogEntry(990001, 'First', false); A_Admin.SetQuestLogEntry(990002, 'Second', true)")
        .expect("host should add both quest rows");
    let values: (String, String, bool, bool) = env
        .eval("local a = C_QuestLog.GetInfo(C_QuestLog.GetLogIndexForQuestID(990001)); local b = C_QuestLog.GetInfo(C_QuestLog.GetLogIndexForQuestID(990002)); return a.title, b.title, C_QuestLog.IsComplete(990001), C_QuestLog.IsComplete(990002)")
        .expect("quest queries should read both supplied rows");
    assert_eq!(values, ("First".into(), "Second".into(), false, true));
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn admin_quest_group_capability_supplies_distinct_values() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("normal environment");
    env.exec("A_Admin.SetQuestGroupCreatable(80000, true); A_Admin.SetQuestGroupCreatable(80001, false)")
        .expect("host should supply both quest capabilities");
    let values: (bool, bool) = env
        .eval("return C_LFGList.CanCreateQuestGroup(80000), C_LFGList.CanCreateQuestGroup(80001)")
        .expect("quest-group query should read supplied state");
    assert_eq!(values, (true, false));
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn admin_quest_log_special_item_supplies_item_and_absence() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("normal environment");
    env.exec("A_Admin.SetQuestLogSpecialItem(2, 19001); A_Admin.SetQuestLogSpecialItem(3, nil)")
        .expect("host should supply distinct special-item inputs");
    let values: (i64, i64) = env
        .eval("local _, a = GetQuestLogSpecialItemInfo(2); local _, b = GetQuestLogSpecialItemInfo(3); return a or 0, b or 0")
        .expect("legacy query should read supplied items");
    assert_eq!(values, (19001, 0));
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn original_quest_helpers_preserve_tracker_consumer_outputs() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_ObjectiveTracker"],
        &[],
        |env, loaded| {
            assert_provider_order(loaded);
            env.exec(include_str!("quest_helper_load_ownership.lua"))
                .expect("supply distinct quest rows and notify the real tracker");
            wow_ui_sim::startup::process_pending_timers(env);
            wow_ui_sim::startup::fire_one_on_update_tick(env);
            assert_tracker_output(env);
            assert_cache_identity_survives_update(env);
        },
    );
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_provider_order(loaded: &[String]) {
    let tracker = loaded
        .iter()
        .position(|name| name == "Blizzard_ObjectiveTracker")
        .unwrap();
    for provider in ["Blizzard_ObjectAPI", "Blizzard_FrameXMLUtil"] {
        let position = loaded.iter().position(|name| name == provider).unwrap();
        assert!(position < tracker, "{provider} must load before the tracker");
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_cache_identity_survives_update(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec("QuestHelperCachedQuest = QuestCache.objects[990001]; A_Admin.FireEvent('QUEST_LOG_UPDATE')")
        .expect("repeat the original tracker event");
    wow_ui_sim::startup::process_pending_timers(env);
    wow_ui_sim::startup::fire_one_on_update_tick(env);
    let same: bool = env
        .eval("return QuestCache.objects[990001] == QuestHelperCachedQuest")
        .expect("original cache object identity should remain stable");
    assert!(same);
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_tracker_output(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let output: (bool, bool, bool, bool, bool, String, bool, bool, bool, bool, bool, bool, String) = env
        .eval(include_str!("quest_helper_load_ownership_assert.lua"))
        .expect("original tracker must render the supplied quest states");
    assert_eq!(output.0, true, "group finder button must be shown: {output:?}");
    assert_eq!(output.1, true, "quest item button must be shown");
    assert_eq!(output.2, true, "false group capability must hide its button");
    assert_eq!(output.3, true, "missing item must hide its button");
    assert_eq!(output.4, true, "cache entry must retain its quest identity");
    assert!(output.5.contains("Group and item"));
}
