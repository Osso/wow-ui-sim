//! Explicit-zone ICU4C date formatting; native WoW policies remain unverified.
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use super::number_formatting::{native_error, push_text, read_locale};
use crate::c_api::intl_native::{self, DateTimeStyle};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

#[derive(Clone, Copy)]
enum Kind {
    Date,
    Time,
    Both,
}

pub(super) fn register(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "FormatDate", |s| format(s, false, Kind::Date))?;
    table_set_rust_fn_static(state, table, "FormatTime", |s| format(s, false, Kind::Time))?;
    table_set_rust_fn_static(state, table, "FormatDateTime", |s| {
        format(s, false, Kind::Both)
    })
}

pub(super) fn register_context(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "FormatDate", |s| format(s, true, Kind::Date))?;
    table_set_rust_fn_static(state, table, "FormatTime", |s| format(s, true, Kind::Time))?;
    table_set_rust_fn_static(state, table, "FormatDateTime", |s| {
        format(s, true, Kind::Both)
    })
}

fn read_style(state: &LuaState, index: i32) -> LuaResult<DateTimeStyle> {
    match stack_val(state, index) {
        Val::Num(0.0) => Ok(DateTimeStyle::None),
        Val::Num(1.0) => Ok(DateTimeStyle::Short),
        Val::Num(2.0) => Ok(DateTimeStyle::Medium),
        Val::Num(3.0) => Ok(DateTimeStyle::Long),
        Val::Num(4.0) => Ok(DateTimeStyle::Full),
        _ => Err(runtime_error(
            "date/time style must be an integer from 0 to 4",
        )),
    }
}

fn read_styles(
    state: &LuaState,
    first: i32,
    kind: Kind,
) -> LuaResult<(DateTimeStyle, DateTimeStyle, i32)> {
    let style = read_style(state, first + 1)?;
    match kind {
        Kind::Date => Ok((style, DateTimeStyle::None, first + 2)),
        Kind::Time => Ok((DateTimeStyle::None, style, first + 2)),
        Kind::Both => Ok((style, read_style(state, first + 2)?, first + 3)),
    }
}

fn format(state: &mut LuaState, context: bool, kind: Kind) -> LuaResult<u32> {
    let first = if context { 2 } else { 1 };
    let locale = read_locale(state, context)?;
    let Val::Num(seconds) = stack_val(state, first) else {
        return Err(runtime_error("Unix time seconds must be a number"));
    };
    let (date, time, zone_index) = read_styles(state, first, kind)?;
    let zone = super::text::read_text(state, zone_index, "date/time zone")?;
    let text =
        intl_native::format_date_time(&locale, seconds, date, time, &zone).map_err(native_error)?;
    push_text(state, &text)
}
