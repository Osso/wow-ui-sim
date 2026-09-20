//! Static stub tables and registration for top-level global functions.

use rilua::vm::closure::RustFn;
use rilua::vm::state::LuaState;

use crate::lua_bridge::FromStack;

use super::{
    is_nil_global, set_global_fn, stub_false, stub_nil, stub_repair_all_cost, stub_true, stub_zero,
};

const CURRENT_EXPANSION_LEVEL: f64 = 10.0;
const CURRENT_REGION_ID: f64 = 1.0;
const NUM_EXPANSIONS: f64 = 11.0;

static GLOBAL_NIL_STUBS: &[&str] = &[
    "AddFriend",
    "AgreeToSurvey",
    "AscendToRank",
    "AuctionHouseShowAuctionator",
    "CameraZoomIn",
    "CameraZoomOut",
    "CheckCharacterUndeleteCooldown",
    "ClearCursor",
    "ClearInspectPlayer",
    "CollapseSkillHeader",
    "ConfirmBossEmote",
    "DismissSummon",
    "DoBattlefieldMaintenance",
    "DoEmote",
    "ExpandSkillHeader",
    "ForceTaint",
    "DropCursorMoney",
    "InspectUnit",
    "LeaveMythicPlusGroup",
    "LogoutStatusFrame_StartLogout",
    "LootSlot",
    "MacroFrameTab_OnClick",
    "OpenWorldMap",
    "PlayMusic",
    "PlaySound",
    "PlaySoundFile",
    "RaidGroupSetRole",
    "RepairAllItems",
    "ReportCheating",
    "RequestInspectData",
    "RequestGuildPartyState",
    "RequestLFDPartyLockInfo",
    "RequestLFDPlayerLockInfo",
    "RequestPVPOptionsEnabled",
    "RequestPVPRewards",
    "RequestRandomBattlegroundInstanceInfo",
    "RequestRatedInfo",
    "RemoveChatWindowChannel",
    // RequestPartyLootMethod is SimState-backed in globals::real::loot_method, not a stub.
    "RequestRaidInfo",
    // QuestMapFrame::UpdatePOIs still calls this legacy helper after
    // QuestMapUpdateAllQuests; the sim's quest blob rendering is state-backed
    // already, so no extra work is needed here.
    "QuestPOIUpdateIcons",
    "GetUnitPowerBarInfo",
    "GetWorldMapActionButtonSpellInfo",
    "ResetCameraPosition",
    "SetActionBarToggles",
    "SetChannelPassword",
    "SetInsertItemsLeftToRight",
    "SetLootThreshold",
    "SetPartyLeader",
    "SetRaidSubgroup",
    // SetSelectedFaction is SimState-backed in faction_probes.rs, not a stub.
    "SetUnitCritterKillCount",
    "SetView",
    // SetWatchedFaction is SimState-backed in faction_probes.rs, not a stub.
    "ShowingCinematic",
    "ShowUIPanel",
    "SortBags",
    "SortReagentBag",
    "SwitchAchievementSearchTab",
    // Classic/Mists Blizzard UI calls this host layout callback from action
    // bars, arena frames, and managed power bars. The simulator's Rust layout
    // pass computes frame positions directly, so the Lua callback is a no-op.
    "UIParent_ManageFramePositions",
    "PlayerFrame_AttachCastBar",
    "PlayerFrame_DetachCastBar",
    "StopCinematic",
    "StopMusic",
    "SwapActionSlots",
    "SwapRangedWeapon",
    "TaxiNodeSetFocus",
    "UnlearnSkill",
    "UnloadUnit",
    "UnmuteFriend",
    "UntrackAchievement",
    "UpdateTransmogrifyOutfit",
    "UseAction",
    "UseContainerItem",
    "UseInventoryItem",
    "UseItemByName",
    "WardrobeFrame_OpenTransmogToItem",
];

static GLOBAL_FALSE_STUBS: &[&str] = &[
    "AreNewRecruitTutorialsEnabled",
    "CanComplainChat",
    "CanComplainMail",
    "CanPartyLFGBackfill",
    "CanSendAuctionQuery",
    "CanShowAchievementUI",
    "CanSummonFriend",
    "CanUseLanguage",
    "DoesCurrentZoneHaveDungeon",
    "GetCVarBool",
    // GetLFGDungeonEncounterInfo is SimState-backed in
    // battlefield_lfg_probes.rs, not a stub.
    "HasLFGRestrictions",
    "InActiveBattlefield",
    "InRepairMode",
    "IsCharacterNewlyBoosted",
    // GetLootMethod / GetMasterLooterThreshold are SimState-backed in
    // globals::real::loot_method, not stubs.
    "InCinematic",
    // IsOnGroundFloorInJailersTower is SimState-backed in torghast.rs, not a stub.
    "IsInCinematicScene",
    // IsShiftKeyDown is SimState-backed in globals::real::modifier_keys, not a stub.
    "IsThreatWarningEnabled",
    "NeedToDisplayDisclaimer",
    "PetUsesPetFrame",
    "PlayerIsPVPInactive",
    "SupportsClipCursor",
    "ShouldShowMawBuffs",
    // PlayerIsTimerunning is SimState-backed in rilua_timerunning.rs, not a stub.
    "ShouldShowLevelSquishDialog",
    "UnitCanAssist",
    "UnitDistanceSquared",
    "UnitInAura",
    "UnitIsCharmed",
    "UnitIsOwnerOrControllerOfUnit",
    "UnitIsPVP",
    "UnitIsOtherPlayersPet",
    "UnitIsBattlePet",
    "UnitIsBattlePetCompanion",
    "UnitIsOtherPlayersBattlePet",
    "UnitIsWildBattlePet",
    "UnitIsBossMob",
    "UnitIsPVPSanctioned",
    "UnitIsQuestBoss",
    "UnitIsTapDenied",
    "UnitLeadsAnyGroup",
    "UnitPVPName",
    "UnitPlayerControlled",
];

