//! C_Container namespace and legacy container global functions.
//!
//! Provides mock inventory data for bag slots and the full C_Container API.

use mlua::{Lua, Result, Value};

/// Register C_Container namespace, C_NewItems, and legacy container globals.
pub fn register_c_container_api(lua: &Lua) -> Result<()> {
    register_c_container(lua)?;
    register_c_new_items(lua)?;
    register_container_globals(lua)?;
    Ok(())
}

/// Register the C_NewItems namespace (new item indicators).
fn register_c_new_items(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsNewItem", lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?)?;
    t.set("RemoveNewItem", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    t.set("ClearAll", lua.create_function(|_, _: ()| Ok(()))?)?;
    lua.globals().set("C_NewItems", t)?;
    Ok(())
}

/// Mock inventory: (bag, slot) → (item_id, stack_count).
fn mock_bag_item(bag: i32, slot: i32) -> Option<(u32, i32)> {
    match (bag, slot) {
        (0, 1) => Some((6948, 1)),   // Hearthstone
        (0, 3) => Some((6948, 1)),   // Hearthstone
        (0, 5) => Some((6948, 1)),   // Hearthstone
        (0, 7) => Some((6948, 1)),   // Hearthstone
        (0, 10) => Some((6948, 1)),  // Hearthstone
        (0, 14) => Some((6948, 1)),  // Hearthstone
        _ => None,
    }
}

/// Build the `containerInfo` table returned by `C_Container.GetContainerItemInfo`.
fn build_container_item_info(lua: &Lua, item_id: u32, stack_count: i32) -> Result<Value> {
    let (name, quality) = if let Some(item) = crate::items::get_item(item_id) {
        (item.name, item.quality)
    } else {
        ("Unknown", 1u8)
    };
    let color = super::c_item_api::quality_color(quality);
    let link = format!(
        "|cff{}|Hitem:{}::::::::80:::::|h[{}]|h|r",
        color, item_id, name
    );
    let t = lua.create_table()?;
    t.set("itemID", item_id)?;
    t.set("iconFileID", 134400)?;
    t.set("stackCount", stack_count)?;
    t.set("quality", quality as i32)?;
    t.set("hyperlink", lua.create_string(&link)?)?;
    t.set("isLocked", false)?;
    t.set("isBound", false)?;
    t.set("isFiltered", false)?;
    t.set("isReadable", false)?;
    t.set("hasNoValue", false)?;
    t.set("hasLoot", false)?;
    Ok(Value::Table(t))
}

/// Register C_Container item query methods.
fn register_c_container_item_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set(
        "GetContainerItemID",
        lua.create_function(|_, (bag, slot): (i32, i32)| {
            Ok(mock_bag_item(bag, slot).map(|(id, _)| id as i64))
        })?,
    )?;
    t.set(
        "GetContainerItemLink",
        lua.create_function(|lua, (bag, slot): (i32, i32)| {
            let Some((item_id, _)) = mock_bag_item(bag, slot) else {
                return Ok(Value::Nil);
            };
            let name = crate::items::get_item(item_id)
                .map(|i| i.name)
                .unwrap_or("Unknown");
            let link = format!(
                "|cffffffff|Hitem:{}::::::::80:::::|h[{}]|h|r",
                item_id, name
            );
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;
    register_c_container_info_methods(lua, t)?;
    Ok(())
}

/// Register GetContainerItemInfo, QuestInfo, and Cooldown.
fn register_c_container_info_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set(
        "GetContainerItemInfo",
        lua.create_function(|lua, (bag, slot): (i32, i32)| {
            let Some((item_id, stack_count)) = mock_bag_item(bag, slot) else {
                return Ok(Value::Nil);
            };
            build_container_item_info(lua, item_id, stack_count)
        })?,
    )?;
    t.set(
        "GetContainerItemQuestInfo",
        lua.create_function(|lua, (_bag, _slot): (i32, i32)| {
            let t = lua.create_table()?;
            t.set("isQuestItem", false)?;
            t.set("questID", Value::Nil)?;
            t.set("isActive", false)?;
            Ok(Value::Table(t))
        })?,
    )?;
    t.set(
        "GetContainerItemCooldown",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok((0.0, 0.0, 1)))?,
    )?;
    Ok(())
}

/// Register C_Container stub methods used by ContainerFrame.lua.
fn register_c_container_stubs(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("IsContainerFiltered", lua.create_function(|_, _bag: i32| Ok(false))?)?;
    t.set(
        "GetBagName",
        lua.create_function(|lua, bag: i32| {
            let name = match bag {
                0 => "Backpack",
                _ => "Bag",
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    t.set(
        "ContainerIDToInventoryID",
        lua.create_function(|_, bag: i32| Ok(if bag > 0 { 19 + bag } else { 0 }))?,
    )?;
    t.set(
        "HasContainerItem",
        lua.create_function(|_, (bag, slot): (i32, i32)| Ok(mock_bag_item(bag, slot).is_some()))?,
    )?;
    t.set("GetBagSlotFlag", lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?)?;
    t.set("SetBagSlotFlag", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    t.set("GetBackpackAutosortDisabled", lua.create_function(|_, _: ()| Ok(false))?)?;
    t.set("SetBackpackAutosortDisabled", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    t.set("GetBackpackSellJunkDisabled", lua.create_function(|_, _: ()| Ok(false))?)?;
    t.set("SetBackpackSellJunkDisabled", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    t.set("GetContainerItemPurchaseInfo", lua.create_function(|_, _: mlua::MultiValue| Ok(Value::Nil))?)?;
    t.set("UseContainerItem", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    t.set("PickupContainerItem", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    t.set("SplitContainerItem", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    t.set("IsBattlePayItem", lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?)?;
    Ok(())
}

/// Register the C_Container namespace.
fn register_c_container(lua: &Lua) -> Result<()> {
    let c_container = lua.create_table()?;

    c_container.set(
        "GetContainerNumSlots",
        lua.create_function(|_, bag: i32| Ok(if bag == 0 { 16 } else { 0 }))?,
    )?;
    register_c_container_item_methods(lua, &c_container)?;
    register_c_container_stubs(lua, &c_container)?;

    lua.globals().set("C_Container", c_container)?;
    Ok(())
}

/// Register legacy global container functions (GetContainerNumSlots, etc.).
fn register_container_globals(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetContainerNumSlots",
        lua.create_function(|_, bag: i32| Ok(if bag == 0 { 16 } else { 0 }))?,
    )?;
    globals.set(
        "GetContainerItemID",
        lua.create_function(|_, (bag, slot): (i32, i32)| {
            Ok(mock_bag_item(bag, slot).map(|(id, _)| id as i64))
        })?,
    )?;
    globals.set(
        "GetContainerItemLink",
        lua.create_function(|lua, (bag, slot): (i32, i32)| {
            let Some((item_id, _)) = mock_bag_item(bag, slot) else {
                return Ok(Value::Nil);
            };
            let name = crate::items::get_item(item_id)
                .map(|i| i.name)
                .unwrap_or("Unknown");
            let link = format!(
                "|cffffffff|Hitem:{}::::::::80:::::|h[{}]|h|r",
                item_id, name
            );
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;

    Ok(())
}
