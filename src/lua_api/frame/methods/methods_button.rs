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

/// Extract texture path from a Lua Value.
///
/// Handles string paths, numeric file data IDs (integer or number), and string
/// representations of numeric IDs. Returns None for userdata (texture objects).
fn extract_texture_path(texture: &Value) -> Result<Option<String>, mlua::Error> {
    Ok(super::methods_helpers::resolve_file_data_id_or_path(texture))
}

/// Resolved texture info: file path and optional UV coords (from atlas lookup).
struct ResolvedTexture {
    path: String,
    tex_coords: Option<(f32, f32, f32, f32)>,
}

/// Resolve a texture string as either an atlas name or a file path.
/// WoW's SetNormalTexture/etc. accept atlas names in addition to file paths.
fn resolve_texture_string(name: &str) -> ResolvedTexture {
    if let Some(lookup) = crate::atlas::get_atlas_info(name) {
        let info = lookup.info;
        ResolvedTexture {
            path: info.file.to_string(),
            tex_coords: Some((
                info.left_tex_coord,
                info.right_tex_coord,
                info.top_tex_coord,
                info.bottom_tex_coord,
            )),
        }
    } else {
        ResolvedTexture {
            path: name.to_string(),
            tex_coords: None,
        }
    }
}

/// Apply a resolved texture (path + optional atlas UVs) to a button field and its child texture.
#[allow(clippy::type_complexity)]
fn apply_button_texture_setter(
    state: &mut crate::lua_api::SimState,
    button_id: u64,
    parent_key: &str,
    texture: &Value,
    set_button_field: fn(&mut crate::widget::Frame, Option<String>, Option<(f32, f32, f32, f32)>),
) -> Result<(), mlua::Error> {
    let path = extract_texture_path(texture)?;
    let is_userdata = matches!(texture, Value::UserData(_));
    if !is_userdata {
        let resolved = path.as_ref().map(|p| resolve_texture_string(p));
        let resolved_path = resolved.as_ref().map(|r| r.path.clone());
        let tex_coords = resolved.as_ref().and_then(|r| r.tex_coords);
        if let Some(frame) = state.widgets.get_mut_visual(button_id) {
            set_button_field(frame, resolved_path.clone(), tex_coords);
        }
        let tex_id = get_or_create_button_texture(state, button_id, parent_key);
        if let Some(tex) = state.widgets.get_mut_visual(tex_id) {
            tex.texture = resolved_path;
            tex.tex_coords = tex_coords;
            tex.atlas_tex_coords = tex_coords;
        }
    } else {
        get_or_create_button_texture(state, button_id, parent_key);
    }
    Ok(())
}

/// Set{Normal,Highlight}Texture - set texture by path, atlas name, or userdata.
fn add_texture_setter_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetNormalTexture", |_, this, texture: Value| {
        let mut state = this.state.borrow_mut();
        apply_button_texture_setter(&mut state, this.id, "NormalTexture", &texture,
            |f, path, coords| { f.normal_texture = path; f.normal_tex_coords = coords; })
    });

    methods.add_method("SetHighlightTexture", |_, this, texture: Value| {
        let mut state = this.state.borrow_mut();
        apply_button_texture_setter(&mut state, this.id, "HighlightTexture", &texture,
            |f, path, coords| { f.highlight_texture = path; f.highlight_tex_coords = coords; })
    });
}

