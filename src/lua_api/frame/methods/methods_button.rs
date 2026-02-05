//! Button-specific methods: SetNormalTexture, SetPushedTexture, font objects, etc.

use super::FrameHandle;
use super::methods_helpers::get_or_create_button_texture;
use mlua::{UserDataMethods, Value};
use std::rc::Rc;

/// Add button-specific methods to FrameHandle UserData.
pub fn add_button_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetNormalFontObject(fontObject) - Set font for normal state
    methods.add_method("SetNormalFontObject", |lua, this, font_object: Value| {
        // Store in global table by frame ID
        let store: mlua::Table = lua.load("_G.__button_font_objects = _G.__button_font_objects or {}; return _G.__button_font_objects").eval()?;
        let key = format!("{}:normal", this.id);
        store.set(key, font_object)?;
        Ok(())
    });

    // GetNormalFontObject() - Get font for normal state
    methods.add_method("GetNormalFontObject", |lua, this, ()| {
        let store: mlua::Table = lua.load("return _G.__button_font_objects or {}").eval()?;
        let key = format!("{}:normal", this.id);
        let font: Value = store.get(key)?;
        Ok(font)
    });

    // SetHighlightFontObject(fontObject) - Set font for highlight state
    methods.add_method("SetHighlightFontObject", |lua, this, font_object: Value| {
        let store: mlua::Table = lua.load("_G.__button_font_objects = _G.__button_font_objects or {}; return _G.__button_font_objects").eval()?;
        let key = format!("{}:highlight", this.id);
        store.set(key, font_object)?;
        Ok(())
    });

    // GetHighlightFontObject() - Get font for highlight state
    methods.add_method("GetHighlightFontObject", |lua, this, ()| {
        let store: mlua::Table = lua.load("return _G.__button_font_objects or {}").eval()?;
        let key = format!("{}:highlight", this.id);
        let font: Value = store.get(key)?;
        Ok(font)
    });

    // SetDisabledFontObject(fontObject) - Set font for disabled state
    methods.add_method("SetDisabledFontObject", |lua, this, font_object: Value| {
        let store: mlua::Table = lua.load("_G.__button_font_objects = _G.__button_font_objects or {}; return _G.__button_font_objects").eval()?;
        let key = format!("{}:disabled", this.id);
        store.set(key, font_object)?;
        Ok(())
    });

    // GetDisabledFontObject() - Get font for disabled state
    methods.add_method("GetDisabledFontObject", |lua, this, ()| {
        let store: mlua::Table = lua.load("return _G.__button_font_objects or {}").eval()?;
        let key = format!("{}:disabled", this.id);
        let font: Value = store.get(key)?;
        Ok(font)
    });

    // SetPushedTextOffset(x, y) - Set text offset when button is pushed
    methods.add_method(
        "SetPushedTextOffset",
        |_, _this, (_x, _y): (f64, f64)| Ok(()),
    );

    // GetPushedTextOffset() - Get text offset when button is pushed
    methods.add_method("GetPushedTextOffset", |_, _this, ()| Ok((0.0_f64, 0.0_f64)));

    // GetNormalTexture() - Get or create the normal state texture
    // In WoW, this returns the texture object, creating it if necessary
    // Always calls get_or_create_button_texture to ensure anchors are set
    methods.add_method("GetNormalTexture", |lua, this, ()| {
        let tex_id =
            get_or_create_button_texture(&mut this.state.borrow_mut(), this.id, "NormalTexture");
        let handle = FrameHandle {
            id: tex_id,
            state: Rc::clone(&this.state),
        };
        lua.create_userdata(handle).map(Value::UserData)
    });

    // GetHighlightTexture() - Get or create the highlight state texture
    methods.add_method("GetHighlightTexture", |lua, this, ()| {
        let tex_id = get_or_create_button_texture(
            &mut this.state.borrow_mut(),
            this.id,
            "HighlightTexture",
        );
        let handle = FrameHandle {
            id: tex_id,
            state: Rc::clone(&this.state),
        };
        lua.create_userdata(handle).map(Value::UserData)
    });

    // GetPushedTexture() - Get or create the pushed state texture
    methods.add_method("GetPushedTexture", |lua, this, ()| {
        let tex_id =
            get_or_create_button_texture(&mut this.state.borrow_mut(), this.id, "PushedTexture");
        let handle = FrameHandle {
            id: tex_id,
            state: Rc::clone(&this.state),
        };
        lua.create_userdata(handle).map(Value::UserData)
    });

    // GetDisabledTexture() - Get or create the disabled state texture
    methods.add_method("GetDisabledTexture", |lua, this, ()| {
        let tex_id =
            get_or_create_button_texture(&mut this.state.borrow_mut(), this.id, "DisabledTexture");
        let handle = FrameHandle {
            id: tex_id,
            state: Rc::clone(&this.state),
        };
        lua.create_userdata(handle).map(Value::UserData)
    });

    // SetNormalTexture(texture) - Set texture for normal state
    methods.add_method("SetNormalTexture", |_, this, texture: Value| {
        let path = match &texture {
            Value::String(s) => Some(s.to_str()?.to_string()),
            _ => None,
        };
        let is_userdata = matches!(texture, Value::UserData(_));
        let mut state = this.state.borrow_mut();

        if !is_userdata {
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.normal_texture = path.clone();
            }
        }

        let tex_id = get_or_create_button_texture(&mut state, this.id, "NormalTexture");
        if !is_userdata {
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                tex.texture = path;
            }
        }

        Ok(())
    });

    // SetHighlightTexture(texture) - Set texture for highlight state
    methods.add_method("SetHighlightTexture", |_, this, texture: Value| {
        let path = match &texture {
            Value::String(s) => Some(s.to_str()?.to_string()),
            _ => None,
        };
        let is_userdata = matches!(texture, Value::UserData(_));
        let mut state = this.state.borrow_mut();

        if !is_userdata {
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.highlight_texture = path.clone();
            }
        }

        let tex_id = get_or_create_button_texture(&mut state, this.id, "HighlightTexture");
        if !is_userdata {
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                tex.texture = path;
            }
        }

        Ok(())
    });

    // SetHighlightAtlas(atlasName) - Set highlight texture via atlas
    methods.add_method("SetHighlightAtlas", |_, this, atlas_name: String| {
        let mut state = this.state.borrow_mut();
        let tex_id = get_or_create_button_texture(&mut state, this.id, "HighlightTexture");
        if let Some(lookup) = crate::atlas::get_atlas_info(&atlas_name) {
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                tex.atlas = Some(atlas_name);
                tex.texture = Some(lookup.info.file.to_string());
            }
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.highlight_texture = Some(lookup.info.file.to_string());
            }
        }
        Ok(())
    });

    // SetPushedTexture(texture) - Set texture for pushed state
    methods.add_method("SetPushedTexture", |_, this, texture: Value| {
        let path = match &texture {
            Value::String(s) => Some(s.to_str()?.to_string()),
            _ => None,
        };
        let is_userdata = matches!(texture, Value::UserData(_));
        let mut state = this.state.borrow_mut();

        if !is_userdata {
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.pushed_texture = path.clone();
            }
        }

        let tex_id = get_or_create_button_texture(&mut state, this.id, "PushedTexture");
        if !is_userdata {
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                tex.texture = path;
            }
        }

        Ok(())
    });

    // SetDisabledTexture(texture) - Set texture for disabled state
    methods.add_method("SetDisabledTexture", |_, this, texture: Value| {
        let path = match &texture {
            Value::String(s) => Some(s.to_str()?.to_string()),
            _ => None,
        };
        let is_userdata = matches!(texture, Value::UserData(_));
        let mut state = this.state.borrow_mut();

        if !is_userdata {
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.disabled_texture = path.clone();
            }
        }

        let tex_id = get_or_create_button_texture(&mut state, this.id, "DisabledTexture");
        if !is_userdata {
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                tex.texture = path;
            }
        }

        Ok(())
    });

    // SetCheckedTexture(texture) - Set texture for checked state (CheckButton)
    methods.add_method("SetCheckedTexture", |_, this, texture: Value| {
        let path = match &texture {
            Value::String(s) => Some(s.to_str()?.to_string()),
            _ => None,
        };
        let is_userdata = matches!(texture, Value::UserData(_));
        let mut state = this.state.borrow_mut();

        // Only overwrite parent path when given a string (not a texture object)
        if !is_userdata {
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.checked_texture = path.clone();
            }
        }

        let tex_id = get_or_create_button_texture(&mut state, this.id, "CheckedTexture");
        if let Some(tex) = state.widgets.get_mut(tex_id) {
            if !is_userdata {
                tex.texture = path;
            }
            // CheckedTexture starts hidden (shown via SetChecked)
            tex.visible = false;
        }

        Ok(())
    });

    // SetDisabledCheckedTexture(texture) - Set texture for disabled-checked state (CheckButton)
    methods.add_method(
        "SetDisabledCheckedTexture",
        |_, this, texture: Value| {
            let path = match &texture {
                Value::String(s) => Some(s.to_str()?.to_string()),
                _ => None,
            };
            let is_userdata = matches!(texture, Value::UserData(_));
            let mut state = this.state.borrow_mut();

            if !is_userdata {
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.disabled_checked_texture = path.clone();
                }
            }

            let tex_id =
                get_or_create_button_texture(&mut state, this.id, "DisabledCheckedTexture");
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                if !is_userdata {
                    tex.texture = path;
                }
                tex.visible = false;
            }

            Ok(())
        },
    );

    // SetLeftTexture(texture) - Set left cap texture for 3-slice buttons
    methods.add_method("SetLeftTexture", |_, this, texture: Value| {
        let path = match texture {
            Value::String(s) => Some(s.to_str()?.to_string()),
            Value::Nil => None,
            _ => None,
        };
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.left_texture = path;
        }
        Ok(())
    });

    // SetMiddleTexture(texture) - Set middle (stretchable) texture for 3-slice buttons
    methods.add_method("SetMiddleTexture", |_, this, texture: Value| {
        let path = match texture {
            Value::String(s) => Some(s.to_str()?.to_string()),
            Value::Nil => None,
            _ => None,
        };
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.middle_texture = path;
        }
        Ok(())
    });

    // SetRightTexture(texture) - Set right cap texture for 3-slice buttons
    methods.add_method("SetRightTexture", |_, this, texture: Value| {
        let path = match texture {
            Value::String(s) => Some(s.to_str()?.to_string()),
            Value::Nil => None,
            _ => None,
        };
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.right_texture = path;
        }
        Ok(())
    });

    // GetFontString() - Get button's text font string
    methods.add_method("GetFontString", |lua, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            if let Some(&text_id) = frame.children_keys.get("Text") {
                drop(state);
                let handle = FrameHandle {
                    id: text_id,
                    state: Rc::clone(&this.state),
                };
                return lua.create_userdata(handle).map(Value::UserData);
            }
        }
        Ok(Value::Nil)
    });

    // SetFontString(fontstring) - Set button's text font string
    methods.add_method("SetFontString", |_, _this, _fontstring: Value| Ok(()));

    // SetEnabled(enabled) - Enable/disable button
    methods.add_method("SetEnabled", |_, _this, _enabled: bool| Ok(()));

    // Enable() - Enable button
    methods.add_method("Enable", |_, _this, ()| Ok(()));

    // Disable() - Disable button
    methods.add_method("Disable", |_, _this, ()| Ok(()));

    // IsEnabled() - Check if button is enabled
    methods.add_method("IsEnabled", |_, _this, ()| Ok(true));

    // Click() - Simulate button click by firing OnClick handler
    methods.add_method("Click", |lua, this, ()| {
        let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();
        if let Some(table) = scripts_table {
            let key = format!("{}_OnClick", this.id);
            let handler: Option<mlua::Function> = table.get(key.as_str()).ok();
            if let Some(handler) = handler {
                let frame_key = format!("__frame_{}", this.id);
                let frame: Value = lua.globals().get(frame_key.as_str())?;
                let button = lua.create_string("LeftButton")?;
                handler.call::<()>((frame, button, false))?;
            }
        }
        Ok(())
    });

    // RegisterForClicks(...) - Register which mouse buttons trigger clicks
    methods.add_method("RegisterForClicks", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });

    // SetButtonState(state, locked) - Set button visual state
    methods.add_method(
        "SetButtonState",
        |_, _this, (_state, _locked): (String, Option<bool>)| Ok(()),
    );

    // GetButtonState() - Get button visual state
    methods.add_method("GetButtonState", |_, _this, ()| Ok("NORMAL".to_string()));
}
