//! pre-existing simulator route preserved from the former blanket _G copy;
//! native secure publication UNKNOWN (R88ac.ac1, R88af.1/.2); remove when Evidence: pr46-49-decomp-answers.md, sections R88ac.ac1, R88af.1.
//! native evidence lands.
//!
//! SOURCE-DERIVED from master 725fb6e3a: each entry records the profiles
//! where its registrar runs and the profiles that use this compatibility
//! route. VERIFIED R88y publication is active only on Retail. Evidence: pr46-49-decomp-answers.md, sections R88y.

use rilua::{LuaApiMut, LuaResult, Val};

use crate::client_profile::ClientProfile;

struct MasterUnitGlobal {
    name: &'static str,
    registered_profiles: &'static [ClientProfile],
    compatibility_profiles: &'static [ClientProfile],
    registrar: &'static str,
}

const fn unit_global(
    name: &'static str,
    registered_profiles: &'static [ClientProfile],
    compatibility_profiles: &'static [ClientProfile],
    registrar: &'static str,
) -> MasterUnitGlobal {
    MasterUnitGlobal {
        name,
        registered_profiles,
        compatibility_profiles,
        registrar,
    }
}

const ALL_PROFILES: &[ClientProfile] = &[
    ClientProfile::Retail,
    ClientProfile::Ptr,
    ClientProfile::Wrath,
    ClientProfile::Mists,
    ClientProfile::Era,
    ClientProfile::Anniversary,
    ClientProfile::WowForever,
];
const WITHOUT_RETAIL_PROFILES: &[ClientProfile] = &[
    ClientProfile::Ptr,
    ClientProfile::Wrath,
    ClientProfile::Mists,
    ClientProfile::Era,
    ClientProfile::Anniversary,
    ClientProfile::WowForever,
];
const RETAIL_PTR_PROFILES: &[ClientProfile] = &[ClientProfile::Retail, ClientProfile::Ptr];
const RETAIL_PTR_FOREVER_PROFILES: &[ClientProfile] = &[
    ClientProfile::Retail,
    ClientProfile::Ptr,
    ClientProfile::WowForever,
];
const RETAIL_ONLY_PROFILE: &[ClientProfile] = &[ClientProfile::Retail];
const UNIT_IN_SUBGROUP_PROFILES: &[ClientProfile] = &[
    ClientProfile::Retail,
    ClientProfile::Ptr,
    ClientProfile::Mists,
    ClientProfile::Era,
    ClientProfile::Anniversary,
    ClientProfile::WowForever,
];
const WRATH_MISTS_PROFILES: &[ClientProfile] = &[ClientProfile::Wrath, ClientProfile::Mists];
const CLASSIC_PVP_PROFILES: &[ClientProfile] = &[
    ClientProfile::Wrath,
    ClientProfile::Mists,
    ClientProfile::Era,
    ClientProfile::Anniversary,
];
const MISTS_ONLY_PROFILE: &[ClientProfile] = &[ClientProfile::Mists];
const RETAIL_FOREVER_PROFILES: &[ClientProfile] =
    &[ClientProfile::Retail, ClientProfile::WowForever];
const FOREVER_ONLY_PROFILE: &[ClientProfile] = &[ClientProfile::WowForever];
const WRATH_FOREVER_PROFILES: &[ClientProfile] = &[ClientProfile::Wrath, ClientProfile::WowForever];
const WRATH_ONLY_PROFILE: &[ClientProfile] = &[ClientProfile::Wrath];
const PTR_ONLY_PROFILE: &[ClientProfile] = &[ClientProfile::Ptr];

