//! Supplied integer rules that force multiaction bar visibility policy.
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn is_forced(state: &mut LuaState) -> LuaResult<u32> {
    let forced = {
        let sim = borrow_state(state)?;
        ["ForcedMultiActionBarSetting", "HideAllMultiActionBars"]
            .into_iter()
            .any(|name| {
                sim.game_rules
                    .rules
                    .get(name)
                    .and_then(|rule| rule.integer_value)
                    .is_some_and(|value| value != 0)
            })
    };
    state.push(Val::Bool(forced));
    Ok(1)
}
