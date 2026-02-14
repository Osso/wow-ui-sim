//! Text/FontString methods: SetText, SetFont, SetJustifyH, etc.

mod decor;
mod measure;

use crate::lua_api::frame::handle::{frame_lud, get_sim_state, lud_to_id};
use crate::lua_api::simple_html::TextStyle;
use crate::widget::WidgetType;
use mlua::{LightUserData, Lua, Value};

/// Known HTML text types for SimpleHTML per-textType methods.
pub(super) fn is_text_type(s: &str) -> bool {
    matches!(s, "h1" | "h2" | "h3" | "p")
}

/// Check if a frame ID corresponds to a SimpleHTML widget.
pub(super) fn is_simple_html(lua: &Lua, id: u64) -> bool {
    let state_rc = get_sim_state(lua);
    let state = state_rc.borrow();
    state.widgets.get(id)
        .is_some_and(|f| f.widget_type == WidgetType::SimpleHTML)
}

/// Extract f32 from a reference to a Lua Value.
pub(super) fn val_to_f32(val: Option<&Value>, default: f32) -> f32 {
    match val {
        Some(Value::Number(n)) => *n as f32,
        Some(Value::Integer(n)) => *n as f32,
        _ => default,
    }
}

/// Extract f64 from a reference to a Lua Value.
pub(super) fn val_to_f64(val: Option<&Value>, default: f64) -> f64 {
    match val {
        Some(Value::Number(n)) => *n,
        Some(Value::Integer(n)) => *n as f64,
        _ => default,
    }
}

/// Add text/FontString methods to the frame methods table.
pub fn add_text_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_text_get_set_methods(lua, methods)?;
    decor::add_decor_methods(lua, methods)?;
    add_set_font_method(lua, methods)?;
    add_get_font_method(lua, methods)?;
    add_font_object_methods(lua, methods)?;
    add_font_object_extra_methods(lua, methods)?;
    add_text_color_methods(lua, methods)?;
    add_justification_methods(lua, methods)?;
    measure::add_measure_methods(lua, methods)?;
    Ok(())
}

/// SetText, GetText, SetFormattedText.
fn add_text_get_set_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetText", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        handle_set_text(lua, id, args)
    })?)?;

    // GetText() - for FontString widgets
    methods.set("GetText", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let text = state
            .widgets
            .get(id)
            .and_then(|f| f.text.clone())
            .unwrap_or_default();
        Ok(text)
    })?)?;

    // SetFormattedText(format, ...) - for FontString widgets (like string.format + SetText)
    // Auto-sizes the FontString to fit the text content
    methods.set("SetFormattedText", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        // Use Lua's string.format to format the text
        let string_table: mlua::Table = lua.globals().get("string")?;
        let format_func: mlua::Function = string_table.get("format")?;
        if let Ok(Value::String(result)) = format_func.call::<Value>(args) {
            let text = result.to_string_lossy().to_string();
            let state_rc = get_sim_state(lua);
            {
                let mut state = state_rc.borrow_mut();
                set_text_on_frame(&mut state, id, Some(text));
            }
            auto_size_fontstring(lua, &state_rc, id);
        }
        Ok(())
    })?)?;

    Ok(())
}

/// Auto-size a FontString's width to match its text content.
///
/// Skips if the FontString has word-wrap with an explicit width constraint.
fn auto_size_fontstring(
    lua: &Lua,
    state_rc: &std::rc::Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    id: u64,
) {
    let measure_info = {
        let state = state_rc.borrow();
        state.widgets.get(id).and_then(|f| {
            if f.widget_type != WidgetType::FontString { return None; }
            if f.word_wrap && f.width > 0.0 { return None; }
            let text = f.text.as_ref()?.clone();
            Some((text, f.font.clone(), f.font_size))
        })
    };
    if let Some((text, font, font_size)) = measure_info {
        if let Some(fs_rc) = lua.app_data_ref::<std::rc::Rc<std::cell::RefCell<crate::render::font::WowFontSystem>>>() {
            let mut fs = fs_rc.borrow_mut();
            let width = fs.measure_text_width(&text, font.as_deref(), font_size);
            let mut state = state_rc.borrow_mut();
            let changed = state.widgets.get(id).map(|f| f.width != width).unwrap_or(false);
            if changed {
                if let Some(frame) = state.widgets.get_mut_visual(id) {
                    frame.width = width;
                }
            }
        }
    }
}

