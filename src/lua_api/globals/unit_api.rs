//! Unit-related WoW API functions.
//!
//! Contains functions for querying unit information like names, classes, races,
//! health, power, auras, and other unit state.

use mlua::{Lua, MultiValue, Result, Value};

/// Class data: (index, display_name, file_name).
const CLASS_DATA: &[(i32, &str, &str)] = &[
    (1, "Warrior", "WARRIOR"),
    (2, "Paladin", "PALADIN"),
    (3, "Hunter", "HUNTER"),
    (4, "Rogue", "ROGUE"),
    (5, "Priest", "PRIEST"),
    (6, "Death Knight", "DEATHKNIGHT"),
    (7, "Shaman", "SHAMAN"),
    (8, "Mage", "MAGE"),
    (9, "Warlock", "WARLOCK"),
    (10, "Monk", "MONK"),
    (11, "Druid", "DRUID"),
    (12, "Demon Hunter", "DEMONHUNTER"),
    (13, "Evoker", "EVOKER"),
];

/// Look up class name and file by 1-based index.
fn class_info_by_index(index: i32) -> (&'static str, &'static str) {
    CLASS_DATA
        .iter()
        .find(|(i, _, _)| *i == index)
        .map(|(_, name, file)| (*name, *file))
        .unwrap_or(("Unknown", "UNKNOWN"))
}

/// Resolve a unit name from a unit ID string.
fn resolve_unit_name(unit: &str) -> &'static str {
    match unit {
        "player" => "SimPlayer",
        _ => "SimUnit",
    }
}

/// Register unit-related global functions.
pub fn register_unit_api(lua: &Lua) -> Result<()> {
    register_identity_functions(lua)?;
    register_class_functions(lua)?;
    register_name_functions(lua)?;
    register_state_functions(lua)?;
    register_group_functions(lua)?;
    register_health_power_functions(lua)?;
    register_threat_functions(lua)?;
    register_classification_functions(lua)?;
    register_casting_functions(lua)?;
    register_aura_functions(lua)?;
    register_weapon_enchant_functions(lua)?;
    register_xp_functions(lua)?;
    register_pvp_vehicle_functions(lua)?;
    register_misc_unit_functions(lua)?;
    register_unit_combat_stat_functions(lua)?;
    Ok(())
}

/// Register UnitRace, UnitSex, UnitGUID, UnitLevel, UnitEffectiveLevel,
/// UnitExists, UnitFactionGroup.
fn register_identity_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "UnitRace",
        lua.create_function(|lua, _unit: String| {
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string("Human")?),
                Value::String(lua.create_string("Human")?),
            ]))
        })?,
    )?;

    globals.set(
        "UnitSex",
        lua.create_function(|_, _unit: String| Ok(2))?,
    )?;

    globals.set(
        "UnitGUID",
        lua.create_function(|lua, unit: String| {
            let guid = match unit.as_str() {
                "player" => "Player-0000-00000001",
                _ => "Creature-0000-00000000",
            };
            Ok(Value::String(lua.create_string(guid)?))
        })?,
    )?;

    globals.set(
        "UnitLevel",
        lua.create_function(|_, _unit: String| Ok(70))?,
    )?;

    globals.set(
        "UnitEffectiveLevel",
        lua.create_function(|_, _unit: String| Ok(70))?,
    )?;

    globals.set(
        "UnitExists",
        lua.create_function(|_, unit: String| {
            Ok(matches!(unit.as_str(), "player" | "target" | "pet"))
        })?,
    )?;

    globals.set(
        "UnitFactionGroup",
        lua.create_function(|lua, _unit: String| {
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string("Alliance")?),
                Value::String(lua.create_string("Alliance")?),
            ]))
        })?,
    )?;

    Ok(())
}

