//! Line-specific methods: SetStartPoint, SetEndPoint, SetThickness, and getters.

use super::FrameHandle;
use crate::widget::{AnchorPoint, LineAnchor};
use mlua::{UserDataMethods, Value};

pub fn add_line_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetStartPoint", |_, this, args: mlua::MultiValue| {
        set_line_point(this, args, true)
    });

    methods.add_method("SetEndPoint", |_, this, args: mlua::MultiValue| {
        set_line_point(this, args, false)
    });

    methods.add_method("SetThickness", |_, this, thickness: f32| {
        let mut state = this.state.borrow_mut();
        if let Some(f) = state.widgets.get_mut_visual(this.id) {
            f.line_thickness = thickness;
        }
        Ok(())
    });

    methods.add_method("GetStartPoint", |lua, this, ()| {
        get_line_point(lua, this, true)
    });

    methods.add_method("GetEndPoint", |lua, this, ()| {
        get_line_point(lua, this, false)
    });

    methods.add_method("GetThickness", |_, this, ()| {
        let state = this.state.borrow();
        let thickness = state.widgets.get(this.id).map_or(1.0, |f| f.line_thickness);
        Ok(thickness)
    });
}

fn set_line_point(this: &FrameHandle, args: mlua::MultiValue, is_start: bool) -> mlua::Result<()> {
    let args: Vec<Value> = args.into_iter().collect();

    let point_str = match args.first() {
        Some(Value::String(s)) => s.to_string_lossy().to_string(),
        _ => return Ok(()),
    };
    let point = AnchorPoint::from_str(&point_str).unwrap_or(AnchorPoint::Center);

    let target_id = match args.get(1) {
        Some(Value::UserData(ud)) => {
            ud.borrow::<FrameHandle>().ok().map(|h| h.id)
        }
        _ => None,
    };

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

    let mut state = this.state.borrow_mut();
    if let Some(f) = state.widgets.get_mut_visual(this.id) {
        if is_start {
            f.line_start = Some(anchor);
        } else {
            f.line_end = Some(anchor);
        }
    }
    Ok(())
}

fn get_line_point(lua: &mlua::Lua, this: &FrameHandle, is_start: bool) -> mlua::Result<mlua::MultiValue> {
    let state = this.state.borrow();
    let anchor = state.widgets.get(this.id).and_then(|f| {
        if is_start { f.line_start.as_ref() } else { f.line_end.as_ref() }
    });

    let Some(anchor) = anchor else {
        return Ok(mlua::MultiValue::new());
    };

    let point_str = lua.create_string(anchor.point.as_str())?;
    let target: Value = if let Some(tid) = anchor.target_id {
        let frame_key = format!("__frame_{}", tid);
        lua.globals().get::<Value>(frame_key.as_str())?
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
