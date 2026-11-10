# Patch 12.1 FrameXML Symbol Inventory

Exhaustive human-readable status inventory for the local 12.1 FrameXML API snapshot. The machine SSOT is `data/patch-api/12.1-framexml.json`; its generated compact checklist is `docs/generated/patch-12-1-framexml-checklist.md`. This page prevents the broad 12.1 audit from silently treating parsed global APIs as the whole source scope.

## Content

**Scope:** 320 added entries and 112 removed entries from `/tmp/warcraft_12_1_framexml.json` (432 entries total). `MacroFrame_SaveMacro` and `PlayerChoiceToggle_TryShow` occur in both source lists, so the snapshot represents 430 distinct names.

**Status rule:** Each entry is classified independently. `implemented` and `best-effort` rows name their modeled behavior and focused coverage. `untriaged` is neutral draft state, not an exception request. Only individually proven unsafe/impossible rows may become `exception-requested`. Vendor presence alone is not focused behavioral coverage.

**Current totals (VERIFIED from manifest, November 9, 2026):** 1 implemented, 334 best-effort, 97 evidence-required, 0 exception-requested, 0 untriaged.

**PTR runtime recheck (`verified: 2026-11-09`):** VERIFIED (full explicit PTR load): 171 of 175 historical added candidates are callable and a byte scan finds their tokens in pinned PTR source; the exact source owners are UNVERIFIED. Four candidates remain unpublished in this simulator runtime; that does not establish native source absence. VERIFIED (full-load runtime): of 99 removed symbols previously labeled vendor-present, 93 are non-callable, four `RaidNotice_*` aliases are callable, and `getglobal`/`setglobal` are Lua compatibility functions. The 93 misses do not establish native absence; their native publication status remains UNVERIFIED. The historical snapshot is unchanged.

### Added symbols

