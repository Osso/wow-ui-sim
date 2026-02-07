//! Backdrop methods: SetBackdrop, SetBackdropColor, etc.

use super::FrameHandle;
use mlua::UserDataMethods;

/// Add backdrop methods to FrameHandle UserData.
pub fn add_backdrop_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_set_backdrop_methods(methods);
    add_backdrop_color_methods(methods);
}

/// Extract a float from a Lua Value (Number or Integer).
fn value_to_f32(v: &mlua::Value) -> f32 {
    match v {
        mlua::Value::Number(n) => *n as f32,
        mlua::Value::Integer(n) => *n as f32,
        _ => 0.0,
    }
}

/// SetBackdrop(backdropInfo) and ApplyBackdrop() - backdrop setup.
fn add_set_backdrop_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetBackdrop", |_, this, backdrop: Option<mlua::Table>| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            if let Some(info) = backdrop {
                frame.backdrop.enabled = true;
                if let Ok(bg_file) = info.get::<String>("bgFile") {
                    frame.backdrop.bg_file = Some(bg_file);
                }
                if let Ok(edge_file) = info.get::<String>("edgeFile") {
                    frame.backdrop.edge_file = Some(edge_file);
                }
                if let Ok(edge_size) = info.get::<f32>("edgeSize") {
                    frame.backdrop.edge_size = edge_size;
                }
                if let Ok(insets) = info.get::<mlua::Table>("insets")
                    && let Ok(left) = insets.get::<f32>("left") {
                        frame.backdrop.insets = left;
                    }
            } else {
                frame.backdrop.enabled = false;
                frame.backdrop.bg_file = None;
                frame.backdrop.edge_file = None;
            }
        }
        Ok(())
    });

    methods.add_method("ApplyBackdrop", |_, this, args: mlua::MultiValue| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.backdrop.enabled = true;
            let args_vec: Vec<mlua::Value> = args.into_iter().collect();
            if args_vec.len() >= 3 {
                let r = value_to_f32(&args_vec[0]);
                let g = value_to_f32(&args_vec[1]);
                let b = value_to_f32(&args_vec[2]);
                let a = if args_vec.len() >= 4 {
                    value_to_f32(&args_vec[3])
                } else {
                    1.0
                };
                frame.backdrop.bg_color = crate::widget::Color::new(r, g, b, a);
            }
        }
        Ok(())
    });
}

/// Backdrop color get/set methods for background and border.
fn add_backdrop_color_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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

    methods.add_method(
        "SetBackdropBorderColor",
        |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.backdrop.enabled = true;
                frame.backdrop.border_color =
                    crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
            }
            Ok(())
        },
    );

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
