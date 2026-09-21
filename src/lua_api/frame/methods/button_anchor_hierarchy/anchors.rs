//! Anchor methods: SetPoint, GetPoint, ClearAllPoints, line endpoints, etc.

#[cfg(feature = "forbidden-aspects")]
use crate::lua_api::frame::methods::forbidden_aspects;
use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, frame_id_from_stack,
    val_to_string,
};
use crate::lua_bridge::{FromStack, IntoStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

use super::shared::{
    frame_global_or_ref, opt_f32, parse_anchor_point_with_compat_warning, resolve_anchor_target_id,
    resolve_relative_point_from_val, unresolved_anchor_key_expr,
};

// ── Line anchor helpers ───────────────────────────────────────────────────────

fn parse_line_anchor_args(
    state: &mut LuaState,
    frame_id: u64,
) -> LuaResult<(crate::widget::AnchorPoint, Option<u64>, f32, f32)> {
    let point_name = String::from_stack(state, 2)?;
    let point = parse_anchor_point_with_compat_warning(state, &point_name).ok_or_else(|| {
        runtime_error(format!(
            "Line anchor point must be a valid region point, got {point_name}"
        ))
    })?;

    let arg3 = stack_val(state, 3);
    let arg4 = stack_val(state, 4);
    let arg5 = stack_val(state, 5);

    let x = match arg3 {
        Val::Num(n) => Some(n as f32),
        _ => None,
    };
    let y = match arg4 {
        Val::Num(n) => Some(n as f32),
        _ => None,
    };
    if let (Some(x_offset), Some(y_offset)) = (x, y) {
        return Ok((point, None, x_offset, y_offset));
    }

    let target_id = resolve_anchor_target_id(state, frame_id, arg3).map(|id| id as u64);
    let x_offset = match arg4 {
        Val::Num(n) => n as f32,
        _ => 0.0,
    };
    let y_offset = match arg5 {
        Val::Num(n) => n as f32,
        _ => 0.0,
    };
    Ok((point, target_id, x_offset, y_offset))
}

fn set_line_endpoint(state: &mut LuaState, is_start: bool) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (point, target_id, x_offset, y_offset) = parse_line_anchor_args(state, id)?;

    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id)
        && frame.widget_type == crate::widget::WidgetType::Line
    {
        let anchor = crate::widget::LineAnchor {
            point,
            target_id,
            x_offset,
            y_offset,
        };
        if is_start {
            frame.line_start = Some(anchor);
        } else {
            frame.line_end = Some(anchor);
        }
        sim.widgets.mark_rect_dirty(id);
    }

    Ok(0)
}

fn get_line_endpoint(state: &mut LuaState, is_start: bool) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let anchor = {
        let sim = borrow_state(state)?;
        let Some(frame) = sim.widgets.get(id) else {
            return Ok(0);
        };
        if frame.widget_type != crate::widget::WidgetType::Line {
            return Ok(0);
        }
        if is_start {
            frame.line_start.clone()
        } else {
            frame.line_end.clone()
        }
    };

    let Some(anchor) = anchor else {
        return Ok(0);
    };

    let point_val = create_string(state, anchor.point.as_str());
    let target_val = match anchor.target_id {
        Some(target_id) => frame_global_or_ref(state, target_id)?,
        None => Val::Nil,
    };
    state.push(point_val);
    state.push(target_val);
    state.push(Val::Num(anchor.x_offset as f64));
    state.push(Val::Num(anchor.y_offset as f64));
    Ok(4)
}

pub(super) fn set_start_point(state: &mut LuaState) -> LuaResult<u32> {
    set_line_endpoint(state, true)
}

pub(super) fn get_start_point(state: &mut LuaState) -> LuaResult<u32> {
    get_line_endpoint(state, true)
}

pub(super) fn set_end_point(state: &mut LuaState) -> LuaResult<u32> {
    set_line_endpoint(state, false)
}

pub(super) fn get_end_point(state: &mut LuaState) -> LuaResult<u32> {
    get_line_endpoint(state, false)
}

// ── SetPoint args parsing ─────────────────────────────────────────────────────

