//! R88ad.9 `GetLoginScreenBackground`: a native background object that accepts
//! the high-resolution ID selects it; otherwise the low-resolution ID is used.

use crate::c_api::helpers::read_untainted_signed32_argument;
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const NAME: &str = "GetLoginScreenBackground";
const USAGE: &str = "Usage: local result = GetLoginScreenBackground(highResFileID, lowResFileID)";

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    LuaApiMut::register_function(state, NAME, get_login_screen_background)
}

fn get_login_screen_background(state: &mut LuaState) -> LuaResult<u32> {
    let high = read_untainted_signed32_argument(state, 1, NAME, USAGE)?;
    let low = read_untainted_signed32_argument(state, 2, NAME, USAGE)?;
    let accepted = borrow_state(state)?
        .glue_state
        .login_background_file_ids
        .contains(&high);
    state.push(Val::Num(f64::from(if accepted { high } else { low })));
    Ok(1)
}
