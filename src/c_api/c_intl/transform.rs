//! Return-only ICU locale transforms; parent reduction is syntactic, not CLDR fallback.
use icu_locale::{LocaleCanonicalizer, LocaleExpander};
use icu_locale_core::Locale;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "TransformLocale", global)
}

pub(super) fn register_context(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "TransformLocale", context)
}

fn global(state: &mut LuaState) -> LuaResult<u32> {
    let bytes = super::storage::current_locale_bytes(state)?;
    apply(state, &bytes, 1)
}

fn context(state: &mut LuaState) -> LuaResult<u32> {
    let bytes = super::storage::context_locale_bytes(state)?;
    apply(state, &bytes, 2)
}

fn apply(state: &mut LuaState, bytes: &[u8], index: i32) -> LuaResult<u32> {
    let locale = super::parse_locale(bytes, "transform")?;
    let result = transform(locale, stack_val(state, index))?;
    let value = state.gc.intern_string(result.as_bytes());
    state.push(Val::Str(value));
    Ok(1)
}

fn transform(mut locale: Locale, operation: Val) -> LuaResult<String> {
    match operation {
        Val::Num(0.0) => {
            LocaleCanonicalizer::new_extended().canonicalize(&mut locale);
        }
        Val::Num(1.0) => {
            LocaleExpander::new_extended().maximize(&mut locale.id);
        }
        Val::Num(2.0) => {
            LocaleExpander::new_extended().minimize(&mut locale.id);
        }
        Val::Num(3.0) => return Ok(locale.id.language.to_string()),
        Val::Num(4.0) => return Ok(locale.id.script.map(|v| v.to_string()).unwrap_or_default()),
        Val::Num(5.0) => return Ok(locale.id.region.map(|v| v.to_string()).unwrap_or_default()),
        Val::Num(6.0) => {
            return Ok(locale
                .id
                .variants
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("-"));
        }
        Val::Num(7.0) => return Ok(parent_locale(&locale)),
        _ => {
            return Err(runtime_error(
                "locale transform must be an integer from 0 to 7",
            ));
        }
    }
    Ok(locale.to_string())
}

fn parent_locale(locale: &Locale) -> String {
    let tag = locale.to_string();
    let mut parts: Vec<&str> = tag.split('-').collect();
    let extension = parts
        .iter()
        .position(|part| *part == "x")
        .or_else(|| parts.iter().rposition(|part| part.len() == 1));
    if let Some(index) = extension {
        parts.truncate(index);
    } else if parts.len() > 1 {
        parts.pop();
    } else {
        return "und".to_owned();
    }
    parts.join("-")
}
