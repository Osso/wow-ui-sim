//! Unit-related WoW API functions.
//!
//! Contains functions for querying unit information like names, classes, races,
//! health, power, auras, and other unit state.

use crate::lua_api::SimState;
use mlua::{Lua, MultiValue, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

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

/// Resolve a unit name, checking target, party members, and player name.
/// Returns owned String to avoid borrow lifetime issues in closures.
fn resolve_unit_name_with_party(unit: &str, state: &SimState) -> String {
    if unit == "player" {
        return state.player_name.clone();
    }
    if unit == "target" {
        return state.current_target.as_ref()
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
    }
    if let Some(idx) = parse_party_index(unit)
        && let Some(m) = state.party_members.get(idx) {
            return m.name.to_string();
        }
    "SimUnit".to_string()
}

/// Parse a "partyN" unit ID and return the 0-based index if valid.
pub fn parse_party_index(unit: &str) -> Option<usize> {
    unit.strip_prefix("party")
        .and_then(|n| n.parse::<usize>().ok())
        .filter(|&n| n >= 1)
        .map(|n| n - 1)
}

/// Register unit-related global functions.
pub fn register_unit_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_identity_functions(lua, state.clone())?;
    register_class_functions(lua, state.clone())?;
    register_name_functions(lua, state.clone())?;
    register_state_functions(lua, state.clone())?;
    register_group_functions(lua, state.clone())?;
    super::unit_health_power_api::register_health_power_functions(lua, state.clone())?;
    register_threat_functions(lua)?;
    register_classification_functions(lua)?;
    register_casting_functions(lua)?;
    register_aura_functions(lua)?;
    register_weapon_enchant_functions(lua)?;
    register_xp_functions(lua)?;
    register_pvp_vehicle_functions(lua, state.clone())?;
    register_misc_unit_functions(lua, state.clone())?;
    super::targeting_api::register_targeting_functions(lua, state)?;
    super::unit_combat_api::register_unit_combat_stat_functions(lua)?;
    Ok(())
}

/// Register UnitRace, UnitSex, UnitGUID, UnitLevel, UnitEffectiveLevel,
/// UnitExists, UnitFactionGroup.
fn register_identity_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_identity_stubs(lua)?;
    register_identity_party_aware(lua, state)
}

/// Register identity functions that don't need party state.
fn register_identity_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set(
        "UnitRace",
        lua.create_function(|lua, _unit: Option<String>| {
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string("Human")?),
                Value::String(lua.create_string("Human")?),
            ]))
        })?,
    )?;
    globals.set("UnitSex", lua.create_function(|_, _unit: Option<String>| Ok(2))?)?;
    globals.set("UnitEffectiveLevel", lua.create_function(|_, _unit: Option<String>| Ok(70))?)?;
    globals.set(
        "UnitFactionGroup",
        lua.create_function(|lua, _unit: Option<String>| {
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string("Alliance")?),
                Value::String(lua.create_string("Alliance")?),
            ]))
        })?,
    )?;
    Ok(())
}

/// Register UnitGUID, UnitLevel, UnitExists with party member awareness.
fn register_identity_party_aware(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_unit_guid(lua, state.clone())?;
    register_unit_level_exists(lua, state)
}

/// Register UnitGUID with party/target awareness.
fn register_unit_guid(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    lua.globals().set(
        "UnitGUID",
        lua.create_function(move |lua, unit: Option<String>| {
            let Some(unit) = unit else { return Ok(Value::Nil) };
            if unit == "player" {
                return Ok(Value::String(lua.create_string("Player-0000-00000001")?));
            }
            if unit == "target" {
                let s = state.borrow();
                let guid = s.current_target.as_ref().map(|t| t.guid.clone())
                    .unwrap_or_else(|| "Creature-0000-00000000".into());
                return Ok(Value::String(lua.create_string(&guid)?));
            }
            if let Some(idx) = parse_party_index(&unit)
                && idx < state.borrow().party_members.len() {
                    let guid = format!("Player-0000-0000000{}", idx + 2);
                    return Ok(Value::String(lua.create_string(&guid)?));
                }
            Ok(Value::String(lua.create_string("Creature-0000-00000000")?))
        })?,
    )
}

