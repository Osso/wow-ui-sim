//! Text/FontString methods: SetText, SetFont, SetJustifyH, etc.

use super::FrameHandle;
use crate::lua_api::simple_html::TextStyle;
use crate::render::font::WowFontSystem;
use crate::widget::WidgetType;
use mlua::{UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Known HTML text types for SimpleHTML per-textType methods.
fn is_text_type(s: &str) -> bool {
    matches!(s, "h1" | "h2" | "h3" | "p")
}

/// Check if a frame ID corresponds to a SimpleHTML widget.
fn is_simple_html(handle: &FrameHandle) -> bool {
    handle.state.borrow().widgets.get(handle.id)
        .is_some_and(|f| f.widget_type == WidgetType::SimpleHTML)
}

/// Add text/FontString methods to FrameHandle UserData.
pub fn add_text_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetText(text) - for FontString and Button widgets
    // Auto-sizes FontStrings to fit content; for Buttons, also sets Text child fontstring
    methods.add_method("SetText", |_, this, text: Option<String>| {
        let mut state = this.state.borrow_mut();

        // Get the Text child ID if this is a Button
        let text_child_id = state
            .widgets
            .get(this.id)
            .and_then(|f| f.children_keys.get("Text").copied());

        if let Some(frame) = state.widgets.get_mut(this.id) {
            // Auto-size height if not set (for FontStrings).
            // Width is NOT auto-sized here; the renderer measures text to get actual width.
            // This avoids centering issues from rough estimates vs actual text measurement.
            if text.is_some() && frame.height == 0.0 {
                frame.height = frame.font_size.max(12.0);
            }
            frame.text = text.clone();
        }

        // For Buttons, also set text on the Text fontstring child
        if let Some(text_id) = text_child_id {
            if let Some(text_fs) = state.widgets.get_mut(text_id) {
                if text.is_some() && text_fs.height == 0.0 {
                    text_fs.height = text_fs.font_size.max(12.0);
                }
                text_fs.text = text;
            }
        }
        Ok(())
    });

    // GetText() - for FontString widgets
    methods.add_method("GetText", |_, this, ()| {
        let state = this.state.borrow();
        let text = state
            .widgets
            .get(this.id)
            .and_then(|f| f.text.clone())
            .unwrap_or_default();
        Ok(text)
    });

    // SetFormattedText(format, ...) - for FontString widgets (like string.format + SetText)
    // Auto-sizes the FontString to fit the text content
    methods.add_method("SetFormattedText", |lua, this, args: mlua::MultiValue| {
        // Use Lua's string.format to format the text
        let string_table: mlua::Table = lua.globals().get("string")?;
        let format_func: mlua::Function = string_table.get("format")?;
        if let Ok(Value::String(result)) = format_func.call::<Value>(args) {
            let text = result.to_string_lossy().to_string();
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                // Auto-size height; width is measured by renderer
                if frame.height == 0.0 {
                    frame.height = frame.font_size.max(12.0);
                }
                frame.text = Some(text);
            }
        }
        Ok(())
    });

    // SetTitle(title) - for PortraitFrame/ButtonFrame templates
    // In WoW: self:GetTitleText():SetText(title) where GetTitleText returns self.TitleContainer.TitleText
    methods.add_method("SetTitle", |_, this, title: Option<String>| {
        let mut state = this.state.borrow_mut();

        // Find TitleContainer.TitleText and update its text
        let title_text_id = state
            .widgets
            .get(this.id)
            .and_then(|f| f.children_keys.get("TitleContainer").copied())
            .and_then(|tc_id| state.widgets.get(tc_id))
            .and_then(|tc| tc.children_keys.get("TitleText").copied());

        if let Some(tt_id) = title_text_id {
            if let Some(title_text) = state.widgets.get_mut(tt_id) {
                title_text.text = title.clone();
                // Auto-size height if not set (FontStrings need height for rendering)
                if title_text.height == 0.0 {
                    title_text.height = title_text.font_size.max(12.0);
                }
            }
        }

        // Also store on frame itself for GetTitle
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.title = title;
        }
        Ok(())
    });

    // GetTitle() - for DefaultPanelTemplate frames
    methods.add_method("GetTitle", |_, this, ()| {
        let state = this.state.borrow();
        let title = state
            .widgets
            .get(this.id)
            .and_then(|f| f.title.clone())
            .unwrap_or_default();
        Ok(title)
    });

    // SetTitleOffsets(x, y) - set title position offset
    methods.add_method("SetTitleOffsets", |_, _this, (_x, _y): (f64, f64)| {
        // Stub - title offsets are a rendering detail
        Ok(())
    });

    // SetBorder(layoutName) - set nine-slice border layout (for PortraitFrameMixin)
    // This is called by ButtonFrameTemplate_HidePortrait to switch to a non-portrait layout
    methods.add_method("SetBorder", |lua, this, layout_name: Option<String>| {
        if let Some(layout) = layout_name {
            // Get the frame name from state
            let state = this.state.borrow();
            let frame_name = state
                .widgets
                .get(this.id)
                .and_then(|f| f.name.clone())
                .unwrap_or_else(|| format!("__frame_{}", this.id));
            drop(state);

            // Try to call NineSliceUtil.ApplyLayout if it exists
            // This will update the NineSlice child's textures
            let code = format!(
                r#"
                local frame = {0}
                local layoutName = "{1}"
                if frame and frame.NineSlice then
                    if NineSliceUtil and NineSliceUtil.ApplyLayout and NineSliceUtil.GetLayout then
                        local layoutTable = NineSliceUtil.GetLayout(layoutName)
                        if layoutTable then
                            NineSliceUtil.ApplyLayout(frame.NineSlice, layoutTable)
                        end
                    else
                        -- Fallback: If layout is NoPortrait variant, update corners directly
                        local ns = frame.NineSlice
                        if layoutName:find("NoPortrait") and ns.TopLeftCorner then
                            -- Switch from portrait corner to regular corner
                            local atlas = ns.TopLeftCorner:GetAtlas()
                            if atlas and atlas:find("Portrait") then
                                local newAtlas = atlas:gsub("Portrait", "")
                                ns.TopLeftCorner:SetAtlas(newAtlas, true)
                            end
                        end
                    end
                end
                "#,
                frame_name, layout
            );
            if let Err(e) = lua.load(&code).exec() {
                eprintln!("SetBorder Lua error: {}", e);
            }
        }
        Ok(())
    });

    // SetBorderColor(r, g, b, a) - set border color
    methods.add_method("SetBorderColor", |_, _this, _args: mlua::MultiValue| Ok(()));

    // SetBorderInsets(left, right, top, bottom) - set border insets
    methods.add_method("SetBorderInsets", |_, _this, _args: mlua::MultiValue| Ok(()));

    // SetPortraitTextureSizeAndOffset(size, x, y) - for portrait frames
    methods.add_method(
        "SetPortraitTextureSizeAndOffset",
        |_, _this, _args: mlua::MultiValue| Ok(()),
    );

    // SetPortraitTextureRaw(tex) - set portrait texture
    methods.add_method("SetPortraitTextureRaw", |_, _this, _tex: Option<String>| {
        Ok(())
    });

    // SetPortraitToAsset(fileID) - set portrait from file ID
    methods.add_method("SetPortraitToAsset", |_, _this, _file_id: i32| Ok(()));

    // SetPortraitToUnit(unit) - set portrait from unit
    methods.add_method("SetPortraitToUnit", |_, _this, _unit: String| Ok(()));

    // SetPortraitShown(shown) - show/hide the portrait container
    // Called by ButtonFrameTemplate_HidePortrait to hide the portrait area
    methods.add_method("SetPortraitShown", |lua, this, shown: bool| {
        // Get the frame name from state
        let state = this.state.borrow();
        let frame_name = state
            .widgets
            .get(this.id)
            .and_then(|f| f.name.clone())
            .unwrap_or_else(|| format!("__frame_{}", this.id));
        drop(state);

        // Hide/show the PortraitContainer child frame
        let code = format!(
            r#"
            local frame = {}
            if frame and frame.PortraitContainer then
                if {} then
                    frame.PortraitContainer:Show()
                else
                    frame.PortraitContainer:Hide()
                end
            end
            "#,
            frame_name,
            if shown { "true" } else { "false" }
        );
        let _ = lua.load(&code).exec();
        Ok(())
    });

    // SetShadowOffset([textType,] x, y) - set shadow offset
    methods.add_method("SetShadowOffset", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        if is_html {
            if let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    let x = val_to_f64(args_vec.get(1), 0.0);
                    let y = val_to_f64(args_vec.get(2), 0.0);
                    let mut state = this.state.borrow_mut();
                    if let Some(data) = state.simple_htmls.get_mut(&this.id) {
                        let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                        style.shadow_offset = (x as f32, y as f32);
                    }
                    return Ok(());
                }
            }
        }

        let x = val_to_f64(args_vec.first(), 0.0);
        let y = val_to_f64(args_vec.get(1), 0.0);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.shadow_offset = (x as f32, y as f32);
        }
        Ok(())
    });

    // GetShadowOffset([textType]) - get shadow offset
    methods.add_method("GetShadowOffset", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let state = this.state.borrow();
                if let Some(data) = state.simple_htmls.get(&this.id) {
                    if let Some(style) = data.text_styles.get(&type_str) {
                        return Ok((style.shadow_offset.0 as f64, style.shadow_offset.1 as f64));
                    }
                }
                return Ok((0.0_f64, 0.0_f64));
            }
        }

        let state = this.state.borrow();
        let (x, y) = state
            .widgets
            .get(this.id)
            .map(|f| f.shadow_offset)
            .unwrap_or((0.0, 0.0));
        Ok((x as f64, y as f64))
    });

    // SetShadowColor([textType,] r, g, b, a) - set shadow color
    methods.add_method("SetShadowColor", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        if is_html {
            if let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    let r = val_to_f32(args_vec.get(1), 0.0);
                    let g = val_to_f32(args_vec.get(2), 0.0);
                    let b = val_to_f32(args_vec.get(3), 0.0);
                    let a = val_to_f32(args_vec.get(4), 1.0);
                    let mut state = this.state.borrow_mut();
                    if let Some(data) = state.simple_htmls.get_mut(&this.id) {
                        let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                        style.shadow_color = (r, g, b, a);
                    }
                    return Ok(());
                }
            }
        }

        let values: Vec<f32> = args_vec
            .iter()
            .filter_map(|v| match v {
                Value::Number(n) => Some(*n as f32),
                Value::Integer(n) => Some(*n as f32),
                _ => None,
            })
            .collect();
        let r = values.first().copied().unwrap_or(0.0);
        let g = values.get(1).copied().unwrap_or(0.0);
        let b = values.get(2).copied().unwrap_or(0.0);
        let a = values.get(3).copied().unwrap_or(1.0);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.shadow_color = crate::widget::Color::new(r, g, b, a);
        }
        Ok(())
    });

    // GetShadowColor([textType]) - get shadow color
    methods.add_method("GetShadowColor", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let state = this.state.borrow();
                if let Some(data) = state.simple_htmls.get(&this.id) {
                    if let Some(style) = data.text_styles.get(&type_str) {
                        return Ok((style.shadow_color.0 as f64, style.shadow_color.1 as f64,
                                   style.shadow_color.2 as f64, style.shadow_color.3 as f64));
                    }
                }
                return Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64));
            }
        }

        let state = this.state.borrow();
        let color = state
            .widgets
            .get(this.id)
            .map(|f| f.shadow_color)
            .unwrap_or(crate::widget::Color::new(0.0, 0.0, 0.0, 0.0));
        Ok((color.r as f64, color.g as f64, color.b as f64, color.a as f64))
    });

    // SetFont([textType,] font, size, flags) - for FontString or SimpleHTML widgets
    // SimpleHTML: SetFont("h1", "Fonts\\FRIZQT__.TTF", 16, "OUTLINE")
    // Others: SetFont("Fonts\\FRIZQT__.TTF", 16, "OUTLINE")
    methods.add_method("SetFont", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        // Check for SimpleHTML per-textType call: first arg is textType, second is font path
        if is_html && args_vec.len() >= 2 {
            if let (Some(Value::String(s1)), Some(Value::String(s2))) = (args_vec.first(), args_vec.get(1)) {
                let type_str = s1.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    let font_path = s2.to_string_lossy().to_string();
                    let size = match args_vec.get(2) {
                        Some(Value::Number(n)) => Some(*n as f32),
                        Some(Value::Integer(n)) => Some(*n as f32),
                        _ => None,
                    };
                    let flags = match args_vec.get(3) {
                        Some(Value::String(s)) => Some(s.to_string_lossy().to_string()),
                        _ => None,
                    };
                    let mut state = this.state.borrow_mut();
                    if let Some(data) = state.simple_htmls.get_mut(&this.id) {
                        let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                        style.font = Some(font_path);
                        if let Some(s) = size {
                            style.font_size = s;
                        }
                        let _ = flags; // flags stored on frame level, not per-style for now
                    }
                    return Ok(true);
                }
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
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.font = Some(font);
            if let Some(s) = size {
                frame.font_size = s;
            }
            if let Some(ref f) = flags {
                frame.font_outline = crate::widget::TextOutline::from_wow_str(f);
            }
        }
        Ok(true)
    });

    // SetTextColor([textType,] r, g, b, a) - for FontString or SimpleHTML widgets
    // SimpleHTML: SetTextColor("h1", r, g, b, a) stores per-textType color
    // Others: SetTextColor(r, g, b, a)
    methods.add_method("SetTextColor", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        // Check if first arg is a text type string (SimpleHTML per-textType call)
        if is_html {
            if let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    let r = val_to_f32(args_vec.get(1), 1.0);
                    let g = val_to_f32(args_vec.get(2), 1.0);
                    let b = val_to_f32(args_vec.get(3), 1.0);
                    let a = val_to_f32(args_vec.get(4), 1.0);
                    let mut state = this.state.borrow_mut();
                    if let Some(data) = state.simple_htmls.get_mut(&this.id) {
                        let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                        style.text_color = (r, g, b, a);
                    }
                    return Ok(());
                }
            }
        }

        // Standard FontString/Frame path
        let r = val_to_f32(args_vec.first(), 1.0);
        let g = val_to_f32(args_vec.get(1), 1.0);
        let b = val_to_f32(args_vec.get(2), 1.0);
        let a = val_to_f32(args_vec.get(3), 1.0);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.text_color = crate::widget::Color::new(r, g, b, a);
        }
        Ok(())
    });

    // GetTextColor([textType]) - for FontString or SimpleHTML widgets
    methods.add_method("GetTextColor", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        // Check for SimpleHTML per-textType call
        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let state = this.state.borrow();
                if let Some(data) = state.simple_htmls.get(&this.id) {
                    if let Some(style) = data.text_styles.get(&type_str) {
                        return Ok((style.text_color.0, style.text_color.1, style.text_color.2, style.text_color.3));
                    }
                }
                return Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32));
            }
        }

        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            Ok((
                frame.text_color.r,
                frame.text_color.g,
                frame.text_color.b,
                frame.text_color.a,
            ))
        } else {
            Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32))
        }
    });

    // SetFontObject([textType,] fontObject or fontName) - copy font properties from a font object
    // SimpleHTML: SetFontObject("h1", fontObject) stores per-textType font object
    methods.add_method("SetFontObject", |lua, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        // Check for SimpleHTML per-textType call
        if is_html && args_vec.len() >= 2 {
            if let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    let font_name = match args_vec.get(1) {
                        Some(Value::String(n)) => Some(n.to_string_lossy().to_string()),
                        Some(Value::Table(t)) => t.get::<Option<String>>("__fontPath").ok().flatten(),
                        _ => None,
                    };
                    let mut state = this.state.borrow_mut();
                    if let Some(data) = state.simple_htmls.get_mut(&this.id) {
                        let style = data.text_styles.entry(type_str.clone()).or_insert_with(TextStyle::default);
                        style.font_object = font_name;
                    }
                    drop(state);
                    let store: mlua::Table = lua
                        .load("_G.__fontstring_font_objects = _G.__fontstring_font_objects or {}; return _G.__fontstring_font_objects")
                        .eval()?;
                    let key = format!("{}_{}", this.id, type_str);
                    if let Some(fo) = args_vec.into_iter().nth(1) {
                        store.set(key, fo)?;
                    }
                    return Ok(());
                }
            }
        }

        // Standard path
        let font_object = args_vec.into_iter().next().unwrap_or(Value::Nil);
        let font_table: Option<mlua::Table> = match &font_object {
            Value::Table(t) => Some(t.clone()),
            Value::String(name) => {
                let name_str = name.to_string_lossy().to_string();
                lua.globals()
                    .get::<Option<mlua::Table>>(name_str)
                    .ok()
                    .flatten()
            }
            _ => None,
        };

        if let Some(ref src) = font_table {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                if let Ok(path) = src.get::<String>("__fontPath") {
                    frame.font = Some(path);
                }
                if let Ok(height) = src.get::<f64>("__fontHeight") {
                    frame.font_size = height as f32;
                }
                if let Ok(flags) = src.get::<String>("__fontFlags") {
                    frame.font_outline = crate::widget::TextOutline::from_wow_str(&flags);
                }
                if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                    src.get::<f64>("__textColorR"),
                    src.get::<f64>("__textColorG"),
                    src.get::<f64>("__textColorB"),
                    src.get::<f64>("__textColorA"),
                ) {
                    frame.text_color =
                        crate::widget::Color::new(r as f32, g as f32, b as f32, a as f32);
                }
                if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                    src.get::<f64>("__shadowColorR"),
                    src.get::<f64>("__shadowColorG"),
                    src.get::<f64>("__shadowColorB"),
                    src.get::<f64>("__shadowColorA"),
                ) {
                    frame.shadow_color =
                        crate::widget::Color::new(r as f32, g as f32, b as f32, a as f32);
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
        }

        let store: mlua::Table = lua
            .load(
                "_G.__fontstring_font_objects = _G.__fontstring_font_objects or {}; return _G.__fontstring_font_objects",
            )
            .eval()?;
        store.set(this.id, font_object)?;

        Ok(())
    });

    // GetFontObject([textType]) - return the font object set via SetFontObject
    methods.add_method("GetFontObject", |lua, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let store: mlua::Table =
                    lua.load("return _G.__fontstring_font_objects or {}").eval()?;
                let key = format!("{}_{}", this.id, type_str);
                let font: Value = store.get(key)?;
                return Ok(font);
            }
        }

        let store: mlua::Table =
            lua.load("return _G.__fontstring_font_objects or {}").eval()?;
        let font: Value = store.get(this.id)?;
        Ok(font)
    });

    // GetFont([textType]) - for FontString or SimpleHTML widgets
    // SimpleHTML: GetFont("h1") returns per-textType font
    methods.add_method("GetFont", |lua, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        // Check for SimpleHTML per-textType call
        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let state = this.state.borrow();
                if let Some(data) = state.simple_htmls.get(&this.id) {
                    if let Some(style) = data.text_styles.get(&type_str) {
                        let font = style.font.as_deref().unwrap_or("Fonts\\FRIZQT__.TTF");
                        return Ok(mlua::MultiValue::from_vec(vec![
                            Value::String(lua.create_string(font)?),
                            Value::Number(style.font_size as f64),
                            Value::String(lua.create_string("")?),
                        ]));
                    }
                }
                // Return defaults for unknown textType
                return Ok(mlua::MultiValue::from_vec(vec![
                    Value::String(lua.create_string("Fonts\\FRIZQT__.TTF")?),
                    Value::Number(12.0),
                    Value::String(lua.create_string("")?),
                ]));
            }
        }

        // Standard path
        let state = this.state.borrow();
        let frame = state.widgets.get(this.id);
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
    });

    // SetJustifyH([textType,] justify) - for FontString or SimpleHTML widgets
    methods.add_method("SetJustifyH", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        if is_html && args_vec.len() >= 2 {
            if let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    if let Some(Value::String(j)) = args_vec.get(1) {
                        let justify = j.to_string_lossy().to_string();
                        let mut state = this.state.borrow_mut();
                        if let Some(data) = state.simple_htmls.get_mut(&this.id) {
                            let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                            style.justify_h = justify;
                        }
                    }
                    return Ok(());
                }
            }
        }

        if let Some(Value::String(j)) = args_vec.first() {
            let justify = j.to_string_lossy().to_string();
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.justify_h = crate::widget::TextJustify::from_wow_str(&justify);
            }
        }
        Ok(())
    });

    // SetJustifyV([textType,] justify) - for FontString or SimpleHTML widgets
    methods.add_method("SetJustifyV", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        if is_html && args_vec.len() >= 2 {
            if let Some(Value::String(s)) = args_vec.first() {
                let type_str = s.to_string_lossy().to_string();
                if is_text_type(&type_str) {
                    if let Some(Value::String(j)) = args_vec.get(1) {
                        let justify = j.to_string_lossy().to_string();
                        let mut state = this.state.borrow_mut();
                        if let Some(data) = state.simple_htmls.get_mut(&this.id) {
                            let style = data.text_styles.entry(type_str).or_insert_with(TextStyle::default);
                            style.justify_v = justify;
                        }
                    }
                    return Ok(());
                }
            }
        }

        if let Some(Value::String(j)) = args_vec.first() {
            let justify = j.to_string_lossy().to_string();
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.justify_v = crate::widget::TextJustify::from_wow_str(&justify);
            }
        }
        Ok(())
    });

    // GetStringWidth() - for FontString widgets
    methods.add_method("GetStringWidth", |lua, this, ()| {
        let state = this.state.borrow();
        let (text, font_path, font_size) = match state.widgets.get(this.id) {
            Some(f) => (
                f.text.clone(),
                f.font.clone(),
                f.font_size,
            ),
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
            // Fallback approximation when no font system is available (e.g. tests)
            Ok(text.len() as f64 * 7.0)
        }
    });

    // GetTextWidth() - alias for GetStringWidth (EditBox uses this)
    methods.add_method("GetTextWidth", |lua, this, ()| {
        let state = this.state.borrow();
        let (text, font_path, font_size) = match state.widgets.get(this.id) {
            Some(f) => (
                f.text.clone(),
                f.font.clone(),
                f.font_size,
            ),
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
    });

    // GetUnboundedStringWidth() - string width without word wrap constraints
    methods.add_method("GetUnboundedStringWidth", |lua, this, ()| {
        let state = this.state.borrow();
        let (text, font_path, font_size) = match state.widgets.get(this.id) {
            Some(f) => (
                f.text.clone(),
                f.font.clone(),
                f.font_size,
            ),
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
    });

    // GetFontObjectForAlphabet(alphabet) - returns self for font localization
    // In WoW this returns different fonts for Latin/Cyrillic/etc.
    // For simulation, just return self
    methods.add_method(
        "GetFontObjectForAlphabet",
        |lua, this, _alphabet: Option<String>| {
            // Return the frame itself (as a userdata) - it's a font object
            let ud = lua.create_userdata(this.clone())?;
            Ok(ud)
        },
    );

    // GetStringHeight() - for FontString widgets
    // Returns the pixel height of the rendered text, accounting for word wrapping.
    methods.add_method("GetStringHeight", |lua, this, ()| {
        let state = this.state.borrow();
        let (text, font_path, font_size, word_wrap, width) = match state.widgets.get(this.id) {
            Some(f) => (
                f.text.clone(),
                f.font.clone(),
                f.font_size,
                f.word_wrap,
                f.width,
            ),
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

    // SetWordWrap(wrap) - for FontString widgets
    methods.add_method("SetWordWrap", |_, this, wrap: bool| {
        if let Ok(mut s) = this.state.try_borrow_mut() {
            if let Some(frame) = s.widgets.get_mut(this.id) {
                frame.word_wrap = wrap;
            }
        }
        Ok(())
    });

    // GetWordWrap() - check if word wrap is enabled
    methods.add_method("GetWordWrap", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow() {
            if let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.word_wrap);
            }
        }
        Ok(false)
    });

    // IsTruncated() - check if text is truncated (for FontString)
    methods.add_method("IsTruncated", |_, _this, ()| {
        // Return false since we don't actually render/measure text
        Ok(false)
    });

    // SetTextScale(scale) - set text scale factor
    methods.add_method("SetTextScale", |_, this, scale: f64| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.text_scale = scale;
        }
        Ok(())
    });

    // GetTextScale() - get text scale factor
    methods.add_method("GetTextScale", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            return Ok(frame.text_scale);
        }
        Ok(1.0_f64)
    });

    // CanWordWrap() - check if word wrap is supported
    methods.add_method("CanWordWrap", |_, _this, ()| Ok(true));

    // GetWrappedWidth() - get width when wrapped
    methods.add_method("GetWrappedWidth", |_, this, ()| {
        let state = this.state.borrow();
        let width = state
            .widgets
            .get(this.id)
            .map(|f| f.width)
            .unwrap_or(0.0);
        Ok(width)
    });

    // SetNonSpaceWrap(wrap) - for FontString widgets
    methods.add_method("SetNonSpaceWrap", |_, _this, _wrap: bool| Ok(()));

    // CanNonSpaceWrap() - check if non-space wrap is supported
    methods.add_method("CanNonSpaceWrap", |_, _this, ()| Ok(true));

    // SetMaxLines(maxLines) - for FontString widgets
    methods.add_method("SetMaxLines", |_, this, max_lines: i32| {
        if let Ok(mut s) = this.state.try_borrow_mut() {
            if let Some(frame) = s.widgets.get_mut(this.id) {
                frame.max_lines = max_lines.max(0) as u32;
            }
        }
        Ok(())
    });

    // GetMaxLines() - for FontString widgets
    methods.add_method("GetMaxLines", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow() {
            if let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.max_lines as i32);
            }
        }
        Ok(0i32) // 0 means unlimited
    });

    // SetIndentedWordWrap([textType,] indent) - for FontString or SimpleHTML widgets
    methods.add_method("SetIndentedWordWrap", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        if is_html && args_vec.len() >= 2 {
            if let Some(Value::String(s)) = args_vec.first() {
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
        }
        // Standard: no-op for FontString
        Ok(())
    });

    // SetSpacing([textType,] spacing) - for FontString or SimpleHTML widgets
    methods.add_method("SetSpacing", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        if is_html && args_vec.len() >= 2 {
            if let Some(Value::String(s)) = args_vec.first() {
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
        }
        // Standard: no-op for FontString
        Ok(())
    });

    // GetSpacing([textType]) - for FontString or SimpleHTML widgets
    methods.add_method("GetSpacing", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let state = this.state.borrow();
                if let Some(data) = state.simple_htmls.get(&this.id) {
                    if let Some(style) = data.text_styles.get(&type_str) {
                        return Ok(style.spacing as f64);
                    }
                }
                return Ok(0.0_f64);
            }
        }
        Ok(0.0_f64)
    });
}

/// Extract f32 from a reference to a Lua Value.
fn val_to_f32(val: Option<&Value>, default: f32) -> f32 {
    match val {
        Some(Value::Number(n)) => *n as f32,
        Some(Value::Integer(n)) => *n as f32,
        _ => default,
    }
}

/// Extract f64 from a reference to a Lua Value.
fn val_to_f64(val: Option<&Value>, default: f64) -> f64 {
    match val {
        Some(Value::Number(n)) => *n,
        Some(Value::Integer(n)) => *n as f64,
        _ => default,
    }
}
