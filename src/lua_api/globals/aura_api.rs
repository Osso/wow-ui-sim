//! Aura/buff API functions.
//!
//! Implements UnitBuff, UnitDebuff, UnitAura, GetPlayerAuraBySpellID,
//! and the AuraUtil namespace stubs.

use crate::lua_api::state::AuraInfo;
use crate::lua_api::SimState;
use mlua::{Lua, MultiValue, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all aura-related global functions.
pub fn register_aura_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_unit_buff(lua, state.clone())?;
    register_unit_debuff(lua)?;
    register_unit_aura(lua, state.clone())?;
    register_get_player_aura_by_spell_id(lua, state)?;
    lua.globals().set("AuraUtil", register_aura_util(lua)?)?;
    Ok(())
}

/// Check if a filter string includes "HARMFUL".
fn filter_is_harmful(filter: &Option<String>) -> bool {
    filter.as_ref().map_or(false, |f| f.contains("HARMFUL"))
}

/// Get the nth player buff (1-based index), or None.
fn get_player_buff(state: &SimState, index: i32) -> Option<&AuraInfo> {
    if index < 1 { return None; }
    state.player_buffs.get((index - 1) as usize)
}

/// Build the old-style multi-return values for UnitBuff/UnitAura.
pub(super) fn build_aura_multi_value(lua: &Lua, aura: &AuraInfo) -> Result<MultiValue> {
    Ok(MultiValue::from_vec(vec![
        Value::String(lua.create_string(aura.name)?),
        Value::Integer(aura.icon as i64),
        Value::Integer(aura.applications as i64),
        Value::Nil, // dispelName (buffs have none)
        Value::Number(aura.duration),
        Value::Number(aura.expiration_time),
        Value::String(lua.create_string(aura.source_unit)?),
        Value::Boolean(aura.is_stealable),
        Value::Boolean(false), // nameplateShowPersonal
        Value::Integer(aura.spell_id as i64),
        Value::Boolean(aura.can_apply_aura),
        Value::Boolean(false), // isBossAura
        Value::Boolean(aura.is_from_player_or_player_pet),
        Value::Boolean(false), // nameplateShowAll
        Value::Number(1.0),    // timeMod
    ]))
}

/// Build an AuraData Lua table from an AuraInfo.
pub(super) fn build_aura_data_table(lua: &Lua, aura: &AuraInfo) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    set_aura_data_core_fields(lua, &t, aura)?;
    set_aura_data_extra_fields(lua, &t, aura)?;
    Ok(t)
}

/// Set the core AuraData fields (name, icon, duration, etc.).
fn set_aura_data_core_fields(lua: &Lua, t: &mlua::Table, aura: &AuraInfo) -> Result<()> {
    t.set("name", lua.create_string(aura.name)?)?;
    t.set("icon", aura.icon)?;
    t.set("applications", aura.applications)?;
    t.set("dispelName", Value::Nil)?;
    t.set("duration", aura.duration)?;
    t.set("expirationTime", aura.expiration_time)?;
    t.set("sourceUnit", lua.create_string(aura.source_unit)?)?;
    t.set("isStealable", aura.is_stealable)?;
    t.set("nameplateShowPersonal", false)?;
    t.set("spellId", aura.spell_id)?;
    Ok(())
}

/// Set the extra AuraData fields (boolean flags, instance ID, points).
fn set_aura_data_extra_fields(lua: &Lua, t: &mlua::Table, aura: &AuraInfo) -> Result<()> {
    t.set("canApplyAura", aura.can_apply_aura)?;
    t.set("isBossAura", false)?;
    t.set("isFromPlayerOrPlayerPet", aura.is_from_player_or_player_pet)?;
    t.set("nameplateShowAll", false)?;
    t.set("timeMod", 1.0)?;
    t.set("points", lua.create_table()?)?;
    t.set("auraInstanceID", aura.aura_instance_id)?;
    t.set("isHelpful", aura.is_helpful)?;
    t.set("isHarmful", false)?;
    t.set("isRaid", false)?;
    t.set("isNameplateOnly", false)?;
    Ok(())
}

/// Register UnitBuff: returns unpacked aura data for the nth player buff.
fn register_unit_buff(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    lua.globals().set(
        "UnitBuff",
        lua.create_function(move |lua, (unit, index, _filter): (String, i32, Option<String>)| {
            if unit != "player" {
                return Ok(MultiValue::new());
            }
            let s = state.borrow();
            match get_player_buff(&s, index) {
                Some(aura) => build_aura_multi_value(lua, aura),
                None => Ok(MultiValue::new()),
            }
        })?,
    )
}

/// Register UnitDebuff: returns nil (no debuffs in sim).
fn register_unit_debuff(lua: &Lua) -> Result<()> {
    lua.globals().set(
        "UnitDebuff",
        lua.create_function(
            |_, (_unit, _index, _filter): (String, i32, Option<String>)| Ok(Value::Nil),
        )?,
    )
}

/// Register UnitAura: returns unpacked aura data filtered by HELPFUL/HARMFUL.
fn register_unit_aura(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    lua.globals().set(
        "UnitAura",
        lua.create_function(move |lua, (unit, index, filter): (String, i32, Option<String>)| {
            if unit != "player" || filter_is_harmful(&filter) {
                return Ok(MultiValue::new());
            }
            let s = state.borrow();
            match get_player_buff(&s, index) {
                Some(aura) => build_aura_multi_value(lua, aura),
                None => Ok(MultiValue::new()),
            }
        })?,
    )
}

/// Register GetPlayerAuraBySpellID: looks up a buff by spell ID.
fn register_get_player_aura_by_spell_id(
    lua: &Lua,
    state: Rc<RefCell<SimState>>,
) -> Result<()> {
    lua.globals().set(
        "GetPlayerAuraBySpellID",
        lua.create_function(move |lua, spell_id: i32| {
            let s = state.borrow();
            let aura = s.player_buffs.iter().find(|a| a.spell_id == spell_id);
            match aura {
                Some(a) => Ok(Value::Table(build_aura_data_table(lua, a)?)),
                None => Ok(Value::Nil),
            }
        })?,
    )
}

/// AuraUtil namespace stubs (overridden by Blizzard AuraUtil.lua in full sim).
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
