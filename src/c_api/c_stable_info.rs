//! State-backed stable APIs; Camelot read data is isolated from other profiles.

pub mod active_list;
#[cfg(feature = "client-wowforever")]
pub mod forever;
pub mod pet_info;
pub mod pet_spec_infos;
mod ptr_constants;

use crate::c_api::helpers::ensure_namespace;
use crate::event::Event;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_stable_info_surface(state: &mut LuaState) -> LuaResult<()> {
    ptr_constants::register(state)?;
    active_list::register(state)?;
    pet_info::register(state)?;
    pet_spec_infos::register(state)?;
    let stable_info = ensure_namespace(state, "C_StableInfo")?;
    table_set_rust_fn_static(
        state,
        stable_info,
        "IsAtPetStable",
        c_stable_info_is_at_pet_stable,
    )?;
    #[cfg(any(
        feature = "client-retail",
        feature = "client-ptr",
        feature = "client-wowforever"
    ))]
    table_set_rust_fn_static(state, stable_info, "ClosePetStables", close_pet_stables)?;
    #[cfg(feature = "rest-policy-flags")]
    table_set_rust_fn_static(state, stable_info, "IsBonusPetSlotAvailable", |s| {
        let available = borrow_state(s)?.pet_bonus_slot_available;
        s.push(Val::Bool(available));
        Ok(1)
    })?;
    #[cfg(feature = "client-wowforever")]
    forever::register(state)?;
    Ok(())
}

pub(crate) fn close_pet_stables(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.pet_stables_open = false;
    sim.events.push(Event {
        name: "PET_STABLE_CLOSED".to_string(),
        args: Vec::new(),
    });
    Ok(0)
}

fn c_stable_info_is_at_pet_stable(state: &mut LuaState) -> LuaResult<u32> {
    let open = borrow_state(state)?.pet_stables_open;
    state.push(Val::Bool(open));
    Ok(1)
}
