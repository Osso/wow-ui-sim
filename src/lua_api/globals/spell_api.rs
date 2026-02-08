//! Spell-related C_* namespaces.
//!
//! - `C_SpellBook` - Spell book functions (backed by spellbook_data)
//! - `C_Spell` - Spell information functions
//! - `C_Traits` - Talent/loadout system (Dragonflight+)

use super::spellbook_data;
use mlua::{Lua, Result, Value};

pub fn register_spell_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("C_SpellBook", register_c_spell_book(lua)?)?;
    globals.set("C_Spell", register_c_spell(lua)?)?;
    globals.set("C_Traits", register_c_traits(lua)?)?;

    Ok(())
}

/// C_SpellBook namespace - spell book functions backed by paladin spellbook data.
fn register_c_spell_book(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GetNumSpellBookSkillLines",
        lua.create_function(|_, ()| Ok(spellbook_data::num_skill_lines()))?,
    )?;
    t.set(
        "GetSpellBookSkillLineInfo",
        lua.create_function(create_skill_line_info)?,
    )?;
    t.set(
        "GetSpellBookItemInfo",
        lua.create_function(create_spell_book_item_info)?,
    )?;
    t.set(
        "GetSpellBookItemName",
        lua.create_function(create_spell_book_item_name)?,
    )?;
    t.set(
        "GetSpellBookItemType",
        lua.create_function(create_spell_book_item_type)?,
    )?;
    t.set(
        "GetSpellBookItemLevelLearned",
        lua.create_function(|_, (_slot, _bank): (i32, Option<i32>)| {
            Ok(1i32) // All spells learned at level 1 for now
        })?,
    )?;
    t.set(
        "IsSpellBookItemPassive",
        lua.create_function(|_, (slot, _bank): (i32, Option<i32>)| {
            Ok(spellbook_data::get_spell_at_slot(slot)
                .is_some_and(|(_, entry, _)| entry.is_passive))
        })?,
    )?;
    t.set(
        "GetSpellBookItemCooldown",
        lua.create_function(create_spell_book_item_cooldown)?,
    )?;
    t.set(
        "GetSpellBookItemAutoCast",
        lua.create_function(|_, (_slot, _bank): (i32, Option<i32>)| {
            Ok((false, false)) // autoCastAllowed, autoCastEnabled
        })?,
    )?;

    register_c_spell_book_queries(&t, lua)?;
    Ok(t)
}

