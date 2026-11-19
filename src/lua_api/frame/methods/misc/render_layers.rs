//! Flatten render layers, window display, and don't-save-position methods.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, frame_id_from_stack, get_or_create_frame_fields, table_get,
    table_set,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::widget::Frame;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        mt,
        "GetEffectivelyFlattensRenderLayers",
        get_effectively_flattens_render_layers,
    )?;
    table_set_rust_fn_static(
        state,
        mt,
        "GetFlattensRenderLayers",
        get_flattens_render_layers,
    )?;
    table_set_rust_fn_static(state, mt, "GetDontSavePosition", get_dont_save_position)?;
    table_set_rust_fn_static(state, mt, "SetDontSavePosition", set_dont_save_position)?;
    table_set_rust_fn_static(state, mt, "GetWindow", get_window)?;
    table_set_rust_fn_static(state, mt, "SetWindow", set_window)?;
    Ok(())
}

pub fn get_effectively_flattens_render_layers(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let val = any_ancestor_flattens(&borrow_state(state)?.widgets, id);
    state.push(Val::Bool(val));
    Ok(1)
}

fn any_ancestor_flattens(widgets: &crate::widget::WidgetRegistry, start: u64) -> bool {
    let mut current = Some(start);
    while let Some(fid) = current {
        let Some(f) = widgets.get(fid) else { break };
        if f.flattens_render_layers {
            return true;
        }
        current = f.parent_id;
    }
    false
}

pub fn get_flattens_render_layers(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_bool(state, |frame| frame.flattens_render_layers)
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

pub fn get_dont_save_position(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_bool(state, |frame| frame.dont_save_position)
}

pub fn set_dont_save_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = matches!(stack_val(state, 2), Val::Bool(true));
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.dont_save_position = value;
    }
    Ok(0)
}

const WINDOW_FIELD: &str = "__window";

pub fn get_window(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let fields = get_or_create_frame_fields(state, id);
    let window = table_get(state, fields, WINDOW_FIELD);
    state.push(window);
    Ok(1)
}

pub fn set_window(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let window = stack_val(state, 2);
    let fields = get_or_create_frame_fields(state, id);
    table_set(state, fields, WINDOW_FIELD, window);
    if crate::widget::clamp_geometry::uses_retail_clamp_producers()
        && let Some(frame) = borrow_state_mut(state)?.widgets.get_mut(id)
        && frame.has_frame_carrier()
    {
        // Existing simulated Window objects are tables. Store only their
        // presence in geometry; ownership/identity stays with the Lua field.
        frame.has_window_movement_context = matches!(window, Val::Table(_));
    }
    Ok(0)
}
