//! Complete supplied bar records projected as fresh Lua snapshots.

use super::queries::read_numeric_id;
use super::{CaptureBarRecord, StatusBarRecord, WidgetId};
use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetStatusBarWidgetVisualizationInfo",
        query_status_bar,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCaptureBarWidgetVisualizationInfo",
        query_capture_bar,
    )
}

fn query_status_bar(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetStatusBarWidgetVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(WidgetId::from_number(id), &inputs.status_bars, |value| {
            match value {
                super::WidgetVisualization::StatusBar(record) => Some(record),
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
    write_status_bar_numbers(state, row, &record);
    write_status_text_numbers(state, row, &record);
    write_status_presentation_numbers(state, row, &record);
    write_status_strings(state, row, &record);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    write_partition_values(state, row, &record.partition_values);
    Ok(1)
}

fn query_capture_bar(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetCaptureBarWidgetVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(WidgetId::from_number(id), &inputs.capture_bars, |value| {
            match value {
                super::WidgetVisualization::CaptureBar(record) => Some(record),
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
    write_capture_bar_numbers(state, row, &record);
    write_capture_presentation_numbers(state, row, &record);
    write_capture_strings(state, row, &record);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    Ok(1)
}

fn write_status_bar_numbers(state: &mut LuaState, row: Val, record: &StatusBarRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("barMin", record.bar_min),
        ("barMax", record.bar_max),
        ("barValue", record.bar_value),
        ("colorTint", f64::from(record.color_tint)),
        ("tooltipLoc", f64::from(record.tooltip_loc)),
        ("fillMotionType", f64::from(record.fill_motion_type)),
        ("fillMinOpacity", f64::from(record.fill_min_opacity)),
        ("fillMaxOpacity", f64::from(record.fill_max_opacity)),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
}

fn write_status_text_numbers(state: &mut LuaState, row: Val, record: &StatusBarRecord) {
    for (key, value) in [
        ("barValueTextType", f64::from(record.bar_value_text_type)),
        (
            "overrideBarTextShownType",
            f64::from(record.override_bar_text_shown_type),
        ),
        (
            "barTextEnabledState",
            f64::from(record.bar_text_enabled_state),
        ),
        ("barTextFontType", f64::from(record.bar_text_font_type)),
        ("barTextSizeType", f64::from(record.bar_text_size_type)),
        ("textEnabledState", f64::from(record.text_enabled_state)),
        ("textFontType", f64::from(record.text_font_type)),
        ("textSizeType", f64::from(record.text_size_type)),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
}

fn write_status_presentation_numbers(state: &mut LuaState, row: Val, record: &StatusBarRecord) {
    for (key, value) in [
        ("glowAnimType", f64::from(record.glow_anim_type)),
        ("showGlowState", f64::from(record.show_glow_state)),
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
}

fn write_status_strings(state: &mut LuaState, row: Val, record: &StatusBarRecord) {
    for (key, text) in [
        ("text", &record.text),
        ("tooltip", &record.tooltip),
        ("overrideBarText", &record.override_bar_text),
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}

fn write_partition_values(state: &mut LuaState, row: Val, values: &[f64]) {
    let partitions = create_table(state);
    state.push(partitions);
    for (index, value) in values.iter().enumerate() {
        set_table_array(state, partitions, (index + 1) as i64, Val::Num(*value));
    }
    table_set(state, row, "partitionValues", partitions);
    state.top -= 1;
}

fn write_capture_bar_numbers(state: &mut LuaState, row: Val, record: &CaptureBarRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("barValue", record.bar_value),
        ("barMinValue", record.bar_min_value),
        ("barMaxValue", record.bar_max_value),
        ("neutralZoneSize", record.neutral_zone_size),
        ("neutralZoneCenter", record.neutral_zone_center),
        ("glowAnimType", f64::from(record.glow_anim_type)),
        ("fillDirectionType", f64::from(record.fill_direction_type)),
        ("tooltipLoc", f64::from(record.tooltip_loc)),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
}

fn write_capture_presentation_numbers(state: &mut LuaState, row: Val, record: &CaptureBarRecord) {
    for (key, value) in [
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
}

fn write_capture_strings(state: &mut LuaState, row: Val, record: &CaptureBarRecord) {
    for (key, text) in [
        ("tooltip", &record.tooltip),
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}
