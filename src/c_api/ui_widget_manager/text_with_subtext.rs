//! Complete supplied TextWithSubtext records projected as fresh Lua snapshots.

use super::queries::read_numeric_id;
use super::{TextWithSubtextRecord, WidgetId};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetTextWithSubtextWidgetVisualizationInfo",
        query_text_with_subtext,
    )
}

fn query_text_with_subtext(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetTextWithSubtextWidgetVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(
            WidgetId::from_number(id),
            &inputs.text_with_subtext,
            |value| match value {
                super::WidgetVisualization::TextWithSubtext(record) => Some(record),
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
    write_text_numbers(state, row, &record);
    write_presentation_numbers(state, row, &record);
    write_strings(state, row, &record);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    Ok(1)
}

fn write_text_numbers(state: &mut LuaState, row: Val, record: &TextWithSubtextRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("enabledState", f64::from(record.enabled_state)),
        ("widgetWidth", record.widget_width),
        ("textSizeType", f64::from(record.text_size_type)),
        ("fontType", f64::from(record.font_type)),
        ("tooltipLoc", f64::from(record.tooltip_loc)),
        ("hAlign", f64::from(record.h_align)),
        ("subTextSizeType", f64::from(record.sub_text_size_type)),
        ("subTextFontType", f64::from(record.sub_text_font_type)),
        ("subTextHAlign", f64::from(record.sub_text_h_align)),
        (
            "subTextEnabledState",
            f64::from(record.sub_text_enabled_state),
        ),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
}

fn write_presentation_numbers(state: &mut LuaState, row: Val, record: &TextWithSubtextRecord) {
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
        ("spacing", record.spacing),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
}

fn write_strings(state: &mut LuaState, row: Val, record: &TextWithSubtextRecord) {
    for (key, text) in [
        ("text", &record.text),
        ("tooltip", &record.tooltip),
        ("subText", &record.sub_text),
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}
