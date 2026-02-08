//! Slider, StatusBar, CheckButton methods and shared SetValue/GetValue/SetMinMaxValues.

use super::widget_tooltip::val_to_f32;
use super::FrameHandle;
use crate::widget::{AttributeValue, Color, WidgetType};
use mlua::{Lua, Result, UserDataMethods, Value};
use std::rc::Rc;

pub fn add_slider_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // Note: SetMinMaxValues/GetMinMaxValues/SetValue/GetValue are in add_shared_value_methods
    add_slider_step_methods(methods);
    add_slider_orientation_methods(methods);
    add_slider_thumb_methods(methods);
    add_slider_drag_methods(methods);
}

pub fn add_statusbar_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // Note: SetMinMaxValues/GetMinMaxValues/SetValue/GetValue are in add_shared_value_methods
    add_statusbar_texture_methods(methods);
    add_statusbar_color_methods(methods);
    add_statusbar_fill_methods(methods);
}

pub fn add_checkbutton_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetChecked", |_, this, checked: bool| {
        {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame
                    .attributes
                    .insert("__checked".to_string(), AttributeValue::Boolean(checked));
            }
            // Also toggle CheckedTexture visibility if it exists
            if let Some(frame) = state.widgets.get(this.id)
                && let Some(&checked_tex_id) = frame.children_keys.get("CheckedTexture")
                    && let Some(tex) = state.widgets.get_mut(checked_tex_id) {
                        tex.visible = checked;
                    }
        }
        Ok(())
    });
    methods.add_method("GetChecked", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id)
            && let Some(AttributeValue::Boolean(checked)) = frame.attributes.get("__checked") {
                return Ok(*checked);
            }
        Ok(false)
    });
    methods.add_method("GetCheckedTexture", |lua, this, ()| {
        get_or_create_child_texture(lua, this, "CheckedTexture")
    });
}

/// Shared SetValue/GetValue/SetMinMaxValues/GetMinMaxValues that dispatch by widget type.
/// Must be registered last so it overwrites both slider and statusbar individual registrations.
pub fn add_shared_value_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_shared_set_value(methods);
    add_shared_get_value(methods);
    add_shared_set_min_max(methods);
    add_shared_get_min_max(methods);
}

// --- Slider methods ---

fn add_slider_step_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetValueStep", |_, this, step: f64| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.slider_step = step;
        }
        Ok(())
    });
    methods.add_method("GetValueStep", |_, this, ()| {
        let state = this.state.borrow();
        let step = state.widgets.get(this.id).map(|f| f.slider_step).unwrap_or(1.0);
        Ok(step)
    });
}

fn add_slider_orientation_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetOrientation", |_, this, orientation: String| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.slider_orientation = orientation.to_uppercase();
        }
        Ok(())
    });
    methods.add_method("GetOrientation", |_, this, ()| {
        let state = this.state.borrow();
        let orientation = state.widgets.get(this.id)
            .map(|f| f.slider_orientation.clone())
            .unwrap_or_else(|| "HORIZONTAL".to_string());
        Ok(orientation)
    });
}

fn add_slider_thumb_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetThumbTexture", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetThumbTexture", |lua, this, ()| {
        get_or_create_child_texture(lua, this, "ThumbTexture")
    });
}

fn add_slider_drag_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetObeyStepOnDrag", |_, this, obey: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.slider_obey_step_on_drag = obey;
        }
        Ok(())
    });
    methods.add_method("SetStepsPerPage", |_, this, steps: i32| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.slider_steps_per_page = steps;
        }
        Ok(())
    });
    methods.add_method("GetStepsPerPage", |_, this, ()| {
        let state = this.state.borrow();
        let steps = state.widgets.get(this.id).map(|f| f.slider_steps_per_page).unwrap_or(1);
        Ok(steps)
    });
}

// --- StatusBar methods ---

