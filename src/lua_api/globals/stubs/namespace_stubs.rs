//! Static stub tables and registration for C_* namespace functions.

use rilua::vm::closure::RustFn;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

use super::{
    is_nil_namespace, set_namespace_fn, stub_empty_table, stub_false, stub_nil, stub_zero,
};

type NsStub = (&'static str, &'static str, RustFn);

fn stub_tracking_result_and_empty_table(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    let table_ref = state.gc.alloc_table(Table::new());
    state.push(Val::Table(table_ref));
    Ok(2)
}

static NAMESPACE_NIL_STUBS: &[NsStub] = &[
    // C_AchievementInfo GetRewardItemID / GetAchievementInfo are
    // SimState-backed (via `achievements` map) in
    // missing_surface/achievement_info.rs, not stubs.
    // C_AreaPoiInfo GetAreaPOIInfo / GetAreaPOISecondsLeft are
    // SimState-backed (via `area_pois` map) in
    // c_api/c_area_poi_info.rs, not stubs.
    // C_AuctionHouse GetAuctionItemSubClasses / GetReplicateItemInfo
    // are SimState-backed in missing_surface/auction_house.rs, not
    // stubs.
    // C_BattleNet GetAccountInfoByGUID / GetFriendAccountInfo /
    // GetGameAccountInfoByGUID / GetFriendNumAccounts / GetNumFriends
    // are SimState-backed in missing_surface/battle_net.rs, not stubs.
    // C_CharacterServices GetActiveCharacterUpgradeBoostType /
    // GetActiveClassTrialBoostType are SimState-backed in
    // c_api/c_character_services.rs, not stubs.
    // C_ChatBubbles.GetAllChatBubbles is SimState-backed in
    // c_api/c_chat_bubbles.rs, not a stub.
    // C_ClassTalents GetActiveConfigID / GetConfigIDsBySpecID /
    // GetHeroTalentSpecsForClassSpec / GetTraitTreeForSpec are
    // TalentState-backed in missing_surface/traits.rs, not stubs.
    // C_Club GetClubMembers / GetSubscribedClubs / GetClubCapacity / IsEnabled
    // are WorldState-backed in missing_surface/club_info.rs, not stubs.
    // C_CurrencyInfo GetCurrencyInfo / GetCurrencyInfoFromLink /
    // GetCurrencyContainerInfo are SimState-backed (via
    // `currency_info` map) in missing_surface/item_spell/c_currency.rs,
    // not stubs.
    // C_DeathRecap GetKillingBlows / GetMostRecentDeathRecap are
    // SimState-backed in c_api/c_death_recap.rs, not stubs.
    // C_EncounterJournal GetEncounterInfo / GetInstanceInfo are
    // static-seeded in missing_surface/encounter_journal.rs, not stubs.
    // C_GossipInfo GetActiveQuests / GetAvailableQuests / GetOptions are
    // SimState-backed in missing_surface/gossip_info.rs. GetPoiForUiMapID
    // lives in workarounds::temporary until gossip POI state exists.
    // C_Heirloom GetHeirloomInfo is WorldState-backed in
    // missing_surface/heirloom.rs, not a stub.
    // GetHeirloomItemIDFromDisplayedSlot was a misnamed stub (the real
    // API is `FromDisplayedIndex`, now registered in heirloom.rs).
    // C_IncomingSummon HasIncomingSummon / IncomingSummonStatus are
    // SimState-backed in c_api/c_summon_info.rs, not stubs.
    // C_Item GetItemIconByID / GetItemNameByID / GetItemQualityByID
    // are ITEM_DB-backed in missing_surface/item_spell/c_item.rs, not
    // stubs.
    // C_LFGInfo CanPlayerUseLFD / GetLFGCategoryInfo / GetSystemPanelData /
    // IsLFGModeActiveForCategory are SimState-backed in
    // c_api/c_lfg_info.rs, not stubs.
    // C_LossOfControl.GetActiveLossOfControlData /
    // GetActiveLossOfControlDataCount are registered in
    // missing_surface/small_namespaces.rs, not stubs.
    // C_Map GetMapArtID / GetMapChildrenInfo / GetPlayerMapPosition
    // are SimState-backed (via `maps` + `player_map_position`) in
    // missing_surface/c_map.rs, not stubs.
    // C_MythicPlus GetCurrentAffixes / GetCurrentSeason / GetRunHistory /
    // GetSeasonBestAffixScoreInfoForMap / GetWeeklyChestRewardLevel are
    // SimState-backed in missing_surface/mythic_plus.rs. GetLastWeeklyChest
    // and Request* live in workarounds::temporary until cache/refresh state exists.
    // C_NamePlate GetNamePlateForUnit / GetNamePlates are registered in
    // missing_surface/nameplate.rs (nil and empty-table respectively).
    // C_PartyInfo probes are registered in c_api/c_party_info.rs.
    // Most C_PetBattles probes are registered in
    // missing_surface/pet_battles.rs; static pet-journal fallbacks live in
    // lua_api::workarounds::temporary until the broader journal model exists.
    // GetBattleState / GetNumPets remain in pet_battles.rs (env_init) so
    // `petIndex > GetNumPets(...)` keeps working as a numeric comparison.
    // C_PlayerInfo probes are registered in missing_surface/player_info.rs.
    // C_QuestLog probes are registered in missing_surface/quest_log.rs.
    // C_RaidFrames
    ("C_RaidFrames", "GetProfile", stub_nil),
    // C_Spell GetMountFromSpell / GetSpellInfo are SimState/spell-data-backed in
    // missing_surface/c_spell.rs, not stubs.
    // C_SummonInfo GetSummonReason / GetSummonConfirmTimeLeft /
    // IsSummonSkippingStartExperience are SimState-backed in
    // c_api/c_summon_info.rs, not stubs.
    // C_System.GetFrameStack is backed by SimState.hovered_frame in
    // missing_surface/small_probes.rs, not a stub.
    // C_Timer.NewTimerID is backed by next_timer_id() in
    // missing_surface/small_probes.rs, not a stub.
    // C_TooltipInfo: all Get* variants are now real implementations in
    // missing_surface/tooltip_info.rs; no stubs needed here.
    // C_TradeSkillUI
    (
        "C_TradeSkillUI",
        "GetAllProfessionTradeSkillLines",
        stub_nil,
    ),
    ("C_TradeSkillUI", "GetBaseProfessionInfo", stub_nil),
    ("C_TradeSkillUI", "GetChildProfessionInfo", stub_nil),
    ("C_TradeSkillUI", "GetCraftingOrderCount", stub_nil),
    ("C_TradeSkillUI", "GetFilteredRecipeIDs", stub_nil),
    ("C_TradeSkillUI", "GetProfessionInfoByRecipeID", stub_nil),
    ("C_TradeSkillUI", "GetProfessions", stub_nil),
    ("C_TradeSkillUI", "GetRecipeInfo", stub_nil),
    ("C_TradeSkillUI", "GetRecipeItemLink", stub_nil),
    ("C_TradeSkillUI", "GetRecipeNumReagents", stub_nil),
    ("C_TradeSkillUI", "GetRecipeReagentInfo", stub_nil),
    ("C_TradeSkillUI", "GetRecipeReagentItemLink", stub_nil),
    ("C_TradeSkillUI", "GetRecipeSchematic", stub_nil),
    ("C_TradeSkillUI", "GetTradeSkillListLink", stub_nil),
    ("C_TradeSkillUI", "GetFilterableInventorySlotName", stub_nil),
    ("C_TradeSkillUI", "SetInventorySlotFilter", stub_nil),
    ("C_TradeSkillUI", "ClearInventorySlotFilter", stub_nil),
    // C_TrophyHall.GetTrophyInfo is registered in
    // missing_surface/small_namespaces.rs, not a stub.
    // C_Tutorial
    ("C_Tutorial", "AcknowledgeTutorial", stub_nil),
    ("C_Tutorial", "HasSeenTutorial", stub_nil),
    // C_UnitAuras GetAuraDataByAuraInstanceID / GetAuraDataByIndex /
    // GetAuraDataBySpellName / GetBuffDataByIndex /
    // GetDebuffDataByIndex are SimState-backed (via `player.buffs`) in
    // globals/auras.rs, not stubs. That registration runs AFTER the
    // stub registrar on purpose, so these entries were dead.
    // C_UIWidgetManager
    ("C_UIWidgetManager", "GetBelowMinimapWidgetSetID", stub_nil),
    (
        "C_UIWidgetManager",
        "GetObjectiveTrackerWidgetSetID",
        stub_nil,
    ),
    ("C_UIWidgetManager", "GetTopCenterWidgetSetID", stub_nil),
    ("C_UIWidgetManager", "GetWidgetSetInfo", stub_nil),
    // C_VoiceChat GetActiveChannelID / GetChannel / GetChannels /
    // GetCurrentVoiceChatConnectionStatusCode / GetMasterVolumeScale /
    // GetMicrophoneVolume / GetOutputVolume are SimState-backed in
    // missing_surface/voice_chat.rs, not stubs.
    // C_WowEntitlements
    ("C_WowEntitlements", "GetAllEntitlementsByType", stub_nil),
    // C_WowLabs
    ("C_WowLabs", "GetMatchmakingEnabled", stub_nil),
    ("C_WowLabsMatchmaking", "CancelQueue", stub_nil),
    ("C_WowLabsMatchmaking", "GetCurrentQueue", stub_nil),
    ("C_WowLabsMatchmaking", "GetQueue", stub_nil),
    ("C_WowLabsMatchmaking", "JoinQueue", stub_nil),
];

