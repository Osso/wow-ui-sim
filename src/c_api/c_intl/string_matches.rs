//! ICU4C collation search; byte-index and failure policies are simulator choices.
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::c_api::intl_native::{self, SearchStrength};
use crate::lua_api::methods::table_set_num;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "FindStringMatches", |s| {
        find_matches(s, false)
    })
}

pub(super) fn register_context(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "FindStringMatches", |s| {
        find_matches(s, true)
    })
}

fn read_strength(state: &LuaState, index: i32) -> LuaResult<SearchStrength> {
    match stack_val(state, index) {
        Val::Num(0.0) => Ok(SearchStrength::Primary),
        Val::Num(1.0) => Ok(SearchStrength::Secondary),
        Val::Num(2.0) => Ok(SearchStrength::Tertiary),
        Val::Num(3.0) => Ok(SearchStrength::Quaternary),
        Val::Num(4.0) => Ok(SearchStrength::Identical),
        _ => Err(runtime_error(
            "search strength must be an integer from 0 to 4",
        )),
    }
}

fn find_matches(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let locale = super::number_formatting::read_locale(state, context)?;
    let text = super::text::read_text(state, first, "search")?;
    let pattern = super::text::read_text(state, first + 1, "search pattern")?;
    let strength = read_strength(state, first + 2)?;
    let offsets = intl_native::find_string_matches(&locale, &text, &pattern, strength)
        .map_err(super::number_formatting::native_error)?;
    let result = state.gc.alloc_table(Table::with_sizes(offsets.len(), 0));
    for (index, offset) in offsets.into_iter().enumerate() {
        table_set_num(state, result, (index + 1) as f64, Val::Num(offset as f64));
    }
    state.push(Val::Table(result));
    Ok(1)
}
