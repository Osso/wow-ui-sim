# Retail 12.1.0 page audit

Prepared 2026-10-04. Static source inspection only: no Cargo, tests, builds, simulator, commits, Git-state changes, agents or model CLIs. Only this evidence directory, the new page ledger, and authorized audit scratch were written.

## Capture and identity

Retained source: `data/patch-api/sources/12.1.0-api-changes.txt`; provenance: `data/patch-api/sources/12.1.0-api-changes.provenance.json`. Pageid 679840; retrieval 2026-10-04; MediaWiki query `prop=extracts&explaintext=1`. Raw SHA-256: `c051a68b442b2e69d602650d1c5ec07c1d0262dd1aea01fe73d3f60cd5f29c3b` (matches provenance). The raw file has **396 physical lines, 333 nonblank lines**; the supplied 395-line description was off by one. Every nonblank line has exactly one ledger row. Blank lines have none.

`source_register` points to that exact retained text, not a fabricated register. Coverage schema and proof policy follow the 12.0.7 ledger. Capabilities remain empty; substantive rows remain `audit-pending`, editorial context alone is `metadata-only`. Triage is a plan, not acceptance evidence. No prior occurrence status was copied.

## Completeness finding: extract is NOT the full API inventory

First operation fetched `https://warcraft.wiki.gg/api.php?action=parse&pageid=679840&prop=wikitext&format=json` with curl, User-Agent `wow-ui-sim-audit/1.0`. Response and decoded 1,580-line wikitext are in `/home/osso-test/.cache/wow-ui-sim-audit/12.1.0-session-2026-10-04/page-parse.json` and `page.wikitext`. This is a fresh page parse, not an old-revision capture; the original provenance provides no revision ID, so same-revision identity cannot be claimed.

Six consolidated **inline `mw-collapsible` wikitable** inventories were dropped by extracts: Global API, FrameXML, ScriptObjects, Widgets, Events and CVars. These lists are written directly in this page's wikitext; **no list transclusion target exists to fetch**. `api`, `tlygo`, `apitooltip` and styling templates format symbol references; they are not external inventory sources. Enumerations and Structures retain their visible symbol/annotation lists in the extract; their linked declarations and native semantics were never captured.

Page table headers report Global API 145 added/19 removed; FrameXML 337 added/124 removed; ScriptObjects 8 added/0 removed; Widgets 45 added/0 removed; Events 43 added/2 removed; CVars 23 added/5 removed. Global API, Widgets and Events also contain changed-signature/payload blocks. Exact omitted table entries and annotations follow below; header counts are reported as page assertions, not silently substituted for entry counts.

Other omissions: the initial June 18 `Bluepost` template's announcement (aura-security rationale, customization/new-API rollout and author outreach); syntax-highlighted examples (aura container/button creation, radial masking, nested/local mixins); the spell list/table after the July 21 healer-buff/HoT classification paragraph; deprecated-wrapper implementation code blocks. Resources lose link targets and previous/next-patch navigation. These omissions must not be treated as audited by the 333-row ledger.

A fuller capture needs revision-pinned raw wikitext plus `action=parse&prop=text` rendered HTML, retaining collapsed tables, template bodies, code examples and link targets; save each byte hash/revision ID in provenance. Expand into a new register with explicit directions, changed annotations and per-source-location identities, reconcile prior 12.1 registers against actual retail declarations, then extend coverage deliberately. Do **not** edit the retained source or invent omitted rows in this ledger. Cached retail docs are current cache evidence, not an authenticated 69587 build. Existing PTR manifest/registers are cross-references only, not retail proof.

## Retained-source counts by section

- Resources: 4
- Notes: 9
- Blue posts: 193
- Global API: 2
- FrameXML: 2
- ScriptObjects: 1
- Widgets: 1
- Events: 1
- CVars: 1
- Enumerations: 59
- Structures: 47
- Deprecated API: 13

## Triage and batch counts

- metadata-only: 64
- implemented-needs-proof: 92
- modelable: 161
- blocked: 16
- inherits-existing-capability: 0

**73 batches**: 26 tests-only first, 47 modelable next. Blocked rows are listed last with exact ledger notes; none receives a design. See [triage](triage.md) and [batches](batches.md).

## Exact omitted consolidated table entries

Supplemental completeness evidence only, NOT source rows or capabilities. `W` is one-based raw-wikitext line. Entries below preserve all inline symbol lists and changed annotations. Page header counts differ from actual Global API additions: 147 entries vs header 145; no silent normalization.

### Global API — added

