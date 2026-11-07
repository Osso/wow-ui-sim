# Patch 12.1 FrameXML Symbol Inventory

Exhaustive human-readable status inventory for the local 12.1 FrameXML API snapshot. The machine SSOT is `data/patch-api/12.1-framexml.json`; its generated compact checklist is `docs/generated/patch-12-1-framexml-checklist.md`. This page prevents the broad 12.1 audit from silently treating parsed global APIs as the whole source scope.

## Content

**Scope:** 320 added entries and 112 removed entries from `/tmp/warcraft_12_1_framexml.json` (432 entries total). `MacroFrame_SaveMacro` and `PlayerChoiceToggle_TryShow` occur in both source lists, so the snapshot represents 430 distinct names.

**Status rule:** Each entry is classified independently. `implemented` and `best-effort` rows name their modeled behavior and focused coverage. `untriaged` is neutral draft state, not an exception request. Only individually proven unsafe/impossible rows may become `exception-requested`. Vendor presence alone is not focused behavioral coverage.

**Current totals:** 1 implemented, 431 best-effort, 0 exception-requested, 0 untriaged.

### Added symbols

| Symbol | Status | Reason |
|---|---|---|
| `AddBehavioralMessagingTrayToStatusFrames` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `AddFriendFrame_Show` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `AddGMChatStatusFrameToStatusFrames` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `AddTicketStatusFrameToStatusFrames` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `AddWowSurveyStatusFrameToStatusFrames` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `AlliedRacesFrame_TryShow` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `AnchorUtil.ApplyFlowLayout` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ApplySecureDelegatesToTable` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ArchaeologyFrame_ToggleUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ArcheologyDigsiteProgressBar_OnSurveyCast` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ArdenwealdGardening_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ArtifactFrame_OnTraitsRefunded` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `AuraUtil.GetAuraBorderColor` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `AuraUtil.GetUnitAuras` | best-effort | Delegates to existing state-backed aura collection; focused 12.1 bridge test. |
| `AuraUtil.IsValidFilterString` | best-effort | Rust validator accepts nonempty `HELPFUL`, `HARMFUL`, `RAID`, `INCLUDE_NAME_PLATE_ONLY`, and `PLAYER` token combinations; focused 12.1 bridge test. |
| `AzeriteEmpoweredItemUI_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `AzeriteEssenceUI_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `BattlefieldMap_ToggleUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `BehavioralMessaging_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `BehavioralMessagingTray_OnNotification` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `Blizzard_HousingCatalogUtil.AddDecorEntryTooltipTrackingText` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `Blizzard_HousingCatalogUtil.TrackHousingDecorID` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `BNet_GetBattleTagComponents` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `BNet_GetBattleTagSelf` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `BNet_GetBroadcastTextSelf` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `BNet_GetFriendLevelRank` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `BNet_IsFriendLevelEqualOrHigher` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `BoostTutorial_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ChallengeModeCompleteBanner_OnChallengeModeCompleted` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ChatAdditionalColor_OpenColorPicker` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ChatFrameUtil.DiscordNameColorize` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `ChatFrameUtil.FormatDiscordMessage` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `ChatFrameUtil.GetNameForDiscordMessage` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `CheckActiveStoreForFree` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CombatAudioAlertUtil.EnumerateInterruptCastInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatAudioAlertUtil.EnumerateInterruptCastSuccessInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatAudioAlertUtil.EnumerateSayCombatEndInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatAudioAlertUtil.EnumerateSayCombatStartInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatAudioAlertUtil.EnumeratetWhenTargetDiesInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatAudioAlertUtil.GetInterruptCastInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatAudioAlertUtil.GetInterruptCastSuccessInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatAudioAlertUtil.GetSayCombatEndInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatAudioAlertUtil.GetSayCombatStartInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatAudioAlertUtil.GetWhenTargetDiesInfo` | best-effort | Stale snapshot addition: recursive PTR source proof finds no publication, and the active runtime namespace leaves this method nil. |
| `CombatText_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CompactUnitFrame_GetOptionDispelIndicatorOverlayAnimation` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CompactUnitFrame_GetOptionDispelIndicatorOverlayType` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CompactUnitFrameLayoutTemplates_LayoutFrameElement` | best-effort | Stale proposed addition: all-LoD PTR runtime leaves the exact path nil. |
| `CompactUnitFrameUtil.ApplyConfig` | best-effort | Stale proposed addition: all-LoD PTR runtime leaves the exact path nil. |
| `CompactUnitFrameUtil.GenerateNewConfig` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ConfirmDisenchantRollDialog_Show` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ConfirmLootRollDialog_Show` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ConfirmTalentWipeDialog_Show` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ContributionCollectionFrame_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CooldownManagerLayout_GetGroupBuffVisualAlerts` | best-effort | Delegates to existing C_UnitAuras group-buff preference state; focused 12.1 bridge test. |
| `CooldownManagerLayout_GetHiddenGroupBuffs` | best-effort | Delegates to existing C_UnitAuras group-buff preference state; focused 12.1 bridge test. |
| `CooldownManagerLayout_SetGroupBuffVisualAlerts` | best-effort | Delegates to existing C_UnitAuras group-buff preference state; focused 12.1 bridge test. |
| `CooldownManagerLayout_SetHiddenGroupBuffs` | best-effort | Delegates to existing C_UnitAuras group-buff preference state; focused 12.1 bridge test. |
| `CooldownViewer_MarkAuraCacheDirty` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CooldownViewerContextMenu_AddAlertEntryButton` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CooldownViewerContextMenu_AddNewAlertButton` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CooldownViewerDraggedItem_Clear` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CooldownViewerDraggedItem_Pickup` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CooldownViewerDraggedItem_SetIsLegalTarget` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `CooldownViewerUtil.AddSoundAlertRadio` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `CooldownViewerUtil.BuildSoundMenus` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `CooldownViewerUtil.GetSoundTypeSoundKit` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `CooldownViewerUtil.GetSoundTypeText` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `CovenantCallings_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `DebugTools_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `DifficultyUtil.GetCreatureDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; preserves arguments, both return values, and later hotfix replacement. |
| `DifficultyUtil.GetDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; preserves arguments, both return values, and later hotfix replacement. |
| `DifficultyUtil.GetQuestDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; preserves arguments, both return values, and later hotfix replacement. |
| `DifficultyUtil.GetRelativeDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; focused test covers `(10, 15)`, two-return fidelity, and hot-swap behavior. |
| `DifficultyUtil.GetScalingQuestDifficultyColor` | best-effort | 12.1 post-load dynamic delegate to the authoritative vendor global; preserves arguments, both return values, and later hotfix replacement. |
| `EditModeManagerFrame_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `EncounterJournal_SetTabVisibe` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `EventTrace_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ExpansionTrial_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `FadingFrame_CopyTextScalingTime` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `FadingFrame_GetTextScalingMinHeight` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `FadingFrame_InitSlot` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `FadingFrame_SetTextScaling` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `FadingFrame_StartTextScaling` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `FadingFrame_StopTextScaling` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `FadingFrame_UpdateTextScaling` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `FriendsListUtil.BuildCharacterClassDisplayText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.BuildCharacterLevelDisplayText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.BuildCharacterNameDisplayText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.BuildFriendNameDisplayText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.BuildLocationDisplayText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.BuildTooltipBroadcastText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GameStateUsesFactions` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetBattleNetFriendGameAccountInfoIfExactlyOneDirectInviteTargetExists` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetBattleNetFriendInviteInfo` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetBattleNetFriendInviteTypeLabel` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetBattleNetFriendPartyInviteRestrictionText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetBattleNetFriendPartyInviteRestriction` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetFormattedCharacterName` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetFriendAccountNameText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetFriendNameColorForFriendType` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetFriendNameDisplayColor` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetFriendNameOfflineDisplayColor` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetGameAccountPartyInviteRestriction` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetLastOnlineText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetRegionName` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.GetRelativeTimeText` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.HasMultipleGameAccounts` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.InviteOrRequestToJoin` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.IsPlayingDifferentWoWProject` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.IsPlayingSameWoWProject` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.IsPlayingWoW` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.IsRequestInviteType` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.IsTitleFriend` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `FriendsListUtil.ShouldShowRichPresenceOnly` | best-effort | Stale snapshot addition: recursive PTR proof finds no qualified publication, and `FriendsListUtil` remains nil after startup. |
| `GameMenuFrame_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GameMenuFrame_IsShown` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GameMenuFrame_Show` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GameRulesUtil.IsPlayerAtEffectiveMaxLevel` | best-effort | Stale proposed addition: all-LoD PTR runtime leaves the exact path nil. |
| `GetBottomManagedFrameContainer` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GetChatAdditionalColor` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GetDiscordUserCommunityLink` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GetDiscordUserLink` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GetGarrisonMissionFrameNameForFollowerType` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GetGarrisonTypeForFollowerType` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GetPlayerBottomManagedFrameContainer` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GetRightManagedFrameContainer` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GetTimeSinceLastQuestProgress` | best-effort | Vendor-present in PTRFeedback. Focused proof pins publication and the current upstream nil-arithmetic invocation defect from undefined `lastProgressTime`; no guessed correction is added. |
| `GetTimeStringFromSeconds` | best-effort | Classified as snapshot cross-flavor contamination: the only PTR definition is in `Mists/UIParent.lua`, loaded exclusively by `Blizzard_UIParent_Mists.toc`; PTR tests verify absence during initialization, post-load compatibility, and settled mainline startup. |
| `GetUIPanelLayoutAttribute` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GetUIPanelLayoutFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GMChatFrame_OnWhisperFromGM` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GossipConfirmDialog_Show` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GuildControlDiscord_Loaded_OnEvent` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GuildControlDiscord_Loaded_OnLoad` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GuildControlDiscord_SetGuildSettingsCheckboxes` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GuildControlRankDiscord_OnLoad` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `GuildControlUI_Discord_HideAll` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `GuildControlUI_Discord_Update` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `GuildControlUI_DiscordFrame_OnLoad` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `GuildControlUI_LoadUI` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `GuildControlUI_OnShow` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `GuildControlUI_SetupDiscord` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `GuildControlUI_SetupSelected` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `GuildControlUI_Setup` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `GuildControlUI_Show` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `GuildControlUI_UnlinkDiscord` | best-effort | Stale snapshot addition: PTR source proof finds no occurrence, and the proposed global remains nil after startup. |
| `HandleQuestSessionInviteToPartyConfirmation` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HelpFrame_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HideAuctionHouseFrame` | best-effort | Classified as snapshot/runtime mismatch: no definition exists in the local PTR Blizzard sources; focused PTR runtime test loads `Blizzard_AuctionHouseUI` and verifies the wrapper remains absent rather than inventing behavior. |
| `HideBarberShopFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HideBlackMarketFrame` | best-effort | Classified as reversed-name snapshot mismatch: PTR defines `BlackMarketFrame_Hide` with panel-hide and close-sound behavior, not `HideBlackMarketFrame`; focused PTR test explicitly loads `Blizzard_BlackMarketUI`, verifies the authoritative helper, and confirms the reversed wrapper remains absent. |
| `HideGarrisonMissionFrames` | best-effort | Classified as snapshot/runtime mismatch: no definition exists in local PTR Blizzard Lua sources; focused PTR test loads `Blizzard_GarrisonUI` with its LoD dependencies and verifies the wrapper remains absent. |
| `HideGarrisonShipyardFrame` | best-effort | Classified as snapshot/runtime mismatch: no definition exists in local PTR Blizzard Lua sources; focused PTR test loads `Blizzard_GarrisonUI` with its LoD dependencies and verifies the wrapper remains absent. |
| `HideGossipFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HideGuildBankFrame` | best-effort | Classified as snapshot/runtime mismatch: no definition exists in local PTR Blizzard Lua sources; focused PTR test explicitly loads `Blizzard_GuildBankUI` and verifies the wrapper remains absent. |
| `HideInstanceBootDialog` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HideInstanceLockDialog` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HideItemUpgradeFrame` | best-effort | Classified as vendor-present: the pinned 70493 PTR cache ships `Blizzard_ItemUpgradeUI/Blizzard_ItemUpgradeUI_Bootstrap.lua` (`data/blizzard-ui-files/ptr.txt:2537`), whose line 13 defines `function HideItemUpgradeFrame()` guarding on `C_AddOns.IsAddOnLoaded` and delegating to `ItemUpgradeFrame_Hide`; the focused PTR test loads the full game UI plus the LoD addon, checks the global's debug source, invokes it, and confirms `ItemUpgradeFrame` stays hidden. The earlier reversed-name classification came from a stale local PTR snapshot. |
| `HideProfessionsCustomerOrdersFrame` | best-effort | Classified as snapshot/runtime mismatch: recursive local PTR source scan finds no definition; focused PTR test loads the required ProfessionsTemplates/AuctionHouse dependencies plus `Blizzard_ProfessionsCustomerOrders`, verifies the frame exists, and confirms the wrapper remains absent. |
| `HideSummonConfirmationDialogs` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HouseFinderFrame_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HousingBulletinBoardFrame_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HousingControls_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `HousingFramesUtil.IsBlueprintCollectionAvailable` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `HousingFramesUtil.IsBlueprintOperationInProgress` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `HousingFramesUtil.ShowBlueprintExport` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `HousingFramesUtil.ShowBlueprintImport` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `HousingFramesUtil.ShowBlueprintRoomExport` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `HousingFramesUtil.TryOpenBlueprintCollection` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `HybridMinimap_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `InputBoxInstructions_OnEnter` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `InputBoxInstructions_OnLeave` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `InputBoxInstructions_ShowTooltipIfTruncated` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `InputUtil.CursorOnUpdate` | best-effort | Stale snapshot namespace move: focused PTR proof keeps the authoritative global behavior and verifies `InputUtil.CursorOnUpdate` remains nil. |
| `InputUtil.CursorUpdate` | best-effort | Stale snapshot namespace move: focused PTR proof keeps the authoritative global behavior and verifies `InputUtil.CursorUpdate` remains nil. |
| `InputUtil.GetCursorDelta` | best-effort | Stale snapshot namespace move: focused PTR proof keeps the authoritative global behavior and verifies `InputUtil.GetCursorDelta` remains nil. |
| `InputUtil.IsMouseOver` | best-effort | Stale snapshot namespace move: focused PTR proof keeps the authoritative global behavior and verifies `InputUtil.IsMouseOver` remains nil. |
| `InputUtil.ShowInspectCursor` | best-effort | Stale snapshot namespace move: focused PTR proof keeps the authoritative global behavior and verifies `InputUtil.ShowInspectCursor` remains nil. |
| `InterfaceUtil.GetScreenHeightScale` | best-effort | Stale snapshot: active PTR has no `InterfaceUtil`; global height-scale helper remains authoritative and returns `1.0` at 768 pixels. |
| `InterfaceUtil.GetScreenWidthScale` | best-effort | Stale snapshot: active PTR has no `InterfaceUtil`; global width-scale helper remains authoritative and returns `1.0` at 1024 pixels. |
| `InterpolatorUtil.GetSmoothProgressChange` | best-effort | Reversed snapshot: PTR UIParent exports global `GetSmoothProgressChange`, not this namespace member; focused PTR proof confirms the member remains nil while the global computes the expected value. |
| `IslandsPartyPoseFrame_TryShow` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `IsMouseoverCastSupported` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `IsSummonConfirmationDialogVisible` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `IsTypeAdditionalChatColor` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ItemUtil.DisplayEquipSlotTooltip` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `ItemUtil.GetEmptyEquipSlotTooltipForSlotName` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `ItemUtil.GetEmptyEquipSlotTooltip` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `ItemUtil.GetEquipSlotTexture` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `ItemUtil.GetValidatedItemLocation` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `Kiosk_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `KioskFrame_HandlePlayerEnteringWorld` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `LandingSoulbinds_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `LFGListApplicationViewer_OpenEditMode` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `LFGListApplicationViewerRemoveEntryButton_OnClick` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `LoadAddOnWithErrorHandling` | implemented | 12.1 wrapper in `src/ptr/compat_bootstrap.lua`; focused delegation test in `global_functions.rs`. |
| `LocaleUtil.GetLocaleDisplayName` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `LocalizePlayerFrame_zhCN` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `LocalizePlayerFrame_zhTW` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `LootFrame_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `MacroFrame_SaveMacro` | best-effort | PTR lifecycle proof verifies the eager UIParent no-op placeholder is harmless, then explicit `Blizzard_MacroUI` loading replaces it with the `MacroFrame:SaveMacro()` delegate. |
| `ManageFramePositions` | best-effort | Stale proposed addition: all-LoD PTR runtime leaves the exact path nil. |
| `MenuUtil.CreateHighlightButton` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `MovePad_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `NarrationUtil.CreateNarrationInfo` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.GetCheckboxContext` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.MakeIndexInfo` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.MakeNarrationStringForMoney` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.MakeNarrationStringFromIndexInfo` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.MakeNarrationStringFromInfo` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.MakeNarrationString` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.NarrateCurrentScreen` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.RegionToNarrationInfo` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.ResolveForwardedRegion` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.SetStaticDescription` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.SetStaticName` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.ShouldBeEnabled` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NarrationUtil.ShouldRegionNavigationSkipTooltips` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `NarrationUtil` remains nil after startup. |
| `NPE_InitializeIfLoaded` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `OpacityFrame_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `OpenEncounterJournalToJourney` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `OpenOrderHallTalentUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `OpenPlayerSpellsToGlyphTarget` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `PhotoSharingFrame_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `PingUtil.SendMacroPing` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `PingUtil.TogglePingTarget` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `PlayerChoiceFrame_TryShow` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `PlayerChoiceToggle_TryShow` | best-effort | PTR LoD behavior: absent before `Blizzard_PlayerChoice`, present afterward; focused proof verifies eligible button visibility, explicit plus OnShow updates, and nil return. |
| `PVPUI_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `RaidWarningUtil.AddMessage` | best-effort | Stale proposed addition: all-LoD PTR runtime leaves the exact path nil. |
| `RaidWarningUtil.ClearBossEmotes` | best-effort | Stale proposed addition: all-LoD PTR runtime leaves the exact path nil. |
| `RaidWarningUtil.UpdateCenterScreenAnchors` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `RecentAlliesUtil.GetBestSocialUIPresenceTypeForStateData` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `RecruitAFriendFrameSocialInitializeAADC` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `RegionUtil.GetTopLeftMost` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `RegionUtil.SortByTopLeft` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `RegisterGameMenuEscHandler` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `RegisterPlayerInteraction` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ReportFrame_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ResetDiscordSettings` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `RestoreGMChatFrameSession` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `SetGhostFrameShown` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `SetPlayerInteractionConditions` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `SettingsPanel_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `SetUIPanelLayoutAttribute` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShakeFrameRandom` | best-effort | Stale snapshot: active PTR leaves the legacy global nil and publishes distinct `ScriptAnimationUtil.ShakeFrameRandom` behavior; focused no-op proof returns a cancellation function. |
| `ShakeFrame` | best-effort | Stale snapshot: active PTR leaves the legacy global nil and publishes distinct `ScriptAnimationUtil.ShakeFrame` behavior; focused locked-region proof returns a cancellation function. |
| `ShouldDisplaySpellCooldown` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowAchievementFrameForAchievement` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowAdventureMapFrameForFollowerType` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowArtifactFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowArtifactRelicForgeFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowAuctionHouseFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowBarberShopFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowBlackMarketFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowChallengesKeystoneFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowFlightMapFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowGarrisonCapacitiveDisplayFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowGarrisonMissionFrameForFollowerType` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowGarrisonRecruiterFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowGarrisonShipyardFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowGuildBankFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowHeirloomsJournalToClosestUpgradeablePage` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowInstanceBootDialog` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowInstanceLockDialog` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowItemSocketingFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowItemUpgradeFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowMatchCelebrationPartyPoseFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowPendingPlayerChoiceResponseUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowPerksProgramFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowProfessionEquipmentHelpTip` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowProfessionsCustomerOrdersFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowProfessionsFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowQuestSessionGroupInviteConfirmation` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowQuestSessionGroupInviteReceivedConfirmation` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowRemixArtifactFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowRuneforgeFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowSummonConfirmationDialog` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ShowTaxiMapFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `SimpleCheckout_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `SocialUIContactsFrameInitializeAADC` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `SocialUIUtil.AddSeparatorToTooltip` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.GetBattleNetFriendTagInterestsUIOrder` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.GetBattleNetFriendTagRoleUIOrder` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.GetBlockedName` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.GetIconForPresenceType` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.GetLabelForBattleNetFriendTag` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.GetLabelForPresenceType` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.GetPresenceTypeSelf` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.InitializeUserScaledDropdownButton` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.InitializeUserScaledDropdownMainTitle` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.InitializeUserScaledDropdownTitle` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SocialUIUtil.SetBattleNetPresenceFromSocialUIPresence` | best-effort | Stale snapshot addition: exact-qualified PTR source proof finds no publication, and `SocialUIUtil` remains nil after startup. |
| `SoulbindViewer_LoadUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `SpellFlyout_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `SplashFrame_EscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `StoreEscapePressed` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `StringUtil.JoinAlternatingConditionalColor` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `TextureUtil.AnimateTexCoords` | best-effort | Stale proposed addition: all-LoD PTR runtime leaves the exact path nil. |
| `TimeUtil.BetterDate` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `TimeUtil.GetRecentTimeDate` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `ToggleRAFPanel` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `ToggleSocialUI` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `TryShowAnimaDiversionFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `TryShowCovenantPreviewFrame` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `UIModeUtil.CreateExtendedBlocklist` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `UIModeUtil.CreateModifiedBlocklist` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `UIModeUtil.IsModeActive` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `UIModeUtil.RegisterMode` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `UIModeUtil.SetModeActive` | best-effort | Stale snapshot addition: colon-aware PTR source proof finds no publication, and startup runtime leaves this method nil. |
| `UnitPopupSharedUtil.IsFriendshipUpgrade` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `UpdateQuestAcceptLogFullDialog` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `VisualAlert_GetTypeTemplate` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `VisualAlert_GetTypeText` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `VisualAlertData_ForEach` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `VisualAlerts_RegisterAll` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `WarfrontsPartyPoseFrame_TryShow` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |
| `WowSurveyStatusFrame_OnSurveyDelivered` | best-effort | Stale snapshot addition: bare token is absent from the complete PTR source corpus, and full explicit addon loading leaves the symbol nil. |

