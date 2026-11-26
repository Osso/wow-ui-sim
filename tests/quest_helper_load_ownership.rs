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
