//! Complete supplied DoubleStatusBar records projected as fresh Lua snapshots.

use super::queries::read_numeric_id;
use super::{DoubleStatusBarRecord, WidgetId};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetDoubleStatusBarWidgetVisualizationInfo",
        query_double_status_bar,
    )
}

fn query_double_status_bar(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetDoubleStatusBarWidgetVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(
            WidgetId::from_number(id),
            &inputs.double_status_bars,
            |value| match value {
                super::WidgetVisualization::DoubleStatusBar(record) => Some(record),
                _ => None,
            },
        )
    };
    let Some(record) = record else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let row = create_table(state);
    state.push(row);
    write_bar_numbers(state, row, &record);
    write_presentation_numbers(state, row, &record);
    write_strings(state, row, &record);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    Ok(1)
}

fn write_bar_numbers(state: &mut LuaState, row: Val, record: &DoubleStatusBarRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("leftBarMin", record.left_bar_min),
        ("leftBarMax", record.left_bar_max),
        ("leftBarValue", record.left_bar_value),
        ("rightBarMin", record.right_bar_min),
        ("rightBarMax", record.right_bar_max),
        ("rightBarValue", record.right_bar_value),
        ("barValueTextType", f64::from(record.bar_value_text_type)),
        ("leftBarTooltipLoc", f64::from(record.left_bar_tooltip_loc)),
        (
            "rightBarTooltipLoc",
            f64::from(record.right_bar_tooltip_loc),
        ),
        ("fillMotionType", f64::from(record.fill_motion_type)),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
}

fn write_presentation_numbers(state: &mut LuaState, row: Val, record: &DoubleStatusBarRecord) {
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

fn write_strings(state: &mut LuaState, row: Val, record: &DoubleStatusBarRecord) {
    for (key, text) in [
        ("leftBarTooltip", &record.left_bar_tooltip),
        ("rightBarTooltip", &record.right_bar_tooltip),
        ("text", &record.text),
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}
