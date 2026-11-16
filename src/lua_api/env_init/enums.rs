//! Enum and constant globals: `Enum.*`, `Constants.*`, LE_* values.

use crate::client_profile::{ACTIVE_RETAIL_API_EPOCH, RetailApiEpoch};
use crate::lua_api::globals::enum_data::{
    COMBAT_AUDIO_PULSE_ENUMS, EXPLICIT_ENUMS, FOREVER_SHARED_ENUMS, SEQUENTIAL_ENUMS,
};
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
-- PingConstantsDocumentation.lua:37-48
Enum.PingSetTargetState = {
    Ok = 0,
    Failed = 1,
    FailedSilent = 2,
}
Enum.PingSetTargetStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
}
"#;

const MISSING_CONSTANTS_LUA: &str = include_str!("../globals/enum_data/missing_constants.lua");
const CONSTANTS_VALUES_LUA: &str = include_str!("../globals/enum_data/constants_values.lua");
const COMPAT_CONSTANTS_LUA: &str = include_str!("../globals/enum_data/compat_constants.lua");
// R88n.10 verifies these native-installed Retail values in both environments.
const NATIVE_CONSTANTS_LUA: &str = r#"
Constants = Constants or {}
local roles = rawget(Constants, "LFG_ROLEConstants")
if type(roles) ~= "table" then
    roles = {}
    rawset(Constants, "LFG_ROLEConstants", roles)
end
roles.LFG_ROLE_ANY = 3
roles.LFG_ROLE_NO_ROLE = -1
"#;
const RETAIL_12_0_0_POST_COMPAT_CONSTANT_OVERRIDES_LUA: &str = r#"
LE_WORLD_ELAPSED_TIMER_TYPE_CHALLENGE_MODE = nil
LE_WORLD_ELAPSED_TIMER_TYPE_NONE = nil
LE_WORLD_ELAPSED_TIMER_TYPE_PROVING_GROUND = nil
"#;

pub(crate) fn init_enum_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_enum_data(lua)?;
    register_enum_compatibility(lua)?;
    register_profile_enum_values(lua)?;
    register_native_constants(lua)?;
    register_base_constants(lua)?;
    register_ptr_constant_overrides(lua)?;
    register_compat_constant_values(lua)?;
    publish_retail_account_store_constants(lua)?;

    Ok(())
}

fn register_enum_data(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_explicit_enum_values(lua)?;
    register_sequential_enum_values(lua)?;
    register_ptr_vc_init_state(lua)
}

fn register_explicit_enum_values(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();
    let enum_table = ensure_global_table(state, "Enum");
    for &(enum_name, entries) in EXPLICIT_ENUMS
        .iter()
        .chain(FOREVER_SHARED_ENUMS.0)
        .chain(COMBAT_AUDIO_PULSE_ENUMS)
    {
        let enum_values = ensure_table_field(state, enum_table, enum_name);
        for &(variant_name, value) in entries {
            crate::c_api::set_namespace_member(
                state,
                enum_values,
                variant_name,
                Val::Num(value as f64),
            )?;
        }
    }
    Ok(())
}

fn register_sequential_enum_values(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();
    let enum_table = ensure_global_table(state, "Enum");
    for &(enum_name, entries) in SEQUENTIAL_ENUMS.iter().chain(FOREVER_SHARED_ENUMS.1) {
        let enum_values = ensure_table_field(state, enum_table, enum_name);
        for (index, &variant_name) in entries.iter().enumerate() {
            crate::c_api::set_namespace_member(
                state,
                enum_values,
                variant_name,
                Val::Num(index as f64),
            )?;
        }
    }
    Ok(())
}

fn register_ptr_vc_init_state(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Ptr {
        return Ok(());
    }
    let state = lua.state_mut();
    let enum_table = ensure_global_table(state, "Enum");
    let (name, entries) = crate::lua_api::globals::enum_data::VC_INIT_STATE;
    let values = ensure_table_field(state, enum_table, name);
    for &(variant, value) in entries {
        crate::c_api::set_namespace_member(state, values, variant, Val::Num(value as f64))?;
    }
    Ok(())
}

fn register_enum_compatibility(lua: &mut rilua::Lua) -> crate::Result<()> {
    exec_source_in_both_environments(lua, MISSING_ENUMS_LUA)?;
    #[cfg(feature = "retail-12-1-5")]
    exec_source_in_both_environments(lua, RETAIL_12_1_5_ENUM_OVERRIDES_LUA)?;
    if ACTIVE_RETAIL_API_EPOCH == RetailApiEpoch::Retail12_0_0 {
        exec_source_in_both_environments(lua, RETAIL_12_0_0_ENUM_OVERRIDES_LUA)?;
    }
    exec_source_in_both_environments(lua, COMPAT_ENUMS_LUA)
}

