//! Settings API - Modern replacement for InterfaceOptions.
//!
//! Provides the Settings table with category registration functions.

use mlua::{Lua, Result, Value};

/// Register the Settings namespace and its methods.
pub fn register_settings_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // Settings API (modern replacement for InterfaceOptions)
    let settings = lua.create_table()?;
    settings.set(
        "RegisterCanvasLayoutCategory",
        lua.create_function(|lua, (_frame, _name, _group): (Value, Option<String>, Option<String>)| {
            // Return a dummy category object
            let category = lua.create_table()?;
            category.set("ID", "CustomCategory")?;
            Ok(category)
        })?,
    )?;
    settings.set(
        "RegisterCanvasLayoutSubcategory",
        lua.create_function(|lua, (_parent, _frame, _name): (Value, Value, Option<String>)| {
            let category = lua.create_table()?;
            category.set("ID", "CustomSubcategory")?;
            Ok(category)
        })?,
    )?;
    settings.set(
        "RegisterAddOnCategory",
        lua.create_function(|_, _category: Value| Ok(()))?,
    )?;
    settings.set(
        "OpenToCategory",
        lua.create_function(|_, _category_id: String| Ok(()))?,
    )?;
    settings.set(
        "RegisterVerticalLayoutCategory",
        lua.create_function(|lua, _name: String| {
            let category = lua.create_table()?;
            category.set("ID", "VerticalCategory")?;
            Ok(category)
        })?,
    )?;
    settings.set(
        "RegisterVerticalLayoutSubcategory",
        lua.create_function(|lua, (_parent, _name): (Value, String)| {
            let category = lua.create_table()?;
            category.set("ID", "VerticalSubcategory")?;
            Ok(category)
        })?,
    )?;
    // GetCategory(categoryID) - returns category by ID
    settings.set(
        "GetCategory",
        lua.create_function(|lua, _category_id: String| {
            let category = lua.create_table()?;
            category.set("ID", _category_id.clone())?;
            category.set("name", _category_id)?;
            Ok(category)
        })?,
    )?;
    globals.set("Settings", settings)?;

    Ok(())
}
