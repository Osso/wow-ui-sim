//! Enum and constant globals: `Enum.*`, `Constants.*`, LE_* values.

use crate::client_profile::{ACTIVE_RETAIL_API_EPOCH, RetailApiEpoch};
use crate::lua_api::globals::enum_data::{EXPLICIT_ENUMS, FOREVER_SHARED_ENUMS, SEQUENTIAL_ENUMS};
use crate::lua_api::methods::{create_table, table_get, table_set};
use rilua::LuaApiMut;
use rilua::Val;

const MISSING_ENUMS_LUA: &str = include_str!("../globals/enum_data/missing_enums.lua");
const COMPAT_ENUMS_LUA: &str = include_str!("../globals/enum_data/compat_enums.lua");
const RETAIL_12_0_0_ENUM_OVERRIDES_LUA: &str = r#"
Enum.EncounterEventFlags = {
    Disabled = 1,
}
Enum.EncounterEventFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
}
Enum.CooldownViewerAlertEventType.OnAuraApplied = nil
Enum.CooldownViewerAlertEventType.OnAuraRemoved = nil
Enum.CooldownViewerAlertEventTypeMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 4,
}
Enum.CombatAudioAlertThrottle = {
    Sample = 0,
    PlayerHealth = 1,
    TargetHealth = 2,
    PlayerCast = 3,
    TargetCast = 4,
    PlayerResource1 = 5,
    PlayerResource2 = 6,
}
Enum.CombatAudioAlertThrottleMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
}
Enum.DamageMeterSpellDetailsDisplayType.Deaths = nil
Enum.DamageMeterSpellDetailsDisplayType.EnemyDamageTaken = nil
Enum.DamageMeterSpellDetailsDisplayTypeMeta.MaxValue = 2
Enum.DamageMeterSpellDetailsDisplayTypeMeta.NumValues = 3
Enum.DamageMeterStorageType.Deaths = nil
Enum.DamageMeterStorageType.EnemyDamageTaken = nil
Enum.DamageMeterStorageTypeMeta.MaxValue = 6
Enum.DamageMeterStorageTypeMeta.NumValues = 7
Enum.DamageMeterType.Deaths = nil
Enum.DamageMeterType.EnemyDamageTaken = nil
Enum.DamageMeterTypeMeta.MaxValue = 8
Enum.DamageMeterTypeMeta.NumValues = 9
Enum.DamageMeterVisibility.InGroup = nil
Enum.DamageMeterVisibilityMeta = {
    MinValue = 0,
    MaxValue = 2,
    NumValues = 3,
}
Enum.EditModeAccountSetting.ShowTotemActionBar = nil
Enum.EditModeAccountSettingMeta.MaxValue = 32
Enum.EditModeAccountSettingMeta.NumValues = 33
Enum.EditModeSystem.TotemActionBar = nil
Enum.EditModeUnitFrameSetting.BigDefensiveIconSize = nil
Enum.EditModeUnitFrameSettingMeta.MaxValue = 20
Enum.EditModeUnitFrameSettingMeta.NumValues = 21
Enum.EditModeEncounterEventsSetting.ShowTooltips = 8
Enum.EditModeEncounterEventsSetting.TooltipAnchor = nil
Enum.EditModeEncounterEventsSetting.ViewType = nil
Enum.EditModeEncounterEventsSetting.FlipHorizontally = nil
Enum.EditModeEncounterEventsSetting.BarWidth = nil
Enum.EditModeEncounterEventsSetting.Padding = nil
Enum.EditModeEncounterEventsSettingMeta = {
    MinValue = 0,
    MaxValue = 9,
    NumValues = 10,
}
Enum.EditModePersonalResourceDisplaySetting = {
    HideHealthAndPower = 0,
    OnlyShowInCombat = 1,
}
Enum.EditModePersonalResourceDisplaySettingMeta = {
    MinValue = 0,
    MaxValue = 1,
    NumValues = 2,
}
Enum.FrameTutorialAccount.TransmogTrialOfStyle = nil
Enum.FrameTutorialAccount.TransmogCustomSetsMigration = nil
Enum.FrameTutorialAccountMeta.MaxValue = 45
Enum.FrameTutorialAccountMeta.NumValues = 45
Enum.TooltipDataLineType = {
    None = 0,
    Blank = 1,
    UnitName = 2,
    GemSocket = 3,
    AzeriteEssenceSlot = 4,
    AzeriteEssencePower = 5,
    LearnableSpell = 6,
    UnitThreat = 7,
    QuestObjective = 8,
    AzeriteItemPowerDescription = 9,
    RuneforgeLegendaryPowerDescription = 10,
    SellPrice = 11,
    ProfessionCraftingQuality = 12,
    SpellName = 13,
    CurrencyTotal = 14,
    ItemEnchantmentPermanent = 15,
    UnitOwner = 16,
    QuestTitle = 17,
    QuestPlayer = 18,
    NestedBlock = 19,
    ItemBinding = 20,
    RestrictedRaceClass = 21,
    RestrictedFaction = 22,
    RestrictedSkill = 23,
    RestrictedPvPMedal = 24,
    RestrictedReputation = 25,
    RestrictedSpellKnown = 26,
    RestrictedLevel = 27,
    EquipSlot = 28,
    ItemName = 29,
    Separator = 30,
    ToyName = 31,
    ToyText = 32,
    ToyEffect = 33,
    ToyDuration = 34,
    RestrictedArena = 35,
    RestrictedBg = 36,
    ToyFlavorText = 37,
    ToyDescription = 38,
    ToySource = 39,
    GemSocketEnchantment = 40,
    ItemLevel = 41,
    ItemUpgradeLevel = 42,
    SpellPassive = 43,
    SpellDescription = 44,
}
Enum.TooltipDataLineTypeMeta = {
    MinValue = 0,
    MaxValue = 44,
    NumValues = 45,
}
Enum.TransmogOutfitDisplayType = {
    Unassigned = 0,
    Assigned = 1,
    Equipped = 2,
    Hidden = 3,
}
Enum.TransmogOutfitDisplayTypeMeta = {
    MinValue = 0,
    MaxValue = 3,
    NumValues = 4,
}
Enum.TransmogOutfitEntryFlags = {
    AutomaticallyAwardedOnLogin = 1,
    UseOverrideName = 2,
    OnlyAvailableDuringEvent = 4,
    SortedToTopOfList = 8,
    UseOverrideCostModifier = 16,
}
Enum.TransmogOutfitEntryFlagsMeta = {
    MinValue = 1,
    MaxValue = 16,
    NumValues = 5,
}
Enum.UICovenantDisplayInfoFlags = {
    DisplayCovenantAsJourney = 1,
    UseJourneyRewardTrack = 2,
}
Enum.UICovenantDisplayInfoFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
}
Enum.SecretAspect.Attributes = nil
Enum.SecretAspect.CooldownStyle = nil
Enum.SecretAspectMeta = {
    MaxValue = 262144,
    MinValue = 1,
    NumValues = 24,
}
"#;
const RETAIL_12_0_0_POST_COMPAT_ENUM_OVERRIDES_LUA: &str = r#"
if Enum.ExpansionLandingPageType then
    Enum.ExpansionLandingPageType.None = nil
    Enum.ExpansionLandingPageType.Dragonflight = nil
    Enum.ExpansionLandingPageType.WarWithin = nil