/// Register UnitLevel and UnitExists with party/target awareness.
fn register_unit_level_exists(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    let st = state.clone();
    globals.set(
        "UnitLevel",
        lua.create_function(move |_, unit: Option<String>| {
            let Some(unit) = unit else { return Ok(0) };
            if unit == "target" {
                let s = st.borrow();
                return Ok(s.current_target.as_ref().map(|t| t.level).unwrap_or(1));
            }
            if let Some(idx) = parse_party_index(&unit) {
                let s = st.borrow();
                if let Some(m) = s.party_members.get(idx) {
                    return Ok(m.level);
                }
            }
            Ok(70)
        })?,
    )?;

    globals.set(
        "UnitExists",
        lua.create_function(move |_, unit: Option<String>| {
            let Some(unit) = unit else { return Ok(false) };
            if matches!(unit.as_str(), "player" | "pet") {
                return Ok(true);
            }
            if unit == "target" {
                return Ok(state.borrow().current_target.is_some());
            }
            if let Some(idx) = parse_party_index(&unit) {
                return Ok(idx < state.borrow().party_members.len());
            }
            Ok(false)
        })?,
    )
}

/// Register UnitClass, UnitClassBase, GetNumClasses, GetClassInfo,
/// LocalizedClassList.
fn register_class_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_unit_class(lua, state)?;
    register_class_lookup_functions(lua)
}

/// Register UnitClass with party member awareness.
fn register_unit_class(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    globals.set(
        "UnitClass",
        lua.create_function(move |lua, unit: Option<String>| {
            let unit = unit.unwrap_or_default();
            let (name, file, idx) = if unit == "target" {
                let s = state.borrow();
                if let Some(t) = &s.current_target {
                    let (n, f) = class_info_by_index(t.class_index);
                    (n, f, t.class_index)
                } else {
                    ("Warrior", "WARRIOR", 1)
                }
            } else if let Some(i) = parse_party_index(&unit) {
                let s = state.borrow();
                if let Some(m) = s.party_members.get(i) {
                    let (n, f) = class_info_by_index(m.class_index);
                    (n, f, m.class_index)
                } else {
                    ("Warrior", "WARRIOR", 1)
                }
            } else {
                ("Warrior", "WARRIOR", 1)
            };
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::String(lua.create_string(file)?),
                Value::Integer(idx as i64),
            ]))
        })?,
    )?;
    Ok(())
}