- W414: `C_AuraContainerUtil.ProcessAuraTooltipBackdropOptions`
- W415: `C_AuraContainerUtil.ProcessAuraTooltipNineSliceOptions`
- W416: `C_AuraContainerUtil.ProcessAuraTooltipTextureSliceOptions`
- W417: `C_AuraContainerUtil.ProcessCustomAuraButtonApplicationBarOptions`
- W418: `C_AuraContainerUtil.ProcessCustomAuraButtonApplicationCountOptions`
- W419: `C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextOptions`
- W420: `C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextureOptions`
- W421: `C_AuraContainerUtil.ProcessCustomAuraButtonDurationBarOptions`
- W422: `C_AuraContainerUtil.ProcessCustomAuraButtonDurationTextOptions`
- W423: `C_BattleNet.AreFriendTagsEnabled`
- W424: `C_BattleNet.AreTitleFriendCustomNamesEnabled`
- W425: `C_BattleNet.AreTitleFriendsEnabled`
- W426: `C_BattleNet.BNCheckTitleFriendInviteToUnit`
- W427: `C_BattleNet.CanToggleHighResTexturesWithoutClientReload`
- W428: `C_BattleNet.GetCustomTitleFriendName`
- W429: `C_BattleNet.GetFriendInviteInfo`
- W430: `C_BattleNet.IsBattleNetFriendsListEnabled`
- W431: `C_BattleNet.IsBattleNetFriendsListSupported`
- W432: `C_BattleNet.SearchFriends`
- W433: `C_BattleNet.SendTitleFriendInviteByName`
- W434: `C_BattleNet.SendVerifiedBattleNetFriendInvite`
- W435: `C_BattleNet.SetAppearOffline`
- W436: `C_BattleNet.SetCustomTitleFriendName`
- W437: `C_BattleNet.SetFriendTags`
- W438: `C_Browser.CloseFullscreenBrowser`
- W439: `C_CVar.AreCVarsLoaded`
- W440: `C_ClientScene.IsSceneTypeActive`
- W441: `C_Club.SendTitleFriendRequest`
- W442: `C_CooldownViewer.GetGroupBuffItems`
- W443: `C_DelvesUI.GetFlavorNodeForCompanion`
- W444: `C_DelvesUI.GetFlavorNodeNameForCompanion`
- W445: `C_DelvesUI.HasActiveLFGLair`
- W446: `C_DelvesUI.HasActiveLair`
- W447: `C_DelvesUI.IsInLair`
- W448: `C_Discord.Authorize`
- W449: `C_Discord.GetDiscordChannelName`
- W450: `C_Discord.GetDiscordUserID`
- W451: `C_Discord.GetDiscordUserName`
- W452: `C_Discord.GetDisplayNameType`
- W453: `C_Discord.GetGuildLinkStatus`
- W454: `C_Discord.GetNumDiscordChannels`
- W455: `C_Discord.GetNumDiscordServers`
- W456: `C_Discord.GetServerLinkableChannels`
- W457: `C_Discord.GetServerName`
- W458: `C_Discord.GuildLink`
- W459: `C_Discord.GuildUnlink`
- W460: `C_Discord.IsEnabled`
- W461: `C_Discord.IsGuildChannelLinked`
- W462: `C_Discord.IsGuildSettingSet`
- W463: `C_Discord.IsUserOAuthed`
- W464: `C_Discord.RefreshAuth`
- W465: `C_Discord.SetGuildSetting`
- W466: `C_Discord.UpdateDiscordServers`
- W467: `C_Discord.UpdateGuildLobby`
- W468: `C_DyeColor.GetDyeColorsForItemLocation`
- W469: `C_DyeColor.GetDyeColorsForItem`
- W470: `C_EncounterJournal.GetBaseDifficultyID`
- W471: `C_EncounterJournal.InstanceHasDifficultyID`
- W472: `C_FriendList.IsLegacyFriendSystemEnabled`
- W473: `C_GuildInfo.IsDiscordStreamSeparate`
- W474: `C_HouseEditor.GetHouseEditorPlayerType`
- W475: `C_Housing.HouseFinderIgnoreNeighborhood`
- W476: `C_Housing.IsInsideOwnedHouseOrPlot`
- W477: `C_Housing.IsInsideOwnedHouse`
- W478: `C_Housing.IsInsideOwnedPlot`
- W479: `C_Housing.ResetHouse`
- W480: `C_HousingBlueprint.CanExportRoom`
- W481: `C_HousingBlueprint.CanExportTypeFromCurrentLocation`
- W482: `C_HousingBlueprint.CanImportTypeFromCurrentLocation`
- W483: `C_HousingBlueprint.DeleteBlueprint`
- W484: `C_HousingBlueprint.ExportBlueprint`
- W485: `C_HousingBlueprint.ExportRoomBlueprint`
- W486: `C_HousingBlueprint.GetBlueprintHyperlink`
- W487: `C_HousingBlueprint.GetBlueprintTypeForCode`
- W488: `C_HousingBlueprint.GetExportAvailability`
- W489: `C_HousingBlueprint.GetFeatureAvailability`
- W490: `C_HousingBlueprint.GetImportAvailability`
- W491: `C_HousingBlueprint.ImportBlueprint`
- W492: `C_HousingBlueprint.IsShareCodeValid`
- W493: `C_HousingBlueprint.RenameBlueprint`
- W494: `C_HousingBlueprint.RequestBlueprintCollection`
- W495: `C_HousingBlueprint.RequestBlueprintContentsForContext`
- W496: `C_HousingBlueprint.RequestBlueprintContents`
- W497: `C_HousingBlueprint.StartImportRoomBlueprint`
- W498: `C_HousingBlueprint.UpdateBlueprintStringFromInput`
- W499: `C_HousingCustomizeMode.ApplyPetToSelectedDecor`
- W500: `C_HousingCustomizeMode.GetSelectedDecorPetInfo`
- W501: `C_HousingDecor.AnyDecorPlacedInRoom`
- W502: `C_HousingDecor.GetAllMaxPlacementBudgets`
- W503: `C_HousingDecor.GetAllSpentPlacementBudgets`
- W504: `C_HousingDecor.GetDecorAssignedPetName`
- W505: `C_HousingDecor.GetDecorCanAttachPet`
- W506: `C_HousingDecor.GetMaxPetPlacementBudget`
- W507: `C_HousingDecor.GetSpentPetPlacementBudget`
- W508: `C_HousingLayout.GetBaseRoomFloor`
- W509: `C_HousingLayout.GetHighestOccupiedFloorIndex`
- W510: `C_HousingLayout.GetLowestOccupiedFloorIndex`
- W511: `C_HousingLayout.GetRoomPlayerIsIn`
- W512: `C_HousingLayout.GetSelectedBlueprintFloorplan`
- W513: `C_HousingLayout.HasSelectedBlueprintFloorplan`
- W514: `C_HousingLayout.RoomHasStairs`
- W515: `C_Item.DoesItemMatchSpellItemCondition`
- W516: `C_LFGInfo.IsInMatchmadeRaidWithoutRoleRequirements`
- W517: `C_LFGList.ConfirmCensoredActiveEntry`
- W518: `C_LFGList.DoesCensoredTextMatch`
- W519: `C_LFGList.IsCensoredActiveEntryUnresolved`
- W520: `C_LFGList.RevealCensoredActiveEntry`
- W521: `C_LFGList.RevealCensoredSearchResult`
- W522: `C_Navigation.GetNextWaypointForMap`
- W523: `C_NeighborhoodInitiative.GetInitiativeTaskRewardScaling`
- W524: `C_PaperDollInfo.CancelTemporaryEnchantment`
- W525: `C_PaperDollInfo.GetInventorySlotInfoForInvSlot`
- W526: `C_PaperDollInfo.GetInventorySlotInfo`
- W527: `C_PaperDollInfo.GetTemporaryEnchantmentInfo`
- W528: `C_PetJournal.GetPetInfoTableBySpeciesID`
- W529: `C_PvP.CanSurrenderArena`
- W530: `C_PvP.JoinRandomTrainingGroundArena`
- W531: `C_PvP.JoinRandomTrainingGroundBattleground`
- W532: `C_QuestHub.IsAreaPOICurrentlyRelatedToHub`
- W533: `C_RecentAllies.SearchRecentAllies`
- W534: `C_RecruitAFriend.IsSystemEnabled`
- W535: `C_RecruitAFriend.IsSystemSupported`
- W536: `C_Roleset.ApplyRolesetFilters`
- W537: `C_Roleset.GetActiveAllowedRolesets`
- W538: `C_Roleset.GetActiveBlockedRolesets`
- W539: `C_SocialQueue.IsSystemEnabled`
- W540: `C_SocialQueue.IsSystemSupported`
- W541: `C_SocialRestrictions.IsFriendsDisabled`
- W542: `C_SocialUI.IsSystemEnabled`
- W543: `C_Sound.PlaySoundWithOptions`
- W544: `C_SpecializationInfo.GetInspectSpecialization`
- W545: `C_Spell.GetLastCategoryCooldownSource`
- W546: `C_Spell.GetSpellDescriptionForItemLocation`
- W547: `C_Spell.TargetSpellChecksItemCondition`
- W548: `C_TransmogOutfitInfo.CanPlayerTransmogSlot`
- W549: `C_TransmogOutfitInfo.IsTransmogEnabled`
- W550: `C_UnitAuras.AddAuraSound`
- W551: `C_UnitAuras.CancelAuraByInstanceID`
- W552: `C_UnitAuras.GetGroupBuffVisualAlerts`
- W553: `C_UnitAuras.GetHiddenGroupBuffs`
- W554: `C_UnitAuras.RemoveAuraSound`
- W555: `C_UnitAuras.SetGroupBuffVisualAlerts`
- W556: `C_UnitAuras.SetHiddenGroupBuffs`
- W557: `GetSpecializationSystem`
- W558: `UnitIsPlayerControlledOrGroupMember`
- W559: `securecopy`
- W560: `settablesecurity`

### Global API — removed

- W563: `BNGetFriendInviteInfo`
- W564: `BNSendVerifiedBattleTagInvite`
- W565: `C_DyeColor.GetDyeColorForItemLocation`
- W566: `C_DyeColor.GetDyeColorForItem`
- W567: `C_Housing.IsInsideOwnHouse`
- W568: `C_HousingLayout.GetNumFloors`
- W569: `C_Ping.GetContextualPingTypeForUnit`
- W570: `C_PvP.JoinRandomTrainingGround`
- W571: `C_RecruitAFriend.IsEnabled`
- W572: `C_SuperTrack.GetNextWaypointForMap`
- W573: `C_UnitAuras.AddPrivateAuraAppliedSound`
- W574: `C_UnitAuras.RemovePrivateAuraAppliedSound`
- W575: `C_UnitAuras.TriggerPrivateAuraShowDispelType`
- W576: `CanSurrenderArena`
- W577: `CancelItemTempEnchantment`
- W578: `GetInspectSpecialization`
- W579: `GetInventorySlotInfo`
- W580: `GetWeaponEnchantInfo`
- W581: `SetTableSecurityOption`

### Global API — changed

- W584: `C_ActionBar.ForceUpdateAction`
- W585: `+ arg2 = suppressEvents`
- W586: `C_CombatAudioAlert.SpeakText`
- W587: `+ ret1 = utteranceID`
- W588: `+ MayReturnNothing`
- W589: `C_HousingDecor.GetMaxPlacementBudget`
- W590: `# ret1.Nilable false -> true`
- W591: `C_HousingDecor.GetSpentPlacementBudget`
- W592: `# ret1.Nilable false -> true`
- W593: `C_HousingLayout.GetRoomPlacementBudget`
- W594: `# ret1.Nilable false -> true`
- W595: `C_HousingLayout.GetSpentPlacementBudget`
- W596: `# ret1.Nilable false -> true`
- W597: `C_Ping.SendMacroPing`
- W598: `# arg1.Name type -> macroInfo`
- W599: `# arg1.Nilable true -> false`
- W600: `# arg1.Type PingSubjectType -> PingMacroInfo`
- W601: `- arg2 = targetToken`
- W602: `C_QuestHub.IsQuestCurrentlyRelatedToHub`
- W603: `# arg2.Name areaPoiID -> hubAreaPoiID`
- W604: `C_RecruitAFriend.CanSummonFriend`
- W605: `# ret1.Name result -> canSummon`
- W606: `+ ret2 = reason`
- W607: `+ MayReturnNothing`
- W608: `C_Sound.PlaySound`
- W609: `+ arg6 = volumeOverride`
- W610: `C_Spell.GetSpellTexture`
- W611: `+ ret3 = conditionalIconID`
- W612: `CreateSecureDelegate`
- W613: `+ arg2 = options`

