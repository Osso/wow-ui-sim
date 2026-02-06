//! Spell-related C_* namespaces.
//!
//! - `C_SpellBook` - Spell book functions
//! - `C_Spell` - Spell information functions
//! - `C_Traits` - Talent/loadout system (Dragonflight+)

use mlua::{Lua, Result, Value};

pub fn register_spell_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("C_SpellBook", register_c_spell_book(lua)?)?;
    globals.set("C_Spell", register_c_spell(lua)?)?;
    globals.set("C_Traits", register_c_traits(lua)?)?;

    Ok(())
}

/// C_SpellBook namespace - spell book functions.
fn register_c_spell_book(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GetSpellBookItemName",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    t.set("GetNumSpellBookSkillLines", lua.create_function(|_, ()| Ok(0))?)?;
    t.set("GetSpellBookSkillLineInfo", lua.create_function(|_, _tab: i32| Ok(Value::Nil))?)?;
    t.set(
        "GetSpellBookItemInfo",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    t.set("HasPetSpells", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetOverrideSpell", lua.create_function(|_, spell_id: i32| Ok(spell_id))?)?;
    t.set(
        "IsSpellKnown",
        lua.create_function(|_, (_spell_id, _is_pet): (i32, Option<bool>)| Ok(false))?,
    )?;

    Ok(t)
}

/// C_Spell namespace - spell information.
fn register_c_spell(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set("GetSpellInfo", lua.create_function(create_spell_info)?)?;
    t.set("GetSpellCharges", lua.create_function(create_spell_charges)?)?;
    t.set("IsSpellPassive", lua.create_function(|_, _spell_id: i32| Ok(false))?)?;
    t.set("GetOverrideSpell", lua.create_function(|_, spell_id: i32| Ok(spell_id))?)?;
    t.set("GetSchoolString", lua.create_function(create_school_string)?)?;
    t.set(
        "GetSpellTexture",
        lua.create_function(|_, _spell_id: i32| Ok(136243))?,
    )?;
    t.set(
        "GetSpellLink",
        lua.create_function(|lua, spell_id: i32| {
            let link = format!("|cff71d5ff|Hspell:{}|h[Spell {}]|h|r", spell_id, spell_id);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;
    t.set(
        "GetSpellName",
        lua.create_function(|lua, spell_id: i32| {
            Ok(Value::String(lua.create_string(&format!("Spell {}", spell_id))?))
        })?,
    )?;
    t.set("GetSpellCooldown", lua.create_function(create_spell_cooldown)?)?;
    t.set(
        "DoesSpellExist",
        lua.create_function(|_, spell_id: i32| Ok(spell_id > 0))?,
    )?;

    Ok(t)
}

fn create_spell_info(lua: &Lua, spell_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    info.set("name", format!("Spell {}", spell_id))?;
    info.set("spellID", spell_id)?;
    info.set("iconID", 136243)?;
    info.set("castTime", 0)?;
    info.set("minRange", 0)?;
    info.set("maxRange", 0)?;
    Ok(Value::Table(info))
}

fn create_spell_charges(lua: &Lua, _spell_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    info.set("currentCharges", 0)?;
    info.set("maxCharges", 0)?;
    info.set("cooldownStartTime", 0)?;
    info.set("cooldownDuration", 0)?;
    info.set("chargeModRate", 1.0)?;
    Ok(Value::Table(info))
}

fn create_school_string(lua: &Lua, school_mask: i32) -> Result<Value> {
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
}

fn create_spell_cooldown(lua: &Lua, _spell_id: i32) -> Result<mlua::Table> {
    let info = lua.create_table()?;
    info.set("startTime", 0.0)?;
    info.set("duration", 0.0)?;
    info.set("isEnabled", true)?;
    info.set("modRate", 1.0)?;
    Ok(info)
}

/// C_Traits namespace - talent/loadout system (Dragonflight+).
fn register_c_traits(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GenerateImportString",
        lua.create_function(|_, _config_id: i32| Ok("dummy_talent_string".to_string()))?,
    )?;
    t.set("GetConfigIDBySystemID", lua.create_function(|_, _system_id: i32| Ok(0))?)?;
    t.set("GetConfigIDByTreeID", lua.create_function(|_, _tree_id: i32| Ok(0))?)?;
    t.set("GetConfigInfo", lua.create_function(create_config_info)?)?;
    t.set(
        "GetNodeInfo",
        lua.create_function(|_, (_config_id, _node_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    t.set(
        "GetEntryInfo",
        lua.create_function(|_, (_config_id, _entry_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    t.set("GetDefinitionInfo", lua.create_function(|_, _def_id: i32| Ok(Value::Nil))?)?;
    t.set(
        "InitializeViewLoadout",
        lua.create_function(|_, (_config_id, _tree_id): (i32, i32)| Ok(true))?,
    )?;
    t.set("GetTreeInfo", lua.create_function(|_, _config_id: i32| Ok(Value::Nil))?)?;
    t.set("GetTreeNodes", lua.create_function(|lua, _tree_id: i32| lua.create_table())?)?;
    t.set(
        "GetTreeCurrencyInfo",
        lua.create_function(|_, (_tree_id, _currency_type): (i32, i32)| Ok(Value::Nil))?,
    )?;
    t.set("GetAllTreeIDs", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetTraitSystemFlags", lua.create_function(|_, _system_id: i32| Ok(0))?)?;
    t.set(
        "CanPurchaseRank",
        lua.create_function(|_, (_config_id, _node_id, _entry_id): (i32, i32, i32)| Ok(false))?,
    )?;
    t.set("GetLoadoutSerializationVersion", lua.create_function(|_, ()| Ok(2i32))?)?;

    Ok(t)
}

fn create_config_info(lua: &Lua, _config_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    info.set("treeIDs", lua.create_table()?)?;
    info.set("ID", 0)?;
    info.set("type", 1)?;
    info.set("name", "")?;
    Ok(Value::Table(info))
}
