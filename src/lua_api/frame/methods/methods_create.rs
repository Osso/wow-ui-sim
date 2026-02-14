//! Child creation methods: CreateTexture, CreateFontString, CreateAnimationGroup, etc.

use crate::lua_api::animation::{AnimGroupHandle, AnimGroupState};
use crate::lua_api::frame::handle::{frame_lud, get_sim_state, lud_to_id};
use crate::widget::{Frame, WidgetType};
use mlua::{LightUserData, Lua, Value};
use std::rc::Rc;

/// Handle $parent substitution in frame names.
fn substitute_parent_name(name: &str, parent_name: Option<&str>) -> String {
    if name.contains("$parent") || name.contains("$Parent") {
        if let Some(pname) = parent_name {
            name.replace("$parent", pname).replace("$Parent", pname)
        } else {
            name.replace("$parent", "").replace("$Parent", "")
        }
    } else {
        name.to_string()
    }
}

/// Resolve a raw name with $parent substitution using the parent widget's name.
fn resolve_child_name(lua: &Lua, name_raw: Option<String>, parent_id: u64) -> Option<String> {
    name_raw.map(|n| {
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let parent_name = state.widgets.get(parent_id).and_then(|f| f.name.as_deref());
        substitute_parent_name(&n, parent_name)
    })
}

/// Extract an optional string from the first element of a MultiValue args list.
fn extract_string_arg(args: &[Value], index: usize) -> Option<String> {
    args.get(index).and_then(|v| {
        if let Value::String(s) = v {
            Some(s.to_string_lossy().to_string())
        } else {
            None
        }
    })
}

/// Register a child widget in the state and cache its LightUserData in `_G`.
///
/// Caches `frame_lud(child_id)` in `_G` via `raw_set` for named children
/// (and always for `__frame_{id}`) so that Lua lookups via `_G["name"]`
/// resolve to the correct LightUserData value.
fn register_child_widget(
    lua: &Lua,
    parent_id: u64,
    child: Frame,
    name: &Option<String>,
) -> mlua::Result<Value> {
    let child_id = child.id;

    {
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        state.widgets.register(child);
        state.widgets.add_child(parent_id, child_id);

        // Inherit strata and level from parent (regions render in parent's context)
        let parent_props = state
            .widgets
            .get(parent_id)
            .map(|p| (p.frame_strata, p.frame_level));
        if let Some((parent_strata, parent_level)) = parent_props {
            if let Some(f) = state.widgets.get_mut_visual(child_id) {
                f.frame_strata = parent_strata;
                f.frame_level = parent_level + 1;
            }
        }
    }

    let lud = frame_lud(child_id);

    // Cache in _G so Lua identity matches for named lookups
    let globals = lua.globals();
    if let Some(n) = name {
        globals.raw_set(n.as_str(), lud.clone())?;
    }
    let frame_key = format!("__frame_{}", child_id);
    globals.raw_set(frame_key.as_str(), lud.clone())?;

    Ok(lud)
}

/// Add child creation methods to the shared methods table.
pub fn add_create_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_create_texture_method(lua, methods)?;
    add_create_mask_texture_method(lua, methods)?;
    add_create_line_method(lua, methods)?;
    add_create_font_string_method(lua, methods)?;
    add_create_animation_group_method(lua, methods)?;
    Ok(())
}

/// CreateTexture(name, layer, inherits, subLevel)
fn add_create_texture_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "CreateTexture",
        lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
            use crate::widget::DrawLayer;

            let id = lud_to_id(ud);
            let args: Vec<Value> = args.into_iter().collect();
            let name_raw = extract_string_arg(&args, 0);
            let layer = extract_string_arg(&args, 1);
            let name = resolve_child_name(lua, name_raw, id);

            let mut texture = Frame::new(WidgetType::Texture, name.clone(), Some(id));

            if let Some(layer_str) = layer
                && let Some(draw_layer) = DrawLayer::from_str(&layer_str)
            {
                texture.draw_layer = draw_layer;
            }

            register_child_widget(lua, id, texture, &name)
        })?,
    )
}