/// Register UnitClass, UnitClassBase, GetNumClasses, GetClassInfo,
/// LocalizedClassList.
fn register_class_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "UnitClass",
        lua.create_function(|lua, _unit: String| {
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string("Warrior")?),
                Value::String(lua.create_string("WARRIOR")?),
                Value::Integer(1),
            ]))
        })?,
    )?;

    globals.set(
        "UnitClassBase",
        lua.create_function(|lua, _unit: String| {
            Ok(Value::String(lua.create_string("WARRIOR")?))
        })?,
    )?;

    globals.set(
        "GetNumClasses",
        lua.create_function(|_, ()| Ok(CLASS_DATA.len() as i32))?,
    )?;

    globals.set(
        "GetClassInfo",
        lua.create_function(|lua, class_index: i32| {
            let (name, file) = class_info_by_index(class_index);
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::String(lua.create_string(file)?),
                Value::Integer(class_index as i64),
            ]))
        })?,
    )?;

    globals.set(
        "LocalizedClassList",
        lua.create_function(|lua, _is_female: Option<bool>| {
            let classes = lua.create_table()?;
            for &(_, name, file) in CLASS_DATA {
                classes.set(file, name)?;
            }
            Ok(classes)
        })?,
    )?;

    Ok(())
}

/// Register UnitName, UnitNameUnmodified, UnitFullName, GetUnitName.
fn register_name_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // UnitName(unit) -> name, nil
    globals.set(
        "UnitName",
        lua.create_function(|lua, unit: String| {
            let name = resolve_unit_name(&unit);
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::Nil,
            ]))
        })?,
    )?;

    // UnitNameUnmodified(unit) -> name, nil
    globals.set(
        "UnitNameUnmodified",
        lua.create_function(|lua, unit: String| {
            let name = resolve_unit_name(&unit);
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::Nil,
            ]))
        })?,
    )?;

    // UnitFullName(unit) -> name, realm
    globals.set(
        "UnitFullName",
        lua.create_function(|lua, unit: String| {
            let name = resolve_unit_name(&unit);
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::String(lua.create_string("SimRealm")?),
            ]))
        })?,
    )?;

    // GetUnitName(unit, showServerName) -> name
    globals.set(
        "GetUnitName",
        lua.create_function(|lua, (unit, _show_server): (String, Option<bool>)| {
            let name = resolve_unit_name(&unit);
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;

    // UnitPVPName(unit) -> name (returns PvP-formatted name with title)
    globals.set(
        "UnitPVPName",
        lua.create_function(|lua, unit: String| {
            let name = resolve_unit_name(&unit);
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;

    Ok(())
}

/// Register unit state boolean functions: alive/dead, AFK/DND, combat
/// relations, visibility.
fn register_state_functions(lua: &Lua) -> Result<()> {
    register_state_boolean_stubs(lua)?;
    register_state_comparisons(lua)?;
    register_state_relations(lua)
}

/// Register single-unit boolean stubs (always false or always true).
fn register_state_boolean_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let false_stubs: &[&str] = &[
        "UnitIsDeadOrGhost",
        "UnitIsDead",
        "UnitIsGhost",
        "UnitIsAFK",
        "UnitIsDND",
        "UnitIsTapDenied",
        "UnitIsCorpse",
        "UnitIsWildBattlePet",
        "UnitIsBattlePetCompanion",
    ];
    for &name in false_stubs {
        globals.set(name, lua.create_function(|_, _unit: String| Ok(false))?)?;
    }
    globals.set("UnitIsConnected", lua.create_function(|_, _unit: String| Ok(true))?)?;
    globals.set("UnitIsVisible", lua.create_function(|_, _unit: String| Ok(true))?)?;
    globals.set("UnitBattlePetLevel", lua.create_function(|_, _unit: String| Ok(0))?)?;
    Ok(())
}

/// Register unit comparison functions (player checks, unit identity).
fn register_state_comparisons(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set(
        "UnitIsPlayer",
        lua.create_function(|_, unit: String| Ok(unit == "player"))?,
    )?;
    globals.set(
        "UnitPlayerControlled",
        lua.create_function(|_, unit: String| Ok(unit == "player" || unit == "pet"))?,
    )?;
    globals.set(
        "UnitIsUnit",
        lua.create_function(|_, (unit1, unit2): (String, String)| Ok(unit1 == unit2))?,
    )?;
    Ok(())
}

/// Register two-unit relation stubs (enemy, friend, attack, assist, range).
fn register_state_relations(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let two_unit_false: &[&str] = &["UnitIsEnemy", "UnitCanAttack"];
    for &name in two_unit_false {
        globals.set(
            name,
            lua.create_function(|_, (_u1, _u2): (String, String)| Ok(false))?,
        )?;
    }
    let two_unit_true: &[&str] = &["UnitIsFriend", "UnitCanAssist"];
    for &name in two_unit_true {
        globals.set(
            name,
            lua.create_function(|_, (_u1, _u2): (String, String)| Ok(true))?,
        )?;
    }
    globals.set("UnitInRange", lua.create_function(|_, _unit: String| Ok((true, true)))?)?;
    Ok(())
}

/// Register UnitInParty, UnitInRaid, UnitIsGroupLeader, UnitIsGroupAssistant.
fn register_group_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "UnitInParty",
        lua.create_function(|_, _unit: String| Ok(false))?,
    )?;
    globals.set(
        "UnitInRaid",
        lua.create_function(|_, _unit: String| Ok(Value::Nil))?,
    )?;
    globals.set(
        "UnitIsGroupLeader",
        lua.create_function(|_, _unit: String| Ok(false))?,
    )?;
    globals.set(
        "UnitIsGroupAssistant",
        lua.create_function(|_, _unit: String| Ok(false))?,
    )?;

    Ok(())
}

/// Register UnitHealth, UnitHealthMax, UnitPower, UnitPowerMax, UnitPowerType,
/// UnitGetIncomingHeals, UnitGetTotalAbsorbs, UnitGetTotalHealAbsorbs.
fn register_health_power_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "UnitHealth",
        lua.create_function(|_, _unit: Value| Ok(100_000i32))?,
    )?;
    globals.set(
        "UnitHealthMax",
        lua.create_function(|_, _unit: Value| Ok(100_000i32))?,
    )?;
    globals.set(
        "UnitPower",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(50_000i32))?,
    )?;
    globals.set(
        "UnitPowerMax",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(100_000i32))?,
    )?;
    globals.set(
        "UnitPowerType",
        lua.create_function(|lua, _unit: Value| {
            Ok((0i32, Value::String(lua.create_string("MANA")?)))
        })?,
    )?;
    globals.set(
        "UnitGetIncomingHeals",
        lua.create_function(|_, (_unit, _healer): (String, Option<String>)| Ok(0i32))?,
    )?;
    globals.set(
        "UnitGetTotalAbsorbs",
        lua.create_function(|_, _unit: String| Ok(0i32))?,
    )?;
    globals.set(
        "UnitGetTotalHealAbsorbs",
        lua.create_function(|_, _unit: String| Ok(0i32))?,
    )?;

    Ok(())
}

