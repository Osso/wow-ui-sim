//! Original WorldMap world-quest reward classification over item rewards.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::QuestRewardItem;

fn reward(item_id: u32, name: &str) -> QuestRewardItem {
    QuestRewardItem {
        name: name.into(),
        texture: "Interface\\Icons\\INV_Misc_QuestionMark".into(),
        count: 1,
        quality: 1,
        is_usable: true,
        item_id: Some(item_id),
    }
}

fn supply_rewards(env: &WowLuaEnv, rewards: Vec<QuestRewardItem>) -> i32 {
    let state = env.state();
    let mut sim = state.borrow_mut();
    let entry = &mut sim.quest_log_entries.entries[0];
    entry.reward_items = rewards;
    entry.currency_rewards.clear();
    entry.quest_id
}

prefork_full_ui_case! {
fn world_quest_reward_type_flags_anima_items(env: &WowLuaEnv) {
    let quest = supply_rewards(env, vec![
        reward(181544, "Confessions of Misdeed"),
        reward(6948, "Hearthstone"),
    ]);
    env.exec(&format!(r#"
        local source = debug.getinfo(WorldMap_GetWorldQuestRewardType, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_UIPanels_Game/Mainline/WorldMapFrame.lua', 1, true), source)
        assert(HaveQuestRewardData({quest}), 'fixture quest exposes reward data')
        local ok, flags = WorldMap_GetWorldQuestRewardType({quest})
        assert(ok and flags == WORLD_QUEST_REWARD_TYPE_FLAG_ANIMA, tostring(flags))
        assert(GetNumQuestLogRewards({quest}) == 2)
        assert(select('#', GetQuestLogRewardInfo(1, {quest})) == 6)
        assert(select(6, GetQuestLogRewardInfo(1, {quest})) == 181544)
        assert(C_Item.IsAnimaItemByID('item:181544') and not C_Item.IsAnimaItemByID(6948))
        assert(not C_Item.IsArtifactPowerItem(181544) and not C_Item.IsArtifactPowerItem(0))
        assert(not pcall(C_Item.IsAnimaItemByID) and not pcall(C_Item.IsArtifactPowerItem))
    "#)).unwrap();
    supply_rewards(env, vec![reward(6948, "Hearthstone")]);
    env.exec(&format!(r#"
        local ok, flags = WorldMap_GetWorldQuestRewardType({quest})
        assert(ok and flags == 0, tostring(flags))
    "#)).unwrap();
}
}
