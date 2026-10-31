//! Complete supplied ButtonHeader records projected as fresh Lua snapshots.

use super::{ButtonHeaderRecord, WidgetId, WidgetSpellButtonRecord};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetButtonHeaderWidgetVisualizationInfo",
        query_header,
    )
}

fn query_header(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(id) = value else {
        return Err(runtime_error(
            "C_UIWidgetManager.GetButtonHeaderWidgetVisualizationInfo: widgetID must be a number",
        ));
    };
    let record = {
        let sim = borrow_state(state)?;
        sim.ui_widget_inputs.resolve_visualization(
            WidgetId::from_number(id),
            &sim.button_header_inputs.widgets,
            |value| match value {
                super::WidgetVisualization::ButtonHeader(record) => Some(record),
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
    write_header_numbers(state, row, &record);
    write_header_strings(state, row, &record);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    write_buttons(state, row, &record.buttons);
    Ok(1)
}

fn write_header_numbers(state: &mut LuaState, row: Val, record: &ButtonHeaderRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
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
}

fn write_header_strings(state: &mut LuaState, row: Val, record: &ButtonHeaderRecord) {
    for (key, text) in [
        ("headerText", &record.header_text),
        ("tooltip", &record.tooltip),
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}

fn write_buttons(state: &mut LuaState, row: Val, records: &[WidgetSpellButtonRecord]) {
    let buttons = create_table(state);
    table_set(state, row, "buttons", buttons);
    for (index, record) in records.iter().enumerate() {
        let button = create_table(state);
        crate::c_api::helpers::set_table_array(state, buttons, (index + 1) as i64, button);
        write_button_fields(state, button, record);
    }
}

fn write_button_fields(state: &mut LuaState, button: Val, record: &WidgetSpellButtonRecord) {
    for (key, value) in [
        ("spellID", record.spell_id),
        ("icon", f64::from(record.icon)),
        ("enabledState", f64::from(record.enabled_state)),
        ("cooldownType", f64::from(record.cooldown_type)),
    ] {
        table_set(state, button, key, Val::Num(value));
    }
    let tooltip = create_string(state, &record.tooltip);
    table_set(state, button, "tooltip", tooltip);
}
