//! C_Item namespace and item-related API functions.
//!
//! Contains item information, container, encoding utilities, and inventory slot functions.

use mlua::{Lua, Result, Value};

/// Register item-related C_* namespaces and global functions.
pub fn register_c_item_api(lua: &Lua) -> Result<()> {
    register_c_item(lua)?;
    super::c_container_api::register_c_container_api(lua)?;
    register_c_encoding_util(lua)?;
    register_legacy_item_globals(lua)?;
    register_spell_globals(lua)?;
    register_inventory_globals(lua)?;
    Ok(())
}

/// Register the C_Item namespace.
fn register_c_item(lua: &Lua) -> Result<()> {
    let c_item = lua.create_table()?;
    register_c_item_info_methods(lua, &c_item)?;
    register_c_item_query_methods(lua, &c_item)?;
    register_c_item_link_methods(lua, &c_item)?;
    register_c_item_stub_methods(lua, &c_item)?;
    lua.globals().set("C_Item", c_item)?;
    Ok(())
}

/// C_Item methods: GetItemInfo, GetItemInfoInstant, GetItemIDForItemInfo.
fn register_c_item_info_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetItemInfo", make_c_item_get_item_info(lua)?)?;
    t.set(
        "GetItemInfoInstant",
        lua.create_function(|lua, item_id: Value| {
            let id = parse_item_id_from_value(&item_id);
            if id == 0 {
                return Ok(mlua::MultiValue::new());
            }
            let (class_name, subclass_name) =
                if let Some(item) = crate::items::get_item(id as u32) {
                    (
                        item_class_from_inv_type(item.inventory_type),
                        inv_type_to_subclass(item.inventory_type),
                    )
                } else {
                    ("Miscellaneous", "Junk")
                };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(id as i64),
                Value::String(lua.create_string(class_name)?),
                Value::String(lua.create_string(subclass_name)?),
                Value::String(lua.create_string("")?),
                Value::Integer(134400),
                Value::Integer(15),
                Value::Integer(0),
            ]))
        })?,
    )?;
    t.set(
        "GetItemIDForItemInfo",
        lua.create_function(|_, item_id: Value| {
            let id = parse_item_id_from_value(&item_id);
            if id == 0 { Ok(Value::Nil) } else { Ok(Value::Integer(id as i64)) }
        })?,
    )?;
    Ok(())
}

/// Build the C_Item.GetItemInfo closure (returns a table of item properties).
fn make_c_item_get_item_info(lua: &Lua) -> Result<mlua::Function> {
    lua.create_function(|lua, item_id: Value| {
        let id = parse_item_id_from_value(&item_id);
        if id == 0 {
            return Ok(Value::Nil);
        }
        let Some(item) = crate::items::get_item(id as u32) else {
            return Ok(Value::Nil);
        };
        let color = quality_color(item.quality);
        let link = format!(
            "|cff{}|Hitem:{}::::::::80:::::|h[{}]|h|r",
            color, id, item.name
        );
        let result = lua.create_table()?;
        result.set("itemName", item.name)?;
        result.set("itemLink", lua.create_string(&link)?)?;
        result.set("itemQuality", item.quality as i32)?;
        result.set("itemLevel", item.item_level as i32)?;
        result.set("itemMinLevel", item.required_level as i32)?;
        result.set("itemType", item_class_from_inv_type(item.inventory_type))?;
        result.set("itemSubType", "")?;
        result.set("itemStackCount", item.stackable as i32)?;
        result.set("itemEquipLoc", inv_type_to_equip_loc(item.inventory_type))?;
        result.set("itemTexture", 134400)?;
        result.set("sellPrice", item.sell_price as i64)?;
        result.set("classID", 15)?;
        result.set("subclassID", 0)?;
        result.set("bindType", item.bonding as i32)?;
        result.set("expacID", item.expansion_id as i32)?;
        result.set("isCraftingReagent", false)?;
        Ok(Value::Table(result))
    })
}

