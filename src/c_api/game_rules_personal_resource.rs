//! Personal-resource display decision from the game rule and effective CVar.

use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = {
        let sim = borrow_state(state)?;
        let disabled = sim
            .game_rules
            .rules
            .get("PersonalResourceDisplayDisabled")
            .is_some_and(|rule| rule.as_float > 0.0 || rule.as_float < 0.0);
        let setting = sim
            .cvars
            .get("nameplateShowSelf")
            .and_then(|value| value.parse::<i64>().ok())
            .is_some_and(|value| value != 0);
        !disabled && setting
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}
