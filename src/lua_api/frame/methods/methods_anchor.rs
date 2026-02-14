//! Anchor/point methods: SetPoint, ClearAllPoints, SetAllPoints, GetPoint, etc.

use crate::lua_api::frame::handle::{extract_frame_id, frame_lud, get_sim_state, lud_to_id};
use mlua::{LightUserData, Lua, Value};

/// Add anchor/point methods to the frame methods table.
pub fn add_anchor_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_set_point_method(lua, methods)?;
    add_clear_and_adjust_methods(lua, methods)?;
    add_set_all_points_method(lua, methods)?;
    add_get_point_methods(lua, methods)?;
    Ok(())
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
/// Handles both LightUserData (direct frame reference) and String (global name
/// lookup via `_G`), matching WoW's SetPoint behavior where string frame
/// names are resolved to the corresponding frame object.
fn get_frame_id(lua: &Lua, v: &Value) -> Option<usize> {
    match v {
        Value::LightUserData(lud) => Some(lud_to_id(*lud) as usize),
        Value::String(s) => {
            let name = s.to_string_lossy();
            if let Ok(val) = lua.globals().get::<Value>(name.as_str()) {
                return extract_frame_id(&val).map(|id| id as usize);
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
    lua: &Lua,
    args: &[Value],
    point: crate::widget::AnchorPoint,
) -> (Option<usize>, crate::widget::AnchorPoint, f32, f32) {
    match args.len() {
        1 => (None, point, 0.0, 0.0),
        2 | 3 => parse_set_point_2_or_3(lua, args, point),
        _ => parse_set_point_full(lua, args, point),
    }
}

/// Parse SetPoint with 2 or 3 arguments (after the point name).
fn parse_set_point_2_or_3(
    lua: &Lua,
    args: &[Value],
    point: crate::widget::AnchorPoint,
) -> (Option<usize>, crate::widget::AnchorPoint, f32, f32) {
    let x = args.get(1).and_then(get_number);
    let y = args.get(2).and_then(get_number);
    if let (Some(x), Some(y)) = (x, y) {
        // SetPoint("point", x, y)
        (None, point, x, y)
    } else {
        let rel_to = args.get(1).and_then(|v| get_frame_id(lua, v));
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

/// Parse SetPoint with 4+ arguments (full form with relativeTo, relativePoint, x, y).
fn parse_set_point_full(
    lua: &Lua,
    args: &[Value],
    point: crate::widget::AnchorPoint,
) -> (Option<usize>, crate::widget::AnchorPoint, f32, f32) {
    let rel_to = args.get(1).and_then(|v| get_frame_id(lua, v));
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

/// SetPoint(point, relativeTo, relativePoint, xOfs, yOfs)
fn add_set_point_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetPoint", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args: Vec<Value> = args.into_iter().collect();
        let point_str = extract_point_str(args.first());
        let point = crate::widget::AnchorPoint::from_str(&point_str).unwrap_or_default();
        let (relative_to, relative_point, x_ofs, y_ofs) = parse_set_point_args(lua, &args, point);

        let state_rc = get_sim_state(lua);
        if !should_update_point(&state_rc.borrow(), id, relative_to, point, relative_point, x_ofs, y_ofs) {
            return Ok(());
        }

        apply_set_point(&state_rc, id, point, relative_to, relative_point, x_ofs, y_ofs);
        Ok(())
    })?)?;
    Ok(())
}

/// Check if the point needs updating (cycle check + duplicate check).
fn should_update_point(
    state: &crate::lua_api::SimState,
    id: u64,
    relative_to: Option<usize>,
    point: crate::widget::AnchorPoint,
    relative_point: crate::widget::AnchorPoint,
    x_ofs: f32,
    y_ofs: f32,
) -> bool {
    // Check for anchor cycles before setting point
    if let Some(rel_id) = relative_to
        && state.widgets.would_create_anchor_cycle(id, rel_id as u64)
    {
        return false;
    }

    // Skip if the anchor already matches
    if let Some(frame) = state.widgets.get(id)
        && let Some(existing) = frame.anchors.iter().find(|a| a.point == point)
        && existing.relative_to_id == relative_to
        && existing.relative_point == relative_point
        && existing.x_offset == x_ofs
        && existing.y_offset == y_ofs
    {
        return false;
    }

    true
}

/// Apply the SetPoint mutation to the widget state.
fn apply_set_point(
    state_rc: &std::cell::RefCell<crate::lua_api::SimState>,
    id: u64,
    point: crate::widget::AnchorPoint,
    relative_to: Option<usize>,
    relative_point: crate::widget::AnchorPoint,
    x_ofs: f32,
    y_ofs: f32,
) {
    let mut state = state_rc.borrow_mut();
    // Update reverse anchor index: remove old target, add new target
    if let Some(frame) = state.widgets.get(id) {
        if let Some(old) = frame.anchors.iter().find(|a| a.point == point) {
            if let Some(old_target) = old.relative_to_id {
                state.widgets.remove_anchor_dependent(old_target as u64, id);
            }
        }
    }
    if let Some(rel_id) = relative_to {
        state.widgets.add_anchor_dependent(rel_id as u64, id);
    }
    if let Some(frame) = state.widgets.get_mut_visual(id) {
        frame.set_point(point, relative_to, relative_point, x_ofs, y_ofs);
    }
    state.widgets.mark_rect_dirty(id);
    state.invalidate_layout_with_dependents(id);
}

/// ClearAllPoints(), ClearPoint(point), ClearPointsOffset(), AdjustPointsOffset(x, y)
fn add_clear_and_adjust_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("ClearAllPoints", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let already_empty = state_rc.borrow().widgets.get(id)
            .map(|f| f.anchors.is_empty()).unwrap_or(true);
        if !already_empty {
            let mut state = state_rc.borrow_mut();
            state.widgets.remove_all_anchor_dependents_for(id);
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.clear_all_points();
            }
            state.widgets.mark_rect_dirty(id);
            state.invalidate_layout(id);
        }
        Ok(())
    })?)?;

    // ClearPoint(point) - remove a specific anchor by point name
    methods.set("ClearPoint", lua.create_function(|lua, (ud, point_name): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let point = crate::widget::AnchorPoint::from_str(&point_name);
        if let Some(point) = point {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            // Remove reverse index entry for the cleared anchor
            if let Some(frame) = state.widgets.get(id) {
                if let Some(anchor) = frame.anchors.iter().find(|a| a.point == point) {
                    if let Some(target) = anchor.relative_to_id {
                        state.widgets.remove_anchor_dependent(target as u64, id);
                    }
                }
            }
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.anchors.retain(|a| a.point != point);
            }
            state.widgets.mark_rect_dirty(id);
            state.invalidate_layout(id);
        }
        Ok(())
    })?)?;

    // ClearPointsOffset() - stub
    methods.set("ClearPointsOffset", lua.create_function(
        |_lua, _ud: LightUserData| Ok(()),
    )?)?;

    methods.set("AdjustPointsOffset", lua.create_function(
        |lua, (ud, x_offset, y_offset): (LightUserData, f32, f32)| {
            let id = lud_to_id(ud);
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                for anchor in &mut frame.anchors {
                    anchor.x_offset += x_offset;
                    anchor.y_offset += y_offset;
                }
            }
            state.widgets.mark_rect_dirty(id);
            state.invalidate_layout(id);
            Ok(())
        },
    )?)?;

    Ok(())
}

