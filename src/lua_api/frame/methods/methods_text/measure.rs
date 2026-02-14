//! Text measurement, word wrap, text scale, and spacing methods.

use super::super::super::handle::{get_sim_state, lud_to_id};
use super::{is_simple_html, is_text_type, val_to_f64};
use crate::lua_api::simple_html::TextStyle;
use crate::render::font::WowFontSystem;
use mlua::{LightUserData, Lua, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Add measurement, word wrap, text scale, and spacing methods.
pub fn add_measure_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_text_measurement_methods(lua, methods)?;
    add_text_height_methods(lua, methods)?;
    add_word_wrap_methods(lua, methods)?;
    add_text_scale_methods(lua, methods)?;
    add_spacing_methods(lua, methods)?;
    Ok(())
}

/// GetStringWidth, GetTextWidth, GetUnboundedStringWidth, GetStringHeight.
fn add_text_measurement_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetStringWidth", lua.create_function(|lua, ud: LightUserData| {
        measure_text_width(lua, lud_to_id(ud))
    })?)?;

    methods.set("GetTextWidth", lua.create_function(|lua, ud: LightUserData| {
        measure_text_width(lua, lud_to_id(ud))
    })?)?;

    methods.set("GetUnboundedStringWidth", lua.create_function(|lua, ud: LightUserData| {
        measure_text_width(lua, lud_to_id(ud))
    })?)?;

    methods.set("GetStringHeight", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        measure_string_height(lua, id)
    })?)?;

    // GetLineHeight - height of a single line at the current font size.
    methods.set("GetLineHeight", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let font_size = state.widgets.get(id).map_or(12.0_f32, |f| f.font_size);
        Ok((font_size * 1.2).ceil() as f64)
    })?)?;

    Ok(())
}

/// Shared implementation for GetStringWidth / GetTextWidth / GetUnboundedStringWidth.
fn measure_text_width(lua: &Lua, id: u64) -> mlua::Result<f64> {
    let state_rc = get_sim_state(lua);
    let state = state_rc.borrow();
    let (text, font_path, font_size) = match state.widgets.get(id) {
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

/// Measure string height, accounting for word wrap.
fn measure_string_height(lua: &Lua, id: u64) -> mlua::Result<f64> {
    let state_rc = get_sim_state(lua);
    let state = state_rc.borrow();
    let (text, font_path, font_size, word_wrap, width) = match state.widgets.get(id) {
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
}

/// SetWordWrap, GetWordWrap, IsTruncated, CanWordWrap, GetWrappedWidth,
/// SetNonSpaceWrap, CanNonSpaceWrap, SetMaxLines, GetMaxLines.
fn add_word_wrap_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetWordWrap", lua.create_function(|lua, (ud, wrap): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(mut s) = state_rc.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(id) {
                frame.word_wrap = wrap;
            }
        Ok(())
    })?)?;

    methods.set("GetWordWrap", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(s) = state_rc.try_borrow()
            && let Some(frame) = s.widgets.get(id) {
                return Ok(frame.word_wrap);
            }
        Ok(false)
    })?)?;

    methods.set("IsTruncated", lua.create_function(|_lua, _ud: LightUserData| Ok(false))?)?;
    methods.set("CanWordWrap", lua.create_function(|_lua, _ud: LightUserData| Ok(true))?)?;

    methods.set("GetWrappedWidth", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let width = state.widgets.get(id).map(|f| f.width).unwrap_or(0.0);
        Ok(width)
    })?)?;

    methods.set("SetNonSpaceWrap", lua.create_function(
        |_lua, (_ud, _wrap): (LightUserData, bool)| Ok(()),
    )?)?;
    methods.set("CanNonSpaceWrap", lua.create_function(|_lua, _ud: LightUserData| Ok(true))?)?;

    add_max_lines_methods(lua, methods)?;
    Ok(())
}

/// SetMaxLines, GetMaxLines.
fn add_max_lines_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetMaxLines", lua.create_function(|lua, (ud, max_lines): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(mut s) = state_rc.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(id) {
                frame.max_lines = max_lines.max(0) as u32;
            }
        Ok(())
    })?)?;

    methods.set("GetMaxLines", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(s) = state_rc.try_borrow()
            && let Some(frame) = s.widgets.get(id) {
                return Ok(frame.max_lines as i32);
            }
        Ok(0i32)
    })?)?;

    Ok(())
}

/// SetTextHeight - sets the font height (effectively font size) for a FontString.
fn add_text_height_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetTextHeight", lua.create_function(|lua, (ud, height): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.font_size = height as f32;
        }
        Ok(())
    })?)?;

    methods.set("GetTextHeight", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            return Ok(frame.font_size as f64);
        }
        Ok(12.0_f64)
    })?)?;

    Ok(())
}

/// SetTextScale, GetTextScale.
fn add_text_scale_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetTextScale", lua.create_function(|lua, (ud, scale): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.text_scale = scale;
        }
        Ok(())
    })?)?;

    methods.set("GetTextScale", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            return Ok(frame.text_scale);
        }
        Ok(1.0_f64)
    })?)?;

    Ok(())
}

/// SetIndentedWordWrap, SetSpacing, GetSpacing.
fn add_spacing_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetIndentedWordWrap", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(lua, id);

        if is_html && args_vec.len() >= 2
            && let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    return set_indented_wrap_html(lua, id, &type_str, &args_vec);
                }
            }
        Ok(())
    })?)?;

    methods.set("SetSpacing", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(lua, id);

        if is_html && args_vec.len() >= 2
            && let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    return set_spacing_html(lua, id, &type_str, &args_vec);
                }
            }
        Ok(())
    })?)?;

    methods.set("GetSpacing", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                return get_spacing_html(lua, id, &type_str);
            }
        }
        Ok(0.0_f64)
    })?)?;

    Ok(())
}

/// Set indented word wrap for a SimpleHTML text type.
fn set_indented_wrap_html(lua: &Lua, id: u64, type_str: &str, args_vec: &[Value]) -> mlua::Result<()> {
    let indent = matches!(args_vec.get(1), Some(Value::Boolean(true)));
    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    if let Some(data) = state.simple_htmls.get_mut(&id) {
        let style = data.text_styles.entry(type_str.to_string()).or_insert_with(TextStyle::default);
        style.indented_word_wrap = indent;
    }
    Ok(())
}

/// Set spacing for a SimpleHTML text type.
fn set_spacing_html(lua: &Lua, id: u64, type_str: &str, args_vec: &[Value]) -> mlua::Result<()> {
    let spacing = val_to_f64(args_vec.get(1), 0.0);
    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    if let Some(data) = state.simple_htmls.get_mut(&id) {
        let style = data.text_styles.entry(type_str.to_string()).or_insert_with(TextStyle::default);
        style.spacing = spacing as f32;
    }
    Ok(())
}

/// Get spacing for a SimpleHTML text type.
fn get_spacing_html(lua: &Lua, id: u64, type_str: &str) -> mlua::Result<f64> {
    let state_rc = get_sim_state(lua);
    let state = state_rc.borrow();
    if let Some(data) = state.simple_htmls.get(&id)
        && let Some(style) = data.text_styles.get(type_str) {
            return Ok(style.spacing as f64);
        }
    Ok(0.0_f64)
}
