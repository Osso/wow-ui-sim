//! EditBox and text-spacing widget methods.

mod registration;
mod selection;

use super::shared::{opt_string, val_to_bool, val_to_f64};
use crate::lua_api::frame::methods::forbidden_aspects::ensure_forbidden_aspect_absent;
use crate::lua_api::frame::methods::text_attribute_event::refresh_auto_text_height_after_width_change;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, frame_id_from_stack, get_or_create_frame_fields,
    table_get, table_set,
};
use crate::lua_bridge::{IntoStack, stack_val};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(super) fn set_focus(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "ScriptedInput", "SetFocus")?;
    let old_focus = {
        let mut sim = borrow_state_mut(state)?;
        let old = sim.focused_frame_id;
        sim.focused_frame_id = Some(id);
        if let Some(old_id) = old
            && old_id != id
        {
            if let Some(f) = sim.widgets.get_mut_visual(old_id) {
                f.editbox_focused = false;
            }
            sim.widgets.mark_visual_dirty(old_id);
        }
        if let Some(f) = sim.widgets.get_mut_visual(id) {
            f.editbox_focused = true;
        }
        sim.widgets.mark_visual_dirty(id);
        old
    };
    if old_focus != Some(id) {
        // TODO: fire OnEditFocusLost on old_focus, OnEditFocusGained on id
    }
    Ok(0)
}

pub(super) fn clear_focus(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "ScriptedInput", "ClearFocus")?;
    let cleared = {
        let mut sim = borrow_state_mut(state)?;
        if sim.focused_frame_id == Some(id) {
            sim.focused_frame_id = None;
            if let Some(f) = sim.widgets.get_mut_visual(id) {
                f.editbox_focused = false;
            }
            sim.widgets.mark_visual_dirty(id);
            true
        } else {
            false
        }
    };
    if cleared {
        // TODO: fire OnEditFocusLost
    }
    Ok(0)
}

pub(super) fn has_focus(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "QueryFocus", "HasFocus")?;
    let sim = borrow_state(state)?;
    let v = sim.focused_frame_id == Some(id);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn has_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .and_then(|f| f.text.as_ref())
        .is_some_and(|t| !t.is_empty());
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_cursor_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "ScriptedInput", "SetCursorPosition")?;
    let pos = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        let len = f.text.as_deref().unwrap_or("").chars().count() as i32;
        f.editbox_cursor_pos = pos.clamp(0, len);
    }
    Ok(0)
}

pub(super) fn get_cursor_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_cursor_pos)
        .unwrap_or(0);
    drop(sim);
    (v as f64).into_stack(state)
}

pub(super) fn get_num_letters(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .and_then(|f| f.text.as_ref())
        .map(|t| t.chars().count())
        .unwrap_or(0);
    drop(sim);
    (v as f64).into_stack(state)
}

pub(super) fn set_max_letters(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let max = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_max_letters = max;
    }
    Ok(0)
}

pub(super) fn get_max_letters(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_max_letters)
        .unwrap_or(0);
    drop(sim);
    (v as f64).into_stack(state)
}

pub(super) fn set_max_bytes(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let max = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_max_bytes = max;
    }
    Ok(0)
}
pub(super) fn get_max_bytes(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_max_bytes)
        .unwrap_or(0);
    drop(sim);
    (v as f64).into_stack(state)
}

pub(super) fn set_multi_line(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_multi_line = value;
    }
    Ok(0)
}

pub(super) fn is_multi_line(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_multi_line)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_auto_focus(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_auto_focus = value;
    }
    Ok(0)
}

pub(super) fn is_auto_focus(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_auto_focus)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_numeric(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_numeric = value;
    }
    Ok(0)
}

pub(super) fn is_numeric(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_numeric)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_password(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_password = value;
    }
    Ok(0)
}

pub(super) fn is_password(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_password)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_secure_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_secure_text = value;
    }
    Ok(0)
}

pub(super) fn is_secure_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_secure_text)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_security_disable_set_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_security_disable_set_text = true;
    }
    Ok(0)
}

