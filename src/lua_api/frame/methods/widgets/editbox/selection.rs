//! EditBox selection and highlight methods.

use super::{borrow_state_mut, char_offset, frame_id_from_stack, stack_val, val_to_f64};
use crate::widget::Frame;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::ops::Range;

pub(super) fn highlight_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let start = val_to_f64(stack_val(state, 2)) as i32;
    let end_raw = stack_val(state, 3);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        let text = f.text.as_deref().unwrap_or("");
        let end = match end_raw {
            Val::Num(n) => n as i32,
            _ => text.len() as i32,
        };
        let (start, end) = normalize_highlight_range(
            char_offset(text, start) as i32,
            char_offset(text, end) as i32,
            text.chars().count() as i32,
        );
        f.editbox_highlight_range = Some((start, end));
    }
    Ok(0)
}

pub(super) fn clear_highlight_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.editbox_highlight_range = None;
    }
    Ok(0)
}

pub(super) fn take_selected_range(frame: &mut Frame) -> Option<Range<usize>> {
    frame
        .editbox_highlight_range
        .take()
        .and_then(|(start, end)| (start != end).then(|| start as usize..end as usize))
}

fn normalize_highlight_range(start: i32, end: i32, len: i32) -> (i32, i32) {
    let start = start.clamp(0, len);
    let end = end.clamp(0, len);
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}
