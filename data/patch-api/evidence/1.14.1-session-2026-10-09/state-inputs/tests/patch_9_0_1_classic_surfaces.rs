//! Classic legacy lookup remains unaffected by retail retirement.
#![cfg(feature = "client-mists")]

#[test]
fn patch_9_0_1_mists_preserves_legacy_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"
assert(type(C_CampaignInfo.GetCurrentCampaignChapterID) == "function", "C_CampaignInfo.GetCurrentCampaignChapterID")
assert(type(C_CampaignInfo.GetCurrentCampaignID) == "function", "C_CampaignInfo.GetCurrentCampaignID")
assert(type(C_Commentator.GetTeamHighlightColor) == "function", "C_Commentator.GetTeamHighlightColor")
assert(type(C_Garrison.GetMissionInfo) == "function", "C_Garrison.GetMissionInfo")
assert(type(C_Garrison.GetTalentTreeInfoForID) == "function", "C_Garrison.GetTalentTreeInfoForID")
assert(type(C_GossipInfo.GetGossipPoiForUiMapID) == "function", "C_GossipInfo.GetGossipPoiForUiMapID")
assert(type(C_GossipInfo.GetGossipPoiInfo) == "function", "C_GossipInfo.GetGossipPoiInfo")
assert(type(C_Item.IsItemCorruptable) == "function", "C_Item.IsItemCorruptable")
assert(type(C_LootJournal.GetFilteredItemSets) == "function", "C_LootJournal.GetFilteredItemSets")
assert(type(C_MountJournal.IsMountEquipmentUnlocked) == "function", "C_MountJournal.IsMountEquipmentUnlocked")
assert(type(C_TransmogCollection.GetAppearanceSourceInfoForTransmog) == "function", "C_TransmogCollection.GetAppearanceSourceInfoForTransmog")
"#).unwrap();
}