/// Register stateless class lookup functions.
fn register_class_lookup_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "UnitClassBase",
        lua.create_function(|lua, _unit: Option<String>| {
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
fn register_name_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    let name_fns: &[&str] = &["UnitName", "UnitNameUnmodified"];
    for &fn_name in name_fns {
        let st = state.clone();
        globals.set(
            fn_name,
            lua.create_function(move |lua, unit: Option<String>| {
                let unit = unit.unwrap_or_default();
                let name = resolve_unit_name_with_party(&unit, &st.borrow());
                Ok(MultiValue::from_vec(vec![
                    Value::String(lua.create_string(name)?),
                    Value::Nil,
                ]))
            })?,
        )?;
    }

    let st = state.clone();
    globals.set(
        "UnitFullName",
        lua.create_function(move |lua, unit: Option<String>| {
            let unit = unit.unwrap_or_default();
            let name = resolve_unit_name_with_party(&unit, &st.borrow());
            Ok(MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::String(lua.create_string("SimRealm")?),
            ]))
        })?,
    )?;

    let st = state.clone();
    globals.set(
        "GetUnitName",
        lua.create_function(move |lua, (unit, _show_server): (Option<String>, Option<bool>)| {
            let unit = unit.unwrap_or_default();
            let name = resolve_unit_name_with_party(&unit, &st.borrow());
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;

    let st = state;
    globals.set(
        "UnitPVPName",
        lua.create_function(move |lua, unit: Option<String>| {
            let unit = unit.unwrap_or_default();
            let name = resolve_unit_name_with_party(&unit, &st.borrow());
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;

    Ok(())
}

/// Register unit state boolean functions: alive/dead, AFK/DND, combat
/// relations, visibility.
fn register_state_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_state_boolean_stubs(lua)?;
    register_death_functions(lua, state.clone())?;
    register_state_comparisons(lua, state.clone())?;
    register_state_relations(lua, state)
}

/// Register UnitIsDead, UnitIsDeadOrGhost with player health awareness.
fn register_death_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    let st = state.clone();
    globals.set(
        "UnitIsDead",
        lua.create_function(move |_, unit: Option<String>| {
            if unit.as_deref() == Some("player") {
                return Ok(st.borrow().player_health <= 0);
            }
            Ok(false)
        })?,
    )?;
    globals.set(
        "UnitIsDeadOrGhost",
        lua.create_function(move |_, unit: Option<String>| {
            if unit.as_deref() == Some("player") {
                return Ok(state.borrow().player_health <= 0);
            }
            Ok(false)
        })?,
    )?;
    Ok(())
}

/// Register single-unit boolean stubs (always false or always true).
fn register_state_boolean_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let false_stubs: &[&str] = &[
        "UnitIsGhost",
        "UnitIsAFK",
        "UnitIsDND",
        "UnitIsTapDenied",
        "UnitIsCorpse",
        "UnitIsWildBattlePet",
        "UnitIsBattlePetCompanion",
    ];
    for &name in false_stubs {
        globals.set(name, lua.create_function(|_, _unit: Option<String>| Ok(false))?)?;
    }
    globals.set("UnitIsConnected", lua.create_function(|_, _unit: Option<String>| Ok(true))?)?;
    globals.set("UnitIsVisible", lua.create_function(|_, _unit: Option<String>| Ok(true))?)?;
    globals.set("UnitBattlePetLevel", lua.create_function(|_, _unit: Option<String>| Ok(0))?)?;
    Ok(())
}

/// Register unit comparison functions (player checks, unit identity).
fn register_state_comparisons(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    globals.set(
        "UnitIsPlayer",
        lua.create_function(move |_, unit: Option<String>| {
            let Some(unit) = unit else { return Ok(false) };
            if unit == "player" {
                return Ok(true);
            }
            if unit == "target" {
                return Ok(state.borrow().current_target.as_ref()
                    .map(|t| t.is_player).unwrap_or(false));
            }
            if let Some(idx) = parse_party_index(&unit) {
                return Ok(idx < state.borrow().party_members.len());
            }
            Ok(false)
        })?,
    )?;
    globals.set(
        "UnitPlayerControlled",
        lua.create_function(|_, unit: Option<String>| {
            Ok(matches!(unit.as_deref(), Some("player" | "pet")))
        })?,
    )?;
    globals.set(
        "UnitIsUnit",
        lua.create_function(|_, (u1, u2): (Option<String>, Option<String>)| {
            Ok(u1.is_some() && u1 == u2)
        })?,
    )?;
    Ok(())
}

/// Register two-unit relation functions with target awareness.
fn register_state_relations(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    let st = state.clone();
    globals.set("UnitIsEnemy", lua.create_function(move |_, (_u1, u2): (Option<String>, Option<String>)| {
        if u2.as_deref() == Some("target") {
            return Ok(st.borrow().current_target.as_ref().map(|t| t.is_enemy).unwrap_or(false));
        }
        Ok(false)
    })?)?;
    let st = state.clone();
    globals.set("UnitCanAttack", lua.create_function(move |_, (_u1, u2): (Option<String>, Option<String>)| {
        if u2.as_deref() == Some("target") {
            return Ok(st.borrow().current_target.as_ref().map(|t| t.is_enemy).unwrap_or(false));
        }
        Ok(false)
    })?)?;
    let st = state.clone();
    globals.set("UnitIsFriend", lua.create_function(move |_, (_u1, u2): (Option<String>, Option<String>)| {
        if u2.as_deref() == Some("target") {
            return Ok(st.borrow().current_target.as_ref().map(|t| !t.is_enemy).unwrap_or(true));
        }
        Ok(true)
    })?)?;
    globals.set("UnitCanAssist", lua.create_function(move |_, (_u1, u2): (Option<String>, Option<String>)| {
        if u2.as_deref() == Some("target") {
            return Ok(state.borrow().current_target.as_ref().map(|t| !t.is_enemy).unwrap_or(true));
        }
        Ok(true)
    })?)?;
    globals.set("UnitInRange", lua.create_function(|_, _unit: Option<String>| Ok((true, true)))?)?;
    Ok(())
}

/// Register UnitInParty, UnitInRaid, UnitIsGroupLeader, UnitIsGroupAssistant.
fn register_group_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    let st = state.clone();
    globals.set(
        "UnitInParty",
        lua.create_function(move |_, unit: Option<String>| {
            let Some(unit) = unit else { return Ok(false) };
            if let Some(idx) = parse_party_index(&unit) {
                return Ok(idx < st.borrow().party_members.len());
            }
            Ok(false)
        })?,
    )?;
    globals.set(
        "UnitInRaid",
        lua.create_function(|_, _unit: Option<String>| Ok(Value::Nil))?,
    )?;
    let st = state;
    globals.set(
        "UnitIsGroupLeader",
        lua.create_function(move |_, unit: Option<String>| {
            let Some(unit) = unit else { return Ok(false) };
            if let Some(idx) = parse_party_index(&unit) {
                let s = st.borrow();
                if let Some(m) = s.party_members.get(idx) {
                    return Ok(m.is_leader);
                }
            }
            Ok(false)
        })?,
    )?;
    globals.set(
        "UnitIsGroupAssistant",
        lua.create_function(|_, _unit: Option<String>| Ok(false))?,
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
        lua.create_function(|lua, _unit: Option<String>| {
            Ok(Value::String(lua.create_string("normal")?))
        })?,
    )?;
    globals.set(
        "UnitCreatureType",
        lua.create_function(|lua, _unit: Option<String>| {
            Ok(Value::String(lua.create_string("Humanoid")?))
        })?,
    )?;
    globals.set(
        "UnitCreatureFamily",
        lua.create_function(|_, _unit: Option<String>| Ok(Value::Nil))?,
    )?;

    Ok(())
}

/// Register UnitCastingInfo, UnitChannelInfo.
fn register_casting_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "UnitCastingInfo",
        lua.create_function(|_, _unit: Option<String>| Ok(Value::Nil))?,
    )?;
    globals.set(
        "UnitChannelInfo",
        lua.create_function(|_, _unit: Option<String>| Ok(Value::Nil))?,
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
    // Random XP so the bar displays something visible at startup.
    let xp_max = 89_750i32;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let xp_current = (nanos % xp_max as u32) as i32;
    globals.set("UnitXP", lua.create_function(move |_, _unit: Option<String>| Ok(xp_current))?)?;
    globals.set("UnitXPMax", lua.create_function(move |_, _unit: Option<String>| Ok(xp_max))?)?;
    globals.set("UnitTrialXP", lua.create_function(|_, _unit: Option<String>| Ok(0i32))?)?;
    globals.set("GetXPExhaustion", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetRestState", lua.create_function(|_, ()| Ok(1i32))?)?;
    Ok(())
}

/// Register PvP and vehicle-related unit functions.
fn register_pvp_vehicle_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
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
        g.set(name, lua.create_function(|_, _unit: Option<String>| Ok(false))?)?;
    }

    g.set("UnitHonorLevel", lua.create_function(|_, _unit: Option<String>| Ok(0i32))?)?;
    g.set("UnitPartialPower", lua.create_function(|_, (_unit, _pt): (Option<String>, Option<i32>)| Ok(0i32))?)?;

    // UnitGroupRolesAssignedEnum -> nil
    g.set("UnitGroupRolesAssignedEnum", lua.create_function(|_, _unit: Option<String>| Ok(Value::Nil))?)?;

    // UnitRealmRelationship -> nil
    g.set("UnitRealmRelationship", lua.create_function(|_, _unit: Option<String>| Ok(Value::Nil))?)?;

    // UnitSelectionColor -> r, g, b, a (red for enemies, green for friendly, white default)
    g.set("UnitSelectionColor", lua.create_function(move |_, unit: Option<String>| {
        if unit.as_deref() == Some("target") {
            let s = state.borrow();
            if let Some(t) = &s.current_target {
                if t.is_enemy {
                    return Ok((1.0f64, 0.0f64, 0.0f64, 1.0f64)); // red
                }
                return Ok((0.0f64, 1.0f64, 0.0f64, 1.0f64)); // green
            }
        }
        Ok((1.0f64, 1.0f64, 1.0f64, 1.0f64))
    })?)?;

    Ok(())
}

