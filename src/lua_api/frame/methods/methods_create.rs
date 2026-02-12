//! Child creation methods: CreateTexture, CreateFontString, CreateAnimationGroup, etc.

use super::FrameHandle;
use crate::lua_api::animation::{AnimGroupHandle, AnimGroupState};
use crate::widget::{Frame, WidgetType};
use mlua::{UserDataMethods, Value};
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
fn resolve_child_name(name_raw: Option<String>, this: &FrameHandle) -> Option<String> {
    name_raw.map(|n| {
        let state = this.state.borrow();
        let parent_name = state.widgets.get(this.id).and_then(|f| f.name.as_deref());
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

/// Register a child widget in the state and create its Lua userdata + globals.
fn register_child_widget(
    lua: &mlua::Lua,
    this: &FrameHandle,
    child: Frame,
    name: &Option<String>,
) -> mlua::Result<mlua::AnyUserData> {
    let child_id = child.id;

    {
        let mut state = this.state.borrow_mut();
        state.widgets.register(child);
        state.widgets.add_child(this.id, child_id);

        // Inherit strata and level from parent (regions render in parent's context)
        let parent_props = state.widgets.get(this.id).map(|p| (p.frame_strata, p.frame_level));
        if let Some((parent_strata, parent_level)) = parent_props {
            if let Some(f) = state.widgets.get_mut(child_id) {
                f.frame_strata = parent_strata;
                f.frame_level = parent_level + 1;
            }
        }
    }

    let handle = FrameHandle {
        id: child_id,
        state: Rc::clone(&this.state),
    };
    let ud = lua.create_userdata(handle)?;

    if let Some(n) = name {
        lua.globals().set(n.as_str(), ud.clone())?;
    }

    let frame_key = format!("__frame_{}", child_id);
    lua.globals().set(frame_key.as_str(), ud.clone())?;

    Ok(ud)
}

/// Add child creation methods to FrameHandle UserData.
pub fn add_create_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_create_texture_method(methods);
    add_create_mask_texture_method(methods);
    add_create_line_method(methods);
    add_create_font_string_method(methods);
    add_create_animation_group_method(methods);
}

/// CreateTexture(name, layer, inherits, subLevel)
fn add_create_texture_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("CreateTexture", |lua, this, args: mlua::MultiValue| {
        use crate::widget::DrawLayer;

        let args: Vec<Value> = args.into_iter().collect();
        let name_raw = extract_string_arg(&args, 0);
        let layer = extract_string_arg(&args, 1);
        let name = resolve_child_name(name_raw, this);

        let mut texture = Frame::new(WidgetType::Texture, name.clone(), Some(this.id));

        if let Some(layer_str) = layer
            && let Some(draw_layer) = DrawLayer::from_str(&layer_str) {
                texture.draw_layer = draw_layer;
            }

        register_child_widget(lua, this, texture, &name)
    });
}

/// CreateMaskTexture(layer, inherits, subLevel) - create a mask texture.
fn add_create_mask_texture_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("CreateMaskTexture", |lua, this, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();
        let name_raw = extract_string_arg(&args, 0);
        let name = resolve_child_name(name_raw, this);
        let mut texture = Frame::new(WidgetType::Texture, name.clone(), Some(this.id));
        texture.is_mask = true;
        register_child_widget(lua, this, texture, &name)
    });
}

/// CreateLine(name, layer, inherits, subLevel) - create a Line (texture with start/end points).
fn add_create_line_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("CreateLine", |lua, this, args: mlua::MultiValue| {
        use crate::widget::DrawLayer;

        let args: Vec<Value> = args.into_iter().collect();
        let name_raw = extract_string_arg(&args, 0);
        let layer = extract_string_arg(&args, 1);
        let name = resolve_child_name(name_raw, this);

        let mut line = Frame::new(WidgetType::Texture, name.clone(), Some(this.id));

        if let Some(layer_str) = layer
            && let Some(draw_layer) = DrawLayer::from_str(&layer_str) {
                line.draw_layer = draw_layer;
            }

        register_child_widget(lua, this, line, &name)
    });
}

/// CreateFontString(name, layer, inherits)
fn add_create_font_string_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("CreateFontString", |lua, this, args: mlua::MultiValue| {
        use crate::widget::DrawLayer;

        let args: Vec<Value> = args.into_iter().collect();
        let name_raw = extract_string_arg(&args, 0);
        let layer = extract_string_arg(&args, 1);
        let inherits = extract_string_arg(&args, 2);
        let name = resolve_child_name(name_raw, this);

        let mut fontstring = Frame::new(WidgetType::FontString, name.clone(), Some(this.id));

        if let Some(layer_str) = layer
            && let Some(draw_layer) = DrawLayer::from_str(&layer_str) {
                fontstring.draw_layer = draw_layer;
            }

        apply_font_inherit(lua, &mut fontstring, inherits.as_deref());

        register_child_widget(lua, this, fontstring, &name)
    });
}

/// Apply font properties from an inherited Font object to a fontstring widget.
fn apply_font_inherit(lua: &mlua::Lua, frame: &mut Frame, inherits: Option<&str>) {
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
fn add_create_animation_group_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "CreateAnimationGroup",
        |lua, this, (name, _inherits): (Option<String>, Option<String>)| {
            let group_id;
            {
                let mut state = this.state.borrow_mut();
                group_id = state.next_anim_group_id;
                state.next_anim_group_id += 1;
                let mut group = AnimGroupState::new(this.id);
                group.name = name;
                state.animation_groups.insert(group_id, group);
            }

            let handle = AnimGroupHandle {
                group_id,
                state: Rc::clone(&this.state),
            };
            lua.create_userdata(handle)
        },
    );
}
