//! B(a/b) normal-environment preservation, not a claimed production RED.
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