/// Set{Pushed,Disabled}Texture - set texture by path, atlas name, or userdata.
fn add_texture_setter_methods_2<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetPushedTexture", |_, this, texture: Value| {
        let mut state = this.state.borrow_mut();
        apply_button_texture_setter(&mut state, this.id, "PushedTexture", &texture,
            |f, path, coords| { f.pushed_texture = path; f.pushed_tex_coords = coords; })
    });

    methods.add_method("SetDisabledTexture", |_, this, texture: Value| {
        let mut state = this.state.borrow_mut();
        apply_button_texture_setter(&mut state, this.id, "DisabledTexture", &texture,
            |f, path, coords| { f.disabled_texture = path; f.disabled_tex_coords = coords; })
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
    methods.add_method("SetNormalAtlas", |_, this, args: mlua::MultiValue| {
        if let Some(atlas_name) = extract_string_arg(&args) {
            let mut state = this.state.borrow_mut();
            let tex_id = get_or_create_button_texture(&mut state, this.id, "NormalTexture");
            apply_atlas_to_button(&mut state, this.id, tex_id, &atlas_name, |f, file, coords| {
                f.normal_texture = Some(file);
                f.normal_tex_coords = Some(coords);
            });
        }
        Ok(())
    });
}

fn add_pushed_atlas_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetPushedAtlas", |_, this, args: mlua::MultiValue| {
        if let Some(atlas_name) = extract_string_arg(&args) {
            let mut state = this.state.borrow_mut();
            let tex_id = get_or_create_button_texture(&mut state, this.id, "PushedTexture");
            apply_atlas_to_button(&mut state, this.id, tex_id, &atlas_name, |f, file, coords| {
                f.pushed_texture = Some(file);
                f.pushed_tex_coords = Some(coords);
            });
        }
        Ok(())
    });
}

fn add_disabled_atlas_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetDisabledAtlas", |_, this, args: mlua::MultiValue| {
        if let Some(atlas_name) = extract_string_arg(&args) {
            let mut state = this.state.borrow_mut();
            let tex_id = get_or_create_button_texture(&mut state, this.id, "DisabledTexture");
            apply_atlas_to_button(&mut state, this.id, tex_id, &atlas_name, |f, file, coords| {
                f.disabled_texture = Some(file);
                f.disabled_tex_coords = Some(coords);
            });
        }
        Ok(())
    });
}

fn add_highlight_atlas_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetHighlightAtlas", |_, this, args: mlua::MultiValue| {
        if let Some(atlas_name) = extract_string_arg(&args) {
            let mut state = this.state.borrow_mut();
            let tex_id = get_or_create_button_texture(&mut state, this.id, "HighlightTexture");
            apply_atlas_to_button(&mut state, this.id, tex_id, &atlas_name, |f, file, coords| {
                f.highlight_texture = Some(file);
                f.highlight_tex_coords = Some(coords);
            });
        }
        Ok(())
    });
}

