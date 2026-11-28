use crate::lua_api::WowLuaEnv;

fn run_scenario(lua: &str) {
    WowLuaEnv::new().unwrap().exec(lua).unwrap();
}

const PLAYER_FEATURES_CASE: &str = r##"
        assert(C_PlayerInfo.CanPlayerUseAreaLoot() == false)
        assert(C_PlayerInfo.HasAccountInventoryLock() == false)
        assert(C_PlayerInfo.IsAccountBankEnabled() == false)
        assert(C_PlayerInfo.IsCharacterBankEnabled() == false)
        assert(C_PlayerInfo.IsPlayerInTimerunningHeroicWorldTier() == false)
        assert(C_PlayerInfo.IsSelfFoundActive() == false)
        A_Admin.SetPlayerFeatureState('{"areaLootAllowed":true,"accountInventoryLocked":false,"accountBankEnabled":true,"characterBankEnabled":false,"heroicWorldTier":true,"selfFoundActive":false}')
        assert(C_PlayerInfo.CanPlayerUseAreaLoot() == true)
        assert(C_PlayerInfo.HasAccountInventoryLock() == false)
        assert(C_PlayerInfo.IsAccountBankEnabled() == true)
        assert(C_PlayerInfo.IsCharacterBankEnabled() == false)
        assert(C_PlayerInfo.IsPlayerInTimerunningHeroicWorldTier() == true)
        assert(C_PlayerInfo.IsSelfFoundActive() == false)
        A_Admin.SetPlayerFeatureState('{"areaLootAllowed":false,"accountInventoryLocked":true,"accountBankEnabled":false,"characterBankEnabled":true,"heroicWorldTier":false,"selfFoundActive":true}')
        assert(C_PlayerInfo.CanPlayerUseAreaLoot() == false)
        assert(C_PlayerInfo.HasAccountInventoryLock() == true)
        assert(C_PlayerInfo.IsAccountBankEnabled() == false)
        assert(C_PlayerInfo.IsCharacterBankEnabled() == true)
        assert(C_PlayerInfo.IsPlayerInTimerunningHeroicWorldTier() == false)
        assert(C_PlayerInfo.IsSelfFoundActive() == true)
        assert(not pcall(A_Admin.SetPlayerFeatureState, '{"unknownInput":true}'))
        assert(C_PlayerInfo.CanPlayerUseAreaLoot() == false)
        assert(C_PlayerInfo.HasAccountInventoryLock() == true)
        assert(C_PlayerInfo.IsAccountBankEnabled() == false)
        assert(C_PlayerInfo.IsCharacterBankEnabled() == true)
        assert(C_PlayerInfo.IsPlayerInTimerunningHeroicWorldTier() == false)
        assert(C_PlayerInfo.IsSelfFoundActive() == true)
"##;

#[test]
fn player_features_reflects_replaced_observation() {
    run_scenario(PLAYER_FEATURES_CASE);
}

const GROUP_ACTIVITY_CASE: &str = r##"
        assert(C_PartyInfo.CanInvite() == false)
        assert(C_PartyInfo.IsDelveComplete() == false)
        A_Admin.SetGroupActivityState('{"invitationsAllowed":true,"delveComplete":false}')
        assert(C_PartyInfo.CanInvite() == true)
        assert(C_PartyInfo.IsDelveComplete() == false)
        A_Admin.SetGroupActivityState('{"invitationsAllowed":false,"delveComplete":true}')
        assert(C_PartyInfo.CanInvite() == false)
        assert(C_PartyInfo.IsDelveComplete() == true)
        assert(not pcall(A_Admin.SetGroupActivityState, '{"unknownInput":true}'))
        assert(C_PartyInfo.CanInvite() == false)
        assert(C_PartyInfo.IsDelveComplete() == true)
"##;

#[test]
fn group_activity_reflects_replaced_observation() {
    run_scenario(GROUP_ACTIVITY_CASE);
}

