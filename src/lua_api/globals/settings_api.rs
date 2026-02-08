//! Settings API - Modern replacement for InterfaceOptions.
//!
//! Provides the Settings table with category registration and lookup.
//! These stubs handle calls before Blizzard_Settings_Shared overwrites them.

use mlua::{Lua, Result, Value};

/// Register the Settings namespace and its methods.
pub fn register_settings_api(lua: &Lua) -> Result<()> {
    let settings = lua.create_table()?;
    // Internal category registry: Settings.__categories = {}
    let categories = lua.create_table()?;
    settings.set("__categories", categories)?;
    register_canvas_methods(lua, &settings)?;
    register_vertical_methods(lua, &settings)?;
    register_misc_methods(lua, &settings)?;
    lua.globals().set("Settings", settings)?;
    Ok(())
}

/// Create a category table with an ID, name, and GetID method.
fn make_category(lua: &Lua, id: &str, name: Option<&str>) -> Result<mlua::Table> {
    let category = lua.create_table()?;
    let id_str = id.to_string();
    category.set("ID", id_str.clone())?;
    category.set("name", name.unwrap_or(id).to_string())?;
    let get_id = lua.create_function(move |_, _self: Value| Ok(id_str.clone()))?;
    category.set("GetID", get_id)?;
    Ok(category)
}

/// Store a category in Settings.__categories by ID.
fn store_category(settings: &mlua::Table, category: &mlua::Table) -> Result<()> {
    let cats: mlua::Table = settings.get("__categories")?;
    let id: String = category.get("ID")?;
    cats.set(id, category.clone())?;
    Ok(())
}

/// Canvas layout category methods.
fn register_canvas_methods(lua: &Lua, settings: &mlua::Table) -> Result<()> {
    let s = settings.clone();
    settings.set(
        "RegisterCanvasLayoutCategory",
        lua.create_function(move |lua, (_frame, name, _group): (Value, Option<String>, Option<String>)| {
            let id = name.as_deref().unwrap_or("CustomCategory");
            let cat = make_category(lua, id, name.as_deref())?;
            store_category(&s, &cat)?;
            Ok(cat)
        })?,
    )?;
    let s = settings.clone();
    settings.set(
        "RegisterCanvasLayoutSubcategory",
        lua.create_function(move |lua, (_parent, _frame, name): (Value, Value, Option<String>)| {
            let id = name.as_deref().unwrap_or("CustomSubcategory");
            let cat = make_category(lua, id, name.as_deref())?;
            store_category(&s, &cat)?;
            Ok(cat)
        })?,
    )?;
    Ok(())
}

/// Vertical layout category methods.
fn register_vertical_methods(lua: &Lua, settings: &mlua::Table) -> Result<()> {
    let s = settings.clone();
    settings.set(
        "RegisterVerticalLayoutCategory",
        lua.create_function(move |lua, name: String| {
            let cat = make_category(lua, &name, Some(&name))?;
            store_category(&s, &cat)?;
            Ok(cat)
        })?,
    )?;
    let s = settings.clone();
    settings.set(
        "RegisterVerticalLayoutSubcategory",
        lua.create_function(move |lua, (_parent, name): (Value, String)| {
            let cat = make_category(lua, &name, Some(&name))?;
            store_category(&s, &cat)?;
            Ok(cat)
        })?,
    )?;
    Ok(())
}

/// RegisterAddOnCategory, OpenToCategory, GetCategory, callbacks.
fn register_misc_methods(lua: &Lua, settings: &mlua::Table) -> Result<()> {
    let s = settings.clone();
    settings.set(
        "RegisterAddOnCategory",
        lua.create_function(move |_, category: mlua::Table| {
            store_category(&s, &category)?;
            Ok(())
        })?,
    )?;
    settings.set(
        "OpenToCategory",
        lua.create_function(|_, (_id, _scroll): (Value, Value)| Ok(()))?,
    )?;
    let s = settings.clone();
    settings.set(
        "GetCategory",
        lua.create_function(move |_, category_id: String| {
            let cats: mlua::Table = s.get("__categories")?;
            let val: Value = cats.get(category_id)?;
            Ok(val)
        })?,
    )?;
    settings.set(
        "SetOnValueChangedCallback",
        lua.create_function(|_, (_name, _callback): (String, mlua::Function)| Ok(()))?,
    )?;
    Ok(())
}
