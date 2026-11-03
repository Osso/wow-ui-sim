//! Forever pet move-to confirmation state, separate from spell targeting and
//! dragged cursor payloads. The selected gamepad UI confirms through a screen
//! click. This models the pending command, not a world position or pet movement.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    if ACTIVE != ClientProfile::WowForever {
        return Ok(());
    }
    let state = lua.state_mut();
    let namespace = super::ensure_namespace(state, "C_PetInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsPetOrderMoveToPending",
        is_pet_order_move_to_pending,
    )?;
    Ok(())
}

fn is_pet_order_move_to_pending(state: &mut LuaState) -> LuaResult<u32> {
    let pending = borrow_state(state)?.pet.move_to_pending;
    state.push(Val::Bool(pending));
    Ok(1)
}

fn set_pending(state: &mut LuaState, pending: bool) -> LuaResult<bool> {
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let changed = sim.pet.move_to_pending != pending;
        sim.pet.move_to_pending = pending;
        sim.pet.move_to_screen_click_started = false;
        changed
    };
    if changed {
        // PetInfoDocumentation declares this event synchronous, with one bool.
        fire_named_event_state(
            state,
            "PET_ORDER_MOVE_TO_PENDING_CHANGED",
            &[Val::Bool(pending)],
        );
    }
    Ok(changed)
}

pub(crate) fn begin_move_to(state: &mut LuaState) -> LuaResult<()> {
    // Reuse the summoned-pet identity used by GetPetIcon/PetAssistMode.
    if borrow_state(state)?.pet.icon.is_some() {
        set_pending(state, true)?;
    }
    Ok(())
}

pub(crate) fn begin_screen_click(state: &mut LuaState) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    sim.pet.move_to_screen_click_started = sim.pet.move_to_pending;
    Ok(())
}

pub(crate) fn confirm_screen_click(state: &mut LuaState) -> LuaResult<()> {
    let confirming = {
        let mut sim = borrow_state_mut(state)?;
        let confirming = sim.pet.move_to_pending && sim.pet.move_to_screen_click_started;
        sim.pet.move_to_screen_click_started = false;
        confirming
    };
    if confirming {
        set_pending(state, false)?;
    }
    Ok(())
}

/// Inferred cancellation policy: selected gamepad Back and Escape both use
/// SpellStopTargeting. Native pet-specific cancellation parity is unverified.
pub(crate) fn cancel_move_to(state: &mut LuaState) -> LuaResult<bool> {
    set_pending(state, false)
}