### FrameXML — added

- W624: `AchievementFrame_RefreshBackButton`
- W625: `AchievementFrame_SetComparisonMode`
- W626: `AddBehavioralMessagingTrayToStatusFrames`
- W627: `AddFriendFrame_Show`
- W628: `AddGMChatStatusFrameToStatusFrames`
- W629: `AddTicketStatusFrameToStatusFrames`
- W630: `AddWowSurveyStatusFrameToStatusFrames`
- W631: `AlliedRacesFrame_TryShow`
- W632: `AnchorUtil.ApplyFlowLayout`
- W633: `AnchorUtil.CreateFlowLayout`
- W634: `ApplySecureDelegatesToTable`
- W635: `ArchaeologyFrame_ToggleUI`
- W636: `ArcheologyDigsiteProgressBar_OnSurveyCast`
- W637: `ArdenwealdGardening_LoadUI`
- W638: `ArtifactFrame_OnTraitsRefunded`
- W639: `AuraUtil.AuraInstanceIDOnlyAuraCompare`
- W640: `AuraUtil.ExpirationAuraCompare`
- W641: `AuraUtil.ExpirationOnlyAuraCompare`
- W642: `AuraUtil.GetAuraBorderColor`
- W643: `AuraUtil.GetAuraDispelTypeIcon`
- W644: `AuraUtil.GetUnitAuras`
- W645: `AuraUtil.ImportantOnlyAuraCompare`
- W646: `AuraUtil.IsValidFilterString`
- W647: `AuraUtil.NameAuraCompare`
- W648: `AuraUtil.NameOnlyAuraCompare`
- W649: `AuraUtil.SetAuraDispelTypeIcon`
- W650: `AzeriteEmpoweredItemUI_LoadUI`
- W651: `AzeriteEssenceUI_LoadUI`
- W652: `BattlefieldMap_ToggleUI`
- W653: `BehavioralMessaging_LoadUI`
- W654: `BehavioralMessagingTray_OnNotification`
- W655: `Blizzard_HousingCatalogUtil.AddDecorEntryTooltipTrackingText`
- W656: `Blizzard_HousingCatalogUtil.TrackHousingDecorID`
- W657: `BNet_GetBattleTagComponents`
- W658: `BNet_GetBattleTagSelf`
- W659: `BNet_GetBroadcastTextSelf`
- W660: `BNet_GetFriendLevelRank`
- W661: `BNet_IsFriendLevelEqualOrHigher`
- W662: `BoostTutorial_LoadUI`
- W663: `CDMDebugGetDebugger`
- W664: `ChallengeModeCompleteBanner_OnChallengeModeCompleted`
- W665: `ChatAdditionalColor_OpenColorPicker`
- W666: `ChatFrameUtil.DiscordNameColorize`
- W667: `ChatFrameUtil.FormatDiscordMessage`
- W668: `ChatFrameUtil.GetNameForDiscordMessage`
- W669: `CheckActiveStoreForFree`
- W670: `CombatAudioAlertUtil.EnumerateInterruptCastInfo`
- W671: `CombatAudioAlertUtil.EnumerateInterruptCastSuccessInfo`
- W672: `CombatAudioAlertUtil.EnumerateSayCombatEndInfo`
- W673: `CombatAudioAlertUtil.EnumerateSayCombatStartInfo`
- W674: `CombatAudioAlertUtil.EnumeratetWhenTargetDiesInfo`
- W675: `CombatAudioAlertUtil.GetInterruptCastInfo`
- W676: `CombatAudioAlertUtil.GetInterruptCastSuccessInfo`
- W677: `CombatAudioAlertUtil.GetSayCombatEndInfo`
- W678: `CombatAudioAlertUtil.GetSayCombatStartInfo`
- W679: `CombatAudioAlertUtil.GetWhenTargetDiesInfo`
- W680: `CombatText_LoadUI`
- W681: `CompactUnitFrame_GetOptionDispelIndicatorOverlayAnimation`
- W682: `CompactUnitFrame_GetOptionDispelIndicatorOverlayType`
- W683: `CompactUnitFrameLayoutTemplates_LayoutFrameElement`
- W684: `CompactUnitFrameUtil.ApplyConfig`
- W685: `CompactUnitFrameUtil.GenerateNewConfig`
- W686: `ConfirmDisenchantRollDialog_Show`
- W687: `ConfirmLootRollDialog_Show`
- W688: `ConfirmTalentWipeDialog_Show`
- W689: `ContributionCollectionFrame_LoadUI`
- W690: `CooldownManagerLayout_GetGroupBuffVisualAlerts`
- W691: `CooldownManagerLayout_GetHiddenGroupBuffs`
- W692: `CooldownManagerLayout_SetGroupBuffVisualAlerts`
- W693: `CooldownManagerLayout_SetHiddenGroupBuffs`
- W694: `CooldownViewer_MarkAuraCacheDirty`
- W695: `CooldownViewerContextMenu_AddAlertEntryButton`
- W696: `CooldownViewerContextMenu_AddNewAlertButton`
- W697: `CooldownViewerDraggedItem_Clear`
- W698: `CooldownViewerDraggedItem_Pickup`
- W699: `CooldownViewerDraggedItem_SetIsLegalTarget`
- W700: `CooldownViewerUtil.AddSoundAlertRadio`
- W701: `CooldownViewerUtil.BuildSoundMenus`
- W702: `CooldownViewerUtil.GetSoundTypeSoundKit`
- W703: `CooldownViewerUtil.GetSoundTypeText`
- W704: `CovenantCallings_LoadUI`
- W705: `DebugTools_LoadUI`
- W706: `DifficultyUtil.GetCreatureDifficultyColor`
- W707: `DifficultyUtil.GetDifficultyColor`
- W708: `DifficultyUtil.GetQuestDifficultyColor`
- W709: `DifficultyUtil.GetRelativeDifficultyColor`
- W710: `DifficultyUtil.GetScalingQuestDifficultyColor`
- W711: `EditModeManagerFrame_EscapePressed`
- W712: `EncounterJournal_OpenToTieredEntrance`
- W713: `EventTrace_LoadUI`
- W714: `ExpansionTrial_LoadUI`
- W715: `FadingFrame_CopyTextScalingTime`
- W716: `FadingFrame_GetTextScalingMinHeight`
- W717: `FadingFrame_InitSlot`
- W718: `FadingFrame_SetTextScaling`
- W719: `FadingFrame_StartTextScaling`
- W720: `FadingFrame_StopTextScaling`
- W721: `FadingFrame_UpdateTextScaling`
- W722: `FriendsFrame_HideAllPotentialSubFrames`
- W723: `FriendsListUtil.BuildCharacterClassDisplayText`
- W724: `FriendsListUtil.BuildCharacterLevelDisplayText`
- W725: `FriendsListUtil.BuildCharacterNameDisplayText`
- W726: `FriendsListUtil.BuildFriendNameDisplayText`
- W727: `FriendsListUtil.BuildLocationDisplayText`
- W728: `FriendsListUtil.BuildTooltipBroadcastText`
- W729: `FriendsListUtil.GameStateUsesFactions`
- W730: `FriendsListUtil.GetBattleNetFriendGameAccountInfoIfExactlyOneDirectInviteTargetExists`
- W731: `FriendsListUtil.GetBattleNetFriendInviteInfo`
- W732: `FriendsListUtil.GetBattleNetFriendInviteTypeLabel`
- W733: `FriendsListUtil.GetBattleNetFriendPartyInviteRestrictionText`
- W734: `FriendsListUtil.GetBattleNetFriendPartyInviteRestriction`
- W735: `FriendsListUtil.GetFormattedCharacterName`
- W736: `FriendsListUtil.GetFriendAccountNameText`
- W737: `FriendsListUtil.GetFriendNameColorForFriendType`
- W738: `FriendsListUtil.GetFriendNameDisplayColor`
- W739: `FriendsListUtil.GetFriendNameOfflineDisplayColor`
- W740: `FriendsListUtil.GetGameAccountPartyInviteRestriction`
- W741: `FriendsListUtil.GetLastOnlineText`
- W742: `FriendsListUtil.GetRegionName`
- W743: `FriendsListUtil.GetRelativeTimeText`
- W744: `FriendsListUtil.HasMultipleGameAccounts`
- W745: `FriendsListUtil.InviteOrRequestToJoin`
- W746: `FriendsListUtil.IsPlayingDifferentWoWProject`
- W747: `FriendsListUtil.IsPlayingSameWoWProject`
- W748: `FriendsListUtil.IsPlayingWoW`
- W749: `FriendsListUtil.IsRequestInviteType`
- W750: `FriendsListUtil.IsTitleFriend`
- W751: `FriendsListUtil.ShouldShowRichPresenceOnly`
- W752: `GameMenuFrame_EscapePressed`
- W753: `GameMenuFrame_IsShown`
- W754: `GameMenuFrame_Show`
- W755: `GameRulesUtil.IsPlayerAtEffectiveMaxLevel`
- W756: `GetBottomManagedFrameContainer`
- W757: `GetChatAdditionalColor`
- W758: `GetDiscordUserCommunityLink`
- W759: `GetDiscordUserLink`
- W760: `GetGarrisonMissionFrameNameForFollowerType`
- W761: `GetGarrisonTypeForFollowerType`
- W762: `GetPlayerBottomManagedFrameContainer`
- W763: `GetRightManagedFrameContainer`
- W764: `GetTimeStringFromSeconds`
- W765: `GetUIPanelLayoutAttribute`
- W766: `GetUIPanelLayoutFrame`
- W767: `GMChatFrame_OnWhisperFromGM`
- W768: `GossipConfirmDialog_Show`
- W769: `GuildControlDiscord_Loaded_OnEvent`
- W770: `GuildControlDiscord_Loaded_OnLoad`
- W771: `GuildControlDiscord_SetGuildSettingsCheckboxes`
- W772: `GuildControlRankDiscord_OnLoad`
- W773: `GuildControlUI_Discord_HideAll`
- W774: `GuildControlUI_Discord_Update`
- W775: `GuildControlUI_DiscordFrame_OnLoad`
- W776: `GuildControlUI_LoadUI`
- W777: `GuildControlUI_OnShow`
- W778: `GuildControlUI_SetupDiscord`
- W779: `GuildControlUI_SetupSelected`
- W780: `GuildControlUI_Setup`
- W781: `GuildControlUI_Show`
- W782: `GuildControlUI_UnlinkDiscord`
- W783: `HandleQuestSessionInviteToPartyConfirmation`
- W784: `HelpFrame_EscapePressed`
- W785: `HideAuctionHouseFrame`
- W786: `HideBarberShopFrame`
- W787: `HideBlackMarketFrame`
- W788: `HideGarrisonMissionFrames`
- W789: `HideGarrisonShipyardFrame`
- W790: `HideGossipFrame`
- W791: `HideGuildBankFrame`
- W792: `HideInstanceBootDialog`
- W793: `HideInstanceLockDialog`
- W794: `HideItemUpgradeFrame`
- W795: `HideProfessionsCustomerOrdersFrame`
- W796: `HideSummonConfirmationDialogs`
- W797: `HouseFinderFrame_LoadUI`
- W798: `HousingBulletinBoardFrame_LoadUI`
- W799: `HousingControls_LoadUI`
- W800: `HousingFramesUtil.IsBlueprintCollectionAvailable`
- W801: `HousingFramesUtil.IsBlueprintOperationInProgress`
- W802: `HousingFramesUtil.ShowBlueprintExport`
- W803: `HousingFramesUtil.ShowBlueprintImport`
- W804: `HousingFramesUtil.ShowBlueprintRoomExport`
- W805: `HousingFramesUtil.TryOpenBlueprintCollection`
- W806: `HousingTutorialUtil.HousingDecorQuestTutorialComplete`
- W807: `HybridMinimap_LoadUI`
- W808: `InputBoxInstructions_OnEnter`
- W809: `InputBoxInstructions_OnLeave`
- W810: `InputBoxInstructions_ShowTooltipIfTruncated`
- W811: `InputUtil.CursorOnUpdate`
- W812: `InputUtil.CursorUpdate`
- W813: `InputUtil.GetCursorDelta`
- W814: `InputUtil.IsMouseOver`
- W815: `InputUtil.ShowInspectCursor`
- W816: `InterfaceUtil.GetScreenHeightScale`
- W817: `InterfaceUtil.GetScreenWidthScale`
- W818: `InterpolatorUtil.GetSmoothProgressChange`
- W819: `IslandsPartyPoseFrame_TryShow`
- W820: `IsMouseoverCastSupported`
- W821: `IsSummonConfirmationDialogVisible`
- W822: `IsTypeAdditionalChatColor`
- W823: `IsUnseenNewSettingInCurrentVersion`
- W824: `ItemUtil.DisplayEquipSlotTooltip`
- W825: `ItemUtil.GetEmptyEquipSlotTooltipForSlotName`
- W826: `ItemUtil.GetEmptyEquipSlotTooltip`
- W827: `ItemUtil.GetEquipSlotTexture`
- W828: `ItemUtil.GetValidatedItemLocation`
- W829: `Kiosk_LoadUI`
- W830: `KioskFrame_HandlePlayerEnteringWorld`
- W831: `LandingSoulbinds_LoadUI`
- W832: `LFGListApplicationViewer_OpenEditMode`
- W833: `LFGListApplicationViewerRemoveEntryButton_OnClick`
- W834: `LoadAddOnWithErrorHandling`
- W835: `LocaleUtil.GetLocaleDisplayName`
- W836: `LocalizePlayerFrame_zhCN`
- W837: `LocalizePlayerFrame_zhTW`
- W838: `LootFrame_EscapePressed`
- W839: `MacroFrame_SaveMacro`
- W840: `ManageFramePositions`
- W841: `MenuUtil.CreateHighlightButton`
- W842: `MovePad_LoadUI`
- W843: `NarrationUtil.CreateNarrationInfo`
- W844: `NarrationUtil.GetCheckboxContext`
- W845: `NarrationUtil.MakeIndexInfo`
- W846: `NarrationUtil.MakeNarrationStringForMoney`
- W847: `NarrationUtil.MakeNarrationStringFromIndexInfo`
- W848: `NarrationUtil.MakeNarrationStringFromInfo`
- W849: `NarrationUtil.MakeNarrationString`
- W850: `NarrationUtil.NarrateCurrentScreen`
- W851: `NarrationUtil.RegionToNarrationInfo`
- W852: `NarrationUtil.ResolveForwardedRegion`
- W853: `NarrationUtil.SetStaticDescription`
- W854: `NarrationUtil.SetStaticName`
- W855: `NarrationUtil.ShouldBeEnabled`
- W856: `NarrationUtil.ShouldRegionNavigationSkipTooltips`
- W857: `NPE_InitializeIfLoaded`
- W858: `OpacityFrame_EscapePressed`
- W859: `OpenEncounterJournalToJourney`
- W860: `OpenEncounterJournalToTieredEntrance`
- W861: `OpenMapToUserWaypoint`
- W862: `OpenOrderHallTalentUI`
- W863: `OpenPlayerSpellsToGlyphTarget`
- W864: `PhotoSharingFrame_EscapePressed`
- W865: `PingUtil.SendMacroPing`
- W866: `PingUtil.TogglePingTarget`
- W867: `PlayerChoiceFrame_TryShow`
- W868: `PlayerChoiceToggle_TryShow`
- W869: `PVPUI_LoadUI`
- W870: `RaidWarningUtil.AddMessage`
- W871: `RaidWarningUtil.ClearBossEmotes`
- W872: `RaidWarningUtil.UpdateCenterScreenAnchors`
- W873: `RecentAlliesUtil.GetBestSocialUIPresenceTypeForStateData`
- W874: `RecruitAFriendFrameSocialViewInitializeAADC`
- W875: `RegionUtil.GetTopLeftMost`
- W876: `RegionUtil.SortByTopLeft`
- W877: `RegisterGameMenuEscHandler`
- W878: `RegisterPlayerInteraction`
- W879: `ReportFrame_EscapePressed`
- W880: `ResetDiscordSettings`
- W881: `RestoreGMChatFrameSession`
- W882: `SetGhostFrameShown`
- W883: `SetPlayerInteractionConditions`
- W884: `SettingsPanel_EscapePressed`
- W885: `SetUIPanelLayoutAttribute`
- W886: `ShakeFrameRandom`
- W887: `ShakeFrame`
- W888: `ShouldDisplaySpellCooldown`
- W889: `ShowAchievementFrameForAchievement`
- W890: `ShowAdventureMapFrameForFollowerType`
- W891: `ShowArtifactFrame`
- W892: `ShowArtifactRelicForgeFrame`
- W893: `ShowAuctionHouseFrame`
- W894: `ShowBarberShopFrame`
- W895: `ShowBlackMarketFrame`
- W896: `ShowChallengesKeystoneFrame`
- W897: `ShowFlightMapFrame`
- W898: `ShowGarrisonCapacitiveDisplayFrame`
- W899: `ShowGarrisonMissionFrameForFollowerType`
- W900: `ShowGarrisonRecruiterFrame`
- W901: `ShowGarrisonShipyardFrame`
- W902: `ShowGuildBankFrame`
- W903: `ShowHeirloomsJournalToClosestUpgradeablePage`
- W904: `ShowInstanceBootDialog`
- W905: `ShowInstanceLockDialog`
- W906: `ShowItemSocketingFrame`
- W907: `ShowItemUpgradeFrame`
- W908: `ShowMatchCelebrationPartyPoseFrame`
- W909: `ShowPendingPlayerChoiceResponseUI`
- W910: `ShowPerksProgramFrame`
- W911: `ShowProfessionEquipmentHelpTip`
- W912: `ShowProfessionsCustomerOrdersFrame`
- W913: `ShowProfessionsFrame`
- W914: `ShowQuestSessionGroupInviteConfirmation`
- W915: `ShowQuestSessionGroupInviteReceivedConfirmation`
- W916: `ShowRemixArtifactFrame`
- W917: `ShowRuneforgeFrame`
- W918: `ShowSummonConfirmationDialog`
- W919: `ShowTaxiMapFrame`
- W920: `SimpleCheckout_EscapePressed`
- W921: `SocialUIContactsFrameInitializeAADC`
- W922: `SocialUIUtil.AddSeparatorToTooltip`
- W923: `SocialUIUtil.GetBattleNetFriendTagInterestsUIOrder`
- W924: `SocialUIUtil.GetBattleNetFriendTagRoleUIOrder`
- W925: `SocialUIUtil.GetBlockedName`
- W926: `SocialUIUtil.GetIconForPresenceType`
- W927: `SocialUIUtil.GetLabelForBattleNetFriendTag`
- W928: `SocialUIUtil.GetLabelForPresenceType`
- W929: `SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo`
- W930: `SocialUIUtil.GetPresenceTypeSelf`
- W931: `SocialUIUtil.InitializeUserScaledDropdownButton`
- W932: `SocialUIUtil.InitializeUserScaledDropdownMainTitle`
- W933: `SocialUIUtil.InitializeUserScaledDropdownTitle`
- W934: `SocialUIUtil.SetBattleNetPresenceFromSocialUIPresence`
- W935: `SoulbindViewer_LoadUI`
- W936: `SpellFlyout_EscapePressed`
- W937: `SplashFrame_EscapePressed`
- W938: `StoreEscapePressed`
- W939: `StringUtil.JoinAlternatingConditionalColor`
- W940: `TextureUtil.AnimateTexCoords`
- W941: `TimeUtil.BetterDate`
- W942: `TimeUtil.GetRecentTimeDate`
- W943: `ToggleRAFPanel`
- W944: `ToggleSocialUI`
- W945: `TryShowAnimaDiversionFrame`
- W946: `TryShowCovenantPreviewFrame`
- W947: `UIModeUtil.CreateExtendedBlocklist`
- W948: `UIModeUtil.CreateModifiedBlocklist`
- W949: `UIModeUtil.IsModeActive`
- W950: `UIModeUtil.RegisterMode`
- W951: `UIModeUtil.SetModeActive`
- W952: `UnitPopupSharedUtil.IsFriendshipUpgrade`
- W953: `UpdateQuestAcceptLogFullDialog`
- W954: `VisualAlert_GetTypeTemplate`
- W955: `VisualAlert_GetTypeText`
- W956: `VisualAlertData_ForEach`
- W957: `VisualAlerts_RegisterAll`
- W958: `WarfrontsPartyPoseFrame_TryShow`
- W959: `WowSurveyStatusFrame_OnSurveyDelivered`
- W960: `assertf`