/// Register UnitThreatSituation, UnitDetailedThreatSituation.
fn register_threat_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "UnitThreatSituation",
        lua.create_function(|_, (_unit, _mob): (String, Option<String>)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "UnitDetailedThreatSituation",
        lua.create_function(|_, (_unit, _mob): (String, Option<String>)| {
            Ok((false, 0i32, 0.0f64, 0.0f64, 0i32))
        })?,
    )?;

    Ok(())
}

/// Register UnitClassification, UnitCreatureType, UnitCreatureFamily.
fn register_classification_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "UnitClassification",
        lua.create_function(|lua, _unit: String| {
            Ok(Value::String(lua.create_string("normal")?))
        })?,
    )?;
    globals.set(
        "UnitCreatureType",
        lua.create_function(|lua, _unit: String| {
            Ok(Value::String(lua.create_string("Humanoid")?))
        })?,
    )?;
    globals.set(
        "UnitCreatureFamily",
        lua.create_function(|_, _unit: String| Ok(Value::Nil))?,
    )?;

    Ok(())
}

/// Register UnitCastingInfo, UnitChannelInfo.
fn register_casting_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "UnitCastingInfo",
        lua.create_function(|_, _unit: String| Ok(Value::Nil))?,
    )?;
    globals.set(
        "UnitChannelInfo",
        lua.create_function(|_, _unit: String| Ok(Value::Nil))?,
    )?;

    Ok(())
}

