//! Player-related API functions.
//!
//! This module provides WoW API functions related to:
//! - BattleNet features (BNFeaturesEnabled, BNConnected, BNGetFriendInfo, etc.)
//! - Specialization info (GetSpecialization, GetSpecializationInfo, etc.)
//! - Action bar functions (HasAction, GetActionInfo, GetActionTexture, etc.)

use mlua::{Lua, Result, Value};

/// Register all player-related API functions to the Lua globals table.
pub fn register_player_api(lua: &Lua) -> Result<()> {
    register_battlenet_functions(lua)?;
    register_specialization_functions(lua)?;
    register_action_bar_functions(lua)?;
    register_timerunning_functions(lua)?;
    Ok(())
}

fn register_timerunning_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("PlayerIsTimerunning", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsPlayerAtEffectiveMaxLevel", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsXPUserDisabled", lua.create_function(|_, ()| Ok(false))?)?;
    Ok(())
}

/// Register BattleNet social functions.
fn register_battlenet_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("BNFeaturesEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNFeaturesEnabledAndConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNGetFriendInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    globals.set("BNGetNumFriends", lua.create_function(|_, ()| Ok((0, 0)))?)?; // online, total
    globals.set("BNGetInfo", lua.create_function(|lua, ()| {
        // Return: presenceID, battleTag, toonID, currentBroadcast, bnetAFK, bnetDND, isRIDEnabled
        Ok((
            Value::Integer(0),
            Value::String(lua.create_string("SimPlayer#0000")?),
            Value::Nil,
            Value::String(lua.create_string("")?),
            Value::Boolean(false),
            Value::Boolean(false),
            Value::Boolean(false),
        ))
    })?)?;

    Ok(())
}

/// Register specialization query functions.
fn register_specialization_functions(lua: &Lua) -> Result<()> {
    register_spec_basic_queries(lua)?;
    register_spec_info_lookups(lua)?;
    Ok(())
}

/// Basic spec queries: GetSpecialization, GetSpecializationInfo, GetNumSpecializations.
fn register_spec_basic_queries(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("GetSpecialization", lua.create_function(|_, ()| Ok(1))?)?;
    globals.set("GetSpecializationInfo", lua.create_function(|lua, spec_index: i32| {
        let (id, name, role) = match spec_index {
            1 => (62, "Arcane", "DAMAGER"),
            2 => (63, "Fire", "DAMAGER"),
            3 => (64, "Frost", "DAMAGER"),
            _ => (62, "Arcane", "DAMAGER"),
        };
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(id),
            Value::String(lua.create_string(name)?),
            Value::String(lua.create_string("Spec description")?),
            Value::Integer(136116),
            Value::String(lua.create_string(role)?),
            Value::Integer(4),
        ]))
    })?)?;
    globals.set("GetNumSpecializations", lua.create_function(|_, ()| Ok(4))?)?;
    globals.set("GetSpecializationRoleByID", lua.create_function(|lua, _spec_id: i32| {
        Ok(Value::String(lua.create_string("DAMAGER")?))
    })?)?;
    globals.set(
        "GetNumSpecializationsForClassID",
        lua.create_function(|_, (_class_id, _sex): (Option<i32>, Option<i32>)| {
            Ok(_class_id.map_or(0, |_| 3i32))
        })?,
    )?;

    Ok(())
}

/// Spec info lookups by ID or class: GetSpecializationInfoByID, ForSpecID, ForClassID.
fn register_spec_info_lookups(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("GetSpecializationInfoByID", lua.create_function(|lua, _spec_id: i32| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(62),
            Value::String(lua.create_string("Arcane")?),
            Value::String(lua.create_string("Spec description")?),
            Value::Integer(136116),
            Value::String(lua.create_string("DAMAGER")?),
            Value::String(lua.create_string("MAGE")?),
        ]))
    })?)?;
    globals.set("GetSpecializationInfoForSpecID", lua.create_function(|lua, _spec_id: i32| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(62),
            Value::String(lua.create_string("Arcane")?),
            Value::String(lua.create_string("Spec description")?),
            Value::Integer(136116),
            Value::String(lua.create_string("DAMAGER")?),
            Value::String(lua.create_string("MAGE")?),
        ]))
    })?)?;
    globals.set("GetSpecializationInfoForClassID", lua.create_function(|lua, (_class_id, spec_index): (i32, i32)| {
        if spec_index < 1 || spec_index > 4 {
            return Ok(mlua::MultiValue::new());
        }
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(62i64 + spec_index as i64 - 1),
            Value::String(lua.create_string("Spec")?),
            Value::String(lua.create_string("Description")?),
            Value::Integer(136116),
            Value::String(lua.create_string("DAMAGER")?),
            Value::Boolean(false),
            Value::Boolean(true),
        ]))
    })?)?;

    Ok(())
}

/// Register action bar query functions (all no-op in simulation).
fn register_action_bar_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("HasAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("GetActionInfo", lua.create_function(|_, _slot: Value| Ok(Value::Nil))?)?;
    globals.set("GetActionTexture", lua.create_function(|_, _slot: Value| Ok(Value::Nil))?)?;
    globals.set("GetActionText", lua.create_function(|_, _slot: Value| Ok(Value::Nil))?)?;
    globals.set("GetActionCount", lua.create_function(|_, _slot: Value| Ok(0))?)?;
    globals.set("GetActionCooldown", lua.create_function(|_, _slot: Value| {
        Ok((0.0_f64, 0.0_f64, 1, 1.0_f64))
    })?)?;
    globals.set("IsUsableAction", lua.create_function(|_, _slot: Value| Ok((false, false)))?)?;
    globals.set("IsConsumableAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsStackableAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsAttackAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsAutoRepeatAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsCurrentAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("GetActionCharges", lua.create_function(|_, _slot: Value| {
        Ok((0, 0, 0.0_f64, 0.0_f64, 1.0_f64))
    })?)?;
    globals.set("GetPossessInfo", lua.create_function(|_, _index: Value| Ok(Value::Nil))?)?;
    globals.set("GetActionBarPage", lua.create_function(|_, ()| Ok(1))?)?;
    globals.set("GetBonusBarOffset", lua.create_function(|_, ()| Ok(0))?)?;
    globals.set("GetOverrideBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetVehicleBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetTempShapeshiftBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("IsPossessBarVisible", lua.create_function(|_, ()| Ok(false))?)?;

    Ok(())
}
