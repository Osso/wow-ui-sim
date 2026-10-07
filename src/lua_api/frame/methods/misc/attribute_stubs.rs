//! Attribute and parent-key stub methods.

use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "CanChangeAttribute", can_change_attribute)?;
    table_set_rust_fn_static(state, mt, "ClearAttribute", clear_attribute)?;
    Ok(())
}

pub fn can_change_attribute(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

pub fn clear_attribute(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
