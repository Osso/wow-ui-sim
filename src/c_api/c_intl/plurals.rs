//! CLDR plural rules over shortest round-trip Lua numbers; native operand policy is unverified.
use fixed_decimal::{Decimal, FloatPrecision};
use icu_plurals::{PluralCategory, PluralRules};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "SelectPlural", |s| select(s, false))
}

pub(super) fn register_context(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SelectPlural", |s| select(s, true))
}

fn read_number(state: &LuaState, index: i32) -> LuaResult<Decimal> {
    let Val::Num(number) = stack_val(state, index) else {
        return Err(runtime_error("plural operand must be a number"));
    };
    if !number.is_finite() {
        return Err(runtime_error("plural operand must be finite"));
    }
    Decimal::try_from_f64(number.abs(), FloatPrecision::RoundTrip)
        .map_err(|error| runtime_error(format!("cannot convert plural operand: {error}")))
}

fn create_rules(state: &mut LuaState, context: bool, index: i32) -> LuaResult<PluralRules> {
    let bytes = if context {
        super::storage::context_locale_bytes(state)?
    } else {
        super::storage::current_locale_bytes(state)?
    };
    let locale = super::parse_locale(&bytes, "plural")?;
    let rules = match stack_val(state, index) {
        Val::Num(0.0) => PluralRules::try_new_cardinal(locale.into()),
        Val::Num(1.0) => PluralRules::try_new_ordinal(locale.into()),
        _ => {
            return Err(runtime_error(
                "plural type must be Cardinal (0) or Ordinal (1)",
            ));
        }
    };
    rules.map_err(|error| runtime_error(format!("cannot create ICU plural rules: {error}")))
}

fn category_name(category: PluralCategory) -> &'static str {
    match category {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    }
}

fn select(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let number = read_number(state, first)?;
    let rules = create_rules(state, context, first + 1)?;
    let category = category_name(rules.category_for(&number));
    let result = state.gc.intern_string(category.as_bytes());
    state.push(Val::Str(result));
    Ok(1)
}