/// Register UnitAura, UnitBuff, UnitDebuff, GetPlayerAuraBySpellID,
/// and the AuraUtil namespace.
fn register_aura_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    let aura_stubs: &[&str] = &["UnitAura", "UnitBuff", "UnitDebuff"];
    for &name in aura_stubs {
        globals.set(
            name,
            lua.create_function(
                |_, (_unit, _index, _filter): (String, i32, Option<String>)| Ok(Value::Nil),
            )?,
        )?;
    }

    globals.set(
        "GetPlayerAuraBySpellID",
        lua.create_function(|_, _spell_id: i32| Ok(Value::Nil))?,
    )?;

    globals.set("AuraUtil", register_aura_util(lua)?)?;
    Ok(())
}

/// AuraUtil namespace stubs.
fn register_aura_util(lua: &Lua) -> Result<mlua::Table> {
    let aura_util = lua.create_table()?;
    aura_util.set(
        "ForEachAura",
        lua.create_function(
            |_,
             (_unit, _filter, _max, _cb, _packed): (
                String,
                String,
                Option<i32>,
                mlua::Function,
                Option<bool>,
            )| { Ok(()) },
        )?,
    )?;
    aura_util.set(
        "FindAura",
        lua.create_function(
            |_,
             (_pred, _unit, _filter, _spell, _caster): (
                mlua::Function,
                String,
                String,
                Option<i32>,
                Option<String>,
            )| Ok(Value::Nil),
        )?,
    )?;
    aura_util.set(
        "UnpackAuraData",
        lua.create_function(|_, _aura_data: Value| Ok(Value::Nil))?,
    )?;
    aura_util.set(
        "FindAuraByName",
        lua.create_function(|_, (_name, _unit, _filter): (String, String, String)| {
            Ok(Value::Nil)
        })?,
    )?;
    Ok(aura_util)
}

/// Register GetWeaponEnchantInfo.
fn register_weapon_enchant_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetWeaponEnchantInfo",
        lua.create_function(|_, ()| {
            Ok((false, 0i32, 0i32, 0i32, false, 0i32, 0i32, 0i32))
        })?,
    )?;

    Ok(())
}

/// Register UnitXP, UnitXPMax, UnitTrialXP, GetXPExhaustion, GetRestState.
fn register_xp_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("UnitXP", lua.create_function(|_, _unit: String| Ok(5000i32))?)?;
    globals.set("UnitXPMax", lua.create_function(|_, _unit: String| Ok(10000i32))?)?;
    globals.set("UnitTrialXP", lua.create_function(|_, _unit: String| Ok(0i32))?)?;
    globals.set("GetXPExhaustion", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetRestState", lua.create_function(|_, ()| Ok(1i32))?)?;
    Ok(())
}

/// Register PvP and vehicle-related unit functions.
fn register_pvp_vehicle_functions(lua: &Lua) -> Result<()> {
    let g = lua.globals();

    let false_unit_stubs: &[&str] = &[
        "UnitIsPVP",
        "UnitIsPVPFreeForAll",
        "UnitIsMercenary",
        "UnitInVehicle",
        "UnitHasVehiclePlayerFrameUI",
        "UnitInVehicleHidesPetFrame",
        "UnitAffectingCombat",
        "UnitInCombat",
        "UnitInPartyIsAI",
    ];
    for &name in false_unit_stubs {
        g.set(name, lua.create_function(|_, _unit: String| Ok(false))?)?;
    }

    g.set("UnitHonorLevel", lua.create_function(|_, _unit: String| Ok(0i32))?)?;
    g.set("UnitPartialPower", lua.create_function(|_, (_unit, _pt): (String, Option<i32>)| Ok(0i32))?)?;

    // UnitGroupRolesAssignedEnum -> nil
    g.set("UnitGroupRolesAssignedEnum", lua.create_function(|_, _unit: String| Ok(Value::Nil))?)?;

    // UnitRealmRelationship -> nil
    g.set("UnitRealmRelationship", lua.create_function(|_, _unit: String| Ok(Value::Nil))?)?;

    // UnitSelectionColor -> r, g, b, a
    g.set("UnitSelectionColor", lua.create_function(|_, _unit: String| {
        Ok((1.0f64, 1.0f64, 1.0f64, 1.0f64))
    })?)?;

    Ok(())
}