### Removed symbols

| Symbol | Status | Reason |
|---|---|---|
| `AddFrameLock` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `AnimatedShine_OnUpdate` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `AnimateTexCoords` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `BattleTagInviteFrame_Show` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `BetterDate` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `BFAMissionFrame_EscapePressed` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `BoostTutorial_AttemptLoad` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `BuildColoredListString` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `BuildIconArray` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `BuildListString` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `BuildMultilineTooltip` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `BuildNewLineListString` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ButtonPulse_OnUpdate` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ClassTrainerFrame_Hide` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ClassTrainerFrame_Show` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `CloseCalendarMenus` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `CommunitiesFrame_IsEnabled` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `CompactUnitFrame_GetOptionDisplayOnlyHealerPowerBars` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `CompactUnitFrame_GetOptionDisplayPowerBar` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `CompactUnitFrame_GetOptionShowDispelIndicatorOverlay` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ConvertRGBtoColorString` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `DisplayTypeUnassignedSupported` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `EventUtil.AreVariablesLoaded` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ExpansionTrial_CheckLoadUI` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `FriendsFrame_CloseQuickJoinHelpTip` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `FriendsFrameAddFriendButton_OnClick` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `FriendsFriends_InitButton` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `FriendsFriends_SetSelection` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `FriendsFriendsButton_SetSelected` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `FriendsFriendsFrame_Close` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `GetDungeonNameWithDifficulty` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `GetNotchHeight` | best-effort | Vendor-present global: focused PTR proof verifies the function remains published and exercises its UI geometry behavior. |
| `GetScaledCursorDelta` | best-effort | Vendor-present global: focused PTR proof verifies the function remains published and exercises its cursor behavior. |
| `GetScaledCursorPositionForFrame` | best-effort | Vendor-present global: focused PTR proof verifies the function remains published and exercises its cursor behavior. |
| `GetScaledCursorPosition` | best-effort | Vendor-present global: focused PTR proof verifies the function remains published and exercises its cursor behavior. |
| `GetScreenHeightScale` | best-effort | Vendor-present despite snapshot removal; focused PTR proof verifies function publication and `1.0` at the 768-pixel fixture height. |
| `GetScreenWidthScale` | best-effort | Vendor-present despite snapshot removal; focused PTR proof verifies function publication and `1.0` at the 1024-pixel fixture width. |
| `GetSmoothProgressChange` | best-effort | Vendor-present despite the snapshot removal: PTR UIParent retains this global function; focused PTR proof verifies representative input `(100, 0, 100, 1)` returns `70`. |
| `GetSocialColoredName` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `GetSortedSelfResurrectOptions` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `GetUIParentOffset` | best-effort | Vendor-present global: focused PTR proof verifies the function remains published and exercises its UI geometry behavior. |
| `HelpPlatesSupported` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `HousingControlsUtil.CanActivateHousingControls` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `IsFrameLockActive` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `IsFrameSmartShown` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `IsLevelAtEffectiveMaxLevel` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `IsPlayerAtEffectiveMaxLevel` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `KeyBindingFrame_LoadUI` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `LocalizePlayerFrame` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `LocalizezhCN` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `LocalizezhTW` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `MacroFrame_SaveMacro` | best-effort | PTR lifecycle proof verifies the eager UIParent no-op placeholder is harmless, then explicit `Blizzard_MacroUI` loading replaces it with the `MacroFrame:SaveMacro()` delegate. |
| `MajorFactions_LoadUI` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `MouseIsOver` | best-effort | Vendor-present global: focused PTR proof verifies the function remains published and exercises its cursor behavior. |
| `NPE_CheckTutorials` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `NPETutorial_AttemptToBegin` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `OpenAchievementFrameToAchievement` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `OrderHallMissionFrame_EscapePressed` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `OrderHallTalentFrame_EscapePressed` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `OutfitterUI_LoadUI` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `PingUtil.GetContextualPingTypeForUnit` | best-effort | Vendor-present colon-defined helper: focused PTR proof verifies GUID/result delegation to `C_Ping` despite the proposed removal. |
| `PlayerChoiceToggle_TryShow` | best-effort | Retained PTR LoD behavior despite snapshot removal: absent before `Blizzard_PlayerChoice`, published afterward with focused button-state proof. |
| `QuickJoin_JoinQueueButtonOnClick` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidBossEmoteFrame_OnEvent` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidBossEmoteFrame_OnLoad` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidBrowser_IsEmpowered` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidNotice_AddMessage` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidNotice_ClearSlot` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidNotice_Clear` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidNotice_FadeInit` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidNotice_OnUpdate` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidNotice_SetSlot` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidNotice_UpdateSlot` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidWarningFrame_OnEvent` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RaidWarningFrame_OnLoad` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RecentTimeDate` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RefreshAuras` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RegisterNewFrameLock` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `RemoveFrameLock` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ReverseQuestObjective` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `SetDesaturation` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `SetFrameLock` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ShowResurrectRequest` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `SmartHide` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `SmartShow` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `TalentFrame_LoadUI` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `TargetFrame_UpdateBuffAnchor` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `TargetFrame_UpdateDebuffAnchor` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ToggleLFGFrame` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ToggleRafPanel` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ToggleRaidBrowser` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `ToggleWoWHackCharacterUI` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `TokenFrame_LoadUI` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `TrialAccountCapReached_Inform` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UIDoFramesIntersect` | best-effort | Vendor-present global: focused PTR proof verifies the function remains published and exercises its UI geometry behavior. |
| `UIParent_ManageFramePositions` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UIParent_OnEvent` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UIParent_OnHide` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UIParent_OnLoad` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UIParent_OnShow` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UIParent_Shared_OnEvent` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UIParent_Shared_OnLoad` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UIParent_UpdateTopFramePositions` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UIParentLoadAddOn` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UnitHasMana` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UpdateFrameLock` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `UpdateUIElementsForClientScene` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `WorldFrame_OnLoad` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `WorldFrame_OnUpdate` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `WoWHackSpellsUI_LoadUI` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `getglobal` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |
| `setglobal` | best-effort | Vendor-present despite proposed removal: all-LoD PTR runtime retains a global function. |

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
- [Final publication matrix](../../../patch-tests/patch_12_1/remaining_observations.rs) — exact all-LoD type contract for seven absent additions and 99 retained removals.

## See Also

- [[patch-api-audit-manifest]] — manifest schema, validation, lifecycle assertions, and checklist generation.
- [[patch-12-1-api-audit]] — patch-level implementation and exception matrix.
- [[client-profiles]] — cumulative retail epoch feature gating.
- [[lua-api]] — runtime Lua surface context.