### FrameXML — removed

- W963: `AddFrameLock`
- W964: `AnimatedShine_OnUpdate`
- W965: `AnimateTexCoords`
- W966: `BattleTagInviteFrame_Show`
- W967: `BetterDate`
- W968: `BFAMissionFrame_EscapePressed`
- W969: `BoostTutorial_AttemptLoad`
- W970: `BuildColoredListString`
- W971: `BuildIconArray`
- W972: `BuildListString`
- W973: `BuildMultilineTooltip`
- W974: `BuildNewLineListString`
- W975: `ButtonPulse_OnUpdate`
- W976: `CanAccessObject`
- W977: `ClassTrainerFrame_Hide`
- W978: `ClassTrainerFrame_Show`
- W979: `CloseCalendarMenus`
- W980: `CommunitiesFrame_IsEnabled`
- W981: `CompactUnitFrame_GetOptionDisplayOnlyHealerPowerBars`
- W982: `CompactUnitFrame_GetOptionDisplayPowerBar`
- W983: `CompactUnitFrame_GetOptionShowDispelIndicatorOverlay`
- W984: `ConvertRGBtoColorString`
- W985: `DisplayTypeUnassignedSupported`
- W986: `EventUtil.AreVariablesLoaded`
- W987: `ExpansionTrial_CheckLoadUI`
- W988: `FriendsFrame_CloseQuickJoinHelpTip`
- W989: `FriendsFrameAddFriendButton_OnClick`
- W990: `FriendsFriends_InitButton`
- W991: `FriendsFriends_SetSelection`
- W992: `FriendsFriendsButton_SetSelected`
- W993: `FriendsFriendsFrame_Close`
- W994: `GetDungeonNameWithDifficulty`
- W995: `GetNotchHeight`
- W996: `GetPlayerKeyState`
- W997: `GetScaledCursorDelta`
- W998: `GetScaledCursorPositionForFrame`
- W999: `GetScaledCursorPosition`
- W1000: `GetScreenHeightScale`
- W1001: `GetScreenWidthScale`
- W1002: `GetSmoothProgressChange`
- W1003: `GetSocialColoredName`
- W1004: `GetSortedSelfResurrectOptions`
- W1005: `GetUIParentOffset`
- W1006: `HelpPlatesSupported`
- W1007: `HousingControlsUtil.CanActivateHousingControls`
- W1008: `HousingTutorialUtil.HousingQuestTutorialComplete`
- W1009: `IsFrameLockActive`
- W1010: `IsFrameSmartShown`
- W1011: `IsLevelAtEffectiveMaxLevel`
- W1012: `IsPlayerAtEffectiveMaxLevel`
- W1013: `KeyBindingFrame_LoadUI`
- W1014: `LocalizePlayerFrame`
- W1015: `LocalizezhCN`
- W1016: `LocalizezhTW`
- W1017: `MacroFrame_SaveMacro`
- W1018: `MajorFactions_LoadUI`
- W1019: `MouseIsOver`
- W1020: `NPE_CheckTutorials`
- W1021: `NPETutorial_AttemptToBegin`
- W1022: `OpenAchievementFrameToAchievement`
- W1023: `OrderHallMissionFrame_EscapePressed`
- W1024: `OrderHallTalentFrame_EscapePressed`
- W1025: `OutfitterUI_LoadUI`
- W1026: `PingUtil.GetContextualPingTypeForUnit`
- W1027: `PlayerChoiceToggle_TryShow`
- W1028: `QuickJoin_JoinQueueButtonOnClick`
- W1029: `RaidBossEmoteFrame_OnEvent`
- W1030: `RaidBossEmoteFrame_OnLoad`
- W1031: `RaidBrowser_IsEmpowered`
- W1032: `RaidNotice_AddMessage`
- W1033: `RaidNotice_ClearSlot`
- W1034: `RaidNotice_Clear`
- W1035: `RaidNotice_FadeInit`
- W1036: `RaidNotice_OnUpdate`
- W1037: `RaidNotice_SetSlot`
- W1038: `RaidNotice_UpdateSlot`
- W1039: `RaidWarningFrame_OnEvent`
- W1040: `RaidWarningFrame_OnLoad`
- W1041: `RecentTimeDate`
- W1042: `RefreshAuras`
- W1043: `RegisterNewFrameLock`
- W1044: `RemoveFrameLock`
- W1045: `ReverseQuestObjective`
- W1046: `SecureAuraHeader_GetUnit`
- W1047: `SecureAuraHeader_OnAttributeChanged`
- W1048: `SecureAuraHeader_OnEvent`
- W1049: `SecureAuraHeader_OnHide`
- W1050: `SecureAuraHeader_OnLoad`
- W1051: `SecureAuraHeader_OnShow`
- W1052: `SecureAuraHeader_OnUpdate`
- W1053: `SecureAuraHeader_UpdateEventRegistrations`
- W1054: `SecureAuraHeader_Update`
- W1055: `SetDesaturation`
- W1056: `SetFrameLock`
- W1057: `ShowResurrectRequest`
- W1058: `SmartHide`
- W1059: `SmartShow`
- W1060: `TalentFrame_LoadUI`
- W1061: `TargetFrame_UpdateBuffAnchor`
- W1062: `TargetFrame_UpdateDebuffAnchor`
- W1063: `ToggleLFGFrame`
- W1064: `ToggleRafPanel`
- W1065: `ToggleRaidBrowser`
- W1066: `ToggleWoWHackCharacterUI`
- W1067: `TokenFrame_LoadUI`
- W1068: `TrialAccountCapReached_Inform`
- W1069: `UIDoFramesIntersect`
- W1070: `UIParent_ManageFramePositions`
- W1071: `UIParent_OnEvent`
- W1072: `UIParent_OnHide`
- W1073: `UIParent_OnLoad`
- W1074: `UIParent_OnShow`
- W1075: `UIParent_Shared_OnEvent`
- W1076: `UIParent_Shared_OnLoad`
- W1077: `UIParent_UpdateTopFramePositions`
- W1078: `UIParentLoadAddOn`
- W1079: `UnitHasMana`
- W1080: `UpdateFrameLock`
- W1081: `UpdateUIElementsForClientScene`
- W1082: `WorldFrame_OnLoad`
- W1083: `WorldFrame_OnUpdate`
- W1084: `WoWHackSpellsUI_LoadUI`
- W1085: `getglobal`
- W1086: `setglobal`

