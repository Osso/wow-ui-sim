//! Non-rendering unit-token assignment with a host identity guard.

use crate::lua_api::globals::unit_misc::{
    existing_guid_for_unit, unit_identity_is_secret_in_state,
};
use crate::lua_api::methods::{borrow_state_mut, frame_id_from_stack, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

fn read_unit_token(state: &LuaState) -> LuaResult<String> {
    let value = unwrap_secret(state, stack_val(state, 2))?;
    if !matches!(value, Val::Str(_)) {
        return Err(runtime_error(
            "model unit assignment requires a unit token string",
        ));
    }
    val_to_string(state, value)
        .ok_or_else(|| runtime_error("model unit token string has been collected"))
}

pub(super) fn assign_unit(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let unit = read_unit_token(state)?;
    let result = {
        let mut sim = borrow_state_mut(state)?;
        if unit_identity_is_secret_in_state(&sim, &unit) {
            Val::Nil
        } else if existing_guid_for_unit(&sim, &unit).is_none() {
            // INFERRED: no identity means no binding; false is not model-load success.
            Val::Bool(false)
        } else {
            let frame = sim
                .widgets
                .get_mut_visual(id)
                .ok_or_else(|| runtime_error("model unit assignment frame does not exist"))?;
            frame.model_state_mut().player_model_state.last_unit = Some(unit);
            Val::Bool(true)
        }
    };
    state.push(result);
    Ok(1)
}