const MASTER_SNAPSHOT_UNIT_GLOBALS: &[MasterUnitGlobal] = &[
    unit_global(
        "UnitAffectingCombat",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:523",
    ),
    unit_global(
        "UnitArmor",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:631",
    ),
    unit_global(
        "UnitAttackBothHands",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:233",
    ),
    unit_global(
        "UnitAttackPower",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:632",
    ),
    unit_global(
        "UnitAttackSpeed",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:639",
    ),
    unit_global(
        "UnitAura",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/auras.rs:124; src/mists/compat_bootstrap.lua:822-824 (Mists override)",
    ),
    unit_global(
        "UnitBattlePetLevel",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:235",
    ),
    unit_global(
        "UnitBattlePetType",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:236",
    ),
    unit_global(
        "UnitBuff",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/auras.rs:122; src/mists/compat_bootstrap.lua:814-816 (Mists override)",
    ),
    unit_global(
        "UnitCanAssist",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:140",
    ),
    unit_global(
        "UnitCanAttack",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:139",
    ),
    unit_global(
        "UnitCanCooperate",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:141",
    ),
    unit_global(
        "UnitCastingDuration",
        RETAIL_PTR_FOREVER_PROFILES,
        RETAIL_PTR_FOREVER_PROFILES,
        "src/lua_api/channeling/durations.rs:16",
    ),
    unit_global(
        "UnitCastingInfo",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:625",
    ),
    unit_global(
        "UnitChannelDuration",
        RETAIL_PTR_FOREVER_PROFILES,
        RETAIL_PTR_FOREVER_PROFILES,
        "src/lua_api/channeling/durations.rs:17",
    ),
    unit_global(
        "UnitChannelInfo",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:626",
    ),
    unit_global(
        "UnitCharacterPoints",
        WRATH_MISTS_PROFILES,
        WRATH_MISTS_PROFILES,
        "src/lua_api/globals/missing_surface/professions_legacy_trainer.rs:58 (Mists); src/wrath/compat_bootstrap.lua:455-458 (Wrath)",
    ),
    unit_global(
        "UnitClass",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:72",
    ),
    unit_global(
        "UnitClassBase",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:513",
    ),
    unit_global(
        "UnitClassFromGUID",
        RETAIL_PTR_FOREVER_PROFILES,
        RETAIL_PTR_FOREVER_PROFILES,
        "src/lua_api/globals/real/player_identity.rs:140",
    ),
    unit_global(
        "UnitClassification",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:75",
    ),
    unit_global(
        "UnitControllingVehicle",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/vehicle_possession.rs:143",
    ),
    unit_global(
        "UnitCreatureFamily",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:517",
    ),
    unit_global(
        "UnitCreatureID",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:515",
    ),
    unit_global(
        "UnitCreatureType",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:76",
    ),
    unit_global(
        "UnitCriticalStrike",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:634",
    ),
    unit_global(
        "UnitDamage",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:637",
    ),
    unit_global(
        "UnitDebuff",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/auras.rs:123; src/mists/compat_bootstrap.lua:818-820 (Mists override)",
    ),
    unit_global(
        "UnitDefense",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:641",
    ),
    unit_global(
        "UnitDefenseSkill",
        FOREVER_ONLY_PROFILE,
        FOREVER_ONLY_PROFILE,
        "src/lua_api/globals/unit_stats.rs:643",
    ),
    unit_global(
        "UnitDetailedThreatSituation",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/workarounds/temporary/unit_threat_defaults.rs:9",
    ),
    unit_global(
        "UnitDistanceSquared",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:129",
    ),
    unit_global(
        "UnitDodge",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:644",
    ),
    unit_global(
        "UnitEffectiveLevel",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:74",
    ),
    unit_global(
        "UnitEmpoweredChannelDuration",
        RETAIL_PTR_FOREVER_PROFILES,
        RETAIL_PTR_FOREVER_PROFILES,
        "src/lua_api/channeling/durations.rs:18",
    ),
    unit_global(
        "UnitEmpoweredStageDurations",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/channeling/durations.rs:20",
    ),
    unit_global(
        "UnitEmpoweredStagePercentages",
        RETAIL_PTR_FOREVER_PROFILES,
        RETAIL_PTR_FOREVER_PROFILES,
        "src/lua_api/channeling/durations.rs:21",
    ),
    unit_global(
        "UnitExists",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:69",
    ),
    unit_global(
        "UnitFactionGroup",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:83",
    ),
    unit_global(
        "UnitFullName",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:512",
    ),
    unit_global(
        "UnitGUID",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:514",
    ),
    unit_global(
        "UnitGetAvailableRoles",
        ALL_PROFILES,
        WITHOUT_RETAIL_PROFILES,
        "src/lua_api/globals/group_queries.rs:187",
    ),
    unit_global(
        "UnitGetDetailedHealPrediction",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:622",
    ),
    unit_global(
        "UnitGetIncomingHeals",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:617",
    ),
    unit_global(
        "UnitGetTotalAbsorbs",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:618",
    ),
    unit_global(
        "UnitGetTotalHealAbsorbs",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:619",
    ),
    unit_global(
        "UnitGroupRolesAssigned",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:177",
    ),
    unit_global(
        "UnitGroupRolesAssignedEnum",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:182",
    ),
    unit_global(
        "UnitHPPerStamina",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:614",
    ),
    unit_global(
        "UnitHasIncomingResurrection",
        ALL_PROFILES,
        WITHOUT_RETAIL_PROFILES,
        "src/lua_api/globals/group_queries.rs:225",
    ),
    unit_global(
        "UnitHasLFGDeserter",
        ALL_PROFILES,
        WITHOUT_RETAIL_PROFILES,
        "src/lua_api/globals/group_queries.rs:128",
    ),
    unit_global(
        "UnitHasLFGRandomCooldown",
        ALL_PROFILES,
        WITHOUT_RETAIL_PROFILES,
        "src/lua_api/globals/group_queries.rs:131",
    ),
    unit_global(
        "UnitHasLootInteraction",
        FOREVER_ONLY_PROFILE,
        FOREVER_ONLY_PROFILE,
        "src/lua_api/globals/real/unit_interaction.rs:12",
    ),
    unit_global(
        "UnitHasPowerType",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:611",
    ),
    unit_global(
        "UnitHasRelicSlot",
        WRATH_FOREVER_PROFILES,
        WRATH_FOREVER_PROFILES,
        "src/lua_api/globals/real/wrath_relic_slot.rs:115 (Wrath); src/lua_api/globals/real/combat_stats.rs:430 (WowForever)",
    ),
    unit_global(
        "UnitHasVehiclePlayerFrameUI",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/vehicle_possession.rs:140",
    ),
    unit_global(
        "UnitHasVehicleUI",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/vehicle_possession.rs:137",
    ),
    unit_global(
        "UnitHealth",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:603",
    ),
    unit_global(
        "UnitHealthMax",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:604",
    ),
    unit_global(
        "UnitHealthMissing",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:606",
    ),
    unit_global(
        "UnitHealthPercent",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:607",
    ),
    unit_global(
        "UnitHonor",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/xp_honor_rest.rs:146",
    ),
    unit_global(
        "UnitHonorLevel",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/state_backed_queries.rs:35",
    ),
    unit_global(
        "UnitHonorMax",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/xp_honor_rest.rs:147",
    ),
    unit_global(
        "UnitInAura",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:130",
    ),
    unit_global(
        "UnitInBattleground",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:92",
    ),
    unit_global(
        "UnitInOtherParty",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:169",
    ),
    unit_global(
        "UnitInParty",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:150",
    ),
    unit_global(
        "UnitInPartyIsAI",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:121",
    ),
    unit_global(
        "UnitInPhase",
        MISTS_ONLY_PROFILE,
        MISTS_ONLY_PROFILE,
        "src/mists/compat_bootstrap.lua:914-915 (Mists)",
    ),
    unit_global(
        "UnitInRaid",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:151",
    ),
    unit_global(
        "UnitInRange",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:84",
    ),
    unit_global(
        "UnitInSubgroup",
        UNIT_IN_SUBGROUP_PROFILES,
        UNIT_IN_SUBGROUP_PROFILES,
        "src/lua_api/globals/group_queries.rs:153",
    ),
    unit_global(
        "UnitInVehicle",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/vehicle_possession.rs:136",
    ),
    unit_global(
        "UnitIsAFK",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:519",
    ),
    unit_global(
        "UnitIsBattlePet",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:132",
    ),
    unit_global(
        "UnitIsBattlePetCompanion",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:133",
    ),
    unit_global(
        "UnitIsBossMob",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:136",
    ),
    unit_global(
        "UnitIsCharmed",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:109",
    ),
    unit_global(
        "UnitIsCivilian",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:217",
    ),
    unit_global(
        "UnitIsConnected",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:215",
    ),
    unit_global(
        "UnitIsCorpse",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:216",
    ),
    unit_global(
        "UnitIsDND",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:520",
    ),
    unit_global(
        "UnitIsDead",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:208",
    ),
    unit_global(
        "UnitIsDeadOrGhost",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:212",
    ),
    unit_global(
        "UnitIsEnemy",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:138",
    ),
    unit_global(
        "UnitIsFeignDeath",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:526",
    ),
    unit_global(
        "UnitIsFriend",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:137",
    ),
    unit_global(
        "UnitIsGameObject",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/unit_interaction.rs:10",
    ),
    unit_global(
        "UnitIsGhost",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:209",
    ),
    unit_global(
        "UnitIsGroupAssistant",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:202",
    ),
    unit_global(
        "UnitIsGroupLeader",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:197",
    ),
    unit_global(
        "UnitIsHumanPlayer",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_probes.rs:109",
    ),
    unit_global(
        "UnitIsInInteractRange",
        FOREVER_ONLY_PROFILE,
        FOREVER_ONLY_PROFILE,
        "src/lua_api/globals/real/unit_interaction.rs:13",
    ),
    unit_global(
        "UnitIsInteractable",
        FOREVER_ONLY_PROFILE,
        FOREVER_ONLY_PROFILE,
        "src/lua_api/globals/real/unit_interaction.rs:14",
    ),
    unit_global(
        "UnitIsLieutenant",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/real/publication_unit_12_0_0.rs:128",
    ),
    unit_global(
        "UnitIsMinion",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/real/publication_unit_12_0_0.rs:129",
    ),
    unit_global(
        "UnitIsNPCAsPlayer",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/real/publication_unit_12_0_0.rs:130",
    ),
    unit_global(
        "UnitIsOtherPlayersBattlePet",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:134",
    ),
    unit_global(
        "UnitIsOtherPlayersPet",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:131",
    ),
    unit_global(
        "UnitIsOwnerOrControllerOfUnit",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:112",
    ),
    unit_global(
        "UnitIsPVP",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:115",
    ),
    unit_global(
        "UnitIsPVPFreeForAll",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:124",
    ),
    unit_global(
        "UnitIsPVPSanctioned",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:137",
    ),
    unit_global(
        "UnitIsPartyLeader",
        WRATH_ONLY_PROFILE,
        WRATH_ONLY_PROFILE,
        "src/lua_api/globals/real/wrath_startup_queries.rs:34",
    ),
    unit_global(
        "UnitIsPlayer",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_probes.rs:105",
    ),
    unit_global(
        "UnitIsPlayerControlledOrGroupMember",
        RETAIL_PTR_FOREVER_PROFILES,
        RETAIL_PTR_FOREVER_PROFILES,
        "src/lua_api/globals/real/unit_relationships.rs:37",
    ),
    unit_global(
        "UnitIsPossessed",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:108",
    ),
    unit_global(
        "UnitIsQuestBoss",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:138",
    ),
    unit_global(
        "UnitIsRaidOfficer",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:192",
    ),
    unit_global(
        "UnitIsTalking",
        WRATH_ONLY_PROFILE,
        WRATH_ONLY_PROFILE,
        "src/lua_api/globals/real/wrath_voice.rs:19 (Wrath)",
    ),
    unit_global(
        "UnitIsTapDenied",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:139",
    ),
    unit_global(
        "UnitIsTapped",
        WRATH_ONLY_PROFILE,
        WRATH_ONLY_PROFILE,
        "src/lua_api/globals/real/wrath_startup_queries.rs:35",
    ),
    unit_global(
        "UnitIsTappedByAllThreatList",
        WRATH_ONLY_PROFILE,
        WRATH_ONLY_PROFILE,
        "src/lua_api/globals/real/wrath_startup_queries.rs:39",
    ),
    unit_global(
        "UnitIsTappedByPlayer",
        WRATH_ONLY_PROFILE,
        WRATH_ONLY_PROFILE,
        "src/lua_api/globals/real/wrath_startup_queries.rs:36",
    ),
    unit_global(
        "UnitIsUnconscious",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:220",
    ),
    unit_global(
        "UnitIsUnit",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:521",
    ),
    unit_global(
        "UnitIsVisible",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:228",
    ),
    unit_global(
        "UnitIsWildBattlePet",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/stubs/global_stubs.rs:135",
    ),
    unit_global(
        "UnitLeadsAnyGroup",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:144",
    ),
    unit_global(
        "UnitLevel",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:73",
    ),
    unit_global(
        "UnitName",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:70",
    ),
    unit_global(
        "UnitNameFromGUID",
        RETAIL_PTR_FOREVER_PROFILES,
        RETAIL_PTR_FOREVER_PROFILES,
        "src/lua_api/globals/real/player_identity.rs:138",
    ),
    unit_global(
        "UnitNameUnmodified",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:71",
    ),
    unit_global(
        "UnitOnTaxi",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/vehicle_possession.rs:144",
    ),
    unit_global(
        "UnitPVPName",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:511",
    ),
    unit_global(
        "UnitPVPRank",
        CLASSIC_PVP_PROFILES,
        CLASSIC_PVP_PROFILES,
        "src/lua_api/globals/pvp_probes.rs:560 (Mists/Era/Anniversary); src/wrath/compat_bootstrap.lua:507-510 (Wrath)",
    ),
    unit_global(
        "UnitParry",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:645",
    ),
    unit_global(
        "UnitPhaseReason",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:127",
    ),
    unit_global(
        "UnitPlayerControlled",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:518",
    ),
    unit_global(
        "UnitPlayerOrPetInParty",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:154",
    ),
    unit_global(
        "UnitPlayerOrPetInRaid",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:159",
    ),
    unit_global(
        "UnitPosition",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/movement_probes.rs:103",
    ),
    unit_global(
        "UnitPower",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:608",
    ),
    unit_global(
        "UnitPowerBarID",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:615",
    ),
    unit_global(
        "UnitPowerBarTimerInfo",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/state_backed_queries.rs:36",
    ),
    unit_global(
        "UnitPowerDisplayMod",
        RETAIL_FOREVER_PROFILES,
        RETAIL_FOREVER_PROFILES,
        "src/lua_api/globals/real/unit_power_display_mod.rs (Retail); src/lua_api/workarounds/temporary/unit_power_display_mod.rs (WowForever)",
    ),
    unit_global(
        "UnitPowerMax",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:609",
    ),
    unit_global(
        "UnitPowerMissing",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:613",
    ),
    unit_global(
        "UnitPowerPercent",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:614",
    ),
    unit_global(
        "UnitPowerType",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/utility_system_spell/spell_api.rs:616",
    ),
    unit_global(
        "UnitRace",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/state_backed_queries.rs:32",
    ),
    unit_global(
        "UnitRangedAttack",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:640",
    ),
    unit_global(
        "UnitRangedAttackPower",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:633",
    ),
    unit_global(
        "UnitRangedCriticalStrike",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:635",
    ),
    unit_global(
        "UnitRangedDamage",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:638",
    ),
    unit_global(
        "UnitReaction",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:646",
    ),
    unit_global(
        "UnitRealmRelationship",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:118",
    ),
    unit_global(
        "UnitResistance",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:650",
    ),
    unit_global(
        "UnitSelectionColor",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:82",
    ),
    unit_global(
        "UnitSetRole",
        RETAIL_PTR_PROFILES,
        PTR_ONLY_PROFILE,
        "src/lua_api/globals/real/unit_roles_assignment.rs:158",
    ),
    unit_global(
        "UnitSetRoleEnum",
        RETAIL_PTR_PROFILES,
        PTR_ONLY_PROFILE,
        "src/lua_api/globals/real/unit_roles_assignment.rs:157",
    ),
    unit_global(
        "UnitSex",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/state_backed_queries.rs:33",
    ),
    unit_global(
        "UnitSexBase",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/state_backed_queries.rs:34",
    ),
    unit_global(
        "UnitShouldDisplaySpellTargetName",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/real/publication_unit_12_0_0.rs:133",
    ),
    unit_global(
        "UnitSpellHaste",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:636",
    ),
    unit_global(
        "UnitSpellTargetClass",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/real/publication_unit_12_0_0.rs:136",
    ),
    unit_global(
        "UnitSpellTargetName",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/real/unit_spell_target_name.rs:69",
    ),
    unit_global(
        "UnitStagger",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/workarounds/temporary/unit_stagger_defaults.rs:9",
    ),
    unit_global(
        "UnitStat",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:649",
    ),
    unit_global(
        "UnitTargetsVehicleInRaidUI",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:164",
    ),
    unit_global(
        "UnitThreatLeadSituation",
        RETAIL_PTR_PROFILES,
        RETAIL_PTR_PROFILES,
        "src/lua_api/globals/real/publication_unit_12_0_0.rs:137",
    ),
    unit_global(
        "UnitThreatPercentageOfLead",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/workarounds/temporary/unit_threat_defaults.rs:15",
    ),
    unit_global(
        "UnitThreatSituation",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:522",
    ),
    unit_global(
        "UnitTokenFromGUID",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_misc.rs:516",
    ),
    unit_global(
        "UnitTreatAsPlayerForDisplay",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/group_queries.rs:79",
    ),
    unit_global(
        "UnitTrialBankedLevels",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/xp_honor_rest.rs:149",
    ),
    unit_global(
        "UnitTrialXP",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/xp_honor_rest.rs:148",
    ),
    unit_global(
        "UnitUsesAmmo",
        FOREVER_ONLY_PROFILE,
        FOREVER_ONLY_PROFILE,
        "src/lua_api/globals/real/combat_stats.rs:463 (WowForever)",
    ),
    unit_global(
        "UnitVehicleSkin",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/real/vehicle_possession.rs:145",
    ),
    unit_global(
        "UnitWeaponAttackPower",
        RETAIL_ONLY_PROFILE,
        RETAIL_ONLY_PROFILE,
        "src/lua_api/globals/unit_stats.rs:655",
    ),
    unit_global(
        "UnitXP",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:647",
    ),
    unit_global(
        "UnitXPMax",
        ALL_PROFILES,
        ALL_PROFILES,
        "src/lua_api/globals/unit_stats.rs:648",
    ),
];

