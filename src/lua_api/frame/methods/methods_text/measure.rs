//! Text measurement, word wrap, text scale, and spacing methods.

use super::super::FrameHandle;
use super::{is_simple_html, is_text_type, val_to_f64};
use crate::lua_api::simple_html::TextStyle;
use crate::render::font::WowFontSystem;
use mlua::{UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Add measurement, word wrap, text scale, and spacing methods.
pub fn add_measure_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_text_measurement_methods(methods);
    add_text_height_methods(methods);
    add_word_wrap_methods(methods);
    add_text_scale_methods(methods);
    add_spacing_methods(methods);
}

/// GetStringWidth, GetTextWidth, GetUnboundedStringWidth, GetStringHeight.
fn add_text_measurement_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetStringWidth", |lua, this, ()| {
        measure_text_width(lua, this)
    });

    methods.add_method("GetTextWidth", |lua, this, ()| {
        measure_text_width(lua, this)
    });

    methods.add_method("GetUnboundedStringWidth", |lua, this, ()| {
        measure_text_width(lua, this)
    });

    methods.add_method("GetStringHeight", |lua, this, ()| {
        let state = this.state.borrow();
        let (text, font_path, font_size, word_wrap, width) = match state.widgets.get(this.id) {
            Some(f) => (f.text.clone(), f.font.clone(), f.font_size, f.word_wrap, f.width),
            None => return Ok(12.0_f64),
        };
        drop(state);
        let text = match text {
            Some(t) if !t.is_empty() => t,
            _ => return Ok((font_size * 1.2).ceil() as f64),
        };
        let wrap_width = if word_wrap && width > 0.0 {
            Some(width)
        } else {
            None
        };
        if let Some(fs_rc) = lua.app_data_ref::<Rc<RefCell<WowFontSystem>>>() {
            let mut fs = fs_rc.borrow_mut();
            Ok(fs.measure_text_height(&text, font_path.as_deref(), font_size, wrap_width) as f64)
        } else {
            Ok((font_size * 1.2).ceil() as f64)
        }
    });

    // GetLineHeight - height of a single line at the current font size.
    methods.add_method("GetLineHeight", |_, this, ()| {
        let state = this.state.borrow();
        let font_size = state.widgets.get(this.id).map_or(12.0_f32, |f| f.font_size);
        Ok((font_size * 1.2).ceil() as f64)
    });
}

/// Shared implementation for GetStringWidth / GetTextWidth / GetUnboundedStringWidth.
fn measure_text_width(lua: &mlua::Lua, this: &FrameHandle) -> mlua::Result<f64> {
    let state = this.state.borrow();
    let (text, font_path, font_size) = match state.widgets.get(this.id) {
        Some(f) => (f.text.clone(), f.font.clone(), f.font_size),
        None => return Ok(0.0),
    };
    drop(state);

    let text = match text {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(0.0),
    };

    if let Some(fs_rc) = lua.app_data_ref::<Rc<RefCell<WowFontSystem>>>() {
        let mut fs = fs_rc.borrow_mut();
        Ok(fs.measure_text_width(&text, font_path.as_deref(), font_size) as f64)
    } else {
        Ok(text.len() as f64 * 7.0)
    }
}

/// SetWordWrap, GetWordWrap, IsTruncated, CanWordWrap, GetWrappedWidth,
/// SetNonSpaceWrap, CanNonSpaceWrap, SetMaxLines, GetMaxLines.
fn add_word_wrap_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetWordWrap", |_, this, wrap: bool| {
        if let Ok(mut s) = this.state.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut(this.id) {
                frame.word_wrap = wrap;
            }
        Ok(())
    });

    methods.add_method("GetWordWrap", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow()
            && let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.word_wrap);
            }
        Ok(false)
    });

    methods.add_method("IsTruncated", |_, _this, ()| Ok(false));
    methods.add_method("CanWordWrap", |_, _this, ()| Ok(true));

    methods.add_method("GetWrappedWidth", |_, this, ()| {
        let state = this.state.borrow();
        let width = state
            .widgets
            .get(this.id)
            .map(|f| f.width)
            .unwrap_or(0.0);
        Ok(width)
    });

    methods.add_method("SetNonSpaceWrap", |_, _this, _wrap: bool| Ok(()));
    methods.add_method("CanNonSpaceWrap", |_, _this, ()| Ok(true));

    methods.add_method("SetMaxLines", |_, this, max_lines: i32| {
        if let Ok(mut s) = this.state.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut(this.id) {
                frame.max_lines = max_lines.max(0) as u32;
            }
        Ok(())
    });

    methods.add_method("GetMaxLines", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow()
            && let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.max_lines as i32);
            }
        Ok(0i32)
    });
}

/// SetTextHeight - sets the font height (effectively font size) for a FontString.
fn add_text_height_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetTextHeight", |_, this, height: f64| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.font_size = height as f32;
        }
        Ok(())
    });

    methods.add_method("GetTextHeight", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            return Ok(frame.font_size as f64);
        }
        Ok(12.0_f64)
    });
}

/// SetTextScale, GetTextScale.
fn add_text_scale_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetTextScale", |_, this, scale: f64| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.text_scale = scale;
        }
        Ok(())
    });

    methods.add_method("GetTextScale", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            return Ok(frame.text_scale);
        }
        Ok(1.0_f64)
    });
}

/// SetIndentedWordWrap, SetSpacing, GetSpacing.
fn add_spacing_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetIndentedWordWrap", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        if is_html && args_vec.len() >= 2
            && let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    let indent = matches!(args_vec.get(1), Some(Value::Boolean(true)));
                    let mut state = this.state.borrow_mut();
                    if let Some(data) = state.simple_htmls.get_mut(&this.id) {
                        let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                        style.indented_word_wrap = indent;
                    }
                    return Ok(());
                }
            }
        Ok(())
    });

    methods.add_method("SetSpacing", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        if is_html && args_vec.len() >= 2
            && let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    let spacing = val_to_f64(args_vec.get(1), 0.0);
                    let mut state = this.state.borrow_mut();
                    if let Some(data) = state.simple_htmls.get_mut(&this.id) {
                        let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                        style.spacing = spacing as f32;
                    }
                    return Ok(());
                }
            }
        Ok(())
    });

    methods.add_method("GetSpacing", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let state = this.state.borrow();
                if let Some(data) = state.simple_htmls.get(&this.id)
                    && let Some(style) = data.text_styles.get(&type_str) {
                        return Ok(style.spacing as f64);
                    }
                return Ok(0.0_f64);
            }
        }
        Ok(0.0_f64)
    });
}
