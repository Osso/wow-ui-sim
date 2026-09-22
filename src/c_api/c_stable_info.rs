//! State-backed stable APIs; Camelot read data is isolated from other profiles.

#[cfg(feature = "client-wowforever")]
pub mod forever;

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_stable_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let stable_info = ensure_namespace(state, "C_StableInfo")?;
    table_set_rust_fn_static(
        state,
        stable_info,
        "IsAtPetStable",
        c_stable_info_is_at_pet_stable,
    )?;
    #[cfg(feature = "client-wowforever")]
    forever::register(state)?;
    Ok(())
}

fn c_stable_info_is_at_pet_stable(state: &mut LuaState) -> LuaResult<u32> {
    let open = borrow_state(state)?.pet_stables_open;
    state.push(Val::Bool(open));
    Ok(1)
}
