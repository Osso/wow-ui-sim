//! Supplied unit-power bar records projected as fresh Lua snapshots.
use super::queries::read_numeric_id;
use super::{UnitPowerBarRecord, WidgetId, WidgetVisualization};
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
        "GetUnitPowerBarWidgetVisualizationInfo",
        query,
    )
}
fn query(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetUnitPowerBarWidgetVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(
            WidgetId::from_number(id),
            &inputs.unit_power_bars,
            |value| match value {
                WidgetVisualization::UnitPowerBar(record) => Some(record),
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
    write_record(state, row, &record);
    Ok(1)
}
fn write_record(state: &mut LuaState, row: Val, record: &UnitPowerBarRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("barMin", record.bar_min),
        ("barMax", record.bar_max),
        ("barValue", record.bar_value),
        ("barValueTextType", f64::from(record.bar_value_text_type)),
        (
            "overrideBarTextShownType",
            f64::from(record.override_bar_text_shown_type),
        ),
        ("tooltipLoc", f64::from(record.tooltip_loc)),
        ("fillMotionType", f64::from(record.fill_motion_type)),
        (
            "flashBlendModeType",
            f64::from(record.flash_blend_mode_type),
        ),
        (
            "sparkBlendModeType",
            f64::from(record.spark_blend_mode_type),
        ),
        ("flashMomentType", f64::from(record.flash_moment_type)),
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
        ("overrideBarText", &record.override_bar_text),
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
}