static GLOBAL_ZERO_STUBS: &[&str] = &[
    // GetActionCooldown is SimState-backed in cooldown_probes.rs, not a stub.
    "GetAuctionHouseDepositRate",
    // GetBackpackCurrencyInfo is profile-wrapped over C_CurrencyInfo.
    // GetBattlefieldInstanceRunTime / GetBattlefieldStatus are
    // SimState-backed in battlefield_lfg_probes.rs, not stubs.
    // GetContainerNumFreeSlots is SimState-backed in inventory_counts.rs,
    // not a stub.
    // GetCurrentArenaSeason returns NO_ARENA_SEASON (0) — headless mode has
    // no active arena season, which routes ConquestFrame_EvaluateSeasonState
    // through the SEASON_STATE_PRESEASON / SEASON_STATE_OFFSEASON branches.
    "GetCurrentArenaSeason",
    "GetCurrentGuildBankTab",
    "GetNumBattlefieldFlagPositions",
    // GetCursorPosition is SimState-backed in real/mouse_probes.rs, not a stub.
    // GetArenaOpponentSpec lives in workarounds::temporary, not a
    // generated zero stub.
    // GetFactionInfoByID is SimState-backed in faction_probes.rs, not a stub.
    // GetGossipNumOptions / GetGossipNumAvailableQuests /
    // GetGossipNumActiveQuests are SimState-backed in real/gossip_probes.rs,
    // not stubs.
    // GetChannelName is SimState-backed in channel_verbs.rs, not a stub.
    "GetCameraZoom",
    "GetGuildBankTabCost",
    "GetGuildBankTabInfo",
    "GetGuildBankText",
    // GetGuildFactionInfo is SimState-backed in faction_probes.rs, not a stub.
    // GetGuildRosterInfo / GetGuildRosterMOTD / GetGuildRosterSize are
    // SimState-backed in guild_probes.rs, not stubs.
    "GetGuildTabardInfo",
    // GetInstanceInfo is SimState-backed in instance_info.rs, not a stub.
    "GetInventoryAlertStatus",
    // GetInventoryItemCooldown is SimState-backed in cooldown_probes.rs,
    // not a stub.
    "GetItemQualityColor",
    // GetLFGDungeonInfo / GetLFGDungeonNumEncounters / GetLFGMode are
    // SimState-backed in battlefield_lfg_probes.rs, not stubs.
    // GetMerchantNumItems is SimState-backed in inventory_counts.rs, not a stub.
    // GetMirrorTimerInfo / GetMirrorTimerProgress are SimState-backed in
    // instance_info.rs, not stubs.
    // GetMouseFocus is SimState-backed in mouse_probes.rs, not a stub.
    "GetNextInteractUnit",
    // GetNumAuctionItems is SimState-backed in inventory_counts.rs, not a stub.
    // GetNumBattlegroundEntries is SimState-backed in
    // battlefield_lfg_probes.rs, not a stub.
    // GetNumClasses is SimState-backed in social_probes.rs, not a stub.
    // GetNumGroupMembers / GetNumPartyMembers / GetNumRaidMembers /
    // GetNumSubgroupMembers are SimState-backed in group_queries.rs,
    // not stubs.
    "GetNumGuildBankTabs",
    // GetNumGuildMembers is SimState-backed in guild_probes.rs, not a stub.
    // GetNumLootItems is SimState-backed in inventory_counts.rs, not a stub.
    // GetNumQuestLogEntries is SimState-backed in quest_surface.rs, not a stub.
    // GetNumShapeshiftForms is SimState-backed in social_probes.rs, not a stub.
    // GetNumSpellTabs is spellbook-backed in real/spell_tabs.rs.
    // GetNumSkillLines / GetNumTalentTabs live in workarounds::temporary,
    // not generated zero stubs.
    // GetNumTitles is SimState-backed in social_probes.rs, not a stub.
    // GetPetExperience / GetPetHappiness / GetPetLoyalty /
    // GetPetTimeInCombat are SimState-backed in real/pet_stats.rs, not stubs.
    // GetPvpTalentSlotInfo lives in workarounds::temporary, not a
    // generated nil/default stub.
    // GetQuestLogTimeLeft / QuestMapUpdateAllQuests are SimState-backed in
    // quest_surface.rs, not stubs.
    // GetRaidRosterInfo is SimState-backed in group_queries.rs, not a stub.
    "GetRelicSlotType",
    // GetRestState is SimState-backed in real/xp_honor_rest.rs, not a stub.
    // GetSelectedSkill / GetSkillLineInfo live in workarounds::temporary,
    // not generated zero/nil stubs.
    "GetSelectedSocial",
    // GetSpellAutocast / GetSpellBonusDamage / GetSpellBonusHealing /
    // GetSpellCooldown / GetSpellLevelLearned are SimState-backed in
    // cooldown_probes.rs, not stubs.
    // GetSpellTabInfo is spellbook-backed in real/spell_tabs.rs.
    // GetTalentInfo lives in workarounds::temporary, not a generated nil stub.
    "GetSummonConfirmSummoner",
    "GetSummonConfirmTimeLeft",
    // GetTitleName is SimState-backed in social_probes.rs, not a stub.
    "GetTradeSkillInfo",
    // GetTradePlayerItemInfo / GetTradeTargetItemInfo are SimState-backed
    // in trade_verbs.rs, not stubs.
    // GetXPExhaustion is SimState-backed in real/xp_honor_rest.rs, not a stub.
    // UnitArmor / UnitAttackPower / UnitCriticalStrike / UnitDamage /
    // UnitDefense / UnitDodge / UnitParry / UnitSpellHaste / UnitStat /
    // UnitResistance / UnitRangedAttackPower / UnitRangedCriticalStrike /
    // UnitRangedDamage / UnitReaction / UnitHealthMax / UnitPowerMax /
    // UnitXP / UnitXPMax are SimState-backed in unit_stats.rs, not stubs.
    "UnitAttackBothHands",
    "UnitAttackSpeed",
    "UnitBattlePetLevel",
    "UnitBattlePetType",
    "UnitIsAFK",
    "UnitIsDND",
    "UnitIsUnit",
    "UnitRangedAttack",
];

