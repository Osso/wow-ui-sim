//! Host prey selection/widget snapshots. No progress or quest choice is synthesized.
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PreyHuntProgressWidget {
    pub shown_state: u8,
    pub progress_state: u8,
    pub tooltip: String,
    pub tooltip_loc: u8,
    pub widget_size_setting: f64,
    pub texture_kit: String,
    pub frame_texture_kit: String,
    pub has_timer: bool,
    pub order_index: f64,
    pub widget_tag: String,
    pub in_anim_type: u8,
    pub out_anim_type: u8,
    pub widget_scale: u8,
    pub layout_direction: u8,
    pub model_scene_layer: u8,
    pub scripted_animation_effect_id: u32,
}
#[derive(Debug, Default)]
pub struct PreyInputs {
    pub active_quest: Option<u32>,
    pub widgets: HashMap<u32, PreyHuntProgressWidget>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let quests = super::ensure_namespace(state, "C_QuestLog")?;
    table_set_rust_fn_static(state, quests, "GetActivePreyQuest", active_quest)?;
    let widgets = super::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        widgets,
        "GetPreyHuntProgressWidgetVisualizationInfo",
        widget,
    )
}
fn active_quest(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED: a host selection requires an existing quest-log entry; abandonment
    // cannot expose an obsolete prey ID. No inference from quest title/tag.
    let id = {
        let sim = borrow_state(state)?;
        sim.prey_inputs.active_quest.filter(|id| {
            sim.quest_log_entries
                .entries
                .iter()
                .any(|q| q.quest_id as u32 == *id)
        })
    };
    let Some(id) = id else {
        return Ok(0);
    };
    state.push(Val::Num(f64::from(id)));
    Ok(1)
}
fn widget(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(id) = value else {
        return Err(rilua::runtime_error("widgetID must be a number"));
    };
    if id <= 0.0 || id > u32::MAX as f64 || id.fract() != 0.0 {
        return Err(rilua::runtime_error("widgetID must be a positive integer"));
    }
    let record = borrow_state(state)?
        .prey_inputs
        .widgets
        .get(&(id as u32))
        .cloned();
    let Some(r) = record else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let result = create_table(state);
    state.push(result);
    for (key, text) in [
        ("tooltip", &r.tooltip),
        ("textureKit", &r.texture_kit),
        ("frameTextureKit", &r.frame_texture_kit),
        ("widgetTag", &r.widget_tag),
    ] {
        let value = create_string(state, text);
        table_set(state, result, key, value);
    }
    table_set(state, result, "hasTimer", Val::Bool(r.has_timer));
    for (key, number) in [
        ("shownState", f64::from(r.shown_state)),
        ("progressState", f64::from(r.progress_state)),
        ("tooltipLoc", f64::from(r.tooltip_loc)),
        ("widgetSizeSetting", r.widget_size_setting),
        ("orderIndex", r.order_index),
        ("inAnimType", f64::from(r.in_anim_type)),
        ("outAnimType", f64::from(r.out_anim_type)),
        ("widgetScale", f64::from(r.widget_scale)),
        ("layoutDirection", f64::from(r.layout_direction)),
        ("modelSceneLayer", f64::from(r.model_scene_layer)),
        (
            "scriptedAnimationEffectID",
            f64::from(r.scripted_animation_effect_id),
        ),
    ] {
        table_set(state, result, key, Val::Num(number));
    }
    Ok(1)
}
