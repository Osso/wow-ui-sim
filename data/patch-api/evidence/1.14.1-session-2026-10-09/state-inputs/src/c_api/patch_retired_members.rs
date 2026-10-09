//! Namespace members (and whole namespaces) the Patch 4.0.1 / 5.1.0 / 6.0.2 / 7.0.3 / 8.1.0 / 8.1.5 / 8.2.0 / 8.2.5 / 8.3.0 / 9.0.1 / 9.0.2 / 9.0.5 / 9.1.0 / 9.1.5 / 9.2.0 / 9.2.5 / 10.0.2 / 10.0.5 / 10.0.7 / 10.1.0 / 10.1.5 / 10.2.0 / 10.2.6 / 11.0.0 / 11.0.2 / 11.0.5 / 11.0.7 / 11.1.0 / 11.1.5 / 11.1.7 / 11.2.0 / 11.2.7 / 12.0.0 / 12.0.1 / 12.0.5 / 12.0.7
//! consolidated API tables list as removed that no other module retires.
//! Marking keeps the namespace `__index` autostub from fabricating them on
//! ordinary lookup.
use rilua::LuaResult;
use rilua::vm::state::LuaState;

// 11.x predates every supported retail epoch; classic profiles do not load this module.
// No current cached retail Lua consumers of these seven members. Do not retire
// UpdateUIParentPosition: cached UIParentUtil still defines and uses it.
// No cached retail Lua consumers; deprecated wrappers remain untouched.
// Qualified and bare-name cached Lua searches retained in the 11.0.0 audit.
// SpellBook transition aliases are deliberately not retired here.
// Qualified current cached retail searches find no consumers of these 10.2.6 removals.
// Bare-name matches are current global Console* successors, not these members.
// No qualified retail consumer; CampaignMixin:UsesNormalQuestIcons is unrelated.
// Qualified cached retail searches find no consumers; bare matches are unrelated APIs.
// Qualified and bare-name cached retail scans find no consumers. Classic profiles
// do not load this module; Blizzard deprecation wrappers remain untouched.
// Exact qualified scans find no 9.1.5 consumers; bare GetCategoryInfo hits
// belong to other namespaces. Blizzard deprecation wrappers remain untouched.
// No qualified or bare-name cached retail consumers; classic profiles do not load this module.
// No qualified retail consumers; bare GetProducts belongs to C_StoreSecure.
// Mists product-choice defaults remain outside this retail-only module.
// No qualified, bare-name cached retail consumers or simulator callers; grep
// evidence retained in the 8.2.0 audit. Classic profiles never load this module.
// 7.0.3: qualified/bare cached retail and complete src/tests grep scans
// find no consumers; no later master/p710-page register re-adds these names.
// Keep Summon: its current simulator callers still use the legacy member.
// 6.0.2: complete qualified/bare grep scans find no cached consumers or
// src/tests callers; master and queued 6.1.0/6.2.0/6.2.2 do not re-add these.
// 5.1.0: whole-word qualified/bare retail and complete src/tests grep scans
// find no consumers; pinned master and four queued retail pages never re-add these.
// 4.0.1 removed these unused global no-ops. Complete cached retail/src/tests
// whole-word scans are retained in the page audit; Classic keeps registration.
pub(crate) const RETIRED_4_0_1_GLOBALS: &[&str] = &["CollapseSkillHeader", "ExpandSkillHeader"];

const RETIRED_5_1_0_MEMBERS: &[(&str, &[&str])] =
    &[("C_PetJournal", &["GetSummonedPetID", "SummonPetByID"])];

const RETIRED_6_0_2_MEMBERS: &[(&str, &[&str])] = &[
    ("C_Scenario", &["GetBonusCriteriaInfo", "GetBonusStepInfo"]),
    ("C_Vignettes", &["GetVignetteInstanceID"]),
];

