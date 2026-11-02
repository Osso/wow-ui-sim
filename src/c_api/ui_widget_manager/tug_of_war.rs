//! Supplied tug-of-war records projected as fresh Lua snapshots.
use super::queries::read_numeric_id;
use super::{TugOfWarRecord, WidgetIconRecord, WidgetId, WidgetVisualization};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        return Ok(());
    }
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetTugOfWarWidgetVisualizationInfo",
        query,
    )
}
fn query(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetTugOfWarWidgetVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(WidgetId::from_number(id), &inputs.tug_of_war, |value| {
            match value {
                WidgetVisualization::TugOfWar(record) => Some(record),
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
fn write_record(state: &mut LuaState, row: Val, record: &TugOfWarRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("minValue", record.min_value),
        ("maxValue", record.max_value),
        ("currentValue", record.current_value),
        ("neutralZoneCenter", record.neutral_zone_center),
        ("neutralZoneSize", record.neutral_zone_size),
        ("glowAnimType", f64::from(record.glow_anim_type)),
        ("tooltipLoc", f64::from(record.tooltip_loc)),
        ("neutralFillStyle", f64::from(record.neutral_fill_style)),
        (
            "markerArrowShownState",
            f64::from(record.marker_arrow_shown_state),
        ),
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
        ("tooltip", &record.tooltip),
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    write_icon(state, row, "leftIconInfo", &record.left_icon_info);
    write_icon(state, row, "rightIconInfo", &record.right_icon_info);
}
fn write_icon(state: &mut LuaState, row: Val, key: &str, record: &WidgetIconRecord) {
    let icon = create_table(state);
    table_set(state, row, key, icon);
    for (key, value) in [
        ("sourceType", f64::from(record.source_type)),
        ("sourceID", record.source_id),
        ("sizeType", f64::from(record.size_type)),
        ("tooltipLoc", f64::from(record.tooltip_loc)),
    ] {
        table_set(state, icon, key, Val::Num(value));
    }
    let tooltip = create_string(state, &record.tooltip);
    table_set(state, icon, "tooltip", tooltip);
}
