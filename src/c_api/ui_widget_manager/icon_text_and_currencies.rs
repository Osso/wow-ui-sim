//! Complete supplied IconTextAndCurrencies records projected as fresh nested Lua snapshots.

use super::queries::read_numeric_id;
use super::{IconTextAndCurrenciesRecord, UiWidgetCurrencyRecord, WidgetId};
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
        "GetIconTextAndCurrenciesWidgetVisualizationInfo",
        query_icon_text_and_currencies,
    )
}

fn query_icon_text_and_currencies(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetIconTextAndCurrenciesWidgetVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(
            WidgetId::from_number(id),
            &inputs.icon_text_and_currencies,
            |value| match value {
                super::WidgetVisualization::IconTextAndCurrencies(record) => Some(record),
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
    write_display_numbers(state, row, &record);
    write_display_strings(state, row, &record);
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    write_currencies(state, row, &record.currencies);
    Ok(1)
}

fn write_display_numbers(state: &mut LuaState, row: Val, record: &IconTextAndCurrenciesRecord) {
    for (key, value) in [
        ("shownState", f64::from(record.shown_state)),
        ("enabledState", f64::from(record.enabled_state)),
        (
            "descriptionShownState",
            f64::from(record.description_shown_state),
        ),
        (
            "descriptionEnabledState",
            f64::from(record.description_enabled_state),
        ),
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

fn write_display_strings(state: &mut LuaState, row: Val, record: &IconTextAndCurrenciesRecord) {
    for (key, text) in [
        ("text", &record.text),
        ("description", &record.description),
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}

fn write_currencies(state: &mut LuaState, row: Val, records: &[UiWidgetCurrencyRecord]) {
    let currencies = create_table(state);
    table_set(state, row, "currencies", currencies);
    for (index, record) in records.iter().enumerate() {
        let currency = create_table(state);
        set_table_array(state, currencies, (index + 1) as i64, currency);
        write_currency_numbers(state, currency, record);
        write_currency_strings(state, currency, record);
        table_set(
            state,
            currency,
            "isCurrencyMaxed",
            Val::Bool(record.is_currency_maxed),
        );
    }
}

fn write_currency_numbers(state: &mut LuaState, row: Val, record: &UiWidgetCurrencyRecord) {
    for (key, value) in [
        ("iconFileID", f64::from(record.icon_file_id)),
        ("textFontType", f64::from(record.text_font_type)),
        ("textSizeType", f64::from(record.text_size_type)),
        ("textEnabledState", f64::from(record.text_enabled_state)),
        ("iconSizeType", f64::from(record.icon_size_type)),
        ("updateAnimType", f64::from(record.update_anim_type)),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
}

fn write_currency_strings(state: &mut LuaState, row: Val, record: &UiWidgetCurrencyRecord) {
    for (key, text) in [
        ("leadingText", &record.leading_text),
        ("text", &record.text),
        ("tooltip", &record.tooltip),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}