const RETIRED_7_0_3_MEMBERS: &[(&str, &[&str])] =
    &[("C_MountJournal", &["GetMountInfo", "GetMountInfoExtra"])];

const RETIRED_8_2_0_MEMBERS: &[(&str, &[&str])] = &[(
    "C_UIWidgetManager",
    &["GetTextureWithStateVisualizationInfo"],
)];

// Qualified/bare cached retail and complete src/tests scans find no consumers.
// SpellBook.IsSpellDisabled and LeaveParty remain reachable for current consumers.
// 8.1.0 qualified/bare cached consumer and complete caller scans are retained.
// Keep C_ConfigurationWarnings: cached consumers and simulator provider callers remain.
const RETIRED_8_1_0_MEMBERS: &[(&str, &[&str])] = &[
    ("C_Map", &["GetBountySetIDForMap"]),
    ("C_Calendar", &["EventGetClubID", "EventSetClubID"]),
];

// 8.1.5: whole-word cached qualified/bare and source/test scans find no consumers.
// Keep CanReportPlayer and SetCommunityID: current consumers/callers remain.
const RETIRED_8_1_5_MEMBERS: &[(&str, &[&str])] = &[
    ("C_AreaPoiInfo", &["GetAreaPOITimeLeft"]),
    ("C_Calendar", &["GetDate"]),
    ("C_ChatInfo", &["ReportPlayer"]),
    ("C_Club", &["AddClubStreamToChatWindow"]),
    (
        "C_DateAndTime",
        &["GetDateFromEpoch", "GetTodaysDate", "GetYesterdaysDate"],
    ),
    ("C_PvP", &["GetBrawlInfo"]),
    ("C_ReportSystem", &["ReportPlayer"]),
    (
        "C_Social",
        &[
            "GetLastScreenshot",
            "GetNumCharactersPerMedia",
            "GetScreenshotByIndex",
        ],
    ),
];

const RETIRED_8_2_5_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_ClubFinder",
        &[
            "ReportPosting",
            "ReturnCommunityApplicantList",
            "ReturnGuildApplicantList",
            "ReturnPendingCommunityApplicantList",
            "ReturnPendingGuildApplicantList",
        ],
    ),
    (
        "C_Commentator",
        &[
            "GetElapsedMs",
            "GetTrackedDefensiveCooldowns",
            "GetTrackedOffensiveCooldowns",
            "IsTrackedDefensiveCooldown",
            "IsTrackedOffensiveCooldown",
        ],
    ),
    ("C_PvP", &["GetMatchPVPStatIDs"]),
    (
        "C_RecruitAFriend",
        &["CheckEmailEnabled", "IsSendingEnabled", "SendRecruit"],
    ),
];

const RETIRED_8_3_0_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_ProductChoice",
        &[
            "GetChoices",
            "GetNumSuppressed",
            "GetProducts",
            "MakeSelection",
        ],
    ),
    ("C_WowTokenPublic", &["SellToken"]),
];

const RETIRED_9_0_1_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_CampaignInfo",
        &["GetCurrentCampaignChapterID", "GetCurrentCampaignID"],
    ),
    ("C_Commentator", &["GetTeamHighlightColor"]),
    ("C_Garrison", &["GetMissionInfo", "GetTalentTreeInfoForID"]),
    (
        "C_GossipInfo",
        &["GetGossipPoiForUiMapID", "GetGossipPoiInfo"],
    ),
    ("C_Item", &["IsItemCorruptable"]),
    ("C_LootJournal", &["GetFilteredItemSets"]),
    ("C_MountJournal", &["IsMountEquipmentUnlocked"]),
    (
        "C_TransmogCollection",
        &["GetAppearanceSourceInfoForTransmog"],
    ),
];

const RETIRED_9_0_5_MEMBERS: &[(&str, &[&str])] = &[("C_Soulbinds", &["GetConduitItemLevel"])];

