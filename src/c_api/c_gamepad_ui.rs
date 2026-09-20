//! Forever Gamepad views use the existing logical pet-action slot space.

use crate::lua_api::state::FIRST_PET_ACTION_SLOT;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let table = super::helpers::ensure_namespace(state, "C_GamepadUI")?;
    table_set_rust_fn_static(
        state,
        table,
        "GetFirstGamepadPetActionStorageSlotIndex",
        first_pet_slot,
    )
}

fn first_pet_slot(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(FIRST_PET_ACTION_SLOT as f64));
    Ok(1)
}
