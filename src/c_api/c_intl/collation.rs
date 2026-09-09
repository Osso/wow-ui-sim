//! ICU collation; native key bytes and failure/security semantics remain unverified.
use std::cmp::Ordering;

use icu_collator::CollatorBorrowed;
use icu_collator::options::{CollatorOptions, Strength};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "CompareStrings", |s| compare(s, false))?;
    table_set_rust_fn_static(state, namespace, "GetSortKey", |s| sort_key(s, false))
}

pub(super) fn register_context(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "CompareStrings", |s| compare(s, true))?;
    table_set_rust_fn_static(state, table, "GetSortKey", |s| sort_key(s, true))
}

fn read_strength(state: &LuaState, index: i32) -> LuaResult<Strength> {
    match stack_val(state, index) {
        Val::Num(0.0) => Ok(Strength::Primary),
        Val::Num(1.0) => Ok(Strength::Secondary),
        Val::Num(2.0) => Ok(Strength::Tertiary),
        Val::Num(3.0) => Ok(Strength::Quaternary),
        Val::Num(4.0) => Ok(Strength::Identical),
        _ => Err(runtime_error(
            "collation strength must be an integer from 0 to 4",
        )),
    }
}

fn create_collator(
    state: &mut LuaState,
    context: bool,
    strength: Strength,
) -> LuaResult<CollatorBorrowed<'static>> {
    let bytes = if context {
        super::storage::context_locale_bytes(state)?
    } else {
        super::storage::current_locale_bytes(state)?
    };
    let locale = super::parse_locale(&bytes, "collation")?;
    let mut options = CollatorOptions::default();
    options.strength = Some(strength);
    CollatorBorrowed::try_new(locale.into(), options)
        .map_err(|error| runtime_error(format!("cannot create ICU collator: {error}")))
}

fn compare(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let left = super::text::read_text(state, first, "collation")?;
    let right = super::text::read_text(state, first + 1, "collation")?;
    let strength = read_strength(state, first + 2)?;
    let collator = create_collator(state, context, strength)?;
    let result = match collator.compare(&left, &right) {
        Ordering::Less => -1.0,
        Ordering::Equal => 0.0,
        Ordering::Greater => 1.0,
    };
    state.push(Val::Num(result));
    Ok(1)
}

fn sort_key(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let text = super::text::read_text(state, first, "collation")?;
    let strength = read_strength(state, first + 1)?;
    let collator = create_collator(state, context, strength)?;
    let mut bytes = Vec::new();
    let Ok(()) = collator.write_sort_key_to(&text, &mut bytes);
    let key = state.gc.intern_string(&bytes);
    state.push(Val::Str(key));
    Ok(1)
}