// No qualified or bare-name cached retail consumers. Retain Soulbinds
// MatchesCurrentSpecSet: bare-name searches find current specialization consumers.
const RETIRED_9_0_2_MEMBERS: &[(&str, &[&str])] = &[
    ("C_CovenantSanctumUI", &["GetRenownMilestones"]),
    (
        "C_Soulbinds",
        &[
            "AddPendingConduit",
            "GetPendingConduitID",
            "GetPendingNodeIDInSoulbind",
            "HasPendingConduitInSoulbind",
            "RemovePendingConduit",
            "ResetSoulbindConduits",
        ],
    ),
    (
        "C_Spell",
        &["GetMawPowerRarityStringAndBorderAtlasBySpellID"],
    ),
    (
        "C_UIWidgetManager",
        &["GetWidgetLayoutDirectionFromWidgetSetID"],
    ),
];

const RETIRED_9_1_5_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_Commentator",
        &["SetBlacklistedAuras", "SetBlacklistedCooldowns"],
    ),
    ("C_ItemUpgrade", &["GetItemLevelIncrement"]),
    ("C_LFGList", &["GetActivityInfo", "GetCategoryInfo"]),
    ("C_LFGuildInfo", &["GetRecruitingGuildTabardInfo"]),
    ("C_PlayerMentorship", &["GetMentorOptionalAchievementIDs"]),
    (
        "C_Soulbinds",
        &[
            "GetConduitChargesCapacity",
            "GetConduitCharges",
            "GetTotalConduitChargesPendingInSoulbind",
            "GetTotalConduitChargesPending",
        ],
    ),
];

// Qualified and bare-name cached retail scans find no consumers; keep deprecation wrappers.
const RETIRED_9_1_0_MEMBERS: &[(&str, &[&str])] = &[
    ("C_BarberShop", &["OldBarberShopLoaded"]),
    ("C_LegendaryCrafting", &["GetRuneforgePowersByClassAndSpec"]),
    ("C_PetJournal", &["GetNumMaxPets"]),
    (
        "C_PlayerChoice",
        &[
            "GetPlayerChoiceInfo",
            "GetPlayerChoiceOptionInfo",
            "GetPlayerChoiceRewardInfo",
        ],
    ),
    ("C_Soulbinds", &["GetConduitRankFromCollection"]),
    ("C_Transmog", &["LoadSources", "ValidateAllPending"]),
    (
        "C_TransmogCollection",
        &[
            "CanSetFavoriteInCategory",
            "GetIllusionFallbackWeaponSource",
            "GetIllusionSourceInfo",
            "GetInspectSources",
            "GetOutfitName",
            "GetOutfitSources",
            "GetShowMissingSourceInItemTooltips",
            "SaveOutfit",
            "SetShowMissingSourceInItemTooltips",
        ],
    ),
    (
        "C_TransmogSets",
        &["GetSetSources", "IsSetCollected", "IsSetUsable"],
    ),
];

// Qualified and bare-name cached retail scans find no consumers of these nine
// members. Live reporting entry points and Blizzard deprecation wrappers remain.
const RETIRED_9_2_5_MEMBERS: &[(&str, &[&str])] = &[
    ("C_Calendar", &["ContextMenuEventComplain"]),
    (
        "C_Cursor",
        &[
            "DropCursorCommunitiesStream",
            "GetCursorCommunitiesStream",
            "SetCursorCommunitiesStream",
        ],
    ),
    ("C_LFGList", &["ReportSearchResult"]),
    (
        "C_ReportSystem",
        &[
            "OpenReportPlayerDialog",
            "SetPendingReportPetTarget",
            "SetPendingReportTargetByGuid",
            "SetPendingReportTarget",
        ],
    ),
];

// No qualified or bare-name cached retail consumers; keep deprecation wrappers.
const RETIRED_9_2_0_MEMBERS: &[(&str, &[&str])] =
    &[("C_PvP", &["GetSpecialEventDetails", "GetSpecialEventInfo"])];

