# Supplied client service and editor observations

**Profiles:** Retail and WoW Forever. The server or scene producer is not modeled for these read-only queries. Each `A_Admin` setter replaces one typed observation record from a JSON object. Omitted fields become false, zero for counts or unsupplied numeric IDs, an empty string or list, or an absent optional value. These defaults describe an empty simulated scenario, not native login state. Invalid JSON, unknown fields, and incorrect field types raise an error without changing the previous record. Setters do not fire gameplay events or grant permission to related mutations.

Retirement: replace each observation with the corresponding service or scene producer when that producer is modeled; do not maintain duplicate state.

All queries below return the named field unless noted. Both profile declarations are cited alongside the registration.

## `A_Admin.SetPlayerFeatureState(json)`

| JSON field | Type | Query |
|---|---|---|
| `areaLootAllowed` | boolean | `C_PlayerInfo.CanPlayerUseAreaLoot` |
| `accountInventoryLocked` | boolean | `C_PlayerInfo.HasAccountInventoryLock` |
| `accountBankEnabled` | boolean | `C_PlayerInfo.IsAccountBankEnabled` |
| `characterBankEnabled` | boolean | `C_PlayerInfo.IsCharacterBankEnabled` |
| `heroicWorldTier` | boolean | `C_PlayerInfo.IsPlayerInTimerunningHeroicWorldTier` |
| `selfFoundActive` | boolean | `C_PlayerInfo.IsSelfFoundActive` |
| `viewableBankTypes` | ordered list of `BankType` (Character 0, Guild 1, Account 2) | `C_Bank.FetchViewableBankTypes` |
| `mentorChannelId` | channel ID (zero when no channel supplied) | `C_ChatInfo.GetMentorChannelID` |

Example: `A_Admin.SetPlayerFeatureState('{"areaLootAllowed":true,"accountInventoryLocked":false,"accountBankEnabled":true,"characterBankEnabled":false,"heroicWorldTier":true,"selfFoundActive":false}')`.

## `A_Admin.SetGroupActivityState(json)`

| JSON field | Type | Query |
|---|---|---|
| `invitationsAllowed` | boolean | `C_PartyInfo.CanInvite` |
| `delveComplete` | boolean | `C_PartyInfo.IsDelveComplete` |

Example: `A_Admin.SetGroupActivityState('{"invitationsAllowed":true,"delveComplete":false}')`.

## `A_Admin.SetPvpProgressState(json)`

| JSON field | Type | Query |
|---|---|---|
| `arenaSkirmishWonToday` | boolean | `C_PvP.HasArenaSkirmishWinToday` |
| `trainingArenaWonToday` | boolean | `C_PvP.HasRandomTrainingGroundArenaWinToday` |
| `trainingBgWonToday` | boolean | `C_PvP.HasRandomTrainingGroundBGWinToday` |
| `activeMatchRegistered` | boolean | `C_PvP.IsActiveMatchRegistered` |
| `warModeEnabled` | boolean | `C_PvP.IsWarModeFeatureEnabled` |
| `weeklyRewardAchieved / lastWeekRewardAchieved / lastWeekRewardClaimed / tierMaxFromWins` | three booleans and unsigned tier number, in order | `C_PvP.GetWeeklyChestInfo` |
| `soloRbgBrawlMinItemLevel` | minimum item level (zero when not supplied) | `C_PvP.GetBrawlSoloRBGMinItemLevel` |

Example: `A_Admin.SetPvpProgressState('{"arenaSkirmishWonToday":true,"trainingArenaWonToday":false,"trainingBgWonToday":true,"activeMatchRegistered":false,"warModeEnabled":true}')`.

## `A_Admin.SetWeeklyRewardStatus(json)`

| JSON field | Type | Query |
|---|---|---|
| `rewardsGenerated` | boolean | `C_WeeklyRewards.HasGeneratedRewards` |
| `weeklyChestRetired` | boolean | `C_WeeklyRewards.IsWeeklyChestRetired` |

Example: `A_Admin.SetWeeklyRewardStatus('{"rewardsGenerated":true,"weeklyChestRetired":false}')`.

## `A_Admin.SetWorldContentAvailability(json)`