/// Register miscellaneous unit query functions.
fn register_misc_unit_functions(lua: &Lua) -> Result<()> {
    let g = lua.globals();

    g.set("UnitPhaseReason", lua.create_function(|_, _unit: String| Ok(Value::Nil))?)?;
    g.set("UnitIsOwnerOrControllerOfUnit", lua.create_function(|_, (_u1, _u2): (String, String)| Ok(false))?)?;
    g.set("UnitIsWarModePhased", lua.create_function(|_, _unit: String| Ok(false))?)?;
    g.set("UnitIsWarModeDesired", lua.create_function(|_, _unit: String| Ok(false))?)?;
    g.set("UnitIsWarModeActive", lua.create_function(|_, _unit: String| Ok(false))?)?;
    g.set("UnitHasMana", lua.create_function(|_, _unit: Value| Ok(true))?)?;
    g.set("UnitHasRelicSlot", lua.create_function(|_, _unit: Value| Ok(false))?)?;
    g.set("IsActiveBattlefieldArena", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetUnitPowerBarInfo", lua.create_function(|_, _unit: Value| Ok(Value::Nil))?)?;
    g.set("IsInGroup", lua.create_function(|_, _flags: Option<i32>| Ok(false))?)?;
    g.set("IsInRaid", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("UnitStagger", lua.create_function(|_, _unit: Value| Ok(0i32))?)?;

    Ok(())
}

/// Unit combat stat functions for PaperDollFrame.
fn register_unit_combat_stat_functions(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    // UnitArmor(unit) -> base, effectiveArmor, armor, posBuff, negBuff
    g.set("UnitArmor", lua.create_function(|_, _unit: Value| {
        Ok((0i32, 0i32, 0i32, 0i32, 0i32))
    })?)?;
    // UnitDamage(unit) -> minDmg, maxDmg, minOff, maxOff, posPhys, negPhys, pct
    g.set("UnitDamage", lua.create_function(|_, _unit: Value| {
        Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, 100.0_f64))
    })?)?;
    // UnitAttackPower(unit) -> base, posBuff, negBuff
    g.set("UnitAttackPower", lua.create_function(|_, _unit: Value| {
        Ok((0i32, 0i32, 0i32))
    })?)?;
    // UnitRangedAttackPower(unit) -> base, posBuff, negBuff
    g.set("UnitRangedAttackPower", lua.create_function(|_, _unit: Value| {
        Ok((0i32, 0i32, 0i32))
    })?)?;
    register_unit_combat_stat_functions_2(lua)?;
    Ok(())
}

/// Additional unit combat stat functions.
fn register_unit_combat_stat_functions_2(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    // UnitAttackSpeed(unit) -> mainSpeed, offSpeed
    g.set("UnitAttackSpeed", lua.create_function(|_, _unit: Value| {
        Ok((2.0_f64, 2.0_f64))
    })?)?;
    // UnitRangedDamage(unit) -> speed, minDmg, maxDmg, posPhys, negPhys, pct
    g.set("UnitRangedDamage", lua.create_function(|_, _unit: Value| {
        Ok((2.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, 100.0_f64))
    })?)?;
    g.set("GetShapeshiftFormInfo", lua.create_function(|_, _idx: Value| {
        Ok((Value::Nil, false, false, 0i32))
    })?)?;
    g.set("GetShapeshiftFormID", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("GetPetActionInfo", lua.create_function(|_, _idx: Value| {
        Ok((Value::Nil, Value::Nil, Value::Nil, false, false))
    })?)?;
    Ok(())
}
