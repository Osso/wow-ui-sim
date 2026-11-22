//! Shared commentator state used by the original UI consumers.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let mouse_disabled_profile = matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr);
    let spectating_profile = matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever);
    if !mouse_disabled_profile && !spectating_profile {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_Commentator")?;
    if mouse_disabled_profile {
        table_set_rust_fn_static(state, namespace, "SetMouseDisabled", set_mouse_disabled)?;
    }
    if spectating_profile {
        table_set_rust_fn_static(state, namespace, "IsSpectating", is_spectating)?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        builder.set_function("SetSpectating", set_spectating)
    } else {
        Ok(builder)
    }
}

// INFERRED R88ek (native evidence pending): reuse the supplied simulator scenario.
fn is_spectating(state: &mut LuaState) -> LuaResult<u32> {
    let spectating = borrow_state(state)?.spectating;
    state.push(Val::Bool(spectating));
    Ok(1)
}

fn set_spectating(state: &mut LuaState) -> LuaResult<u32> {
    let spectating = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.spectating = spectating;
    Ok(0)
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