const RETIRED_10_0_2_MEMBERS: &[(&str, &[&str])] = &[
    ("C_ChallengeMode", &["SetKeystoneTooltip"]),
    (
        "C_ItemInteraction",
        &[
            "SetCorruptionReforgerItemTooltip",
            "SetItemConversionOutputTooltip",
        ],
    ),
    ("C_ProfSpecs", &["GetUnspentPointsForSkillLine"]),
    (
        "C_TradeSkillUI",
        &[
            "CancelCraftingOrder",
            "CompleteCraftingOrder",
            "DeclineCraftingOrder",
            "GetCraftingOrders",
            "GetCurrentOrder",
            "GetRecipeTools",
            "HasCraftingOrderFavorites",
            "HasMaxCraftingOrderFavorites",
            "IsCraftingOrderFavorite",
            "ListCraftingOrder",
            "QueryCraftingOrdersFavorites",
            "QueryCraftingOrders",
            "RecipeCanBeRecrafted",
            "SetCraftingOrderFavorite",
            "SetTooltipRecipeResultItem",
            "StartCraftingOrder",
        ],
    ),
];

const RETIRED_10_1_0_MEMBERS: &[(&str, &[&str])] = &[
    ("C_CharacterServices", &["AssignPFCDistribution"]),
    (
        "C_LootHistory",
        &[
            "CanMasterLoot",
            "GetExpiration",
            "GetItem",
            "GetNumItems",
            "GetPlayerInfo",
            "GiveMasterLoot",
            "SetExpiration",
        ],
    ),
    (
        "C_TooltipInfo",
        &["GetQuestLogRewardSpell", "GetQuestRewardSpell"],
    ),
];

// Qualified and bare-name cached retail Lua scans find no 10.0.5 consumers.
const RETIRED_10_0_5_MEMBERS: &[(&str, &[&str])] = &[(
    "C_TradeSkillUI",
    &[
        "ContinueRecast",
        "GetRecipeRepeatCount",
        "HasRecipesTracked",
    ],
)];

