//! Line-specific methods: SetStartPoint, SetEndPoint, SetThickness, and getters.

use crate::lua_api::frame::handle::{extract_frame_id, frame_lud, get_sim_state, lud_to_id};
use crate::widget::{AnchorPoint, LineAnchor};
use mlua::{LightUserData, Lua, Value};

pub fn add_line_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetStartPoint", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        set_line_point(lua, lud_to_id(ud), args, true)
    })?)?;

    methods.set("SetEndPoint", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        set_line_point(lua, lud_to_id(ud), args, false)
    })?)?;

    methods.set("SetThickness", lua.create_function(|lua, (ud, thickness): (LightUserData, f32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(f) = state.widgets.get_mut_visual(id) {
            f.line_thickness = thickness;
        }
        Ok(())
    })?)?;

    methods.set("GetStartPoint", lua.create_function(|lua, ud: LightUserData| {
        get_line_point(lua, lud_to_id(ud), true)
    })?)?;

    methods.set("GetEndPoint", lua.create_function(|lua, ud: LightUserData| {
        get_line_point(lua, lud_to_id(ud), false)
    })?)?;

    methods.set("GetThickness", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let thickness = state.widgets.get(id).map_or(1.0, |f| f.line_thickness);
        Ok(thickness)
    })?)?;

    Ok(())
}

fn set_line_point(lua: &Lua, id: u64, args: mlua::MultiValue, is_start: bool) -> mlua::Result<()> {
    let args: Vec<Value> = args.into_iter().collect();

    let point_str = match args.first() {
        Some(Value::String(s)) => s.to_string_lossy().to_string(),
        _ => return Ok(()),
    };
    let point = AnchorPoint::from_str(&point_str).unwrap_or(AnchorPoint::Center);

    let target_id = args.get(1).and_then(extract_frame_id);

    let x_offset = match args.get(2) {
        Some(Value::Number(n)) => *n as f32,
        Some(Value::Integer(n)) => *n as f32,
        _ => 0.0,
    };
    let y_offset = match args.get(3) {
        Some(Value::Number(n)) => *n as f32,
        Some(Value::Integer(n)) => *n as f32,
        _ => 0.0,
    };

    let anchor = LineAnchor {
        point,
        target_id,
        x_offset,
        y_offset,
    };

    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    if let Some(f) = state.widgets.get_mut_visual(id) {
        if is_start {
            f.line_start = Some(anchor);
        } else {
            f.line_end = Some(anchor);
        }
    }
    Ok(())
}

fn get_line_point(lua: &Lua, id: u64, is_start: bool) -> mlua::Result<mlua::MultiValue> {
    let state_rc = get_sim_state(lua);
    let state = state_rc.borrow();
    let anchor = state.widgets.get(id).and_then(|f| {
        if is_start { f.line_start.as_ref() } else { f.line_end.as_ref() }
    });

    let Some(anchor) = anchor else {
        return Ok(mlua::MultiValue::new());
    };

    let point_str = lua.create_string(anchor.point.as_str())?;
    let target: Value = if let Some(tid) = anchor.target_id {
        frame_lud(tid)
    } else {
        Value::Nil
    };

    Ok(mlua::MultiValue::from_iter([
        Value::String(point_str),
        target,
        Value::Number(anchor.x_offset as f64),
        Value::Number(anchor.y_offset as f64),
    ]))
}
