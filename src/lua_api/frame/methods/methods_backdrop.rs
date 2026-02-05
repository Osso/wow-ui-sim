//! Backdrop methods: SetBackdrop, SetBackdropColor, etc.

use super::FrameHandle;
use mlua::UserDataMethods;

/// Add backdrop methods to FrameHandle UserData.
pub fn add_backdrop_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetBackdrop(backdropInfo) - WoW backdrop system for frame backgrounds
    methods.add_method("SetBackdrop", |_, this, backdrop: Option<mlua::Table>| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            if let Some(info) = backdrop {
                frame.backdrop.enabled = true;
                // Parse texture paths
                if let Ok(bg_file) = info.get::<String>("bgFile") {
                    frame.backdrop.bg_file = Some(bg_file);
                }
                if let Ok(edge_file) = info.get::<String>("edgeFile") {
                    frame.backdrop.edge_file = Some(edge_file);
                }
                // Parse edge size if provided
                if let Ok(edge_size) = info.get::<f32>("edgeSize") {
                    frame.backdrop.edge_size = edge_size;
                }
                // Parse insets if provided
                if let Ok(insets) = info.get::<mlua::Table>("insets") {
                    if let Ok(left) = insets.get::<f32>("left") {
                        frame.backdrop.insets = left;
                    }
                }
            } else {
                frame.backdrop.enabled = false;
                frame.backdrop.bg_file = None;
                frame.backdrop.edge_file = None;
            }
        }
        Ok(())
    });

    // ApplyBackdrop() - Apply backdrop template (used by DBM and other addons)
    methods.add_method("ApplyBackdrop", |_, this, args: mlua::MultiValue| {
        // ApplyBackdrop can take optional r, g, b, a parameters for background color
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.backdrop.enabled = true;
            // Parse optional color arguments
            let args_vec: Vec<mlua::Value> = args.into_iter().collect();
            if args_vec.len() >= 3 {
                let r = match &args_vec[0] {
                    mlua::Value::Number(n) => *n as f32,
                    mlua::Value::Integer(n) => *n as f32,
                    _ => 0.0,
                };
                let g = match &args_vec[1] {
                    mlua::Value::Number(n) => *n as f32,
                    mlua::Value::Integer(n) => *n as f32,
                    _ => 0.0,
                };
                let b = match &args_vec[2] {
                    mlua::Value::Number(n) => *n as f32,
                    mlua::Value::Integer(n) => *n as f32,
                    _ => 0.0,
                };
                let a = if args_vec.len() >= 4 {
                    match &args_vec[3] {
                        mlua::Value::Number(n) => *n as f32,
                        mlua::Value::Integer(n) => *n as f32,
                        _ => 1.0,
                    }
                } else {
                    1.0
                };
                frame.backdrop.bg_color = crate::widget::Color::new(r, g, b, a);
            }
        }
        Ok(())
    });

    // SetBackdropColor(r, g, b, a) - Set backdrop background color
    methods.add_method(
        "SetBackdropColor",
        |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.backdrop.enabled = true;
                frame.backdrop.bg_color = crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
            }
            Ok(())
        },
    );

    // GetBackdropColor() - Get backdrop background color
    methods.add_method("GetBackdropColor", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            Ok((
                frame.backdrop.bg_color.r,
                frame.backdrop.bg_color.g,
                frame.backdrop.bg_color.b,
                frame.backdrop.bg_color.a,
            ))
        } else {
            Ok((0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32))
        }
    });

    // SetBackdropBorderColor(r, g, b, a) - Set backdrop border color
    methods.add_method(
        "SetBackdropBorderColor",
        |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.backdrop.enabled = true;
                frame.backdrop.border_color = crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
            }
            Ok(())
        },
    );

    // GetBackdropBorderColor() - Get backdrop border color
    methods.add_method("GetBackdropBorderColor", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            Ok((
                frame.backdrop.border_color.r,
                frame.backdrop.border_color.g,
                frame.backdrop.border_color.b,
                frame.backdrop.border_color.a,
            ))
        } else {
            Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32))
        }
    });
}
