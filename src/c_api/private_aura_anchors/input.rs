//! Complete, secret-aware registration parsing before record or ID mutation.

use super::{AnchorBinding, AnchorRecord, IconInfo};
use crate::lua_api::methods::{borrow_state, native_frame_id_from_val, table_get, val_to_string};
use rilua::table_security::{check_table_access, is_secret_value};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn reject_secret(state: &LuaState, value: Val, field: &str) -> LuaResult<()> {
    if is_secret_value(state, value) {
        return Err(runtime_error(format!(
            "private aura anchor {field}: secret access is not modeled"
        )));
    }
    Ok(())
}

fn read_table(state: &LuaState, value: Val, field: &str) -> LuaResult<Val> {
    reject_secret(state, value, field)?;
    let Val::Table(reference) = value else {
        return Err(runtime_error(format!(
            "private aura anchor {field} must be a table"
        )));
    };
    check_table_access(state, reference, None)?;
    Ok(value)
}

fn read_field(state: &mut LuaState, table: Val, field: &str) -> LuaResult<Val> {
    let Val::Table(reference) = table else {
        return Err(runtime_error(
            "private aura anchor structure must be a table",
        ));
    };
    check_table_access(state, reference, None)?;
    let value = table_get(state, table, field);
    reject_secret(state, value, field)?;
    Ok(value)
}

pub(super) fn read_number(state: &LuaState, value: Val, field: &str) -> LuaResult<f64> {
    reject_secret(state, value, field)?;
    match value {
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(runtime_error(format!(
            "private aura anchor {field} must be a finite number"
        ))),
    }
}

pub(super) fn read_string(state: &LuaState, value: Val, field: &str) -> LuaResult<String> {
    reject_secret(state, value, field)?;
    if !matches!(value, Val::Str(_)) {
        return Err(runtime_error(format!(
            "private aura anchor {field} must be a string"
        )));
    }
    val_to_string(state, value)
        .ok_or_else(|| runtime_error(format!("private aura anchor {field} must be UTF-8")))
}

fn number_field(state: &mut LuaState, table: Val, field: &str) -> LuaResult<f64> {
    let value = read_field(state, table, field)?;
    read_number(state, value, field)
}

fn string_field(state: &mut LuaState, table: Val, field: &str) -> LuaResult<String> {
    let value = read_field(state, table, field)?;
    read_string(state, value, field)
}

fn bool_field(state: &mut LuaState, table: Val, field: &str) -> LuaResult<bool> {
    match read_field(state, table, field)? {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(format!(
            "private aura anchor {field} must be a boolean"
        ))),
    }
}

fn frame_field(state: &mut LuaState, table: Val, field: &str) -> LuaResult<u64> {
    let value = read_field(state, table, field)?;
    read_table(state, value, field)?;
    let id = native_frame_id_from_val(state, value)
        .ok_or_else(|| runtime_error(format!("private aura anchor {field} must be a frame")))?;
    if borrow_state(state)?.widgets.get(id).is_none() {
        return Err(runtime_error(format!(
            "private aura anchor {field} frame does not exist"
        )));
    }
    // frame_ref's registry cache and pinned backing table retain original fields through GC.
    Ok(id)
}

fn point_field(
    state: &mut LuaState,
    table: Val,
    field: &str,
) -> LuaResult<crate::widget::AnchorPoint> {
    let value = string_field(state, table, field)?;
    crate::widget::AnchorPoint::from_str(&value)
        .ok_or_else(|| runtime_error(format!("private aura anchor {field} is not a frame point")))
}

fn read_binding(state: &mut LuaState, value: Val, field: &str) -> LuaResult<AnchorBinding> {
    let table = read_table(state, value, field)?;
    Ok(AnchorBinding {
        point: point_field(state, table, "point")?,
        relative_to: frame_field(state, table, "relativeTo")?,
        relative_point: point_field(state, table, "relativePoint")?,
        offset_x: number_field(state, table, "offsetX")?,
        offset_y: number_field(state, table, "offsetY")?,
    })
}

fn read_icon(state: &mut LuaState, value: Val) -> LuaResult<IconInfo> {
    let table = read_table(state, value, "iconInfo")?;
    let anchor = read_field(state, table, "iconAnchor")?;
    let border = read_field(state, table, "borderScale")?;
    Ok(IconInfo {
        anchor: read_binding(state, anchor, "iconAnchor")?,
        width: number_field(state, table, "iconWidth")?,
        height: number_field(state, table, "iconHeight")?,
        border_scale: if border == Val::Nil {
            None
        } else {
            Some(read_number(state, border, "borderScale")?)
        },
    })
}

pub(super) fn read_record(state: &mut LuaState, value: Val) -> LuaResult<AnchorRecord> {
    let table = read_table(state, value, "args")?;
    let icon = read_field(state, table, "iconInfo")?;
    let duration = read_field(state, table, "durationAnchor")?;
    Ok(AnchorRecord {
        unit_token: string_field(state, table, "unitToken")?,
        aura_index: number_field(state, table, "auraIndex")?,
        parent: frame_field(state, table, "parent")?,
        show_cooldown_frame: bool_field(state, table, "showCooldownFrame")?,
        show_cooldown_edge: bool_field(state, table, "showCooldownEdge")?,
        show_countdown_numbers: bool_field(state, table, "showCountdownNumbers")?,
        show_dispel_icon: bool_field(state, table, "showDispelIcon")?,
        is_container: bool_field(state, table, "isContainer")?,
        icon: if icon == Val::Nil {
            None
        } else {
            Some(read_icon(state, icon)?)
        },
        duration_anchor: if duration == Val::Nil {
            None
        } else {
            Some(read_binding(state, duration, "durationAnchor")?)
        },
    })
}
