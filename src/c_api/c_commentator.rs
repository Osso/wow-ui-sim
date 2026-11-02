//! Shared mouse-disable setting used by the commentator consumer.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_Commentator")?;
    table_set_rust_fn_static(state, namespace, "SetMouseDisabled", set_mouse_disabled)
}

fn set_mouse_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let disabled = match value {
        Val::Nil => {
            return Err(runtime_error(
                "Usage: C_Commentator.SetMouseDisabled(disabled)",
            ));
        }
        Val::Bool(false) => false,
        _ => true,
    };
    let requested = {
        let mut sim = borrow_state_mut(state)?;
        sim.mouse_disabled = Some(disabled);
        let available = sim.mouse_force_available
            || (!disabled
                && sim.mouse_availability_source
                && (!sim.mouse_availability_inhibited || sim.mouse_inhibition_cancelled));
        let changed = sim.mouse_available != Some(available);
        sim.mouse_available = Some(available);
        if !changed {
            None
        } else if available {
            sim.mouse_available_cursor_requested
        } else {
            Some(false)
        }
    };
    if let Some(visible) = requested {
        crate::lua_api::globals::real::mouse_probes::set_cursor_visibility(state, visible)?;
    }
    Ok(0)
}
