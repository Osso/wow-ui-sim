//! Complete supplied ItemDisplay records projected as fresh nested Lua snapshots.

use super::queries::read_numeric_id;
use super::{ItemDisplayRecord, UiWidgetItemRecord, WidgetId};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetItemDisplayVisualizationInfo",
        query_item_display,
    )
}

fn query_item_display(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetItemDisplayVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(WidgetId::from_number(id), &inputs.item_display, |value| {
            match value {
                super::WidgetVisualization::ItemDisplay(record) => Some(record),
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
    write_display_numbers(state, row, &record);
    write_display_strings(state, row, &record);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    write_item_info(state, row, &record.item_info);
    Ok(1)
}

fn write_display_numbers(state: &mut LuaState, row: Val, record: &ItemDisplayRecord) {
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

fn write_display_strings(state: &mut LuaState, row: Val, record: &ItemDisplayRecord) {
    for (key, text) in [
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}

fn write_item_info(state: &mut LuaState, row: Val, record: &UiWidgetItemRecord) {
    let item = create_table(state);
    table_set(state, row, "itemInfo", item);
    write_item_numbers(state, item, record);
    write_item_strings(state, item, record);
    table_set(
        state,
        item,
        "tooltipEnabled",
        Val::Bool(record.tooltip_enabled),
    );
    table_set(
        state,
        item,
        "showAsEarned",
        Val::Bool(record.show_as_earned),
    );
    if let Some(count) = record.stack_count {
        table_set(state, item, "stackCount", Val::Num(count));
    }
}

fn write_item_numbers(state: &mut LuaState, row: Val, record: &UiWidgetItemRecord) {
    for (key, value) in [
        ("itemID", record.item_id),
        ("textDisplayStyle", f64::from(record.text_display_style)),
        ("iconSizeType", f64::from(record.icon_size_type)),
        (
            "infoTextEnabledState",
            f64::from(record.info_text_enabled_state),
        ),
        (
            "itemNameTextFontType",
            f64::from(record.item_name_text_font_type),
        ),
        (
            "itemNameTextSizeType",
            f64::from(record.item_name_text_size_type),
        ),
        ("infoTextFontType", f64::from(record.info_text_font_type)),
        ("infoTextSizeType", f64::from(record.info_text_size_type)),
        (
            "itemNameCustomColor",
            f64::from(record.item_name_custom_color),
        ),
        (
            "itemNameCustomColorOverrideState",
            f64::from(record.item_name_custom_color_override_state),
        ),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
}

fn write_item_strings(state: &mut LuaState, row: Val, record: &UiWidgetItemRecord) {
    for (key, text) in [
        ("overrideItemName", &record.override_item_name),
        ("infoText", &record.info_text),
        ("overrideTooltip", &record.override_tooltip),
    ] {
        if let Some(text) = text {
            let value = create_string(state, text);
            table_set(state, row, key, value);
        }
    }
}
