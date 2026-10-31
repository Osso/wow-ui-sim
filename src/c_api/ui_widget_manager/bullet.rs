//! Supplied BulletTextList snapshots with fresh, ordered line arrays.

use super::queries::read_numeric_id;
use super::{BulletTextListRecord, WidgetId};
use crate::c_api::helpers::set_table_array;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return Ok(());
    }
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetBulletTextListWidgetVisualizationInfo",
        query_bullet_text_list,
    )
}

fn query_bullet_text_list(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetBulletTextListWidgetVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(
            WidgetId::from_number(id),
            &inputs.bullet_text_lists,
            |value| match value {
                super::WidgetVisualization::BulletTextList(record) => Some(record),
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
    write_numbers(state, row, &record);
    write_strings(state, row, &record);
    write_lines(state, row, &record.lines);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    Ok(1)
}

fn write_numbers(state: &mut LuaState, row: Val, record: &BulletTextListRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("enabledState", f64::from(record.enabled_state)),
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

fn write_strings(state: &mut LuaState, row: Val, record: &BulletTextListRecord) {
    for (key, text) in [
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}

fn write_lines(state: &mut LuaState, row: Val, texts: &[String]) {
    let lines = create_table(state);
    state.push(lines);
    for (index, text) in texts.iter().enumerate() {
        let value = create_string(state, text);
        set_table_array(state, lines, (index + 1) as i64, value);
    }
    table_set(state, row, "lines", lines);
    state.top -= 1;
}