pub(crate) fn copy_legacy_unit_globals_to_secureenv(lua: &mut rilua::Lua) -> LuaResult<()> {
    let active_profile = crate::client_profile::ACTIVE;
    for entry in MASTER_SNAPSHOT_UNIT_GLOBALS {
        if !entry.compatibility_profiles.contains(&active_profile) {
            continue;
        }
        debug_assert!(
            entry.registered_profiles.contains(&active_profile),
            "compatibility profile has no source registration for {} ({})",
            entry.name,
            entry.registrar
        );
        let value = LuaApiMut::get_global_val(lua, entry.name);
        if matches!(&value, Val::Function(_)) {
            crate::lua_api::globals::security::set_secure_env_key_state(
                lua.state_mut(),
                entry.name,
                value,
            )?;
        }
    }
    Ok(())
}

#[cfg(all(
    test,
    any(
        feature = "client-retail",
        feature = "client-ptr",
        feature = "client-wrath",
        feature = "client-mists",
        feature = "client-era",
        feature = "client-anniversary",
        feature = "client-wowforever"
    )
))]
mod tests {
    use super::{ClientProfile, MASTER_SNAPSHOT_UNIT_GLOBALS};
    use crate::lua_api::WowLuaEnv;

    // This is a registrar-drift guard for ordinary _G, not a secureenv or native-publication assertion.
    fn assert_registered_globals_exist(profile: ClientProfile) {
        assert_eq!(crate::client_profile::ACTIVE, profile);
        let sources = missing_registered_global_sources(profile);
        assert!(
            sources.is_empty(),
            "registered Unit globals missing from _G for {profile:?}: {sources}"
        );
    }