fn add_statusbar_texture_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetStatusBarTexture", |_, this, texture: Value| {
        let (path, bar_id) = match &texture {
            Value::String(s) => (Some(s.to_string_lossy().to_string()), None),
            Value::UserData(ud) => {
                let id = ud.borrow::<FrameHandle>().ok().map(|h| h.id);
                (None, id)
            }
            _ => (None, None),
        };
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.statusbar_texture_path = path.clone();
            if let Some(id) = bar_id {
                frame.statusbar_bar_id = Some(id);
            }
        }
        // When called with a string, apply atlas/texture to the bar texture child.
        if let Some(ref tex_str) = path {
            let bar_child_id = find_bar_texture_child(&state.widgets, this.id);
            if let Some(child_id) = bar_child_id {
                apply_bar_texture(&mut state.widgets, child_id, tex_str);
                anchor_bar_to_parent(&mut state.widgets, child_id, this.id);
            }
        }
        // The bar texture fills its parent; apply SetAllPoints anchors.
        if let Some(id) = bar_id {
            anchor_bar_to_parent(&mut state.widgets, id, this.id);
        }
        Ok(())
    });
    methods.add_method("GetStatusBarTexture", |lua, this, ()| {
        get_or_create_child_texture(lua, this, "StatusBarTexture")
    });
    methods.add_method("SetRotatesTexture", |_, _this, _rotates: bool| Ok(()));
}

fn add_statusbar_color_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetStatusBarColor", |_, this, args: mlua::MultiValue| {
        let mut it = args.into_iter();
        let r = val_to_f32(it.next(), 1.0);
        let g = val_to_f32(it.next(), 1.0);
        let b = val_to_f32(it.next(), 1.0);
        let a = val_to_f32(it.next(), 1.0);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.statusbar_color = Some(Color::new(r, g, b, a));
        }
        Ok(())
    });
    methods.add_method("GetStatusBarColor", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id)
            && let Some(c) = &frame.statusbar_color {
                return Ok((c.r, c.g, c.b, c.a));
            }
        Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32))
    });
}

fn add_statusbar_fill_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetFillStyle", |_, this, style: String| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.statusbar_fill_style = style;
        }
        Ok(())
    });
    methods.add_method("SetReverseFill", |_, this, reverse: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.statusbar_reverse_fill = reverse;
        }
        Ok(())
    });
}

// --- Shared value methods ---

fn add_shared_set_value<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetValue", |lua, this, value: f64| {
        let wtype = {
            let s = this.state.borrow();
            s.widgets.get(this.id).map(|f| f.widget_type)
        };
        match wtype {
            Some(WidgetType::Slider) => set_slider_value(lua, this, value)?,
            Some(WidgetType::StatusBar) => set_statusbar_value(lua, this, value)?,
            _ => {}
        }
        Ok(())
    });
}

fn add_shared_get_value<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetValue", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            return Ok(match frame.widget_type {
                WidgetType::Slider => frame.slider_value,
                WidgetType::StatusBar => frame.statusbar_value,
                _ => 0.0,
            });
        }
        Ok(0.0_f64)
    });
}

fn add_shared_set_min_max<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetMinMaxValues", |_, this, args: mlua::MultiValue| {
        let mut it = args.into_iter();
        let min = match it.next() {
            Some(Value::Number(n)) => n,
            Some(Value::Integer(n)) => n as f64,
            _ => 0.0,
        };
        let max = match it.next() {
            Some(Value::Number(n)) => n,
            Some(Value::Integer(n)) => n as f64,
            _ => 1.0,
        };
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            match frame.widget_type {
                WidgetType::Slider => {
                    frame.slider_min = min;
                    frame.slider_max = max;
                    frame.slider_value = frame.slider_value.clamp(min, max);
                }
                WidgetType::StatusBar => {
                    frame.statusbar_min = min;
                    frame.statusbar_max = max;
                    frame.statusbar_value = frame.statusbar_value.clamp(min, max);
                }
                _ => {}
            }
        }
        Ok(())
    });
}

fn add_shared_get_min_max<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetMinMaxValues", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            return Ok(match frame.widget_type {
                WidgetType::Slider => (frame.slider_min, frame.slider_max),
                WidgetType::StatusBar => (frame.statusbar_min, frame.statusbar_max),
                _ => (0.0, 1.0),
            });
        }
        Ok((0.0_f64, 1.0_f64))
    });
}

// --- Helper functions ---

fn set_slider_value(lua: &mlua::Lua, this: &FrameHandle, value: f64) -> mlua::Result<()> {
    let clamped = {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            let clamped = value.clamp(frame.slider_min, frame.slider_max);
            frame.slider_value = clamped;
            clamped
        } else {
            return Ok(());
        }
    };
    fire_value_changed(lua, this.id, clamped)
}