/// SetText(text [, r, g, b, wrap]) - universal handler for all widget types.
/// Tooltip: clears lines and sets first line with optional color/wrap.
/// SimpleHTML: strips HTML tags before storing.
/// Button: propagates text to the child Text FontString.
/// FontString: auto-sizes height and width to fit content.
fn handle_set_text(lua: &Lua, id: u64, args: mlua::MultiValue) -> mlua::Result<()> {
    let mut args_iter = args.into_iter();
    let text_str = match args_iter.next() {
        Some(mlua::Value::String(s)) => Some(s.to_string_lossy().to_string()),
        Some(mlua::Value::Integer(n)) => Some(n.to_string()),
        Some(mlua::Value::Number(n)) => Some(n.to_string()),
        _ => None,
    };

    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();

    if let Some(ref text) = text_str {
        update_tooltip_line(&mut state, id, text, &mut args_iter);
    }

    let (text_child_id, is_html) = {
        let f = state.widgets.get(id);
        let child = f.and_then(|f| f.children_keys.get("Text").copied());
        let html = state.simple_htmls.contains_key(&id);
        (child, html)
    };

    let store_text = text_str.map(|t| {
        if is_html {
            super::widget_tooltip::strip_html_tags(&t)
        } else {
            t
        }
    });

    set_text_on_frame(&mut state, id, store_text.clone());

    // For Buttons, also set text on the Text fontstring child
    if let Some(text_id) = text_child_id {
        set_text_on_frame(&mut state, text_id, store_text);
    }

    let ids_to_measure = collect_fontstring_measure_ids(&state, id, text_child_id);
    drop(state);

    measure_and_apply_widths(lua, &state_rc, &ids_to_measure);
    Ok(())
}

/// Collect FontString IDs that need width measurement after text changes.
fn collect_fontstring_measure_ids(
    state: &std::cell::RefMut<'_, crate::lua_api::SimState>,
    id: u64,
    text_child_id: Option<u64>,
) -> Vec<(u64, String, Option<String>, f32)> {
    [Some(id), text_child_id]
        .into_iter()
        .flatten()
        .filter_map(|fid| {
            let f = state.widgets.get(fid)?;
            if f.widget_type != WidgetType::FontString { return None; }
            if f.word_wrap && f.width > 0.0 { return None; }
            let text = f.text.as_ref()?.clone();
            Some((fid, text, f.font.clone(), f.font_size))
        })
        .collect()
}

/// Measure text widths and apply to frames that changed.
fn measure_and_apply_widths(
    lua: &Lua,
    state_rc: &std::rc::Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    ids_to_measure: &[(u64, String, Option<String>, f32)],
) {
    if ids_to_measure.is_empty() { return; }
    if let Some(fs_rc) = lua.app_data_ref::<std::rc::Rc<std::cell::RefCell<crate::render::font::WowFontSystem>>>() {
        let mut fs = fs_rc.borrow_mut();
        let mut state = state_rc.borrow_mut();
        for (fid, text, font, font_size) in ids_to_measure {
            let width = fs.measure_text_width(text, font.as_deref(), *font_size);
            let changed = state.widgets.get(*fid).map(|f| f.width != width).unwrap_or(false);
            if changed {
                if let Some(frame) = state.widgets.get_mut_visual(*fid) {
                    frame.width = width;
                }
            }
        }
    }
}

