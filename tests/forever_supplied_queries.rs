//! API tests for documented Forever queries without a pinned Lua consumer.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn chat_messaging_lockdown_reads_supplied_boolean() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_ChatInfo.InChatMessagingLockdown() == false)
        A_Admin.SetChatMessagingLockdown(true)
        assert(C_ChatInfo.InChatMessagingLockdown() == true)
        A_Admin.SetChatMessagingLockdown(false)
        assert(C_ChatInfo.InChatMessagingLockdown() == false)
    "#).unwrap();
}

#[test]
fn active_timerunning_season_is_separate_from_player_season() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_TimerunningUI.GetActiveTimerunningSeasonID() == 0)
        A_Admin.SetActiveTimerunningSeasonID(3)
        A_Admin.SetTimerunningSeasonID(4)
        assert(C_TimerunningUI.GetActiveTimerunningSeasonID() == 3)
        A_Admin.SetActiveTimerunningSeasonID(8)
        assert(C_TimerunningUI.GetActiveTimerunningSeasonID() == 8)
    "#).unwrap();
}

#[test]
fn item_stats_and_deltas_project_distinct_supplied_links() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_Item.GetItemStats('item:25') == nil)
        A_Admin.SetItemStats('item:25', {ITEM_MOD_STAMINA_SHORT = 7, ITEM_MOD_STRENGTH_SHORT = 13})
        A_Admin.SetItemStats('item:36', {ITEM_MOD_STAMINA_SHORT = 21, ITEM_MOD_AGILITY_SHORT = 6})
        local a, b = C_Item.GetItemStats('item:25'), C_Item.GetItemStats('item:36')
        assert(a.ITEM_MOD_STAMINA_SHORT == 7 and a.ITEM_MOD_STRENGTH_SHORT == 13)
        assert(b.ITEM_MOD_STAMINA_SHORT == 21 and b.ITEM_MOD_AGILITY_SHORT == 6)
        local delta = C_Item.GetItemStatDelta('item:25', 'item:36')
        assert(delta.ITEM_MOD_STAMINA_SHORT == -14 and delta.ITEM_MOD_STRENGTH_SHORT == 13)
        assert(delta.ITEM_MOD_AGILITY_SHORT == -6)
        local inverse = C_Item.GetItemStatDelta('item:36', 'item:25')
        assert(inverse.ITEM_MOD_STAMINA_SHORT == 14 and inverse.ITEM_MOD_AGILITY_SHORT == 6)
        assert(inverse.ITEM_MOD_STRENGTH_SHORT == -13)
        assert(C_Item.GetItemStatDelta('item:25', 'item:404') == nil)
    "#).unwrap();
}