end
if Enum.ExpansionLandingPageTypeMeta then
    Enum.ExpansionLandingPageTypeMeta.MaxValue = nil
    Enum.ExpansionLandingPageTypeMeta.MinValue = nil
    Enum.ExpansionLandingPageTypeMeta.NumValues = nil
end
"#;
#[cfg(feature = "retail-12-1-5")]
const RETAIL_12_1_5_ENUM_OVERRIDES_LUA: &str = r#"
Enum.BagFlag.IgnoreSoulbound = nil
Enum.BagFlagMeta.NumValues = 27
Enum.CurioRarityMeta = {
    MaxValue = 5,
    MinValue = 1,
    NumValues = 5,
}
"#;

const MISSING_CONSTANTS_LUA: &str = include_str!("../globals/enum_data/missing_constants.lua");
const CONSTANTS_VALUES_LUA: &str = include_str!("../globals/enum_data/constants_values.lua");
const COMPAT_CONSTANTS_LUA: &str = include_str!("../globals/enum_data/compat_constants.lua");
const RETAIL_12_0_0_POST_COMPAT_CONSTANT_OVERRIDES_LUA: &str = r#"
LE_WORLD_ELAPSED_TIMER_TYPE_CHALLENGE_MODE = nil
LE_WORLD_ELAPSED_TIMER_TYPE_NONE = nil
LE_WORLD_ELAPSED_TIMER_TYPE_PROVING_GROUND = nil
"#;

