//! C_Item namespace and item-related API functions.
//!
//! Contains item information, container, encoding utilities, and inventory slot functions.

use mlua::{Lua, Result, Value};

/// Register item-related C_* namespaces and global functions.
pub fn register_c_item_api(lua: &Lua) -> Result<()> {
    register_c_item(lua)?;
    register_c_container(lua)?;
    register_c_encoding_util(lua)?;
    register_legacy_item_globals(lua)?;
    register_spell_globals(lua)?;
    register_container_globals(lua)?;
    register_inventory_globals(lua)?;
    Ok(())
}

/// Register the C_Item namespace.
fn register_c_item(lua: &Lua) -> Result<()> {
    let c_item = lua.create_table()?;

    c_item.set(
        "GetItemInfo",
        lua.create_function(|_, _item_id: Value| Ok(Value::Nil))?,
    )?;

    c_item.set(
        "GetItemInfoInstant",
        lua.create_function(|lua, item_id: Value| {
            let id = parse_item_id_from_value(&item_id);
            if id == 0 {
                return Ok(mlua::MultiValue::new());
            }
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(id as i64),
                Value::String(lua.create_string("Miscellaneous")?),
                Value::String(lua.create_string("Junk")?),
                Value::String(lua.create_string("")?),
                Value::Integer(134400),
                Value::Integer(15),
                Value::Integer(0),
            ]))
        })?,
    )?;

    c_item.set(
        "GetItemIDForItemInfo",
        lua.create_function(|_, item_id: Value| {
            let id = parse_item_id_from_value(&item_id);
            if id == 0 {
                Ok(Value::Nil)
            } else {
                Ok(Value::Integer(id as i64))
            }
        })?,
    )?;

    c_item.set(
        "GetItemIconByID",
        lua.create_function(|_, _item_id: i32| Ok(134400i32))?,
    )?;

    c_item.set(
        "GetItemSubClassInfo",
        lua.create_function(|lua, (class_id, subclass_id): (i32, i32)| {
            let name = item_subclass_name(class_id, subclass_id);
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;

    c_item.set(
        "GetItemCount",
        lua.create_function(
            |_, (_id, _bank, _charges, _reagent): (Value, Option<bool>, Option<bool>, Option<bool>)| {
                Ok(0)
            },
        )?,
    )?;

    c_item.set(
        "GetItemClassInfo",
        lua.create_function(|lua, class_id: i32| {
            let name = item_class_name(class_id);
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;

    c_item.set(
        "GetItemSpecInfo",
        lua.create_function(|lua, _item_id: Value| lua.create_table())?,
    )?;

    c_item.set(
        "GetItemNameByID",
        lua.create_function(|lua, item_id: i32| {
            Ok(Value::String(
                lua.create_string(&format!("Item {}", item_id))?,
            ))
        })?,
    )?;

    c_item.set(
        "GetDetailedItemLevelInfo",
        lua.create_function(|_, _item_link: Value| Ok((0i32, 0i32, 0i32)))?,
    )?;

    c_item.set(
        "IsItemBindToAccountUntilEquip",
        lua.create_function(|_, _item_link: Value| Ok(false))?,
    )?;

    c_item.set(
        "GetItemLink",
        lua.create_function(|lua, item_id: i32| {
            let link = format!(
                "|cffffffff|Hitem:{}::::::::80:::::|h[Item {}]|h|r",
                item_id, item_id
            );
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;

    c_item.set(
        "GetItemQualityByID",
        lua.create_function(|_, _item_id: i32| Ok(1i32))?,
    )?;

    c_item.set(
        "GetItemLearnTransmogSet",
        lua.create_function(|_, _item_id: i32| Ok(Value::Nil))?,
    )?;

    c_item.set(
        "RequestLoadItemDataByID",
        lua.create_function(|_, _item_id: i32| Ok(()))?,
    )?;

    lua.globals().set("C_Item", c_item)?;
    Ok(())
}

/// Register the C_Container namespace.
fn register_c_container(lua: &Lua) -> Result<()> {
    let c_container = lua.create_table()?;

    c_container.set(
        "GetContainerNumSlots",
        lua.create_function(|_, bag: i32| Ok(if bag == 0 { 16 } else { 0 }))?,
    )?;
    c_container.set(
        "GetContainerItemID",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_container.set(
        "GetContainerItemLink",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_container.set(
        "GetContainerItemInfo",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;

    lua.globals().set("C_Container", c_container)?;
    Ok(())
}

/// Register the C_EncodingUtil namespace (stub compression/encoding).
fn register_c_encoding_util(lua: &Lua) -> Result<()> {
    let c_encoding = lua.create_table()?;

    c_encoding.set(
        "CompressString",
        lua.create_function(|lua, (data, _method): (String, Option<i32>)| {
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    c_encoding.set(
        "DecompressString",
        lua.create_function(|lua, (data, _method): (String, Option<i32>)| {
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    c_encoding.set(
        "EncodeBase64",
        lua.create_function(|lua, data: String| Ok(Value::String(lua.create_string(&data)?)))?,
    )?;
    c_encoding.set(
        "DecodeBase64",
        lua.create_function(|lua, data: String| Ok(Value::String(lua.create_string(&data)?)))?,
    )?;

    lua.globals().set("C_EncodingUtil", c_encoding)?;
    Ok(())
}

/// Register legacy global item functions (GetItemInfo, GetItemID, etc.).
fn register_legacy_item_globals(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetItemInfo",
        lua.create_function(|_, _item_id: Value| Ok(Value::Nil))?,
    )?;

    globals.set(
        "GetItemID",
        lua.create_function(|_, item_link: Option<String>| {
            Ok(item_link.and_then(|link| parse_item_id_from_link(&link)))
        })?,
    )?;

    globals.set(
        "GetItemCount",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(0))?,
    )?;

    globals.set(
        "GetItemClassInfo",
        lua.create_function(|lua, class_id: i32| {
            let name = item_class_name_extended(class_id);
            if name.is_empty() {
                Ok(Value::Nil)
            } else {
                Ok(Value::String(lua.create_string(name)?))
            }
        })?,
    )?;

    globals.set(
        "GetItemSpecInfo",
        lua.create_function(|_, _item_id: i32| Ok(Value::Nil))?,
    )?;

    globals.set(
        "IsArtifactRelicItem",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;

    globals.set(
        "GetTradeSkillTexture",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;

    Ok(())
}

/// Register spell-related global functions.
fn register_spell_globals(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetSpellLink",
        lua.create_function(|lua, spell_id: i32| {
            let link = format!("|Hspell:{}|h[Spell {}]|h", spell_id, spell_id);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;

    globals.set(
        "GetSpellIcon",
        lua.create_function(|_, _spell_id: i32| Ok(136243))?,
    )?;

    globals.set(
        "GetSpellTexture",
        lua.create_function(|_, _spell_id: i32| Ok(136243))?,
    )?;

    globals.set(
        "GetSpellCooldown",
        lua.create_function(|_, _spell_id: Value| Ok((0.0_f64, 0.0_f64, 1, 1.0_f64)))?,
    )?;

    globals.set(
        "IsSpellKnown",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?,
    )?;

    globals.set(
        "IsPlayerSpell",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;

    globals.set(
        "IsSpellKnownOrOverridesKnown",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;

    globals.set(
        "SendChatMessage",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;

    Ok(())
}

/// Register legacy global container functions.
fn register_container_globals(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetContainerNumSlots",
        lua.create_function(|_, bag: i32| Ok(if bag == 0 { 16 } else { 0 }))?,
    )?;
    globals.set(
        "GetContainerItemID",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetContainerItemLink",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;

    Ok(())
}

/// Register inventory slot functions.
fn register_inventory_globals(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetInventorySlotInfo",
        lua.create_function(|_, slot_name: String| Ok(inventory_slot_id(&slot_name)))?,
    )?;
    globals.set(
        "GetInventoryItemLink",
        lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetInventoryItemID",
        lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetInventoryItemTexture",
        lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetInventoryItemCount",
        lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(0))?,
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse an item ID from a WoW item link string (e.g. "|Hitem:12345:...|h").
fn parse_item_id_from_link(link: &str) -> Option<i32> {
    let start = link.find("|Hitem:")? + 7;
    let rest = &link[start..];
    let end = rest.find(':')?;
    rest[..end].parse().ok()
}

/// Parse an item ID from a Lua Value (integer, number, or item link string).
fn parse_item_id_from_value(value: &Value) -> i32 {
    match value {
        Value::Integer(n) => *n as i32,
        Value::Number(n) => *n as i32,
        Value::String(s) => {
            if let Ok(s) = s.to_str() {
                parse_item_id_from_link(&s).unwrap_or(0)
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Map item class ID to name (C_Item version — always returns a value).
fn item_class_name(class_id: i32) -> &'static str {
    match class_id {
        0 => "Consumable",
        1 => "Container",
        2 => "Weapon",
        3 => "Gem",
        4 => "Armor",
        5 => "Reagent",
        6 => "Projectile",
        7 => "Tradeskill",
        8 => "Item Enhancement",
        9 => "Recipe",
        10 => "Currency (Obsolete)",
        11 => "Quiver",
        12 => "Quest",
        13 => "Key",
        14 => "Permanent (Obsolete)",
        15 => "Miscellaneous",
        16 => "Glyph",
        17 => "Battle Pets",
        18 => "WoW Token",
        _ => "Unknown",
    }
}

/// Map item class ID to name (legacy global version — includes Profession, returns empty for unknown).
fn item_class_name_extended(class_id: i32) -> &'static str {
    match class_id {
        0 => "Consumable",
        1 => "Container",
        2 => "Weapon",
        3 => "Gem",
        4 => "Armor",
        5 => "Reagent",
        6 => "Projectile",
        7 => "Tradeskill",
        8 => "Item Enhancement",
        9 => "Recipe",
        10 => "Currency (deprecated)",
        11 => "Quiver",
        12 => "Quest",
        13 => "Key",
        14 => "Permanent (deprecated)",
        15 => "Miscellaneous",
        16 => "Glyph",
        17 => "Battle Pets",
        18 => "WoW Token",
        19 => "Profession",
        _ => "",
    }
}

/// Map item subclass to name for weapon/armor classes.
fn item_subclass_name(class_id: i32, subclass_id: i32) -> &'static str {
    match (class_id, subclass_id) {
        (2, 0) => "One-Handed Axes",
        (2, 1) => "Two-Handed Axes",
        (2, 2) => "Bows",
        (2, 3) => "Guns",
        (2, 4) => "One-Handed Maces",
        (2, 5) => "Two-Handed Maces",
        (2, 6) => "Polearms",
        (2, 7) => "One-Handed Swords",
        (2, 8) => "Two-Handed Swords",
        (2, 9) => "Warglaives",
        (2, 10) => "Staves",
        (2, 13) => "Fist Weapons",
        (2, 14) => "Miscellaneous",
        (2, 15) => "Daggers",
        (2, 16) => "Thrown",
        (2, 18) => "Crossbows",
        (2, 19) => "Wands",
        (2, 20) => "Fishing Poles",
        (4, 0) => "Miscellaneous",
        (4, 1) => "Cloth",
        (4, 2) => "Leather",
        (4, 3) => "Mail",
        (4, 4) => "Plate",
        (4, 6) => "Shield",
        _ => "Unknown",
    }
}

/// Map inventory slot name to slot ID.
fn inventory_slot_id(slot_name: &str) -> i32 {
    match slot_name {
        "HeadSlot" => 1,
        "NeckSlot" => 2,
        "ShoulderSlot" => 3,
        "BackSlot" => 15,
        "ChestSlot" => 5,
        "ShirtSlot" => 4,
        "TabardSlot" => 19,
        "WristSlot" => 9,
        "HandsSlot" => 10,
        "WaistSlot" => 6,
        "LegsSlot" => 7,
        "FeetSlot" => 8,
        "Finger0Slot" => 11,
        "Finger1Slot" => 12,
        "Trinket0Slot" => 13,
        "Trinket1Slot" => 14,
        "MainHandSlot" => 16,
        "SecondaryHandSlot" => 17,
        "RangedSlot" => 18,
        "AmmoSlot" => 0,
        _ => 0,
    }
}
