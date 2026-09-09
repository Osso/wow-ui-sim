//! Shared strict UTF-8 input validation for PTR text operations.
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn read_text(state: &LuaState, index: i32, label: &str) -> LuaResult<String> {
    let Val::Str(reference) = stack_val(state, index) else {
        return Err(runtime_error(format!("{label} text must be a string")));
    };
    let bytes = state
        .gc
        .string_arena
        .get(reference)
        .ok_or_else(|| runtime_error(format!("{label} text has been collected")))?;
    std::str::from_utf8(bytes.data())
        .map(str::to_owned)
        .map_err(|_| runtime_error(format!("{label} text must be valid UTF-8")))
}

pub(super) fn push_length(state: &mut LuaState, index: i32) -> LuaResult<u32> {
    let text = read_text(state, index, "length")?;
    state.push(Val::Num(text.chars().count() as f64));
    Ok(1)
}
