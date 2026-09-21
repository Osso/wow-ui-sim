//! Slider, CheckButton, shared value (Slider+StatusBar), and ScrollFrame methods.

use super::shared::{opt_string, val_to_bool, val_to_f64};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, extract_frame_id, frame_id_from_stack, frame_ref,
    sync_child_to_rilua, table_get,
};
use crate::lua_api::script_helpers::{
    call_error_handler_state, get_script, protected_lua_pcall_state,
};
use crate::lua_bridge::{IntoStack, stack_val, table_set_rust_fn, table_set_rust_fn_static};
use crate::widget::WidgetType;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

mod checkbutton;
mod colorselect;
mod scroll_controller;
mod thumb_texture;
pub(super) use checkbutton::register_checkbutton;
pub(super) use checkbutton::toggle_checkbutton_for_click;
pub(super) use colorselect::register_colorselect;

// ---------------------------------------------------------------------------
// Slider
// ---------------------------------------------------------------------------

pub(super) fn set_value_step(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let step = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.slider_step = step;
    }
    Ok(0)
}

pub(super) fn get_value_step(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim.widgets.get(id).map(|f| f.slider_step).unwrap_or(1.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_orientation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let orientation = opt_string(state, 2)
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "HORIZONTAL".to_string());
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.slider_orientation = orientation;
    }
    Ok(0)
}

pub(super) fn get_orientation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let orientation = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| f.slider_orientation.clone())
            .unwrap_or_else(|| "HORIZONTAL".to_string())
    };
    let val = crate::lua_api::methods::create_string(state, &orientation);
    val.into_stack(state)
}

pub(super) fn set_obey_step_on_drag(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let obey = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.slider_obey_step_on_drag = obey;
    }
    Ok(0)
}

pub(super) fn get_obey_step_on_drag(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.slider_obey_step_on_drag)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_steps_per_page(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let steps = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.slider_steps_per_page = steps;
    }
    Ok(0)
}

pub(super) fn get_steps_per_page(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.slider_steps_per_page)
        .unwrap_or(1);
    drop(sim);
    (v as f64).into_stack(state)
}

pub(super) fn is_dragging_thumb(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim.active_slider_thumb_drag_frame == Some(id);
    drop(sim);
    v.into_stack(state)
}

// ---------------------------------------------------------------------------
// Shared value (Slider + StatusBar)
// ---------------------------------------------------------------------------

/// Clamp and apply `value` to a Slider, returning the clamped value.
/// Returns `None` if the value was unchanged (no update needed).
fn apply_slider_value(sim: &mut crate::lua_api::SimState, id: u64, value: f64) -> Option<f64> {
    let f = sim.widgets.get(id)?;
    let clamped = value.clamp(f.slider_min, f.slider_max);
    if clamped == f.slider_value {
        return None;
    }
    sim.widgets.get_mut_visual(id).unwrap().slider_value = clamped;
    Some(clamped)
}

/// Apply `value` to a StatusBar, clamping and updating interpolation state.
fn apply_statusbar_value(sim: &mut crate::lua_api::SimState, id: u64, value: f64) {
    let Some(f) = sim.widgets.get(id) else {
        return;
    };
    let clamped = value.clamp(f.statusbar_min, f.statusbar_max);
    if clamped == f.statusbar_value {
        return;
    }
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.statusbar_value = clamped;
        f.statusbar_interpolated_value = clamped;
        f.statusbar_interpolation_target = None;
    }
}

fn apply_statusbar_interpolated_value(sim: &mut crate::lua_api::SimState, id: u64, value: f64) {
    let Some(f) = sim.widgets.get(id) else {
        return;
    };
    let clamped = value.clamp(f.statusbar_min, f.statusbar_max);
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.statusbar_value = clamped;
        f.statusbar_interpolation_target = Some(clamped);
    }
}

pub(super) fn shared_set_value(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_f64(stack_val(state, 2));
    let interpolation_mode = opt_string(state, 3);
    let wtype = borrow_state(state)?.widgets.get(id).map(|f| f.widget_type);
    match wtype {
        Some(WidgetType::Slider) => {
            let clamped = {
                let mut sim = borrow_state_mut(state)?;
                apply_slider_value(&mut sim, id, value)
            };
            // TODO: fire OnValueChanged (needs rilua script dispatch)
            let _ = clamped;
        }
        Some(WidgetType::StatusBar) => {
            let mut sim = borrow_state_mut(state)?;
            if interpolation_mode.is_some() {
                apply_statusbar_interpolated_value(&mut sim, id, value);
            } else {
                apply_statusbar_value(&mut sim, id, value);
            }
        }
        _ => {}
    }
    Ok(0)
}

