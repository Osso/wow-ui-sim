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
    let globals = lua.globals();

    // Single-unit functions that always return false
    let false_stubs: &[&str] = &[
        "UnitIsDeadOrGhost",
        "UnitIsDead",
        "UnitIsGhost",
        "UnitIsAFK",
        "UnitIsDND",
        "UnitIsTapDenied",
    ];
    for &name in false_stubs {
        globals.set(name, lua.create_function(|_, _unit: String| Ok(false))?)?;
    }

    // Single-unit functions that always return true
    globals.set(
        "UnitIsConnected",
        lua.create_function(|_, _unit: String| Ok(true))?,
    )?;
    globals.set(
        "UnitIsVisible",
        lua.create_function(|_, _unit: String| Ok(true))?,
    )?;

    // Unit comparison: player-specific checks
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

    // Two-unit relation stubs
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

    globals.set(
        "UnitInRange",
        lua.create_function(|_, _unit: String| Ok((true, true)))?,
    )?;

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
        lua.create_function(|_, _unit: String| Ok(100_000i32))?,
    )?;
    globals.set(
        "UnitHealthMax",
        lua.create_function(|_, _unit: String| Ok(100_000i32))?,
    )?;
    globals.set(
        "UnitPower",
        lua.create_function(|_, (_unit, _power_type): (String, Option<i32>)| Ok(50_000i32))?,
    )?;
    globals.set(
        "UnitPowerMax",
        lua.create_function(|_, (_unit, _power_type): (String, Option<i32>)| Ok(100_000i32))?,
    )?;
    globals.set(
        "UnitPowerType",
        lua.create_function(|lua, _unit: String| {
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

    // Individual aura query stubs
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

    // AuraUtil namespace
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
    globals.set("AuraUtil", aura_util)?;

    Ok(())
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
