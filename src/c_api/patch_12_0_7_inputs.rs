//! Shared authentication boundary for B19–B22 host queries.
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Authenticate every argument, including extras, before interpreting selectors.
pub(crate) fn authenticate_arguments(state: &LuaState) -> LuaResult<Vec<Val>> {
    state.stack[state.base..state.top]
        .iter()
        .map(|value| rilua::table_security::unwrap_secret(state, *value))
        .collect()
}

/// INFERRED: only exact i32 selectors are modeled; malformed selectors miss.
pub(crate) fn integer_selector(value: Val) -> Option<i32> {
    let Val::Num(number) = value else {
        return None;
    };
    let integer = number as i32;
    (f64::from(integer) == number).then_some(integer)
}
