//! EditBox presentation at dirty flushes and text notifications at host boundaries.

use std::cell::RefCell;
use std::rc::Rc;

use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, Val};

use super::env::{WowLuaAppData, WowLuaEnv};
use super::state::SimState;
use crate::render::editbox_layout::{
    EditBoxLayoutRequest, PreparedEditBoxLayout, prepare_editbox_layout,
};
use crate::render::font::WowFontSystem;
use crate::widget::editbox::EditBoxLayoutInput;

type CursorGeometry = (f32, f32, f32, f32);
type FontSystem = Rc<RefCell<WowFontSystem>>;

enum LayoutRefresh {
    Unavailable,
    Unchanged,
    Changed(EditBoxLayoutInput),
}

struct EditBoxUpdate {
    cursor: Option<CursorGeometry>,
    text_changed: Option<bool>,
}

pub(crate) fn flush(env: &WowLuaEnv) -> crate::Result<()> {
    let fonts = {
        let mut lua = env.rilua_mut();
        lua.state_mut()
            .app_data::<WowLuaAppData>()
            .and_then(|data| data.font_system.clone())
    };
    flush_updates(env.state(), fonts.as_ref(), |id, name, args| {
        env.dispatch_script_handler(id, name, args).map(|_| ())
    })
}

/// INFERRED R88et-c: defer host-input changes until keyboard dispatch completes.
pub(crate) fn flush_at_host_boundary(env: &WowLuaEnv) -> crate::Result<()> {
    let mut lua = env.rilua_mut();
    let state = lua.state_mut();
    let defer_user_input = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| crate::Error::Other("missing WowLuaAppData".to_string()))?
        .host_input_depth
        .get()
        > 0;
    flush_state_at_host_boundary_with_mode(state, defer_user_input)
}

pub(crate) fn flush_state_at_host_boundary(state: &mut LuaState) -> crate::Result<()> {
    flush_state_at_host_boundary_with_mode(state, false)
}

fn flush_state_at_host_boundary_with_mode(
    state: &mut LuaState,
    defer_user_input: bool,
) -> crate::Result<()> {
    if state.call_depth != 0 {
        return Ok(());
    }
    let sim = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| crate::Error::Other("missing WowLuaAppData".to_string()))?
        .sim_state
        .clone();
    loop {
        let pending = take_pending_text_changes(&sim, defer_user_input);
        if pending.is_empty() {
            return Ok(());
        }
        for (id, user_input) in pending {
            super::env_events::fire_script_handler_state(
                state,
                id,
                "OnTextChanged",
                vec![Val::Bool(user_input)],
            )?;
        }
    }
}

/// Host boundaries without pending notifications must not need a mutable
/// state borrow, so callers may keep reading `SimState` across `eval`.
fn has_pending_text_changes(sim: &SimState, defer_user_input: bool) -> bool {
    sim.widgets.iter_editbox_ids().any(|id| {
        sim.widgets
            .get(id)
            .and_then(|frame| frame.editbox_presentation.as_ref())
            .is_some_and(|presentation| presentation.has_text_change(defer_user_input))
    })
}

fn take_pending_text_changes(
    sim: &Rc<RefCell<SimState>>,
    defer_user_input: bool,
) -> Vec<(u64, bool)> {
    if !has_pending_text_changes(&sim.borrow(), defer_user_input) {
        return Vec::new();
    }
    let mut sim = sim.borrow_mut();
    let mut ids = sim.widgets.iter_editbox_ids().collect::<Vec<_>>();
    ids.sort_unstable();
    ids.into_iter()
        .filter_map(|id| {
            take_visible_text_change(&mut sim, id, defer_user_input)
                .map(|user_input| (id, user_input))
        })
        .collect()
}

/// The loader may already be executing inside Lua. Never borrow its owning cell.
pub(crate) fn flush_state(state: &mut LuaState) -> crate::Result<()> {
    let (sim, fonts) = {
        let data = state
            .app_data::<WowLuaAppData>()
            .ok_or_else(|| crate::Error::Other("missing WowLuaAppData".to_string()))?;
        (Rc::clone(&data.sim_state), data.font_system.clone())
    };
    flush_updates(&sim, fonts.as_ref(), |id, name, args| {
        super::env_events::fire_script_handler_state(state, id, name, args)
    })
}

fn flush_updates(
    sim: &Rc<RefCell<SimState>>,
    fonts: Option<&FontSystem>,
    mut dispatch: impl FnMut(u64, &str, Vec<Val>) -> crate::Result<()>,
) -> crate::Result<()> {
    for id in editbox_ids(sim) {
        let Some(update) = refresh_editbox(sim, id, fonts) else {
            continue;
        };
        // Preparation and commit have released every state/font borrow here.
        if let Some((x, y, width, height)) = update.cursor {
            dispatch(
                id,
                "OnCursorChanged",
                vec![
                    Val::Num(x.into()),
                    Val::Num(y.into()),
                    Val::Num(width.into()),
                    Val::Num(height.into()),
                ],
            )?;
        }
        if let Some(user_input) = update.text_changed {
            dispatch(id, "OnTextChanged", vec![Val::Bool(user_input)])?;
        }
    }
    Ok(())
}