| Symbol | Status | Reason |
|---|---|---|
| `AddBehavioralMessagingTrayToStatusFrames` | best-effort | VERIFIED (runtime): Full PTR loading confirms AddBehavioralMessagingTrayToStatusFrames callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `AddFriendFrame_Show` | best-effort | VERIFIED (runtime): Full PTR loading confirms AddFriendFrame_Show callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `AddGMChatStatusFrameToStatusFrames` | best-effort | VERIFIED (runtime): Full PTR loading confirms AddGMChatStatusFrameToStatusFrames callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `AddTicketStatusFrameToStatusFrames` | best-effort | VERIFIED (runtime): Full PTR loading confirms AddTicketStatusFrameToStatusFrames callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `AddWowSurveyStatusFrameToStatusFrames` | best-effort | VERIFIED (runtime): Full PTR loading confirms AddWowSurveyStatusFrameToStatusFrames callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `AlliedRacesFrame_TryShow` | best-effort | VERIFIED (runtime): Full PTR loading confirms AlliedRacesFrame_TryShow callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `AnchorUtil.ApplyFlowLayout` | best-effort | VERIFIED (runtime): Full PTR loading confirms AnchorUtil.ApplyFlowLayout callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ApplySecureDelegatesToTable` | best-effort | VERIFIED (runtime): Full PTR loading confirms ApplySecureDelegatesToTable callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ArchaeologyFrame_ToggleUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms ArchaeologyFrame_ToggleUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ArcheologyDigsiteProgressBar_OnSurveyCast` | best-effort | VERIFIED (runtime): Full PTR loading confirms ArcheologyDigsiteProgressBar_OnSurveyCast callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ArdenwealdGardening_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms ArdenwealdGardening_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ArtifactFrame_OnTraitsRefunded` | best-effort | VERIFIED (runtime): Full PTR loading confirms ArtifactFrame_OnTraitsRefunded callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `AuraUtil.GetAuraBorderColor` | best-effort | VERIFIED (runtime): Full PTR loading confirms AuraUtil.GetAuraBorderColor callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `AuraUtil.GetUnitAuras` | best-effort | Delegates to existing state-backed aura collection; focused 12.1 bridge test. |
| `AuraUtil.IsValidFilterString` | best-effort | Rust validator accepts nonempty `HELPFUL`, `HARMFUL`, `RAID`, `INCLUDE_NAME_PLATE_ONLY`, and `PLAYER` token combinations; focused 12.1 bridge test. |
| `AzeriteEmpoweredItemUI_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms AzeriteEmpoweredItemUI_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `AzeriteEssenceUI_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms AzeriteEssenceUI_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `BattlefieldMap_ToggleUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms BattlefieldMap_ToggleUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `BehavioralMessaging_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms BehavioralMessaging_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `BehavioralMessagingTray_OnNotification` | best-effort | VERIFIED (runtime): Full PTR loading confirms BehavioralMessagingTray_OnNotification callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `Blizzard_HousingCatalogUtil.AddDecorEntryTooltipTrackingText` | best-effort | VERIFIED (runtime): Full PTR loading confirms Blizzard_HousingCatalogUtil.AddDecorEntryTooltipTrackingText callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `Blizzard_HousingCatalogUtil.TrackHousingDecorID` | best-effort | VERIFIED (runtime): Full PTR loading confirms Blizzard_HousingCatalogUtil.TrackHousingDecorID callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `BNet_GetBattleTagComponents` | best-effort | VERIFIED (runtime): Full PTR loading confirms BNet_GetBattleTagComponents callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `BNet_GetBattleTagSelf` | best-effort | VERIFIED (runtime): Full PTR loading confirms BNet_GetBattleTagSelf callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `BNet_GetBroadcastTextSelf` | best-effort | VERIFIED (runtime): Full PTR loading confirms BNet_GetBroadcastTextSelf callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `BNet_GetFriendLevelRank` | best-effort | VERIFIED (runtime): Full PTR loading confirms BNet_GetFriendLevelRank callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `BNet_IsFriendLevelEqualOrHigher` | best-effort | VERIFIED (runtime): Full PTR loading confirms BNet_IsFriendLevelEqualOrHigher callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `BoostTutorial_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms BoostTutorial_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ChallengeModeCompleteBanner_OnChallengeModeCompleted` | best-effort | VERIFIED (runtime): Full PTR loading confirms ChallengeModeCompleteBanner_OnChallengeModeCompleted callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ChatAdditionalColor_OpenColorPicker` | best-effort | VERIFIED (runtime): Full PTR loading confirms ChatAdditionalColor_OpenColorPicker callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ChatFrameUtil.DiscordNameColorize` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms ChatFrameUtil.DiscordNameColorize is callable. |
| `ChatFrameUtil.FormatDiscordMessage` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms ChatFrameUtil.FormatDiscordMessage is callable. |
| `ChatFrameUtil.GetNameForDiscordMessage` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms ChatFrameUtil.GetNameForDiscordMessage is callable. |
| `CheckActiveStoreForFree` | best-effort | VERIFIED (runtime): Full PTR loading confirms CheckActiveStoreForFree callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CombatAudioAlertUtil.EnumerateInterruptCastInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.EnumerateInterruptCastInfo is callable. |
| `CombatAudioAlertUtil.EnumerateInterruptCastSuccessInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.EnumerateInterruptCastSuccessInfo is callable. |
| `CombatAudioAlertUtil.EnumerateSayCombatEndInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.EnumerateSayCombatEndInfo is callable. |
| `CombatAudioAlertUtil.EnumerateSayCombatStartInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.EnumerateSayCombatStartInfo is callable. |
| `CombatAudioAlertUtil.EnumeratetWhenTargetDiesInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.EnumeratetWhenTargetDiesInfo is callable. |
| `CombatAudioAlertUtil.GetInterruptCastInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.GetInterruptCastInfo is callable. |
| `CombatAudioAlertUtil.GetInterruptCastSuccessInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.GetInterruptCastSuccessInfo is callable. |
| `CombatAudioAlertUtil.GetSayCombatEndInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.GetSayCombatEndInfo is callable. |
| `CombatAudioAlertUtil.GetSayCombatStartInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.GetSayCombatStartInfo is callable. |
| `CombatAudioAlertUtil.GetWhenTargetDiesInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CombatAudioAlertUtil.GetWhenTargetDiesInfo is callable. |
| `CombatText_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms CombatText_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CompactUnitFrame_GetOptionDispelIndicatorOverlayAnimation` | best-effort | VERIFIED (runtime): Full PTR loading confirms CompactUnitFrame_GetOptionDispelIndicatorOverlayAnimation callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CompactUnitFrame_GetOptionDispelIndicatorOverlayType` | best-effort | VERIFIED (runtime): Full PTR loading confirms CompactUnitFrame_GetOptionDispelIndicatorOverlayType callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CompactUnitFrameLayoutTemplates_LayoutFrameElement` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms CompactUnitFrameLayoutTemplates_LayoutFrameElement is callable. |
| `CompactUnitFrameUtil.ApplyConfig` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms CompactUnitFrameUtil.ApplyConfig is callable. |
| `CompactUnitFrameUtil.GenerateNewConfig` | best-effort | VERIFIED (runtime): Full PTR loading confirms CompactUnitFrameUtil.GenerateNewConfig callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ConfirmDisenchantRollDialog_Show` | best-effort | VERIFIED (runtime): Full PTR loading confirms ConfirmDisenchantRollDialog_Show callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ConfirmLootRollDialog_Show` | best-effort | VERIFIED (runtime): Full PTR loading confirms ConfirmLootRollDialog_Show callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ConfirmTalentWipeDialog_Show` | best-effort | VERIFIED (runtime): Full PTR loading confirms ConfirmTalentWipeDialog_Show callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ContributionCollectionFrame_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms ContributionCollectionFrame_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CooldownManagerLayout_GetGroupBuffVisualAlerts` | best-effort | Delegates to existing C_UnitAuras group-buff preference state; focused 12.1 bridge test. |
| `CooldownManagerLayout_GetHiddenGroupBuffs` | best-effort | Delegates to existing C_UnitAuras group-buff preference state; focused 12.1 bridge test. |
| `CooldownManagerLayout_SetGroupBuffVisualAlerts` | best-effort | Delegates to existing C_UnitAuras group-buff preference state; focused 12.1 bridge test. |
| `CooldownManagerLayout_SetHiddenGroupBuffs` | best-effort | Delegates to existing C_UnitAuras group-buff preference state; focused 12.1 bridge test. |
| `CooldownViewer_MarkAuraCacheDirty` | best-effort | VERIFIED (runtime): Full PTR loading confirms CooldownViewer_MarkAuraCacheDirty callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CooldownViewerContextMenu_AddAlertEntryButton` | best-effort | VERIFIED (runtime): Full PTR loading confirms CooldownViewerContextMenu_AddAlertEntryButton callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CooldownViewerContextMenu_AddNewAlertButton` | best-effort | VERIFIED (runtime): Full PTR loading confirms CooldownViewerContextMenu_AddNewAlertButton callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CooldownViewerDraggedItem_Clear` | best-effort | VERIFIED (runtime): Full PTR loading confirms CooldownViewerDraggedItem_Clear callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CooldownViewerDraggedItem_Pickup` | best-effort | VERIFIED (runtime): Full PTR loading confirms CooldownViewerDraggedItem_Pickup callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CooldownViewerDraggedItem_SetIsLegalTarget` | best-effort | VERIFIED (runtime): Full PTR loading confirms CooldownViewerDraggedItem_SetIsLegalTarget callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `CooldownViewerUtil.AddSoundAlertRadio` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CooldownViewerUtil.AddSoundAlertRadio is callable. |
| `CooldownViewerUtil.BuildSoundMenus` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CooldownViewerUtil.BuildSoundMenus is callable. |
| `CooldownViewerUtil.GetSoundTypeSoundKit` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CooldownViewerUtil.GetSoundTypeSoundKit is callable. |
| `CooldownViewerUtil.GetSoundTypeText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms CooldownViewerUtil.GetSoundTypeText is callable. |
| `CovenantCallings_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms CovenantCallings_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `DebugTools_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms DebugTools_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `DifficultyUtil.GetCreatureDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; preserves arguments, both return values, and later hotfix replacement. |
| `DifficultyUtil.GetDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; preserves arguments, both return values, and later hotfix replacement. |
| `DifficultyUtil.GetQuestDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; preserves arguments, both return values, and later hotfix replacement. |
| `DifficultyUtil.GetRelativeDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; focused test covers `(10, 15)`, two-return fidelity, and hot-swap behavior. |
| `DifficultyUtil.GetScalingQuestDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; preserves arguments, both return values, and later hotfix replacement. |
| `EditModeManagerFrame_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms EditModeManagerFrame_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `EncounterJournal_SetTabVisibe` | evidence-required | VERIFIED (runtime observation): Full explicit PTR loading leaves EncounterJournal_SetTabVisibe unpublished; this runtime result does not establish native source absence. |
| `EventTrace_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms EventTrace_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ExpansionTrial_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms ExpansionTrial_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `FadingFrame_CopyTextScalingTime` | best-effort | VERIFIED (runtime): Full PTR loading confirms FadingFrame_CopyTextScalingTime callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `FadingFrame_GetTextScalingMinHeight` | best-effort | VERIFIED (runtime): Full PTR loading confirms FadingFrame_GetTextScalingMinHeight callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `FadingFrame_InitSlot` | best-effort | VERIFIED (runtime): Full PTR loading confirms FadingFrame_InitSlot callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `FadingFrame_SetTextScaling` | best-effort | VERIFIED (runtime): Full PTR loading confirms FadingFrame_SetTextScaling callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `FadingFrame_StartTextScaling` | best-effort | VERIFIED (runtime): Full PTR loading confirms FadingFrame_StartTextScaling callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `FadingFrame_StopTextScaling` | best-effort | VERIFIED (runtime): Full PTR loading confirms FadingFrame_StopTextScaling callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `FadingFrame_UpdateTextScaling` | best-effort | VERIFIED (runtime): Full PTR loading confirms FadingFrame_UpdateTextScaling callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `FriendsListUtil.BuildCharacterClassDisplayText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.BuildCharacterClassDisplayText is callable. |
| `FriendsListUtil.BuildCharacterLevelDisplayText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.BuildCharacterLevelDisplayText is callable. |
| `FriendsListUtil.BuildCharacterNameDisplayText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.BuildCharacterNameDisplayText is callable. |
| `FriendsListUtil.BuildFriendNameDisplayText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.BuildFriendNameDisplayText is callable. |
| `FriendsListUtil.BuildLocationDisplayText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.BuildLocationDisplayText is callable. |
| `FriendsListUtil.BuildTooltipBroadcastText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.BuildTooltipBroadcastText is callable. |
| `FriendsListUtil.GameStateUsesFactions` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GameStateUsesFactions is callable. |
| `FriendsListUtil.GetBattleNetFriendGameAccountInfoIfExactlyOneDirectInviteTargetExists` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetBattleNetFriendGameAccountInfoIfExactlyOneDirectInviteTargetExists is callable. |
| `FriendsListUtil.GetBattleNetFriendInviteInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetBattleNetFriendInviteInfo is callable. |
| `FriendsListUtil.GetBattleNetFriendInviteTypeLabel` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetBattleNetFriendInviteTypeLabel is callable. |
| `FriendsListUtil.GetBattleNetFriendPartyInviteRestrictionText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetBattleNetFriendPartyInviteRestrictionText is callable. |
| `FriendsListUtil.GetBattleNetFriendPartyInviteRestriction` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetBattleNetFriendPartyInviteRestriction is callable. |
| `FriendsListUtil.GetFormattedCharacterName` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetFormattedCharacterName is callable. |
| `FriendsListUtil.GetFriendAccountNameText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetFriendAccountNameText is callable. |
| `FriendsListUtil.GetFriendNameColorForFriendType` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetFriendNameColorForFriendType is callable. |
| `FriendsListUtil.GetFriendNameDisplayColor` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetFriendNameDisplayColor is callable. |
| `FriendsListUtil.GetFriendNameOfflineDisplayColor` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetFriendNameOfflineDisplayColor is callable. |
| `FriendsListUtil.GetGameAccountPartyInviteRestriction` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetGameAccountPartyInviteRestriction is callable. |
| `FriendsListUtil.GetLastOnlineText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetLastOnlineText is callable. |
| `FriendsListUtil.GetRegionName` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetRegionName is callable. |
| `FriendsListUtil.GetRelativeTimeText` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.GetRelativeTimeText is callable. |
| `FriendsListUtil.HasMultipleGameAccounts` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.HasMultipleGameAccounts is callable. |
| `FriendsListUtil.InviteOrRequestToJoin` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.InviteOrRequestToJoin is callable. |
| `FriendsListUtil.IsPlayingDifferentWoWProject` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.IsPlayingDifferentWoWProject is callable. |
| `FriendsListUtil.IsPlayingSameWoWProject` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.IsPlayingSameWoWProject is callable. |
| `FriendsListUtil.IsPlayingWoW` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.IsPlayingWoW is callable. |
| `FriendsListUtil.IsRequestInviteType` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.IsRequestInviteType is callable. |
| `FriendsListUtil.IsTitleFriend` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.IsTitleFriend is callable. |
| `FriendsListUtil.ShouldShowRichPresenceOnly` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms FriendsListUtil.ShouldShowRichPresenceOnly is callable. |
| `GameMenuFrame_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms GameMenuFrame_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GameMenuFrame_IsShown` | best-effort | VERIFIED (runtime): Full PTR loading confirms GameMenuFrame_IsShown callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GameMenuFrame_Show` | best-effort | VERIFIED (runtime): Full PTR loading confirms GameMenuFrame_Show callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GameRulesUtil.IsPlayerAtEffectiveMaxLevel` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms GameRulesUtil.IsPlayerAtEffectiveMaxLevel is callable. |
| `GetBottomManagedFrameContainer` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetBottomManagedFrameContainer callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GetChatAdditionalColor` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetChatAdditionalColor callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GetDiscordUserCommunityLink` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetDiscordUserCommunityLink callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GetDiscordUserLink` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetDiscordUserLink callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GetGarrisonMissionFrameNameForFollowerType` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetGarrisonMissionFrameNameForFollowerType callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GetGarrisonTypeForFollowerType` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetGarrisonTypeForFollowerType callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GetPlayerBottomManagedFrameContainer` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetPlayerBottomManagedFrameContainer callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GetRightManagedFrameContainer` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetRightManagedFrameContainer callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GetTimeSinceLastQuestProgress` | best-effort | Vendor-present in PTRFeedback. Focused proof pins publication and the current upstream nil-arithmetic invocation defect from undefined `lastProgressTime`; no guessed correction is added. |
| `GetTimeStringFromSeconds` | best-effort | Classified as snapshot cross-flavor contamination: the only PTR definition is in `Mists/UIParent.lua`, loaded exclusively by `Blizzard_UIParent_Mists.toc`; PTR tests verify absence during initialization, post-load compatibility, and settled mainline startup. |
| `GetUIPanelLayoutAttribute` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetUIPanelLayoutAttribute callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GetUIPanelLayoutFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms GetUIPanelLayoutFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GMChatFrame_OnWhisperFromGM` | best-effort | VERIFIED (runtime): Full PTR loading confirms GMChatFrame_OnWhisperFromGM callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GossipConfirmDialog_Show` | best-effort | VERIFIED (runtime): Full PTR loading confirms GossipConfirmDialog_Show callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GuildControlDiscord_Loaded_OnEvent` | best-effort | VERIFIED (runtime): Full PTR loading confirms GuildControlDiscord_Loaded_OnEvent callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GuildControlDiscord_Loaded_OnLoad` | best-effort | VERIFIED (runtime): Full PTR loading confirms GuildControlDiscord_Loaded_OnLoad callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GuildControlDiscord_SetGuildSettingsCheckboxes` | best-effort | VERIFIED (runtime): Full PTR loading confirms GuildControlDiscord_SetGuildSettingsCheckboxes callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GuildControlRankDiscord_OnLoad` | best-effort | VERIFIED (runtime): Full PTR loading confirms GuildControlRankDiscord_OnLoad callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `GuildControlUI_Discord_HideAll` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_Discord_HideAll is callable. |
| `GuildControlUI_Discord_Update` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_Discord_Update is callable. |
| `GuildControlUI_DiscordFrame_OnLoad` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_DiscordFrame_OnLoad is callable. |
| `GuildControlUI_LoadUI` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_LoadUI is callable. |
| `GuildControlUI_OnShow` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_OnShow is callable. |
| `GuildControlUI_SetupDiscord` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_SetupDiscord is callable. |
| `GuildControlUI_SetupSelected` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_SetupSelected is callable. |
| `GuildControlUI_Setup` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_Setup is callable. |
| `GuildControlUI_Show` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_Show is callable. |
| `GuildControlUI_UnlinkDiscord` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms GuildControlUI_UnlinkDiscord is callable. |
| `HandleQuestSessionInviteToPartyConfirmation` | best-effort | VERIFIED (runtime): Full PTR loading confirms HandleQuestSessionInviteToPartyConfirmation callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HelpFrame_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms HelpFrame_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HideAuctionHouseFrame` | best-effort | Classified as snapshot/runtime mismatch: no definition exists in the local PTR Blizzard sources; focused PTR runtime test loads `Blizzard_AuctionHouseUI` and verifies the wrapper remains absent rather than inventing behavior. |
| `HideBarberShopFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms HideBarberShopFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HideBlackMarketFrame` | best-effort | Classified as reversed-name snapshot mismatch: PTR defines `BlackMarketFrame_Hide` with panel-hide and close-sound behavior, not `HideBlackMarketFrame`; focused PTR test explicitly loads `Blizzard_BlackMarketUI`, verifies the authoritative helper, and confirms the reversed wrapper remains absent. |
| `HideGarrisonMissionFrames` | best-effort | Classified as snapshot/runtime mismatch: no definition exists in local PTR Blizzard Lua sources; focused PTR test loads `Blizzard_GarrisonUI` with its LoD dependencies and verifies the wrapper remains absent. |
| `HideGarrisonShipyardFrame` | best-effort | Classified as snapshot/runtime mismatch: no definition exists in local PTR Blizzard Lua sources; focused PTR test loads `Blizzard_GarrisonUI` with its LoD dependencies and verifies the wrapper remains absent. |
| `HideGossipFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms HideGossipFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HideGuildBankFrame` | best-effort | Classified as snapshot/runtime mismatch: no definition exists in local PTR Blizzard Lua sources; focused PTR test explicitly loads `Blizzard_GuildBankUI` and verifies the wrapper remains absent. |
| `HideInstanceBootDialog` | best-effort | VERIFIED (runtime): Full PTR loading confirms HideInstanceBootDialog callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HideInstanceLockDialog` | best-effort | VERIFIED (runtime): Full PTR loading confirms HideInstanceLockDialog callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HideItemUpgradeFrame` | best-effort | Classified as vendor-present: the pinned 70493 PTR cache ships `Blizzard_ItemUpgradeUI/Blizzard_ItemUpgradeUI_Bootstrap.lua` (`data/blizzard-ui-files/ptr.txt:2537`), whose line 13 defines `function HideItemUpgradeFrame()` guarding on `C_AddOns.IsAddOnLoaded` and delegating to `ItemUpgradeFrame_Hide`; the focused PTR test loads the full game UI plus the LoD addon, checks the global's debug source, invokes it, and confirms `ItemUpgradeFrame` stays hidden. The earlier reversed-name classification came from a stale local PTR snapshot. |
| `HideProfessionsCustomerOrdersFrame` | best-effort | Classified as snapshot/runtime mismatch: recursive local PTR source scan finds no definition; focused PTR test loads the required ProfessionsTemplates/AuctionHouse dependencies plus `Blizzard_ProfessionsCustomerOrders`, verifies the frame exists, and confirms the wrapper remains absent. |
| `HideSummonConfirmationDialogs` | best-effort | VERIFIED (runtime): Full PTR loading confirms HideSummonConfirmationDialogs callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HouseFinderFrame_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms HouseFinderFrame_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HousingBulletinBoardFrame_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms HousingBulletinBoardFrame_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HousingControls_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms HousingControls_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `HousingFramesUtil.IsBlueprintCollectionAvailable` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms HousingFramesUtil.IsBlueprintCollectionAvailable is callable. |
| `HousingFramesUtil.IsBlueprintOperationInProgress` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms HousingFramesUtil.IsBlueprintOperationInProgress is callable. |
| `HousingFramesUtil.ShowBlueprintExport` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms HousingFramesUtil.ShowBlueprintExport is callable. |
| `HousingFramesUtil.ShowBlueprintImport` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms HousingFramesUtil.ShowBlueprintImport is callable. |
| `HousingFramesUtil.ShowBlueprintRoomExport` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms HousingFramesUtil.ShowBlueprintRoomExport is callable. |
| `HousingFramesUtil.TryOpenBlueprintCollection` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms HousingFramesUtil.TryOpenBlueprintCollection is callable. |
| `HybridMinimap_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms HybridMinimap_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `InputBoxInstructions_OnEnter` | best-effort | VERIFIED (runtime): Full PTR loading confirms InputBoxInstructions_OnEnter callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `InputBoxInstructions_OnLeave` | best-effort | VERIFIED (runtime): Full PTR loading confirms InputBoxInstructions_OnLeave callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `InputBoxInstructions_ShowTooltipIfTruncated` | best-effort | VERIFIED (runtime): Full PTR loading confirms InputBoxInstructions_ShowTooltipIfTruncated callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `InputUtil.CursorOnUpdate` | best-effort | VERIFIED (runtime): Full PTR runtime confirms InputUtil.CursorOnUpdate is callable under InputUtil. |
| `InputUtil.CursorUpdate` | best-effort | VERIFIED (runtime): Full PTR runtime confirms InputUtil.CursorUpdate is callable under InputUtil. |
| `InputUtil.GetCursorDelta` | best-effort | VERIFIED (runtime): Full PTR runtime confirms InputUtil.GetCursorDelta is callable under InputUtil. |
| `InputUtil.IsMouseOver` | best-effort | VERIFIED (runtime): Full PTR runtime confirms InputUtil.IsMouseOver is callable under InputUtil. |
| `InputUtil.ShowInspectCursor` | best-effort | VERIFIED (runtime): Full PTR runtime confirms InputUtil.ShowInspectCursor is callable under InputUtil. |
| `InterfaceUtil.GetScreenHeightScale` | best-effort | VERIFIED (runtime): Full PTR runtime confirms InterfaceUtil.GetScreenHeightScale is callable and returns the scale value. |
| `InterfaceUtil.GetScreenWidthScale` | best-effort | VERIFIED (runtime): Full PTR runtime confirms InterfaceUtil.GetScreenWidthScale is callable and returns the scale value. |
| `InterpolatorUtil.GetSmoothProgressChange` | best-effort | Reversed snapshot: PTR UIParent exports global `GetSmoothProgressChange`, not this namespace member; focused PTR proof confirms the member remains nil while the global computes the expected value. |
| `IslandsPartyPoseFrame_TryShow` | best-effort | VERIFIED (runtime): Full PTR loading confirms IslandsPartyPoseFrame_TryShow callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `IsMouseoverCastSupported` | best-effort | VERIFIED (runtime): Full PTR loading confirms IsMouseoverCastSupported callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `IsSummonConfirmationDialogVisible` | best-effort | VERIFIED (runtime): Full PTR loading confirms IsSummonConfirmationDialogVisible callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `IsTypeAdditionalChatColor` | best-effort | VERIFIED (runtime): Full PTR loading confirms IsTypeAdditionalChatColor callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ItemUtil.DisplayEquipSlotTooltip` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms ItemUtil.DisplayEquipSlotTooltip is callable. |
| `ItemUtil.GetEmptyEquipSlotTooltipForSlotName` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms ItemUtil.GetEmptyEquipSlotTooltipForSlotName is callable. |
| `ItemUtil.GetEmptyEquipSlotTooltip` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms ItemUtil.GetEmptyEquipSlotTooltip is callable. |
| `ItemUtil.GetEquipSlotTexture` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms ItemUtil.GetEquipSlotTexture is callable. |
| `ItemUtil.GetValidatedItemLocation` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms ItemUtil.GetValidatedItemLocation is callable. |
| `Kiosk_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms Kiosk_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `KioskFrame_HandlePlayerEnteringWorld` | evidence-required | VERIFIED (runtime observation): Full explicit PTR loading leaves KioskFrame_HandlePlayerEnteringWorld unpublished; this runtime result does not establish native source absence. |
| `LandingSoulbinds_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms LandingSoulbinds_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `LFGListApplicationViewer_OpenEditMode` | best-effort | VERIFIED (runtime): Full PTR loading confirms LFGListApplicationViewer_OpenEditMode callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `LFGListApplicationViewerRemoveEntryButton_OnClick` | best-effort | VERIFIED (runtime): Full PTR loading confirms LFGListApplicationViewerRemoveEntryButton_OnClick callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `LoadAddOnWithErrorHandling` | implemented | 12.1 wrapper in `src/ptr/compat_bootstrap.lua`; focused delegation test in `global_functions.rs`. |
| `LocaleUtil.GetLocaleDisplayName` | best-effort | VERIFIED (runtime): Full PTR loading confirms LocaleUtil.GetLocaleDisplayName callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `LocalizePlayerFrame_zhCN` | best-effort | VERIFIED (runtime): Full PTR loading confirms LocalizePlayerFrame_zhCN callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `LocalizePlayerFrame_zhTW` | best-effort | VERIFIED (runtime): Full PTR loading confirms LocalizePlayerFrame_zhTW callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `LootFrame_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms LootFrame_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `MacroFrame_SaveMacro` | best-effort | PTR lifecycle proof verifies the eager UIParent no-op placeholder is harmless, then explicit `Blizzard_MacroUI` loading replaces it with the `MacroFrame:SaveMacro()` delegate. |
| `ManageFramePositions` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms ManageFramePositions is callable. |
| `MenuUtil.CreateHighlightButton` | best-effort | VERIFIED (runtime): Full PTR loading confirms MenuUtil.CreateHighlightButton callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `MovePad_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms MovePad_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `NarrationUtil.CreateNarrationInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.CreateNarrationInfo is callable. |
| `NarrationUtil.GetCheckboxContext` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.GetCheckboxContext is callable. |
| `NarrationUtil.MakeIndexInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.MakeIndexInfo is callable. |
| `NarrationUtil.MakeNarrationStringForMoney` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.MakeNarrationStringForMoney is callable. |
| `NarrationUtil.MakeNarrationStringFromIndexInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.MakeNarrationStringFromIndexInfo is callable. |
| `NarrationUtil.MakeNarrationStringFromInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.MakeNarrationStringFromInfo is callable. |
| `NarrationUtil.MakeNarrationString` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.MakeNarrationString is callable. |
| `NarrationUtil.NarrateCurrentScreen` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.NarrateCurrentScreen is callable. |
| `NarrationUtil.RegionToNarrationInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.RegionToNarrationInfo is callable. |
| `NarrationUtil.ResolveForwardedRegion` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.ResolveForwardedRegion is callable. |
| `NarrationUtil.SetStaticDescription` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.SetStaticDescription is callable. |
| `NarrationUtil.SetStaticName` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.SetStaticName is callable. |
| `NarrationUtil.ShouldBeEnabled` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.ShouldBeEnabled is callable. |
| `NarrationUtil.ShouldRegionNavigationSkipTooltips` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms NarrationUtil.ShouldRegionNavigationSkipTooltips is callable. |
| `NPE_InitializeIfLoaded` | best-effort | VERIFIED (runtime): Full PTR loading confirms NPE_InitializeIfLoaded callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `OpacityFrame_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms OpacityFrame_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `OpenEncounterJournalToJourney` | best-effort | VERIFIED (runtime): Full PTR loading confirms OpenEncounterJournalToJourney callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `OpenOrderHallTalentUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms OpenOrderHallTalentUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `OpenPlayerSpellsToGlyphTarget` | best-effort | VERIFIED (runtime): Full PTR loading confirms OpenPlayerSpellsToGlyphTarget callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `PhotoSharingFrame_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms PhotoSharingFrame_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `PingUtil.SendMacroPing` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms PingUtil.SendMacroPing is callable. |
| `PingUtil.TogglePingTarget` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms PingUtil.TogglePingTarget is callable. |
| `PlayerChoiceFrame_TryShow` | best-effort | VERIFIED (runtime): Full PTR loading confirms PlayerChoiceFrame_TryShow callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `PlayerChoiceToggle_TryShow` | best-effort | PTR LoD behavior: absent before `Blizzard_PlayerChoice`, present afterward; focused proof verifies eligible button visibility, explicit plus OnShow updates, and nil return. |
| `PVPUI_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms PVPUI_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `RaidWarningUtil.AddMessage` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms RaidWarningUtil.AddMessage is callable. |
| `RaidWarningUtil.ClearBossEmotes` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms RaidWarningUtil.ClearBossEmotes is callable. |
| `RaidWarningUtil.UpdateCenterScreenAnchors` | best-effort | VERIFIED (runtime): Full PTR loading confirms RaidWarningUtil.UpdateCenterScreenAnchors callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `RecentAlliesUtil.GetBestSocialUIPresenceTypeForStateData` | best-effort | VERIFIED (runtime): Full PTR loading confirms RecentAlliesUtil.GetBestSocialUIPresenceTypeForStateData callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `RecruitAFriendFrameSocialInitializeAADC` | evidence-required | VERIFIED (runtime observation): Full explicit PTR loading leaves RecruitAFriendFrameSocialInitializeAADC unpublished; this runtime result does not establish native source absence. |
| `RegionUtil.GetTopLeftMost` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms RegionUtil.GetTopLeftMost is callable. |
| `RegionUtil.SortByTopLeft` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms RegionUtil.SortByTopLeft is callable. |
| `RegisterGameMenuEscHandler` | best-effort | VERIFIED (runtime): Full PTR loading confirms RegisterGameMenuEscHandler callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `RegisterPlayerInteraction` | best-effort | VERIFIED (runtime): Full PTR loading confirms RegisterPlayerInteraction callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ReportFrame_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms ReportFrame_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ResetDiscordSettings` | best-effort | VERIFIED (runtime): Full PTR loading confirms ResetDiscordSettings callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `RestoreGMChatFrameSession` | best-effort | VERIFIED (runtime): Full PTR loading confirms RestoreGMChatFrameSession callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `SetGhostFrameShown` | best-effort | VERIFIED (runtime): Full PTR loading confirms SetGhostFrameShown callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `SetPlayerInteractionConditions` | best-effort | VERIFIED (runtime): Full PTR loading confirms SetPlayerInteractionConditions callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `SettingsPanel_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms SettingsPanel_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `SetUIPanelLayoutAttribute` | best-effort | VERIFIED (runtime): Full PTR loading confirms SetUIPanelLayoutAttribute callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShakeFrameRandom` | best-effort | VERIFIED (runtime): Full PTR runtime publishes the legacy global ShakeFrameRandom and its ScriptAnimationUtil counterpart. |
| `ShakeFrame` | best-effort | VERIFIED (runtime): Full PTR runtime publishes the legacy global ShakeFrame and its ScriptAnimationUtil counterpart. |
| `ShouldDisplaySpellCooldown` | evidence-required | VERIFIED (runtime observation): Full explicit PTR loading leaves ShouldDisplaySpellCooldown unpublished; this runtime result does not establish native source absence. |
| `ShowAchievementFrameForAchievement` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowAchievementFrameForAchievement callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowAdventureMapFrameForFollowerType` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowAdventureMapFrameForFollowerType callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowArtifactFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowArtifactFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowArtifactRelicForgeFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowArtifactRelicForgeFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowAuctionHouseFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowAuctionHouseFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowBarberShopFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowBarberShopFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowBlackMarketFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowBlackMarketFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowChallengesKeystoneFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowChallengesKeystoneFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowFlightMapFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowFlightMapFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowGarrisonCapacitiveDisplayFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowGarrisonCapacitiveDisplayFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowGarrisonMissionFrameForFollowerType` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowGarrisonMissionFrameForFollowerType callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowGarrisonRecruiterFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowGarrisonRecruiterFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowGarrisonShipyardFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowGarrisonShipyardFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowGuildBankFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowGuildBankFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowHeirloomsJournalToClosestUpgradeablePage` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowHeirloomsJournalToClosestUpgradeablePage callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowInstanceBootDialog` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowInstanceBootDialog callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowInstanceLockDialog` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowInstanceLockDialog callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowItemSocketingFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowItemSocketingFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowItemUpgradeFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowItemUpgradeFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowMatchCelebrationPartyPoseFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowMatchCelebrationPartyPoseFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowPendingPlayerChoiceResponseUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowPendingPlayerChoiceResponseUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowPerksProgramFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowPerksProgramFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowProfessionEquipmentHelpTip` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowProfessionEquipmentHelpTip callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowProfessionsCustomerOrdersFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowProfessionsCustomerOrdersFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowProfessionsFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowProfessionsFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowQuestSessionGroupInviteConfirmation` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowQuestSessionGroupInviteConfirmation callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowQuestSessionGroupInviteReceivedConfirmation` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowQuestSessionGroupInviteReceivedConfirmation callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowRemixArtifactFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowRemixArtifactFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowRuneforgeFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowRuneforgeFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowSummonConfirmationDialog` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowSummonConfirmationDialog callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ShowTaxiMapFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms ShowTaxiMapFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `SimpleCheckout_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms SimpleCheckout_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `SocialUIContactsFrameInitializeAADC` | best-effort | VERIFIED (runtime): Full PTR loading confirms SocialUIContactsFrameInitializeAADC callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `SocialUIUtil.AddSeparatorToTooltip` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.AddSeparatorToTooltip is callable. |
| `SocialUIUtil.GetBattleNetFriendTagInterestsUIOrder` | best-effort | VERIFIED (runtime): Full PTR load leaves the former getter SocialUIUtil.GetBattleNetFriendTagInterestsUIOrder absent; current tag-order accessors remain callable. |
| `SocialUIUtil.GetBattleNetFriendTagRoleUIOrder` | best-effort | VERIFIED (runtime): Full PTR load leaves the former getter SocialUIUtil.GetBattleNetFriendTagRoleUIOrder absent; current tag-order accessors remain callable. |
| `SocialUIUtil.GetBlockedName` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.GetBlockedName is callable. |
| `SocialUIUtil.GetIconForPresenceType` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.GetIconForPresenceType is callable. |
| `SocialUIUtil.GetLabelForBattleNetFriendTag` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.GetLabelForBattleNetFriendTag is callable. |
| `SocialUIUtil.GetLabelForPresenceType` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.GetLabelForPresenceType is callable. |
| `SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo is callable. |
| `SocialUIUtil.GetPresenceTypeSelf` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.GetPresenceTypeSelf is callable. |
| `SocialUIUtil.InitializeUserScaledDropdownButton` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.InitializeUserScaledDropdownButton is callable. |
| `SocialUIUtil.InitializeUserScaledDropdownMainTitle` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.InitializeUserScaledDropdownMainTitle is callable. |
| `SocialUIUtil.InitializeUserScaledDropdownTitle` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.InitializeUserScaledDropdownTitle is callable. |
| `SocialUIUtil.SetBattleNetPresenceFromSocialUIPresence` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms SocialUIUtil.SetBattleNetPresenceFromSocialUIPresence is callable. |
| `SoulbindViewer_LoadUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms SoulbindViewer_LoadUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `SpellFlyout_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms SpellFlyout_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `SplashFrame_EscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms SplashFrame_EscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `StoreEscapePressed` | best-effort | VERIFIED (runtime): Full PTR loading confirms StoreEscapePressed callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `StringUtil.JoinAlternatingConditionalColor` | best-effort | VERIFIED (runtime): Full PTR loading confirms StringUtil.JoinAlternatingConditionalColor callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `TextureUtil.AnimateTexCoords` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms TextureUtil.AnimateTexCoords is callable. |
| `TimeUtil.BetterDate` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms TimeUtil.BetterDate is callable. |
| `TimeUtil.GetRecentTimeDate` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms TimeUtil.GetRecentTimeDate is callable. |
| `ToggleRAFPanel` | best-effort | VERIFIED (runtime): Full PTR loading confirms ToggleRAFPanel callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `ToggleSocialUI` | best-effort | VERIFIED (runtime): Full PTR loading confirms ToggleSocialUI callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `TryShowAnimaDiversionFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms TryShowAnimaDiversionFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `TryShowCovenantPreviewFrame` | best-effort | VERIFIED (runtime): Full PTR loading confirms TryShowCovenantPreviewFrame callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `UIModeUtil.CreateExtendedBlocklist` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms UIModeUtil.CreateExtendedBlocklist is callable. |
| `UIModeUtil.CreateModifiedBlocklist` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms UIModeUtil.CreateModifiedBlocklist is callable. |
| `UIModeUtil.IsModeActive` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms UIModeUtil.IsModeActive is callable. |
| `UIModeUtil.RegisterMode` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms UIModeUtil.RegisterMode is callable. |
| `UIModeUtil.SetModeActive` | best-effort | VERIFIED (runtime): Full explicit PTR LoD loading confirms UIModeUtil.SetModeActive is callable. |
| `UnitPopupSharedUtil.IsFriendshipUpgrade` | best-effort | VERIFIED (runtime): Full PTR loading confirms UnitPopupSharedUtil.IsFriendshipUpgrade callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `UpdateQuestAcceptLogFullDialog` | best-effort | VERIFIED (runtime): Full PTR loading confirms UpdateQuestAcceptLogFullDialog callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `VisualAlert_GetTypeTemplate` | best-effort | VERIFIED (runtime): Full PTR loading confirms VisualAlert_GetTypeTemplate callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `VisualAlert_GetTypeText` | best-effort | VERIFIED (runtime): Full PTR loading confirms VisualAlert_GetTypeText callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `VisualAlertData_ForEach` | best-effort | VERIFIED (runtime): Full PTR loading confirms VisualAlertData_ForEach callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `VisualAlerts_RegisterAll` | best-effort | VERIFIED (runtime): Full PTR loading confirms VisualAlerts_RegisterAll callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `WarfrontsPartyPoseFrame_TryShow` | best-effort | VERIFIED (runtime): Full PTR loading confirms WarfrontsPartyPoseFrame_TryShow callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |
| `WowSurveyStatusFrame_OnSurveyDelivered` | best-effort | VERIFIED (runtime): Full PTR loading confirms WowSurveyStatusFrame_OnSurveyDelivered callable; a byte scan finds its token in pinned PTR source, but the source owner is not mapped. |

### Removed symbols

| Symbol | Status | Reason |
|---|---|---|
| `AddFrameLock` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes AddFrameLock as non-callable; this does not establish native API absence. |
| `AnimatedShine_OnUpdate` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes AnimatedShine_OnUpdate as non-callable; this does not establish native API absence. |
| `AnimateTexCoords` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes AnimateTexCoords as non-callable; this does not establish native API absence. |
| `BattleTagInviteFrame_Show` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes BattleTagInviteFrame_Show as non-callable; this does not establish native API absence. |
| `BetterDate` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes BetterDate as non-callable; this does not establish native API absence. |
| `BFAMissionFrame_EscapePressed` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes BFAMissionFrame_EscapePressed as non-callable; this does not establish native API absence. |
| `BoostTutorial_AttemptLoad` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes BoostTutorial_AttemptLoad as non-callable; this does not establish native API absence. |
| `BuildColoredListString` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes BuildColoredListString as non-callable; this does not establish native API absence. |
| `BuildIconArray` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes BuildIconArray as non-callable; this does not establish native API absence. |
| `BuildListString` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes BuildListString as non-callable; this does not establish native API absence. |
| `BuildMultilineTooltip` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes BuildMultilineTooltip as non-callable; this does not establish native API absence. |
| `BuildNewLineListString` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes BuildNewLineListString as non-callable; this does not establish native API absence. |
| `ButtonPulse_OnUpdate` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ButtonPulse_OnUpdate as non-callable; this does not establish native API absence. |
| `ClassTrainerFrame_Hide` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ClassTrainerFrame_Hide as non-callable; this does not establish native API absence. |
| `ClassTrainerFrame_Show` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ClassTrainerFrame_Show as non-callable; this does not establish native API absence. |
| `CloseCalendarMenus` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes CloseCalendarMenus as non-callable; this does not establish native API absence. |
| `CommunitiesFrame_IsEnabled` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes CommunitiesFrame_IsEnabled as non-callable; this does not establish native API absence. |
| `CompactUnitFrame_GetOptionDisplayOnlyHealerPowerBars` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes CompactUnitFrame_GetOptionDisplayOnlyHealerPowerBars as non-callable; this does not establish native API absence. |
| `CompactUnitFrame_GetOptionDisplayPowerBar` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes CompactUnitFrame_GetOptionDisplayPowerBar as non-callable; this does not establish native API absence. |
| `CompactUnitFrame_GetOptionShowDispelIndicatorOverlay` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes CompactUnitFrame_GetOptionShowDispelIndicatorOverlay as non-callable; this does not establish native API absence. |
| `ConvertRGBtoColorString` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ConvertRGBtoColorString as non-callable; this does not establish native API absence. |
| `DisplayTypeUnassignedSupported` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes DisplayTypeUnassignedSupported as non-callable; this does not establish native API absence. |
| `EventUtil.AreVariablesLoaded` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes EventUtil.AreVariablesLoaded as non-callable; this does not establish native API absence. |
| `ExpansionTrial_CheckLoadUI` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ExpansionTrial_CheckLoadUI as non-callable; this does not establish native API absence. |
| `FriendsFrame_CloseQuickJoinHelpTip` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes FriendsFrame_CloseQuickJoinHelpTip as non-callable; this does not establish native API absence. |
| `FriendsFrameAddFriendButton_OnClick` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes FriendsFrameAddFriendButton_OnClick as non-callable; this does not establish native API absence. |
| `FriendsFriends_InitButton` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes FriendsFriends_InitButton as non-callable; this does not establish native API absence. |
| `FriendsFriends_SetSelection` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes FriendsFriends_SetSelection as non-callable; this does not establish native API absence. |
| `FriendsFriendsButton_SetSelected` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes FriendsFriendsButton_SetSelected as non-callable; this does not establish native API absence. |
| `FriendsFriendsFrame_Close` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes FriendsFriendsFrame_Close as non-callable; this does not establish native API absence. |
| `GetDungeonNameWithDifficulty` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes GetDungeonNameWithDifficulty as non-callable; this does not establish native API absence. |
| `GetNotchHeight` | best-effort | VERIFIED (runtime): Full PTR runtime leaves the former global GetNotchHeight nil; UpdateUIParentPosition remains callable and applies the supplied offsets. |
| `GetScaledCursorDelta` | best-effort | VERIFIED (runtime): Full PTR runtime leaves the former global GetScaledCursorDelta nil while the InputUtil helpers are callable. |
| `GetScaledCursorPositionForFrame` | best-effort | VERIFIED (runtime): Full PTR runtime leaves the former global GetScaledCursorPositionForFrame nil while the InputUtil helpers are callable. |
| `GetScaledCursorPosition` | best-effort | VERIFIED (runtime): Full PTR runtime leaves the former global GetScaledCursorPosition nil while the InputUtil helpers are callable. |
| `GetScreenHeightScale` | best-effort | VERIFIED (runtime): Full PTR runtime leaves the former global GetScreenHeightScale nil; the InterfaceUtil method is callable. |
| `GetScreenWidthScale` | best-effort | VERIFIED (runtime): Full PTR runtime leaves the former global GetScreenWidthScale nil; the InterfaceUtil method is callable. |
| `GetSmoothProgressChange` | best-effort | Vendor-present despite the snapshot removal: PTR UIParent retains this global function; focused PTR proof verifies representative input `(100, 0, 100, 1)` returns `70`. |
| `GetSocialColoredName` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes GetSocialColoredName as non-callable; this does not establish native API absence. |
| `GetSortedSelfResurrectOptions` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes GetSortedSelfResurrectOptions as non-callable; this does not establish native API absence. |
| `GetUIParentOffset` | best-effort | VERIFIED (runtime): Full PTR runtime leaves the former global GetUIParentOffset nil; UpdateUIParentPosition remains callable and applies the supplied offsets. |
| `HelpPlatesSupported` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes HelpPlatesSupported as non-callable; this does not establish native API absence. |
| `HousingControlsUtil.CanActivateHousingControls` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes HousingControlsUtil.CanActivateHousingControls as non-callable; this does not establish native API absence. |
| `IsFrameLockActive` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes IsFrameLockActive as non-callable; this does not establish native API absence. |
| `IsFrameSmartShown` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes IsFrameSmartShown as non-callable; this does not establish native API absence. |
| `IsLevelAtEffectiveMaxLevel` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes IsLevelAtEffectiveMaxLevel as non-callable; this does not establish native API absence. |
| `IsPlayerAtEffectiveMaxLevel` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes IsPlayerAtEffectiveMaxLevel as non-callable; this does not establish native API absence. |
| `KeyBindingFrame_LoadUI` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes KeyBindingFrame_LoadUI as non-callable; this does not establish native API absence. |
| `LocalizePlayerFrame` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes LocalizePlayerFrame as non-callable; this does not establish native API absence. |
| `LocalizezhCN` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes LocalizezhCN as non-callable; this does not establish native API absence. |
| `LocalizezhTW` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes LocalizezhTW as non-callable; this does not establish native API absence. |
| `MacroFrame_SaveMacro` | best-effort | PTR lifecycle proof verifies the eager UIParent no-op placeholder is harmless, then explicit `Blizzard_MacroUI` loading replaces it with the `MacroFrame:SaveMacro()` delegate. |
| `MajorFactions_LoadUI` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes MajorFactions_LoadUI as non-callable; this does not establish native API absence. |
| `MouseIsOver` | best-effort | VERIFIED (runtime): Full PTR runtime leaves the former global MouseIsOver nil while the InputUtil helpers are callable. |
| `NPE_CheckTutorials` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes NPE_CheckTutorials as non-callable; this does not establish native API absence. |
| `NPETutorial_AttemptToBegin` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes NPETutorial_AttemptToBegin as non-callable; this does not establish native API absence. |
| `OpenAchievementFrameToAchievement` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes OpenAchievementFrameToAchievement as non-callable; this does not establish native API absence. |
| `OrderHallMissionFrame_EscapePressed` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes OrderHallMissionFrame_EscapePressed as non-callable; this does not establish native API absence. |
| `OrderHallTalentFrame_EscapePressed` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes OrderHallTalentFrame_EscapePressed as non-callable; this does not establish native API absence. |
| `OutfitterUI_LoadUI` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes OutfitterUI_LoadUI as non-callable; this does not establish native API absence. |
| `PingUtil.GetContextualPingTypeForUnit` | best-effort | VERIFIED (runtime): Full PTR load leaves PingUtil.GetContextualPingTypeForUnit and C_Ping.GetContextualPingTypeForUnit absent. |
| `PlayerChoiceToggle_TryShow` | best-effort | Retained PTR LoD behavior despite snapshot removal: absent before `Blizzard_PlayerChoice`, published afterward with focused button-state proof. |
| `QuickJoin_JoinQueueButtonOnClick` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes QuickJoin_JoinQueueButtonOnClick as non-callable; this does not establish native API absence. |
| `RaidBossEmoteFrame_OnEvent` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RaidBossEmoteFrame_OnEvent as non-callable; this does not establish native API absence. |
| `RaidBossEmoteFrame_OnLoad` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RaidBossEmoteFrame_OnLoad as non-callable; this does not establish native API absence. |
| `RaidBrowser_IsEmpowered` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RaidBrowser_IsEmpowered as non-callable; this does not establish native API absence. |
| `RaidNotice_AddMessage` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms RaidNotice_AddMessage is callable as a selected deprecated global alias. |
| `RaidNotice_ClearSlot` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RaidNotice_ClearSlot as non-callable; this does not establish native API absence. |
| `RaidNotice_Clear` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms RaidNotice_Clear is callable as a selected deprecated global alias. |
| `RaidNotice_FadeInit` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms RaidNotice_FadeInit is callable as a selected deprecated global alias. |
| `RaidNotice_OnUpdate` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RaidNotice_OnUpdate as non-callable; this does not establish native API absence. |
| `RaidNotice_SetSlot` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RaidNotice_SetSlot as non-callable; this does not establish native API absence. |
| `RaidNotice_UpdateSlot` | best-effort | VERIFIED (runtime): Full-LoD PTR runtime confirms RaidNotice_UpdateSlot is callable as a selected deprecated global alias. |
| `RaidWarningFrame_OnEvent` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RaidWarningFrame_OnEvent as non-callable; this does not establish native API absence. |
| `RaidWarningFrame_OnLoad` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RaidWarningFrame_OnLoad as non-callable; this does not establish native API absence. |
| `RecentTimeDate` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RecentTimeDate as non-callable; this does not establish native API absence. |
| `RefreshAuras` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RefreshAuras as non-callable; this does not establish native API absence. |
| `RegisterNewFrameLock` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RegisterNewFrameLock as non-callable; this does not establish native API absence. |
| `RemoveFrameLock` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes RemoveFrameLock as non-callable; this does not establish native API absence. |
| `ReverseQuestObjective` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ReverseQuestObjective as non-callable; this does not establish native API absence. |
| `SetDesaturation` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes SetDesaturation as non-callable; this does not establish native API absence. |
| `SetFrameLock` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes SetFrameLock as non-callable; this does not establish native API absence. |
| `ShowResurrectRequest` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ShowResurrectRequest as non-callable; this does not establish native API absence. |
| `SmartHide` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes SmartHide as non-callable; this does not establish native API absence. |
| `SmartShow` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes SmartShow as non-callable; this does not establish native API absence. |
| `TalentFrame_LoadUI` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes TalentFrame_LoadUI as non-callable; this does not establish native API absence. |
| `TargetFrame_UpdateBuffAnchor` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes TargetFrame_UpdateBuffAnchor as non-callable; this does not establish native API absence. |
| `TargetFrame_UpdateDebuffAnchor` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes TargetFrame_UpdateDebuffAnchor as non-callable; this does not establish native API absence. |
| `ToggleLFGFrame` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ToggleLFGFrame as non-callable; this does not establish native API absence. |
| `ToggleRafPanel` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ToggleRafPanel as non-callable; this does not establish native API absence. |
| `ToggleRaidBrowser` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ToggleRaidBrowser as non-callable; this does not establish native API absence. |
| `ToggleWoWHackCharacterUI` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes ToggleWoWHackCharacterUI as non-callable; this does not establish native API absence. |
| `TokenFrame_LoadUI` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes TokenFrame_LoadUI as non-callable; this does not establish native API absence. |
| `TrialAccountCapReached_Inform` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes TrialAccountCapReached_Inform as non-callable; this does not establish native API absence. |
| `UIDoFramesIntersect` | best-effort | VERIFIED (runtime): Full PTR runtime leaves the former global UIDoFramesIntersect nil; UpdateUIParentPosition remains callable and applies the supplied offsets. |
| `UIParent_ManageFramePositions` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UIParent_ManageFramePositions as non-callable; this does not establish native API absence. |
| `UIParent_OnEvent` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UIParent_OnEvent as non-callable; this does not establish native API absence. |
| `UIParent_OnHide` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UIParent_OnHide as non-callable; this does not establish native API absence. |
| `UIParent_OnLoad` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UIParent_OnLoad as non-callable; this does not establish native API absence. |
| `UIParent_OnShow` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UIParent_OnShow as non-callable; this does not establish native API absence. |
| `UIParent_Shared_OnEvent` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UIParent_Shared_OnEvent as non-callable; this does not establish native API absence. |
| `UIParent_Shared_OnLoad` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UIParent_Shared_OnLoad as non-callable; this does not establish native API absence. |
| `UIParent_UpdateTopFramePositions` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UIParent_UpdateTopFramePositions as non-callable; this does not establish native API absence. |
| `UIParentLoadAddOn` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UIParentLoadAddOn as non-callable; this does not establish native API absence. |
| `UnitHasMana` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UnitHasMana as non-callable; this does not establish native API absence. |
| `UpdateFrameLock` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UpdateFrameLock as non-callable; this does not establish native API absence. |
| `UpdateUIElementsForClientScene` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes UpdateUIElementsForClientScene as non-callable; this does not establish native API absence. |
| `WorldFrame_OnLoad` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes WorldFrame_OnLoad as non-callable; this does not establish native API absence. |
| `WorldFrame_OnUpdate` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes WorldFrame_OnUpdate as non-callable; this does not establish native API absence. |
| `WoWHackSpellsUI_LoadUI` | evidence-required | VERIFIED (runtime observation): Full explicit PTR addon loading observes WoWHackSpellsUI_LoadUI as non-callable; this does not establish native API absence. |
| `getglobal` | best-effort | VERIFIED (runtime): The full-load simulator runtime exposes getglobal as a Lua 5.1 compatibility function, not as a PTR FrameXML publication. |
| `setglobal` | best-effort | VERIFIED (runtime): The full-load simulator runtime exposes setglobal as a Lua 5.1 compatibility function, not as a PTR FrameXML publication. |

## Sources

- [Raw 12.1 FrameXML patch list](../../../data/patch-api/sources/12.1-framexml.json) — checked-in exact added/removed symbol arrays.
- [12.1 FrameXML audit register](../../../data/patch-api/12.1-framexml.json) — per-row machine SSOT and lifecycle/evidence metadata.
- [Generated compact checklist](../../generated/patch-12-1-framexml-checklist.md) — one itemized line per patch-list occurrence.
- [PTR compatibility bootstrap](../../../src/ptr/compat_bootstrap.lua) — implemented `LoadAddOnWithErrorHandling` wrapper and post-load dynamic `DifficultyUtil` color delegates.
- [Global-function regressions](../../../src/loader/tests/wow_api_globals/global_functions.rs) — focused `LoadAddOnWithErrorHandling` wrapper regression.
- [PTR compatibility tests](../../../src/ptr/compat_bootstrap.rs) — focused reset, argument/return, hot-swap, missing-global, and preservation coverage for the five `DifficultyUtil` delegates.
- [FrameXMLUtil lifecycle tests](../../../tests/blizzard_frame_xml_util_loads.rs) — full PTR Game UI lifecycle/threshold proof and older-retail non-exposure proof.
- [Garrison UI tests](../../../tests/blizzard_garrison_ui_loads.rs) — recursive PTR Garrison source scan plus explicit LoD addon-load proof for the two absent Garrison hide wrappers.
- [Customer Orders tests](../../../tests/blizzard_professions_customer_orders_loads.rs) — recursive PTR source scan plus explicit dependency/addon-load proof for the absent CustomerOrders hide wrapper.
- [Input utility snapshot proof](../../../patch-tests/patch_12_1/input_util.rs) — focused PTR publication and cursor-behavior proof for the stale namespace move and retained globals.
- [UI geometry snapshot proof](../../../patch-tests/patch_12_1/ui_geometry.rs) — focused PTR intersection, notch normalization, and UIParent offset proof for three retained globals.
- [Combat audio snapshot proof](../../../patch-tests/patch_12_1/combat_audio.rs) — recursive PTR source scan and runtime absence proof for ten stale proposed methods.
- [Friends list snapshot proof](../../../patch-tests/patch_12_1/friends_list.rs) — exact-qualified PTR source scan and runtime absence proof for 29 stale proposed methods.
- [Social UI snapshot proof](../../../patch-tests/patch_12_1/social_ui.rs) — exact-qualified PTR source scan and runtime absence proof for 13 stale proposed methods.
- [Narration snapshot proof](../../../patch-tests/patch_12_1/narration.rs) — exact-qualified PTR source scan and runtime absence proof for 14 stale proposed methods.
- [Guild control snapshot proof](../../../patch-tests/patch_12_1/guild_control.rs) — PTR source and runtime absence proof for ten stale proposed globals.
- [Utility namespace snapshot proof](../../../patch-tests/patch_12_1/utility_namespaces.rs) — colon-aware source/runtime proof for 29 stale additions and one retained Ping helper.
- [Conservative source-absence proof](../../../patch-tests/patch_12_1/source_absent.rs) — itemized bare-token source and full-load runtime absence proof for 175 proposed additions.
- [PTR source-audit contract](../../specs/ptr-source-audit-fixtures.md) — pinned 12.1.7 evidence, conditional deprecated RaidWarning publishers and the limit on native-absence claims.
- [Final publication matrix](../../../patch-tests/patch_12_1/remaining_observations.rs) — seven source-declared publications, four conditional deprecated RaidWarning aliases, and three absent native members; it does not claim native absence for historical paths that are not callable in the simulator.

## See Also

- [[patch-api-audit-manifest]] — manifest schema, validation, lifecycle assertions, and checklist generation.
- [[patch-12-1-api-audit]] — patch-level implementation and exception matrix.
- [[client-profiles]] — cumulative retail epoch feature gating.
- [[lua-api]] — runtime Lua surface context.