| JSON field | Type | Query |
|---|---|---|
| `clubFinderShown` | boolean | `C_ClubFinder.ShouldShowClubFinder` |
| `reservoirAccessible` | boolean | `C_CovenantSanctumUI.CanAccessReservoir` |
| `sanctumType` | optional numeric ID; returns nil when absent | `C_CovenantSanctumUI.GetSanctumType` |
| `invasionsInPlayerArea` | boolean | `C_InvasionInfo.AreInvasionsAvailable` |
| `immersiveInteractionActive` | boolean | `C_ImmersiveInteraction.HasImmersiveInteraction` |
| `raidLockExtensionEnabled` | boolean | `C_RaidLocks.IsRaidLockExtendFeatureEnabled` |
| `availableCampaignIds` | ordered array of numeric IDs | `C_CampaignInfo.GetAvailableCampaigns` |
| `defaultOrderSkillLineId` | optional numeric ID; returns nil when absent | `C_CraftingOrders.GetDefaultOrdersSkillLine` |
| `delveSeasonAffixSpellIds` | ordered array of numeric IDs | `C_DelvesUI.GetDelvesAffixSpellsForSeason` |
| `clubFinderFocusFlags` | list of distinct focus flag IDs; returns count | `C_ClubFinder.GetPlayerSettingsFocusFlagsSelectedCount` |
| `delveSeasonFactionId` | faction ID (zero when no season supplied) | `C_DelvesUI.GetDelvesFactionForSeason` |
| `pvpAllowed / pvpFailureReason` | boolean and string (default empty reason); returned in that order | `C_LFGInfo.CanPlayerUsePVP` |

Example: `A_Admin.SetWorldContentAvailability('{"clubFinderShown":true,"reservoirAccessible":false,"sanctumType":3,"invasionsInPlayerArea":false,"immersiveInteractionActive":true,"raidLockExtensionEnabled":false,"availableCampaignIds":[838,1296],"defaultOrderSkillLineId":8,"delveSeasonAffixSpellIds":[840,1298]}')`.

## `A_Admin.SetMinimapPresentationState(json)`

| JSON field | Type | Query |
|---|---|---|
| `rotationIgnored` | boolean | `C_Minimap.IsRotateMinimapIgnored` |
| `trackingBattlePets` | boolean | `C_Minimap.IsTrackingBattlePets` |
| `questPoiWorldEffectCount` | nonnegative integer count | `C_Minimap.GetNumQuestPOIWorldEffects` |

Example: `A_Admin.SetMinimapPresentationState('{"rotationIgnored":true,"trackingBattlePets":false,"questPoiWorldEffectCount":9}')`.

## `A_Admin.SetHousingEditorObservation(json)`

| JSON field | Type | Query |
|---|---|---|
| `basicExteriorHovered` | boolean | `C_HousingBasicMode.IsHouseExteriorHovered` |
| `basicHoveredDecorGuid` | optional nonempty GUID; query is whether supplied | `C_HousingBasicMode.IsHoveringDecor` |
| `cleanupHoveredDecorGuid` | optional nonempty GUID; query is whether supplied | `C_HousingCleanupMode.IsHoveringDecor` |
| `customizeHoveredDecorGuid` | optional nonempty GUID; query is whether supplied | `C_HousingCustomizeMode.IsHoveringDecor` |
| `expertExteriorHovered` | boolean | `C_HousingExpertMode.IsHouseExteriorHovered` |
| `expertHoveredDecorGuid` | optional nonempty GUID; query is whether supplied | `C_HousingExpertMode.IsHoveringDecor` |
| `selectedRoomGuid` | optional GUID; no return when absent | `C_HousingLayout.GetSelectedRoom` |
| `selectedStairwellRoomCount` | nonnegative integer count | `C_HousingLayout.GetSelectedStairwellRoomCount` |
| `previewDyeItemCount` | nonnegative integer count | `C_HousingCustomizeMode.GetNumDyesToSpendOnSelectedDecor` |
| `inOtherPlayersPlot` | boolean | `C_HousingNeighborhood.IsPlayerInOtherPlayersPlot` |

Example: `A_Admin.SetHousingEditorObservation('{"basicExteriorHovered":true,"basicHoveredDecorGuid":null,"cleanupHoveredDecorGuid":"HousingDecor-0-0-0-0-000000000067","customizeHoveredDecorGuid":null,"expertExteriorHovered":true,"expertHoveredDecorGuid":null,"selectedRoomGuid":"Housing-0-0-0-0-000000004001","selectedStairwellRoomCount":14,"previewDyeItemCount":15,"inOtherPlayersPlot":true}')`.

## `A_Admin.SetCollectionObservation(json)`

| JSON field | Type | Query |
|---|---|---|
| `equippedArtifactRelicItemIds` | list of slotted item IDs of the equipped artifact (not the viewed artifact); returns true when nonempty | `C_ArtifactUI.DoesEquippedArtifactHaveAnyRelicsSlotted` |
| `favoriteAppearanceIds` | list of favorite appearance IDs; returns true when nonempty | `C_TransmogCollection.HasFavorites` |

## `A_Admin.SetGamePadConfigurationIDs(json)`

Replaces the supplied configuration ID list with an ordered JSON array. Each object has optional `vendorID` and `productID` unsigned numeric fields. `C_GamePad.GetAllConfigIDs` returns independent copies of these records. This input does not connect a physical device or alter any active mapping.

Example: `A_Admin.SetGamePadConfigurationIDs('[{"vendorID":1356,"productID":1476},{"vendorID":1118}]')`.
