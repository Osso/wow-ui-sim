//! R88k.2 ordinary argument conversions. Secret access uses the VM's existing
//! untainted-access policy, not a claim about unknown native policy writers.
use super::container_purchase::USAGE;
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

fn argument_error(state: &LuaState, index: i32, reason: &str) -> rilua::LuaError {
    if state.call_stack.is_empty() {
        return rilua::runtime_error(format!("bad argument #{index} ({reason})"));
    }
    rilua::stdlib::arg_error(state, index as usize, reason)
}

fn required(state: &LuaState, index: i32) -> LuaResult<Val> {
    let original = stack_val(state, index);
    if matches!(original, Val::Nil) {
        return Err(argument_error(state, index, USAGE));
    }
    unwrap_secret(state, original).map_err(|_| argument_error(state, index, &format!(
        "{USAGE}. Secret values are only allowed during untainted execution for this argument."
    )))
}

fn numeric(state: &LuaState, index: i32, min: f64, max: f64) -> LuaResult<f64> {
    let value = required(state, index)?;
    let number = match value {
        Val::Num(n) => Some(n),
        Val::Str(r) => state
            .gc
            .string_arena
            .get(r)
            .and_then(|s| super::parse_numeric_string(s.data())),
        _ => None,
    };
    number.filter(|n| (min..=max).contains(n)).ok_or_else(|| {
        argument_error(
            state,
            index,
            &format!("outside of expected range {min:.0} to {max:.0} - {USAGE}"),
        )
    })
}

pub(super) fn read(state: &LuaState) -> LuaResult<(i32, i32, bool)> {
    let container =
        (numeric(state, 1, -2147483648.0, 2147483647.0)?.trunc() as i32).wrapping_sub(1);
    let slot = (numeric(state, 2, 0.0, 4294967295.0)? - 1.0).trunc() as i64 as i32;
    let equipped = !matches!(required(state, 3)?, Val::Nil | Val::Bool(false));
    Ok((container, slot, equipped))
}
