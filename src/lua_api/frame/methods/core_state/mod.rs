//! rilua RustFn equivalents of frame state methods.
//!
//! Covers the methods from `methods_core_state.rs`, `methods_visibility.rs`,
//! and `methods_core_region.rs`. Each function maps 1-to-1 with its mlua
//! counterpart using `frame_id_from_stack` + `borrow_state`/`borrow_state_mut`.

mod helpers;

pub mod alpha;
pub mod identity;
pub mod input;
pub mod region;
mod round_layout;
pub mod scale;
pub mod size;
pub mod strata_level;
pub mod visibility;

// Re-export all public functions so callers can use
// `core_state::get_width` etc. as before.
pub use alpha::*;
pub use identity::*;
pub use input::*;
pub use region::*;
pub use scale::*;
pub use size::*;
pub use strata_level::*;
pub use visibility::*;

use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

pub fn register_all(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    register_size(state, mt)?;
    register_visibility(state, mt)?;
    register_alpha(state, mt)?;
    register_strata_level(state, mt)?;
    register_identity(state, mt)?;
    register_input(state, mt)?;
    register_scale(state, mt)?;
    register_region(state, mt)?;
    if cfg!(any(
        feature = "retail-12-1-5",
        feature = "client-wowforever"
    )) {
        round_layout::register(state, mt)?;
    }
    Ok(())
}

fn register_size(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "GetWidth", get_width)?;
    table_set_rust_fn_static(state, mt, "GetHeight", get_height)?;
    table_set_rust_fn_static(state, mt, "GetSize", get_size)?;
    table_set_rust_fn_static(state, mt, "SetSize", set_size)?;
    table_set_rust_fn_static(state, mt, "SetFixedSize", set_fixed_size)?;
    table_set_rust_fn_static(state, mt, "SetWidth", set_width)?;
    table_set_rust_fn_static(state, mt, "SetHeight", set_height)?;
    Ok(())
}

fn register_visibility(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "Show", show)?;
    table_set_rust_fn_static(state, mt, "Hide", hide)?;
    table_set_rust_fn_static(state, mt, "SetShown", set_shown)?;
    table_set_rust_fn_static(state, mt, "IsVisible", is_visible)?;
    table_set_rust_fn_static(state, mt, "IsShown", is_shown)?;
    table_set_rust_fn_static(state, mt, "SetCollapsesLayout", set_collapses_layout)?;
    table_set_rust_fn_static(state, mt, "CollapsesLayout", collapses_layout)?;
    table_set_rust_fn_static(state, mt, "IsCollapsed", is_collapsed)?;
    // Dropdown menus (always closed in headless mode)
    table_set_rust_fn_static(state, mt, "IsMenuOpen", is_menu_open)?;
    Ok(())
}

fn register_alpha(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "SetAlpha", set_alpha)?;
    table_set_rust_fn_static(state, mt, "GetAlpha", get_alpha)?;
    table_set_rust_fn_static(state, mt, "GetEffectiveAlpha", get_effective_alpha)?;
    table_set_rust_fn_static(state, mt, "SetAlphaFromBoolean", set_alpha_from_boolean)?;
    table_set_rust_fn_static(state, mt, "SetIgnoreParentAlpha", set_ignore_parent_alpha)?;
    table_set_rust_fn_static(state, mt, "GetIgnoreParentAlpha", get_ignore_parent_alpha)?;
    table_set_rust_fn_static(state, mt, "IsIgnoringParentAlpha", is_ignoring_parent_alpha)?;
    Ok(())
}

fn register_strata_level(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "SetFrameStrata", set_frame_strata)?;
    table_set_rust_fn_static(state, mt, "GetFrameStrata", get_frame_strata)?;
    table_set_rust_fn_static(state, mt, "SetFixedFrameStrata", set_fixed_frame_strata)?;
    table_set_rust_fn_static(state, mt, "HasFixedFrameStrata", has_fixed_frame_strata)?;
    table_set_rust_fn_static(state, mt, "SetFrameLevel", set_frame_level)?;
    table_set_rust_fn_static(state, mt, "GetFrameLevel", get_frame_level)?;
    table_set_rust_fn_static(state, mt, "SetFixedFrameLevel", set_fixed_frame_level)?;
    table_set_rust_fn_static(state, mt, "HasFixedFrameLevel", has_fixed_frame_level)?;
    table_set_rust_fn_static(state, mt, "SetToplevel", set_toplevel)?;
    table_set_rust_fn_static(state, mt, "IsToplevel", is_toplevel)?;
    Ok(())
}

