//! Namespace members (and whole namespaces) the Patch 11.0.5 / 11.0.7 / 11.1.0 / 11.1.5 / 11.1.7 / 11.2.0 / 11.2.7 / 12.0.0 / 12.0.1 / 12.0.5 / 12.0.7
//! consolidated API tables list as removed that no other module retires.
//! Marking keeps the namespace `__index` autostub from fabricating them on
//! ordinary lookup.
use rilua::LuaResult;
use rilua::vm::state::LuaState;

// 11.x predates every supported retail epoch; classic profiles do not load this module.
// No current cached retail Lua consumers of these seven members. Do not retire
// UpdateUIParentPosition: cached UIParentUtil still defines and uses it.
// No cached retail Lua consumers; deprecated wrappers remain untouched.
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
