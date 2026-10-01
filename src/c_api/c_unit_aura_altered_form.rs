//! Retail 12.0.5 altered-form query over explicit player input.

use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(state, namespace, "WantsAlteredForm", wants_altered_form)
}

fn wants_altered_form(state: &mut LuaState) -> LuaResult<u32> {
    // Authenticate only the documented AllowedWhenUntainted unit argument;
    // the VM preserves caller taint and the original secret value.
    let value = unwrap_secret(state, stack_val(state, 1))?;
    // INFERRED: require a string, with no numeric coercion or nil default.
    let unit = val_to_string(state, value).ok_or_else(|| {
        runtime_error("C_UnitAuras.WantsAlteredForm requires a UTF-8 UnitToken string")
    })?;
    let wants = {
        let sim = borrow_state(state)?;
        // INFERRED: only identities matching the modeled player have input.
        existing_guid_for_unit(&sim, &unit).is_some_and(|guid| {
            existing_guid_for_unit(&sim, "player").as_ref() == Some(&guid)
                && sim.player.wants_altered_form
        })
    };
    state.push(Val::Bool(wants));
    Ok(1)
}
