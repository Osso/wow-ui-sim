//! Complete supplied SpellDisplay records projected as fresh nested Lua snapshots.

use super::queries::read_numeric_id;
use super::{SpellDisplayRecord, WidgetId, WidgetSpellInfoRecord};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let publish = matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr);
    if !publish {
        return Ok(());
    }
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetSpellDisplayVisualizationInfo",
        query_spell_display,
    )
}

fn query_spell_display(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetSpellDisplayVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(WidgetId::from_number(id), &inputs.spell_display, |value| {
            match value {
                super::WidgetVisualization::SpellDisplay(record) => Some(record),
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
    write_spell_info(state, row, &record.spell_info);
    Ok(1)
}

fn write_display_numbers(state: &mut LuaState, row: Val, record: &SpellDisplayRecord) {
    for (key, value) in [
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

fn write_display_strings(state: &mut LuaState, row: Val, record: &SpellDisplayRecord) {
    for (key, text) in [
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}

fn write_spell_info(state: &mut LuaState, row: Val, record: &WidgetSpellInfoRecord) {
    let spell_info = create_table(state);
    table_set(state, row, "spellInfo", spell_info);
    write_spell_numbers(state, spell_info, record);
    write_spell_strings(state, spell_info, record);
    table_set(
        state,
        spell_info,
        "showAsEarned",
        Val::Bool(record.show_as_earned),
    );
}

fn write_spell_numbers(state: &mut LuaState, row: Val, record: &WidgetSpellInfoRecord) {
    for (key, value) in [
        ("spellID", record.spell_id),
        ("shownState", f64::from(record.shown_state)),
        ("enabledState", f64::from(record.enabled_state)),
        ("stackDisplay", record.stack_display),
        ("iconSizeType", f64::from(record.icon_size_type)),
        ("iconDisplayType", f64::from(record.icon_display_type)),
        ("textShownState", f64::from(record.text_shown_state)),
        ("borderColor", f64::from(record.border_color)),
        ("textFontType", f64::from(record.text_font_type)),
        ("textSizeType", f64::from(record.text_size_type)),
        ("hAlignType", f64::from(record.h_align_type)),
        ("tint", f64::from(record.tint)),
        ("showGlowState", f64::from(record.show_glow_state)),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
}

fn write_spell_strings(state: &mut LuaState, row: Val, record: &WidgetSpellInfoRecord) {
    for (key, text) in [("tooltip", &record.tooltip), ("text", &record.text)] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
}
