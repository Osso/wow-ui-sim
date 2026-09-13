//! Ordinary housing basic-mode state. Placement and collision effects are unmodeled.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let basic_mode = ensure_namespace(state, "C_HousingBasicMode")?;
    table_set_rust_fn_static(
        state,
        basic_mode,
        "IsFreePlaceEnabled",
        is_free_place_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        basic_mode,
        "SetFreePlaceEnabled",
        set_free_place_enabled,
    )
}

fn is_free_place_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = !borrow_state(state)?.housing.free_place_disabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn set_free_place_enabled(state: &mut LuaState) -> LuaResult<u32> {
    // Simulator validation policy; native coercion and exact errors are unverified.
    let Val::Bool(enabled) = stack_val(state, 1) else {
        return Err(rilua::runtime_error(
            "SetFreePlaceEnabled requires a boolean",
        ));
    };
    borrow_state_mut(state)?.housing.free_place_disabled = !enabled;
    Ok(0)
}
