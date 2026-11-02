//! Supplied double-state icon rows projected as fresh Lua snapshots.
use super::queries::read_numeric_id;
use super::{DoubleStateIconRowRecord, WidgetId, WidgetStateIconRecord, WidgetVisualization};
use crate::c_api::helpers::set_table_array;
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
        "GetDoubleStateIconRowVisualizationInfo",
        query,
    )
}

fn query(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_numeric_id(
        state,
        "C_UIWidgetManager.GetDoubleStateIconRowVisualizationInfo: widgetID must be a number",
    )?;
    let record = {
        let sim = borrow_state(state)?;
        let inputs = &sim.ui_widget_inputs;
        inputs.resolve_visualization(
            WidgetId::from_number(id),
            &inputs.double_state_icons,
            |value| match value {
                WidgetVisualization::DoubleStateIconRow(record) => Some(record),
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

fn write_record(state: &mut LuaState, row: Val, record: &DoubleStateIconRowRecord) {
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
    for (key, text) in [
        ("textureKit", &record.texture_kit),
        ("frameTextureKit", &record.frame_texture_kit),
        ("widgetTag", &record.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
    table_set(state, row, "hasTimer", Val::Bool(record.has_timer));
    write_icons(state, row, "leftIcons", &record.left_icons);
    write_icons(state, row, "rightIcons", &record.right_icons);
}

fn write_icons(state: &mut LuaState, row: Val, key: &str, records: &[WidgetStateIconRecord]) {
    let icons = create_table(state);
    table_set(state, row, key, icons);
    for (index, record) in records.iter().enumerate() {
        let icon = create_table(state);
        set_table_array(state, icons, (index + 1) as i64, icon);
        table_set(
            state,
            icon,
            "iconState",
            Val::Num(f64::from(record.icon_state)),
        );
        for (key, text) in [
            ("state1Tooltip", &record.state1_tooltip),
            ("state2Tooltip", &record.state2_tooltip),
        ] {
            let value = create_string(state, text);
            table_set(state, icon, key, value);
        }
    }
}
