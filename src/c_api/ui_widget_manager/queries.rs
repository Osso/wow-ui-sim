//! Supplied widget-set and IconAndText inputs projected as fresh Lua snapshots.

use super::{IconAndTextRecord, WidgetId, WidgetSetEntry};
use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Wrath {
        super::unit_context::register(state, namespace)?;
    }
    table_set_rust_fn_static(state, namespace, "GetWidgetSetInfo", query_set_info)?;
    table_set_rust_fn_static(state, namespace, "GetAllWidgetsBySetID", query_set_widgets)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetIconAndTextWidgetVisualizationInfo",
        query_icon_and_text,
    )?;
    super::bars::register(state)?;
    super::double_status_bar::register(state)?;
    super::double_icon_and_text::register(state)?;
    super::icon_text_and_background::register(state)?;
    super::icon_text_and_currencies::register(state)?;
    super::horizontal_currencies::register(state)?;
    super::item_display::register(state)?;
    super::texture_and_text::register(state)?;
    super::texture_and_text_row::register(state)?;
    super::text_column_row::register(state)?;
    super::spacer::register(state)?;
    super::discrete::register(state)?;
    super::text_state::register(state)?;
    super::text_with_subtext::register(state)?;
    super::bullet::register(state)?;
    super::spell_display::register(state)?;
    super::hud::register(state)
}

pub(super) fn read_numeric_id(state: &mut LuaState, error_message: &'static str) -> LuaResult<f64> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(number) = value else {
        return Err(runtime_error(error_message));
    };
    Ok(number)
}

fn query_set_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetWidgetSetInfo: widgetSetID must be a number",
    )?;
    let record = borrow_state(state)?
        .ui_widget_inputs
        .sets
        .get(&WidgetId::from_number(id))
        .cloned();
    let Some(record) = record else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let row = create_table(state);
    state.push(row);
    table_set(
        state,
        row,
        "layoutDirection",
        Val::Num(f64::from(record.layout_direction)),
    );
    table_set(
        state,
        row,
        "verticalPadding",
        Val::Num(record.vertical_padding),
    );
    Ok(1)
}

fn query_set_widgets(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetAllWidgetsBySetID: widgetSetID must be a number",
    )?;
    let records = borrow_state(state)?
        .ui_widget_inputs
        .sets
        .get(&WidgetId::from_number(id))
        .map(|record| record.widgets.clone());
    let rows = create_table(state);
    state.push(rows);
    if let Some(records) = records {
        for (index, record) in records.iter().enumerate() {
            let row = build_widget_info(state, id, record);
            set_table_array(state, rows, (index + 1) as i64, row);
        }
    }
    Ok(1)
}

/// Shared membership/event projection. Temporarily roots the row while populating it;
/// the caller must immediately attach or push the result before allocating again.
pub(super) fn build_widget_info(
    state: &mut LuaState,
    widget_set_id: f64,
    record: &WidgetSetEntry,
) -> Val {
    let row = create_table(state);
    state.push(row);
    for (key, value) in [
        ("widgetID", record.widget_id),
        ("widgetSetID", widget_set_id),
        ("widgetType", f64::from(record.widget_type)),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
    if let Some(token) = &record.unit_token {
        let value = create_string(state, token);
        table_set(state, row, "unitToken", value);
    }
    state.top -= 1;
    row
}

fn query_icon_and_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(WidgetId::from_number(id), &inputs.icon_and_text, |value| {
            match value {
                super::WidgetVisualization::IconAndText(record) => Some(record),
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
    write_icon_numbers(state, row, &record);
    write_icon_strings(state, row, &record);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    Ok(1)
}

fn write_icon_numbers(state: &mut LuaState, row: Val, record: &IconAndTextRecord) {
    for (key, value) in [
        ("state", f64::from(record.state)),
        ("tooltipLoc", f64::from(record.tooltip_loc)),
        ("shiftTextType", f64::from(record.shift_text_type)),
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

fn write_icon_strings(state: &mut LuaState, row: Val, record: &IconAndTextRecord) {
    for (key, text) in [
        ("text", &record.text),
        ("tooltip", &record.tooltip),
        ("dynamicTooltip", &record.dynamic_tooltip),
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}