fn register_identity(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "GetName", get_name)?;
    table_set_rust_fn_static(state, mt, "GetDebugName", get_debug_name)?;
    table_set_rust_fn_static(state, mt, "GetObjectType", get_object_type)?;
    table_set_rust_fn_static(state, mt, "IsObjectType", is_object_type)?;
    table_set_rust_fn_static(state, mt, "SetID", set_id)?;
    table_set_rust_fn_static(state, mt, "GetID", get_id)?;
    table_set_rust_fn_static(state, mt, "GetMapID", get_map_id)?;
    table_set_rust_fn_static(state, mt, "GetUiMapID", get_ui_map_id)?;
    table_set_rust_fn_static(state, mt, "SetMapID", set_map_id)?;
    table_set_rust_fn_static(state, mt, "SetUiMapID", set_map_id)?;
    Ok(())
}

fn register_input(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "EnableMouse", enable_mouse)?;
    table_set_rust_fn_static(state, mt, "IsMouseEnabled", is_mouse_enabled)?;
    table_set_rust_fn_static(state, mt, "EnableMouseWheel", enable_mouse_wheel)?;
    table_set_rust_fn_static(state, mt, "IsMouseWheelEnabled", is_mouse_wheel_enabled)?;
    table_set_rust_fn_static(state, mt, "EnableKeyboard", enable_keyboard)?;
    table_set_rust_fn_static(state, mt, "IsKeyboardEnabled", is_keyboard_enabled)?;
    table_set_rust_fn_static(state, mt, "RegisterForMouse", register_for_mouse)?;
    table_set_rust_fn_static(state, mt, "EnableMouseMotion", enable_mouse_motion)?;
    table_set_rust_fn_static(state, mt, "IsMouseMotionEnabled", is_mouse_motion_enabled)?;
    table_set_rust_fn_static(state, mt, "SetMouseMotionEnabled", set_mouse_motion_enabled)?;
    table_set_rust_fn_static(state, mt, "SetMouseClickEnabled", set_mouse_click_enabled)?;
    table_set_rust_fn_static(state, mt, "IsMouseClickEnabled", is_mouse_click_enabled)?;
    Ok(())
}

fn register_scale(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "GetScale", get_scale)?;
    table_set_rust_fn_static(state, mt, "GetEffectiveScale", get_effective_scale)?;
    table_set_rust_fn_static(state, mt, "SetScale", set_scale)?;
    table_set_rust_fn_static(state, mt, "SetIgnoreParentScale", set_ignore_parent_scale)?;
    table_set_rust_fn_static(state, mt, "GetIgnoreParentScale", get_ignore_parent_scale)?;
    table_set_rust_fn_static(state, mt, "IsIgnoringParentScale", is_ignoring_parent_scale)?;
    Ok(())
}

fn register_region(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "IsRectValid", is_rect_valid)?;
    table_set_rust_fn_static(state, mt, "IsMouseMotionFocus", is_mouse_motion_focus)?;
    table_set_rust_fn_static(state, mt, "IsObjectLoaded", is_object_loaded)?;
    table_set_rust_fn_static(state, mt, "IsMouseOver", is_mouse_over)?;
    table_set_rust_fn_static(state, mt, "StopAnimating", stop_animating)?;
    table_set_rust_fn_static(state, mt, "GetSourceLocation", get_source_location)?;
    table_set_rust_fn_static(state, mt, "Intersects", intersects)?;
    table_set_rust_fn_static(state, mt, "IsDrawLayerEnabled", is_draw_layer_enabled)?;
    table_set_rust_fn_static(state, mt, "SetDrawLayerEnabled", set_draw_layer_enabled)?;
    Ok(())
}