/// Update tooltip line data with optional r, g, b, wrap args.
fn update_tooltip_line(
    state: &mut std::cell::RefMut<'_, crate::lua_api::SimState>,
    id: u64,
    text: &str,
    args_iter: &mut std::collections::vec_deque::IntoIter<mlua::Value>,
) {
    if let Some(td) = state.tooltips.get_mut(&id) {
        let r = val_to_f32(args_iter.next().as_ref(), 1.0);
        let g = val_to_f32(args_iter.next().as_ref(), 1.0);
        let b = val_to_f32(args_iter.next().as_ref(), 1.0);
        let wrap = matches!(args_iter.next(), Some(mlua::Value::Boolean(true)));
        td.lines.clear();
        td.lines.push(crate::lua_api::tooltip::TooltipLine {
            left_text: text.to_string(),
            left_color: (r, g, b),
            right_text: None,
            right_color: (1.0, 1.0, 1.0),
            wrap,
        });
    }
}

/// Set text on a frame, auto-sizing height if needed.
///
/// FontStrings auto-size their height to fit text content, matching WoW
/// behavior where GetHeight() returns the rendered text height regardless
/// of any XML Size element.
fn set_text_on_frame(
    state: &mut std::cell::RefMut<'_, crate::lua_api::SimState>,
    id: u64,
    text: Option<String>,
) {
    // Skip get_mut() (and render_dirty) when text is unchanged
    if let Some(frame) = state.widgets.get(id) {
        let needs_height = text.is_some()
            && frame.widget_type == crate::widget::WidgetType::FontString
            && frame.height < frame.font_size.max(12.0);
        if frame.text == text && !needs_height {
            return;
        }
    }
    if let Some(frame) = state.widgets.get_mut_visual(id) {
        let min_height = frame.font_size.max(12.0);
        let is_fontstring = frame.widget_type == crate::widget::WidgetType::FontString;
        if text.is_some() && is_fontstring && frame.height < min_height {
            frame.height = min_height;
        }
        frame.text = text;
    }
}

/// SetFont([textType,] font, size, flags).
fn add_set_font_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetFont", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(lua, id);

        // Check for SimpleHTML per-textType call
        if is_html && args_vec.len() >= 2
            && let (Some(Value::String(s1)), Some(Value::String(s2))) = (args_vec.first(), args_vec.get(1)) {
                let type_str = s1.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    return set_font_for_text_type(lua, id, &type_str, s2, &args_vec);
                }
            }

        // Standard FontString path
        let font = match args_vec.first() {
            Some(Value::String(s)) => s.to_string_lossy().to_string(),
            _ => return Ok(true),
        };
        let size = match args_vec.get(1) {
            Some(Value::Number(n)) => Some(*n as f32),
            Some(Value::Integer(n)) => Some(*n as f32),
            _ => None,
        };
        let flags = match args_vec.get(2) {
            Some(Value::String(s)) => Some(s.to_string_lossy().to_string()),
            _ => None,
        };
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.font = Some(font);
            if let Some(s) = size {
                frame.font_size = s;
            }
            if let Some(ref f) = flags {
                frame.font_outline = crate::widget::TextOutline::from_wow_str(f);
            }
        }
        Ok(true)
    })?)?;
    Ok(())
}

/// Handle SetFont for a SimpleHTML per-textType call.
fn set_font_for_text_type(
    lua: &Lua,
    id: u64,
    type_str: &str,
    font_str: &mlua::String,
    args_vec: &[Value],
) -> mlua::Result<bool> {
    let font_path = font_str.to_string_lossy().to_string();
    let size = match args_vec.get(2) {
        Some(Value::Number(n)) => Some(*n as f32),
        Some(Value::Integer(n)) => Some(*n as f32),
        _ => None,
    };
    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    if let Some(data) = state.simple_htmls.get_mut(&id) {
        let style = data.text_styles.entry(type_str.to_string()).or_insert_with(TextStyle::default);
        style.font = Some(font_path);
        if let Some(s) = size {
            style.font_size = s;
        }
    }
    Ok(true)
}

