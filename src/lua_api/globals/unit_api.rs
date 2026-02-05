//! Unit-related WoW API functions.
//!
//! Contains functions for querying unit information like names, classes, races,
//! health, power, auras, and other unit state.

use mlua::{Lua, Result, Value};

/// Register unit-related global functions.
pub fn register_unit_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // UnitRace(unit) - Return race name and file
    globals.set(
        "UnitRace",
        lua.create_function(|lua, _unit: String| {
            // Return: raceName, raceFile
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string("Human")?),
                Value::String(lua.create_string("Human")?),
            ]))
        })?,
    )?;

    // UnitSex(unit) - Return sex ID
    globals.set(
        "UnitSex",
        lua.create_function(|_, _unit: String| {
            // Return: 2 for male, 3 for female (matches Enum.UnitSex)
            Ok(2)
        })?,
    )?;

    // UnitClass(unit) - Return class info
    globals.set(
        "UnitClass",
        lua.create_function(|lua, _unit: String| {
            // Return: className, classFile, classID
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string("Warrior")?),
                Value::String(lua.create_string("WARRIOR")?),
                Value::Integer(1),
            ]))
        })?,
    )?;

    // UnitClassBase(unit) - Returns class file name only (no localization)
    globals.set(
        "UnitClassBase",
        lua.create_function(|lua, _unit: String| {
            Ok(Value::String(lua.create_string("WARRIOR")?))
        })?,
    )?;

    // GetNumClasses() - Returns number of playable classes
    globals.set("GetNumClasses", lua.create_function(|_, ()| Ok(13i32))?)?;

    // GetClassInfo(classIndex) - Returns className, classFile, classID
    globals.set(
        "GetClassInfo",
        lua.create_function(|lua, class_index: i32| {
            let (name, file) = match class_index {
                1 => ("Warrior", "WARRIOR"),
                2 => ("Paladin", "PALADIN"),
                3 => ("Hunter", "HUNTER"),
                4 => ("Rogue", "ROGUE"),
                5 => ("Priest", "PRIEST"),
                6 => ("Death Knight", "DEATHKNIGHT"),
                7 => ("Shaman", "SHAMAN"),
                8 => ("Mage", "MAGE"),
                9 => ("Warlock", "WARLOCK"),
                10 => ("Monk", "MONK"),
                11 => ("Druid", "DRUID"),
                12 => ("Demon Hunter", "DEMONHUNTER"),
                13 => ("Evoker", "EVOKER"),
                _ => ("Unknown", "UNKNOWN"),
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::String(lua.create_string(file)?),
                Value::Integer(class_index as i64),
            ]))
        })?,
    )?;

    // LocalizedClassList(isFemale) - Returns table mapping classFile to localized name
    globals.set(
        "LocalizedClassList",
        lua.create_function(|lua, _is_female: Option<bool>| {
            let classes = lua.create_table()?;
            classes.set("WARRIOR", "Warrior")?;
            classes.set("PALADIN", "Paladin")?;
            classes.set("HUNTER", "Hunter")?;
            classes.set("ROGUE", "Rogue")?;
            classes.set("PRIEST", "Priest")?;
            classes.set("DEATHKNIGHT", "Death Knight")?;
            classes.set("SHAMAN", "Shaman")?;
            classes.set("MAGE", "Mage")?;
            classes.set("WARLOCK", "Warlock")?;
            classes.set("MONK", "Monk")?;
            classes.set("DRUID", "Druid")?;
            classes.set("DEMONHUNTER", "Demon Hunter")?;
            classes.set("EVOKER", "Evoker")?;
            Ok(classes)
        })?,
    )?;

    // UnitName(unit) - Return name and realm
    globals.set(
        "UnitName",
        lua.create_function(|lua, unit: String| {
            let name = match unit.as_str() {
                "player" => "SimPlayer",
                _ => "SimUnit",
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::Nil,
            ]))
        })?,
    )?;

    // UnitNameUnmodified(unit) - Return raw name (used for BattleTag lookups)
    globals.set(
        "UnitNameUnmodified",
        lua.create_function(|lua, unit: String| {
            let name = match unit.as_str() {
                "player" => "SimPlayer",
                _ => "SimUnit",
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::Nil,
            ]))
        })?,
    )?;

    // UnitFullName(unit) - Return name with realm
    globals.set(
        "UnitFullName",
        lua.create_function(|lua, unit: String| {
            let name = match unit.as_str() {
                "player" => "SimPlayer",
                _ => "SimUnit",
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::String(lua.create_string("SimRealm")?),
            ]))
        })?,
    )?;

    // GetUnitName(unit, showServerName) - alias for UnitName with server name option
    globals.set(
        "GetUnitName",
        lua.create_function(|lua, (unit, _show_server): (String, Option<bool>)| {
            let name = match unit.as_str() {
                "player" => "SimPlayer",
                _ => "SimUnit",
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;

    // UnitGUID(unit) - Return unit GUID
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

    // UnitLevel(unit) - Return unit level
    globals.set(
        "UnitLevel",
        lua.create_function(|_, _unit: String| Ok(70))?,
    )?;

    // UnitEffectiveLevel(unit) - Return effective level (after scaling)
    globals.set(
        "UnitEffectiveLevel",
        lua.create_function(|_, _unit: String| Ok(70))?,
    )?;

    // UnitExists(unit) - Check if unit exists
    globals.set(
        "UnitExists",
        lua.create_function(|_, unit: String| {
            Ok(matches!(unit.as_str(), "player" | "target" | "pet"))
        })?,
    )?;

    // UnitFactionGroup(unit) - Return faction
    globals.set(
        "UnitFactionGroup",
        lua.create_function(|lua, _unit: String| {
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string("Alliance")?),
                Value::String(lua.create_string("Alliance")?),
            ]))
        })?,
    )?;

    // Unit state functions
    globals.set(
        "UnitIsDeadOrGhost",
        lua.create_function(|_, _unit: String| Ok(false))?,
    )?;
    globals.set(
        "UnitIsDead",
        lua.create_function(|_, _unit: String| Ok(false))?,
    )?;
    globals.set(
        "UnitIsGhost",
        lua.create_function(|_, _unit: String| Ok(false))?,
    )?;
    globals.set(
        "UnitIsAFK",
        lua.create_function(|_, _unit: String| Ok(false))?,
    )?;
    globals.set(
        "UnitIsDND",
        lua.create_function(|_, _unit: String| Ok(false))?,
    )?;
    globals.set(
        "UnitIsConnected",
        lua.create_function(|_, _unit: String| Ok(true))?,
    )?;
    globals.set(
        "UnitIsPlayer",
        lua.create_function(|_, unit: String| Ok(unit == "player"))?,
    )?;
    globals.set(
        "UnitPlayerControlled",
        lua.create_function(|_, unit: String| Ok(unit == "player" || unit == "pet"))?,
    )?;
    globals.set(
        "UnitIsTapDenied",
        lua.create_function(|_, _unit: String| Ok(false))?,
    )?;
    globals.set(
        "UnitIsEnemy",
        lua.create_function(|_, (_unit1, _unit2): (String, String)| Ok(false))?,
    )?;
    globals.set(
        "UnitIsFriend",
        lua.create_function(|_, (_unit1, _unit2): (String, String)| Ok(true))?,
    )?;
    globals.set(
        "UnitCanAttack",
        lua.create_function(|_, (_unit1, _unit2): (String, String)| Ok(false))?,
    )?;
    globals.set(
        "UnitCanAssist",
        lua.create_function(|_, (_unit1, _unit2): (String, String)| Ok(true))?,
    )?;
    globals.set(
        "UnitIsUnit",
        lua.create_function(|_, (unit1, unit2): (String, String)| Ok(unit1 == unit2))?,
    )?;
    globals.set(
        "UnitIsVisible",
        lua.create_function(|_, _unit: String| Ok(true))?,
    )?;
    globals.set(
        "UnitInRange",
        lua.create_function(|_, _unit: String| Ok((true, true)))?,
    )?;

    // Group/party functions
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

    // Health/Power functions
    globals.set(
        "UnitHealth",
        lua.create_function(|_, _unit: String| Ok(100000i32))?,
    )?;
    globals.set(
        "UnitHealthMax",
        lua.create_function(|_, _unit: String| Ok(100000i32))?,
    )?;
    globals.set(
        "UnitPower",
        lua.create_function(|_, (_unit, _power_type): (String, Option<i32>)| Ok(50000i32))?,
    )?;
    globals.set(
        "UnitPowerMax",
        lua.create_function(|_, (_unit, _power_type): (String, Option<i32>)| Ok(100000i32))?,
    )?;
    globals.set(
        "UnitPowerType",
        lua.create_function(|lua, _unit: String| {
            // Returns: powerType, powerToken
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

    // Threat functions
    globals.set(
        "UnitThreatSituation",
        lua.create_function(|_, (_unit, _mob_unit): (String, Option<String>)| {
            // Returns: 0=none, 1=other tanks, 2=your threat, 3=tanking
            Ok(Value::Nil)
        })?,
    )?;
    globals.set(
        "UnitDetailedThreatSituation",
        lua.create_function(
            |_, (_unit, _mob_unit): (String, Option<String>)| {
                // Returns: isTanking, status, scaledPercent, rawPercent, threatValue
                Ok((false, 0i32, 0.0f64, 0.0f64, 0i32))
            },
        )?,
    )?;

    // Classification functions
    globals.set(
        "UnitClassification",
        lua.create_function(|lua, _unit: String| {
            // Returns: "normal", "elite", "rare", "rareelite", "worldboss", "trivial", "minus"
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

    // Casting functions
    globals.set(
        "UnitCastingInfo",
        lua.create_function(|_, _unit: String| {
            // Returns: name, text, texture, startTime, endTime, isTradeSkill, castID, notInterruptible, spellID
            Ok(Value::Nil)
        })?,
    )?;
    globals.set(
        "UnitChannelInfo",
        lua.create_function(|_, _unit: String| {
            // Returns: name, text, texture, startTime, endTime, isTradeSkill, notInterruptible, spellID
            Ok(Value::Nil)
        })?,
    )?;

    // Aura functions - no auras in simulation
    globals.set(
        "UnitAura",
        lua.create_function(
            |_, (_unit, _index, _filter): (String, i32, Option<String>)| Ok(Value::Nil),
        )?,
    )?;
    globals.set(
        "UnitBuff",
        lua.create_function(
            |_, (_unit, _index, _filter): (String, i32, Option<String>)| Ok(Value::Nil),
        )?,
    )?;
    globals.set(
        "UnitDebuff",
        lua.create_function(
            |_, (_unit, _index, _filter): (String, i32, Option<String>)| Ok(Value::Nil),
        )?,
    )?;
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
             (_unit, _filter, _max_count, _callback, _use_packed): (
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
             (_predicate, _unit, _filter, _spell_id, _caster): (
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
        lua.create_function(|_, (_name, _unit, _filter): (String, String, String)| Ok(Value::Nil))?,
    )?;
    globals.set("AuraUtil", aura_util)?;

    // Weapon enchant info
    globals.set(
        "GetWeaponEnchantInfo",
        lua.create_function(|_, ()| {
            // Returns: hasMainHandEnchant, mainHandExpiration, mainHandCharges, mainHandEnchantID,
            //          hasOffHandEnchant, offHandExpiration, offHandCharges, offHandEnchantID
            Ok((false, 0i32, 0i32, 0i32, false, 0i32, 0i32, 0i32))
        })?,
    )?;

    Ok(())
}
