//! INFERRED (R88dy, native evidence pending): Retail power display modifiers are
//! supplied per power type; an unset type keeps the existing default of 1.0.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "UnitPowerDisplayMod", unit_power_display_mod)?;
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetUnitPowerDisplayMod", set_unit_power_display_mod)
}

fn unit_power_display_mod(state: &mut LuaState) -> LuaResult<u32> {
    let power_type = i32::from_stack(state, 1)?;
    let modifier = borrow_state(state)?
        .player
        .power_display_modifiers
        .get(&power_type)
        .copied()
        .unwrap_or(1.0);
    state.push(Val::Num(modifier));
    Ok(1)
}

fn set_unit_power_display_mod(state: &mut LuaState) -> LuaResult<u32> {
    let power_type = i32::from_stack(state, 1)?;
    let modifier = f64::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .player
        .power_display_modifiers
        .insert(power_type, modifier);
    Ok(0)
}
