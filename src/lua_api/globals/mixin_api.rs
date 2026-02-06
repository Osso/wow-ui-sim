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
    globals.set("POIButtonMixin", register_poi_button_mixin(lua)?)?;
    globals.set("TaggableObjectMixin", register_taggable_object_mixin(lua)?)?;
    globals.set("MapCanvasPinMixin", register_map_canvas_pin_mixin(lua)?)?;
    globals.set("Menu", register_menu(lua)?)?;
    globals.set("MenuUtil", register_menu_util(lua)?)?;
    Ok(())
}

/// POIButtonMixin - mixin for quest POI buttons on world map.
fn register_poi_button_mixin(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("OnLoad", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("OnShow", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("OnHide", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("OnEnter", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("OnLeave", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("OnClick", lua.create_function(|_, (_self, _button): (Value, Option<String>)| Ok(()))?)?;
    t.set("UpdateButtonStyle", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("SetSelected", lua.create_function(|_, (_self, _selected): (Value, bool)| Ok(()))?)?;
    t.set("GetSelected", lua.create_function(|_, _self: Value| Ok(false))?)?;
    Ok(t)
}

/// TaggableObjectMixin - mixin for objects that can have tags.
fn register_taggable_object_mixin(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    add_taggable_methods(lua, &t)?;
    Ok(t)
}

/// Add taggable object methods to a table (shared by TaggableObjectMixin and MapCanvasPinMixin).
fn add_taggable_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("AddTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    t.set("RemoveTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    t.set("MatchesTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(false))?)?;
    t.set("MatchesAnyTag", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    t.set("MatchesAllTags", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    Ok(())
}

/// MapCanvasPinMixin - mixin for map pins on WorldMapFrame canvas.
fn register_map_canvas_pin_mixin(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    // Inherited taggable methods
    add_taggable_methods(lua, &t)?;

    // Pin-specific event handlers
    t.set("OnLoad", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("OnAcquired", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    t.set("OnReleased", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("OnClick", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    t.set("OnMouseEnter", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("OnMouseLeave", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("OnMouseDown", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    t.set("OnMouseUp", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;

    // Pin positioning and frame level
    add_pin_positioning_methods(lua, &t)?;

    // Nudge settings
    add_pin_nudge_methods(lua, &t)?;

    // Misc pin methods
    t.set("DisableInheritedMotionScriptsWarning", lua.create_function(|_, _self: Value| Ok(false))?)?;
    t.set("ShouldMouseButtonBePassthrough", lua.create_function(|_, (_self, _button): (Value, String)| Ok(false))?)?;
    t.set("CheckMouseButtonPassthrough", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    t.set("AddIconWidgets", lua.create_function(|_, _self: Value| Ok(()))?)?;

    Ok(t)
}

/// Add pin positioning and frame level methods.
fn add_pin_positioning_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetMap", lua.create_function(|_, _self: Value| Ok(Value::Nil))?)?;
    t.set("SetPosition", lua.create_function(|_, (_self, _x, _y): (Value, f64, f64)| Ok(()))?)?;
    t.set("SetFrameLevelType", lua.create_function(|_, (_self, _type): (Value, String)| Ok(()))?)?;
    t.set("GetFrameLevelType", lua.create_function(|lua, _self: Value| Ok(Value::String(lua.create_string("PIN_FRAME_LEVEL_DEFAULT")?)))?)?;
    t.set("UseFrameLevelType", lua.create_function(|_, (_self, _type): (Value, String)| Ok(()))?)?;
    t.set("ApplyFrameLevel", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("ApplyCurrentPosition", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("ApplyCurrentScale", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("ApplyCurrentAlpha", lua.create_function(|_, _self: Value| Ok(()))?)?;
    t.set("SetScalingLimits", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    t.set("SetAlphaLimits", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    Ok(())
}

/// Add nudge-related methods for map pins.
fn add_pin_nudge_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("SetNudgeSourceRadius", lua.create_function(|_, (_self, _radius): (Value, f64)| Ok(()))?)?;
    t.set("SetNudgeSourceMagnitude", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    t.set("SetNudgeTargetFactor", lua.create_function(|_, (_self, _factor): (Value, f64)| Ok(()))?)?;
    t.set("SetNudgeZoomedInFactor", lua.create_function(|_, (_self, _factor): (Value, f64)| Ok(()))?)?;
    t.set("SetNudgeZoomedOutFactor", lua.create_function(|_, (_self, _factor): (Value, f64)| Ok(()))?)?;
    Ok(())
}

/// Menu - new context menu system (WoW 10.0+).
fn register_menu(lua: &Lua) -> Result<mlua::Table> {
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

    Ok(menu)
}

/// MenuUtil - utility functions for the new menu system.
fn register_menu_util(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("CreateRootMenuDescription", lua.create_function(|lua, _menu_tag: Option<String>| {
        let desc = lua.create_table()?;
        desc.set("CreateButton", lua.create_function(|_, (_self, _text, _callback): (Value, String, Option<mlua::Function>)| Ok(Value::Nil))?)?;
        desc.set("CreateTitle", lua.create_function(|_, (_self, _text): (Value, String)| Ok(Value::Nil))?)?;
        desc.set("CreateDivider", lua.create_function(|_, _self: Value| Ok(Value::Nil))?)?;
        Ok(desc)
    })?)?;
    t.set("SetElementData", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    t.set("GetElementData", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(Value::Nil))?)?;
    Ok(t)
}
