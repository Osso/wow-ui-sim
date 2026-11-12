//! Secret-checked argument readers shared by the `C_TooltipInfo` selectors.
//!
//! WM94 §1 establishes the numeric parsers: every supplied numeric argument
//! passes a secret-access check before conversion, signed conversion rejects
//! non-integers, and unsigned/index conversion rejects negatives.

use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::{stack_val, val_to_i32};
use rilua::table_security::{can_access_secrets, is_secret_value, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Signed numeric argument (WM94 §1: `outside of expected range -2147483648 to 2147483647`).
pub(crate) fn signed(state: &LuaState, position: i32) -> LuaResult<i32> {
    val_to_i32(unwrap_secret(state, stack_val(state, position))?, position)
}

/// Apply the currently modelled AllowedWhenUntainted policy using the
/// native argument-error wording. R7's other access-mode messages are unknown
/// to the VM and are not inferred here.
pub(crate) fn allowed_when_untainted(
    state: &LuaState,
    position: i32,
    api_name: &str,
) -> LuaResult<Val> {
    let value = stack_val(state, position);
    if is_secret_value(state, value) && !can_access_secrets(state) {
        return Err(rilua::runtime_error(format!(
            "{api_name}: Secret values are only allowed during untainted execution for this argument."
        )));
    }
    unwrap_secret(state, value)
}

/// Round 7 numeric parser: accepts numeric strings, checks the full numeric
/// range before truncating toward zero, and reports the getter's baked usage.
pub(crate) fn round7_signed(state: &LuaState, position: i32, usage: &str) -> LuaResult<i32> {
    let value = allowed_when_untainted(state, position, tooltip_api_name(usage))?;
    let number =
        numeric_argument(state, value).ok_or_else(|| rilua::runtime_error(usage.to_string()))?;
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(rilua::runtime_error(format!(
            "outside of expected range -2147483648 to 2147483647 - {usage}"
        )));
    }
    Ok(number.trunc() as i32)
}

/// Round 7 optional unsigned rank. Absent or nil is not an argument error.
pub(crate) fn round7_optional_unsigned(
    state: &LuaState,
    position: i32,
    usage: &str,
) -> LuaResult<Option<u32>> {
    if stack_val(state, position).is_nil() {
        return Ok(None);
    }
    let value = allowed_when_untainted(state, position, tooltip_api_name(usage))?;
    let number =
        numeric_argument(state, value).ok_or_else(|| rilua::runtime_error(usage.to_string()))?;
    if !number.is_finite() || number < 0.0 || number > f64::from(u32::MAX) {
        return Err(rilua::runtime_error(format!(
            "outside of expected range 0 to 4294967295 - {usage}"
        )));
    }
    Ok(Some(number.trunc() as u32))
}

/// Optional strict boolean used by Round 7's party-progress flags. The native
/// invalid-boolean diagnostic is not established; preserve the public usage.
pub(crate) fn round7_optional_bool(
    state: &LuaState,
    position: i32,
    usage: &str,
) -> LuaResult<bool> {
    match allowed_when_untainted(state, position, tooltip_api_name(usage))? {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(rilua::runtime_error(usage.to_string())),
    }
}

/// Convert the GUID reader's string-or-number inputs without inventing an
/// identity mapping. An unparseable string is a miss, as in the native reader.
pub(crate) fn round7_guid(state: &LuaState, position: i32, usage: &str) -> LuaResult<String> {
    let value = allowed_when_untainted(state, position, tooltip_api_name(usage))?;
    match value {
        Val::Num(number) => Ok(number.to_string()),
        Val::Str(_) => Ok(val_to_string(state, value).unwrap_or_default()),
        _ => Err(rilua::runtime_error(usage.to_string())),
    }
}

fn tooltip_api_name(usage: &str) -> &str {
    usage
        .strip_prefix("Usage: local data = ")
        .and_then(|name| name.split_once('(').map(|(name, _)| name))
        .unwrap_or("C_TooltipInfo")
}

fn numeric_argument(state: &LuaState, value: Val) -> Option<f64> {
    match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => val_to_string(state, value)?.trim().parse().ok(),
        _ => None,
    }
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

/// One-based `luaIndex` argument converted to a zero-based ordinal.
///
/// The native index reader accepts the closed domain 0..4294967295 and converts
/// by subtracting one, then truncating toward zero (decomp Round 6 §2a; WM82:
/// "the index reader subtracts one before truncation"). A converted slot below
/// zero is an absent target, not an argument error, so Lua index 0 and a
/// fractional index below 1 select nothing and the first ordinal respectively.
/// Wrong types and out-of-domain numbers keep the shared reader's error.
///
/// Only the Retail/PTR merchant-cost getter reads this conversion today, so the
/// helper is compiled with that epoch to keep other profiles warning-free.
#[cfg(feature = "retail-12-1-5")]
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
