//! ICU4C naming and transliteration; native WoW naming/error policies are unknown.
use rilua::LuaResult;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

use super::number_formatting::{native_error, push_text, read_locale};
use crate::c_api::intl_native;
use crate::lua_bridge::table_set_rust_fn_static;

pub(super) fn register(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "GetDisplayName", |s| display_name(s, false))?;
    table_set_rust_fn_static(state, table, "Transliterate", transliterate)
}

pub(super) fn register_context(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "GetDisplayName", |s| display_name(s, true))
}

fn display_name(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let target = read_locale(state, context)?;
    let index = if context { 2 } else { 1 };
    let display = super::text::read_text(state, index, "display locale")?;
    let display = super::parse_locale(display.as_bytes(), "display name")?.to_string();
    let result = intl_native::display_name(&target, &display).map_err(native_error)?;
    push_text(state, &result)
}

fn transliterate(state: &mut LuaState) -> LuaResult<u32> {
    let text = super::text::read_text(state, 1, "transliteration")?;
    let id = super::text::read_text(state, 2, "transliterator ID")?;
    let result = intl_native::transliterate(&text, &id).map_err(native_error)?;
    push_text(state, &result)
}
