//! Anchor/point methods: SetPoint, ClearAllPoints, SetAllPoints, GetPoint, etc.

use super::FrameHandle;
use mlua::{UserDataMethods, Value};

/// Add anchor/point methods to FrameHandle UserData.
pub fn add_anchor_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_set_point_method(methods);
    add_clear_and_adjust_methods(methods);
    add_set_all_points_method(methods);
    add_get_point_methods(methods);
}

/// Helper to extract numeric value from Value (handles both Number and Integer).
fn get_number(v: &Value) -> Option<f32> {
    match v {
        Value::Number(n) => Some(*n as f32),
        Value::Integer(n) => Some(*n as f32),
        _ => None,
    }
}

/// Helper to extract frame ID from Value.
///
/// Handles both UserData (direct frame reference) and String (global name
/// lookup via `_G`), matching WoW's SetPoint behavior where string frame
/// names are resolved to the corresponding frame object.
fn get_frame_id(lua: &mlua::Lua, v: &Value) -> Option<usize> {
    match v {
        Value::UserData(ud) => {
            if let Ok(frame_handle) = ud.borrow::<FrameHandle>() {
                return Some(frame_handle.id as usize);
            }
            None
        }
        Value::String(s) => {
            let name = s.to_string_lossy();
            if let Ok(val) = lua.globals().get::<Value>(name.as_str()) {
                if let Value::UserData(ud) = val {
                    if let Ok(fh) = ud.borrow::<FrameHandle>() {
                        return Some(fh.id as usize);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Extract an anchor point string from a Value, defaulting to "CENTER".
fn extract_point_str(v: Option<&Value>) -> String {
    v.and_then(|v| {
        if let Value::String(s) = v {
            Some(s.to_string_lossy().to_string())
        } else {
            None
        }
    })
    .unwrap_or_else(|| "CENTER".to_string())
}

/// Parse variable SetPoint arguments into (relative_to, relative_point, x_ofs, y_ofs).
fn parse_set_point_args(
    lua: &mlua::Lua,
    args: &[Value],
    point: crate::widget::AnchorPoint,
) -> (Option<usize>, crate::widget::AnchorPoint, f32, f32) {
    match args.len() {
        1 => (None, point, 0.0, 0.0),
        2 | 3 => {
            let x = args.get(1).and_then(get_number);
            let y = args.get(2).and_then(get_number);
            if let (Some(x), Some(y)) = (x, y) {
                // SetPoint("point", x, y)
                (None, point, x, y)
            } else {
                let rel_to = args.get(1).map(|v| get_frame_id(lua, v)).flatten();
                // Check if 3rd arg is a relativePoint string:
                // SetPoint("point", relativeTo, "relativePoint")
                let rel_point = args.get(2).and_then(|v| {
                    if let Value::String(s) = v {
                        crate::widget::AnchorPoint::from_str(&s.to_string_lossy())
                    } else {
                        None
                    }
                }).unwrap_or(point);
                (rel_to, rel_point, 0.0, 0.0)
            }
        }
        _ => {
            let rel_to = args.get(1).map(|v| get_frame_id(lua, v)).flatten();
            let rel_point_str = args.get(2).and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            });
            let rel_point = rel_point_str
                .and_then(|s| crate::widget::AnchorPoint::from_str(&s))
                .unwrap_or(point);
            let x = args.get(3).and_then(get_number).unwrap_or(0.0);
            let y = args.get(4).and_then(get_number).unwrap_or(0.0);
            (rel_to, rel_point, x, y)
        }
    }
}

/// SetPoint(point, relativeTo, relativePoint, xOfs, yOfs)
fn add_set_point_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetPoint", |lua, this, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();
        let point_str = extract_point_str(args.first());
        let point = crate::widget::AnchorPoint::from_str(&point_str).unwrap_or_default();
        let (relative_to, relative_point, x_ofs, y_ofs) = parse_set_point_args(lua, &args, point);

        let state = this.state.borrow();

        // Check for anchor cycles before setting point
        if let Some(rel_id) = relative_to
            && state.widgets.would_create_anchor_cycle(this.id, rel_id as u64) {
                return Ok(());
            }

        // Skip get_mut if the anchor already matches
        if let Some(frame) = state.widgets.get(this.id)
            && let Some(existing) = frame.anchors.iter().find(|a| a.point == point)
                && existing.relative_to_id == relative_to
                    && existing.relative_point == relative_point
                    && existing.x_offset == x_ofs
                    && existing.y_offset == y_ofs
                {
                    return Ok(());
                }
        drop(state);

        let mut state = this.state.borrow_mut();
        // Update reverse anchor index: remove old target, add new target
        if let Some(frame) = state.widgets.get(this.id) {
            if let Some(old) = frame.anchors.iter().find(|a| a.point == point) {
                if let Some(old_target) = old.relative_to_id {
                    state.widgets.remove_anchor_dependent(old_target as u64, this.id);
                }
            }
        }
        if let Some(rel_id) = relative_to {
            state.widgets.add_anchor_dependent(rel_id as u64, this.id);
        }
        if let Some(frame) = state.widgets.get_mut_visual(this.id) {
            frame.set_point(point, relative_to, relative_point, x_ofs, y_ofs);
        }
        state.widgets.mark_rect_dirty_subtree(this.id);
        state.invalidate_layout_with_dependents(this.id);
        Ok(())
    });
}

/// ClearAllPoints(), ClearPoint(point), ClearPointsOffset(), AdjustPointsOffset(x, y)
fn add_clear_and_adjust_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("ClearAllPoints", |_, this, ()| {
        let already_empty = this.state.borrow().widgets.get(this.id)
            .map(|f| f.anchors.is_empty()).unwrap_or(true);
        if !already_empty {
            let mut state = this.state.borrow_mut();
            state.widgets.remove_all_anchor_dependents_for(this.id);
            if let Some(frame) = state.widgets.get_mut_visual(this.id) {
                frame.clear_all_points();
            }
            state.widgets.mark_rect_dirty_subtree(this.id);
            state.invalidate_layout(this.id);
        }
        Ok(())
    });

    // ClearPoint(point) - remove a specific anchor by point name
    methods.add_method("ClearPoint", |_, this, point_name: String| {
        let point = crate::widget::AnchorPoint::from_str(&point_name);
        if let Some(point) = point {
            let mut state = this.state.borrow_mut();
            // Remove reverse index entry for the cleared anchor
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(anchor) = frame.anchors.iter().find(|a| a.point == point) {
                    if let Some(target) = anchor.relative_to_id {
                        state.widgets.remove_anchor_dependent(target as u64, this.id);
                    }
                }
            }
            if let Some(frame) = state.widgets.get_mut_visual(this.id) {
                frame.anchors.retain(|a| a.point != point);
            }
            state.widgets.mark_rect_dirty_subtree(this.id);
            state.invalidate_layout(this.id);
        }
        Ok(())
    });

    // ClearPointsOffset() - stub
    methods.add_method("ClearPointsOffset", |_, _this, ()| Ok(()));

    methods.add_method(
        "AdjustPointsOffset",
        |_, this, (x_offset, y_offset): (f32, f32)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(this.id) {
                for anchor in &mut frame.anchors {
                    anchor.x_offset += x_offset;
                    anchor.y_offset += y_offset;
                }
            }
            state.widgets.mark_rect_dirty_subtree(this.id);
            state.invalidate_layout(this.id);
            Ok(())
        },
    );
}