/// Extract a string from the first argument of a MultiValue, ignoring non-strings.
fn extract_string_arg(args: &mlua::MultiValue) -> Option<String> {
    args.iter().next().and_then(|v| match v {
        Value::String(s) => Some(s.to_string_lossy().to_string()),
        _ => None,
    })
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
    // Skip if the child texture already has this atlas set
    let already_set = state.widgets.get(tex_id)
        .map(|t| t.atlas.as_deref() == Some(atlas_name))
        .unwrap_or(false);
    if already_set {
        return;
    }

    if let Some(lookup) = crate::atlas::get_atlas_info(atlas_name) {
        let tex_coords = (
            lookup.info.left_tex_coord,
            lookup.info.right_tex_coord,
            lookup.info.top_tex_coord,
            lookup.info.bottom_tex_coord,
        );
        if let Some(tex) = state.widgets.get_mut_visual(tex_id) {
            tex.atlas = Some(atlas_name.to_string());
            tex.texture = Some(lookup.info.file.to_string());
            tex.tex_coords = Some(tex_coords);
        }
        if let Some(frame) = state.widgets.get_mut_visual(button_id) {
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

        if !is_userdata
            && let Some(frame) = state.widgets.get_mut_visual(this.id) {
                frame.checked_texture = path.clone();
            }

        let tex_id = get_or_create_button_texture(&mut state, this.id, "CheckedTexture");
        if let Some(tex) = state.widgets.get_mut_visual(tex_id) {
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

            if !is_userdata
                && let Some(frame) = state.widgets.get_mut_visual(this.id) {
                    frame.disabled_checked_texture = path.clone();
                }

            let tex_id =
                get_or_create_button_texture(&mut state, this.id, "DisabledCheckedTexture");
            if let Some(tex) = state.widgets.get_mut_visual(tex_id) {
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
        if let Some(frame) = state.widgets.get_mut_visual(this.id) {
            frame.left_texture = path;
        }
        Ok(())
    });

    methods.add_method("SetMiddleTexture", |_, this, texture: Value| {
        let path = value_to_optional_path(texture)?;
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(this.id) {
            frame.middle_texture = path;
        }
        Ok(())
    });

    methods.add_method("SetRightTexture", |_, this, texture: Value| {
        let path = value_to_optional_path(texture)?;
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(this.id) {
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
///
/// Mixins (e.g. ScrollingFontMixin) can override GetFontString with a Lua function.
/// Since mlua's add_method takes priority over __index, we check for Lua overrides
/// in __frame_fields before using the default (same pattern as GetAtlas).
fn add_font_string_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetFontString", |lua, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id)
            && let Some(&text_id) = frame.children_keys.get("Text") {
                drop(state);
                let handle = FrameHandle {
                    id: text_id,
                    state: Rc::clone(&this.state),
                };
                return lua.create_userdata(handle).map(Value::UserData);
            }
        drop(state);
        if let Some((func, ud)) = super::methods_helpers::get_mixin_override(lua, this.id, "GetFontString") {
            return func.call::<Value>(ud).map(Ok)?;
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

    methods.add_method("IsEnabled", |lua, this, ()| {
        // Check for Lua mixin override first (e.g. ConsolidatedBuffsMixin:IsEnabled)
        if let Some((func, ud)) = super::methods_helpers::get_mixin_override(lua, this.id, "IsEnabled") {
            return func.call::<Value>(ud);
        }
        let state = this.state.borrow();
        let enabled = state
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
            .unwrap_or(true);
        Ok(Value::Boolean(enabled))
    });
}

/// Update the `__enabled` attribute on a widget.
fn set_enabled_attribute(state: &mut crate::lua_api::SimState, id: u64, enabled: bool) {
    // Skip if the value is already set to avoid dirtying the frame on no-op calls
    // (e.g. LeaveInstanceGroupButton calls SetEnabled every OnUpdate tick).
    if let Some(frame) = state.widgets.get(id) {
        if let Some(crate::widget::AttributeValue::Boolean(cur)) = frame.attributes.get("__enabled") {
            if *cur == enabled {
                return;
            }
        }
    }
    if let Some(frame) = state.widgets.get_mut_visual(id) {
        frame.attributes.insert(
            "__enabled".to_string(),
            crate::widget::AttributeValue::Boolean(enabled),
        );
    }
}

/// Click, RegisterForClicks - click simulation and registration.
fn add_click_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("Click", |lua, this, ()| {
        if let Some(handler) = crate::lua_api::script_helpers::get_script(lua, this.id, "OnClick") {
            let frame_key = format!("__frame_{}", this.id);
            let frame: Value = lua.globals().get(frame_key.as_str())?;
            let button = lua.create_string("LeftButton")?;
            handler.call::<()>((frame, button, false))?;
        }
        Ok(())
    });

    methods.add_method("RegisterForClicks", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
}

/// SetButtonState / GetButtonState - control button visual state from Lua.
fn add_button_state_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "SetButtonState",
        |_, this, (state_str, _locked): (String, Option<bool>)| {
            let val = if state_str.eq_ignore_ascii_case("PUSHED") { 1 } else { 0 };
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(this.id) {
                frame.button_state = val;
            }
            Ok(())
        },
    );
    methods.add_method("GetButtonState", |_, this, ()| {
        let state = this.state.borrow();
        let val = state.widgets.get(this.id).map(|f| f.button_state).unwrap_or(0);
        Ok(if val == 1 { "PUSHED".to_string() } else { "NORMAL".to_string() })
    });
    methods.add_method("LockHighlight", |_, _this, ()| Ok(()));
    methods.add_method("UnlockHighlight", |_, _this, ()| Ok(()));
}
