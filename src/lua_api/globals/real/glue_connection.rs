//! R43 connection-backed non-C glue global; see glue-login-flow.md.
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        state.global,
        "CanCreateCharacter",
        can_create_character,
    )
}

fn can_create_character(state: &mut LuaState) -> LuaResult<u32> {
    let allowed = {
        let sim = borrow_state(state)?;
        let login = &sim.glue_login;
        let info = login.received_connection_info;
        // An empty host GUID denotes the native zero/absent GUID, not a character.
        let count = login
            .characters
            .iter()
            .filter(|slot| !slot.character_id.is_empty())
            .count();
        info.ready && (count as i64) < i64::from(info.character_limit)
    };
    state.push(Val::Bool(allowed));
    Ok(1)
}