/// SetAllPoints(relativeTo) - sets TOPLEFT and BOTTOMRIGHT to fill a relative frame.
fn add_set_all_points_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetAllPoints", |_, this, arg: Option<Value>| {
        let (should_set, relative_to_id) = match &arg {
            Some(Value::Boolean(false)) => (false, None),
            Some(Value::UserData(ud)) => {
                if let Ok(handle) = ud.borrow::<FrameHandle>() {
                    (true, Some(handle.id as usize))
                } else {
                    (true, None)
                }
            }
            _ => (true, None),
        };

        if should_set {
            let mut state = this.state.borrow_mut();

            if let Some(rel_id) = relative_to_id
                && state.widgets.would_create_anchor_cycle(this.id, rel_id as u64) {
                    return Ok(());
                }

            // Update reverse index: remove old, add new
            state.widgets.remove_all_anchor_dependents_for(this.id);
            if let Some(rel_id) = relative_to_id {
                state.widgets.add_anchor_dependent(rel_id as u64, this.id);
            }

            if let Some(frame) = state.widgets.get_mut_visual(this.id) {
                frame.clear_all_points();
                frame.set_point(
                    crate::widget::AnchorPoint::TopLeft,
                    relative_to_id,
                    crate::widget::AnchorPoint::TopLeft,
                    0.0,
                    0.0,
                );
                frame.set_point(
                    crate::widget::AnchorPoint::BottomRight,
                    relative_to_id,
                    crate::widget::AnchorPoint::BottomRight,
                    0.0,
                    0.0,
                );
            }
            state.widgets.mark_rect_dirty_subtree(this.id);
            state.invalidate_layout(this.id);
        }
        Ok(())
    });
}

/// GetPoint, GetNumPoints, GetPointByName - querying anchor points.
fn add_get_point_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetPoint", |lua, this, index: Option<i32>| {
        let idx = index.unwrap_or(1) - 1;
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id)
            && let Some(anchor) = frame.anchors.get(idx as usize) {
                let rel_id = anchor.relative_to_id.or(frame.parent_id.map(|p| p as usize));
                let relative_to: Value = if let Some(id) = rel_id {
                    let frame_ref_key = format!("__frame_{}", id);
                    lua.globals()
                        .get(frame_ref_key.as_str())
                        .unwrap_or(Value::Nil)
                } else {
                    Value::Nil
                };
                return Ok(mlua::MultiValue::from_vec(vec![
                    Value::String(lua.create_string(anchor.point.as_str())?),
                    relative_to,
                    Value::String(lua.create_string(anchor.relative_point.as_str())?),
                    Value::Number(anchor.x_offset as f64),
                    Value::Number(anchor.y_offset as f64),
                ]));
            }
        Ok(mlua::MultiValue::new())
    });

    methods.add_method("GetNumPoints", |_, this, ()| {
        let state = this.state.borrow();
        let count = state
            .widgets
            .get(this.id)
            .map(|f| f.anchors.len())
            .unwrap_or(0);
        Ok(count as i32)
    });

    methods.add_method("GetPointByName", |lua, this, point_name: String| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            let point_upper = point_name.to_uppercase();
            for anchor in &frame.anchors {
                if anchor.point.as_str().to_uppercase() == point_upper {
                    return Ok(mlua::MultiValue::from_vec(vec![
                        Value::String(lua.create_string(anchor.point.as_str())?),
                        Value::Nil,
                        Value::String(lua.create_string(anchor.relative_point.as_str())?),
                        Value::Number(anchor.x_offset as f64),
                        Value::Number(anchor.y_offset as f64),
                    ]));
                }
            }
        }
        Ok(mlua::MultiValue::new())
    });
}
