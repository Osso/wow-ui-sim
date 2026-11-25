//! Forever specialization-system selector state used by the legacy global getter.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetSpecializationSystem", get_specialization_system)?;
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetSpecializationSystem", set_specialization_system)
}

fn get_specialization_system(state: &mut LuaState) -> LuaResult<u32> {
    let system = borrow_state(state)?.specialization_system;
    state.push(Val::Num(system as f64));
    Ok(1)
}

fn set_specialization_system(state: &mut LuaState) -> LuaResult<u32> {
    let system = u32::from_stack(state, 1)?;
    if system > 1 {
        return Err(rilua::runtime_error("specialization system must be 0 or 1"));
    }
    borrow_state_mut(state)?.specialization_system = system;
    Ok(0)
}