static NAMESPACE_FALSE_STUBS: &[NsStub] = &[
    // C_AchievementInfo IsValidAchievement is SimState-backed in
    // missing_surface/achievement_info.rs, not a stub.
    // C_Bank.HasFullBankAccess is registered (returns true) in
    // missing_surface/small_namespaces.rs, not a stub.
    // C_BattleNet
    ("C_BattleNet", "IsAccountMuted", stub_false),
    // C_CharacterServices HasRequiredServiceForCharacterUpgrade lives in
    // workarounds::temporary::character_services_defaults, not a stub.
    // C_ClassTalents CanChangeTalents / GetHasStarterBuild /
    // IsStarterBuildActive are TalentState-backed in
    // missing_surface/traits.rs, not stubs.
    // C_Club IsEnabled is WorldState-backed in missing_surface/club_info.rs, not a stub.
    // C_GarrisonInfo.HasGarrison is registered in
    // missing_surface/small_namespaces.rs, not a stub.
    // C_IncomingSummon
    (
        "C_IncomingSummon",
        "HasIncomingSummonFromFriend",
        stub_false,
    ),
    // C_Item
    ("C_Item", "IsItemTransmogrifiable", stub_false),
    // C_TradeSkillUI
    ("C_TradeSkillUI", "AreAnyInventorySlotsFiltered", stub_false),
    ("C_TradeSkillUI", "IsInventorySlotFiltered", stub_false),
    // C_LFGInfo — CanPlayerUseLFD / GetLFGCategoryInfo / GetSystemPanelData /
    // IsLFGModeActiveForCategory are SimState-backed in c_api/c_lfg_info.rs.
    // CanPlayerUsePremadeGroup is SimState-backed in c_api/c_lfg_info.rs.
    // C_Map.IsMapValidForNavigation is registered in
    // missing_surface/small_namespaces.rs, not a stub.
    // C_MythicPlus IsMythicPlusActive / IsWeeklyRewardAvailable are SimState-backed
    // in missing_surface/mythic_plus.rs, not stubs.
    // C_PartyInfo probes are registered in c_api/c_party_info.rs.
    // C_PvP.IsMatchConsideredArena is registered in
    // missing_surface/small_namespaces.rs, not a stub.
    // C_PhotoSharing.IsAuthorized / IsEnabled are SimState-backed in photo_sharing.rs.
    // C_PlayerInfo probes are registered in missing_surface/player_info.rs.
    // C_QuestLog probes are registered in missing_surface/quest_log.rs.
    // C_Spell GetVisibilityInfo / IsPriorityAura / IsSelfBuff / IsSpellUsable /
    // TargetSpell* are SimState/spell-data-backed in missing_surface/c_spell.rs.
    // C_SummonInfo IsSummonSkippingStartExperience is SimState-backed in
    // c_api/c_summon_info.rs, not a stub.
    // C_StableInfo.IsAtPetStable is SimState-backed in c_api/c_stable_info.rs,
    // not a stub.
    // C_VoiceChat IsDeafened / IsEnabled / IsMuted / IsParentalDisabled /
    // IsTalking are SimState-backed in missing_surface/voice_chat.rs, not stubs.
    // C_WowLabs
    ("C_WowLabs", "IsEnabled", stub_false),
];