pub(super) fn set_alphabetic_only(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_alphabetic_only = value;
    }
    Ok(0)
}

pub(super) fn is_alphabetic_only(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_alphabetic_only)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_numeric_full_range(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_numeric_full_range = value;
    }
    Ok(0)
}

pub(super) fn is_numeric_full_range(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_numeric_full_range)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_count_invisible_letters(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_count_invisible_letters = value;
    }
    Ok(0)
}

pub(super) fn is_count_invisible_letters(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_count_invisible_letters)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_visible_text_byte_limit(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let limit = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_visible_text_byte_limit = limit;
    }
    Ok(0)
}

pub(super) fn get_visible_text_byte_limit(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let limit = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_visible_text_byte_limit)
        .unwrap_or(0);
    drop(sim);
    (limit as f64).into_stack(state)
}

pub(super) fn set_security_disable_paste(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_security_disable_paste = true;
    }
    Ok(0)
}

pub(super) fn is_in_ime_composition_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_in_ime_composition_mode)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_highlight_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let r = val_to_f64(stack_val(state, 2)) as f32;
    let g = val_to_f64(stack_val(state, 3)) as f32;
    let b = val_to_f64(stack_val(state, 4)) as f32;
    let a = match stack_val(state, 5) {
        Val::Num(n) => n as f32,
        _ => 1.0,
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_highlight_color = crate::widget::Color::new(r, g, b, a);
    }
    Ok(0)
}

pub(super) fn get_highlight_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let color = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_highlight_color)
        .unwrap_or(crate::widget::Color::new(1.0, 1.0, 1.0, 1.0));
    drop(sim);
    (
        color.r as f64,
        color.g as f64,
        color.b as f64,
        color.a as f64,
    )
        .into_stack(state)
}

pub(super) fn get_utf8_cursor_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let cursor = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_cursor_pos.max(0) as usize)
        .unwrap_or(0);
    let byte_pos = sim
        .widgets
        .get(id)
        .and_then(|f| f.text.as_ref())
        .map(|text| {
            text.chars()
                .take(cursor)
                .map(|ch| ch.len_utf8())
                .sum::<usize>()
        })
        .unwrap_or(0);
    drop(sim);
    (byte_pos as f64).into_stack(state)
}

fn desired_width_field(state: &mut LuaState, id: u64) -> Option<f32> {
    let fields = get_or_create_frame_fields(state, id);
    match table_get(state, fields, "desiredWidth") {
        rilua::Val::Num(width) => Some(width as f32),
        _ => None,
    }
}

fn set_width_from_desired(state: &mut LuaState, id: u64, width: f32) {
    if let Ok(mut sim) = borrow_state_mut(state) {
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.width = width;
            frame.width_is_text_auto = false;
        }
        sim.widgets.mark_rect_dirty(id);
    }
    refresh_auto_text_height_after_width_change(state, id);
}

pub(super) fn set_desired_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let width = val_to_f64(stack_val(state, 2)) as f32;
    let fields = get_or_create_frame_fields(state, id);
    table_set(state, fields, "desiredWidth", rilua::Val::Num(width as f64));
    set_width_from_desired(state, id, width);
    Ok(0)
}

pub(super) fn get_desired_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let width = desired_width_field(state, id).or_else(|| {
        let sim = borrow_state(state).ok()?;
        sim.widgets.get(id).map(|f| f.width)
    });
    state.push(rilua::Val::Num(width.unwrap_or(0.0) as f64));
    Ok(1)
}

pub(super) fn get_scaled_desired_width(state: &mut LuaState) -> LuaResult<u32> {
    get_desired_width(state)
}

pub(super) fn get_desired_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let height = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).map(|f| f.height)
    };
    state.push(rilua::Val::Num(height.unwrap_or(0.0) as f64));
    Ok(1)
}

pub(super) fn get_scaled_desired_height(state: &mut LuaState) -> LuaResult<u32> {
    get_desired_height(state)
}

