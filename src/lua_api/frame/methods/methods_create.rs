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

/// Add child creation methods to FrameHandle UserData.
pub fn add_create_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // CreateTexture(name, layer, inherits, subLevel)
    methods.add_method("CreateTexture", |lua, this, args: mlua::MultiValue| {
        use crate::widget::DrawLayer;

        let args: Vec<Value> = args.into_iter().collect();

        let name_raw: Option<String> = args.first().and_then(|v| {
            if let Value::String(s) = v {
                Some(s.to_string_lossy().to_string())
            } else {
                None
            }
        });

        // Parse layer argument (second parameter)
        let layer: Option<String> = args.get(1).and_then(|v| {
            if let Value::String(s) = v {
                Some(s.to_string_lossy().to_string())
            } else {
                None
            }
        });

        // Handle $parent substitution
        let name: Option<String> = name_raw.map(|n| {
            let state = this.state.borrow();
            let parent_name = state.widgets.get(this.id).and_then(|f| f.name.as_deref());
            substitute_parent_name(&n, parent_name)
        });

        let mut texture = Frame::new(WidgetType::Texture, name.clone(), Some(this.id));

        // Apply layer if specified
        if let Some(layer_str) = layer {
            if let Some(draw_layer) = DrawLayer::from_str(&layer_str) {
                texture.draw_layer = draw_layer;
            }
        }

        let texture_id = texture.id;

        {
            let mut state = this.state.borrow_mut();
            state.widgets.register(texture);
            state.widgets.add_child(this.id, texture_id);
        }

        let handle = FrameHandle {
            id: texture_id,
            state: Rc::clone(&this.state),
        };

        let ud = lua.create_userdata(handle)?;

        if let Some(ref n) = name {
            lua.globals().set(n.as_str(), ud.clone())?;
        }

        let frame_key = format!("__frame_{}", texture_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        Ok(ud)
    });

    // CreateMaskTexture(layer, inherits, subLevel) - create a mask texture (stub)
    methods.add_method("CreateMaskTexture", |lua, this, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();

        let name_raw: Option<String> = args.first().and_then(|v| {
            if let Value::String(s) = v {
                Some(s.to_string_lossy().to_string())
            } else {
                None
            }
        });

        // Handle $parent substitution
        let name: Option<String> = name_raw.map(|n| {
            let state = this.state.borrow();
            let parent_name = state.widgets.get(this.id).and_then(|f| f.name.as_deref());
            substitute_parent_name(&n, parent_name)
        });

        // Create a texture and return it as a mask texture (they're essentially the same)
        let texture = Frame::new(WidgetType::Texture, name.clone(), Some(this.id));
        let texture_id = texture.id;

        {
            let mut state = this.state.borrow_mut();
            state.widgets.register(texture);
            state.widgets.add_child(this.id, texture_id);
        }

        let handle = FrameHandle {
            id: texture_id,
            state: Rc::clone(&this.state),
        };

        let ud = lua.create_userdata(handle)?;

        if let Some(ref n) = name {
            lua.globals().set(n.as_str(), ud.clone())?;
        }

        let frame_key = format!("__frame_{}", texture_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        Ok(ud)
    });

    // CreateFontString(name, layer, inherits)
    methods.add_method("CreateFontString", |lua, this, args: mlua::MultiValue| {
        use crate::widget::DrawLayer;

        let args: Vec<Value> = args.into_iter().collect();

        let name_raw: Option<String> = args.first().and_then(|v| {
            if let Value::String(s) = v {
                Some(s.to_string_lossy().to_string())
            } else {
                None
            }
        });

        // Parse layer argument (second parameter)
        let layer: Option<String> = args.get(1).and_then(|v| {
            if let Value::String(s) = v {
                Some(s.to_string_lossy().to_string())
            } else {
                None
            }
        });

        // Handle $parent substitution
        let name: Option<String> = name_raw.map(|n| {
            let state = this.state.borrow();
            let parent_name = state.widgets.get(this.id).and_then(|f| f.name.as_deref());
            substitute_parent_name(&n, parent_name)
        });

        let mut fontstring = Frame::new(WidgetType::FontString, name.clone(), Some(this.id));

        // Apply layer if specified
        if let Some(layer_str) = layer {
            if let Some(draw_layer) = DrawLayer::from_str(&layer_str) {
                fontstring.draw_layer = draw_layer;
            }
        }

        let fontstring_id = fontstring.id;

        {
            let mut state = this.state.borrow_mut();
            state.widgets.register(fontstring);
            state.widgets.add_child(this.id, fontstring_id);
        }

        let handle = FrameHandle {
            id: fontstring_id,
            state: Rc::clone(&this.state),
        };

        let ud = lua.create_userdata(handle)?;

        if let Some(ref n) = name {
            lua.globals().set(n.as_str(), ud.clone())?;
        }

        let frame_key = format!("__frame_{}", fontstring_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        Ok(ud)
    });

    // CreateAnimationGroup(name, inherits)
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
            Ok(lua.create_userdata(handle)?)
        },
    );
}
