use super::model::EventInfo;
use crate::lua_api::methods::table_get;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn read(state: &LuaState) -> LuaResult<(EventInfo, bool)> {
    let request = stack_val(state, 1);
    if !matches!(request, Val::Table(_)) {
        return Err(runtime_error("AddScriptEvent requires a request table"));
    }
    let spell_id = integer(state, request, "spellID", None, u32::MAX)?;
    let override_name = text(state, request, "overrideName")?;
    let name = if override_name.is_empty() {
        crate::spells::get_spell(spell_id)
            .map_or_else(Vec::new, |spell| spell.name.as_bytes().to_vec())
    } else {
        override_name
    };
    Ok((
        EventInfo {
            id: 0,
            spell_id,
            name,
            icon: integer(state, request, "iconFileID", None, u32::MAX)?,
            duration: seconds(state, request, "duration", None)?,
            max_queue_duration: seconds(state, request, "maxQueueDuration", Some(0.0))?,
            icons: integer(state, request, "icons", Some(0.0), u32::MAX)?,
            severity: integer(state, request, "severity", Some(1.0), 2)? as u8,
        },
        boolean(state, request, "paused")?,
    ))
}

fn number(state: &LuaState, table: Val, key: &str, default: Option<f64>) -> LuaResult<f64> {
    let value = match table_get(state, table, key) {
        Val::Num(value) => Some(value),
        Val::Nil => default,
        _ => None,
    };
    value
        .filter(|value| value.is_finite())
        .ok_or_else(|| runtime_error(format!("AddScriptEvent {key} must be a finite number")))
}

fn seconds(state: &LuaState, table: Val, key: &str, default: Option<f64>) -> LuaResult<f64> {
    let value = number(state, table, key, default)?;
    if value < 0.0 {
        return Err(runtime_error(format!(
            "AddScriptEvent {key} must be nonnegative"
        )));
    }
    Ok(value)
}

fn integer(
    state: &LuaState,
    table: Val,
    key: &str,
    default: Option<f64>,
    maximum: u32,
) -> LuaResult<u32> {
    let value = number(state, table, key, default)?;
    let in_range = (0.0..=f64::from(maximum)).contains(&value);
    if !in_range || value.fract() != 0.0 {
        return Err(runtime_error(format!(
            "AddScriptEvent {key} must be a supported integer"
        )));
    }
    Ok(value as u32)
}

fn text(state: &LuaState, table: Val, key: &str) -> LuaResult<Vec<u8>> {
    match table_get(state, table, key) {
        Val::Nil => Ok(Vec::new()),
        Val::Str(value) => state
            .gc
            .string_arena
            .get(value)
            .map(|s| s.data().to_vec())
            .ok_or_else(|| runtime_error("request string was collected")),
        _ => Err(runtime_error(format!(
            "AddScriptEvent {key} must be a string"
        ))),
    }
}

fn boolean(state: &LuaState, table: Val, key: &str) -> LuaResult<bool> {
    match table_get(state, table, key) {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(format!(
            "AddScriptEvent {key} must be a boolean"
        ))),
    }
}
