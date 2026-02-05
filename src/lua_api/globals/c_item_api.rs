//! C_Item namespace and item-related API functions.
//!
//! Contains item information, container, encoding utilities, and inventory slot functions.

use mlua::{Lua, Result, Value};

/// Register item-related C_* namespaces and global functions.
pub fn register_c_item_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // C_Item namespace - item information
    let c_item = lua.create_table()?;
    c_item.set(
        "GetItemInfo",
        lua.create_function(|_, _item_id: Value| {
            // Return nil - no item info in simulation
            Ok(Value::Nil)
        })?,
    )?;
    c_item.set(
        "GetItemInfoInstant",
        lua.create_function(|lua, item_id: Value| {
            // GetItemInfoInstant returns: itemID, itemType, itemSubType, itemEquipLoc, icon, classID, subClassID
            // We only have item ID, so return that with stub values
            let id = match item_id {
                Value::Integer(n) => n as i32,
                Value::Number(n) => n as i32,
                Value::String(s) => {
                    // Could be item link or name
                    if let Ok(s) = s.to_str() {
                        if let Some(start) = s.find("|Hitem:") {
                            let rest = &s[start + 7..];
                            if let Some(end) = rest.find(':') {
                                rest[..end].parse().unwrap_or(0)
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                _ => return Ok(mlua::MultiValue::new()),
            };
            if id == 0 {
                return Ok(mlua::MultiValue::new());
            }
            // Return: itemID, itemType, itemSubType, itemEquipLoc, icon, classID, subClassID
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(id as i64),       // itemID
                Value::String(lua.create_string("Miscellaneous")?), // itemType
                Value::String(lua.create_string("Junk")?), // itemSubType
                Value::String(lua.create_string("")?), // itemEquipLoc
                Value::Integer(134400),          // icon (INV_Misc_Bag_07)
                Value::Integer(15),              // classID (Miscellaneous)
                Value::Integer(0),               // subClassID
            ]))
        })?,
    )?;
    c_item.set(
        "GetItemIDForItemInfo",
        lua.create_function(|_, item_id: Value| {
            // GetItemIDForItemInfo extracts item ID from itemID, name, or link
            let id = match item_id {
                Value::Integer(n) => n as i32,
                Value::Number(n) => n as i32,
                Value::String(s) => {
                    if let Ok(s) = s.to_str() {
                        if let Some(start) = s.find("|Hitem:") {
                            let rest = &s[start + 7..];
                            if let Some(end) = rest.find(':') {
                                rest[..end].parse().unwrap_or(0)
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                _ => 0,
            };
            if id == 0 {
                Ok(Value::Nil)
            } else {
                Ok(Value::Integer(id as i64))
            }
        })?,
    )?;
    c_item.set(
        "GetItemIconByID",
        lua.create_function(|_, _item_id: i32| Ok(134400i32))?, // INV_Misc_Bag_07
    )?;
    c_item.set(
        "GetItemSubClassInfo",
        lua.create_function(|lua, (class_id, subclass_id): (i32, i32)| {
            // Return item subclass name based on class/subclass IDs
            let name = match (class_id, subclass_id) {
                // Weapons (class 2)
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
                // Armor (class 4)
                (4, 0) => "Miscellaneous",
                (4, 1) => "Cloth",
                (4, 2) => "Leather",
                (4, 3) => "Mail",
                (4, 4) => "Plate",
                (4, 6) => "Shield",
                _ => "Unknown",
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    c_item.set(
        "GetItemCount",
        lua.create_function(|_, (_item_id, _include_bank, _include_charges, _include_reagent_bank): (Value, Option<bool>, Option<bool>, Option<bool>)| {
            // No items in simulation
            Ok(0)
        })?,
    )?;
    c_item.set(
        "GetItemClassInfo",
        lua.create_function(|lua, class_id: i32| {
            let name = match class_id {
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
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    c_item.set(
        "GetItemSpecInfo",
        lua.create_function(|lua, _item_id: Value| {
            // Returns a table of spec IDs that can use this item, or nil if all specs can
            lua.create_table()
        })?,
    )?;
    c_item.set(
        "GetItemNameByID",
        lua.create_function(|lua, item_id: i32| {
            Ok(Value::String(lua.create_string(&format!("Item {}", item_id))?))
        })?,
    )?;
    c_item.set(
        "GetDetailedItemLevelInfo",
        lua.create_function(|_, _item_link: Value| {
            // Returns: actualItemLevel, previewLevel, sparseItemLevel
            Ok((0i32, 0i32, 0i32))
        })?,
    )?;
    c_item.set(
        "IsItemBindToAccountUntilEquip",
        lua.create_function(|_, _item_link: Value| Ok(false))?,
    )?;
    c_item.set(
        "GetItemLink",
        lua.create_function(|lua, item_id: i32| {
            let link = format!("|cffffffff|Hitem:{}::::::::80:::::|h[Item {}]|h|r", item_id, item_id);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;
    c_item.set(
        "GetItemQualityByID",
        lua.create_function(|_, _item_id: i32| Ok(1i32))?, // Common quality
    )?;
    c_item.set(
        "GetItemLearnTransmogSet",
        lua.create_function(|_, _item_id: i32| {
            // Returns nil if item doesn't teach a transmog set
            Ok(Value::Nil)
        })?,
    )?;
    c_item.set(
        "RequestLoadItemDataByID",
        lua.create_function(|_, _item_id: i32| {
            // Request asynchronous item data loading - stub that does nothing
            Ok(())
        })?,
    )?;
    globals.set("C_Item", c_item)?;

    // Legacy global GetItemInfo
    globals.set(
        "GetItemInfo",
        lua.create_function(|_, _item_id: Value| Ok(Value::Nil))?,
    )?;

    // GetItemID(itemLink) - Extract item ID from item link
    globals.set(
        "GetItemID",
        lua.create_function(|_, item_link: Option<String>| {
            // Parse item link format: |Hitem:12345:...| and extract 12345
            if let Some(link) = item_link {
                if let Some(start) = link.find("|Hitem:") {
                    let rest = &link[start + 7..];
                    if let Some(end) = rest.find(':') {
                        if let Ok(id) = rest[..end].parse::<i32>() {
                            return Ok(Some(id));
                        }
                    }
                }
            }
            Ok(None)
        })?,
    )?;

    // GetItemCount(itemID, includeBankItems, includeCharges) - Get count of item in inventory
    globals.set(
        "GetItemCount",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(0))?,
    )?;

    // GetItemClassInfo(classID) - Get item class name
    globals.set(
        "GetItemClassInfo",
        lua.create_function(|lua, class_id: i32| {
            let name = match class_id {
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
            };
            if name.is_empty() {
                Ok(Value::Nil)
            } else {
                Ok(Value::String(lua.create_string(name)?))
            }
        })?,
    )?;

    // GetItemSpecInfo(itemID) - Get spec info for item
    globals.set(
        "GetItemSpecInfo",
        lua.create_function(|_, _item_id: i32| Ok(Value::Nil))?,
    )?;

    // IsArtifactRelicItem(itemID) - Check if item is artifact relic
    globals.set(
        "IsArtifactRelicItem",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;

    // GetTradeSkillTexture(index) - Get tradeskill icon
    globals.set(
        "GetTradeSkillTexture",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;

    // GetSpellLink(spellID) - Get spell link
    globals.set(
        "GetSpellLink",
        lua.create_function(|lua, spell_id: i32| {
            // Return a basic spell link format
            let link = format!("|Hspell:{}|h[Spell {}]|h", spell_id, spell_id);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;

    // GetSpellIcon(spellID) - Get spell icon texture
    globals.set(
        "GetSpellIcon",
        lua.create_function(|_, _spell_id: i32| Ok(136243))?, // INV_Misc_QuestionMark
    )?;

    // GetSpellTexture(spellID) - Get spell icon texture (alternative API)
    globals.set(
        "GetSpellTexture",
        lua.create_function(|_, _spell_id: i32| Ok(136243))?, // INV_Misc_QuestionMark
    )?;

    // GetSpellCooldown(spellID) - Get spell cooldown info
    globals.set(
        "GetSpellCooldown",
        lua.create_function(|_, _spell_id: Value| {
            // Return: start, duration, enabled, modRate
            Ok((0.0_f64, 0.0_f64, 1, 1.0_f64))
        })?,
    )?;

    // IsSpellKnown(spellID, isPetSpell) - Check if spell is known
    globals.set(
        "IsSpellKnown",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?,
    )?;

    // IsPlayerSpell(spellID) - Check if spell is a player spell
    globals.set(
        "IsPlayerSpell",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;

    // IsSpellKnownOrOverridesKnown(spellID) - Check if spell or override is known
    globals.set(
        "IsSpellKnownOrOverridesKnown",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;

    // SendChatMessage(msg, chatType, languageID, target) - Send chat message
    globals.set(
        "SendChatMessage",
        lua.create_function(|_, _args: mlua::MultiValue| {
            // No-op in simulation
            Ok(())
        })?,
    )?;

    // C_Container namespace - bag/container functions
    let c_container = lua.create_table()?;
    c_container.set(
        "GetContainerNumSlots",
        lua.create_function(|_, bag: i32| {
            // Return bag slot counts (0 = backpack has 16 slots, bags 1-4 vary)
            Ok(if bag == 0 { 16 } else { 0 })
        })?,
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
    globals.set("C_Container", c_container)?;

    // C_EncodingUtil namespace - string encoding/compression utilities
    // These are stubs - actual compression/encoding not implemented
    let c_encoding_util = lua.create_table()?;
    c_encoding_util.set(
        "CompressString",
        lua.create_function(|lua, (data, _method): (String, Option<i32>)| {
            // Return the data as-is (no actual compression in simulator)
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    c_encoding_util.set(
        "DecompressString",
        lua.create_function(|lua, (data, _method): (String, Option<i32>)| {
            // Return the data as-is (no actual decompression in simulator)
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    c_encoding_util.set(
        "EncodeBase64",
        lua.create_function(|lua, data: String| {
            // Return the data as-is (no actual base64 encoding)
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    c_encoding_util.set(
        "DecodeBase64",
        lua.create_function(|lua, data: String| {
            // Return the data as-is (no actual base64 decoding)
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    globals.set("C_EncodingUtil", c_encoding_util)?;

    // Legacy global container functions
    let get_container_num_slots = lua.create_function(|_, bag: i32| {
        Ok(if bag == 0 { 16 } else { 0 })
    })?;
    globals.set("GetContainerNumSlots", get_container_num_slots)?;
    globals.set(
        "GetContainerItemID",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetContainerItemLink",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;

    // Inventory slot functions
    globals.set(
        "GetInventorySlotInfo",
        lua.create_function(|_, slot_name: String| {
            // Return slot ID for known slot names
            let slot_id = match slot_name.as_str() {
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
            };
            Ok(slot_id)
        })?,
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