const PVP_PROGRESS_CASE: &str = r##"
        assert(C_PvP.HasArenaSkirmishWinToday() == false)
        assert(C_PvP.HasRandomTrainingGroundArenaWinToday() == false)
        assert(C_PvP.HasRandomTrainingGroundBGWinToday() == false)
        assert(C_PvP.IsActiveMatchRegistered() == false)
        assert(C_PvP.IsWarModeFeatureEnabled() == false)
        A_Admin.SetPvpProgressState('{"arenaSkirmishWonToday":true,"trainingArenaWonToday":false,"trainingBgWonToday":true,"activeMatchRegistered":false,"warModeEnabled":true}')
        assert(C_PvP.HasArenaSkirmishWinToday() == true)
        assert(C_PvP.HasRandomTrainingGroundArenaWinToday() == false)
        assert(C_PvP.HasRandomTrainingGroundBGWinToday() == true)
        assert(C_PvP.IsActiveMatchRegistered() == false)
        assert(C_PvP.IsWarModeFeatureEnabled() == true)
        A_Admin.SetPvpProgressState('{"arenaSkirmishWonToday":false,"trainingArenaWonToday":true,"trainingBgWonToday":false,"activeMatchRegistered":true,"warModeEnabled":false}')
        assert(C_PvP.HasArenaSkirmishWinToday() == false)
        assert(C_PvP.HasRandomTrainingGroundArenaWinToday() == true)
        assert(C_PvP.HasRandomTrainingGroundBGWinToday() == false)
        assert(C_PvP.IsActiveMatchRegistered() == true)
        assert(C_PvP.IsWarModeFeatureEnabled() == false)
        assert(not pcall(A_Admin.SetPvpProgressState, '{"unknownInput":true}'))
        assert(C_PvP.HasArenaSkirmishWinToday() == false)
        assert(C_PvP.HasRandomTrainingGroundArenaWinToday() == true)
        assert(C_PvP.HasRandomTrainingGroundBGWinToday() == false)
        assert(C_PvP.IsActiveMatchRegistered() == true)
        assert(C_PvP.IsWarModeFeatureEnabled() == false)
"##;

#[test]
fn pvp_progress_reflects_replaced_observation() {
    run_scenario(PVP_PROGRESS_CASE);
}

const WEEKLY_REWARD_STATUS_CASE: &str = r##"
        assert(C_WeeklyRewards.HasGeneratedRewards() == false)
        assert(C_WeeklyRewards.IsWeeklyChestRetired() == false)
        A_Admin.SetWeeklyRewardStatus('{"rewardsGenerated":true,"weeklyChestRetired":false}')
        assert(C_WeeklyRewards.HasGeneratedRewards() == true)
        assert(C_WeeklyRewards.IsWeeklyChestRetired() == false)
        A_Admin.SetWeeklyRewardStatus('{"rewardsGenerated":false,"weeklyChestRetired":true}')
        assert(C_WeeklyRewards.HasGeneratedRewards() == false)
        assert(C_WeeklyRewards.IsWeeklyChestRetired() == true)
        assert(not pcall(A_Admin.SetWeeklyRewardStatus, '{"unknownInput":true}'))
        assert(C_WeeklyRewards.HasGeneratedRewards() == false)
        assert(C_WeeklyRewards.IsWeeklyChestRetired() == true)
"##;

#[test]
fn weekly_reward_status_reflects_replaced_observation() {
    run_scenario(WEEKLY_REWARD_STATUS_CASE);
}