/// C_Item query methods: icon, subclass, count, class, spec, name, level.
fn register_c_item_query_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetItemIconByID", lua.create_function(|_, _id: i32| Ok(134400i32))?)?;
    t.set(
        "GetItemSubClassInfo",
        lua.create_function(|lua, (class_id, subclass_id): (i32, i32)| {
            Ok(Value::String(lua.create_string(item_subclass_name(class_id, subclass_id))?))
        })?,
    )?;
    t.set(
        "GetItemCount",
        lua.create_function(
            |_, (_id, _b, _c, _r): (Value, Option<bool>, Option<bool>, Option<bool>)| Ok(0),
        )?,
    )?;
    t.set(
        "GetItemClassInfo",
        lua.create_function(|lua, class_id: i32| {
            Ok(Value::String(lua.create_string(item_class_name(class_id))?))
        })?,
    )?;
    t.set("GetItemSpecInfo", lua.create_function(|lua, _id: Value| lua.create_table())?)?;
    t.set(
        "GetItemNameByID",
        lua.create_function(|lua, item_id: i32| {
            let name = crate::items::get_item(item_id as u32).map(|i| i.name).unwrap_or("Unknown");
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    t.set(
        "GetDetailedItemLevelInfo",
        lua.create_function(|_, item_link: Value| {
            let id = parse_item_id_from_value(&item_link);
            let level = crate::items::get_item(id as u32)
                .map(|i| i.item_level as i32)
                .unwrap_or(0);
            Ok((level, 0i32, level))
        })?,
    )?;
    Ok(())
}

/// C_Item link and quality methods.
fn register_c_item_link_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("IsItemBindToAccountUntilEquip", lua.create_function(|_, _v: Value| Ok(false))?)?;
    t.set(
        "GetItemLink",
        lua.create_function(|lua, item_id: i32| {
            let (name, color) = if let Some(item) = crate::items::get_item(item_id as u32) {
                (item.name, quality_color(item.quality))
            } else {
                ("Unknown", "ffffff")
            };
            let link = format!("|cff{}|Hitem:{}::::::::80:::::|h[{}]|h|r", color, item_id, name);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;
    t.set(
        "GetItemQualityByID",
        lua.create_function(|_, item_id: i32| {
            Ok(crate::items::get_item(item_id as u32).map(|i| i.quality as i32).unwrap_or(1))
        })?,
    )?;
    Ok(())
}

/// C_Item stub methods (transmog, load, existence, sockets).
fn register_c_item_stub_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetItemLearnTransmogSet", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("RequestLoadItemDataByID", lua.create_function(|_, _id: i32| Ok(()))?)?;
    t.set("DoesItemExist", lua.create_function(|_, _loc: Value| Ok(false))?)?;
    t.set("CanViewItemPowers", lua.create_function(|_, _loc: Value| Ok(false))?)?;
    t.set("GetItemNumSockets", lua.create_function(|_, _loc: Value| Ok(0i32))?)?;
    t.set("GetItemGemID", lua.create_function(|_, _args: mlua::MultiValue| Ok(0i32))?)?;
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
    globals.set("GetItemInfo", make_legacy_get_item_info(lua)?)?;
    globals.set(
        "GetItemID",
        lua.create_function(|_, item_link: Option<String>| {
            Ok(item_link.and_then(|link| parse_item_id_from_link(&link)))
        })?,
    )?;
    globals.set("GetItemCount", lua.create_function(|_, _args: mlua::MultiValue| Ok(0))?)?;
    register_legacy_item_stubs(lua)?;
    Ok(())
}

/// Legacy global stubs: GetItemClassInfo, GetItemSpecInfo, IsArtifactRelicItem, etc.
fn register_legacy_item_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set(
        "GetItemClassInfo",
        lua.create_function(|lua, class_id: i32| {
            let name = item_class_name_extended(class_id);
            if name.is_empty() { Ok(Value::Nil) } else { Ok(Value::String(lua.create_string(name)?)) }
        })?,
    )?;
    globals.set("GetItemSpecInfo", lua.create_function(|_, _item_id: i32| Ok(Value::Nil))?)?;
    globals.set("IsArtifactRelicItem", lua.create_function(|_, _item_id: i32| Ok(false))?)?;
    globals.set("GetTradeSkillTexture", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    Ok(())
}

/// Build the legacy global GetItemInfo closure (returns 17 positional values).
fn make_legacy_get_item_info(lua: &Lua) -> Result<mlua::Function> {
    lua.create_function(|lua, item_id: Value| {
        let id = parse_item_id_from_value(&item_id);
        if id == 0 {
            return Ok(mlua::MultiValue::new());
        }
        let Some(item) = crate::items::get_item(id as u32) else {
            return Ok(mlua::MultiValue::new());
        };
        let color = quality_color(item.quality);
        let link = format!(
            "|cff{}|Hitem:{}::::::::80:::::|h[{}]|h|r",
            color, id, item.name
        );
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string(item.name)?),
            Value::String(lua.create_string(&link)?),
            Value::Integer(item.quality as i64),
            Value::Integer(item.item_level as i64),
            Value::Integer(item.required_level as i64),
            Value::String(lua.create_string(item_class_from_inv_type(item.inventory_type))?),
            Value::String(lua.create_string("")?),
            Value::Integer(item.stackable as i64),
            Value::String(lua.create_string(inv_type_to_equip_loc(item.inventory_type))?),
            Value::Integer(134400),
            Value::Integer(item.sell_price as i64),
            Value::Integer(15),
            Value::Integer(0),
            Value::Integer(item.bonding as i64),
            Value::Integer(item.expansion_id as i64),
            Value::Nil,
            Value::Boolean(false),
        ]))
    })
}

/// Register spell-related global functions.
fn register_spell_globals(lua: &Lua) -> Result<()> {
    register_spell_query_globals(lua)?;
    register_spell_stub_globals(lua)?;
    Ok(())
}

