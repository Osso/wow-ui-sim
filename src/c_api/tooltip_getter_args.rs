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

/// One-based index argument (Round 6 §GetBuybackItem(a); the same reader family
/// serves the loot getters and `GetTotem`).
///
/// VERIFIED: a number is required, the accepted domain is 0 to 4294967295, and the
/// value is reduced by 1.0 then truncated toward zero with no integral test, so 1.9
/// selects the first entry. A raw 0 becomes unsigned all-ones, which the caller's
/// live-count bound rejects like any other past-count index.
///
/// `None` means "past the end of the list": the caller publishes its miss arm.
pub(crate) fn one_based_index(
    state: &LuaState,
    position: i32,
    usage: &str,
) -> LuaResult<Option<usize>> {
    // VERIFIED: a non-number takes the same argument-error route as a missing one.
    let Val::Num(number) = unwrap_secret(state, stack_val(state, position))? else {
        return Err(rilua::runtime_error(usage.to_string()));
    };
    // VERIFIED: negatives, values above 4294967295 and non-finite values compose
    // `outside of expected range 0 to 4294967295 - <usage>`.
    if !(0.0..=f64::from(u32::MAX)).contains(&number) {
        return Err(rilua::runtime_error(format!(
            "outside of expected range 0 to 4294967295 - {usage}"
        )));
    }
    let truncated = (number - 1.0).trunc();
    Ok((truncated >= 0.0).then_some(truncated as usize))
}

/// Optional unsigned numeric argument; a missing/nil value is skipped (WM94 §1).
pub(crate) fn optional_unsigned(state: &LuaState, position: i32) -> LuaResult<Option<u32>> {
    match stack_val(state, position) {
        Val::Nil => Ok(None),
        _ => unsigned(state, position).map(Some),
    }
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