/// Spell knowledge and lookup queries for C_SpellBook.
fn register_c_spell_book_queries(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set("HasPetSpells", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;
    t.set(
        "IsSpellKnown",
        lua.create_function(|_, (spell_id, _bank): (i32, Option<i32>)| {
            Ok(spellbook_data::is_spell_known(spell_id as u32))
        })?,
    )?;
    t.set(
        "IsSpellInSpellBook",
        lua.create_function(
            |_, (spell_id, _bank, _include_overrides): (i32, Option<i32>, Option<bool>)| {
                Ok(spellbook_data::find_spell_slot(spell_id as u32).is_some())
            },
        )?,
    )?;
    t.set(
        "FindSpellBookSlotForSpell",
        lua.create_function(|_, (spell_id, _bank): (i32, Option<i32>)| {
            match spellbook_data::find_spell_slot(spell_id as u32) {
                Some((slot, bank)) => Ok(mlua::MultiValue::from_vec(vec![
                    Value::Integer(slot as i64),
                    Value::Integer(bank as i64),
                ])),
                None => Ok(mlua::MultiValue::new()),
            }
        })?,
    )?;
    Ok(())
}

fn create_spell_book_item_name(
    lua: &Lua,
    (slot, _bank): (i32, Option<i32>),
) -> Result<mlua::MultiValue> {
    let Some((_, entry, _)) = spellbook_data::get_spell_at_slot(slot) else {
        return Ok(mlua::MultiValue::new());
    };
    let spell = crate::spells::get_spell(entry.spell_id);
    let name = spell.map(|s| s.name).unwrap_or("Unknown");
    let subtext = spell.map(|s| s.subtext).unwrap_or("");
    Ok(mlua::MultiValue::from_vec(vec![
        Value::String(lua.create_string(name)?),
        Value::String(lua.create_string(subtext)?),
    ]))
}

fn create_spell_book_item_type(
    _: &Lua,
    (slot, _bank): (i32, Option<i32>),
) -> Result<mlua::MultiValue> {
    let Some((_, entry, skill_line)) = spellbook_data::get_spell_at_slot(slot) else {
        return Ok(mlua::MultiValue::new());
    };
    // itemType: 1=Spell, 2=FutureSpell (off-spec unlearned)
    let item_type = if skill_line.off_spec_id.is_some() {
        2
    } else {
        1
    };
    Ok(mlua::MultiValue::from_vec(vec![
        Value::Integer(item_type),
        Value::Integer(entry.spell_id as i64),
    ]))
}

fn create_spell_book_item_cooldown(lua: &Lua, (_slot, _bank): (i32, Option<i32>)) -> Result<Value> {
    let info = lua.create_table()?;
    info.set("startTime", 0.0)?;
    info.set("duration", 0.0)?;
    info.set("isEnabled", true)?;
    info.set("modRate", 1.0)?;
    Ok(Value::Table(info))
}

/// Build a SpellBookSkillLineInfo table for a skill line index.
fn create_skill_line_info(lua: &Lua, index: i32) -> Result<Value> {
    let Some(skill_line) = spellbook_data::get_skill_line(index) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("name", skill_line.name)?;
    info.set("iconID", skill_line.icon_id as i64)?;
    info.set("itemIndexOffset", spellbook_data::skill_line_offset(index))?;
    info.set("numSpellBookItems", skill_line.spells.len() as i32)?;
    info.set("isGuild", false)?;
    info.set("shouldHide", false)?;
    if let Some(spec_id) = skill_line.spec_id {
        info.set("specID", spec_id)?;
    }
    if let Some(off_spec_id) = skill_line.off_spec_id {
        info.set("offSpecID", off_spec_id)?;
    }
    Ok(Value::Table(info))
}

/// Build a SpellBookItemInfo table for a slot index.
fn create_spell_book_item_info(lua: &Lua, (slot, _bank): (i32, Option<i32>)) -> Result<Value> {
    let Some((skill_line_idx, entry, skill_line)) = spellbook_data::get_spell_at_slot(slot) else {
        return Ok(Value::Nil);
    };
    let spell = crate::spells::get_spell(entry.spell_id);
    let name = spell.map(|s| s.name).unwrap_or("Unknown");
    let subtext = spell.map(|s| s.subtext).unwrap_or("");
    let icon_id = spell.map(|s| s.icon_file_data_id).unwrap_or(136243);
    let is_off_spec = skill_line.off_spec_id.is_some();
    let item_type = if is_off_spec { 2 } else { 1 }; // FutureSpell for off-spec

    let info = lua.create_table()?;
    info.set("actionID", entry.spell_id as i64)?;
    info.set("spellID", entry.spell_id as i64)?;
    info.set("itemType", item_type)?;
    info.set("name", name)?;
    info.set("subName", subtext)?;
    info.set("iconID", icon_id as i64)?;
    info.set("isPassive", entry.is_passive)?;
    info.set("isOffSpec", is_off_spec)?;
    info.set("skillLineIndex", skill_line_idx)?;
    Ok(Value::Table(info))
}

/// C_Spell namespace - spell information.
fn register_c_spell(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set("GetSpellInfo", lua.create_function(create_spell_info)?)?;
    t.set("GetSpellCharges", lua.create_function(create_spell_charges)?)?;
    t.set("IsSpellPassive", lua.create_function(|_, spell_id: i32| {
        Ok(spellbook_data::find_spell_slot(spell_id as u32)
            .and_then(|(slot, _)| spellbook_data::get_spell_at_slot(slot))
            .is_some_and(|(_, entry, _)| entry.is_passive))
    })?)?;
    t.set("GetOverrideSpell", lua.create_function(|_, spell_id: i32| Ok(spell_id))?)?;
    t.set("GetSchoolString", lua.create_function(create_school_string)?)?;
    t.set("GetSpellTexture", lua.create_function(create_spell_texture)?)?;
    t.set("GetSpellLink", lua.create_function(create_spell_link)?)?;
    t.set("GetSpellName", lua.create_function(create_spell_name)?)?;
    t.set("GetSpellCooldown", lua.create_function(create_spell_cooldown)?)?;
    t.set("DoesSpellExist", lua.create_function(|_, spell_id: i32| {
        Ok(spell_id > 0 && crate::spells::get_spell(spell_id as u32).is_some())
    })?)?;
    t.set("RequestLoadSpellData", lua.create_function(|_, _spell_id: i32| Ok(()))?)?;

    Ok(t)
}

fn create_spell_texture(_: &Lua, spell_id: i32) -> Result<&'static str> {
    let file_id = crate::spells::get_spell(spell_id as u32)
        .map(|s| s.icon_file_data_id)
        .unwrap_or(136243);
    Ok(crate::manifest_interface_data::get_texture_path(file_id).unwrap_or(""))
}

