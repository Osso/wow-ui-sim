//! Button-specific methods: SetNormalTexture, SetPushedTexture, font objects, etc.

use super::methods_helpers::get_or_create_button_texture;
use crate::lua_api::frame::handle::{frame_lud, get_sim_state, lud_to_id};
use mlua::{LightUserData, Lua, Value};

/// Add button-specific methods to the shared methods table.
pub fn add_button_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_font_object_methods(lua, methods)?;
    add_pushed_text_offset_methods(lua, methods)?;
    add_texture_getter_methods(lua, methods)?;
    add_texture_setter_methods(lua, methods)?;
    add_texture_setter_methods_2(lua, methods)?;
    add_atlas_setter_methods(lua, methods)?;
    add_checked_texture_methods(lua, methods)?;
    add_three_slice_methods(lua, methods)?;
    add_font_string_methods(lua, methods)?;
    add_enable_disable_methods(lua, methods)?;
    add_click_methods(lua, methods)?;
    add_button_state_methods(lua, methods)?;
    Ok(())
}

/// Set/Get font objects for normal, highlight, and disabled states.
///
/// Stores font objects in `_G.__button_font_objects` keyed by `"{frame_id}:{state}"`.
fn add_font_object_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    for (set_name, get_name, state_key) in [
        ("SetNormalFontObject", "GetNormalFontObject", "normal"),
        ("SetHighlightFontObject", "GetHighlightFontObject", "highlight"),
        ("SetDisabledFontObject", "GetDisabledFontObject", "disabled"),
    ] {
        methods.set(set_name, lua.create_function(move |lua, (ud, font_object): (LightUserData, Value)| {
            let id = lud_to_id(ud);
            let store: mlua::Table = lua
                .load("_G.__button_font_objects = _G.__button_font_objects or {}; return _G.__button_font_objects")
                .eval()?;
            let key = format!("{}:{}", id, state_key);
            store.set(key, font_object)?;
            Ok(())
        })?)?;

        methods.set(get_name, lua.create_function(move |lua, ud: LightUserData| {
            let id = lud_to_id(ud);
            let store: mlua::Table = lua
                .load("return _G.__button_font_objects or {}")
                .eval()?;
            let key = format!("{}:{}", id, state_key);
            let font: Value = store.get(key)?;
            Ok(font)
        })?)?;
    }
    Ok(())
}

/// SetPushedTextOffset / GetPushedTextOffset stubs.
fn add_pushed_text_offset_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetPushedTextOffset", lua.create_function(
        |_, (_ud, _x, _y): (LightUserData, f64, f64)| Ok(()),
    )?)?;
    methods.set("GetPushedTextOffset", lua.create_function(
        |_, _ud: LightUserData| Ok((0.0_f64, 0.0_f64)),
    )?)?;
    Ok(())
}

/// Get{Normal,Highlight,Pushed,Disabled}Texture - return or create texture children.
fn add_texture_getter_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    for (method_name, parent_key) in [
        ("GetNormalTexture", "NormalTexture"),
        ("GetHighlightTexture", "HighlightTexture"),
        ("GetPushedTexture", "PushedTexture"),
        ("GetDisabledTexture", "DisabledTexture"),
    ] {
        methods.set(method_name, lua.create_function(move |lua, ud: LightUserData| {
            let id = lud_to_id(ud);
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            let tex_id = get_or_create_button_texture(&mut state, id, parent_key);
            Ok(frame_lud(tex_id))
        })?)?;
    }
    Ok(())
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
    let is_userdata = matches!(texture, Value::LightUserData(_));
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
fn add_texture_setter_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetNormalTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        apply_button_texture_setter(&mut state, id, "NormalTexture", &texture,
            |f, path, coords| { f.normal_texture = path; f.normal_tex_coords = coords; })
    })?)?;

    methods.set("SetHighlightTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        apply_button_texture_setter(&mut state, id, "HighlightTexture", &texture,
            |f, path, coords| { f.highlight_texture = path; f.highlight_tex_coords = coords; })
    })?)?;

    Ok(())
}

/// Set{Pushed,Disabled}Texture - set texture by path, atlas name, or userdata.
fn add_texture_setter_methods_2(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetPushedTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        apply_button_texture_setter(&mut state, id, "PushedTexture", &texture,
            |f, path, coords| { f.pushed_texture = path; f.pushed_tex_coords = coords; })
    })?)?;

    methods.set("SetDisabledTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        apply_button_texture_setter(&mut state, id, "DisabledTexture", &texture,
            |f, path, coords| { f.disabled_texture = path; f.disabled_tex_coords = coords; })
    })?)?;

    Ok(())
}

/// Set{Normal,Pushed,Disabled,Highlight}Atlas - set button textures via atlas lookup.
fn add_atlas_setter_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_normal_atlas_method(lua, methods)?;
    add_pushed_atlas_method(lua, methods)?;
    add_disabled_atlas_method(lua, methods)?;
    add_highlight_atlas_method(lua, methods)?;
    Ok(())
}

fn add_normal_atlas_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetNormalAtlas", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        if let Some(atlas_name) = extract_string_arg(&args) {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            let id = lud_to_id(ud);
            let tex_id = get_or_create_button_texture(&mut state, id, "NormalTexture");
            apply_atlas_to_button(&mut state, id, tex_id, &atlas_name, |f, file, coords| {
                f.normal_texture = Some(file);
                f.normal_tex_coords = Some(coords);
            });
        }
        Ok(())
    })?)?;
    Ok(())
}

