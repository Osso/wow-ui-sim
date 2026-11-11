use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaError, LuaResult, RuntimeError, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        return Ok(());
    }
    table_set_rust_fn_static(state, state.global, "GetLootSpecialization", get)?;
    table_set_rust_fn_static(state, state.global, "SetLootSpecialization", set)?;
    Ok(())
}

fn get(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.loot_specialization;
    state.push(Val::Num(id as f64));
    Ok(1)
}

/// No server transport is modeled; keep the verified argument boundary without
/// inventing a local update or a `PLAYER_LOOT_SPEC_UPDATED` response.
fn set(state: &mut LuaState) -> LuaResult<u32> {
    if !matches!(stack_val(state, 1), Val::Num(_)) {
        return Err(LuaError::Runtime(RuntimeError {
            message: "Usage: SetLootSpecialization(specializationID)".into(),
            level: 1,
            traceback: vec![],
        }));
    }
    Ok(0)
}
