//! Catalogue-checked ICU4C currency metadata; native WoW/data equivalence is unverified.
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::c_api::intl_native::{self, CurrencyNameStyle};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

use super::number_formatting::{native_error, push_text, read_locale};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetCurrencyName", |s| {
        currency_name(s, false)
    })?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCurrencyFractionDigits",
        fraction_digits,
    )
}

pub(super) fn register_context(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "GetCurrencyName", |s| {
        currency_name(s, true)
    })
}

fn read_style(state: &LuaState, index: i32) -> LuaResult<CurrencyNameStyle> {
    match stack_val(state, index) {
        Val::Num(0.0) => Ok(CurrencyNameStyle::Symbol),
        Val::Num(1.0) => Ok(CurrencyNameStyle::NarrowSymbol),
        Val::Num(2.0) => Ok(CurrencyNameStyle::Long),
        Val::Num(3.0) => Ok(CurrencyNameStyle::FormalSymbol),
        Val::Num(4.0) => Ok(CurrencyNameStyle::VariantSymbol),
        _ => Err(runtime_error(
            "currency name style must be an integer from 0 to 4",
        )),
    }
}

fn currency_name(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let locale = read_locale(state, context)?;
    let code = super::text::read_text(state, first, "currency metadata")?;
    let style = read_style(state, first + 1)?;
    let Some(name) = intl_native::currency_name(&locale, &code, style).map_err(native_error)?
    else {
        return Ok(0);
    };
    push_text(state, &name)
}

fn fraction_digits(state: &mut LuaState) -> LuaResult<u32> {
    let code = super::text::read_text(state, 1, "currency metadata")?;
    let Some(digits) = intl_native::currency_fraction_digits(&code).map_err(native_error)? else {
        return Ok(0);
    };
    state.push(Val::Num(f64::from(digits)));
    Ok(1)
}