fn set_statusbar_value(lua: &mlua::Lua, this: &FrameHandle, value: f64) -> mlua::Result<()> {
    let clamped = {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            let clamped = value.clamp(frame.statusbar_min, frame.statusbar_max);
            frame.statusbar_value = clamped;
            clamped
        } else {
            return Ok(());
        }
    };
    fire_value_changed(lua, this.id, clamped)
}

/// Fire OnValueChanged script with the new value as argument.
fn fire_value_changed(lua: &mlua::Lua, frame_id: u64, value: f64) -> mlua::Result<()> {
    if let Ok(Some(scripts_table)) = lua.globals().get::<Option<mlua::Table>>("__scripts") {
        let key = format!("{}_OnValueChanged", frame_id);
        if let Ok(Some(func)) = scripts_table.get::<Option<mlua::Function>>(key.as_str()) {
            let frame_key = format!("__frame_{}", frame_id);
            if let Ok(frame_ud) = lua.globals().get::<Value>(frame_key.as_str()) {
                let _ = func.call::<()>((frame_ud, value));
            }
        }
    }
    Ok(())
}

/// Find the bar texture child of a StatusBar by stored ID or children_keys.
fn find_bar_texture_child(widgets: &crate::widget::WidgetRegistry, parent_id: u64) -> Option<u64> {
    let frame = widgets.get(parent_id)?;
    frame.statusbar_bar_id
        .or_else(|| frame.children_keys.get("BarTexture").copied())
        .or_else(|| frame.children_keys.get("StatusBarTexture").copied())
        .or_else(|| frame.children_keys.get("Bar").copied())
}

/// Apply a texture path or atlas name to a bar texture child.
fn apply_bar_texture(widgets: &mut crate::widget::WidgetRegistry, child_id: u64, tex_str: &str) {
    // Try atlas lookup first
    if let Some(lookup) = crate::atlas::get_atlas_info(tex_str) {
        let info = lookup.info;
        if let Some(frame) = widgets.get_mut(child_id) {
            frame.texture = Some(info.file.to_string());
            let uvs = (info.left_tex_coord, info.right_tex_coord, info.top_tex_coord, info.bottom_tex_coord);
            frame.atlas_tex_coords = Some(uvs);
            frame.tex_coords = Some(uvs);
            frame.horiz_tile = info.tiles_horizontally;
            frame.vert_tile = info.tiles_vertically;
            frame.atlas = Some(tex_str.to_string());
        }
    } else if let Some(frame) = widgets.get_mut(child_id) {
        // Treat as a file path
        frame.texture = Some(tex_str.to_string());
        frame.atlas = None;
        frame.tex_coords = None;
        frame.atlas_tex_coords = None;
    }
}

/// Apply SetAllPoints-style anchors to make a bar texture fill its parent.
fn anchor_bar_to_parent(widgets: &mut crate::widget::WidgetRegistry, bar_id: u64, parent_id: u64) {
    use crate::widget::{Anchor, AnchorPoint};
    if let Some(bar) = widgets.get_mut(bar_id) {
        bar.anchors = vec![
            Anchor {
                point: AnchorPoint::TopLeft,
                relative_to: None,
                relative_to_id: Some(parent_id as usize),
                relative_point: AnchorPoint::TopLeft,
                x_offset: 0.0,
                y_offset: 0.0,
            },
            Anchor {
                point: AnchorPoint::BottomRight,
                relative_to: None,
                relative_to_id: Some(parent_id as usize),
                relative_point: AnchorPoint::BottomRight,
                x_offset: 0.0,
                y_offset: 0.0,
            },
        ];
    }
}

/// Look up or create a child texture by key and return it as a FrameHandle userdata.
/// Used by GetThumbTexture, GetStatusBarTexture, GetCheckedTexture, etc.
pub(super) fn get_or_create_child_texture(
    lua: &Lua,
    this: &FrameHandle,
    key: &str,
) -> Result<Value> {
    let mut state = this.state.borrow_mut();
    let tex_id = super::methods_helpers::get_or_create_button_texture(
        &mut state, this.id, key,
    );
    drop(state);
    let handle = FrameHandle {
        id: tex_id,
        state: Rc::clone(&this.state),
    };
    let ud = lua.create_userdata(handle)?;
    Ok(Value::UserData(ud))
}