static NAMESPACE_ZERO_STUBS: &[NsStub] = &[
    // C_BattleNet
    // C_BattleNet GetFriendNumAccounts / GetNumFriends are
    // SimState-backed in missing_surface/battle_net.rs, not stubs.
    // C_Club GetClubCapacity is WorldState-backed in missing_surface/club_info.rs, not a stub.
    // C_GarrisonInfo.GetGarrisonType is registered in
    // missing_surface/small_namespaces.rs, not a stub.
    // C_MythicPlus GetOwnedKeystoneLevel / GetWeeklyBestForMap are SimState-backed
    // in missing_surface/mythic_plus.rs, not stubs.
    // C_PartyInfo GetActiveGroupType is registered in c_api/c_party_info.rs.
    // C_QuestLog probes are registered in missing_surface/quest_log.rs.
    // C_Spell GetSpellCooldown is SimState-backed in missing_surface/c_spell.rs.
    // C_SummonInfo GetSummonConfirmTimeLeft is SimState-backed in
    // c_api/c_summon_info.rs, not a stub.
    // C_TradeSkillUI
    ("C_TradeSkillUI", "GetNumRecipes", stub_zero),
    ("C_TradeSkillUI", "GetNumTradeSkills", stub_zero),
    (
        "C_TradeSkillUI",
        "GetAllFilterableInventorySlotsCount",
        stub_zero,
    ),
    // C_VoiceChat GetNumActiveChannels / GetNumMembers are SimState-backed in
    // missing_surface/voice_chat.rs, not stubs.
];