type ParsedSetPointArgs = (
    Option<usize>,
    Option<String>,
    crate::widget::AnchorPoint,
    f32,
    f32,
);

pub(super) fn parse_set_point_args(
    state: &mut LuaState,
    frame_id: u64,
    point: crate::widget::AnchorPoint,
) -> LuaResult<ParsedSetPointArgs> {
    let arg3 = stack_val(state, 3);
    let arg4 = stack_val(state, 4);
    let arg5 = stack_val(state, 5);
    let arg6 = stack_val(state, 6);

    if arg3 == Val::Nil {
        if arg4 == Val::Nil {
            let x_offset = num_opt(arg5).unwrap_or(0.0);
            let y_offset = num_opt(arg6).unwrap_or(0.0);
            return Ok((None, None, point, x_offset, y_offset));
        }
        if matches!(arg4, Val::Num(_)) {
            let x_offset = num_opt(arg4).unwrap_or(0.0);
            let y_offset = num_opt(arg5).unwrap_or(0.0);
            return Ok((None, None, point, x_offset, y_offset));
        }

        let relative_point = resolve_relative_point_from_val(state, arg4, point)?;
        let x_offset = num_opt(arg5).unwrap_or(0.0);
        let y_offset = num_opt(arg6).unwrap_or(0.0);
        return Ok((None, None, relative_point, x_offset, y_offset));
    }

    if let (Some(x_offset), Some(y_offset)) = (num_opt(arg3), num_opt(arg4)) {
        return Ok((None, None, point, x_offset, y_offset));
    }

    let relative_to = resolve_anchor_target_id(state, frame_id, arg3);
    // A $parent-style key that fails eager resolution is kept for lazy
    // resolution: the XML loader creates <Layers> regions before child
    // <Frames>, so the referenced sibling may not exist yet.
    let pending_key = if relative_to.is_none() {
        unresolved_anchor_key_expr(state, arg3)
    } else {
        None
    };
    if matches!(arg4, Val::Num(_)) {
        let x_offset = num_opt(arg4).unwrap_or(0.0);
        let y_offset = num_opt(arg5).unwrap_or(0.0);
        return Ok((relative_to, pending_key, point, x_offset, y_offset));
    }

    let relative_point = resolve_relative_point_from_val(state, arg4, point)?;
    let x_offset = num_opt(arg5).unwrap_or(0.0);
    let y_offset = num_opt(arg6).unwrap_or(0.0);
    Ok((relative_to, pending_key, relative_point, x_offset, y_offset))
}

fn num_opt(v: Val) -> Option<f32> {
    match v {
        Val::Num(n) => Some(n as f32),
        _ => None,
    }
}

// ── Anchor methods ────────────────────────────────────────────────────────────

/// ClearAllPoints()
pub(super) fn clear_all_points(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "ClearAllPoints");
        return Ok(0);
    }
    let already_empty = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| f.anchors.is_empty())
            .unwrap_or(true)
    };
    if !already_empty {
        let mut sim = borrow_state_mut(state)?;
        sim.widgets.remove_all_anchor_dependents_for(id);
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.clear_all_points();
        }
        sim.widgets.mark_rect_dirty(id);
    }
    Ok(0)
}

/// ClearPoint(pointName)
pub(super) fn clear_point(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let point_name = String::from_stack(state, 2)?;
    let Some(point) = crate::widget::AnchorPoint::from_str(&point_name) else {
        return Ok(0);
    };
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "ClearPoint");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    let target_id = sim
        .widgets
        .get(id)
        .and_then(|f| f.anchors.iter().find(|a| a.point == point))
        .and_then(|a| a.relative_to_id);
    if let Some(target) = target_id {
        sim.widgets.remove_anchor_dependent(target as u64, id);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.anchors.retain(|a| a.point != point);
    }
    sim.widgets.mark_rect_dirty(id);
    Ok(0)
}

/// ClearPointsOffset() — no-op stub
pub(super) fn clear_points_offset(state: &mut LuaState) -> LuaResult<u32> {
    let _id = frame_id_from_stack(state, 1)?;
    Ok(0)
}