/// GetFont([textType]).
fn add_get_font_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetFont", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();

        // Check for SimpleHTML per-textType call
        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                return get_font_for_text_type(lua, id, &type_str);
            }
        }

        // Standard path
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let frame = state.widgets.get(id);
        let font_path = frame
            .and_then(|f| f.font.as_deref())
            .unwrap_or("Fonts\\FRIZQT__.TTF");
        let font_size = frame.map(|f| f.font_size).unwrap_or(12.0);
        let flags = frame
            .map(|f| match f.font_outline {
                crate::widget::TextOutline::None => "",
                crate::widget::TextOutline::Outline => "OUTLINE",
                crate::widget::TextOutline::ThickOutline => "THICKOUTLINE",
            })
            .unwrap_or("");
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string(font_path)?),
            Value::Number(font_size as f64),
            Value::String(lua.create_string(flags)?),
        ]))
    })?)?;
    Ok(())
}

/// Handle GetFont for a SimpleHTML per-textType call.
fn get_font_for_text_type(
    lua: &Lua,
    id: u64,
    type_str: &str,
) -> mlua::Result<mlua::MultiValue> {
    let state_rc = get_sim_state(lua);
    let state = state_rc.borrow();
    if let Some(data) = state.simple_htmls.get(&id)
        && let Some(style) = data.text_styles.get(type_str) {
            let font = style.font.as_deref().unwrap_or("Fonts\\FRIZQT__.TTF");
            return Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string(font)?),
                Value::Number(style.font_size as f64),
                Value::String(lua.create_string("")?),
            ]));
        }
    Ok(mlua::MultiValue::from_vec(vec![
        Value::String(lua.create_string("Fonts\\FRIZQT__.TTF")?),
        Value::Number(12.0),
        Value::String(lua.create_string("")?),
    ]))
}

/// SetFontObject, GetFontObject.
fn add_font_object_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // SetFontObject([textType,] fontObject or fontName) - copy font properties from a font object
    methods.set("SetFontObject", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(lua, id);

        // Check for SimpleHTML per-textType call
        if is_html && args_vec.len() >= 2
            && let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    return set_font_object_for_text_type(lua, id, &type_str, &args_vec);
                }
            }

        // Standard path
        let font_object = args_vec.into_iter().next().unwrap_or(Value::Nil);
        let font_table = resolve_font_table(lua, &font_object);
        apply_font_table_to_frame(lua, id, font_table.as_ref());

        let store: mlua::Table = lua
            .load(
                "_G.__fontstring_font_objects = _G.__fontstring_font_objects or {}; return _G.__fontstring_font_objects",
            )
            .eval()?;
        store.set(id, font_object)?;

        Ok(())
    })?)?;

    // GetFontObject([textType]) - return the font object set via SetFontObject
    methods.set("GetFontObject", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let store: mlua::Table =
                    lua.load("return _G.__fontstring_font_objects or {}").eval()?;
                let key = format!("{}_{}", id, type_str);
                let font: Value = store.get(key)?;
                return Ok(font);
            }
        }

        let store: mlua::Table =
            lua.load("return _G.__fontstring_font_objects or {}").eval()?;
        let font: Value = store.get(id)?;
        Ok(font)
    })?)?;

    Ok(())
}

/// GetFontObjectForAlphabet, SetFontObjectsToTry, GetNumLines.
fn add_font_object_extra_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // GetFontObjectForAlphabet(alphabet) - returns self for font localization
    methods.set("GetFontObjectForAlphabet", lua.create_function(
        |_lua, (ud, _alphabet): (LightUserData, Option<String>)| {
            let id = lud_to_id(ud);
            Ok(frame_lud(id))
        },
    )?)?;

    // SetFontObjectsToTry(fontObject1, fontObject2, ...) - set fallback font objects
    methods.set("SetFontObjectsToTry", lua.create_function(
        |lua, (ud, args): (LightUserData, mlua::MultiValue)| {
            let id = lud_to_id(ud);
            if let Some(first) = args.into_iter().next() {
                let font_table = resolve_font_table(lua, &first);
                apply_font_table_to_frame(lua, id, font_table.as_ref());
            }
            Ok(())
        },
    )?)?;

    // GetNumLines() - return number of visible text lines
    methods.set("GetNumLines", lua.create_function(
        |_lua, _ud: LightUserData| Ok(1_i32),
    )?)?;

    Ok(())
}