static GLOBAL_CUSTOM_STUBS: &[(&'static str, RustFn)] = &[
    ("GetActionBarToggles", stub_action_bar_toggles),
    ("GetReadyCheckStatus", stub_nil),
    ("GetReadyCheckTimeLeft", stub_zero),
    // GetRestrictedAccountData is SimState-backed in real/xp_honor_rest.rs.
    ("GetClassicExpansionLevel", stub_current_expansion_level),
    ("ClassicExpansionAtLeast", stub_classic_expansion_at_least),
    ("GetCurrentRegion", stub_current_region),
    ("GetServerExpansionLevel", stub_current_expansion_level),
    ("GetNumExpansions", stub_num_expansions),
    ("GetRepairAllCost", stub_repair_all_cost),
    // Transmog wardrobe needs `IsUnitModelReadyForUI("player")` to return
    // true so model-reload paths progress instead of bailing out. We don't
    // simulate model load timing — the model is always "ready".
    ("IsUnitModelReadyForUI", stub_true),
    // GetUICameraInfo returns nothing; callers nil-check the first return
    // and skip the camera setup branch when unset. This is fine for a
    // headless 2D simulator that doesn't render 3D models anyway.
    ("GetUICameraInfo", stub_nil),
];

fn stub_current_expansion_level(state: &mut LuaState) -> rilua::LuaResult<u32> {
    state.push(rilua::Val::Num(CURRENT_EXPANSION_LEVEL));
    Ok(1)
}

fn stub_current_region(state: &mut LuaState) -> rilua::LuaResult<u32> {
    state.push(rilua::Val::Num(CURRENT_REGION_ID));
    Ok(1)
}

fn stub_classic_expansion_at_least(state: &mut LuaState) -> rilua::LuaResult<u32> {
    let level = f64::from_stack(state, 1).unwrap_or(0.0);
    state.push(rilua::Val::Bool(CURRENT_EXPANSION_LEVEL >= level));
    Ok(1)
}

fn stub_num_expansions(state: &mut LuaState) -> rilua::LuaResult<u32> {
    state.push(rilua::Val::Num(NUM_EXPANSIONS));
    Ok(1)
}

fn stub_action_bar_toggles(state: &mut LuaState) -> rilua::LuaResult<u32> {
    for _ in 0..7 {
        state.push(rilua::Val::Bool(false));
    }
    Ok(7)
}

pub(super) fn register_global_stubs(state: &mut LuaState) {
    for &name in GLOBAL_NIL_STUBS {
        if is_nil_global(state, name) {
            set_global_fn(state, name, stub_nil);
        }
    }
    for &name in GLOBAL_FALSE_STUBS {
        if is_nil_global(state, name) {
            set_global_fn(state, name, stub_false);
        }
    }
    for &name in GLOBAL_ZERO_STUBS {
        if is_nil_global(state, name) {
            set_global_fn(state, name, stub_zero);
        }
    }
    for &(name, func) in GLOBAL_CUSTOM_STUBS {
        if is_nil_global(state, name) {
            set_global_fn(state, name, func);
        }
    }
}