/// CreateMaskTexture(layer, inherits, subLevel) - create a mask texture.
fn add_create_mask_texture_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "CreateMaskTexture",
        lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
            let id = lud_to_id(ud);
            let args: Vec<Value> = args.into_iter().collect();
            let name_raw = extract_string_arg(&args, 0);
            let name = resolve_child_name(lua, name_raw, id);
            let mut texture = Frame::new(WidgetType::Texture, name.clone(), Some(id));
            texture.is_mask = true;
            register_child_widget(lua, id, texture, &name)
        })?,
    )
}

/// CreateLine(name, layer, inherits, subLevel) - create a Line (texture with start/end points).
fn add_create_line_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "CreateLine",
        lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
            use crate::widget::DrawLayer;

            let id = lud_to_id(ud);
            let args: Vec<Value> = args.into_iter().collect();
            let name_raw = extract_string_arg(&args, 0);
            let layer = extract_string_arg(&args, 1);
            let name = resolve_child_name(lua, name_raw, id);

            let mut line = Frame::new(WidgetType::Line, name.clone(), Some(id));

            if let Some(layer_str) = layer
                && let Some(draw_layer) = DrawLayer::from_str(&layer_str)
            {
                line.draw_layer = draw_layer;
            }

            register_child_widget(lua, id, line, &name)
        })?,
    )
}

/// CreateFontString(name, layer, inherits)
fn add_create_font_string_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "CreateFontString",
        lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
            use crate::widget::DrawLayer;

            let id = lud_to_id(ud);
            let args: Vec<Value> = args.into_iter().collect();
            let name_raw = extract_string_arg(&args, 0);
            let layer = extract_string_arg(&args, 1);
            let inherits = extract_string_arg(&args, 2);
            let name = resolve_child_name(lua, name_raw, id);

            let mut fontstring = Frame::new(WidgetType::FontString, name.clone(), Some(id));

            if let Some(layer_str) = layer
                && let Some(draw_layer) = DrawLayer::from_str(&layer_str)
            {
                fontstring.draw_layer = draw_layer;
            }

            apply_font_inherit(lua, &mut fontstring, inherits.as_deref());

            register_child_widget(lua, id, fontstring, &name)
        })?,
    )
}

/// Apply font properties from an inherited Font object to a fontstring widget.
fn apply_font_inherit(lua: &Lua, frame: &mut Frame, inherits: Option<&str>) {
    let Some(name) = inherits else { return };
    let Ok(globals) = lua.globals().get::<Value>(name) else { return };
    let Value::Table(tbl) = globals else { return };
    if let Ok(path) = tbl.get::<String>("__font") {
        frame.font = Some(path);
    }
    if let Ok(height) = tbl.get::<f64>("__height") {
        frame.font_size = height as f32;
    }
    if let Ok(outline) = tbl.get::<String>("__outline") {
        frame.font_outline = crate::widget::TextOutline::from_wow_str(&outline);
    }
    if let Ok(h) = tbl.get::<String>("__justifyH") {
        frame.justify_h = crate::widget::TextJustify::from_wow_str(&h);
    }
}

/// CreateAnimationGroup(name, inherits)
fn add_create_animation_group_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "CreateAnimationGroup",
        lua.create_function(
            |lua, (ud, name, _inherits): (LightUserData, Option<String>, Option<String>)| {
                let id = lud_to_id(ud);
                let state_rc = get_sim_state(lua);
                let group_id;
                {
                    let mut state = state_rc.borrow_mut();
                    group_id = state.next_anim_group_id;
                    state.next_anim_group_id += 1;
                    let mut group = AnimGroupState::new(id);
                    group.name = name;
                    state.animation_groups.insert(group_id, group);
                }

                let handle = AnimGroupHandle {
                    group_id,
                    state: Rc::clone(&state_rc),
                };
                lua.create_userdata(handle)
            },
        )?,
    )
}
