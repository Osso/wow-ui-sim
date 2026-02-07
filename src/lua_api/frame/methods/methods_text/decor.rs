//! Frame decoration text methods: title, border, portrait, shadow.

use super::super::FrameHandle;
use super::{is_simple_html, is_text_type, val_to_f32, val_to_f64};
use crate::lua_api::simple_html::TextStyle;
use mlua::{UserDataMethods, Value};

/// Add title, border, portrait, and shadow methods.
pub fn add_decor_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_title_methods(methods);
    add_border_methods(methods);
    add_portrait_methods(methods);
    add_shadow_offset_methods(methods);
    add_shadow_color_methods(methods);
}

/// SetTitle, GetTitle, SetTitleOffsets.
fn add_title_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetTitle", |_, this, title: Option<String>| {
        let mut state = this.state.borrow_mut();

        let title_text_id = state
            .widgets
            .get(this.id)
            .and_then(|f| f.children_keys.get("TitleContainer").copied())
            .and_then(|tc_id| state.widgets.get(tc_id))
            .and_then(|tc| tc.children_keys.get("TitleText").copied());

        if let Some(tt_id) = title_text_id
            && let Some(title_text) = state.widgets.get_mut(tt_id) {
                title_text.text = title.clone();
                if title_text.height == 0.0 {
                    title_text.height = title_text.font_size.max(12.0);
                }
            }

        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.title = title;
        }
        Ok(())
    });

    methods.add_method("GetTitle", |_, this, ()| {
        let state = this.state.borrow();
        let title = state
            .widgets
            .get(this.id)
            .and_then(|f| f.title.clone())
            .unwrap_or_default();
        Ok(title)
    });

    methods.add_method("SetTitleOffsets", |_, _this, _args: mlua::MultiValue| Ok(()));
}

/// SetBorder, SetBorderColor, SetBorderInsets.
fn add_border_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetBorder", |lua, this, layout_name: Option<String>| {
        if let Some(layout) = layout_name {
            let state = this.state.borrow();
            let frame_name = state
                .widgets
                .get(this.id)
                .and_then(|f| f.name.clone())
                .unwrap_or_else(|| format!("__frame_{}", this.id));
            drop(state);

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

    methods.add_method("SetBorderColor", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetBorderInsets", |_, _this, _args: mlua::MultiValue| Ok(()));
}

/// Portrait-related stubs.
fn add_portrait_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "SetPortraitTextureSizeAndOffset",
        |_, _this, _args: mlua::MultiValue| Ok(()),
    );
    methods.add_method("SetPortraitTextureRaw", |_, _this, _tex: Option<String>| Ok(()));
    methods.add_method("SetPortraitToAsset", |_, _this, _asset: mlua::Value| Ok(()));
    methods.add_method("SetPortraitToUnit", |_, _this, _unit: String| Ok(()));

    methods.add_method("SetPortraitShown", |lua, this, shown: bool| {
        let state = this.state.borrow();
        let frame_name = state
            .widgets
            .get(this.id)
            .and_then(|f| f.name.clone())
            .unwrap_or_else(|| format!("__frame_{}", this.id));
        drop(state);

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
}

/// Extract numeric RGBA values from a mixed argument list, skipping non-numbers.
fn extract_rgba(args: &[Value]) -> (f32, f32, f32, f32) {
    let values: Vec<f32> = args
        .iter()
        .filter_map(|v| match v {
            Value::Number(n) => Some(*n as f32),
            Value::Integer(n) => Some(*n as f32),
            _ => None,
        })
        .collect();
    (
        values.first().copied().unwrap_or(0.0),
        values.get(1).copied().unwrap_or(0.0),
        values.get(2).copied().unwrap_or(0.0),
        values.get(3).copied().unwrap_or(1.0),
    )
}

/// SetShadowOffset, GetShadowOffset.
fn add_shadow_offset_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetShadowOffset", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let is_html = is_simple_html(this);

        if is_html
            && let Some(Value::String(s)) = args_vec.first() {
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

        let x = val_to_f64(args_vec.first(), 0.0);
        let y = val_to_f64(args_vec.get(1), 0.0);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.shadow_offset = (x as f32, y as f32);
        }
        Ok(())
    });

    methods.add_method("GetShadowOffset", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let state = this.state.borrow();
                if let Some(data) = state.simple_htmls.get(&this.id)
                    && let Some(style) = data.text_styles.get(&type_str) {
                        return Ok((style.shadow_offset.0 as f64, style.shadow_offset.1 as f64));
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
}

/// SetShadowColor, GetShadowColor.
fn add_shadow_color_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetShadowColor", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        if is_simple_html(this)
            && let Some(Value::String(s)) = args_vec.first() {
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

        let (r, g, b, a) = extract_rgba(&args_vec);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.shadow_color = crate::widget::Color::new(r, g, b, a);
        }
        Ok(())
    });

    methods.add_method("GetShadowColor", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();

        if let Some(Value::String(s)) = args_vec.first() {
            let type_str = s.to_string_lossy().to_string();
            if is_text_type(&type_str) {
                let state = this.state.borrow();
                if let Some(data) = state.simple_htmls.get(&this.id)
                    && let Some(style) = data.text_styles.get(&type_str) {
                        return Ok((style.shadow_color.0 as f64, style.shadow_color.1 as f64,
                                   style.shadow_color.2 as f64, style.shadow_color.3 as f64));
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
}
