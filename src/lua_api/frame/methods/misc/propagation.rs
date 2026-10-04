//! Mouse click/motion and hyperlink propagation methods.

use crate::lua_api::frame::methods::forbidden_aspects;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use crate::widget::Frame;
use rilua::vm::closure::RustFn;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    for (name, function) in PROPAGATION_METHODS {
        register_propagation_method(state, mt, name, *function)?;
    }
    Ok(())
}

fn register_propagation_method(
    state: &mut LuaState,
    mt: GcRef<Table>,
    name: &'static str,
    function: RustFn,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, name, function)
}

const PROPAGATION_METHODS: &[(&str, RustFn)] = &[
    ("CanPropagateMouseClicks", can_propagate_mouse_clicks),
    ("CanPropagateMouseMotion", can_propagate_mouse_motion),
    (
        "DoesHyperlinkPropagateToParent",
        does_hyperlink_propagate_to_parent,
    ),
    (
        "SetHyperlinkPropagateToParent",
        set_hyperlink_propagate_to_parent,
    ),
    ("SetPropagateMouseClicks", set_propagate_mouse_clicks),
    ("SetPropagateMouseMotion", set_propagate_mouse_motion),
];

pub fn can_propagate_mouse_clicks(state: &mut LuaState) -> LuaResult<u32> {
    push_forced_mouse_propagation(state, |frame| frame.propagate_mouse_clicks)
}

/// AlwaysPropagateInput forces propagation without rewriting the stored preference.
fn push_forced_mouse_propagation(
    state: &mut LuaState,
    read: impl FnOnce(&Frame) -> bool,
) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let requested = borrow_state(state)?.widgets.get(id).is_some_and(read);
    let propagate = requested || forbidden_aspects::input_propagation_forced(state, id)?;
    state.push(Val::Bool(propagate));
    Ok(1)
}

/// Clearing propagation is rejected under AlwaysPropagateInput.
/// INFERRED: mirrors SetPropagateKeyboardInput(false); the mouse setters are unannotated.
fn ensure_mouse_propagation_clearable(state: &mut LuaState, method_name: &str) -> LuaResult<()> {
    let id = frame_id_from_stack(state, 1)?;
    if bool::from_stack(state, 2)? {
        return Ok(());
    }
    forbidden_aspects::ensure_forbidden_aspect_absent(
        state,
        id,
        "AlwaysPropagateInput",
        method_name,
    )
}

fn push_frame_bool(state: &mut LuaState, read: impl FnOnce(&Frame) -> bool) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let val = borrow_state(state)?
        .widgets
        .get(id)
        .map(read)
        .unwrap_or(false);
    state.push(Val::Bool(val));
    Ok(1)
}

pub fn can_propagate_mouse_motion(state: &mut LuaState) -> LuaResult<u32> {
    push_forced_mouse_propagation(state, |frame| frame.propagate_mouse_motion)
}

pub fn does_hyperlink_propagate_to_parent(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_bool(state, |frame| frame.propagate_hyperlinks_to_parent)
}

pub fn set_hyperlink_propagate_to_parent(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    forbidden_aspects::ensure_forbidden_aspect_absent(
        state,
        id,
        "UntrustedScriptExecution",
        "SetHyperlinkPropagateToParent",
    )?;
    set_frame_bool(state, |frame, value| {
        frame.propagate_hyperlinks_to_parent = value
    })
}

fn set_frame_bool(state: &mut LuaState, apply: impl FnOnce(&mut Frame, bool)) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = bool::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        apply(frame, value);
    }
    Ok(0)
}

pub fn set_propagate_mouse_clicks(state: &mut LuaState) -> LuaResult<u32> {
    ensure_mouse_propagation_clearable(state, "SetPropagateMouseClicks")?;
    set_frame_bool(state, |frame, value| frame.propagate_mouse_clicks = value)
}

pub fn set_propagate_mouse_motion(state: &mut LuaState) -> LuaResult<u32> {
    ensure_mouse_propagation_clearable(state, "SetPropagateMouseMotion")?;
    set_frame_bool(state, |frame, value| frame.propagate_mouse_motion = value)
}
