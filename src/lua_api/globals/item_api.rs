//! Item class for async item loading
//!
//! Provides Item.CreateFromItemID and Item.CreateFromItemLink for addons
//! that need to load item data asynchronously (LegionRemixHelper, etc.)

use mlua::{Lua, Result, Value};

pub fn register_item_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // Item class - for async item loading (used by LegionRemixHelper, etc)
    let item_class = lua.create_table()?;
    item_class.set(
        "CreateFromItemID",
        lua.create_function(|lua, (_self, item_id): (Value, i32)| {
            // Create an item object with callback methods
            let item = lua.create_table()?;
            item.set("itemID", item_id)?;

            // ContinueOnItemLoad - calls callback immediately in simulation
            item.set(
                "ContinueOnItemLoad",
                lua.create_function(|_, (this, callback): (mlua::Table, mlua::Function)| {
                    // In simulation, immediately call the callback
                    callback.call::<()>(())?;
                    let _ = this; // Silence unused warning
                    Ok(())
                })?,
            )?;

            // GetItemID
            item.set(
                "GetItemID",
                lua.create_function(|_, this: mlua::Table| {
                    this.get::<i32>("itemID")
                })?,
            )?;

            // GetItemName - return placeholder name
            item.set(
                "GetItemName",
                lua.create_function(|lua, this: mlua::Table| {
                    let id: i32 = this.get("itemID")?;
                    Ok(Value::String(lua.create_string(&format!("Item {}", id))?))
                })?,
            )?;

            // GetItemLink
            item.set(
                "GetItemLink",
                lua.create_function(|lua, this: mlua::Table| {
                    let id: i32 = this.get("itemID")?;
                    let link = format!("|cff1eff00|Hitem:{}::::::::60:::::|h[Item {}]|h|r", id, id);
                    Ok(Value::String(lua.create_string(&link)?))
                })?,
            )?;

            // GetItemIcon
            item.set(
                "GetItemIcon",
                lua.create_function(|_, _this: mlua::Table| {
                    Ok(134400i32) // INV_Misc_QuestionMark
                })?,
            )?;

            // GetItemQuality
            item.set(
                "GetItemQuality",
                lua.create_function(|_, _this: mlua::Table| {
                    Ok(1i32) // Common quality
                })?,
            )?;

            // IsItemDataCached - always true in simulation
            item.set(
                "IsItemDataCached",
                lua.create_function(|_, _this: mlua::Table| Ok(true))?,
            )?;

            Ok(item)
        })?,
    )?;
    item_class.set(
        "CreateFromItemLink",
        lua.create_function(|lua, (_self, item_link): (Value, String)| {
            // Extract item ID from link if possible, otherwise use 0
            let item_id = item_link
                .split("item:")
                .nth(1)
                .and_then(|s| s.split(':').next())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            // Reuse CreateFromItemID logic
            let item = lua.create_table()?;
            item.set("itemID", item_id)?;

            item.set(
                "ContinueOnItemLoad",
                lua.create_function(|_, (this, callback): (mlua::Table, mlua::Function)| {
                    callback.call::<()>(())?;
                    let _ = this;
                    Ok(())
                })?,
            )?;

            item.set(
                "GetItemID",
                lua.create_function(|_, this: mlua::Table| {
                    this.get::<i32>("itemID")
                })?,
            )?;

            item.set(
                "GetItemName",
                lua.create_function(|lua, this: mlua::Table| {
                    let id: i32 = this.get("itemID")?;
                    Ok(Value::String(lua.create_string(&format!("Item {}", id))?))
                })?,
            )?;

            item.set(
                "GetItemLink",
                lua.create_function(|lua, this: mlua::Table| {
                    let id: i32 = this.get("itemID")?;
                    let link = format!("|cff1eff00|Hitem:{}::::::::60:::::|h[Item {}]|h|r", id, id);
                    Ok(Value::String(lua.create_string(&link)?))
                })?,
            )?;

            item.set(
                "GetItemIcon",
                lua.create_function(|_, _this: mlua::Table| Ok(134400i32))?,
            )?;

            item.set(
                "GetItemQuality",
                lua.create_function(|_, _this: mlua::Table| Ok(1i32))?,
            )?;

            item.set(
                "IsItemDataCached",
                lua.create_function(|_, _this: mlua::Table| Ok(true))?,
            )?;

            Ok(item)
        })?,
    )?;
    globals.set("Item", item_class)?;

    Ok(())
}
