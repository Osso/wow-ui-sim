//! Simple getters/setters: clear, num_messages, max_lines, fading, time_visible,
//! fade_duration, fade_power, insert_mode, text_copyable, has_message_by_id,
//! get_message_info, indented_word_wrap.

use super::super::shared::{opt_string, val_to_bool, val_to_f64};
use super::scroll::message_frame_scroll_limit;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, frame_id_from_stack, get_or_create_frame_fields,
    table_get, table_set,
};
use crate::lua_bridge::{IntoStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const INDENTED_WORD_WRAP_FIELD: &str = "_mf_indented_word_wrap";

fn indented_word_wrap_field_name(state: &LuaState) -> String {
    match stack_val(state, 2) {
        Val::Bool(_) | Val::Nil => INDENTED_WORD_WRAP_FIELD.to_string(),
        value => {
            let suffix = match value {
                Val::Str(raw) => state
                    .gc
                    .string_arena
                    .get(raw)
                    .map(|text| String::from_utf8_lossy(text.data()).to_string())
                    .unwrap_or_default(),
                _ => String::new(),
            };
            format!("{INDENTED_WORD_WRAP_FIELD}_{suffix}")
        }
    }
}

pub(in super::super) fn clear(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(data) = sim.message_frames.get_mut(&id) {
        data.messages.clear();
        data.scroll_offset = 0;
    }
    Ok(0)
}

pub(super) fn clear_text(state: &mut LuaState) -> LuaResult<u32> {
    clear(state)
}

pub(super) fn get_num_messages(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let count = sim
        .message_frames
        .get(&id)
        .map(|d| d.messages.len())
        .unwrap_or(0) as f64;
    drop(sim);
    count.into_stack(state)
}

pub(super) fn set_max_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let max = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    let data = sim.message_frames.entry(id).or_default();
    data.max_lines = max.max(1) as usize;
    while data.messages.len() > data.max_lines {
        super::scroll::truncate_messages(data);
    }
    data.scroll_offset = data
        .scroll_offset
        .clamp(0, message_frame_scroll_limit(data));
    data.display_dirty = true;
    Ok(0)
}

pub(super) fn get_max_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .message_frames
        .get(&id)
        .map(|d| d.max_lines)
        .unwrap_or(120) as f64;
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_fading(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    sim.message_frames.entry(id).or_default().fading = value;
    Ok(0)
}

pub(super) fn get_fading(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .message_frames
        .get(&id)
        .map(|d| d.fading)
        .unwrap_or(true);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_time_visible(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let v = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    sim.message_frames.entry(id).or_default().time_visible = v;
    Ok(0)
}

pub(super) fn get_time_visible(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .message_frames
        .get(&id)
        .map(|d| d.time_visible)
        .unwrap_or(10.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_fade_duration(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let v = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    sim.message_frames.entry(id).or_default().fade_duration = v;
    Ok(0)
}

pub(super) fn get_fade_duration(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .message_frames
        .get(&id)
        .map(|d| d.fade_duration)
        .unwrap_or(3.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_fade_power(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let v = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    sim.message_frames.entry(id).or_default().fade_power = v;
    Ok(0)
}

pub(super) fn get_fade_power(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .message_frames
        .get(&id)
        .map(|d| d.fade_power)
        .unwrap_or(1.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_insert_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mode = opt_string(state, 2).unwrap_or_else(|| "BOTTOM".to_string());
    let mut sim = borrow_state_mut(state)?;
    sim.message_frames.entry(id).or_default().insert_mode = mode;
    Ok(0)
}

pub(super) fn get_insert_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mode = {
        let sim = borrow_state(state)?;
        sim.message_frames
            .get(&id)
            .map(|d| d.insert_mode.clone())
            .unwrap_or_else(|| "BOTTOM".to_string())
    };
    let v = create_string(state, &mode);
    v.into_stack(state)
}

pub(super) fn set_text_copyable(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let v = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    sim.message_frames.entry(id).or_default().text_copyable = v;
    Ok(0)
}

pub(super) fn is_text_copyable(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .message_frames
        .get(&id)
        .map(|d| d.text_copyable)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn has_message_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let msg_id = val_to_f64(stack_val(state, 2)) as i64;
    let sim = borrow_state(state)?;
    let found = sim
        .message_frames
        .get(&id)
        .map(|d| d.messages.iter().any(|m| m.message_id == Some(msg_id)))
        .unwrap_or(false);
    drop(sim);
    found.into_stack(state)
}

pub(super) fn get_message_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let index = val_to_f64(stack_val(state, 2)) as i32;
    let sim = borrow_state(state)?;
    let (text, r, g, b, a, ts) = sim
        .message_frames
        .get(&id)
        .and_then(|d| d.messages.get((index - 1) as usize))
        .map(|m| {
            (
                m.text.clone(),
                m.r as f64,
                m.g as f64,
                m.b as f64,
                m.a as f64,
                m.timestamp,
            )
        })
        .unwrap_or((String::new(), 1.0, 1.0, 1.0, 1.0, 0.0));
    drop(sim);
    let text_val = create_string(state, &text);
    text_val.into_stack(state)?;
    state.push(Val::Num(r));
    state.push(Val::Num(g));
    state.push(Val::Num(b));
    state.push(Val::Num(a));
    state.push(Val::Num(ts));
    Ok(6)
}

pub(super) fn set_indented_word_wrap(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value_index = match stack_val(state, 2) {
        Val::Bool(_) => 2,
        _ => 3,
    };
    let v = val_to_bool(stack_val(state, value_index));
    let fields = get_or_create_frame_fields(state, id);
    let field_name = indented_word_wrap_field_name(state);
    table_set(state, fields, &field_name, Val::Bool(v));
    Ok(0)
}

pub(super) fn get_indented_word_wrap(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let fields = get_or_create_frame_fields(state, id);
    let field_name = indented_word_wrap_field_name(state);
    let v = match table_get(state, fields, &field_name) {
        Val::Bool(b) => b,
        _ => false,
    };
    v.into_stack(state)
}
