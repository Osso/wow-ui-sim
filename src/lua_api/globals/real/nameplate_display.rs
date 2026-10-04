//! Host-backed player-display classification, independent of human identity.
//! The follower GUID input is explicit; absent-unit/type policies are INFERRED.

use crate::lua_api::globals::{unit_misc, unit_probes};
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn unit_treat_as_player_for_display(state: &mut LuaState) -> LuaResult<u32> {
    // Cached AllowedWhenUntainted: authenticate before reading world state.
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let unit = match value {
        Val::Nil => None,
        Val::Str(_) => Some(val_to_string(state, value).ok_or_else(|| {
            runtime_error("UnitTreatAsPlayerForDisplay: unit must be a UTF-8 string")
        })?),
        _ => {
            return Err(runtime_error(
                "UnitTreatAsPlayerForDisplay: unit must be a string or nil",
            ));
        }
    };
    let treat_as_player = match unit {
        Some(unit) => {
            let sim = borrow_state(state)?;
            let is_player = unit_probes::resolve_unit_is_player(&sim, &unit);
            let is_follower = unit_misc::existing_guid_for_unit(&sim, &unit)
                .is_some_and(|guid| sim.npc_follower_guids.contains(&guid));
            is_player || is_follower
        }
        None => false,
    };
    state.push(Val::Bool(treat_as_player));
    Ok(1)
}