pub(super) fn shared_get_value(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    crate::lua_api::frame::methods::secret_origin::require_timing_readable(state, id)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| match f.widget_type {
            WidgetType::Slider => f.slider_value,
            WidgetType::StatusBar => f.statusbar_value,
            _ => 0.0,
        })
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn shared_set_min_max_values(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let min = val_to_f64(stack_val(state, 2));
    let max = val_to_f64(stack_val(state, 3));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        match f.widget_type {
            WidgetType::Slider => {
                f.slider_min = min;
                f.slider_max = max;
                f.slider_value = f.slider_value.clamp(min, max);
            }
            WidgetType::StatusBar => {
                f.statusbar_min = min;
                f.statusbar_max = max;
                f.statusbar_value = f.statusbar_value.clamp(min, max);
                f.statusbar_interpolated_value = f.statusbar_interpolated_value.clamp(min, max);
                f.statusbar_interpolation_target = f
                    .statusbar_interpolation_target
                    .map(|t| t.clamp(min, max))
                    .filter(|&t| t != f.statusbar_interpolated_value);
            }
            _ => {}
        }
    }
    Ok(0)
}

pub(super) fn shared_get_min_max_values(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    crate::lua_api::frame::methods::secret_origin::require_timing_readable(state, id)?;
    let sim = borrow_state(state)?;
    let (min, max) = sim
        .widgets
        .get(id)
        .map(|f| match f.widget_type {
            WidgetType::Slider => (f.slider_min, f.slider_max),
            WidgetType::StatusBar => (f.statusbar_min, f.statusbar_max),
            _ => (0.0, 1.0),
        })
        .unwrap_or((0.0, 1.0));
    drop(sim);
    (min, max).into_stack(state)
}

// ---------------------------------------------------------------------------
// ScrollFrame
// ---------------------------------------------------------------------------

fn scroll_child_extent(frame: &crate::widget::Frame, axis: char) -> f64 {
    match (frame.scroll_child_rect_size, axis) {
        (Some((width, _)), 'h') => width as f64,
        (Some((_, height)), 'v') => height as f64,
        _ => 0.0,
    }
}

fn scroll_range(frame: &crate::widget::Frame, axis: char) -> f64 {
    let child_extent = scroll_child_extent(frame, axis);
    let own_extent = match axis {
        'h' => frame.width as f64,
        'v' => frame.height as f64,
        _ => 0.0,
    };
    (child_extent - own_extent).max(0.0)
}

fn scroll_ranges(frame: &crate::widget::Frame) -> (f64, f64) {
    (scroll_range(frame, 'h'), scroll_range(frame, 'v'))
}

fn scroll_ranges_from_size(
    scroll_child_rect_size: Option<(f32, f32)>,
    own_width: f32,
    own_height: f32,
) -> (f64, f64) {
    let Some((child_width, child_height)) = scroll_child_rect_size else {
        return (0.0, 0.0);
    };
    (
        (child_width as f64 - own_width as f64).max(0.0),
        (child_height as f64 - own_height as f64).max(0.0),
    )
}

fn scroll_frames_affected_by_resize(state: &mut LuaState, resized_id: u64) -> LuaResult<Vec<u64>> {
    let sim = borrow_state(state)?;
    let mut affected = Vec::new();
    if sim
        .widgets
        .get(resized_id)
        .is_some_and(|frame| frame.scroll_child_id.is_some())
    {
        affected.push(resized_id);
    }

    let mut current_id = sim
        .widgets
        .get(resized_id)
        .and_then(|frame| frame.parent_id);
    while let Some(id) = current_id {
        let Some(frame) = sim.widgets.get(id) else {
            break;
        };
        if frame.scroll_child_id == Some(resized_id) {
            affected.push(id);
        }
        current_id = frame.parent_id;
    }
    Ok(affected)
}

fn fire_scroll_frame_event(
    state: &mut LuaState,
    frame_id: u64,
    handler_name: &str,
    args: &[Val],
) -> LuaResult<()> {
    call_scroll_frame_handler(state, frame_id, handler_name, args)?;
    let intrinsic_name = format!("{handler_name}_Intrinsic");
    call_scroll_frame_intrinsic(state, frame_id, &intrinsic_name, args)
}

fn call_scroll_frame_handler(
    state: &mut LuaState,
    frame_id: u64,
    handler_name: &str,
    args: &[Val],
) -> LuaResult<()> {
    let Some(handler) = get_script(state, frame_id, handler_name) else {
        return Ok(());
    };
    call_scroll_frame_function(state, frame_id, handler, args)
}

fn call_scroll_frame_intrinsic(
    state: &mut LuaState,
    frame_id: u64,
    intrinsic_name: &str,
    args: &[Val],
) -> LuaResult<()> {
    let frame = frame_ref(state, frame_id)?;
    let intrinsic = table_get(state, frame, intrinsic_name);
    if matches!(intrinsic, Val::Function(_)) {
        call_scroll_frame_function(state, frame_id, intrinsic, args)?;
    }
    Ok(())
}