// Qualified and bare-name cached retail Lua scans find no 10.0.7 consumers.
const RETIRED_10_0_7_MEMBERS: &[(&str, &[&str])] = &[
    ("C_QuestOffer", &["GetHideRequiredItemsOnTurnIn"]),
    (
        "C_Social",
        &[
            "GetLastAchievement",
            "GetLastItem",
            "GetLastScreenshotIndex",
            "GetMaxTweetLength",
            "GetScreenshotInfoByIndex",
            "GetTweetLength",
            "IsSocialEnabled",
            "RegisterSocialBrowser",
            "SetTextureToScreenshot",
            "TwitterCheckStatus",
            "TwitterConnect",
            "TwitterDisconnect",
            "TwitterGetMSTillCanPost",
            "TwitterPostAchievement",
            "TwitterPostItem",
            "TwitterPostMessage",
            "TwitterPostScreenshot",
        ],
    ),
];
const RETIRED_10_1_5_MEMBERS: &[(&str, &[&str])] = &[("C_CampaignInfo", &["UsesNormalQuestIcons"])];
const RETIRED_10_2_0_MEMBERS: &[(&str, &[&str])] = &[(
    "C_Console",
    &["GetFontHeight", "PrintAllMatchingCommands", "SetFontHeight"],
)];
const RETIRED_10_2_6_MEMBERS: &[(&str, &[&str])] = &[
    ("C_CameraDefaults", &["GetCameraFOVDefaults"]),
    ("C_TaskQuest", &["GetUIWidgetSetIDFromQuestID"]),
];
const RETIRED_11_0_0_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_MajorFactions",
        &[
            "GetFeatureAbilities",
            "IsPlayerInRenownCatchUpMode",
            "RequestCatchUpState",
        ],
    ),
    ("C_Map", &["IsMapValidForNavBarDropDown"]),
    ("C_PvP", &["GetSoloRBGMinItemLevel"]),
    ("C_Scenario", &["GetCriteriaInfo", "GetCriteriaInfoByStep"]),
    ("C_Traits", &["GetStagedPurchases"]),
    ("C_TransmogSets", &["GetBaseSetsCounts"]),
];
const RETIRED_11_0_2_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_GameModeManager",
        &["GetFeatureSetting", "IsFeatureEnabled"],
    ),
    ("C_WeeklyRewards", &["GetWeeklyRewardTextureKit"]),
];
const RETIRED_11_0_5_MEMBERS: &[(&str, &[&str])] = &[
    ("C_AuctionHouse", &["RequestFavorites"]),
    ("C_MajorFactions", &["GetCovenantIDForMajorFaction"]),
];
const RETIRED_11_0_7_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_ArrowCalloutManager",
        &[
            "HideWorldLootObjectCallout",
            "SetWorldLootObjectCalloutFromGUID",
            "SwapWorldLootObjectCallout",
        ],
    ),
    (
        "C_WorldLootObject",
        &["GetCurrentWorldLootObjectSwapInventoryType"],
    ),
];
const RETIRED_11_1_0_MEMBERS: &[(&str, &[&str])] = &[
    ("C_BarberShop", &["GetCustomizationScope"]),
    (
        "C_TransmogCollection",
        &["CanAppearanceBeDisplayedOnPlayer"],
    ),
];
const RETIRED_11_1_5_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_GameEnvironmentManager",
        &[
            "GetCurrentEventRealmQueues",
            "GetCurrentGameEnvironment",
            "RequestGameEnvironment",
        ],
    ),
    (
        "C_GameModeManager",
        &[
            "GetCurrentGameModeRecordID",
            "GetCurrentGameMode",
            "GetGameModeDisplayInfo",
        ],
    ),
    ("C_SpellBook", &["GetTrackedNameplateCooldownSpells"]),
];
const RETIRED_11_1_7_MEMBERS: &[(&str, &[&str])] =
    &[("C_Debug", &["PrintToDebugWindow", "ViewInDebugWindow"])];
const RETIRED_11_2_0_MEMBERS: &[(&str, &[&str])] = &[
    ("C_Bank", &["FetchNextPurchasableBankTabCost"]),
    ("C_Container", &["SortReagentBankBags"]),
    (
        "C_TooltipInfo",
        &["GetVoidDepositItem", "GetVoidItem", "GetVoidWithdrawalItem"],
    ),
];
const RETIRED_11_2_7_MEMBERS: &[(&str, &[&str])] = &[
    ("C_CharacterServices", &["RPEResetCharacter"]),
    ("C_ReturningPlayerUI", &["AcceptPrompt", "DeclinePrompt"]),
];