/// Handle SetFontObject for a SimpleHTML per-textType call.
fn set_font_object_for_text_type(
    lua: &Lua,
    id: u64,
    type_str: &str,
    args_vec: &[Value],
) -> mlua::Result<()> {
    let font_name = match args_vec.get(1) {
        Some(Value::String(n)) => Some(n.to_string_lossy().to_string()),
        Some(Value::Table(t)) => t.get::<Option<String>>("__fontPath").ok().flatten(),
        _ => None,
    };
    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    if let Some(data) = state.simple_htmls.get_mut(&id) {
        let style = data.text_styles.entry(type_str.to_string()).or_insert_with(TextStyle::default);
        style.font_object = font_name;
    }
    drop(state);
    let store: mlua::Table = lua
        .load("_G.__fontstring_font_objects = _G.__fontstring_font_objects or {}; return _G.__fontstring_font_objects")
        .eval()?;
    let key = format!("{}_{}", id, type_str);
    if let Some(fo) = args_vec.get(1).cloned() {
        store.set(key, fo)?;
    }
    Ok(())
}

/// Resolve a font object Value (table or name string) into an optional Table.
fn resolve_font_table(lua: &Lua, font_object: &Value) -> Option<mlua::Table> {
    match font_object {
        Value::Table(t) => Some(t.clone()),
        Value::String(name) => {
            let name_str = name.to_string_lossy().to_string();
            lua.globals()
                .get::<Option<mlua::Table>>(name_str)
                .ok()
                .flatten()
        }
        _ => None,
    }
}

/// Apply font properties from a Lua font table to the Rust frame.
///
/// Supports two naming conventions:
/// - XML Font objects: `__font`, `__height`, `__outline`
/// - Lua-created font objects: `__fontPath`, `__fontHeight`, `__fontFlags`
fn apply_font_table_to_frame(lua: &Lua, id: u64, font_table: Option<&mlua::Table>) {
    let Some(src) = font_table else { return };
    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    let Some(frame) = state.widgets.get_mut_visual(id) else { return };

    if let Ok(path) = src.get::<String>("__fontPath").or_else(|_| src.get::<String>("__font")) {
        frame.font = Some(path);
    }
    if let Ok(height) = src.get::<f64>("__fontHeight").or_else(|_| src.get::<f64>("__height")) {
        frame.font_size = height as f32;
    }
    if let Ok(flags) = src.get::<String>("__fontFlags").or_else(|_| src.get::<String>("__outline")) {
        frame.font_outline = crate::widget::TextOutline::from_wow_str(&flags);
    }
    apply_font_table_colors(src, frame);
}

/// Apply color and alignment properties from a font table to a frame.
fn apply_font_table_colors(src: &mlua::Table, frame: &mut crate::widget::Frame) {
    if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
        src.get::<f64>("__textColorR"),
        src.get::<f64>("__textColorG"),
        src.get::<f64>("__textColorB"),
        src.get::<f64>("__textColorA"),
    ) {
        frame.text_color = crate::widget::Color::new(r as f32, g as f32, b as f32, a as f32);
    }
    if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
        src.get::<f64>("__shadowColorR"),
        src.get::<f64>("__shadowColorG"),
        src.get::<f64>("__shadowColorB"),
        src.get::<f64>("__shadowColorA"),
    ) {
        frame.shadow_color = crate::widget::Color::new(r as f32, g as f32, b as f32, a as f32);
    }
    if let (Ok(x), Ok(y)) = (
        src.get::<f64>("__shadowOffsetX"),
        src.get::<f64>("__shadowOffsetY"),
    ) {
        frame.shadow_offset = (x as f32, y as f32);
    }
    if let Ok(h) = src.get::<String>("__justifyH") {
        frame.justify_h = crate::widget::TextJustify::from_wow_str(&h);
    }
    if let Ok(v) = src.get::<String>("__justifyV") {
        frame.justify_v = crate::widget::TextJustify::from_wow_str(&v);
    }
}

