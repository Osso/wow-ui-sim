//! Supplied TextColumnRow records projected as fresh, deeply independent snapshots.

use super::queries::read_numeric_id;
use super::{TextColumnRowEntryInfo, TextColumnRowVisualizationInfo, WidgetId};
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
        "GetTextColumnRowVisualizationInfo",
        query_text_column_row,
    )
}

fn query_text_column_row(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetTextColumnRowVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(
            WidgetId::from_number(id),
            &inputs.text_column_rows,
            |value| match value {
                super::WidgetVisualization::TextColumnRow(record) => Some(record),
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

fn write_row_numbers(state: &mut LuaState, row: Val, record: &TextColumnRowVisualizationInfo) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("textSizeType", f64::from(record.text_size_type)),
        ("fontType", f64::from(record.font_type)),
        ("tooltipLoc", f64::from(record.tooltip_loc)),
        ("bottomPadding", record.bottom_padding),
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

fn write_row_strings(state: &mut LuaState, row: Val, record: &TextColumnRowVisualizationInfo) {
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

fn write_entries(state: &mut LuaState, row: Val, records: &[TextColumnRowEntryInfo]) {
    let entries = create_table(state);
    table_set(state, row, "entries", entries);
    for (index, record) in records.iter().enumerate() {
        let entry = create_table(state);
        set_table_array(state, entries, (index + 1) as i64, entry);
        let text = create_string(state, &record.text);
        table_set(state, entry, "text", text);
        for (key, value) in [
            ("enabledState", f64::from(record.enabled_state)),
            ("hAlign", f64::from(record.h_align)),
            ("columnWidth", record.column_width),
        ] {
            table_set(state, entry, key, Val::Num(value));
        }
    }
}
