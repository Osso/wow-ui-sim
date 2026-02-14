//! Miscellaneous widget methods: ColorSelect, drag/move/resize, SimpleHTML, and stubs.

use super::FrameHandle;
use crate::widget::AttributeValue;
use mlua::{UserDataMethods, Value};

pub fn add_colorselect_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_colorselect_rgb_methods(methods);
    add_colorselect_hsv_methods(methods);
}

pub fn add_drag_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_drag_start_stop_methods(methods);
    add_drag_clamp_methods(methods);
    add_drag_resize_methods(methods);
}

pub fn add_simplehtml_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetHyperlinkFormat(format)
    methods.add_method("SetHyperlinkFormat", |_, this, format: String| {
        let mut state = this.state.borrow_mut();
        if let Some(data) = state.simple_htmls.get_mut(&this.id) {
            data.hyperlink_format = format;
        }
        Ok(())
    });

    // GetHyperlinkFormat()
    methods.add_method("GetHyperlinkFormat", |_, this, ()| {
        let state = this.state.borrow();
        let format = state
            .simple_htmls
            .get(&this.id)
            .map(|d| d.hyperlink_format.clone())
            .unwrap_or_else(|| "|H%s|h%s|h".to_string());
        Ok(format)
    });

    // SetHyperlinksEnabled(enabled)
    methods.add_method("SetHyperlinksEnabled", |_, this, enabled: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(data) = state.simple_htmls.get_mut(&this.id) {
            data.hyperlinks_enabled = enabled;
        }
        Ok(())
    });

    // GetHyperlinksEnabled()
    methods.add_method("GetHyperlinksEnabled", |_, this, ()| {
        let state = this.state.borrow();
        let enabled = state
            .simple_htmls
            .get(&this.id)
            .map(|d| d.hyperlinks_enabled)
            .unwrap_or(true);
        Ok(enabled)
    });

    // GetContentHeight() - estimate based on text length and font size
    methods.add_method("GetContentHeight", |_, this, ()| {
        let state = this.state.borrow();
        let frame = match state.widgets.get(this.id) {
            Some(f) => f,
            None => return Ok(0.0_f64),
        };
        let text = match &frame.text {
            Some(t) if !t.is_empty() => t,
            _ => return Ok(0.0_f64),
        };
        let font_size = frame.font_size.max(12.0) as f64;
        let line_height = font_size * 1.2;
        let width = frame.width.max(200.0) as f64;
        let chars_per_line = (width / (font_size * 0.6)).max(1.0);
        let estimated_lines = (text.len() as f64 / chars_per_line).ceil().max(1.0);
        Ok(estimated_lines * line_height)
    });

    // GetTextData() - return empty table (no HTML parsing yet)
    methods.add_method("GetTextData", |lua, _this, ()| {
        let table = lua.create_table()?;
        Ok(table)
    });
}

/// Miscellaneous widget method stubs referenced by Blizzard UI.
pub fn add_misc_widget_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetupMenu(generator) - dropdown/context menu setup (used everywhere)
    methods.add_method("SetupMenu", |_, _this, _args: mlua::MultiValue| Ok(()));

    // SetAlertContainer(container) - alert frame system
    methods.add_method("SetAlertContainer", |_, _this, _container: Value| Ok(()));

    // SetColorFill(r, g, b, a) - StatusBar fill color
    methods.add_method("SetColorFill", |_, _this, _args: mlua::MultiValue| Ok(()));

    // SetTextToFit(text) - FontString method that auto-sizes
    methods.add_method("SetTextToFit", |_, this, text: Option<String>| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(this.id) {
            frame.text = text;
        }
        Ok(())
    });

    // SetSelectionTranslator(func) - tab/dropdown selection
    methods.add_method("SetSelectionTranslator", |_, _this, _func: Value| Ok(()));

    // SetItemButtonScale(scale) - item button sizing
    methods.add_method("SetItemButtonScale", |_, _this, _scale: Value| Ok(()));

    // Mixin override: ModelScenelRotateButtonMixin defines SetRotationIncrement(increment)
    methods.add_method("SetRotationIncrement", |lua, this, inc: Value| {
        if let Some((func, ud)) = super::methods_helpers::get_mixin_override(lua, this.id, "SetRotationIncrement") {
            return func.call::<()>((ud, inc));
        }
        Ok(())
    });

    // Init() - DO NOT add a Rust stub here. ScrollBoxListMixin:Init (Lua) must
    // be callable via __index. A Rust method takes priority and would shadow it.

    // UpdateItemContextMatching() - item slot context matching (PaperDoll)
    methods.add_method("UpdateItemContextMatching", |_, _this, _args: mlua::MultiValue| Ok(()));

    // UpdateHeight() - recalculate height (UIWidget containers)
    methods.add_method("UpdateHeight", |_, _this, ()| Ok(()));

    // SetDefaultText(text) - dropdown default text
    methods.add_method("SetDefaultText", |_, _this, _text: Value| Ok(()));

    // SetVisuals(info) - UnitFrame spark/bar visual configuration
    methods.add_method("SetVisuals", |_, _this, _args: mlua::MultiValue| Ok(()));

    // NOTE: Initialize is intentionally NOT defined here as a no-op.
    // It must be dispatched via Lua mixin methods (e.g. AlternatePowerBarBaseMixin,
    // EvokerEbonMightBarMixin) through the __index custom fields lookup.
    // A Rust add_method shadows __index, breaking mixin dispatch.

    // RegisterForWidgetSet(widgetSetID, layoutFunc, initFunc, attachedUnitInfo) - UIWidget container
    methods.add_method("RegisterForWidgetSet", |_, _this, _args: mlua::MultiValue| Ok(()));

    // UnregisterForWidgetSet() - UIWidget container unregister
    methods.add_method("UnregisterForWidgetSet", |_, _this, _args: mlua::MultiValue| Ok(()));
}