### ScriptObjects — added

- W1097: `DurationTextBinding:Assign`
- W1098: `DurationTextBinding:ClearTextColorCurve`
- W1099: `DurationTextBinding:Copy`
- W1100: `DurationTextBinding:GetFormattedTextColor`
- W1101: `DurationTextBinding:GetTextColorCurve`
- W1102: `DurationTextBinding:SetTextColorCurve`
- W1103: `SecondsFormatter:GetRounding`
- W1104: `SecondsFormatter:SetRounding`

### Widgets — added

- W1117: `FrameScriptObject:AddAccessRestrictions`
- W1118: `FrameScriptObject:AddForbiddenAspects`
- W1119: `FrameScriptObject:AddSecretAspect`
- W1120: `FrameScriptObject:CanBeAccessedInContext`
- W1121: `FrameScriptObject:GetAccessRestrictions`
- W1122: `FrameScriptObject:GetForbiddenAspects`
- W1123: `FrameScriptObject:GetInheritableForbiddenAspects`
- W1124: `FrameScriptObject:GetObjectTable`
- W1125: `FrameScriptObject:HasAccessConstraints`
- W1126: `FrameScriptObject:HasAnyAccessRestrictions`
- W1127: `FrameScriptObject:HasAnyForbiddenAspects`
- W1128: `FontString:SetDesaturateEmbeddedTextures`
- W1129: `TextureBase:ClearRadialProgressBar`
- W1130: `TextureBase:ClearSVG`
- W1131: `TextureBase:GetRadialProgressBarEndOffset`
- W1132: `TextureBase:GetRadialProgressBarFeather`
- W1133: `TextureBase:GetRadialProgressBarPercent`
- W1134: `TextureBase:GetRadialProgressBarReverse`
- W1135: `TextureBase:GetRadialProgressBarStartOffset`
- W1136: `TextureBase:SetRadialProgressBarEndOffset`
- W1137: `TextureBase:SetRadialProgressBarFeather`
- W1138: `TextureBase:SetRadialProgressBarPercent`
- W1139: `TextureBase:SetRadialProgressBarReverse`
- W1140: `TextureBase:SetRadialProgressBarStartOffset`
- W1141: `TextureBase:SetSVG`
- W1142: `RadialProgress:GetFromPercent`
- W1143: `RadialProgress:GetToPercent`
- W1144: `RadialProgress:SetFromPercent`
- W1145: `RadialProgress:SetToPercent`
- W1146: `Frame:AddRoleset`
- W1147: `Frame:CreateVectorGraphics`
- W1148: `Frame:GetOnUpdateMode`
- W1149: `Frame:GetRolesetNames`
- W1150: `Frame:IsRolesetFiltered`
- W1151: `Frame:RemoveRoleset`
- W1152: `Frame:ResizeToBoundsRect`
- W1153: `Frame:SetOnUpdateMode`
- W1154: `Frame:SetRolesets`
- W1155: `Minimap:SetIconScale`
- W1156: `StatusBar:GetRenderMode`
- W1157: `StatusBar:SetRenderMode`
- W1158: `VectorGraphics:ClearSVG`
- W1159: `VectorGraphics:GetSVGFileID`
- W1160: `VectorGraphics:HasSVG`
- W1161: `VectorGraphics:SetSVG`