/// AdjustPointsOffset(x, y)
pub(super) fn adjust_points_offset(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x_offset = f64::from_stack(state, 2)? as f32;
    let y_offset = f64::from_stack(state, 3)? as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        for anchor in &mut frame.anchors {
            anchor.x_offset += x_offset;
            anchor.y_offset += y_offset;
        }
    }
    sim.widgets.mark_rect_dirty(id);
    Ok(0)
}

/// GetNumPoints() -> count
pub(super) fn get_num_points(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let count = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).map(|f| f.anchors.len()).unwrap_or(0) as i32
    };
    count.into_stack(state)
}

/// GetPoint([index]) -> point, relativeTo, relativePoint, xOfs, yOfs
pub(super) fn get_point(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let index = opt_f32(state, 2).map(|n| n as i32).unwrap_or(1);
    let idx = (index - 1).max(0) as usize;
    let anchor_data = extract_anchor_by_index(state, id, idx)?;
    let Some((point, relative_to_id, relative_point, x_offset, y_offset)) = anchor_data else {
        return Ok(0);
    };
    push_anchor_values(
        state,
        point,
        relative_to_id,
        relative_point,
        x_offset,
        y_offset,
    )
}

fn extract_anchor_by_index(
    state: &mut LuaState,
    id: u64,
    idx: usize,
) -> LuaResult<
    Option<(
        crate::widget::AnchorPoint,
        Option<usize>,
        crate::widget::AnchorPoint,
        f32,
        f32,
    )>,
> {
    let sim = borrow_state(state)?;
    let Some(frame) = sim.widgets.get(id) else {
        return Ok(None);
    };
    let mut sorted: Vec<_> = frame.anchors.iter().collect();
    sorted.sort_by_key(|a| a.point.sort_key());
    let Some(anchor) = sorted.get(idx) else {
        return Ok(None);
    };
    Ok(Some((
        anchor.point,
        anchor.relative_to_id,
        anchor.relative_point,
        anchor.x_offset,
        anchor.y_offset,
    )))
}

/// GetPointByName(pointName) -> point, relativeTo, relativePoint, xOfs, yOfs
pub(super) fn get_point_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let point_name = String::from_stack(state, 2)?;
    let point_upper = point_name.to_uppercase();
    let anchor_data = {
        let sim = borrow_state(state)?;
        let Some(frame) = sim.widgets.get(id) else {
            return Ok(0);
        };
        frame
            .anchors
            .iter()
            .find(|a| a.point.as_str().to_uppercase() == point_upper)
            .map(|a| {
                (
                    a.point,
                    a.relative_to_id,
                    a.relative_point,
                    a.x_offset,
                    a.y_offset,
                )
            })
    };
    let Some((point, relative_to_id, relative_point, x_offset, y_offset)) = anchor_data else {
        return Ok(0);
    };
    push_anchor_values(
        state,
        point,
        relative_to_id,
        relative_point,
        x_offset,
        y_offset,
    )
}

fn push_anchor_values(
    state: &mut LuaState,
    point: crate::widget::AnchorPoint,
    relative_to_id: Option<usize>,
    relative_point: crate::widget::AnchorPoint,
    x_offset: f32,
    y_offset: f32,
) -> LuaResult<u32> {
    let point_str = create_string(state, point.as_str());
    state.push(point_str);
    match relative_to_id {
        Some(rid) => {
            let rel_val = frame_global_or_ref(state, rid as u64)?;
            state.push(rel_val);
        }
        None => state.push(Val::Nil),
    }
    let rel_point_str = create_string(state, relative_point.as_str());
    state.push(rel_point_str);
    state.push(Val::Num(x_offset as f64));
    state.push(Val::Num(y_offset as f64));
    Ok(5)
}

#[derive(Clone)]
struct SetPointRequest {
    point: crate::widget::AnchorPoint,
    relative_to: Option<usize>,
    /// Unresolved `$parent`-style key expression, stored on the anchor for
    /// lazy resolution via `resolve_named_anchor_targets_for_frame`.
    pending_key: Option<String>,
    relative_point: crate::widget::AnchorPoint,
    x_offset: f32,
    y_offset: f32,
}