const RETIRED_12_0_0_MEMBERS: &[(&str, &[&str])] = &[
    ("C_CatalogShop", &["OpenCatalogShopInteraction"]),
    // Deprecated 11.x APIs removed by the 12.0.0 non-inventory extract.
    ("C_ChallengeMode", &["GetCompletionInfo"]),
    ("C_MythicPlus", &["IsWeeklyRewardAvailable"]),
    ("C_QuestLog", &["IsLegendaryQuest", "IsQuestRepeatableType"]),
    ("C_EventUtils", &["NotifySettingsLoaded"]),
    ("C_HouseExterior", &["GetCurrentHouseExteriorTypeName"]),
    ("C_HousingBasicMode", &["IsNudgeEnabled", "SetNudgeEnabled"]),
    ("C_HousingDecor", &["GetMaxDecorPlaced"]),
    (
        "C_NamePlate",
        &[
            "GetNamePlateEnemyClickThrough",
            "GetNamePlateEnemyPreferredClickInsets",
            "GetNamePlateEnemySize",
            "GetNamePlateFriendlyClickThrough",
            "GetNamePlateFriendlyPreferredClickInsets",
            "GetNamePlateFriendlySize",
            "GetNamePlateSelfClickThrough",
            "GetNamePlateSelfPreferredClickInsets",
            "GetNamePlateSelfSize",
            "GetNumNamePlateMotionTypes",
            "SetNamePlateEnemyClickThrough",
            "SetNamePlateEnemyPreferredClickInsets",
            "SetNamePlateEnemySize",
            "SetNamePlateFriendlyClickThrough",
            "SetNamePlateFriendlyPreferredClickInsets",
            "SetNamePlateFriendlySize",
            "SetNamePlateSelfClickThrough",
            "SetNamePlateSelfPreferredClickInsets",
            "SetNamePlateSelfSize",
        ],
    ),
    ("C_PlayerInfo", &["CanPlayerUseEventScheduler"]),
    (
        "C_PvP",
        &[
            "CanDisplayDamage",
            "CanDisplayHealing",
            "CanDisplayKillingBlows",
        ],
    ),
    ("C_StorePublic", &["IsDisabledByParentalControls"]),
    (
        "C_TaskQuest",
        &[
            "GetQuestIconUIWidgetSet",
            "GetQuestTooltipUIWidgetSet",
            "GetQuestsForPlayerByMapID",
        ],
    ),
    ("C_TooltipInfo", &["GetTransmogrifyItem"]),
    (
        "C_TradeSkillUI",
        &[
            "GetReagentRequirementItemIDs",
            "GetRecipeFixedReagentItemLink",
            "GetRecipeQualityReagentItemLink",
        ],
    ),
    (
        "C_Transmog",
        &[
            "ApplyAllPending",
            "CanTransmogItem",
            "CanTransmogItemWithItem",
            "ClearAllPending",
            "ClearPending",
            "Close",
            "GetApplyCost",
            "GetApplyWarnings",
            "GetBaseCategory",
            "GetCreatureDisplayIDForSource",
            "GetPending",
            "GetSlotEffectiveCategory",
            "GetSlotInfo",
            "GetSlotUseError",
            "IsSlotBeingCollapsed",
            "IsTransmogEnabled",
            "LoadOutfit",
            "SetPending",
        ],
    ),
    (
        "C_TransmogCollection",
        &[
            "DeleteOutfit",
            "GetItemTransmogInfoListFromOutfitHyperlink",
            "GetNumMaxOutfits",
            "GetOutfitHyperlinkFromItemTransmogInfoList",
            "GetOutfitInfo",
            "GetOutfitItemTransmogInfoList",
            "GetOutfits",
            "ModifyOutfit",
            "NewOutfit",
            "RenameOutfit",
        ],
    ),
];

// There is no 12.0.1 epoch. Retire at the first supported epoch after the
// removal, preserving the documented 12.0.0 surface.
#[cfg(feature = "retail-12-0-5")]
const RETIRED_12_0_1_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_CombatAudioAlert",
        &["GetSpeakerVolume", "SetSpeakerVolume"],
    ),
    (
        "C_NamePlate",
        &["GetTargetClampingInsets", "SetTargetClampingInsets"],
    ),
];

#[cfg(feature = "retail-12-0-5")]
const RETIRED_12_0_5_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_GossipInfo",
        &["GetActiveDelveGossip", "GetGossipDelveMapID"],
    ),
    ("C_NamePlateManager", &["SetNamePlateHitTestFrame"]),
];

/// Removed whole; its members moved to `C_PhotoSharing`.
#[cfg(feature = "retail-12-0-5")]
const RETIRED_12_0_5_NAMESPACES: &[&str] = &["C_HousingPhotoSharing"];

#[cfg(feature = "retail-12-0-7")]
const RETIRED_12_0_7_MEMBERS: &[(&str, &[&str])] = &[
    ("C_DurationUtil", &["GetCurrentTime"]),
    ("C_HousingLayout", &["IsDraggingStairwell"]),
    ("C_Minimap", &["GetObjectIconTextureCoords"]),
    ("C_Scenario", &["GetScenarioIconInfo"]),
];