fn add_pushed_atlas_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetPushedAtlas", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        if let Some(atlas_name) = extract_string_arg(&args) {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            let id = lud_to_id(ud);
            let tex_id = get_or_create_button_texture(&mut state, id, "PushedTexture");
            apply_atlas_to_button(&mut state, id, tex_id, &atlas_name, |f, file, coords| {
                f.pushed_texture = Some(file);
                f.pushed_tex_coords = Some(coords);
            });
        }
        Ok(())
    })?)?;
    Ok(())
}

fn add_disabled_atlas_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetDisabledAtlas", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        if let Some(atlas_name) = extract_string_arg(&args) {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            let id = lud_to_id(ud);
            let tex_id = get_or_create_button_texture(&mut state, id, "DisabledTexture");
            apply_atlas_to_button(&mut state, id, tex_id, &atlas_name, |f, file, coords| {
                f.disabled_texture = Some(file);
                f.disabled_tex_coords = Some(coords);
            });
        }
        Ok(())
    })?)?;
    Ok(())
}

fn add_highlight_atlas_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetHighlightAtlas", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        if let Some(atlas_name) = extract_string_arg(&args) {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            let id = lud_to_id(ud);
            let tex_id = get_or_create_button_texture(&mut state, id, "HighlightTexture");
            apply_atlas_to_button(&mut state, id, tex_id, &atlas_name, |f, file, coords| {
                f.highlight_texture = Some(file);
                f.highlight_tex_coords = Some(coords);
            });
        }
        Ok(())
    })?)?;
    Ok(())
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
fn add_checked_texture_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetCheckedTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let path = extract_texture_path(&texture)?;
        let is_userdata = matches!(texture, Value::LightUserData(_));
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();

        if !is_userdata
            && let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.checked_texture = path.clone();
            }

        let tex_id = get_or_create_button_texture(&mut state, id, "CheckedTexture");
        if let Some(tex) = state.widgets.get_mut_visual(tex_id) {
            if !is_userdata {
                tex.texture = path;
            }
            tex.visible = false;
        }
        Ok(())
    })?)?;

    methods.set("SetDisabledCheckedTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let path = extract_texture_path(&texture)?;
        let is_userdata = matches!(texture, Value::LightUserData(_));
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();

        if !is_userdata
            && let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.disabled_checked_texture = path.clone();
            }

        let tex_id =
            get_or_create_button_texture(&mut state, id, "DisabledCheckedTexture");
        if let Some(tex) = state.widgets.get_mut_visual(tex_id) {
            if !is_userdata {
                tex.texture = path;
            }
            tex.visible = false;
        }
        Ok(())
    })?)?;

    Ok(())
}

/// Set{Left,Middle,Right}Texture - three-slice button cap textures.
fn add_three_slice_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetLeftTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let path = value_to_optional_path(texture)?;
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.left_texture = path;
        }
        Ok(())
    })?)?;

    methods.set("SetMiddleTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let path = value_to_optional_path(texture)?;
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.middle_texture = path;
        }
        Ok(())
    })?)?;

    methods.set("SetRightTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let path = value_to_optional_path(texture)?;
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.right_texture = path;
        }
        Ok(())
    })?)?;

    Ok(())
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
fn add_font_string_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetFontString", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id)
            && let Some(&text_id) = frame.children_keys.get("Text") {
                drop(state);
                return Ok(frame_lud(text_id));
            }
        drop(state);
        if let Some((func, ud_val)) = super::methods_helpers::get_mixin_override(lua, id, "GetFontString") {
            return func.call::<Value>(ud_val).map(Ok)?;
        }
        Ok(Value::Nil)
    })?)?;

    methods.set("SetFontString", lua.create_function(
        |_, (_ud, _fontstring): (LightUserData, Value)| Ok(()),
    )?)?;

    Ok(())
}

/// SetEnabled, Enable, Disable, IsEnabled - button enabled/disabled state.
fn add_enable_disable_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetEnabled", lua.create_function(|lua, (ud, enabled): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        set_enabled_attribute(&mut state_rc.borrow_mut(), id, enabled);
        Ok(())
    })?)?;

    methods.set("Enable", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        set_enabled_attribute(&mut state_rc.borrow_mut(), id, true);
        Ok(())
    })?)?;

    methods.set("Disable", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        set_enabled_attribute(&mut state_rc.borrow_mut(), id, false);
        Ok(())
    })?)?;

    methods.set("IsEnabled", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        // Check for Lua mixin override first (e.g. ConsolidatedBuffsMixin:IsEnabled)
        if let Some((func, ud_val)) = super::methods_helpers::get_mixin_override(lua, id, "IsEnabled") {
            return func.call::<Value>(ud_val);
        }
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let enabled = state
            .widgets
            .get(id)
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
    })?)?;

    Ok(())
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
fn add_click_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("Click", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        if let Some(handler) = crate::lua_api::script_helpers::get_script(lua, id, "OnClick") {
            let frame_val = frame_lud(id);
            let button = lua.create_string("LeftButton")?;
            handler.call::<()>((frame_val, button, false))?;
        }
        Ok(())
    })?)?;

    methods.set("RegisterForClicks", lua.create_function(
        |_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()),
    )?)?;

    Ok(())
}

/// SetButtonState / GetButtonState - control button visual state from Lua.
fn add_button_state_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetButtonState", lua.create_function(
        |lua, (ud, state_str, _locked): (LightUserData, String, Option<bool>)| {
            let id = lud_to_id(ud);
            let val = if state_str.eq_ignore_ascii_case("PUSHED") { 1 } else { 0 };
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.button_state = val;
            }
            Ok(())
        },
    )?)?;

    methods.set("GetButtonState", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let val = state.widgets.get(id).map(|f| f.button_state).unwrap_or(0);
        Ok(if val == 1 { "PUSHED".to_string() } else { "NORMAL".to_string() })
    })?)?;

    methods.set("LockHighlight", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("UnlockHighlight", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;

    Ok(())
}