fn register_profile_enum_values(lua: &mut rilua::Lua) -> crate::Result<()> {
    crate::c_api::ptr_edit_mode_enums::register(lua.state_mut());
    #[cfg(feature = "retail-12-0-5")]
    crate::c_api::patch_12_0_5_enums::register(lua.state_mut());
    #[cfg(feature = "client-wowforever")]
    {
        crate::c_api::forever_edit_mode_enums::register(lua.state_mut());
        crate::c_api::forever_finite_constants::register_player_swing_type(lua.state_mut());
    }
    if ACTIVE_RETAIL_API_EPOCH == RetailApiEpoch::Retail12_0_0 {
        exec_source_in_both_environments(lua, RETAIL_12_0_0_POST_COMPAT_ENUM_OVERRIDES_LUA)?;
    }
    #[cfg(feature = "on-update-modes")]
    crate::c_api::on_update_modes::register(lua.state_mut());
    #[cfg(feature = "aura-instance-enumeration")]
    {
        let state = lua.state_mut();
        let enum_table = ensure_global_table(state, "Enum");
        crate::c_api::c_unit_auras::register_sound_trigger_enum(state, enum_table);
    }
    Ok(())
}

fn register_native_constants(lua: &mut rilua::Lua) -> crate::Result<()> {
    exec_source_in_both_environments(lua, NATIVE_CONSTANTS_LUA)
}

fn register_base_constants(lua: &mut rilua::Lua) -> crate::Result<()> {
    const LAZY_CONSTANT_TABLES_LUA: &str = r#"
        setmetatable(Constants, {
            __index = function(t, key)
                local value = {}
                rawset(t, key, value)
                return value
            end,
        })
        "#;
    // R88t.t2 publishes native Constants.* tables without member flags. Keep
    // missing and compatibility datasets ordinary-only.
    let publish_generated_constants = crate::client_profile::is_active_retail_12_1_5();
    if publish_generated_constants {
        exec_source_in_both_environments(lua, LAZY_CONSTANT_TABLES_LUA)?;
    } else {
        lua.exec(LAZY_CONSTANT_TABLES_LUA)?;
    }
    lua.exec(MISSING_CONSTANTS_LUA)?;
    #[cfg(feature = "client-wowforever")]
    lua.exec("LE_UNIT_STAT_SPIRIT = 5")?;
    lua.exec(CONSTANTS_VALUES_LUA)?;
    if publish_generated_constants {
        exec_source_in_secure_environment(lua, CONSTANTS_VALUES_LUA)?;
        // The metatable above creates the generated Constants.* tables. Do not
        // leave it as a secure lookup fallback after the constants are installed.
        exec_source_in_secure_environment(lua, "setmetatable(Constants, nil)")?;
    }
    Ok(())
}

fn register_ptr_constant_overrides(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Ptr {
        return Ok(());
    }
    // PTR 12.1.7.70493 LevelConstantsDocumentation.lua:12; Retail omits this constant.
    lua.exec("Constants.LevelConstsExposed.MIN_TALENT_LEVEL = 10")?;
    // PTR CatalogShopConstantsDocumentation.lua:89.
    lua.exec("Constants.CatalogShopVirtualCurrencyConstants.TALEBOUND_VC_CURRENCY_CODE = 'XZC'")?;
    Ok(())
}

fn register_compat_constant_values(lua: &mut rilua::Lua) -> crate::Result<()> {
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

// R88t verifies direct, no-flag publication of only these AccountStoreConsts values.
fn publish_retail_account_store_constants(lua: &mut rilua::Lua) -> crate::Result<()> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(());
    }

    const MEMBERS: &[&str] = &[
        "PlunderstormStoreFrontID",
        "WowhackStoreFrontID",
        "PlunderstormPlunderCurrencyID",
        "KegLegRenownCurrencyID",
    ];

    let state = lua.state_mut();
    let constants = ensure_global_table(state, "Constants");
    let Val::Table(account_store) = ensure_table_field(state, constants, "AccountStoreConsts")
    else {
        unreachable!("AccountStoreConsts must be a table");
    };
    for member in MEMBERS {
        let value = crate::lua_api::methods::table_get(state, Val::Table(account_store), member);
        if value.is_nil() {
            return Err(crate::Error::Other(format!(
                "missing ordinary Constants.AccountStoreConsts.{member} value"
            )));
        }
        crate::c_api::set_namespace_member(state, Val::Table(account_store), member, value)?;
    }
    Ok(())
}

fn exec_source_in_both_environments(lua: &mut rilua::Lua, source: &str) -> crate::Result<()> {
    lua.exec(source)?;
    exec_source_in_secure_environment(lua, source)
}

fn exec_source_in_secure_environment(lua: &mut rilua::Lua, source: &str) -> crate::Result<()> {
    use rilua::LuaApiMut;

    let chunk = LuaApiMut::load(lua, source)?;
    crate::lua_api::globals::security::mark_secure(lua, &chunk)?;
    lua.call_function(&chunk, &[])?;
    Ok(())
}

fn ensure_global_table(state: &mut rilua::vm::state::LuaState, key: &'static str) -> Val {
    Val::Table(
        crate::c_api::ensure_namespace(state, key)
            .expect("secure environment must exist before Enum registration"),
    )
}

fn ensure_table_field(state: &mut rilua::vm::state::LuaState, parent: Val, key: &str) -> Val {
    let Val::Table(parent_ref) = parent else {
        unreachable!("Enum parent must be a table");
    };
    Val::Table(
        crate::c_api::ensure_namespace_table_field(state, parent_ref, key)
            .expect("Enum parent must have a secure registration peer"),
    )
}
