//! `time([dateTable])`, WoW's global alias of Lua 5.1 `os.time`.
//!
//! Lua 5.1 `os.time` reads date fields with `lua_isnumber`, which accepts
//! numeric strings; rilua's `os.time` reads only numbers, so string fields are
//! converted here before the call.

use crate::lua_api::methods::{create_table, table_get, table_set_static, val_to_string};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const DATE_FIELDS: [&str; 7] = ["sec", "min", "hour", "day", "month", "year", "isdst"];

fn time(state: &mut LuaState) -> LuaResult<u32> {
    let base = state.base;
    if base < state.top && matches!(state.stack[base], Val::Table(_)) {
        let date = numeric_date_table(state, state.stack[base]);
        state.stack[base] = date;
    }
    rilua::stdlib::os::os_time(state)
}

fn numeric_date_table(state: &mut LuaState, source: Val) -> Val {
    let date = create_table(state);
    for field in DATE_FIELDS {
        let value = table_get(state, source, field);
        let value = numeric_string_value(state, value);
        table_set_static(state, date, field, value);
    }
    date
}

fn numeric_string_value(state: &LuaState, value: Val) -> Val {
    if !matches!(value, Val::Str(_)) {
        return value;
    }
    val_to_string(state, value)
        .and_then(|text| text.trim().parse::<f64>().ok())
        .map_or(value, Val::Num)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "time", time)
}