const CONTENT_AVAILABILITY_CASE: &str = r##"
        assert(C_ClubFinder.ShouldShowClubFinder() == false)
        assert(C_CovenantSanctumUI.CanAccessReservoir() == false)
        assert(C_CovenantSanctumUI.GetSanctumType() == nil)
        assert(C_InvasionInfo.AreInvasionsAvailable() == false)
        assert(C_ImmersiveInteraction.HasImmersiveInteraction() == false)
        assert(C_RaidLocks.IsRaidLockExtendFeatureEnabled() == false)
        assert(#C_CampaignInfo.GetAvailableCampaigns() == 0)
        assert(C_CraftingOrders.GetDefaultOrdersSkillLine() == nil)
        assert(#C_DelvesUI.GetDelvesAffixSpellsForSeason() == 0)
        A_Admin.SetWorldContentAvailability('{"clubFinderShown":true,"reservoirAccessible":false,"sanctumType":3,"invasionsInPlayerArea":false,"immersiveInteractionActive":true,"raidLockExtensionEnabled":false,"availableCampaignIds":[838,1296],"defaultOrderSkillLineId":8,"delveSeasonAffixSpellIds":[840,1298]}')
        assert(C_ClubFinder.ShouldShowClubFinder() == true)
        assert(C_CovenantSanctumUI.CanAccessReservoir() == false)
        assert(C_CovenantSanctumUI.GetSanctumType() == 3)
        assert(C_InvasionInfo.AreInvasionsAvailable() == false)
        assert(C_ImmersiveInteraction.HasImmersiveInteraction() == true)
        assert(C_RaidLocks.IsRaidLockExtendFeatureEnabled() == false)
        assert(#C_CampaignInfo.GetAvailableCampaigns() == 2 and C_CampaignInfo.GetAvailableCampaigns()[1] == 838 and C_CampaignInfo.GetAvailableCampaigns()[2] == 1296)
        assert(C_CraftingOrders.GetDefaultOrdersSkillLine() == 8)
        assert(#C_DelvesUI.GetDelvesAffixSpellsForSeason() == 2 and C_DelvesUI.GetDelvesAffixSpellsForSeason()[1] == 840 and C_DelvesUI.GetDelvesAffixSpellsForSeason()[2] == 1298)
        A_Admin.SetWorldContentAvailability('{"clubFinderShown":false,"reservoirAccessible":true,"sanctumType":null,"invasionsInPlayerArea":true,"immersiveInteractionActive":false,"raidLockExtensionEnabled":true,"availableCampaignIds":[1297],"defaultOrderSkillLineId":null,"delveSeasonAffixSpellIds":[1299]}')
        assert(C_ClubFinder.ShouldShowClubFinder() == false)
        assert(C_CovenantSanctumUI.CanAccessReservoir() == true)
        assert(C_CovenantSanctumUI.GetSanctumType() == nil)
        assert(C_InvasionInfo.AreInvasionsAvailable() == true)
        assert(C_ImmersiveInteraction.HasImmersiveInteraction() == false)
        assert(C_RaidLocks.IsRaidLockExtendFeatureEnabled() == true)
        assert(#C_CampaignInfo.GetAvailableCampaigns() == 1 and C_CampaignInfo.GetAvailableCampaigns()[1] == 1297)
        assert(C_CraftingOrders.GetDefaultOrdersSkillLine() == nil)
        assert(#C_DelvesUI.GetDelvesAffixSpellsForSeason() == 1 and C_DelvesUI.GetDelvesAffixSpellsForSeason()[1] == 1299)
        assert(not pcall(A_Admin.SetWorldContentAvailability, '{"unknownInput":true}'))
        assert(C_ClubFinder.ShouldShowClubFinder() == false)
        assert(C_CovenantSanctumUI.CanAccessReservoir() == true)
        assert(C_CovenantSanctumUI.GetSanctumType() == nil)
        assert(C_InvasionInfo.AreInvasionsAvailable() == true)
        assert(C_ImmersiveInteraction.HasImmersiveInteraction() == false)
        assert(C_RaidLocks.IsRaidLockExtendFeatureEnabled() == true)
        assert(#C_CampaignInfo.GetAvailableCampaigns() == 1 and C_CampaignInfo.GetAvailableCampaigns()[1] == 1297)
        assert(C_CraftingOrders.GetDefaultOrdersSkillLine() == nil)
        assert(#C_DelvesUI.GetDelvesAffixSpellsForSeason() == 1 and C_DelvesUI.GetDelvesAffixSpellsForSeason()[1] == 1299)
"##;

#[test]
fn content_availability_reflects_replaced_observation() {
    run_scenario(CONTENT_AVAILABILITY_CASE);
}

const MINIMAP_PRESENTATION_CASE: &str = r##"
        assert(C_Minimap.IsRotateMinimapIgnored() == false)
        assert(C_Minimap.IsTrackingBattlePets() == false)
        assert(C_Minimap.GetNumQuestPOIWorldEffects() == 0)
        A_Admin.SetMinimapPresentationState('{"rotationIgnored":true,"trackingBattlePets":false,"questPoiWorldEffectCount":9}')
        assert(C_Minimap.IsRotateMinimapIgnored() == true)
        assert(C_Minimap.IsTrackingBattlePets() == false)
        assert(C_Minimap.GetNumQuestPOIWorldEffects() == 9)
        A_Admin.SetMinimapPresentationState('{"rotationIgnored":false,"trackingBattlePets":true,"questPoiWorldEffectCount":31}')
        assert(C_Minimap.IsRotateMinimapIgnored() == false)
        assert(C_Minimap.IsTrackingBattlePets() == true)
        assert(C_Minimap.GetNumQuestPOIWorldEffects() == 31)
        assert(not pcall(A_Admin.SetMinimapPresentationState, '{"unknownInput":true}'))
        assert(C_Minimap.IsRotateMinimapIgnored() == false)
        assert(C_Minimap.IsTrackingBattlePets() == true)
        assert(C_Minimap.GetNumQuestPOIWorldEffects() == 31)
"##;

#[test]
fn minimap_presentation_reflects_replaced_observation() {
    run_scenario(MINIMAP_PRESENTATION_CASE);
}

const HOUSING_OBSERVATION_CASE: &str = r##"
        assert(C_HousingBasicMode.IsHouseExteriorHovered() == false)
        assert(C_HousingBasicMode.IsHoveringDecor() == false)
        assert(C_HousingCleanupMode.IsHoveringDecor() == false)
        assert(C_HousingCustomizeMode.IsHoveringDecor() == false)
        assert(C_HousingExpertMode.IsHouseExteriorHovered() == false)
        assert(C_HousingExpertMode.IsHoveringDecor() == false)
        assert(select("#", C_HousingLayout.GetSelectedRoom()) == 0)
        assert(C_HousingLayout.GetSelectedStairwellRoomCount() == 0)
        assert(C_HousingCustomizeMode.GetNumDyesToSpendOnSelectedDecor() == 0)
        assert(C_HousingNeighborhood.IsPlayerInOtherPlayersPlot() == false)
        A_Admin.SetHousingEditorObservation('{"basicExteriorHovered":true,"basicHoveredDecorGuid":null,"cleanupHoveredDecorGuid":"HousingDecor-0-0-0-0-000000000067","customizeHoveredDecorGuid":null,"expertExteriorHovered":true,"expertHoveredDecorGuid":null,"selectedRoomGuid":"Housing-0-0-0-0-000000004001","selectedStairwellRoomCount":14,"previewDyeItemCount":15,"inOtherPlayersPlot":true}')
        assert(C_HousingBasicMode.IsHouseExteriorHovered() == true)
        assert(C_HousingBasicMode.IsHoveringDecor() == false)
        assert(C_HousingCleanupMode.IsHoveringDecor() == true)
        assert(C_HousingCustomizeMode.IsHoveringDecor() == false)
        assert(C_HousingExpertMode.IsHouseExteriorHovered() == true)
        assert(C_HousingExpertMode.IsHoveringDecor() == false)
        assert(C_HousingLayout.GetSelectedRoom() == "Housing-0-0-0-0-000000004001")
        assert(C_HousingLayout.GetSelectedStairwellRoomCount() == 14)
        assert(C_HousingCustomizeMode.GetNumDyesToSpendOnSelectedDecor() == 15)
        assert(C_HousingNeighborhood.IsPlayerInOtherPlayersPlot() == true)
        A_Admin.SetHousingEditorObservation('{"basicExteriorHovered":false,"basicHoveredDecorGuid":"HousingDecor-0-0-0-0-0000000000CA","cleanupHoveredDecorGuid":null,"customizeHoveredDecorGuid":"HousingDecor-0-0-0-0-0000000000CC","expertExteriorHovered":false,"expertHoveredDecorGuid":"HousingDecor-0-0-0-0-0000000000CE","selectedRoomGuid":null,"selectedStairwellRoomCount":36,"previewDyeItemCount":37,"inOtherPlayersPlot":false}')
        assert(C_HousingBasicMode.IsHouseExteriorHovered() == false)
        assert(C_HousingBasicMode.IsHoveringDecor() == true)
        assert(C_HousingCleanupMode.IsHoveringDecor() == false)
        assert(C_HousingCustomizeMode.IsHoveringDecor() == true)
        assert(C_HousingExpertMode.IsHouseExteriorHovered() == false)
        assert(C_HousingExpertMode.IsHoveringDecor() == true)
        assert(select("#", C_HousingLayout.GetSelectedRoom()) == 0)
        assert(C_HousingLayout.GetSelectedStairwellRoomCount() == 36)
        assert(C_HousingCustomizeMode.GetNumDyesToSpendOnSelectedDecor() == 37)
        assert(C_HousingNeighborhood.IsPlayerInOtherPlayersPlot() == false)
        assert(not pcall(A_Admin.SetHousingEditorObservation, '{"unknownInput":true}'))
        assert(C_HousingBasicMode.IsHouseExteriorHovered() == false)
        assert(C_HousingBasicMode.IsHoveringDecor() == true)
        assert(C_HousingCleanupMode.IsHoveringDecor() == false)
        assert(C_HousingCustomizeMode.IsHoveringDecor() == true)
        assert(C_HousingExpertMode.IsHouseExteriorHovered() == false)
        assert(C_HousingExpertMode.IsHoveringDecor() == true)
        assert(select("#", C_HousingLayout.GetSelectedRoom()) == 0)
        assert(C_HousingLayout.GetSelectedStairwellRoomCount() == 36)
        assert(C_HousingCustomizeMode.GetNumDyesToSpendOnSelectedDecor() == 37)
        assert(C_HousingNeighborhood.IsPlayerInOtherPlayersPlot() == false)
"##;

#[test]
fn housing_observation_reflects_replaced_observation() {
    run_scenario(HOUSING_OBSERVATION_CASE);
}
