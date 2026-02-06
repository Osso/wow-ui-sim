//! Child creation methods: CreateTexture, CreateFontString, CreateAnimationGroup, etc.

use super::FrameHandle;
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

    // CreateAnimationGroup(name, inherits) - create animation group (stub)
    methods.add_method(
        "CreateAnimationGroup",
        |lua, _this, (_name, _inherits): (Option<String>, Option<String>)| {
            create_animation_group(lua)
        },
    );
}

/// Create a stub animation group table with all required methods.
fn create_animation_group(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
    let anim_group = lua.create_table()?;
    anim_group.set("Play", lua.create_function(|_, _self: Value| Ok(()))?)?;
    anim_group.set("Stop", lua.create_function(|_, _self: Value| Ok(()))?)?;
    anim_group.set("Pause", lua.create_function(|_, _self: Value| Ok(()))?)?;
    anim_group.set("Finish", lua.create_function(|_, _self: Value| Ok(()))?)?;
    anim_group.set(
        "IsPlaying",
        lua.create_function(|_, _self: Value| Ok(false))?,
    )?;
    anim_group.set(
        "IsPaused",
        lua.create_function(|_, _self: Value| Ok(false))?,
    )?;
    anim_group.set("IsDone", lua.create_function(|_, _self: Value| Ok(true))?)?;
    anim_group.set(
        "SetLooping",
        lua.create_function(|_, (_self, _looping): (Value, Option<String>)| Ok(()))?,
    )?;
    anim_group.set(
        "GetLooping",
        lua.create_function(|lua, _self: Value| {
            Ok(Value::String(lua.create_string("NONE")?))
        })?,
    )?;
    anim_group.set(
        "GetParent",
        lua.create_function(|_, _self: Value| Ok(Value::Nil))?,
    )?;
    anim_group.set(
        "GetAnimations",
        lua.create_function(|_, _self: Value| Ok(mlua::MultiValue::new()))?,
    )?;
    anim_group.set(
        "SetToFinalAlpha",
        lua.create_function(|_, (_self, _final): (Value, bool)| Ok(()))?,
    )?;
    anim_group.set(
        "GetToFinalAlpha",
        lua.create_function(|_, _self: Value| Ok(false))?,
    )?;
    anim_group.set("CreateAnimation", lua.create_function(create_animation)?)?;
    anim_group.set(
        "SetScript",
        lua.create_function(
            |_, (_self, _event, _handler): (Value, String, Option<mlua::Function>)| Ok(()),
        )?,
    )?;
    anim_group.set(
        "GetScript",
        lua.create_function(|_, (_self, _event): (Value, String)| Ok(Value::Nil))?,
    )?;
    anim_group.set(
        "HasScript",
        lua.create_function(|_, (_self, _event): (Value, String)| Ok(false))?,
    )?;
    anim_group.set(
        "HookScript",
        lua.create_function(
            |_, (_self, _event, _handler): (Value, String, Option<mlua::Function>)| Ok(()),
        )?,
    )?;
    anim_group.set(
        "Restart",
        lua.create_function(|_, (_self, _reverse): (Value, Option<bool>)| Ok(()))?,
    )?;
    anim_group.set(
        "PlaySynced",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    anim_group.set(
        "GetAnimationSpeedMultiplier",
        lua.create_function(|_, _self: Value| Ok(1.0_f64))?,
    )?;
    anim_group.set(
        "SetAnimationSpeedMultiplier",
        lua.create_function(|_, (_self, _mult): (Value, f64)| Ok(()))?,
    )?;
    anim_group.set(
        "IsSetToFinalAlpha",
        lua.create_function(|_, _self: Value| Ok(false))?,
    )?;
    anim_group.set(
        "IsPendingFinish",
        lua.create_function(|_, _self: Value| Ok(false))?,
    )?;
    anim_group.set(
        "IsReverse",
        lua.create_function(|_, _self: Value| Ok(false))?,
    )?;
    anim_group.set(
        "GetDuration",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    anim_group.set(
        "GetElapsed",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    anim_group.set(
        "GetProgress",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    anim_group.set(
        "GetLoopState",
        lua.create_function(|lua, _self: Value| Ok(Value::String(lua.create_string("NONE")?)))?,
    )?;
    anim_group.set(
        "RemoveAnimations",
        lua.create_function(|_, _self: Value| Ok(()))?,
    )?;
    anim_group.set(
        "GetName",
        lua.create_function(|_, _self: Value| Ok(Value::Nil))?,
    )?;
    anim_group.set(
        "SetAlpha",
        lua.create_function(|_, (_self, _alpha): (Value, f64)| Ok(()))?,
    )?;
    anim_group.set(
        "GetAlpha",
        lua.create_function(|_, _self: Value| Ok(1.0_f64))?,
    )?;
    Ok(anim_group)
}

/// Create a stub animation table with all required methods.
fn create_animation(
    lua: &mlua::Lua,
    (_self, _anim_type, _name, _inherits): (Value, Option<String>, Option<String>, Option<String>),
) -> mlua::Result<mlua::Table> {
    let anim = lua.create_table()?;
    // Duration methods
    anim.set(
        "SetDuration",
        lua.create_function(|_, (_self, _dur): (Value, f64)| Ok(()))?,
    )?;
    anim.set(
        "GetDuration",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    // Delay methods
    anim.set(
        "SetStartDelay",
        lua.create_function(|_, (_self, _delay): (Value, f64)| Ok(()))?,
    )?;
    anim.set(
        "SetEndDelay",
        lua.create_function(|_, (_self, _delay): (Value, f64)| Ok(()))?,
    )?;
    // Order and smoothing
    anim.set(
        "SetOrder",
        lua.create_function(|_, (_self, _order): (Value, i32)| Ok(()))?,
    )?;
    anim.set(
        "SetSmoothing",
        lua.create_function(|_, (_self, _smooth): (Value, String)| Ok(()))?,
    )?;
    // Alpha animation methods
    anim.set(
        "SetFromAlpha",
        lua.create_function(|_, (_self, _alpha): (Value, f64)| Ok(()))?,
    )?;
    anim.set(
        "SetToAlpha",
        lua.create_function(|_, (_self, _alpha): (Value, f64)| Ok(()))?,
    )?;
    // Transform methods
    anim.set(
        "SetChange",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    anim.set(
        "SetScale",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    anim.set(
        "SetScaleFrom",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    anim.set(
        "SetScaleTo",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    anim.set(
        "SetOffset",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    anim.set(
        "SetDegrees",
        lua.create_function(|_, (_self, _degrees): (Value, f64)| Ok(()))?,
    )?;
    anim.set(
        "SetOrigin",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    // Playback control
    anim.set("Play", lua.create_function(|_, _self: Value| Ok(()))?)?;
    anim.set("Stop", lua.create_function(|_, _self: Value| Ok(()))?)?;
    anim.set("Pause", lua.create_function(|_, _self: Value| Ok(()))?)?;
    anim.set(
        "Restart",
        lua.create_function(|_, _self: Value| Ok(()))?,
    )?;
    anim.set("Finish", lua.create_function(|_, _self: Value| Ok(()))?)?;
    anim.set(
        "IsPlaying",
        lua.create_function(|_, _self: Value| Ok(false))?,
    )?;
    anim.set(
        "IsPaused",
        lua.create_function(|_, _self: Value| Ok(false))?,
    )?;
    anim.set("IsDone", lua.create_function(|_, _self: Value| Ok(true))?)?;
    anim.set(
        "IsStopped",
        lua.create_function(|_, _self: Value| Ok(true))?,
    )?;
    anim.set(
        "IsDelaying",
        lua.create_function(|_, _self: Value| Ok(false))?,
    )?;
    // Parent accessors
    anim.set(
        "GetParent",
        lua.create_function(|_, _self: Value| Ok(Value::Nil))?,
    )?;
    anim.set(
        "GetRegionParent",
        lua.create_function(|_, _self: Value| Ok(Value::Nil))?,
    )?;
    // Progress and timing
    anim.set(
        "GetProgress",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    anim.set(
        "GetSmoothProgress",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    anim.set(
        "GetElapsed",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    anim.set(
        "GetStartDelay",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    anim.set(
        "GetEndDelay",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    anim.set(
        "GetOrder",
        lua.create_function(|_, _self: Value| Ok(0_i32))?,
    )?;
    anim.set(
        "GetSmoothing",
        lua.create_function(|lua, _self: Value| Ok(Value::String(lua.create_string("NONE")?)))?,
    )?;
    // Target methods
    anim.set(
        "GetTarget",
        lua.create_function(|_, _self: Value| Ok(Value::Nil))?,
    )?;
    anim.set(
        "SetTarget",
        lua.create_function(|_, (_self, _target): (Value, Value)| Ok(()))?,
    )?;
    anim.set(
        "SetChildKey",
        lua.create_function(|_, (_self, _key): (Value, String)| Ok(()))?,
    )?;
    anim.set(
        "SetTargetKey",
        lua.create_function(|_, (_self, _key): (Value, String)| Ok(()))?,
    )?;
    anim.set(
        "SetTargetName",
        lua.create_function(|_, (_self, _name): (Value, String)| Ok(()))?,
    )?;
    anim.set(
        "SetTargetParent",
        lua.create_function(|_, _self: Value| Ok(()))?,
    )?;
    // Alpha getters
    anim.set(
        "GetFromAlpha",
        lua.create_function(|_, _self: Value| Ok(0.0_f64))?,
    )?;
    anim.set(
        "GetToAlpha",
        lua.create_function(|_, _self: Value| Ok(1.0_f64))?,
    )?;
    // Name
    anim.set(
        "GetName",
        lua.create_function(|_, _self: Value| Ok(Value::Nil))?,
    )?;
    // Script handlers
    anim.set(
        "SetScript",
        lua.create_function(
            |_, (_self, _event, _handler): (Value, String, Option<mlua::Function>)| Ok(()),
        )?,
    )?;
    anim.set(
        "GetScript",
        lua.create_function(|_, (_self, _event): (Value, String)| Ok(Value::Nil))?,
    )?;
    anim.set(
        "HasScript",
        lua.create_function(|_, (_self, _event): (Value, String)| Ok(false))?,
    )?;
    anim.set(
        "HookScript",
        lua.create_function(
            |_, (_self, _event, _handler): (Value, String, Option<mlua::Function>)| Ok(()),
        )?,
    )?;
    Ok(anim)
}
