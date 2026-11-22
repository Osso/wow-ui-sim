//! Supplied integer rules that force multiaction bar visibility policy.
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn is_forced(state: &mut LuaState) -> LuaResult<u32> {
    let keys = ["ForcedMultiActionBarSetting", "HideAllMultiActionBars"]
        .map(|name| super::c_game_rules::key_from_name(state, name));
    let forced = {
        let sim = borrow_state(state)?;
        keys.iter().any(|key| {
            sim.game_rules
                .rules
                .get(key)
                .and_then(|rule| rule.integer_value)
                .is_some_and(|value| value != 0)
        })
    };
    state.push(Val::Bool(forced));
    Ok(1)
}
