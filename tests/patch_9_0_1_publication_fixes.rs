//! Unused historical members stay absent under repeated namespace lookup.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
assert(rawget(C_CampaignInfo, "GetCurrentCampaignChapterID") == nil, "C_CampaignInfo.GetCurrentCampaignChapterID raw")
assert(C_CampaignInfo.GetCurrentCampaignChapterID == nil, "C_CampaignInfo.GetCurrentCampaignChapterID lookup")
assert(C_CampaignInfo.GetCurrentCampaignChapterID == nil, "C_CampaignInfo.GetCurrentCampaignChapterID repeat")
assert(rawget(C_CampaignInfo, "GetCurrentCampaignID") == nil, "C_CampaignInfo.GetCurrentCampaignID raw")
assert(C_CampaignInfo.GetCurrentCampaignID == nil, "C_CampaignInfo.GetCurrentCampaignID lookup")
assert(C_CampaignInfo.GetCurrentCampaignID == nil, "C_CampaignInfo.GetCurrentCampaignID repeat")
assert(rawget(C_Commentator, "GetTeamHighlightColor") == nil, "C_Commentator.GetTeamHighlightColor raw")
assert(C_Commentator.GetTeamHighlightColor == nil, "C_Commentator.GetTeamHighlightColor lookup")
assert(C_Commentator.GetTeamHighlightColor == nil, "C_Commentator.GetTeamHighlightColor repeat")
assert(rawget(C_Garrison, "GetMissionInfo") == nil, "C_Garrison.GetMissionInfo raw")
assert(C_Garrison.GetMissionInfo == nil, "C_Garrison.GetMissionInfo lookup")
assert(C_Garrison.GetMissionInfo == nil, "C_Garrison.GetMissionInfo repeat")
assert(rawget(C_Garrison, "GetTalentTreeInfoForID") == nil, "C_Garrison.GetTalentTreeInfoForID raw")
assert(C_Garrison.GetTalentTreeInfoForID == nil, "C_Garrison.GetTalentTreeInfoForID lookup")
assert(C_Garrison.GetTalentTreeInfoForID == nil, "C_Garrison.GetTalentTreeInfoForID repeat")
assert(rawget(C_GossipInfo, "GetGossipPoiForUiMapID") == nil, "C_GossipInfo.GetGossipPoiForUiMapID raw")
assert(C_GossipInfo.GetGossipPoiForUiMapID == nil, "C_GossipInfo.GetGossipPoiForUiMapID lookup")
assert(C_GossipInfo.GetGossipPoiForUiMapID == nil, "C_GossipInfo.GetGossipPoiForUiMapID repeat")
assert(rawget(C_GossipInfo, "GetGossipPoiInfo") == nil, "C_GossipInfo.GetGossipPoiInfo raw")
assert(C_GossipInfo.GetGossipPoiInfo == nil, "C_GossipInfo.GetGossipPoiInfo lookup")
assert(C_GossipInfo.GetGossipPoiInfo == nil, "C_GossipInfo.GetGossipPoiInfo repeat")
assert(rawget(C_Item, "IsItemCorruptable") == nil, "C_Item.IsItemCorruptable raw")
assert(C_Item.IsItemCorruptable == nil, "C_Item.IsItemCorruptable lookup")
assert(C_Item.IsItemCorruptable == nil, "C_Item.IsItemCorruptable repeat")
assert(rawget(C_LootJournal, "GetFilteredItemSets") == nil, "C_LootJournal.GetFilteredItemSets raw")
assert(C_LootJournal.GetFilteredItemSets == nil, "C_LootJournal.GetFilteredItemSets lookup")
assert(C_LootJournal.GetFilteredItemSets == nil, "C_LootJournal.GetFilteredItemSets repeat")
assert(rawget(C_MountJournal, "IsMountEquipmentUnlocked") == nil, "C_MountJournal.IsMountEquipmentUnlocked raw")
assert(C_MountJournal.IsMountEquipmentUnlocked == nil, "C_MountJournal.IsMountEquipmentUnlocked lookup")
assert(C_MountJournal.IsMountEquipmentUnlocked == nil, "C_MountJournal.IsMountEquipmentUnlocked repeat")
assert(rawget(C_TransmogCollection, "GetAppearanceSourceInfoForTransmog") == nil, "C_TransmogCollection.GetAppearanceSourceInfoForTransmog raw")
assert(C_TransmogCollection.GetAppearanceSourceInfoForTransmog == nil, "C_TransmogCollection.GetAppearanceSourceInfoForTransmog lookup")
assert(C_TransmogCollection.GetAppearanceSourceInfoForTransmog == nil, "C_TransmogCollection.GetAppearanceSourceInfoForTransmog repeat")
"#;

#[test]
fn patch_9_0_1_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
