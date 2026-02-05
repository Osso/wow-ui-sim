//! WoW UI Mixin API
//!
//! This module provides mixin tables used by WoW UI frames:
//! - POIButtonMixin - Quest POI buttons on world map
//! - TaggableObjectMixin - Base mixin for taggable objects
//! - MapCanvasPinMixin - Map pins on WorldMapFrame canvas
//! - Menu - New context menu system (WoW 10.0+)
//! - MenuUtil - Utility functions for the menu system

use mlua::{Lua, Result, Value};

/// Register all mixin API tables with the Lua environment.
pub fn register_mixin_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // POIButtonMixin - mixin for quest POI buttons on world map
    let poi_button_mixin = lua.create_table()?;
    poi_button_mixin.set("OnLoad", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnShow", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnHide", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnEnter", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnLeave", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnClick", lua.create_function(|_, (_self, _button): (Value, Option<String>)| Ok(()))?)?;
    poi_button_mixin.set("UpdateButtonStyle", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("SetSelected", lua.create_function(|_, (_self, _selected): (Value, bool)| Ok(()))?)?;
    poi_button_mixin.set("GetSelected", lua.create_function(|_, _self: Value| Ok(false))?)?;
    globals.set("POIButtonMixin", poi_button_mixin)?;

    // TaggableObjectMixin - mixin for objects that can have tags (used by MapCanvasPinMixin)
    let taggable_object_mixin = lua.create_table()?;
    taggable_object_mixin.set("AddTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    taggable_object_mixin.set("RemoveTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    taggable_object_mixin.set("MatchesTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(false))?)?;
    taggable_object_mixin.set("MatchesAnyTag", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    taggable_object_mixin.set("MatchesAllTags", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    globals.set("TaggableObjectMixin", taggable_object_mixin)?;

    // MapCanvasPinMixin - mixin for map pins on WorldMapFrame canvas (inherits from TaggableObjectMixin)
    let map_canvas_pin_mixin = lua.create_table()?;
    // Methods from TaggableObjectMixin (duplicated for mixin inheritance pattern)
    map_canvas_pin_mixin.set("AddTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    map_canvas_pin_mixin.set("RemoveTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    map_canvas_pin_mixin.set("MatchesTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(false))?)?;
    map_canvas_pin_mixin.set("MatchesAnyTag", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    map_canvas_pin_mixin.set("MatchesAllTags", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    // MapCanvasPinMixin specific methods
    map_canvas_pin_mixin.set("OnLoad", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnAcquired", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnReleased", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnClick", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnMouseEnter", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnMouseLeave", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnMouseDown", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnMouseUp", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("GetMap", lua.create_function(|_, _self: Value| Ok(Value::Nil))?)?;
    map_canvas_pin_mixin.set("SetPosition", lua.create_function(|_, (_self, _x, _y): (Value, f64, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetFrameLevelType", lua.create_function(|_, (_self, _type): (Value, String)| Ok(()))?)?;
    map_canvas_pin_mixin.set("GetFrameLevelType", lua.create_function(|lua, _self: Value| Ok(Value::String(lua.create_string("PIN_FRAME_LEVEL_DEFAULT")?)))?)?;
    map_canvas_pin_mixin.set("UseFrameLevelType", lua.create_function(|_, (_self, _type): (Value, String)| Ok(()))?)?;
    map_canvas_pin_mixin.set("ApplyFrameLevel", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("ApplyCurrentPosition", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("ApplyCurrentScale", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("ApplyCurrentAlpha", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetScalingLimits", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetAlphaLimits", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeSourceRadius", lua.create_function(|_, (_self, _radius): (Value, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeSourceMagnitude", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeTargetFactor", lua.create_function(|_, (_self, _factor): (Value, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeZoomedInFactor", lua.create_function(|_, (_self, _factor): (Value, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeZoomedOutFactor", lua.create_function(|_, (_self, _factor): (Value, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("DisableInheritedMotionScriptsWarning", lua.create_function(|_, _self: Value| Ok(false))?)?;
    map_canvas_pin_mixin.set("ShouldMouseButtonBePassthrough", lua.create_function(|_, (_self, _button): (Value, String)| Ok(false))?)?;
    map_canvas_pin_mixin.set("CheckMouseButtonPassthrough", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("AddIconWidgets", lua.create_function(|_, _self: Value| Ok(()))?)?;
    globals.set("MapCanvasPinMixin", map_canvas_pin_mixin)?;

    // Menu - new context menu system (WoW 10.0+)
    let menu = lua.create_table()?;
    menu.set("GetOpenMenu", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    menu.set("GetOpenMenuTags", lua.create_function(|lua, ()| Ok(lua.create_table()?))?)?;
    menu.set("PopupMenu", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    menu.set("OpenMenu", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    menu.set("CloseAll", lua.create_function(|_, ()| Ok(()))?)?;
    menu.set("ModifyMenu", lua.create_function(|_, (_owner, _generator_fn): (Value, mlua::Function)| Ok(()))?)?;
    let menu_response = lua.create_table()?;
    menu_response.set("Close", 0)?;
    menu_response.set("Open", 1)?;
    menu_response.set("Refresh", 2)?;
    menu_response.set("CloseAll", 3)?;
    menu.set("Response", menu_response)?;
    globals.set("Menu", menu)?;

    // MenuUtil - utility functions for the new menu system
    let menu_util = lua.create_table()?;
    menu_util.set("CreateRootMenuDescription", lua.create_function(|lua, _menu_tag: Option<String>| {
        let desc = lua.create_table()?;
        desc.set("CreateButton", lua.create_function(|_, (_self, _text, _callback): (Value, String, Option<mlua::Function>)| Ok(Value::Nil))?)?;
        desc.set("CreateTitle", lua.create_function(|_, (_self, _text): (Value, String)| Ok(Value::Nil))?)?;
        desc.set("CreateDivider", lua.create_function(|_, _self: Value| Ok(Value::Nil))?)?;
        Ok(desc)
    })?)?;
    menu_util.set("SetElementData", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    menu_util.set("GetElementData", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(Value::Nil))?)?;
    globals.set("MenuUtil", menu_util)?;

    Ok(())
}
