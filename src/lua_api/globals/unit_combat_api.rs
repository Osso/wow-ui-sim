//! Unit combat stat functions for PaperDollFrame.

use mlua::{Lua, Result, Value};

/// Register all unit combat stat functions.
pub fn register_unit_combat_stat_functions(lua: &Lua) -> Result<()> {
    register_melee_stats(lua)?;
    register_ranged_and_form_stats(lua)
}

/// Register melee combat stat functions.
fn register_melee_stats(lua: &Lua) -> Result<()> {
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
    // UnitAttackSpeed(unit) -> mainSpeed, offSpeed
    g.set("UnitAttackSpeed", lua.create_function(|_, _unit: Value| {
        Ok((2.0_f64, 2.0_f64))
    })?)?;
    Ok(())
}

/// Register ranged, shapeshift, and pet combat functions.
fn register_ranged_and_form_stats(lua: &Lua) -> Result<()> {
    let g = lua.globals();
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
    g.set("PetHasActionBar", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("IsPetAttackAction", lua.create_function(|_, _idx: Value| Ok(false))?)?;
    Ok(())
}
