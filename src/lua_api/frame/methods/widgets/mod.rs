//! rilua RustFn equivalents of widget-specific frame methods.
//!
//! Covers: Texture/DrawLayer, Cooldown, EditBox, Slider/CheckButton/StatusBar
//! shared value, StatusBar, ScrollFrame, Model/ModelScene, and GameTooltip
//! widget methods.
//!
//! Each method is a plain `fn(&mut LuaState) -> LuaResult<u32>` (i.e. a
//! `RustFn`) that uses the helpers in `methods` to extract the frame
//! ID from the first stack argument and borrow `SimState`.

mod browser;
mod checkout;
mod cooldown;
mod editbox;
pub mod message_frame;
mod model;
pub(super) mod shared;
mod slider;
mod statusbar;
mod texture;
mod tooltip;

use crate::lua_api::methods::{borrow_state, frame_id_from_stack};
use crate::lua_bridge::table_set_rust_fn_static;
use crate::widget::WidgetType;
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

pub(crate) fn refresh_scroll_frames_for_resized_frame(
    state: &mut LuaState,
    resized_id: u64,
) -> LuaResult<()> {
    slider::refresh_scroll_frames_for_resized_frame(state, resized_id)
}

pub(crate) fn toggle_checkbutton_for_click(state: &mut LuaState, id: u64) -> LuaResult<()> {
    slider::toggle_checkbutton_for_click(state, id)
}

/// Dispatch the shared Clear name without overwriting either widget's handler.
fn clear(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let widget_type = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.widget_type);
    match widget_type {
        Some(WidgetType::Cooldown) => cooldown::clear(state),
        Some(WidgetType::MessageFrame) => message_frame::clear(state),
        _ => Err(rilua::runtime_error(
            "Clear is unsupported for this widget type",
        )),
    }
}

/// Register all widget-specific methods on the frame metatable.
///
/// Call this after the standard frame metatable has been created,
/// passing its `GcRef<Table>` as `metatable`.
pub fn register_all(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    browser::register_browser(state, metatable)?;
    texture::register_texture(state, metatable)?;
    cooldown::register_cooldown(state, metatable)?;
    checkout::register_checkout(state, metatable)?;
    editbox::register_editbox(state, metatable)?;
    slider::register_slider(state, metatable)?;
    slider::register_shared_value(state, metatable)?;
    slider::register_checkbutton(state, metatable)?;
    slider::register_colorselect(state, metatable)?;
    slider::register_scrollframe(state, metatable)?;
    statusbar::register_statusbar(state, metatable)?;
    model::register_model(state, metatable)?;
    tooltip::register_tooltip(state, metatable)?;
    message_frame::register_message_frame(state, metatable)?;
    table_set_rust_fn_static(state, metatable, "Clear", clear)
}