pub(crate) fn init_enum_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    {
        let state = lua.state_mut();
        let enum_table = ensure_global_table(state, "Enum");
        for &(enum_name, entries) in EXPLICIT_ENUMS.iter().chain(FOREVER_SHARED_ENUMS.0) {
            let enum_values = ensure_table_field(state, enum_table, enum_name);
            for &(variant_name, value) in entries {
                table_set(state, enum_values, variant_name, Val::Num(value as f64));
            }
        }
        for &(enum_name, entries) in SEQUENTIAL_ENUMS.iter().chain(FOREVER_SHARED_ENUMS.1) {
            let enum_values = ensure_table_field(state, enum_table, enum_name);
            for (index, &variant_name) in entries.iter().enumerate() {
                table_set(state, enum_values, variant_name, Val::Num(index as f64));
            }
        }
    }
    lua.exec(MISSING_ENUMS_LUA)?;
    #[cfg(feature = "retail-12-1-5")]
    lua.exec(RETAIL_12_1_5_ENUM_OVERRIDES_LUA)?;
    if ACTIVE_RETAIL_API_EPOCH == RetailApiEpoch::Retail12_0_0 {
        lua.exec(RETAIL_12_0_0_ENUM_OVERRIDES_LUA)?;
    }
    lua.exec(COMPAT_ENUMS_LUA)?;
    #[cfg(feature = "client-wowforever")]
    crate::c_api::forever_edit_mode_enums::register(lua.state_mut());
    if ACTIVE_RETAIL_API_EPOCH == RetailApiEpoch::Retail12_0_0 {
        lua.exec(RETAIL_12_0_0_POST_COMPAT_ENUM_OVERRIDES_LUA)?;
    }
    #[cfg(feature = "on-update-modes")]
    crate::c_api::on_update_modes::register(lua.state_mut());
    #[cfg(feature = "retail-12-1-0")]
    {
        let state = lua.state_mut();
        let enum_table = ensure_global_table(state, "Enum");
        crate::c_api::c_unit_auras::register_sound_trigger_enum(state, enum_table);
    }
    lua.exec(
        r#"
        Constants = Constants or {}
        setmetatable(Constants, {
            __index = function(t, key)
                local value = {}
                rawset(t, key, value)
                return value
            end,
        })
        "#,
    )?;
    lua.exec(MISSING_CONSTANTS_LUA)?;
    lua.exec(CONSTANTS_VALUES_LUA)?;
    lua.exec(COMPAT_CONSTANTS_LUA)?;
    #[cfg(feature = "client-wowforever")]
    lua.exec(
        // Forever 1.60.1.69913 RaidMarkerConstantsDocumentation.lua.
        r#"
        Constants.RaidMarkerConsts = {
            MAX_RAID_TARGETS_USER = 8,
            MAX_RAID_TARGETS_RESTRICTED = 8,
            MAX_VALID_RAID_TARGETS = 0,
            MAX_RAID_MARKERS = 8,
        }
        "#,
    )?;
    if ACTIVE_RETAIL_API_EPOCH == RetailApiEpoch::Retail12_0_0 {
        lua.exec(RETAIL_12_0_0_POST_COMPAT_CONSTANT_OVERRIDES_LUA)?;
    }
    Ok(())
}

fn ensure_global_table(state: &mut rilua::vm::state::LuaState, key: &str) -> Val {
    ensure_table_field(state, Val::Table(state.global), key)
}

fn ensure_table_field(state: &mut rilua::vm::state::LuaState, parent: Val, key: &str) -> Val {
    let existing = table_get(state, parent, key);
    if matches!(existing, Val::Table(_)) {
        return existing;
    }
    let table = create_table(state);
    table_set(state, parent, key, table);
    table
}
