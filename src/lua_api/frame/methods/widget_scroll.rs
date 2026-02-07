//! ScrollFrame and ScrollBox widget methods.

use super::widget_tooltip::fire_tooltip_script;
use super::FrameHandle;
use mlua::{UserDataMethods, Value};

pub fn add_scrollframe_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_scrollframe_child_methods(methods);
    add_scrollframe_offset_methods(methods);
    add_scrollframe_range_methods(methods);
}

pub fn add_scrollbox_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("RegisterCallback", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
    methods.add_method("ForEachFrame", |_, _this, _callback: mlua::Function| Ok(()));
    methods.add_method("UnregisterCallback", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
    methods.add_method("CanInterpolateScroll", |_, _this, ()| Ok(false));
    methods.add_method("SetInterpolateScroll", |_, _this, _enabled: bool| Ok(()));
}

fn add_scrollframe_child_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetScrollChild", |_, this, child: Value| {
        let child_id = match &child {
            Value::UserData(ud) => ud.borrow::<FrameHandle>().ok().map(|h| h.id),
            _ => None,
        };
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.scroll_child_id = child_id;
        }
        Ok(())
    });
    methods.add_method("GetScrollChild", |lua, this, ()| {
        let child_id = {
            let state = this.state.borrow();
            state.widgets.get(this.id).and_then(|f| f.scroll_child_id)
        };
        match child_id {
            Some(id) => {
                let key = format!("__frame_{}", id);
                lua.globals().get::<Value>(key.as_str())
            }
            None => Ok(Value::Nil),
        }
    });
    methods.add_method("UpdateScrollChildRect", |_, _this, ()| Ok(()));
}

fn add_scrollframe_offset_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetHorizontalScroll", |_, this, offset: f64| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.scroll_horizontal = offset;
        }
        Ok(())
    });
    methods.add_method("GetHorizontalScroll", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.scroll_horizontal).unwrap_or(0.0))
    });
    methods.add_method("SetVerticalScroll", |lua, this, offset: f64| {
        {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.scroll_vertical = offset;
            }
        }
        fire_tooltip_script(lua, this.id, "OnScrollRangeChanged")?;
        Ok(())
    });
    methods.add_method("GetVerticalScroll", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.scroll_vertical).unwrap_or(0.0))
    });
}

fn add_scrollframe_range_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetHorizontalScrollRange", |_, this, ()| {
        let state = this.state.borrow();
        let frame = match state.widgets.get(this.id) {
            Some(f) => f,
            None => return Ok(0.0_f64),
        };
        let child_width = frame.scroll_child_id
            .and_then(|cid| state.widgets.get(cid))
            .map(|c| c.width as f64)
            .unwrap_or(0.0);
        Ok((child_width - frame.width as f64).max(0.0))
    });
    methods.add_method("GetVerticalScrollRange", |_, this, ()| {
        let state = this.state.borrow();
        let frame = match state.widgets.get(this.id) {
            Some(f) => f,
            None => return Ok(0.0_f64),
        };
        let child_height = frame.scroll_child_id
            .and_then(|cid| state.widgets.get(cid))
            .map(|c| c.height as f64)
            .unwrap_or(0.0);
        Ok((child_height - frame.height as f64).max(0.0))
    });
}
