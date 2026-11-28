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

#[test]
fn bank_and_mentor_observation_replacement() {
    run_scenario(
        r#"
        assert(#C_Bank.FetchViewableBankTypes() == 0 and C_ChatInfo.GetMentorChannelID() == 0)
        A_Admin.SetPlayerFeatureState('{"viewableBankTypes":[0,2],"mentorChannelId":19}')
        local banks = C_Bank.FetchViewableBankTypes()
        assert(#banks == 2 and banks[1] == 0 and banks[2] == 2 and C_ChatInfo.GetMentorChannelID() == 19)
        A_Admin.SetPlayerFeatureState('{"viewableBankTypes":[1],"mentorChannelId":42}')
        banks = C_Bank.FetchViewableBankTypes()
        assert(#banks == 1 and banks[1] == 1 and C_ChatInfo.GetMentorChannelID() == 42)
        assert(not pcall(A_Admin.SetPlayerFeatureState,'{"viewableBankTypes":[9]}'))
        assert(C_Bank.FetchViewableBankTypes()[1] == 1)
    "#,
    );
}

#[test]
fn availability_catalog_and_pvp_policy_replacement() {
    run_scenario(
        r#"
        A_Admin.SetWorldContentAvailability('{"clubFinderFocusFlags":[1,2,2,4],"delveSeasonFactionId":2712,"pvpAllowed":false,"pvpFailureReason":"Scenario restriction"}')
        assert(C_ClubFinder.GetPlayerSettingsFocusFlagsSelectedCount() == 3)
        assert(C_DelvesUI.GetDelvesFactionForSeason() == 2712)
        local allowed, reason = C_LFGInfo.CanPlayerUsePVP()
        assert(allowed == false and reason == "Scenario restriction")
        A_Admin.SetWorldContentAvailability('{"clubFinderFocusFlags":[2],"delveSeasonFactionId":2562,"pvpAllowed":true,"pvpFailureReason":""}')
        assert(C_ClubFinder.GetPlayerSettingsFocusFlagsSelectedCount() == 1)
        assert(C_DelvesUI.GetDelvesFactionForSeason() == 2562)
        allowed, reason = C_LFGInfo.CanPlayerUsePVP()
        assert(allowed == true and reason == "")
        A_Admin.SetWorldContentAvailability('{}')
        assert(C_ClubFinder.GetPlayerSettingsFocusFlagsSelectedCount() == 0 and C_DelvesUI.GetDelvesFactionForSeason() == 0)
    "#,
    );
}

#[test]
fn pvp_reward_observation_replacement() {
    run_scenario(
        r#"
        A_Admin.SetPvpProgressState('{"weeklyRewardAchieved":true,"lastWeekRewardAchieved":false,"lastWeekRewardClaimed":true,"tierMaxFromWins":3,"soloRbgBrawlMinItemLevel":660}')
        local earned, last, claimed, tier = C_PvP.GetWeeklyChestInfo()
        assert(earned == true and last == false and claimed == true and tier == 3)
        assert(C_PvP.GetBrawlSoloRBGMinItemLevel() == 660)
        A_Admin.SetPvpProgressState('{"weeklyRewardAchieved":false,"lastWeekRewardAchieved":true,"lastWeekRewardClaimed":false,"tierMaxFromWins":7,"soloRbgBrawlMinItemLevel":710}')
        earned, last, claimed, tier = C_PvP.GetWeeklyChestInfo()
        assert(earned == false and last == true and claimed == false and tier == 7)
        assert(C_PvP.GetBrawlSoloRBGMinItemLevel() == 710)
    "#,
    );
}

#[test]
fn collection_observation_is_independent_for_relics_and_favorites() {
    run_scenario(
        r#"
        assert(not C_ArtifactUI.DoesEquippedArtifactHaveAnyRelicsSlotted() and not C_TransmogCollection.HasFavorites())
        A_Admin.SetCollectionObservation('{"equippedArtifactRelicItemIds":[12345],"favoriteAppearanceIds":[]}')
        assert(C_ArtifactUI.DoesEquippedArtifactHaveAnyRelicsSlotted() and not C_TransmogCollection.HasFavorites())
        A_Admin.SetCollectionObservation('{"equippedArtifactRelicItemIds":[],"favoriteAppearanceIds":[321,654]}')
        assert(not C_ArtifactUI.DoesEquippedArtifactHaveAnyRelicsSlotted() and C_TransmogCollection.HasFavorites())
    "#,
    );
}

#[test]
fn gamepad_config_ids_preserve_optional_components_and_order() {
    run_scenario(
        r#"
        assert(#C_GamePad.GetAllConfigIDs() == 0)
        A_Admin.SetGamePadConfigurationIDs('[{"vendorID":1356,"productID":1476},{"vendorID":1118}]')
        local ids = C_GamePad.GetAllConfigIDs()
        assert(#ids == 2 and ids[1].vendorID == 1356 and ids[1].productID == 1476)
        assert(ids[2].vendorID == 1118 and ids[2].productID == nil)
        ids[1].vendorID = 0
        assert(C_GamePad.GetAllConfigIDs()[1].vendorID == 1356)
        A_Admin.SetGamePadConfigurationIDs('[{"productID":654}]')
        ids = C_GamePad.GetAllConfigIDs()
        assert(#ids == 1 and ids[1].vendorID == nil and ids[1].productID == 654)
    "#,
    );
}
