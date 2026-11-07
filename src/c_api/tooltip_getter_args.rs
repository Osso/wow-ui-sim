//! Secret-checked argument readers shared by the `C_TooltipInfo` selectors.
//!
//! WM94 §1 establishes the numeric parsers: every supplied numeric argument
//! passes a secret-access check before conversion, signed conversion rejects
//! non-integers, and unsigned/index conversion rejects negatives.

use crate::lua_bridge::{stack_val, val_to_i32};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Signed numeric argument (WM94 §1: `outside of expected range -2147483648 to 2147483647`).
pub(crate) fn signed(state: &LuaState, position: i32) -> LuaResult<i32> {
    val_to_i32(unwrap_secret(state, stack_val(state, position))?, position)
}

/// Unsigned numeric argument (WM94 §1: `outside of expected range 0 to 4294967295`).
pub(crate) fn unsigned(state: &LuaState, position: i32) -> LuaResult<u32> {
    match unwrap_secret(state, stack_val(state, position))? {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&number) =>
        {
            Ok(number as u32)
        }
        _ => Err(rilua::runtime_error(format!(
            "expected unsigned integer at argument {position}"
        ))),
    }
}

/// Optional unsigned numeric argument; a missing/nil value is skipped (WM94 §1).
pub(crate) fn optional_unsigned(state: &LuaState, position: i32) -> LuaResult<Option<u32>> {
    match stack_val(state, position) {
        Val::Nil => Ok(None),
        _ => unsigned(state, position).map(Some),
    }
}

/// One-based `luaIndex` argument converted to a zero-based ordinal.
///
/// The native index reader accepts the closed domain 0..4294967295 and converts
/// by subtracting one, then truncating toward zero (decomp Round 6 §2a; WM82:
/// "the index reader subtracts one before truncation"). A converted slot below
/// zero is an absent target, not an argument error, so Lua index 0 and a
/// fractional index below 1 select nothing and the first ordinal respectively.
/// Wrong types and out-of-domain numbers keep the shared reader's error.
pub(crate) fn lua_index(state: &LuaState, position: i32) -> LuaResult<Option<usize>> {
    let value = unwrap_secret(state, stack_val(state, position))?;
    let number = match value {
        Val::Num(number) if number.is_finite() && (0.0..=f64::from(u32::MAX)).contains(&number) => {
            number
        }
        _ => {
            return Err(rilua::runtime_error(format!(
                "expected unsigned integer at argument {position}"
            )));
        }
    };
    Ok(usize::try_from((number - 1.0) as i64).ok())
}

/// Optional boolean flag; a missing/nil value defaults false and a present value
/// uses Lua truthiness (WM94 §6), after the shared secret-access check.
/// Optional boolean flag; a missing/nil value defaults false and a present value
/// uses Lua truthiness (WM94 §6), after the shared secret-access check.
/// Used only by the brawl tooltip getter on the profiles that publish it.
#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
pub(crate) fn optional_bool(state: &LuaState, position: i32) -> LuaResult<bool> {
    let value = unwrap_secret(state, stack_val(state, position))?;
    Ok(value.is_truthy())
}

/// Optional owning-item link; WM94 §5 accepts a string or a number-convertible
/// value and skips a missing/nil one. The present value is secret-checked before
/// conversion; its parsed item fields only feed the native spell request.
pub(crate) fn optional_link(state: &LuaState, position: i32) -> LuaResult<Option<String>> {
    let value = unwrap_secret(state, stack_val(state, position))?;
    if value.is_nil() {
        return Ok(None);
    }
    crate::lua_api::methods::val_to_string(state, value)
        .map(Some)
        .ok_or_else(|| {
            rilua::runtime_error(format!("expected a link value at argument {position}"))
        })
}