/// SetAllPoints(relativeTo) - sets TOPLEFT and BOTTOMRIGHT to fill a relative frame.
fn add_set_all_points_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetAllPoints", lua.create_function(|lua, (ud, arg): (LightUserData, Option<Value>)| {
        let id = lud_to_id(ud);
        let (should_set, relative_to_id) = match &arg {
            Some(Value::Boolean(false)) => (false, None),
            Some(Value::LightUserData(lud)) => (true, Some(lud_to_id(*lud) as usize)),
            _ => (true, None),
        };

        if should_set {
            let state_rc = get_sim_state(lua);
            apply_set_all_points(&state_rc, id, relative_to_id);
        }
        Ok(())
    })?)?;
    Ok(())
}

/// Apply SetAllPoints mutation: clear anchors and set TOPLEFT + BOTTOMRIGHT.
fn apply_set_all_points(
    state_rc: &std::cell::RefCell<crate::lua_api::SimState>,
    id: u64,
    relative_to_id: Option<usize>,
) {
    let mut state = state_rc.borrow_mut();

    if let Some(rel_id) = relative_to_id
        && state.widgets.would_create_anchor_cycle(id, rel_id as u64)
    {
        return;
    }

    // Update reverse index: remove old, add new
    state.widgets.remove_all_anchor_dependents_for(id);
    if let Some(rel_id) = relative_to_id {
        state.widgets.add_anchor_dependent(rel_id as u64, id);
    }

    if let Some(frame) = state.widgets.get_mut_visual(id) {
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
    state.widgets.mark_rect_dirty(id);
    state.invalidate_layout(id);
}

/// GetPoint, GetNumPoints, GetPointByName - querying anchor points.
fn add_get_point_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_get_point(lua, methods)?;
    add_get_num_points(lua, methods)?;
    add_get_point_by_name(lua, methods)?;
    Ok(())
}

/// GetPoint(index) - return anchor details at the given 1-based index.
fn add_get_point(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetPoint", lua.create_function(|lua, (ud, index): (LightUserData, Option<i32>)| {
        let id = lud_to_id(ud);
        let idx = index.unwrap_or(1) - 1;
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id)
            && let Some(anchor) = frame.anchors.get(idx as usize)
        {
            let rel_id = anchor.relative_to_id.or(frame.parent_id.map(|p| p as usize));
            let relative_to: Value = if let Some(rid) = rel_id {
                frame_lud(rid as u64)
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
    })?)?;
    Ok(())
}

/// GetNumPoints() - return the number of anchors on this frame.
fn add_get_num_points(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetNumPoints", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let count = state
            .widgets
            .get(id)
            .map(|f| f.anchors.len())
            .unwrap_or(0);
        Ok(count as i32)
    })?)?;
    Ok(())
}

/// GetPointByName(pointName) - return anchor details by point name string.
fn add_get_point_by_name(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetPointByName", lua.create_function(
        |lua, (ud, point_name): (LightUserData, String)| {
            let id = lud_to_id(ud);
            let state_rc = get_sim_state(lua);
            let state = state_rc.borrow();
            if let Some(frame) = state.widgets.get(id) {
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
        },
    )?)?;
    Ok(())
}
