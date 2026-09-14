//! Global outfit-situations setting; outfit behavior remains unmodeled.

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_TransmogOutfitInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetOutfitSituationsEnabled",
        get_outfit_situations_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetOutfitSituationsEnabled",
        set_outfit_situations_enabled,
    )
}

fn get_outfit_situations_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.outfit_situations_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn set_outfit_situations_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(enabled) = stack_val(state, 1) else {
        return Err(rilua::runtime_error(
            "SetOutfitSituationsEnabled requires a boolean",
        ));
    };
    borrow_state_mut(state)?.outfit_situations_enabled = enabled;
    Ok(0)
}