/// Apply SetTextColor for SimpleHTML typed text styles.
fn set_text_color_html(lua: &Lua, id: u64, args: &[Value], type_str: String) {
    let r = val_to_f32(args.get(1), 1.0);
    let g = val_to_f32(args.get(2), 1.0);
    let b = val_to_f32(args.get(3), 1.0);
    let a = val_to_f32(args.get(4), 1.0);
    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    if let Some(data) = state.simple_htmls.get_mut(&id) {
        let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
        style.text_color = (r, g, b, a);
    }
}

/// Apply SetTextColor for standard FontString/Frame widgets.
fn set_text_color_standard(lua: &Lua, id: u64, args: &[Value]) {
    let r = val_to_f32(args.first(), 1.0);
    let g = val_to_f32(args.get(1), 1.0);
    let b = val_to_f32(args.get(2), 1.0);
    let a = val_to_f32(args.get(3), 1.0);
    let new_color = crate::widget::Color::new(r, g, b, a);
    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    let unchanged = state.widgets.get(id)
        .is_some_and(|f| f.text_color == new_color);
    if !unchanged {
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.text_color = new_color;
        }
    }
}

/// SetTextColor, GetTextColor.
fn add_text_color_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetTextColor", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        if is_simple_html(lua, id)
            && let Some(Value::String(s)) = args_vec.first()
        {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                set_text_color_html(lua, id, &args_vec, type_str);
                return Ok(());
            }
        }
        set_text_color_standard(lua, id, &args_vec);
        Ok(())
    })?)?;

    // GetTextColor([textType]) - for FontString or SimpleHTML widgets
    methods.set("GetTextColor", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let state_rc = get_sim_state(lua);
                let state = state_rc.borrow();
                if let Some(data) = state.simple_htmls.get(&id)
                    && let Some(style) = data.text_styles.get(&type_str) {
                        return Ok((style.text_color.0, style.text_color.1, style.text_color.2, style.text_color.3));
                    }
                return Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32));
            }
        }

        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            Ok((
                frame.text_color.r,
                frame.text_color.g,
                frame.text_color.b,
                frame.text_color.a,
            ))
        } else {
            Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32))
        }
    })?)?;

    Ok(())
}

/// SetJustifyH, SetJustifyV.
fn add_justification_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // SetJustifyH([textType,] justify) - for FontString or SimpleHTML widgets
    methods.set("SetJustifyH", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(lua, id);

        if is_html && args_vec.len() >= 2
            && let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    if let Some(Value::String(j)) = args_vec.get(1) {
                        let justify = j.to_string_lossy().to_string();
                        let state_rc = get_sim_state(lua);
                        let mut state = state_rc.borrow_mut();
                        if let Some(data) = state.simple_htmls.get_mut(&id) {
                            let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                            style.justify_h = justify;
                        }
                    }
                    return Ok(());
                }
            }

        if let Some(Value::String(j)) = args_vec.first() {
            let justify = j.to_string_lossy().to_string();
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.justify_h = crate::widget::TextJustify::from_wow_str(&justify);
            }
        }
        Ok(())
    })?)?;

    // SetJustifyV([textType,] justify) - for FontString or SimpleHTML widgets
    methods.set("SetJustifyV", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(lua, id);

        if is_html && args_vec.len() >= 2
            && let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    if let Some(Value::String(j)) = args_vec.get(1) {
                        let justify = j.to_string_lossy().to_string();
                        let state_rc = get_sim_state(lua);
                        let mut state = state_rc.borrow_mut();
                        if let Some(data) = state.simple_htmls.get_mut(&id) {
                            let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                            style.justify_v = justify;
                        }
                    }
                    return Ok(());
                }
            }

        if let Some(Value::String(j)) = args_vec.first() {
            let justify = j.to_string_lossy().to_string();
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.justify_v = crate::widget::TextJustify::from_wow_str(&justify);
            }
        }
        Ok(())
    })?)?;

    Ok(())
}
