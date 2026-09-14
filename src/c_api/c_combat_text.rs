//! Explicit active-unit selection; combat-text event routing remains unmodeled.

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_CombatText")?;
    table_set_rust_fn_static(state, namespace, "GetActiveUnit", get_active_unit)?;
    table_set_rust_fn_static(state, namespace, "SetActiveUnit", set_active_unit)
}

fn get_active_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = borrow_state(state)?.combat_text_active_unit.clone();
    let value = match unit {
        Some(unit) => create_string(state, &unit),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

fn set_active_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    borrow_state_mut(state)?.combat_text_active_unit = Some(unit);
    Ok(0)
}