/// SetPoint(point [, relativeTo [, relativePoint]] [, xOfs, yOfs])
pub(super) fn set_point(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let point = set_point_anchor_from_stack(state)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetPoint");
        return Ok(0);
    }

    let request = set_point_request(state, id, point)?;
    if is_set_point_unchanged(state, id, &request)? {
        return Ok(0);
    }

    #[cfg(feature = "forbidden-aspects")]
    if let Some(relative_to) = request.relative_to {
        forbidden_aspects::ensure_forbidden_aspects_already_owned(
            state,
            id,
            relative_to as u64,
            forbidden_aspects::INHERITANCE_LAYOUT,
            "SetPoint",
        )?;
    }

    ensure_no_anchor_cycle(state, id, request.relative_to, "SetPoint")?;

    let mut sim = borrow_state_mut(state)?;
    apply_set_point(&mut sim, id, request)
}

fn set_point_anchor_from_stack(state: &mut LuaState) -> LuaResult<crate::widget::AnchorPoint> {
    let point_name = String::from_stack(state, 2)?;
    let normalized = normalize_anchor_point_name(&point_name);
    let Some(point) = parse_anchor_point_with_compat_warning(state, normalized) else {
        return Err(runtime_error(format!(
            "Frame:SetPoint(): Invalid region point {point_name}"
        )));
    };
    Ok(point)
}

fn normalize_anchor_point_name(point_name: &str) -> &str {
    point_name
        .split(['"', ',', ' '])
        .next()
        .unwrap_or(point_name)
}

fn set_point_request(
    state: &mut LuaState,
    id: u64,
    point: crate::widget::AnchorPoint,
) -> LuaResult<SetPointRequest> {
    let (relative_to, pending_key, relative_point, x_offset, y_offset) =
        parse_set_point_args(state, id, point)?;
    // Pending keys keep relative_to_id unset: layout falls back to the parent
    // until the finalize pass resolves the key against the parent table.
    let resolved_relative_to = if pending_key.is_some() {
        None
    } else {
        match relative_to {
            Some(relative_to) => Some(relative_to),
            None => set_point_parent_id(state, id)?,
        }
    };
    Ok(SetPointRequest {
        point,
        relative_to: resolved_relative_to,
        pending_key,
        relative_point,
        x_offset,
        y_offset,
    })
}

fn set_point_parent_id(state: &mut LuaState, id: u64) -> LuaResult<Option<usize>> {
    let sim = borrow_state(state)?;
    let parent_id = sim
        .widgets
        .get(id)
        .and_then(|frame| frame.parent_id)
        .map(|parent_id| parent_id as usize);
    Ok(parent_id)
}

fn is_set_point_unchanged(
    state: &mut LuaState,
    id: u64,
    request: &SetPointRequest,
) -> LuaResult<bool> {
    if request.pending_key.is_some() {
        return Ok(false);
    }
    let sim = borrow_state(state)?;
    let unchanged = sim
        .widgets
        .get(id)
        .and_then(|frame| {
            frame
                .anchors
                .iter()
                .find(|anchor| anchor.point == request.point)
        })
        .map(|anchor| {
            anchor.relative_to_id == request.relative_to
                && anchor.relative_point == request.relative_point
                && anchor.x_offset == request.x_offset
                && anchor.y_offset == request.y_offset
        })
        .unwrap_or(false);
    Ok(unchanged)
}

fn ensure_no_anchor_cycle(
    state: &mut LuaState,
    frame_id: u64,
    relative_to: Option<usize>,
    method_name: &str,
) -> LuaResult<()> {
    let Some(rel_id) = relative_to else {
        return Ok(());
    };
    let cycle = {
        let sim = borrow_state(state)?;
        sim.widgets.describe_anchor_cycle(frame_id, rel_id as u64)
    };
    if let Some(cycle) = cycle {
        let message = format_anchor_cycle_error(state, method_name, frame_id, &cycle)?;
        return Err(runtime_error(message));
    }
    Ok(())
}