pub(crate) fn mark_retired_members(state: &mut LuaState) -> LuaResult<()> {
    mark_members(state, RETIRED_5_1_0_MEMBERS)?;
    mark_members(state, RETIRED_8_1_0_MEMBERS)?;
    mark_members(state, RETIRED_8_1_5_MEMBERS)?;
    mark_members(state, RETIRED_8_2_0_MEMBERS)?;
    mark_members(state, RETIRED_8_2_5_MEMBERS)?;
    mark_members(state, RETIRED_6_0_2_MEMBERS)?;
    mark_members(state, RETIRED_7_0_3_MEMBERS)?;
    mark_members(state, RETIRED_8_3_0_MEMBERS)?;
    mark_members(state, RETIRED_9_0_1_MEMBERS)?;
    mark_members(state, RETIRED_9_0_5_MEMBERS)?;
    mark_members(state, RETIRED_9_0_2_MEMBERS)?;
    mark_members(state, RETIRED_9_1_5_MEMBERS)?;
    mark_members(state, RETIRED_9_1_0_MEMBERS)?;
    mark_members(state, RETIRED_9_2_5_MEMBERS)?;
    mark_members(state, RETIRED_9_2_0_MEMBERS)?;
    mark_members(state, RETIRED_10_0_2_MEMBERS)?;
    mark_members(state, RETIRED_10_0_5_MEMBERS)?;
    mark_members(state, RETIRED_10_1_0_MEMBERS)?;
    mark_members(state, RETIRED_10_0_7_MEMBERS)?;
    mark_members(state, RETIRED_10_1_5_MEMBERS)?;
    mark_members(state, RETIRED_10_2_0_MEMBERS)?;
    mark_members(state, RETIRED_10_2_6_MEMBERS)?;
    mark_members(state, RETIRED_11_0_0_MEMBERS)?;
    mark_members(state, RETIRED_11_0_2_MEMBERS)?;
    mark_members(state, RETIRED_11_0_5_MEMBERS)?;
    mark_members(state, RETIRED_11_0_7_MEMBERS)?;
    mark_members(state, RETIRED_11_1_0_MEMBERS)?;
    mark_members(state, RETIRED_11_1_5_MEMBERS)?;
    mark_members(state, RETIRED_11_1_7_MEMBERS)?;
    mark_members(state, RETIRED_11_2_0_MEMBERS)?;
    mark_members(state, RETIRED_11_2_7_MEMBERS)?;
    mark_members(state, RETIRED_12_0_0_MEMBERS)?;
    #[cfg(feature = "retail-12-0-5")]
    {
        mark_members(state, RETIRED_12_0_1_MEMBERS)?;
        mark_members(state, RETIRED_12_0_5_MEMBERS)?;
        mark_namespaces_absent(state, RETIRED_12_0_5_NAMESPACES)?;
    }
    #[cfg(feature = "retail-12-0-7")]
    mark_members(state, RETIRED_12_0_7_MEMBERS)?;
    Ok(())
}

fn mark_members(state: &mut LuaState, retired: &[(&'static str, &[&str])]) -> LuaResult<()> {
    for (namespace, members) in retired {
        let table = super::ensure_namespace(state, namespace)?;
        super::mark_namespace_keys_removed(state, table, members);
    }
    Ok(())
}

#[cfg(feature = "retail-12-0-5")]
fn mark_namespaces_absent(state: &mut LuaState, names: &[&'static str]) -> LuaResult<()> {
    let absent = super::ensure_namespace(state, "__wow_absent_namespaces")?;
    for name in names {
        crate::lua_api::methods::table_set_static(
            state,
            rilua::Val::Table(absent),
            name,
            rilua::Val::Bool(true),
        );
    }
    Ok(())
}
