//! Spell-related C_* namespaces.
//!
//! - `C_SpellBook` - Spell book functions
//! - `C_Spell` - Spell information functions
//! - `C_Traits` - Talent/loadout system (Dragonflight+)

use mlua::{Lua, Result, Value};

pub fn register_spell_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // C_SpellBook namespace - spell book functions
    let c_spell_book = lua.create_table()?;
    c_spell_book.set(
        "GetSpellBookItemName",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    c_spell_book.set(
        "GetNumSpellBookSkillLines",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_spell_book.set(
        "GetSpellBookSkillLineInfo",
        lua.create_function(|_, _tab: i32| Ok(Value::Nil))?,
    )?;
    c_spell_book.set(
        "GetSpellBookItemInfo",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    c_spell_book.set(
        "HasPetSpells",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_spell_book.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;
    c_spell_book.set(
        "IsSpellKnown",
        lua.create_function(|_, (_spell_id, _is_pet): (i32, Option<bool>)| Ok(false))?,
    )?;
    globals.set("C_SpellBook", c_spell_book)?;

    // C_Spell namespace - spell information
    let c_spell = lua.create_table()?;
    // C_Spell.GetSpellInfo(spellID) - returns a SpellInfo table in modern API
    c_spell.set(
        "GetSpellInfo",
        lua.create_function(|lua, spell_id: i32| {
            // Return a spell info table with common fields
            let info = lua.create_table()?;
            info.set("name", format!("Spell {}", spell_id))?;
            info.set("spellID", spell_id)?;
            info.set("iconID", 136243)?; // INV_Misc_QuestionMark
            info.set("castTime", 0)?;
            info.set("minRange", 0)?;
            info.set("maxRange", 0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    // C_Spell.GetSpellCharges(spellID) - returns charges info
    c_spell.set(
        "GetSpellCharges",
        lua.create_function(|lua, _spell_id: i32| {
            // Return a charges table (most spells don't have charges)
            let info = lua.create_table()?;
            info.set("currentCharges", 0)?;
            info.set("maxCharges", 0)?;
            info.set("cooldownStartTime", 0)?;
            info.set("cooldownDuration", 0)?;
            info.set("chargeModRate", 1.0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_spell.set(
        "IsSpellPassive",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;
    c_spell.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;
    c_spell.set(
        "GetSchoolString",
        lua.create_function(|lua, school_mask: i32| {
            // WoW spell school bitmask to name
            let name = match school_mask {
                1 => "Physical",
                2 => "Holy",
                4 => "Fire",
                8 => "Nature",
                16 => "Frost",
                32 => "Shadow",
                64 => "Arcane",
                _ => "Unknown",
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    c_spell.set(
        "GetSpellTexture",
        lua.create_function(|_, _spell_id: i32| {
            // Return a generic spell icon ID
            Ok(136243) // INV_Misc_QuestionMark
        })?,
    )?;
    c_spell.set(
        "GetSpellLink",
        lua.create_function(|lua, spell_id: i32| {
            let link = format!("|cff71d5ff|Hspell:{}|h[Spell {}]|h|r", spell_id, spell_id);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;
    c_spell.set(
        "GetSpellName",
        lua.create_function(|lua, spell_id: i32| {
            // Return spell name (just "Spell {id}" for simulation)
            Ok(Value::String(lua.create_string(&format!("Spell {}", spell_id))?))
        })?,
    )?;
    c_spell.set(
        "GetSpellCooldown",
        lua.create_function(|lua, _spell_id: i32| {
            // Returns cooldown info table: { startTime, duration, isEnabled, modRate }
            let info = lua.create_table()?;
            info.set("startTime", 0.0)?;
            info.set("duration", 0.0)?;
            info.set("isEnabled", true)?;
            info.set("modRate", 1.0)?;
            Ok(info)
        })?,
    )?;
    c_spell.set(
        "DoesSpellExist",
        lua.create_function(|_, spell_id: i32| {
            // In simulation, assume all positive spell IDs exist
            Ok(spell_id > 0)
        })?,
    )?;
    globals.set("C_Spell", c_spell)?;

    // C_Traits namespace - talent/loadout system (Dragonflight+)
    let c_traits = lua.create_table()?;
    c_traits.set(
        "GenerateImportString",
        lua.create_function(|_, _config_id: i32| {
            // Return a dummy talent string
            Ok("dummy_talent_string".to_string())
        })?,
    )?;
    c_traits.set(
        "GetConfigIDBySystemID",
        lua.create_function(|_, _system_id: i32| Ok(0))?,
    )?;
    c_traits.set(
        "GetConfigIDByTreeID",
        lua.create_function(|_, _tree_id: i32| Ok(0))?,
    )?;
    c_traits.set(
        "GetConfigInfo",
        lua.create_function(|lua, _config_id: i32| {
            // Return a stub config info table with empty treeIDs
            let info = lua.create_table()?;
            info.set("treeIDs", lua.create_table()?)?; // Empty array
            info.set("ID", 0)?;
            info.set("type", 1)?; // Enum.TraitConfigType.Combat
            info.set("name", "")?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_traits.set(
        "GetNodeInfo",
        lua.create_function(|_, (_config_id, _node_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetEntryInfo",
        lua.create_function(|_, (_config_id, _entry_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetDefinitionInfo",
        lua.create_function(|_, _def_id: i32| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "InitializeViewLoadout",
        lua.create_function(|_, (_config_id, _tree_id): (i32, i32)| Ok(true))?,
    )?;
    c_traits.set(
        "GetTreeInfo",
        lua.create_function(|_, _config_id: i32| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetTreeNodes",
        lua.create_function(|lua, _tree_id: i32| lua.create_table())?,
    )?;
    c_traits.set(
        "GetTreeCurrencyInfo",
        lua.create_function(|_, (_tree_id, _currency_type): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetAllTreeIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_traits.set(
        "GetTraitSystemFlags",
        lua.create_function(|_, _system_id: i32| Ok(0))?,
    )?;
    c_traits.set(
        "CanPurchaseRank",
        lua.create_function(|_, (_config_id, _node_id, _entry_id): (i32, i32, i32)| Ok(false))?,
    )?;
    c_traits.set(
        "GetLoadoutSerializationVersion",
        lua.create_function(|_, ()| Ok(2i32))?,
    )?;
    globals.set("C_Traits", c_traits)?;

    Ok(())
}
