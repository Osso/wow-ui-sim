//! Host-owned optional entrance title; no synthetic location or localization.

use crate::lua_api::methods::{borrow_state, create_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// INFERRED: the result is public and extra arguments are ignored unread.
pub(crate) fn get_title(state: &mut LuaState) -> LuaResult<u32> {
    let title = borrow_state(state)?.delve_entrance_title.clone();
    match title {
        Some(title) => {
            let title = create_string(state, &title);
            state.push(title);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}