/// Spell query globals: link, icon, texture.
fn register_spell_query_globals(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetSpellLink",
        lua.create_function(|lua, spell_id: i32| {
            let name = crate::spells::get_spell(spell_id as u32)
                .map(|s| s.name)
                .unwrap_or("Unknown");
            let link = format!("|cff71d5ff|Hspell:{}|h[{}]|h|r", spell_id, name);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;

    globals.set(
        "GetSpellIcon",
        lua.create_function(|_, spell_id: i32| {
            let icon = crate::spells::get_spell(spell_id as u32)
                .map(|s| s.icon_file_data_id)
                .unwrap_or(136243);
            Ok(icon)
        })?,
    )?;

    globals.set(
        "GetSpellTexture",
        lua.create_function(|_, spell_id: i32| {
            let file_id = crate::spells::get_spell(spell_id as u32)
                .map(|s| s.icon_file_data_id)
                .unwrap_or(136243);
            Ok(crate::manifest_interface_data::get_texture_path(file_id).unwrap_or(""))
        })?,
    )?;

    Ok(())
}

/// Spell stub globals: cooldown, known checks, chat.
fn register_spell_stub_globals(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetSpellCooldown",
        lua.create_function(|_, _spell_id: Value| Ok((0.0_f64, 0.0_f64, 1, 1.0_f64)))?,
    )?;

    globals.set(
        "IsSpellKnown",
        lua.create_function(|_, args: mlua::MultiValue| {
            let spell_id = args.iter().next()
                .and_then(|v| match v { mlua::Value::Integer(n) => Some(*n as u32), _ => None })
                .unwrap_or(0);
            Ok(super::spellbook_data::is_spell_known(spell_id))
        })?,
    )?;

    globals.set(
        "IsPlayerSpell",
        lua.create_function(|_, spell_id: i32| {
            Ok(super::spellbook_data::is_spell_known(spell_id as u32))
        })?,
    )?;

    globals.set(
        "IsSpellKnownOrOverridesKnown",
        lua.create_function(|_, spell_id: i32| {
            Ok(super::spellbook_data::find_spell_slot(spell_id as u32).is_some())
        })?,
    )?;

    globals.set(
        "SpellCanTargetItem",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?,
    )?;
    globals.set(
        "SpellCanTargetItemID",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?,
    )?;

    globals.set(
        "SendChatMessage",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;

    // SpellGetVisibilityInfo(spellId, context) -> hasCustom, alwaysShowMine, showForMySpec
    globals.set(
        "SpellGetVisibilityInfo",
        lua.create_function(|_, (_spell_id, _ctx): (i32, String)| Ok((false, false, false)))?,
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

/// Quality ID to color hex string.
pub(super) fn quality_color(quality: u8) -> &'static str {
    match quality {
        0 => "9d9d9d",
        1 => "ffffff",
        2 => "1eff00",
        3 => "0070dd",
        4 => "a335ee",
        5 => "ff8000",
        6 => "e6cc80",
        7 => "00ccff",
        _ => "ffffff",
    }
}

/// Map inventory type to a rough item class name.
fn item_class_from_inv_type(inv_type: u8) -> &'static str {
    match inv_type {
        13 | 15 | 17 | 21 | 22 | 25 | 26 => "Weapon",
        1..=12 | 14 | 16 | 23 => "Armor",
        _ => "Miscellaneous",
    }
}

/// Map inventory type to a rough subclass name.
fn inv_type_to_subclass(inv_type: u8) -> &'static str {
    match inv_type {
        1 => "Head",
        2 => "Neck",
        3 => "Shoulder",
        4 => "Shirt",
        5 => "Chest",
        6 => "Waist",
        7 => "Legs",
        8 => "Feet",
        9 => "Wrist",
        10 => "Hands",
        11 => "Finger",
        12 => "Trinket",
        14 => "Shield",
        16 => "Back",
        _ => "Junk",
    }
}

/// Map inventory type to WoW equip location string.
fn inv_type_to_equip_loc(inv_type: u8) -> &'static str {
    match inv_type {
        1 => "INVTYPE_HEAD",
        2 => "INVTYPE_NECK",
        3 => "INVTYPE_SHOULDER",
        4 => "INVTYPE_BODY",
        5 => "INVTYPE_CHEST",
        6 => "INVTYPE_WAIST",
        7 => "INVTYPE_LEGS",
        8 => "INVTYPE_FEET",
        9 => "INVTYPE_WRIST",
        10 => "INVTYPE_HAND",
        11 => "INVTYPE_FINGER",
        12 => "INVTYPE_TRINKET",
        13 => "INVTYPE_WEAPON",
        14 => "INVTYPE_SHIELD",
        15 => "INVTYPE_RANGED",
        16 => "INVTYPE_CLOAK",
        17 => "INVTYPE_2HWEAPON",
        20 => "INVTYPE_ROBE",
        21 => "INVTYPE_WEAPONMAINHAND",
        22 => "INVTYPE_WEAPONOFFHAND",
        23 => "INVTYPE_HOLDABLE",
        25 => "INVTYPE_THROWN",
        26 => "INVTYPE_RANGEDRIGHT",
        _ => "",
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
