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

#[cfg(feature = "client-retail")]
#[test]
fn original_quest_helpers_own_normal_tracker_dependency_closure() {
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &cache,
        &["Blizzard_ObjectiveTracker"],
        &[],
    );
    for owner in [
        "Blizzard_ObjectAPI",
        "Blizzard_FrameXMLUtil",
        "Blizzard_ObjectiveTracker",
    ] {
        assert!(
            loaded.iter().any(|name| name == owner),
            "{owner}: {loaded:?}"
        );
    }
    println!("B(a) normal discovered closure: {loaded:?}");
    env.exec(
        r#"
        assert(Enum.QuestClassification.Campaign == 2)
        assert(Enum.QuestClassification.Calling == 3)
        local function requireSource(fn, suffix)
            local info = debug.getinfo(fn, 'S')
            local source = info.source:gsub('\\', '/')
            assert(source:find(suffix, 1, true), source)
            print('B(a) original source: ' .. source .. ':' .. info.linedefined)
        end
        requireSource(QuestCache.Get, 'Blizzard_ObjectAPI/Mainline/ObjectCache.lua')
        requireSource(QuestUtil.CanCreateQuestGroup, 'Blizzard_FrameXMLUtil/Mainline/QuestUtils.lua')
        requireSource(QuestUtil.QuestShowsItemByIndex, 'Blizzard_FrameXMLUtil/Mainline/QuestUtils.lua')
        requireSource(QuestObjectiveTracker.BuildQuestWatchInfos, 'Blizzard_ObjectiveTracker/Blizzard_QuestObjectiveTracker.lua')

        -- Use existing watched-quest inputs and the actual consumer, not fake
        -- quest flags, overridden APIs, missing dependencies or extracted Lua.
        assert(C_QuestLog.GetNumQuestWatches() > 0, 'fixture must contain real watched inputs')
        local infos = QuestObjectiveTracker:BuildQuestWatchInfos()
        assert(#infos > 0, 'original consumer must enumerate more than an empty domain')
        for _, entry in ipairs(infos) do
            local id = entry.quest:GetID()
            assert(QuestCache:Get(id) == entry.quest, 'original ObjectCache retains identity')
            assert(QuestObjectiveTracker:ShouldDisplayQuest(entry.quest))
        end
    "#,
    )
    .expect("normal closure must reach original quest helpers and tracker consumer");
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
