//! Supplied zone-control entries projected as fresh Lua snapshots.
use super::queries::read_numeric_id;
use super::{WidgetId, WidgetVisualization, ZoneControlRecord, ZoneEntryRecord};
use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        return Ok(());
    }
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(state, namespace, "GetZoneControlVisualizationInfo", query)
}

fn query(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetZoneControlVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(WidgetId::from_number(id), &inputs.zone_controls, |value| {
            match value {
                WidgetVisualization::ZoneControl(record) => Some(record),
                _ => None,
            }
        })
    };
    let Some(record) = record else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let row = create_table(state);
    state.push(row);
    write_record(state, row, &record);
    Ok(1)
}

fn write_record(state: &mut LuaState, row: Val, record: &ZoneControlRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("mode", f64::from(record.mode)),
        ("leadingEdgeType", f64::from(record.leading_edge_type)),
        ("dangerFlashType", f64::from(record.danger_flash_type)),
        ("tooltipLoc", f64::from(record.tooltip_loc)),
        ("widgetSizeSetting", record.widget_size_setting),
        ("orderIndex", record.order_index),
        ("inAnimType", f64::from(record.in_anim_type)),
        ("outAnimType", f64::from(record.out_anim_type)),
        ("widgetScale", f64::from(record.widget_scale)),
        ("layoutDirection", f64::from(record.layout_direction)),
        ("modelSceneLayer", f64::from(record.model_scene_layer)),
        (
            "scriptedAnimationEffectID",
            record.scripted_animation_effect_id,
        ),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
    for (key, text) in [
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    let entries = create_table(state);
    table_set(state, row, "zoneEntries", entries);
    for (index, entry) in record.zone_entries.iter().enumerate() {
        let child = create_table(state);
        set_table_array(state, entries, (index + 1) as i64, child);
        write_zone_entry(state, child, entry);
    }
}

pub(super) fn write_zone_entry(state: &mut LuaState, row: Val, record: &ZoneEntryRecord) {
    for (key, value) in [
        ("state", f64::from(record.state)),
        ("activeState", f64::from(record.active_state)),
        ("fillType", f64::from(record.fill_type)),
        ("min", record.min),
        ("max", record.max),
        ("current", record.current),
        ("capturePoint", record.capture_point),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
    let tooltip = create_string(state, &record.tooltip);
    table_set(state, row, "tooltip", tooltip);
}
