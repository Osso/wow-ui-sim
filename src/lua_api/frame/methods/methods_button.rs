//! Button-specific methods: SetNormalTexture, SetPushedTexture, font objects, etc.

use super::FrameHandle;
use super::methods_helpers::get_or_create_button_texture;
use mlua::{UserDataMethods, Value};
use std::rc::Rc;

/// Add button-specific methods to FrameHandle UserData.
pub fn add_button_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_font_object_methods(methods);
    add_pushed_text_offset_methods(methods);
    add_texture_getter_methods(methods);
    add_texture_setter_methods(methods);
    add_texture_setter_methods_2(methods);
    add_atlas_setter_methods(methods);
    add_checked_texture_methods(methods);
    add_three_slice_methods(methods);
    add_font_string_methods(methods);
    add_enable_disable_methods(methods);
    add_click_methods(methods);
    add_button_state_methods(methods);
}

/// Set/Get font objects for normal, highlight, and disabled states.
///
/// Stores font objects in `_G.__button_font_objects` keyed by `"{frame_id}:{state}"`.
fn add_font_object_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    for (set_name, get_name, state_key) in [
        ("SetNormalFontObject", "GetNormalFontObject", "normal"),
        ("SetHighlightFontObject", "GetHighlightFontObject", "highlight"),
        ("SetDisabledFontObject", "GetDisabledFontObject", "disabled"),
    ] {
        methods.add_method(set_name, move |lua, this, font_object: Value| {
            let store: mlua::Table = lua
                .load("_G.__button_font_objects = _G.__button_font_objects or {}; return _G.__button_font_objects")
                .eval()?;
            let key = format!("{}:{}", this.id, state_key);
            store.set(key, font_object)?;
            Ok(())
        });

        methods.add_method(get_name, move |lua, this, ()| {
            let store: mlua::Table = lua
                .load("return _G.__button_font_objects or {}")
                .eval()?;
            let key = format!("{}:{}", this.id, state_key);
            let font: Value = store.get(key)?;
            Ok(font)
        });
    }
}

/// SetPushedTextOffset / GetPushedTextOffset stubs.
fn add_pushed_text_offset_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "SetPushedTextOffset",
        |_, _this, (_x, _y): (f64, f64)| Ok(()),
    );
    methods.add_method("GetPushedTextOffset", |_, _this, ()| Ok((0.0_f64, 0.0_f64)));
}

/// Get{Normal,Highlight,Pushed,Disabled}Texture - return or create texture children.
fn add_texture_getter_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    for (method_name, parent_key) in [
        ("GetNormalTexture", "NormalTexture"),
        ("GetHighlightTexture", "HighlightTexture"),
        ("GetPushedTexture", "PushedTexture"),
        ("GetDisabledTexture", "DisabledTexture"),
    ] {
        methods.add_method(method_name, move |lua, this, ()| {
            let tex_id = get_or_create_button_texture(
                &mut this.state.borrow_mut(),
                this.id,
                parent_key,
            );
            let handle = FrameHandle {
                id: tex_id,
                state: Rc::clone(&this.state),
            };
            lua.create_userdata(handle).map(Value::UserData)
        });
    }
}

/// Extract texture path from a Lua Value (String -> Some(path), other -> None).
fn extract_texture_path(texture: &Value) -> Result<Option<String>, mlua::Error> {
    match texture {
        Value::String(s) => Ok(Some(s.to_str()?.to_string())),
        _ => Ok(None),
    }
}

/// Set{Normal,Highlight}Texture - set texture by path or userdata.
fn add_texture_setter_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetNormalTexture", |_, this, texture: Value| {
        let path = extract_texture_path(&texture)?;
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

    methods.add_method("SetHighlightTexture", |_, this, texture: Value| {
        let path = extract_texture_path(&texture)?;
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
}

/// Set{Pushed,Disabled}Texture - set texture by path or userdata.
fn add_texture_setter_methods_2<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetPushedTexture", |_, this, texture: Value| {
        let path = extract_texture_path(&texture)?;
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

    methods.add_method("SetDisabledTexture", |_, this, texture: Value| {
        let path = extract_texture_path(&texture)?;
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
}

/// Set{Normal,Pushed,Disabled,Highlight}Atlas - set button textures via atlas lookup.
fn add_atlas_setter_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_normal_atlas_method(methods);
    add_pushed_atlas_method(methods);
    add_disabled_atlas_method(methods);
    add_highlight_atlas_method(methods);
}

fn add_normal_atlas_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetNormalAtlas", |_, this, atlas_name: String| {
        let mut state = this.state.borrow_mut();
        let tex_id = get_or_create_button_texture(&mut state, this.id, "NormalTexture");
        apply_atlas_to_button(&mut state, this.id, tex_id, &atlas_name, |f, file, coords| {
            f.normal_texture = Some(file);
            f.normal_tex_coords = Some(coords);
        });
        Ok(())
    });
}

fn add_pushed_atlas_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetPushedAtlas", |_, this, atlas_name: String| {
        let mut state = this.state.borrow_mut();
        let tex_id = get_or_create_button_texture(&mut state, this.id, "PushedTexture");
        apply_atlas_to_button(&mut state, this.id, tex_id, &atlas_name, |f, file, coords| {
            f.pushed_texture = Some(file);
            f.pushed_tex_coords = Some(coords);
        });
        Ok(())
    });
}