### Widgets — changed

- W1166: `Animation:GetScript`
- W1167: `# arg1.Type cstring -> ScriptTypeName`
- W1168: `# arg2.Nilable true -> false`
- W1169: `# arg2.Type number -> ScriptBindingType`
- W1170: `+ arg2.Default = Extrinsic`
- W1171: `# ret1.Type luaFunction -> LuaFunctionReference`
- W1172: `+ ChecksForbiddenAspects`
- W1173: `+ RequiresSupportedScript`
- W1174: `Animation:HookScript`
- W1175: `# arg1.Type cstring -> ScriptTypeName`
- W1176: `# arg2.Type luaFunction -> LuaFunctionReference`
- W1177: `# arg3.Nilable true -> false`
- W1178: `# arg3.Type number -> ScriptBindingType`
- W1179: `+ arg3.Default = Extrinsic`
- W1180: `+ ChecksForbiddenAspects`
- W1181: `+ ret1 = success`
- W1182: `+ RequiresAssignableScript`
- W1183: `# SecretArguments AllowedWhenUntainted -> NotAllowed`
- W1184: `Animation:SetScript`
- W1185: `# arg1.Type cstring -> ScriptTypeName`
- W1186: `# arg2.Type luaFunction -> LuaFunctionReference`
- W1187: `+ ChecksForbiddenAspects`
- W1188: `+ RequiresAssignableScript`
- W1189: `# SecretArguments AllowedWhenUntainted -> NotAllowed`
- W1190: `AnimationGroup:GetScript`
- W1191: `# arg1.Type cstring -> ScriptTypeName`
- W1192: `# arg2.Nilable true -> false`
- W1193: `# arg2.Type number -> ScriptBindingType`
- W1194: `+ arg2.Default = Extrinsic`
- W1195: `# ret1.Type luaFunction -> LuaFunctionReference`
- W1196: `+ ChecksForbiddenAspects`
- W1197: `+ RequiresSupportedScript`
- W1198: `AnimationGroup:HookScript`
- W1199: `# arg1.Type cstring -> ScriptTypeName`
- W1200: `# arg2.Type luaFunction -> LuaFunctionReference`
- W1201: `# arg3.Nilable true -> false`
- W1202: `# arg3.Type number -> ScriptBindingType`
- W1203: `+ arg3.Default = Extrinsic`
- W1204: `+ ret1 = success`
- W1205: `+ ChecksForbiddenAspects`
- W1206: `+ RequiresAssignableScript`
- W1207: `# SecretArguments AllowedWhenUntainted -> NotAllowed`
- W1208: `AnimationGroup:SetScript`
- W1209: `# arg1.Type cstring -> ScriptTypeName`
- W1210: `# arg2.Type luaFunction -> LuaFunctionReference`
- W1211: `+ ChecksForbiddenAspects`
- W1212: `+ RequiresAssignableScript`
- W1213: `# SecretArguments AllowedWhenUntainted -> NotAllowed`
- W1214: `FrameScriptObject:SetToDefaults`
- W1215: `+ ChecksForbiddenAspects`
- W1216: `ScriptRegion:ClearScripts`
- W1217: `+ ChecksForbiddenAspects`
- W1218: `ScriptRegion:GetScript`
- W1219: `# arg1.Type cstring -> ScriptTypeName`
- W1220: `# arg2.Nilable true -> false`
- W1221: `# arg2.Type number -> ScriptBindingType`
- W1222: `+ arg2.Default = Extrinsic`
- W1223: `# ret1.Type luaFunction -> LuaFunctionReference`
- W1224: `+ ChecksForbiddenAspects`
- W1225: `+ RequiresSupportedScript`
- W1226: `ScriptRegion:HookScript`
- W1227: `# arg1.Type cstring -> ScriptTypeName`
- W1228: `# arg2.Type luaFunction -> LuaFunctionReference`
- W1229: `# arg3.Nilable true -> false`
- W1230: `# arg3.Type number -> ScriptBindingType`
- W1231: `+ arg3.Default = Extrinsic`
- W1232: `+ ret1 = success`
- W1233: `+ ChecksForbiddenAspects`
- W1234: `+ RequiresAssignableScript`
- W1235: `# SecretArguments AllowedWhenUntainted -> NotAllowed`
- W1236: `ScriptRegion:SetScript`
- W1237: `# arg1.Type cstring -> ScriptTypeName`
- W1238: `# arg2.Type luaFunction -> LuaFunctionReference`
- W1239: `+ ChecksForbiddenAspects`
- W1240: `+ RequiresAssignableScript`
- W1241: `# SecretArguments AllowedWhenUntainted -> NotAllowed`