// --- ColorSelect ---

fn add_colorselect_rgb_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetColorRGB(r, g, b) - Set the RGB color
    methods.add_method("SetColorRGB", |_, this, (r, g, b): (f64, f64, f64)| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(this.id) {
            frame
                .attributes
                .insert("colorR".to_string(), AttributeValue::Number(r));
            frame
                .attributes
                .insert("colorG".to_string(), AttributeValue::Number(g));
            frame
                .attributes
                .insert("colorB".to_string(), AttributeValue::Number(b));
        }
        Ok(())
    });

    // GetColorRGB() - Get the RGB color
    methods.add_method("GetColorRGB", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            let get_num = |key: &str| -> f64 {
                match frame.attributes.get(key) {
                    Some(AttributeValue::Number(n)) => *n,
                    _ => 1.0,
                }
            };
            let r = get_num("colorR");
            let g = get_num("colorG");
            let b = get_num("colorB");
            return Ok((r, g, b));
        }
        Ok((1.0, 1.0, 1.0))
    });
}

fn add_colorselect_hsv_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetColorHSV(h, s, v) - Set the HSV color
    methods.add_method("SetColorHSV", |_, this, (h, s, v): (f64, f64, f64)| {
        let (r, g, b) = hsv_to_rgb(h, s, v);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(this.id) {
            frame
                .attributes
                .insert("colorR".to_string(), AttributeValue::Number(r));
            frame
                .attributes
                .insert("colorG".to_string(), AttributeValue::Number(g));
            frame
                .attributes
                .insert("colorB".to_string(), AttributeValue::Number(b));
            frame
                .attributes
                .insert("colorH".to_string(), AttributeValue::Number(h % 360.0));
            frame
                .attributes
                .insert("colorS".to_string(), AttributeValue::Number(s));
            frame
                .attributes
                .insert("colorV".to_string(), AttributeValue::Number(v));
        }
        Ok(())
    });

    // GetColorHSV() - Get the HSV color
    methods.add_method("GetColorHSV", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            let get_num = |key: &str| -> Option<f64> {
                match frame.attributes.get(key) {
                    Some(AttributeValue::Number(n)) => Some(*n),
                    _ => None,
                }
            };
            // Check if we have stored HSV values
            if let (Some(h), Some(s), Some(v)) =
                (get_num("colorH"), get_num("colorS"), get_num("colorV"))
            {
                return Ok((h, s, v));
            }
            // Otherwise convert from RGB
            let r: f64 = get_num("colorR").unwrap_or(1.0);
            let g: f64 = get_num("colorG").unwrap_or(1.0);
            let b: f64 = get_num("colorB").unwrap_or(1.0);
            return Ok(rgb_to_hsv(r, g, b));
        }
        Ok((0.0, 0.0, 1.0))
    });
}

// --- Drag/Move/Resize ---

fn add_drag_start_stop_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("StartMoving", |_, this, ()| {
        if let Ok(mut s) = this.state.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(this.id)
                && frame.movable {
                    frame.is_moving = true;
                }
        Ok(())
    });
    methods.add_method("StopMovingOrSizing", |_, this, ()| {
        if let Ok(mut s) = this.state.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(this.id) {
                frame.is_moving = false;
            }
        Ok(())
    });
    methods.add_method("SetMovable", |_, this, movable: bool| {
        if let Ok(mut s) = this.state.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(this.id) {
                frame.movable = movable;
            }
        Ok(())
    });
    methods.add_method("IsMovable", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow()
            && let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.movable);
            }
        Ok(false)
    });
    methods.add_method("SetResizable", |_, this, resizable: bool| {
        if let Ok(mut s) = this.state.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(this.id) {
                frame.resizable = resizable;
            }
        Ok(())
    });
    methods.add_method("IsResizable", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow()
            && let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.resizable);
            }
        Ok(false)
    });
}

fn add_drag_clamp_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetClampedToScreen", |_, this, clamped: bool| {
        if let Ok(mut s) = this.state.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(this.id) {
                frame.clamped_to_screen = clamped;
            }
        Ok(())
    });
    methods.add_method("IsClampedToScreen", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow()
            && let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.clamped_to_screen);
            }
        Ok(false)
    });
    methods.add_method("SetClampRectInsets", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
}

fn add_drag_resize_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetResizeBounds", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetResizeBounds", |_, _this, ()| {
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32))
    });
    // Legacy resize bound methods (deprecated in favor of SetResizeBounds)
    methods.add_method("SetMinResize", |_, _this, (_w, _h): (f32, f32)| Ok(()));
    methods.add_method("SetMaxResize", |_, _this, (_w, _h): (f32, f32)| Ok(()));
    methods.add_method("StartSizing", |_, _this, _point: Option<String>| Ok(()));
    methods.add_method("RegisterForDrag", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetUserPlaced", |_, _this, _user_placed: bool| Ok(()));
    methods.add_method("IsUserPlaced", |_, _this, ()| Ok(false));
    methods.add_method("SetDontSavePosition", |_, _this, _dont_save: bool| Ok(()));
}

// --- Color conversion helpers ---

/// Convert HSV to RGB.
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r1 + m, g1 + m, b1 + m)
}

/// Convert RGB to HSV.
fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, v)
}