fn editbox_ids(sim: &Rc<RefCell<SimState>>) -> Vec<u64> {
    let mut sim = sim.borrow_mut();
    sim.ensure_layout_rects();
    let mut ids = sim.widgets.iter_editbox_ids().collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn read_layout_refresh(sim: &Rc<RefCell<SimState>>, id: u64) -> LayoutRefresh {
    let sim = sim.borrow();
    let Some(frame) = sim.widgets.get(id) else {
        return LayoutRefresh::Unavailable;
    };
    let pristine = frame.editbox_presentation.is_none()
        && frame.text.as_deref().unwrap_or_default().is_empty()
        && frame.editbox_cursor_pos == 0;
    if pristine && !sim.widgets.is_ancestor_visible(id) {
        return LayoutRefresh::Unavailable;
    }
    let Some(rect) = frame.layout_rect else {
        return LayoutRefresh::Unavailable;
    };
    let size = (rect.width, rect.height);
    let unchanged = frame
        .editbox_presentation
        .as_ref()
        .and_then(|presentation| presentation.input.as_ref())
        .is_some_and(|input| input.matches_frame(frame, size));
    if unchanged {
        LayoutRefresh::Unchanged
    } else {
        LayoutRefresh::Changed(EditBoxLayoutInput::from_frame(frame, size))
    }
}

fn refresh_editbox(
    sim: &Rc<RefCell<SimState>>,
    id: u64,
    fonts: Option<&FontSystem>,
) -> Option<EditBoxUpdate> {
    match read_layout_refresh(sim, id) {
        LayoutRefresh::Unavailable => take_text_change(sim, id),
        LayoutRefresh::Unchanged => take_text_change(sim, id),
        LayoutRefresh::Changed(input) => {
            let layout = prepare_layout(fonts, &input);
            commit_presentation(sim, id, input, layout)
        }
    }
}

fn prepare_layout(
    fonts: Option<&FontSystem>,
    input: &EditBoxLayoutInput,
) -> Option<PreparedEditBoxLayout> {
    let scale = input.settings.scale;
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    prepare_editbox_layout(&mut fonts?.borrow_mut(), &layout_request(input))
}

fn layout_request(input: &EditBoxLayoutInput) -> EditBoxLayoutRequest<'_> {
    let settings = input.settings;
    let (_, _, width, height) = settings.content_bounds();
    EditBoxLayoutRequest {
        text: input.text.as_deref().unwrap_or_default(),
        font_path: input.font.as_deref(),
        font_size: settings.font_size,
        content_width: width,
        content_height: height,
        word_wrap: settings.word_wrap,
        line_spacing: settings.spacing,
        smooth_scaling: settings.smooth_scaling,
        justify_h: settings.justify_h,
        justify_v: settings.justify_v,
        cursor_scalar: input.cursor,
    }
}

fn frame_cursor_geometry(
    input: &EditBoxLayoutInput,
    (x, y, height): (f32, f32, f32),
) -> CursorGeometry {
    let settings = input.settings;
    let (left, top, _, _) = settings.content_bounds();
    (
        (left + x) / settings.scale,
        -(top + y) / settings.scale,
        1.5,
        height / settings.scale,
    )
}

fn commit_presentation(
    sim: &Rc<RefCell<SimState>>,
    id: u64,
    input: EditBoxLayoutInput,
    layout: Option<PreparedEditBoxLayout>,
) -> Option<EditBoxUpdate> {
    let cursor = layout
        .as_ref()
        .and_then(|layout| layout.cursor)
        .map(|cursor| frame_cursor_geometry(&input, cursor));
    let mut sim = sim.borrow_mut();
    let visible = sim.widgets.is_ancestor_visible(id);
    let frame = sim.widgets.get_mut(id)?;
    let presentation = frame.editbox_presentation_mut();
    presentation.input = Some(input);
    presentation.shaped_text = layout.map(|layout| layout.buffer);
    presentation.cursor_geometry = cursor;
    let text_changed = if visible {
        presentation.take_text_change(false)
    } else {
        None
    };
    sim.widgets.mark_visual_dirty(id);
    Some(EditBoxUpdate {
        cursor,
        text_changed,
    })
}

fn take_text_change(sim: &Rc<RefCell<SimState>>, id: u64) -> Option<EditBoxUpdate> {
    let text_changed = {
        let mut sim = sim.borrow_mut();
        take_visible_text_change(&mut sim, id, false)
    };
    Some(EditBoxUpdate {
        cursor: None,
        text_changed,
    })
}

/// INFERRED — R88g8 (native evidence pending): hidden EditBoxes retain notifications until visible.
fn take_visible_text_change(sim: &mut SimState, id: u64, defer_user_input: bool) -> Option<bool> {
    if !sim.widgets.is_ancestor_visible(id) {
        return None;
    }
    let frame = sim.widgets.get_mut(id)?;
    frame
        .editbox_presentation
        .as_mut()?
        .take_text_change(defer_user_input)
}
