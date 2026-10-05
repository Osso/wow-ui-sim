//! Severity colors and synthetic Edit Mode previews, not encounter warning storage or dispatch.
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::lua_api::methods::{call_function_state, create_string, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

const PREVIEW_DURATION_SECONDS: f64 = 5.0;
const PREVIEW_ICON_FILE_ID: f64 = 136122.0;

pub(crate) fn register_preview(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetColorForSeverity", color_for_severity)?;
    table_set_rust_fn_static(state, namespace, "GetEditModeWarningInfo", preview)
}

/// Simulator palette (white, amber, red); the native client's colors are not documented.
fn severity_rgb(severity: u8) -> (f64, f64, f64) {
    match severity {
        0 => (1.0, 1.0, 1.0),
        1 => (1.0, 0.75, 0.1),
        _ => (1.0, 0.15, 0.05),
    }
}

/// Fresh ColorMixin per call so callers can mutate their copy.
fn create_severity_color(state: &mut LuaState, severity: u8) -> LuaResult<Val> {
    let (red, green, blue) = severity_rgb(severity);
    let factory = crate::c_api::global_val(state, "CreateColor");
    call_function_state(
        state,
        factory,
        &[
            Val::Num(red),
            Val::Num(green),
            Val::Num(blue),
            Val::Num(1.0),
        ],
    )
}

fn color_for_severity(state: &mut LuaState) -> LuaResult<u32> {
    let severity = read_severity(state)?;
    let color = create_severity_color(state, severity)?;
    state.push(color);
    Ok(1)
}

fn read_severity(state: &LuaState) -> LuaResult<u8> {
    match stack_val(state, 1) {
        Val::Num(0.0) => Ok(0),
        Val::Num(1.0) => Ok(1),
        Val::Num(2.0) => Ok(2),
        _ => Err(runtime_error(
            "warning severity must be Low (0), Medium (1), or High (2)",
        )),
    }
}

fn preview(state: &mut LuaState) -> LuaResult<u32> {
    let severity = read_severity(state)?;
    let text = match severity {
        0 => "Simulated Low Warning",
        1 => "Simulated Medium Warning",
        _ => "Simulated High Warning",
    };
    let color = create_severity_color(state, severity)?;
    let info = create_table(state);
    publish_preview_identity(state, info, text);
    publish_preview_display(state, info, severity);
    table_set(state, info, "color", color);
    state.push(info);
    Ok(1)
}

fn publish_preview_identity(state: &mut LuaState, info: Val, text: &str) {
    for (field, value) in [
        ("text", text),
        ("casterGUID", "Sim-Warning-Caster"),
        ("casterName", "Simulator Caster"),
        ("targetGUID", "Sim-Warning-Target"),
        ("targetName", "Simulator Target"),
    ] {
        let value = create_string(state, value);
        table_set(state, info, field, value);
    }
}

fn publish_preview_display(state: &mut LuaState, info: Val, severity: u8) {
    for (field, value) in [
        ("iconFileID", Val::Num(PREVIEW_ICON_FILE_ID)),
        ("tooltipSpellID", Val::Num(0.0)),
        ("isDeadly", Val::Bool(severity == 2)),
        ("duration", Val::Num(PREVIEW_DURATION_SECONDS)),
        ("severity", Val::Num(f64::from(severity))),
        ("shouldPlaySound", Val::Bool(false)),
        ("shouldShowChatMessage", Val::Bool(false)),
        ("shouldShowWarning", Val::Bool(true)),
    ] {
        table_set(state, info, field, value);
    }
}
