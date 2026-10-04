//! Non-rendering actor identity query for the 12.0.7 ConditionalSecret removal.

use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::{borrow_state, native_frame_id_from_val};
use crate::lua_bridge::{IntoStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

/// INFERRED NotAllowed: later cache omits the input annotation.
/// Check receiver and every extra before validating receiver identity.
fn reject_secret_arguments(state: &LuaState) -> LuaResult<()> {
    for value in state.stack.iter().take(state.top).skip(state.base) {
        if rilua::table_security::is_secret_value(state, *value) {
            return Err(runtime_error(
                "GetModelUnitGUID does not accept secret arguments",
            ));
        }
    }
    Ok(())
}

pub(super) fn get_model_unit_guid(state: &mut LuaState) -> LuaResult<u32> {
    reject_secret_arguments(state)?;
    // Native backing ignores forgeable frame[0] and Lua GetObjectType overrides.
    let id = native_frame_id_from_val(state, stack_val(state, 1))
        .ok_or_else(|| runtime_error("GetModelUnitGUID expects a native actor handle"))?;
    let guid = {
        let sim = borrow_state(state)?;
        let actor = sim
            .widgets
            .get(id)
            .filter(|frame| frame.object_type_name.as_deref() == Some("ModelSceneActor"))
            .ok_or_else(|| runtime_error("GetModelUnitGUID expects a live ModelSceneActor"))?;
        // INFERRED: binding is a token; resolve its current host identity live.
        // INFERRED: unbound/missing identity is an empty, nonnil WOWGUID string.
        actor
            .model_state()
            .player_model_state
            .last_unit
            .as_deref()
            .and_then(|unit| existing_guid_for_unit(&sim, unit))
            .unwrap_or_default()
    };
    // Source removes ConditionalSecret: never wrap this getter's output.
    guid.into_stack(state)
}
