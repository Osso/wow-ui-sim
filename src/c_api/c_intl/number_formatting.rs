//! ICU4C number/currency operations; native WoW defaults and errors are unverified.
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::c_api::intl_native::{self, NumberStyle};
use crate::lua_api::methods::table_set_static;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "FormatNumber", |s| {
        format_number(s, false)
    })?;
    table_set_rust_fn_static(state, namespace, "ParseNumber", |s| parse_number(s, false))?;
    table_set_rust_fn_static(state, namespace, "FormatCurrency", |s| {
        format_currency(s, false)
    })?;
    table_set_rust_fn_static(state, namespace, "ParseCurrency", |s| {
        parse_currency(s, false)
    })
}

pub(super) fn register_context(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "FormatNumber", |s| format_number(s, true))?;
    table_set_rust_fn_static(state, metatable, "ParseNumber", |s| parse_number(s, true))?;
    table_set_rust_fn_static(state, metatable, "FormatCurrency", |s| {
        format_currency(s, true)
    })?;
    table_set_rust_fn_static(state, metatable, "ParseCurrency", |s| {
        parse_currency(s, true)
    })
}

pub(super) fn read_locale(state: &mut LuaState, context: bool) -> LuaResult<String> {
    let bytes = if context {
        super::storage::context_locale_bytes(state)?
    } else {
        super::storage::current_locale_bytes(state)?
    };
    super::parse_locale(&bytes, "number formatting").map(|locale| locale.to_string())
}

fn read_number(state: &LuaState, index: i32) -> LuaResult<f64> {
    match stack_val(state, index) {
        Val::Num(value) => Ok(value),
        _ => Err(runtime_error("formatting operand must be a number")),
    }
}

fn read_style(state: &LuaState, index: i32) -> LuaResult<NumberStyle> {
    match stack_val(state, index) {
        Val::Num(0.0) => Ok(NumberStyle::Decimal),
        Val::Num(1.0) => Ok(NumberStyle::Integer),
        Val::Num(2.0) => Ok(NumberStyle::Percent),
        Val::Num(3.0) => Ok(NumberStyle::Currency),
        _ => Err(runtime_error("number style must be an integer from 0 to 3")),
    }
}

pub(super) fn native_error(error: intl_native::Error) -> rilua::LuaError {
    runtime_error(error.to_string())
}

pub(super) fn push_text(state: &mut LuaState, text: &str) -> LuaResult<u32> {
    let value = state.gc.intern_string(text.as_bytes());
    state.push(Val::Str(value));
    Ok(1)
}

fn format_number(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let locale = read_locale(state, context)?;
    let number = read_number(state, first)?;
    let style = read_style(state, first + 1)?;
    let result = intl_native::format_number(&locale, number, style).map_err(native_error)?;
    push_text(state, &result)
}

fn format_currency(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let locale = read_locale(state, context)?;
    let number = read_number(state, first)?;
    let code = super::text::read_text(state, first + 1, "currency")?;
    let result = intl_native::format_currency(&locale, number, &code).map_err(native_error)?;
    push_text(state, &result)
}

fn parse_number(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let locale = read_locale(state, context)?;
    let text = super::text::read_text(state, first, "number parsing")?;
    let style = read_style(state, first + 1)?;
    let Some(number) = intl_native::parse_number(&locale, &text, style).map_err(native_error)?
    else {
        return Ok(0);
    };
    state.push(Val::Num(number));
    Ok(1)
}

fn parse_currency(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let locale = read_locale(state, context)?;
    let text = super::text::read_text(state, first, "currency parsing")?;
    let Some(parsed) = intl_native::parse_currency(&locale, &text).map_err(native_error)? else {
        return Ok(0);
    };
    let code = state.gc.intern_string(parsed.currency_code.as_bytes());
    let result = state.gc.alloc_table(Table::with_sizes(0, 2));
    table_set_static(state, Val::Table(result), "amount", Val::Num(parsed.amount));
    table_set_static(state, Val::Table(result), "currencyCode", Val::Str(code));
    state.push(Val::Table(result));
    Ok(1)
}