fn call_scroll_frame_function(
    state: &mut LuaState,
    frame_id: u64,
    function: Val,
    args: &[Val],
) -> LuaResult<()> {
    let frame = frame_ref(state, frame_id)?;
    let mut call_args = Vec::with_capacity(args.len() + 1);
    call_args.push(frame);
    call_args.extend_from_slice(args);
    if let Err(error_msg) = protected_lua_pcall_state(state, function, &call_args) {
        call_error_handler_state(state, &error_msg);
    }
    Ok(())
}

pub(super) fn refresh_scroll_frames_for_resized_frame(
    state: &mut LuaState,
    resized_id: u64,
) -> LuaResult<()> {
    for scroll_frame_id in scroll_frames_affected_by_resize(state, resized_id)? {
        refresh_scroll_child_rect(state, scroll_frame_id)?;
    }
    Ok(())
}

fn scroll_child_subtree_bounds(
    registry: &crate::widget::WidgetRegistry,
    root_id: u64,
    screen_width: f32,
    screen_height: f32,
) -> Option<(f32, f32)> {
    let mut cache = crate::layout::LayoutCache::default();
    let mut stack = vec![root_id];
    let mut bounds: Option<(f32, f32, f32, f32)> = None;

    while let Some(id) = stack.pop() {
        let rect = crate::layout::compute_frame_rect_cached(
            registry,
            id,
            screen_width,
            screen_height,
            &mut cache,
        )
        .rect;
        bounds = Some(match bounds {
            Some((min_x, min_y, max_x, max_y)) => (
                min_x.min(rect.x),
                min_y.min(rect.y),
                max_x.max(rect.x + rect.width),
                max_y.max(rect.y + rect.height),
            ),
            None => (rect.x, rect.y, rect.x + rect.width, rect.y + rect.height),
        });
        if let Some(frame) = registry.get(id) {
            stack.extend(frame.children.iter().copied());
        }
    }

    bounds.map(|(min_x, min_y, max_x, max_y)| (max_x - min_x, max_y - min_y))
}

pub(super) fn get_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.scroll_horizontal)
        .unwrap_or(0.0);
    state.push(Val::Num(offset));
    Ok(1)
}

pub(super) fn set_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    let range = sim
        .widgets
        .get(id)
        .map(|frame| scroll_range(frame, 'h'))
        .unwrap_or(0.0);
    let new_offset = offset.clamp(0.0, range);
    if sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.scroll_horizontal == new_offset)
    {
        return Ok(0);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.scroll_horizontal = new_offset;
    }
    drop(sim);
    fire_scroll_frame_event(state, id, "OnHorizontalScroll", &[Val::Num(new_offset)])?;
    Ok(0)
}

pub(super) fn get_horizontal_scroll_range(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let range = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| scroll_range(frame, 'h'))
        .unwrap_or(0.0);
    state.push(Val::Num(range));
    Ok(1)
}

pub(super) fn get_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.scroll_vertical)
        .unwrap_or(0.0);
    state.push(Val::Num(offset));
    Ok(1)
}

pub(super) fn set_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    let range = sim
        .widgets
        .get(id)
        .map(|frame| scroll_range(frame, 'v'))
        .unwrap_or(0.0);
    let new_offset = offset.clamp(0.0, range);
    if sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.scroll_vertical == new_offset)
    {
        return Ok(0);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.scroll_vertical = new_offset;
    }
    drop(sim);
    fire_scroll_frame_event(state, id, "OnVerticalScroll", &[Val::Num(new_offset)])?;
    Ok(0)
}

pub(super) fn get_vertical_scroll_range(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let range = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| scroll_range(frame, 'v'))
        .unwrap_or(0.0);
    state.push(Val::Num(range));
    Ok(1)
}