/// Register miscellaneous unit query functions.
fn register_misc_unit_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let g = lua.globals();

    g.set("UnitPhaseReason", lua.create_function(|_, _unit: Option<String>| Ok(Value::Nil))?)?;
    g.set("UnitIsOwnerOrControllerOfUnit", lua.create_function(|_, (_u1, _u2): (String, String)| Ok(false))?)?;
    g.set("UnitIsWarModePhased", lua.create_function(|_, _unit: Option<String>| Ok(false))?)?;
    g.set("UnitIsWarModeDesired", lua.create_function(|_, _unit: Option<String>| Ok(false))?)?;
    g.set("UnitIsWarModeActive", lua.create_function(|_, _unit: Option<String>| Ok(false))?)?;
    g.set("UnitHasMana", lua.create_function(|_, _unit: Value| Ok(true))?)?;
    g.set("UnitHasRelicSlot", lua.create_function(|_, _unit: Value| Ok(false))?)?;
    g.set("IsActiveBattlefieldArena", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetUnitPowerBarInfo", lua.create_function(|_, _unit: Value| Ok(Value::Nil))?)?;
    let st = state.clone();
    g.set("IsInGroup", lua.create_function(move |_, _flags: Option<i32>| {
        Ok(!st.borrow().party_members.is_empty())
    })?)?;
    g.set("IsInRaid", lua.create_function(|_, ()| Ok(false))?)?;
    let st = state.clone();
    g.set("GetNumSubgroupMembers", lua.create_function(move |_, ()| {
        Ok(st.borrow().party_members.len() as i32)
    })?)?;
    g.set("GetNumGroupMembers", lua.create_function(move |_, ()| {
        let count = state.borrow().party_members.len() as i32;
        Ok(if count > 0 { count + 1 } else { 0 })
    })?)?;
    g.set("UnitStagger", lua.create_function(|_, _unit: Value| Ok(0i32))?)?;

    Ok(())
}

