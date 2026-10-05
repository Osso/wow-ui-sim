//! Host-owned tiered entrance state: optional title and entrance type; no
//! synthetic location or localization.

use crate::lua_api::methods::borrow_state;
#[cfg(feature = "retail-12-0-7")]
use crate::lua_api::methods::create_string;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// `Enum.TieredEntranceType.Delve` (DelvesConstantsDocumentation.lua).
pub(crate) const TIERED_ENTRANCE_TYPE_DELVE: i32 = 1;

/// INFERRED: the result is public and extra arguments are ignored unread.
#[cfg(feature = "retail-12-0-7")]
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

pub(crate) fn get_tiered_entrance_type(state: &mut LuaState) -> LuaResult<u32> {
    let entrance_type = borrow_state(state)?.tiered_entrance_type;
    state.push(Val::Num(f64::from(entrance_type)));
    Ok(1)
}
