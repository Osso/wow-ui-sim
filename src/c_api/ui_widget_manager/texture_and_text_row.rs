//! Complete supplied row records projected as fresh, deeply independent Lua tables.

use super::queries::read_numeric_id;
use super::{TextureAndTextEntryRecord, TextureAndTextRowRecord, WidgetId};
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
        "GetTextureAndTextRowVisualizationInfo",
        query_texture_and_text_row,
    )
}

fn query_texture_and_text_row(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetTextureAndTextRowVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(
            WidgetId::from_number(id),
            &inputs.texture_and_text_row,
            |value| match value {
                super::WidgetVisualization::TextureAndTextRow(record) => Some(record),
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
    write_row_numbers(state, row, &record);
    write_row_strings(state, row, &record);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    write_entries(state, row, &record.entries);
    Ok(1)
}

fn write_row_numbers(state: &mut LuaState, row: Val, record: &TextureAndTextRowRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("textSizeType", f64::from(record.text_size_type)),
        ("groupAlignment", f64::from(record.group_alignment)),
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
    if let Some(width) = record.fixed_width {
        table_set(state, row, "fixedWidth", Val::Num(width));
    }
}

fn write_row_strings(state: &mut LuaState, row: Val, record: &TextureAndTextRowRecord) {
    for (key, text) in [
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}

fn write_entries(state: &mut LuaState, row: Val, records: &[TextureAndTextEntryRecord]) {
    let entries = create_table(state);
    table_set(state, row, "entries", entries);
    for (index, record) in records.iter().enumerate() {
        let entry = create_table(state);
        set_table_array(state, entries, (index + 1) as i64, entry);
        write_entry_strings(state, entry, record);
    }
}

fn write_entry_strings(state: &mut LuaState, entry: Val, record: &TextureAndTextEntryRecord) {
    for (key, text) in [("text", &record.text), ("tooltip", &record.tooltip)] {
        let value = create_string(state, text);
        table_set(state, entry, key, value);
    }
}