fn add_disabled_atlas_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetDisabledAtlas", |_, this, atlas_name: String| {
        let mut state = this.state.borrow_mut();
        let tex_id = get_or_create_button_texture(&mut state, this.id, "DisabledTexture");
        apply_atlas_to_button(&mut state, this.id, tex_id, &atlas_name, |f, file, coords| {
            f.disabled_texture = Some(file);
            f.disabled_tex_coords = Some(coords);
        });
        Ok(())
    });
}

fn add_highlight_atlas_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetHighlightAtlas", |_, this, atlas_name: String| {
        let mut state = this.state.borrow_mut();
        let tex_id = get_or_create_button_texture(&mut state, this.id, "HighlightTexture");
        apply_atlas_to_button(&mut state, this.id, tex_id, &atlas_name, |f, file, coords| {
            f.highlight_texture = Some(file);
            f.highlight_tex_coords = Some(coords);
        });
        Ok(())
    });
}

/// Apply atlas info to both the child texture widget and the parent button field.
fn apply_atlas_to_button<F>(
    state: &mut std::cell::RefMut<'_, crate::lua_api::SimState>,
    button_id: u64,
    tex_id: u64,
    atlas_name: &str,
    set_button_field: F,
) where
    F: FnOnce(&mut crate::widget::Frame, String, (f32, f32, f32, f32)),
{
    if let Some(lookup) = crate::atlas::get_atlas_info(atlas_name) {
        let tex_coords = (
            lookup.info.left_tex_coord,
            lookup.info.right_tex_coord,
            lookup.info.top_tex_coord,
            lookup.info.bottom_tex_coord,
        );
        if let Some(tex) = state.widgets.get_mut(tex_id) {
            tex.atlas = Some(atlas_name.to_string());
            tex.texture = Some(lookup.info.file.to_string());
            tex.tex_coords = Some(tex_coords);
        }
        if let Some(frame) = state.widgets.get_mut(button_id) {
            set_button_field(frame, lookup.info.file.to_string(), tex_coords);
        }
    }
}

/// Set{Checked,DisabledChecked}Texture - checked state textures for CheckButton.
///
/// These textures start hidden (shown via SetChecked).
fn add_checked_texture_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetCheckedTexture", |_, this, texture: Value| {
        let path = extract_texture_path(&texture)?;
        let is_userdata = matches!(texture, Value::UserData(_));
        let mut state = this.state.borrow_mut();

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
            tex.visible = false;
        }
        Ok(())
    });

    methods.add_method(
        "SetDisabledCheckedTexture",
        |_, this, texture: Value| {
            let path = extract_texture_path(&texture)?;
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
}

/// Set{Left,Middle,Right}Texture - three-slice button cap textures.
fn add_three_slice_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetLeftTexture", |_, this, texture: Value| {
        let path = value_to_optional_path(texture)?;
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.left_texture = path;
        }
        Ok(())
    });

    methods.add_method("SetMiddleTexture", |_, this, texture: Value| {
        let path = value_to_optional_path(texture)?;
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.middle_texture = path;
        }
        Ok(())
    });

    methods.add_method("SetRightTexture", |_, this, texture: Value| {
        let path = value_to_optional_path(texture)?;
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.right_texture = path;
        }
        Ok(())
    });
}

/// Convert a Lua Value to an optional texture path string.
/// String -> Some(path), Nil or other -> None.
fn value_to_optional_path(value: Value) -> Result<Option<String>, mlua::Error> {
    match value {
        Value::String(s) => Ok(Some(s.to_str()?.to_string())),
        _ => Ok(None),
    }
}

/// GetFontString / SetFontString - access the button's Text child.
fn add_font_string_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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

    methods.add_method("SetFontString", |_, _this, _fontstring: Value| Ok(()));
}

/// SetEnabled, Enable, Disable, IsEnabled - button enabled/disabled state.
fn add_enable_disable_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetEnabled", |_, this, enabled: bool| {
        set_enabled_attribute(&mut this.state.borrow_mut(), this.id, enabled);
        Ok(())
    });

    methods.add_method("Enable", |_, this, ()| {
        set_enabled_attribute(&mut this.state.borrow_mut(), this.id, true);
        Ok(())
    });

    methods.add_method("Disable", |_, this, ()| {
        set_enabled_attribute(&mut this.state.borrow_mut(), this.id, false);
        Ok(())
    });

    methods.add_method("IsEnabled", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state
            .widgets
            .get(this.id)
            .and_then(|f| f.attributes.get("__enabled"))
            .and_then(|v| {
                if let crate::widget::AttributeValue::Boolean(b) = v {
                    Some(*b)
                } else {
                    None
                }
            })
            .unwrap_or(true))
    });
}

/// Update the `__enabled` attribute on a widget.
fn set_enabled_attribute(state: &mut crate::lua_api::SimState, id: u64, enabled: bool) {
    if let Some(frame) = state.widgets.get_mut(id) {
        frame.attributes.insert(
            "__enabled".to_string(),
            crate::widget::AttributeValue::Boolean(enabled),
        );
    }
}

/// Click, RegisterForClicks - click simulation and registration.
fn add_click_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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

    methods.add_method("RegisterForClicks", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
}

/// SetButtonState / GetButtonState stubs.
fn add_button_state_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "SetButtonState",
        |_, _this, (_state, _locked): (String, Option<bool>)| Ok(()),
    );
    methods.add_method("GetButtonState", |_, _this, ()| Ok("NORMAL".to_string()));
}
