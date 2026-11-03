//! Identity and map-ID methods: SetID, GetID, GetMapID, GetUiMapID, SetMapID,
//! GetName, GetDebugName, GetObjectType, IsObjectType.

use super::helpers::frame_id;
use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const INTERNAL_GENERATED_NAME_PREFIXES: &[&str] = &[
    "__tpl_",
    "__tex_",
    "__font_",
    "__line_",
    "__anim_",
    "__frame_",
    "__Blizzard_",
];

pub fn set_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let user_id = i32::from_stack(state, 2)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetID");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut(id) {
        f.user_id = user_id;
    }
    Ok(0)
}

pub fn get_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim.widgets.get(id).map(|f| f.user_id).unwrap_or(0);
    drop(sim);
    state.push(Val::Num(result as f64));
    Ok(1)
}

pub fn get_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim
        .quest_blobs
        .get(&id)
        .map(|b| b.map_id as i32)
        .unwrap_or(0);
    drop(sim);
    state.push(Val::Num(result as f64));
    Ok(1)
}

pub fn get_ui_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim
        .fog_of_war_frames
        .get(&id)
        .and_then(|fog| fog.ui_map_id)
        .or_else(|| {
            sim.unit_position_frames
                .get(&id)
                .and_then(|unit_state| unit_state.ui_map_id)
        })
        .unwrap_or_else(|| {
            sim.quest_blobs
                .get(&id)
                .map(|b| b.map_id as i32)
                .unwrap_or(0)
        });
    drop(sim);
    state.push(Val::Num(result as f64));
    Ok(1)
}

pub fn set_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let map_id = i32::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    if frame_object_type_matches(&sim, id, "FogOfWarFrame") {
        set_fog_of_war_map_id(&mut sim, id, map_id);
    } else if frame_object_type_matches(&sim, id, "UnitPositionFrame") {
        set_unit_position_map_id(&mut sim, id, map_id);
    } else {
        sim.quest_blobs.entry(id).or_default().map_id = map_id as u32;
    }
    Ok(0)
}

fn frame_object_type_matches(
    sim: &crate::lua_api::state::SimState,
    id: u64,
    expected: &str,
) -> bool {
    sim.widgets
        .get(id)
        .and_then(|frame| frame.object_type_name.as_deref())
        .is_some_and(|name| name.eq_ignore_ascii_case(expected))
}

fn set_fog_of_war_map_id(sim: &mut crate::lua_api::state::SimState, id: u64, map_id: i32) {
    sim.fog_of_war_frames.entry(id).or_default().ui_map_id = Some(map_id);
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.fog_of_war_ui_map_id = Some(map_id);
    }
}

fn set_unit_position_map_id(sim: &mut crate::lua_api::state::SimState, id: u64, map_id: i32) {
    sim.unit_position_frames
        .entry(id)
        .or_insert_with(|| crate::lua_api::state::UnitPositionFrameState {
            ui_map_id: None,
            units: Vec::new(),
            unit_colors: std::collections::HashMap::new(),
            mouse_over_units: Vec::new(),
            player_ping_scale: 1.0,
            player_ping_textures: std::collections::HashMap::new(),
            player_ping_active: false,
            player_ping_duration: None,
            player_ping_fade_duration: None,
            is_finalized: false,
        })
        .ui_map_id = Some(map_id);
}

pub fn get_name(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let name = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .and_then(|frame| frame.name.as_deref())
            .filter(|name| !is_internal_generated_name(name))
            .map(str::to_owned)
    };
    let name_val = match name {
        Some(name) => create_string(state, &name),
        None => Val::Nil,
    };
    state.push(name_val);
    Ok(1)
}

pub fn get_debug_name(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let debug_name = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|frame| {
                frame
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("{}:{id}", frame.widget_type.as_str()))
            })
            .unwrap_or_else(|| format!("Frame:{id}"))
    };
    let debug_name_val = create_string(state, &debug_name);
    state.push(debug_name_val);
    Ok(1)
}

fn is_internal_generated_name(name: &str) -> bool {
    INTERNAL_GENERATED_NAME_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

pub fn get_object_type(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let object_type = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|frame| {
                if matches!(frame.widget_type, crate::widget::WidgetType::WorldFrame) {
                    return "Frame".to_string();
                }
                frame
                    .object_type_name
                    .clone()
                    .unwrap_or_else(|| frame.widget_type.as_str().to_string())
            })
            .unwrap_or_else(|| "Frame".to_string())
    };
    let object_type_val = create_string(state, &object_type);
    state.push(object_type_val);
    Ok(1)
}

pub fn is_object_type(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let requested = String::from_stack(state, 2)?;
    let result = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|frame| {
                if matches!(frame.widget_type, crate::widget::WidgetType::WorldFrame) {
                    return requested.eq_ignore_ascii_case("WorldFrame")
                        || requested.eq_ignore_ascii_case("Region");
                }
                if requested.eq_ignore_ascii_case("Region") {
                    return true;
                }
                let actual = frame
                    .object_type_name
                    .as_deref()
                    .unwrap_or(frame.widget_type.as_str());
                actual.eq_ignore_ascii_case(&requested)
                    || frame.widget_type.as_str().eq_ignore_ascii_case(&requested)
                    || widget_type_inherits(frame.widget_type, &requested)
                    || (requested.eq_ignore_ascii_case("Animation")
                        && sim.anim_frame_to_anim.contains_key(&id))
            })
            .unwrap_or(false)
    };
    state.push(Val::Bool(result));
    Ok(1)
}

fn widget_type_inherits(widget_type: crate::widget::WidgetType, requested: &str) -> bool {
    match widget_type {
        crate::widget::WidgetType::Button => requested.eq_ignore_ascii_case("Frame"),
        crate::widget::WidgetType::CheckButton => {
            requested.eq_ignore_ascii_case("Button") || requested.eq_ignore_ascii_case("Frame")
        }
        crate::widget::WidgetType::Cooldown
        | crate::widget::WidgetType::EditBox
        | crate::widget::WidgetType::GameTooltip
        | crate::widget::WidgetType::MessageFrame
        | crate::widget::WidgetType::Minimap
        | crate::widget::WidgetType::Model
        | crate::widget::WidgetType::ModelScene
        | crate::widget::WidgetType::ScrollFrame
        | crate::widget::WidgetType::SimpleHTML
        | crate::widget::WidgetType::Slider
        | crate::widget::WidgetType::StatusBar
        | crate::widget::WidgetType::ColorSelect => requested.eq_ignore_ascii_case("Frame"),
        crate::widget::WidgetType::PlayerModel => {
            requested.eq_ignore_ascii_case("Model") || requested.eq_ignore_ascii_case("Frame")
        }
        crate::widget::WidgetType::FontString
        | crate::widget::WidgetType::Line
        | crate::widget::WidgetType::Texture => requested.eq_ignore_ascii_case("Region"),
        crate::widget::WidgetType::Frame | crate::widget::WidgetType::WorldFrame => false,
    }
}