fn apply_set_point(
    sim: &mut crate::lua_api::SimState,
    id: u64,
    request: SetPointRequest,
) -> LuaResult<u32> {
    if let Some(old_target) = sim.widgets.get(id).and_then(|f| {
        f.anchors
            .iter()
            .find(|a| a.point == request.point)
            .and_then(|a| a.relative_to_id)
    }) {
        sim.widgets.remove_anchor_dependent(old_target as u64, id);
    }
    if let Some(rel_id) = request.relative_to {
        sim.widgets.add_anchor_dependent(rel_id as u64, id);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        match request.pending_key {
            Some(pending_key) => frame.set_point_with_name(
                request.point,
                Some(pending_key),
                request.relative_point,
                request.x_offset,
                request.y_offset,
            ),
            None => frame.set_point(
                request.point,
                request.relative_to,
                request.relative_point,
                request.x_offset,
                request.y_offset,
            ),
        }
    }
    sim.widgets.mark_rect_dirty(id);
    Ok(0)
}

/// SetAllPoints([relativeTo])
pub(super) fn set_all_points(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let arg = stack_val(state, 2);
    if arg == Val::Bool(false) {
        return Ok(0);
    }
    let has_target_arg = state.top.saturating_sub(state.base) >= 2;
    let relative_to_id = resolve_set_all_points_target(state, id, arg, has_target_arg)?;
    ensure_no_anchor_cycle(state, id, relative_to_id, "SetAllPoints")?;
    let mut sim = borrow_state_mut(state)?;
    sim.widgets.remove_all_anchor_dependents_for(id);
    if let Some(rel_id) = relative_to_id {
        sim.widgets.add_anchor_dependent(rel_id as u64, id);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        anchor_frame_to_all_corners(frame, relative_to_id);
    }
    sim.widgets.mark_rect_dirty(id);
    Ok(0)
}

fn resolve_set_all_points_target(
    state: &mut LuaState,
    id: u64,
    arg: Val,
    has_target_arg: bool,
) -> LuaResult<Option<usize>> {
    use crate::lua_api::methods::extract_frame_id;
    if has_target_arg && matches!(arg, Val::Nil) {
        return Ok(None);
    }
    if let Some(rid) = extract_frame_id(state, arg) {
        return Ok(Some(rid as usize));
    }
    let sim = borrow_state(state)?;
    Ok(sim
        .widgets
        .get(id)
        .and_then(|f| f.parent_id)
        .map(|p| p as usize))
}

fn anchor_frame_to_all_corners(frame: &mut crate::widget::Frame, relative_to_id: Option<usize>) {
    frame.clear_all_points();
    for corner in [
        crate::widget::AnchorPoint::TopLeft,
        crate::widget::AnchorPoint::BottomRight,
    ] {
        frame.set_point(corner, relative_to_id, corner, 0.0, 0.0);
    }
}

fn format_anchor_cycle_error(
    state: &mut LuaState,
    method_name: &str,
    frame_id: u64,
    cycle: &crate::widget::AnchorCyclePath,
) -> LuaResult<String> {
    if cycle.relative_to_id == frame_id {
        return Ok(format!(
            "Action[SetPoint] failed because[Cannot anchor to itself]: attempted from: Frame:{method_name}."
        ));
    }

    let relative = frame_cycle_hex_id(state, cycle.relative_to_id)?;
    let dependent = frame_cycle_hex_id(state, cycle.dependent_id)?;
    let mut message = format!(
        "Action[SetPoint] failed because[Cannot anchor to a region dependent on it]: attempted from: Frame:{method_name}.\nRelative: [{relative}]\nDependent: [{dependent}]",
    );

    if !cycle.dependent_ancestors.is_empty() {
        message.push_str("\nDependent ancestors:");
        for ancestor_id in &cycle.dependent_ancestors {
            message.push_str("\n[");
            message.push_str(&frame_cycle_hex_id(state, *ancestor_id)?);
            message.push(']');
        }
    }

    Ok(message)
}

fn frame_cycle_hex_id(state: &mut LuaState, frame_id: u64) -> LuaResult<String> {
    let tostring_key = state.gc.intern_string(b"tostring");
    let tostring_fn = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(tostring_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    let frame = frame_global_or_ref(state, frame_id)?;
    let rendered = call_function_state(state, tostring_fn, &[frame])?;
    let text = val_to_string(state, rendered).unwrap_or_default();
    Ok(text
        .rsplit("0x")
        .next()
        .unwrap_or(text.as_str())
        .to_string())
}