    fn missing_registered_global_sources(profile: ClientProfile) -> String {
        let entries: Vec<_> = MASTER_SNAPSHOT_UNIT_GLOBALS
            .iter()
            .filter(|entry| entry.registered_profiles.contains(&profile))
            .collect();
        let script = build_missing_registered_globals_script(&entries);
        let env = WowLuaEnv::new().expect("initialize Lua environment");
        let missing_csv = env.eval::<String>(&script).expect("check _G registrations");
        let missing_names = missing_csv
            .split(',')
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>();
        registered_global_miss_sources(&entries, missing_names)
    }

    fn build_missing_registered_globals_script(entries: &[&super::MasterUnitGlobal]) -> String {
        let names = entries
            .iter()
            .map(|entry| format!("'{}'", entry.name))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            concat!(
                "local names = {{{names}}}\n",
                "local missing = {{}}\n",
                "for _, name in ipairs(names) do\n",
                "    if type(rawget(_G, name)) ~= 'function' then\n",
                "        missing[#missing + 1] = name\n",
                "    end\n",
                "end\n",
                "return table.concat(missing, ',')"
            ),
            names = names
        )
    }

    fn registered_global_miss_sources(
        entries: &[&super::MasterUnitGlobal],
        missing_names: Vec<&str>,
    ) -> String {
        missing_names
            .into_iter()
            .filter_map(|name| {
                entries
                    .iter()
                    .find(|entry| entry.name == name)
                    .map(|entry| format!("{} ({})", name, entry.registrar))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    #[cfg(feature = "client-retail")]
    #[test]
    fn retail_tagged_unit_globals_exist_in_global_environment() {
        assert_registered_globals_exist(ClientProfile::Retail);
    }

    #[cfg(feature = "client-ptr")]
    #[test]
    fn ptr_tagged_unit_globals_exist_in_global_environment() {
        assert_registered_globals_exist(ClientProfile::Ptr);
    }

    #[cfg(feature = "client-wrath")]
    #[test]
    fn wrath_tagged_unit_globals_exist_in_global_environment() {
        assert_registered_globals_exist(ClientProfile::Wrath);
    }

    #[cfg(feature = "client-mists")]
    #[test]
    fn mists_tagged_unit_globals_exist_in_global_environment() {
        assert_registered_globals_exist(ClientProfile::Mists);
    }

    #[cfg(feature = "client-era")]
    #[test]
    fn era_tagged_unit_globals_exist_in_global_environment() {
        assert_registered_globals_exist(ClientProfile::Era);
    }

    #[cfg(feature = "client-anniversary")]
    #[test]
    fn anniversary_tagged_unit_globals_exist_in_global_environment() {
        assert_registered_globals_exist(ClientProfile::Anniversary);
    }

    #[cfg(feature = "client-wowforever")]
    #[test]
    fn wowforever_tagged_unit_globals_exist_in_global_environment() {
        assert_registered_globals_exist(ClientProfile::WowForever);
    }
}