pub(super) fn get_scroll_child(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let child_id = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).and_then(|frame| frame.scroll_child_id)
    };
    match child_id {
        Some(child_id) => {
            let child = frame_ref(state, child_id)?;
            state.push(child);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn set_scroll_child(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let child = stack_val(state, 2);
    let Some(child_id) = extract_frame_id(state, child) else {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.scroll_child_id = None;
            frame.scroll_child_rect_size = None;
        }
        return Ok(0);
    };
    {
        let mut sim = borrow_state_mut(state)?;
        crate::lua_api::frame::methods::widget_scroll::assign_scroll_child(
            &mut sim, id, child_id, true,
        );
    }
    let _ = sync_child_to_rilua(state, id, "ScrollChild", child_id);
    Ok(0)
}

pub(super) fn update_scroll_child_rect(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    refresh_scroll_child_rect(state, id)?;
    Ok(0)
}

fn refresh_scroll_child_rect(state: &mut LuaState, id: u64) -> LuaResult<()> {
    let (old_ranges, new_size, own_width, own_height) = {
        let sim = borrow_state(state)?;
        let Some(frame) = sim.widgets.get(id) else {
            return Ok(());
        };
        let new_size = frame.scroll_child_id.and_then(|child_id| {
            scroll_child_subtree_bounds(&sim.widgets, child_id, sim.screen_width, sim.screen_height)
        });
        (scroll_ranges(frame), new_size, frame.width, frame.height)
    };
    let new_ranges = scroll_ranges_from_size(new_size, own_width, own_height);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.scroll_child_rect_size = new_size;
    }
    drop(sim);
    if old_ranges != new_ranges {
        fire_scroll_frame_event(
            state,
            id,
            "OnScrollRangeChanged",
            &[Val::Num(new_ranges.0), Val::Num(new_ranges.1)],
        )?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// register_slider / register_shared_value / register_scrollframe / register_checkbutton
// ---------------------------------------------------------------------------

pub(super) fn register_slider(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "SetValueStep", set_value_step)?;
    table_set_rust_fn_static(state, metatable, "GetValueStep", get_value_step)?;
    table_set_rust_fn_static(state, metatable, "SetOrientation", set_orientation)?;
    table_set_rust_fn_static(state, metatable, "GetOrientation", get_orientation)?;
    table_set_rust_fn_static(state, metatable, "SetObeyStepOnDrag", set_obey_step_on_drag)?;
    table_set_rust_fn_static(state, metatable, "GetObeyStepOnDrag", get_obey_step_on_drag)?;
    table_set_rust_fn_static(state, metatable, "SetStepsPerPage", set_steps_per_page)?;
    table_set_rust_fn_static(state, metatable, "GetStepsPerPage", get_steps_per_page)?;
    table_set_rust_fn_static(state, metatable, "IsDraggingThumb", is_dragging_thumb)?;
    table_set_rust_fn_static(
        state,
        metatable,
        "SetThumbTexture",
        thumb_texture::set_thumb_texture,
    )?;
    table_set_rust_fn_static(
        state,
        metatable,
        "GetThumbTexture",
        thumb_texture::get_thumb_texture,
    )?;
    table_set_rust_fn_static(
        state,
        metatable,
        "SetScrollPercentage",
        scroll_controller::set_scroll_percentage,
    )?;
    table_set_rust_fn_static(
        state,
        metatable,
        "GetScrollPercentage",
        scroll_controller::get_scroll_percentage,
    )?;
    table_set_rust_fn_static(
        state,
        metatable,
        "SetVisibleExtentPercentage",
        scroll_controller::set_visible_extent_percentage,
    )?;
    table_set_rust_fn_static(
        state,
        metatable,
        "GetVisibleExtentPercentage",
        scroll_controller::get_visible_extent_percentage,
    )?;
    table_set_rust_fn_static(
        state,
        metatable,
        "SetPanExtentPercentage",
        scroll_controller::set_pan_extent_percentage,
    )?;
    table_set_rust_fn_static(
        state,
        metatable,
        "GetPanExtentPercentage",
        scroll_controller::get_pan_extent_percentage,
    )?;
    table_set_rust_fn_static(
        state,
        metatable,
        "ScrollStepInDirection",
        scroll_controller::scroll_step_in_direction,
    )?;
    Ok(())
}

pub(super) fn register_shared_value(
    state: &mut LuaState,
    metatable: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "SetValue", shared_set_value)?;
    table_set_rust_fn_static(state, metatable, "GetValue", shared_get_value)?;
    table_set_rust_fn_static(
        state,
        metatable,
        "SetMinMaxValues",
        shared_set_min_max_values,
    )?;
    table_set_rust_fn_static(
        state,
        metatable,
        "GetMinMaxValues",
        shared_get_min_max_values,
    )?;
    Ok(())
}

const SCROLLFRAME_METHODS: &[(&str, rilua::vm::closure::RustFn)] = &[
    ("GetHorizontalScroll", get_horizontal_scroll),
    ("SetHorizontalScroll", set_horizontal_scroll),
    ("GetHorizontalScrollRange", get_horizontal_scroll_range),
    ("GetVerticalScroll", get_vertical_scroll),
    ("SetVerticalScroll", set_vertical_scroll),
    ("GetVerticalScrollRange", get_vertical_scroll_range),
    ("GetScrollChild", get_scroll_child),
    ("SetScrollChild", set_scroll_child),
    ("UpdateScrollChildRect", update_scroll_child_rect),
];

pub(super) fn register_scrollframe(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    for (name, func) in SCROLLFRAME_METHODS {
        table_set_rust_fn(state, metatable, name, *func)?;
    }
    Ok(())
}