static NAMESPACE_EMPTY_TABLE_STUBS: &[NsStub] = &[
    ("C_AreaPoiInfo", "GetDelvesForMap", stub_empty_table),
    ("C_AreaPoiInfo", "GetEventsForMap", stub_empty_table),
    ("C_AreaPoiInfo", "GetQuestHubsForMap", stub_empty_table),
    // C_AuctionHouse GetBrowseResults is SimState-backed in
    // missing_surface/auction_house.rs, not a stub.
    // C_CinematicList
    ("C_CinematicList", "GetUICinematicList", stub_empty_table),
    // C_ClassTalents GetConfigIDsBySpecID is TalentState-backed in
    // missing_surface/traits.rs, not a stub.
    // C_Club GetClubMembers / GetStreams / GetSubscribedClubs are
    // registered in missing_surface/club_info.rs, not stubs.
    (
        "C_ContentTracking",
        "GetCollectableSourceTypes",
        stub_empty_table,
    ),
    // C_GossipInfo GetActiveQuests / GetAvailableQuests / GetOptions are
    // SimState-backed in missing_surface/gossip_info.rs, not stubs.
    ("C_DeathInfo", "GetGraveyardsForMap", stub_empty_table),
    (
        "C_EncounterJournal",
        "GetDungeonEntrancesForMap",
        stub_empty_table,
    ),
    ("C_EncounterJournal", "GetEncountersOnMap", stub_empty_table),
    (
        "C_Garrison",
        "GetGarrisonPlotsInstancesForMap",
        stub_empty_table,
    ),
    ("C_Garrison", "GetBuildingSizes", stub_empty_table),
    (
        "C_Garrison",
        "GetRecruiterAbilityCategories",
        stub_empty_table,
    ),
    // C_LFGInfo GetSystemPanelData is SimState-backed in c_api/c_lfg_info.rs.
    ("C_Map", "GetMapBannersForMap", stub_empty_table),
    ("C_Map", "GetMapLinksForMap", stub_empty_table),
    // C_NamePlate GetNamePlates is registered in missing_surface/nameplate.rs.
    // C_PartyInfo GetActiveCategories is registered in c_api/c_party_info.rs.
    ("C_QuestLine", "GetAvailableQuestLines", stub_empty_table),
    ("C_QuestLine", "GetForceVisibleQuests", stub_empty_table),
    ("C_ResearchInfo", "GetDigSitesForMap", stub_empty_table),
    // C_ZoneAbility
    ("C_ZoneAbility", "GetActiveAbilities", stub_empty_table),
    // C_QuestLog probes are registered in missing_surface/quest_log.rs.
    // C_TradeSkillUI
    (
        "C_TradeSkillUI",
        "GetAllProfessionTradeSkillLines",
        stub_empty_table,
    ),
    ("C_TradeSkillUI", "GetFilteredRecipeIDs", stub_empty_table),
    (
        "C_TradeSkillUI",
        "GetAllFilterableInventorySlots",
        stub_empty_table,
    ),
    (
        "C_TradeSkillUI",
        "GetFilterableInventorySlots",
        stub_empty_table,
    ),
    // C_UnitAuras
    // GetAuraSlots returns (continuationToken, slot1, slot2, ...). Callers
    // drive AuraUtil.ForEachAura via a `repeat ... until token == nil` loop
    // (Blizzard_FrameXMLUtil/AuraUtil.lua:114-117). Returning an empty table
    // as the first value makes that token truthy and loops forever.
    ("C_UnitAuras", "GetAuraSlots", stub_nil),
    // C_VoiceChat GetChannels is SimState-backed in missing_surface/voice_chat.rs, not a stub.
    // C_WowLabs
    ("C_WowLabs", "GetAvailableQueues", stub_empty_table),
];

static NAMESPACE_CUSTOM_STUBS: &[NsStub] = &[(
    "C_ContentTracking",
    "GetTrackablesOnMap",
    stub_tracking_result_and_empty_table,
)];

pub(super) fn register_namespace_stubs(state: &mut LuaState) {
    for &(ns, method, func) in NAMESPACE_NIL_STUBS {
        if is_nil_namespace(state, ns, method) {
            set_namespace_fn(state, ns, method, func);
        }
    }
    for &(ns, method, func) in NAMESPACE_FALSE_STUBS {
        if is_nil_namespace(state, ns, method) {
            set_namespace_fn(state, ns, method, func);
        }
    }
    for &(ns, method, func) in NAMESPACE_ZERO_STUBS {
        if is_nil_namespace(state, ns, method) {
            set_namespace_fn(state, ns, method, func);
        }
    }
    for &(ns, method, func) in NAMESPACE_EMPTY_TABLE_STUBS {
        if is_nil_namespace(state, ns, method) {
            set_namespace_fn(state, ns, method, func);
        }
    }
    for &(ns, method, func) in NAMESPACE_CUSTOM_STUBS {
        if is_nil_namespace(state, ns, method) {
            set_namespace_fn(state, ns, method, func);
        }
    }
}
