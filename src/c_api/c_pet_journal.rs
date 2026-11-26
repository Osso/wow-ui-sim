//! State-backed C_PetJournal queries used by legacy pet-journal consumers.

use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_PetJournal")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsCurrentlySummoned",
        is_currently_summoned,
    )
}

fn is_currently_summoned(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let pet_guid = val_to_string(state, value)
        .ok_or_else(|| rilua::runtime_error("C_PetJournal.IsCurrentlySummoned requires a petID"))?;
    let is_summoned =
        borrow_state(state)?.pet.summoned_battle_pet_guid.as_deref() == Some(pet_guid.as_str());
    state.push(rilua::Val::Bool(is_summoned));
    Ok(1)
}
