//! Legacy C_Debug output retained only on profiles predating retail retirement.

use crate::lua_api::methods::{borrow_state_mut, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Debug")?;
    for name in ["PrintToDebugWindow", "ViewInDebugWindow"] {
        table_set_rust_fn_static(state, namespace, name, append_debug_output)?;
    }
    Ok(())
}

fn append_debug_output(state: &mut LuaState) -> LuaResult<u32> {
    let nargs = (state.top as i32 - state.base as i32).max(0) as usize;
    let line = (1..=nargs)
        .map(|index| format_stack_value(state, stack_val(state, index as i32)))
        .collect::<Vec<_>>()
        .join("\t");
    borrow_state_mut(state)?.console_output.push(line);
    Ok(0)
}

fn format_stack_value(state: &LuaState, value: Val) -> String {
    match value {
        Val::Nil => "nil".to_string(),
        Val::Bool(true) => "true".to_string(),
        Val::Bool(false) => "false".to_string(),
        Val::Num(number) => format_stack_number(number),
        Val::Str(_) => val_to_string(state, value).unwrap_or_default(),
        _ => format!("{value:?}"),
    }
}

fn format_stack_number(number: f64) -> String {
    if number.fract() == 0.0 && number.abs() < 1e15 {
        format!("{}", number as i64)
    } else {
        format!("{number}")
    }
}