fn create_spell_link(lua: &Lua, spell_id: i32) -> Result<Value> {
    let name = crate::spells::get_spell(spell_id as u32)
        .map(|s| s.name)
        .unwrap_or("Unknown");
    let link = format!("|cff71d5ff|Hspell:{}|h[{}]|h|r", spell_id, name);
    Ok(Value::String(lua.create_string(&link)?))
}

fn create_spell_name(lua: &Lua, spell_id: i32) -> Result<Value> {
    let name = crate::spells::get_spell(spell_id as u32)
        .map(|s| s.name)
        .unwrap_or("Unknown");
    Ok(Value::String(lua.create_string(name)?))
}

/// Cast time in milliseconds for spells that have one (WoW API returns ms).
fn spell_cast_time(spell_id: i32) -> i32 {
    match spell_id {
        19750 => 1500,  // Flash of Light
        82326 => 2500,  // Holy Light
        7328 => 10000,  // Redemption (10s res)
        _ => 0,
    }
}

fn create_spell_info(lua: &Lua, spell_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    if let Some(spell) = crate::spells::get_spell(spell_id as u32) {
        info.set("name", spell.name)?;
        info.set("iconID", spell.icon_file_data_id as i64)?;
    } else {
        info.set("name", format!("Spell {}", spell_id))?;
        info.set("iconID", 136243)?;
    }
    info.set("spellID", spell_id)?;
    info.set("castTime", spell_cast_time(spell_id))?;
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
    register_c_traits_config(&t, lua)?;
    register_c_traits_tree(&t, lua)?;
    Ok(t)
}

/// C_Traits config and node lookup stubs.
fn register_c_traits_config(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set("GenerateImportString", lua.create_function(|_, _id: i32| Ok("dummy_talent_string".to_string()))?)?;
    t.set("GetConfigIDBySystemID", lua.create_function(|_, _id: i32| Ok(0))?)?;
    t.set("GetConfigIDByTreeID", lua.create_function(|_, _id: i32| Ok(0))?)?;
    t.set("GetConfigInfo", lua.create_function(create_config_info)?)?;
    t.set("GetNodeInfo", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(Value::Nil))?)?;
    t.set("GetEntryInfo", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(Value::Nil))?)?;
    t.set("GetDefinitionInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("CanPurchaseRank", lua.create_function(|_, (_a, _b, _c): (i32, i32, i32)| Ok(false))?)?;
    t.set("GetLoadoutSerializationVersion", lua.create_function(|_, ()| Ok(2i32))?)?;
    Ok(())
}

/// C_Traits tree query stubs.
fn register_c_traits_tree(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set("InitializeViewLoadout", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(true))?)?;
    t.set("GetTreeInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetTreeNodes", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("GetTreeCurrencyInfo", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(Value::Nil))?)?;
    t.set("GetAllTreeIDs", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetTraitSystemFlags", lua.create_function(|_, _id: i32| Ok(0))?)?;
    Ok(())
}

fn create_config_info(lua: &Lua, _config_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    info.set("treeIDs", lua.create_table()?)?;
    info.set("ID", 0)?;
    info.set("type", 1)?;
    info.set("name", "")?;
    Ok(Value::Table(info))
}
