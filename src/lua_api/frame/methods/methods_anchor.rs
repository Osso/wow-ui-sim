//! Anchor/point methods: SetPoint, ClearAllPoints, SetAllPoints, GetPoint, etc.

use super::FrameHandle;
use mlua::{UserDataMethods, Value};

/// Add anchor/point methods to FrameHandle UserData.
pub fn add_anchor_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetPoint(point, relativeTo, relativePoint, xOfs, yOfs)
    methods.add_method("SetPoint", |_, this, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();

        let point_str = args
            .first()
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "CENTER".to_string());

        let point = crate::widget::AnchorPoint::from_str(&point_str).unwrap_or_default();

        // Helper to extract numeric value from Value (handles both Number and Integer)
        fn get_number(v: &Value) -> Option<f32> {
            match v {
                Value::Number(n) => Some(*n as f32),
                Value::Integer(n) => Some(*n as f32),
                _ => None,
            }
        }

        // Helper to extract frame ID from Value
        let get_frame_id = |v: &Value| -> Option<usize> {
            if let Value::UserData(ud) = v {
                if let Ok(frame_handle) = ud.borrow::<FrameHandle>() {
                    return Some(frame_handle.id as usize);
                }
            }
            None
        };

        // Parse the variable arguments
        let (relative_to, relative_point, x_ofs, y_ofs) = match args.len() {
            1 => (None, point, 0.0, 0.0),
            2 | 3 => {
                // SetPoint("CENTER", x, y) or SetPoint("CENTER", relativeTo)
                let x = args.get(1).and_then(get_number);
                let y = args.get(2).and_then(get_number);
                if let (Some(x), Some(y)) = (x, y) {
                    (None, point, x, y)
                } else {
                    // Could be SetPoint("CENTER", relativeTo) - get frame ID
                    let rel_to = args.get(1).and_then(get_frame_id);
                    (rel_to, point, 0.0, 0.0)
                }
            }
            _ => {
                // Full form: SetPoint(point, relativeTo, relativePoint, x, y)
                let rel_to = args.get(1).and_then(get_frame_id);

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
        };

        let mut state = this.state.borrow_mut();

        // Check for anchor cycles before setting point
        if let Some(rel_id) = relative_to {
            if state.widgets.would_create_anchor_cycle(this.id, rel_id as u64) {
                // Silently ignore the anchor to prevent cycles (matches WoW behavior)
                // WoW logs an error but doesn't crash
                return Ok(());
            }
        }

        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.set_point(point, relative_to, relative_point, x_ofs, y_ofs);
        }
        Ok(())
    });

    // ClearAllPoints()
    methods.add_method("ClearAllPoints", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.clear_all_points();
        }
        Ok(())
    });

    // AdjustPointsOffset(x, y) - Adjusts the offsets of all anchor points
    methods.add_method(
        "AdjustPointsOffset",
        |_, this, (x_offset, y_offset): (f32, f32)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                for anchor in &mut frame.anchors {
                    anchor.x_offset += x_offset;
                    anchor.y_offset += y_offset;
                }
            }
            Ok(())
        },
    );

    // SetAllPoints(relativeTo)
    // SetAllPoints accepts: nil, frame, or boolean (true = parent, false = no-op)
    // Sets TOPLEFT→TOPLEFT and BOTTOMRIGHT→BOTTOMRIGHT to the relative frame
    methods.add_method("SetAllPoints", |_, this, arg: Option<Value>| {
        // Handle boolean case: true means use parent, false is a no-op
        let (should_set, relative_to_id) = match &arg {
            Some(Value::Boolean(false)) => (false, None),
            Some(Value::UserData(ud)) => {
                // Extract frame ID from userdata
                if let Ok(handle) = ud.borrow::<FrameHandle>() {
                    (true, Some(handle.id as usize))
                } else {
                    (true, None)
                }
            }
            _ => (true, None), // nil, true => use parent (None)
        };

        if should_set {
            let mut state = this.state.borrow_mut();

            // Check for anchor cycles before setting points
            if let Some(rel_id) = relative_to_id {
                if state.widgets.would_create_anchor_cycle(this.id, rel_id as u64) {
                    return Ok(());
                }
            }

            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.clear_all_points();
                // SetAllPoints makes the frame fill its relative frame
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
        }
        Ok(())
    });

    // GetPoint(index) -> point, relativeTo, relativePoint, xOfs, yOfs
    methods.add_method("GetPoint", |lua, this, index: Option<i32>| {
        let idx = index.unwrap_or(1) - 1; // Lua is 1-indexed
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            if let Some(anchor) = frame.anchors.get(idx as usize) {
                // Get the relative frame reference if we have an ID
                let relative_to: Value = if let Some(rel_id) = anchor.relative_to_id {
                    // Look up the frame reference from globals
                    let frame_ref_key = format!("__frame_{}", rel_id);
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
        }
        Ok(mlua::MultiValue::new())
    });

    // GetNumPoints()
    methods.add_method("GetNumPoints", |_, this, ()| {
        let state = this.state.borrow();
        let count = state
            .widgets
            .get(this.id)
            .map(|f| f.anchors.len())
            .unwrap_or(0);
        Ok(count as i32)
    });

    // GetPointByName(pointName) -> point, relativeTo, relativePoint, xOfs, yOfs
    // Finds an anchor by its point name (e.g., "TOPLEFT", "CENTER")
    methods.add_method("GetPointByName", |lua, this, point_name: String| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            let point_upper = point_name.to_uppercase();
            for anchor in &frame.anchors {
                if anchor.point.as_str().to_uppercase() == point_upper {
                    return Ok(mlua::MultiValue::from_vec(vec![
                        Value::String(lua.create_string(anchor.point.as_str())?),
                        Value::Nil, // relativeTo (would need to return frame reference)
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