pub(super) fn update_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if let Some(width) = desired_width_field(state, id) {
        set_width_from_desired(state, id, width);
    }
    Ok(0)
}

pub(super) fn on_text_scale_updated(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if let Some(width) = desired_width_field(state, id) {
        set_width_from_desired(state, id, width);
    }
    Ok(0)
}

pub(super) fn set_number(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let n = val_to_f64(stack_val(state, 2));
    let s = n.to_string();
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.text_stripped = Some(s.clone());
        f.text = Some(s);
    }
    Ok(0)
}

pub(super) fn get_number(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .and_then(|f| f.text.as_ref())
        .and_then(|t| t.parse::<f64>().ok())
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn add_history_line(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text = opt_string(state, 2).unwrap_or_default();
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_history.push(text);
        let max = f.editbox_history_max;
        if max > 0 && f.editbox_history.len() > max as usize {
            f.editbox_history.remove(0);
        }
    }
    Ok(0)
}

pub(super) fn get_history_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_history.len())
        .unwrap_or(0);
    drop(sim);
    (v as f64).into_stack(state)
}

pub(super) fn set_history_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let max = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_history_max = max;
    }
    Ok(0)
}

pub(super) fn clear_history(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_history.clear();
    }
    Ok(0)
}

pub(super) fn get_input_language(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let lang = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| f.editbox_input_language.clone())
            .unwrap_or_else(|| "ROMAN".to_string())
    };
    let val = create_string(state, &lang);
    val.into_stack(state)
}

pub(super) fn toggle_input_language(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_input_language = if f.editbox_input_language == "ROMAN" {
            "NATIVE".to_string()
        } else {
            "ROMAN".to_string()
        };
    }
    Ok(0)
}

pub(super) fn reset_input_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_input_language = "ROMAN".to_string();
    }
    Ok(0)
}

pub(super) fn set_text_insets(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let l = val_to_f64(stack_val(state, 2)) as f32;
    let r = val_to_f64(stack_val(state, 3)) as f32;
    let t = val_to_f64(stack_val(state, 4)) as f32;
    let b = val_to_f64(stack_val(state, 5)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_text_insets = (l, r, t, b);
    }
    Ok(0)
}

pub(super) fn get_text_insets(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let (l, r, t, b) = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_text_insets)
        .unwrap_or((0.0, 0.0, 0.0, 0.0));
    drop(sim);
    (l as f64, r as f64, t as f64, b as f64).into_stack(state)
}

/// `SetSpacing(spacing)` — FontString/EditBox line spacing.
pub(super) fn set_spacing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let spacing = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.text_line_spacing = spacing;
    }
    Ok(0)
}

/// `GetSpacing()` — returns the spacing stored by `SetSpacing`.
pub(super) fn get_spacing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let spacing = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| f.text_line_spacing)
            .unwrap_or(0.0)
    };
    (spacing as f64).into_stack(state)
}

pub(super) fn get_display_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .and_then(|f| f.text.clone())
            .unwrap_or_default()
    };
    let val = create_string(state, &text);
    val.into_stack(state)
}

pub(super) fn insert(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text = opt_string(state, 2).unwrap_or_default();
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        let pos = f.editbox_cursor_pos.max(0) as usize;
        let current = f.text.get_or_insert_with(String::new);
        let insert_at = pos.min(current.len());
        current.insert_str(insert_at, &text);
        f.text_stripped = Some(crate::render::strip_wow_markup(current));
        f.text_segments.clear();
        f.editbox_cursor_pos = (insert_at + text.len()) as i32;
    }
    Ok(0)
}

pub(super) fn set_blink_speed(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let speed = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_blink_speed = speed;
    }
    Ok(0)
}

pub(super) fn get_blink_speed(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_blink_speed)
        .unwrap_or(0.5);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_alt_arrow_key_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_alt_arrow_key_mode = value;
    }
    Ok(0)
}

pub(super) fn get_alt_arrow_key_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.editbox_alt_arrow_key_mode)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn register_editbox(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    registration::register_editbox(state, metatable)
}