### Events — added

- W1251: `BATTLE_NET_FRIEND_TAG_ENABLED_STATUS_UPDATED`
- W1252: `BATTLE_NET_TITLE_FRIEND_CUSTOM_NAME_ENABLED_STATUS_UPDATED`
- W1253: `CHAT_MSG_GUILD_DISCORD`
- W1254: `CONFIRM_BATTLE_NET_FRIEND_INVITE_SHOW`
- W1255: `DISCORD_GUILD_ACHIEVEMENT`
- W1256: `DISCORD_GUILD_LOBBY_UPDATE`
- W1257: `DISCORD_GUILD_SETTINGS_UPDATE`
- W1258: `DISCORD_LINK_UPDATE`
- W1259: `DISCORD_SERVER_LIST_UPDATE`
- W1260: `DISCORD_STATUS_UPDATE`
- W1261: `EXTERNAL_EVENT_LAUNCH_URL_FAILED`
- W1262: `FULLSCREEN_BROWSER_SPINNER_HIDE`
- W1263: `FULLSCREEN_BROWSER_SPINNER_SHOW`
- W1264: `GROUP_BUFF_VISUAL_ALERTS_CHANGED`
- W1265: `GUILD_RANKS_UPDATE_ACTIVE_PLAYER`
- W1266: `HIDDEN_GROUP_BUFFS_CHANGED`
- W1267: `HOUSE_RESET_COMPLETED`
- W1268: `HOUSE_RESET_FAILED`
- W1269: `HOUSING_BLUEPRINT_COLLECTION_FAILURE`
- W1270: `HOUSING_BLUEPRINT_COLLECTION_RECEIVED`
- W1271: `HOUSING_BLUEPRINT_CONTENTS_FAILURE`
- W1272: `HOUSING_BLUEPRINT_CONTENTS_RECEIVED`
- W1273: `HOUSING_BLUEPRINT_DELETE_FAILURE`
- W1274: `HOUSING_BLUEPRINT_DELETE_SUCCESS`
- W1275: `HOUSING_BLUEPRINT_EXPORT_FAILURE`
- W1276: `HOUSING_BLUEPRINT_EXPORT_SUCCESS`
- W1277: `HOUSING_BLUEPRINT_IMPORT_FAILURE`
- W1278: `HOUSING_BLUEPRINT_IMPORT_STARTED`
- W1279: `HOUSING_BLUEPRINT_IMPORT_SUCCESS`
- W1280: `HOUSING_BLUEPRINT_RENAME_FAILURE`
- W1281: `HOUSING_BLUEPRINT_RENAME_SUCCESS`
- W1282: `HOUSING_BLUEPRINTS_AVAILABILITY_CHANGED`
- W1283: `HOUSING_LAYOUT_OCCUPIED_FLOOR_RANGE_CHANGED`
- W1284: `HOUSING_NEW_DECOR_PLACE_COMPLETE`
- W1285: `IGNORE_NEIGHBORHOOD_RESPONSE`
- W1286: `LEGACY_FRIEND_SYSTEM_STATUS_UPDATED`
- W1287: `LFG_LIST_CENSORED_ACTIVE_ENTRY_UPDATE`
- W1288: `LFG_LIST_REVEALED_CENSORED_ACTIVE_ENTRY`
- W1289: `SOCIAL_UI_FRIENDS_LIST_SYSTEM_STATUS_UPDATED`
- W1290: `SOCIAL_UI_SOCIAL_QUEUE_SYSTEM_STATUS_UPDATED`
- W1291: `SOCIAL_UI_SYSTEM_STATUS_UPDATED`
- W1292: `UNIT_PING_PIN_ADDED`
- W1293: `UNIT_PING_PIN_REMOVED`

### Events — removed

- W1296: `BATTLETAG_INVITE_SHOW`
- W1297: `HOUSING_LAYOUT_NUM_FLOORS_CHANGED`

### Events — changed

- W1300: `CHAT_MSG_*`
- W1301: `+ discordInfo`
- W1302: `SPELL_UPDATE_COOLDOWN`
- W1303: `+ itemID`

### CVars — added

- W1313: `accessibilityScreenNarrationEnabled [default=1; cat=4; desc=Enables screen narration for accessibility]`
- W1314: `accessibilityScreenNarrationSpeechRate [default=0; cat=4; desc=Speed at which voice narration speaks]`
- W1315: `accessibilityScreenNarrationSpeechVolume [default=100; cat=4; desc=Volume at which voice narration speaks]`
- W1316: `accessibilityScreenNarrationVoice [default=1; cat=4; desc=Voice option used with screen narration]`
- W1317: `AftermathShaderDebug [default=0; desc=Enables NVIDIA Aftermath shader debugging] {{test-inline}}`
- W1318: `discordClientEnabled [default=1; cat=0; desc=Enable the discord client integration]`
- W1319: `discordDisplayName [default=0; cat=4; desc=The name to show for text from you in-game from Discord]`
- W1320: `nameplateCheckDistanceForTarget [default=0; cat=4; desc=If false, show our target's nameplate even if they're very far away.]`
- W1321: `nameplateForceShowUnitName [default=0; cat=4; desc=If true, nameplates will always show the unit name regardless of other unit name settings.]`
- W1322: `nameplateNotSelectedAlpha [default=-1.000000; cat=4; desc=When you have a target, the alpha of other nameplates (not used if value is negative).]`
- W1323: `nameplatePlayRemovalAnimation [default=1; cat=4; desc=If true, play a scale/alpha animation when a nameplate is removed. If false, remove the nameplate instantly.]`
- W1324: `nameplateShowAllPersonalAuras [default=0; cat=4; desc=If true, show all personal auras on nameplates, regardless of whether they are normally flagged to be shown.]`
- W1325: `nameplateShowFriendlyRealmName [default=1; scope=Account; cat=4; desc=Used to show or hide the realm name in friendly player unit nameplate names.]`
- W1326: `nameplateShowFriends [cat=4]`
- W1327: `pingTarget [default=0; cat=4; desc=Determines how pinging in the world should behave for the ping system.]`
- W1328: `raidFramesDispelIndicatorOverlayAnimation [default=0; scope=Character; cat=4; desc=When showing dispel indicators, use a pulsing alpha anim]`
- W1329: `showPingsOnRaidFrames [default=1; cat=4; desc=Enables ping details being shown on raid frames.]`
- W1330: `showScreenNarrationDialog [default=1; cat=4; desc=Show screen narration dialog on startup]`
- W1331: `taintLogObjectSecrets [default=0; cat=0; desc=If enabled, include additional taint log entries when script objects gain secret aspects or values.]`
- W1332: `tooltipShowAuraSpellIDs [default=0; cat=4; desc=Show spell IDs in tooltips for unit auras.]`
- W1333: `userFontScaleGlue [default=1.390000; cat=4; desc=glues: Defines the scale of the font used in places around the UI where readability requires larger defaults which are still customizable by the user.]`
- W1334: `worldMapShowCursorCoords [default=1; scope=Account; cat=4; desc=Show cursor coordinates on the world map]`
- W1335: `worldMapShowPlayerCoords [default=1; scope=Account; cat=4; desc=Show player coordinates on the world map]`

### CVars — removed

- W1338: `auctionDisplayOnCharacter [default=0; scope=Account; cat=4; desc=Show auction items on the dress-up paperdoll]`
- W1339: `auctionSortByBuyoutPrice [default=0; scope=Character; cat=4; desc=Sort auction items by buyout price instead of current bid price]`
- W1340: `auctionSortByUnitPrice [default=0; scope=Character; cat=4; desc=Sort auction items by unit price instead of total stack price]`
- W1341: `lastLockedDelvesCompanionAbilities [cat=4; desc=Stores the nodeIDs of the locked delve companion abilities, to highlight them when unlocked.]`
- W1342: `SlugSupersampling [default=1; cat=0; desc=The slug glyph shader performs adaptive supersampling for high-quality rendering at small font sizes]`

## Omitted healer-buff/HoT list

All entries below follow retained source L199 but were absent from its extract. Native secrecy flags are not supplied by this list.
- W286: 355941 Dream Breath
- W287: 363502 Dream Flight
- W288: 364343 Echo
- W289: 366155 Reversion
- W290: 367364 Echo Reversion
- W291: 373267 Lifebind
- W292: 376788 Echo Dream Breath
- W293: 409895 Verdant Embrace
- W298: 360827 Blistering Scales
- W299: 395152, 395296 Ebon Might
- W300: 410089 Prescience
- W301: 410263 Inferno's Blessing
- W302: 410686 Symbiotic Bloom
- W303: 413984 Shifting Sands
- W308: 774 Rejuv
- W309: 8936 Regrowth
- W310: 33763 Lifebloom
- W311: 48438 Wild Growth
- W312: 155777 Germination
- W313: 439530 Symbiotic Blooms
- W318: 17 Power Word: Shield
- W319: 194384 Atonement
- W320: 1253593 Void Shield
- W321: 1300008 Power Word: Shield (Unfolding Vision)
- W322: 1300009 Void Shield (Unfolding Vision)
- W327: 139 Renew
- W328: 41635 Prayer of Mending
- W329: 77489 Echo of Light
- W334: 115175 Soothing Mist
- W335: 119611 Renewing Mist
- W336: 124682 Enveloping Mist
- W337: 450769 Aspect of Harmony
- W338: 1292922 Coalescence
- W343: 974, 383648 Earth Shield
- W344: 61295 Riptide
- W345: 382024 Earthliving Weapon
- W346: 207400 Ancestral Vigor
- W347: 444490 Hydrobubble
- W352: 53563 Beacon of Light
- W353: 156322 Eternal Flame
- W354: 156910 Beacon of Faith
- W355: 1244893 Beacon of the Savior
- W356: 200025 Beacon of the Virtue
- W357: 431381 Dawnlight

## Exact omitted code/example locations

Eight `<syntaxhighlight>` blocks are absent from the plaintext:

- W56–75: target HELPFUL container, five explicitly created buttons, icon/duration regions and AddAuraFrame registration (historical PTR1 example).
- W112–118: radial percentage, start/end offsets, feather and reverse example.
- W126–135 and W136–143: private addon table containing a nested mixin, then XML local-source Mixins lookup.
- W1458–1471: getglobal/setglobal compatibility definitions using `_G`.
- W1476–1490: Battle.net deprecated wrappers: BNSendVerifiedBattleTagInvite ignores caller arguments and returns no results; BNGetFriendInviteInfo adapts the replacement info record to the legacy tuple.
- W1495–1546: Housing budget wrappers (C_HousingDecor.GetSpentPlacementBudget/GetMaxPlacementBudget and C_HousingLayout.GetSpentPlacementBudget/GetRoomPlacementBudget map newer nil to legacy zero), plural dye-to-first-result/nil adaptation and C_Housing.IsInsideOwnHouse alias.
- W1551–1580: raid-warning AddMessage/Clear and deprecated pool-backed configuration wrappers. These code-only behaviors are not covered by the retained named rename rows.

The initial `Bluepost` body is W20–29; it contains the announcement's aura-security rationale and addon transition/outreach guidance. It is not a consolidated API-list transclusion.

## Static accounting verification

Python-only JSON/byte/accounting inspection; no project tests, builds, Cargo or simulator execution. Source hash equals provenance; one unique ledger and triage entry per nonblank source line; only metadata rows leave audit-pending; every nonblocked substantive row belongs to one numbered batch, tests-only batches precede modelable batches, and blocked batch notes match ledger notes exactly. Cited file/line locations were checked for existence; this does not establish behavior. Scratch proof ledger: `/home/osso-test/.cache/wow-ui-sim-audit/12.1.0-session-2026-10-04/static-validation.json`.

## Supplemental raw-wikitext inventory audit — 2026-10-04

Added [wikitext register](../../sources/12.1.0-wikitext-register.json), [static triage](triage-wikitext.md), and [batch queue](batches-wikitext.md). Register parses six dropped collapsed inventories from committed raw wikitext, revision **6886719**, SHA-256 `de9e45f6e43c66c78be2d9e9c6a6e7945cd91676448bbe502a6a4db0d75d8bde`. The earlier unpinned-fetch caveat above describes that earlier operation; this supplemental register uses the committed provenance revision, not a fresh network fetch.

| Section | Added | Removed | Changed |
|---|---:|---:|---:|
| global-api | 147 | 19 | 12 |
| framexml | 337 | 124 | 0 |
| scriptobjects | 8 | 0 | 0 |
| widgets | 45 | 0 | 11 |
| events | 43 | 2 | 2 |
| cvars | 23 | 5 | 0 |

**778 entries appended; 1111 ledger rows total.** All new rows have empty capabilities and audit-pending status; `supplemental_register` identifies the new source file. Only header discrepancy: Global API 147 additions versus page header 145. Other added/removed counts match; changed blocks have no declared totals. CHAT_MSG_* remains one wildcard occurrence.

Static triage: **579 implemented-needs-proof**; **72 modelable**; **111 already-removed/absent-as-required**; **16 needs-look**.

**252 batches**: 215 tests-only candidates first; 34 modelable (maximum eight symbols); 3 deferred needs-look.

Limits: exact-token Python src/ search and cached declaration inspection only, not runtime proof or full registration reachability. Plain Blizzard Lua additions are grouped by owning cached file for future load/call proof; no vendor modifications proposed. Cache is not authenticated build 69587. Previous 12.1 FrameXML/behavior source registers and PTR machine ledgers are cross-references, never copied proof. Restricted-environment removed helpers are deferred; source mentions on other publication surfaces do not establish a removed global.

Surprises: deprecated wrappers are listed as removed by page convention and can remain conditional on loadDeprecationFallbacks; explicit post-startup removal filters still take precedence where present. Some supposedly removed event/CVar names remain in simulator tables. Several existing CVar defaults differ from the page, e.g. accessibilityScreenNarrationEnabled 0 versus 1 (`src/cvars.rs:385`, W1313); publication alone is not fidelity. Triage retains these caveats rather than granting coverage.

Validation: scratch parse.py/search.py/triage.py/batches.py/finish.py; JSON round-trip format and trailing newline checked; unique source IDs; all 778 entries occur exactly once in batches and once in triage; changed annotations retained; original 333 ledger objects remain byte-identical in serialized content. No cargo, tests, builds, simulator, agents/model CLIs or git-state operations ran. Changes remain uncommitted for main-session integration.
