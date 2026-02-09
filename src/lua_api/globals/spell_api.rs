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
    t.set("IsAutoAttackSpell", lua.create_function(|_, _spell_id: i32| Ok(false))?)?;
    t.set("IsRangedAutoAttackSpell", lua.create_function(|_, _spell_id: i32| Ok(false))?)?;

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
/// Backed by static data from `data/traits.rs`.
fn register_c_traits(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    register_c_traits_config(&t, lua)?;
    register_c_traits_tree(&t, lua)?;
    register_c_traits_node(&t, lua)?;
    Ok(t)
}

/// C_Traits config-level APIs.
fn register_c_traits_config(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set("GenerateImportString", lua.create_function(|_, _id: i32| Ok("dummy_talent_string".to_string()))?)?;
    t.set("GetConfigIDBySystemID", lua.create_function(|_, _id: i32| Ok(1i32))?)?;
    t.set("GetConfigIDByTreeID", lua.create_function(|_, _id: i32| Ok(1i32))?)?;
    t.set("GetConfigInfo", lua.create_function(create_config_info)?)?;
    t.set("CanPurchaseRank", lua.create_function(|_, (_a, _b, _c): (i32, i32, i32)| Ok(false))?)?;
    t.set("GetLoadoutSerializationVersion", lua.create_function(|_, ()| Ok(2i32))?)?;
    Ok(())
}

/// C_Traits tree-level APIs.
fn register_c_traits_tree(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set("InitializeViewLoadout", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(true))?)?;
    t.set("GetTreeInfo", lua.create_function(create_tree_info)?)?;
    t.set("GetTreeNodes", lua.create_function(create_tree_nodes)?)?;
    t.set("GetTreeCurrencyInfo", lua.create_function(create_tree_currency_info)?)?;
    t.set("GetAllTreeIDs", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetTraitSystemFlags", lua.create_function(|_, _id: i32| Ok(0))?)?;
    Ok(())
}

/// C_Traits node/entry/definition-level APIs.
fn register_c_traits_node(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set("GetNodeInfo", lua.create_function(create_node_info)?)?;
    t.set("GetEntryInfo", lua.create_function(create_entry_info)?)?;
    t.set("GetDefinitionInfo", lua.create_function(create_definition_info)?)?;
    t.set("GetConditionInfo", lua.create_function(create_condition_info)?)?;
    t.set("GetSubTreeInfo", lua.create_function(create_sub_tree_info)?)?;
    Ok(())
}

fn create_config_info(lua: &Lua, _config_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    // Return tree 790 (Paladin) as the configured tree
    let tree_ids = lua.create_table()?;
    tree_ids.set(1, 790)?;
    info.set("treeIDs", tree_ids)?;
    info.set("ID", 1)?;
    info.set("type", 1)?;
    info.set("name", "")?;
    Ok(Value::Table(info))
}

fn create_tree_info(lua: &Lua, (config_id, tree_id): (i32, i32)) -> Result<Value> {
    use crate::traits::TRAIT_TREE_DB;
    if TRAIT_TREE_DB.get(&(tree_id as u32)).is_none() {
        return Ok(Value::Nil);
    }
    let info = lua.create_table()?;
    info.set("ID", tree_id)?;
    info.set("gates", lua.create_table()?)?;
    info.set("hideSinglePurchaseNodes", false)?;
    info.set("configID", config_id)?;
    info.set("minZoom", 1.0)?;
    info.set("maxZoom", 1.0)?;
    info.set("buttonSize", 40)?;
    info.set("isLinkedToActiveConfigID", true)?;
    Ok(Value::Table(info))
}

fn create_tree_nodes(lua: &Lua, tree_id: i32) -> Result<mlua::Table> {
    use crate::traits::TRAIT_TREE_DB;
    let t = lua.create_table()?;
    if let Some(tree) = TRAIT_TREE_DB.get(&(tree_id as u32)) {
        for (i, &node_id) in tree.node_ids.iter().enumerate() {
            t.set(i as i64 + 1, node_id as i64)?;
        }
    }
    Ok(t)
}

fn create_tree_currency_info(lua: &Lua, (_config_id, tree_id): (i32, i32)) -> Result<Value> {
    use crate::traits::{TRAIT_TREE_DB, TRAIT_CURRENCY_DB};
    let Some(tree) = TRAIT_TREE_DB.get(&(tree_id as u32)) else {
        return Ok(Value::Nil);
    };
    let arr = lua.create_table()?;
    for (i, &cid) in tree.currency_ids.iter().enumerate() {
        let entry = lua.create_table()?;
        entry.set("traitCurrencyID", cid as i64)?;
        // Simulate having max currency spent
        let quantity = if let Some(c) = TRAIT_CURRENCY_DB.get(&cid) {
            if c.currency_type == 1 { 50 } else { 0 }
        } else { 0 };
        entry.set("quantity", quantity)?;
        entry.set("maxQuantity", quantity)?;
        entry.set("spent", 0)?;
        entry.set("flags", 0)?;
        arr.set(i as i64 + 1, entry)?;
    }
    Ok(Value::Table(arr))
}

fn create_node_info(lua: &Lua, (_config_id, node_id): (i32, i32)) -> Result<Value> {
    use crate::traits::TRAIT_NODE_DB;
    let Some(node) = TRAIT_NODE_DB.get(&(node_id as u32)) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("ID", node_id)?;
    info.set("posX", node.pos_x)?;
    info.set("posY", node.pos_y)?;
    info.set("type", node.node_type as i32)?;
    info.set("flags", node.flags as i32)?;
    info.set("subTreeID", node.sub_tree_id as i64)?;

    build_node_entry_ids(lua, &info, node)?;
    build_node_edges(lua, &info, node)?;
    build_node_cond_ids(lua, &info, node)?;
    build_node_group_ids(lua, &info, node)?;

    // State: fully talented
    let max_ranks = node_max_ranks(node);
    info.set("currentRank", max_ranks)?;
    info.set("activeRank", max_ranks)?;
    info.set("ranksPurchased", max_ranks)?;
    info.set("maxRanks", max_ranks)?;
    info.set("activeEntry", node.entry_ids.first().copied().unwrap_or(0) as i64)?;
    info.set("isAvailable", true)?;
    info.set("canPurchaseRank", false)?;
    info.set("canRefundRank", false)?;
    info.set("meetsEdgeRequirements", true)?;
    info.set("isCascadeRepurchasable", false)?;
    Ok(Value::Table(info))
}

fn build_node_entry_ids(lua: &Lua, info: &mlua::Table, node: &crate::traits::TraitNodeInfo) -> Result<()> {
    let entry_ids = lua.create_table()?;
    for (i, &eid) in node.entry_ids.iter().enumerate() {
        let e = lua.create_table()?;
        e.set("entryID", eid as i64)?;
        e.set("rank", 0)?;
        entry_ids.set(i as i64 + 1, e)?;
    }
    info.set("entryIDs", entry_ids)?;
    Ok(())
}

fn build_node_edges(lua: &Lua, info: &mlua::Table, node: &crate::traits::TraitNodeInfo) -> Result<()> {
    let edges = lua.create_table()?;
    for (i, edge) in node.edges.iter().enumerate() {
        let e = lua.create_table()?;
        e.set("targetNode", edge.source_node_id as i64)?;
        e.set("type", edge.edge_type as i32)?;
        e.set("visualStyle", edge.visual_style as i32)?;
        e.set("isActive", true)?;
        edges.set(i as i64 + 1, e)?;
    }
    info.set("visibleEdges", edges)?;
    Ok(())
}

fn build_node_cond_ids(lua: &Lua, info: &mlua::Table, node: &crate::traits::TraitNodeInfo) -> Result<()> {
    let cond_ids = lua.create_table()?;
    for (i, &cid) in node.cond_ids.iter().enumerate() {
        cond_ids.set(i as i64 + 1, cid as i64)?;
    }
    info.set("conditionIDs", cond_ids)?;
    Ok(())
}

fn build_node_group_ids(lua: &Lua, info: &mlua::Table, node: &crate::traits::TraitNodeInfo) -> Result<()> {
    let group_ids = lua.create_table()?;
    for (i, &gid) in node.group_ids.iter().enumerate() {
        group_ids.set(i as i64 + 1, gid as i64)?;
    }
    info.set("groupIDs", group_ids)?;
    Ok(())
}

/// Get max ranks for a node from its first entry.
fn node_max_ranks(node: &crate::traits::TraitNodeInfo) -> i32 {
    use crate::traits::TRAIT_ENTRY_DB;
    node.entry_ids.first()
        .and_then(|eid| TRAIT_ENTRY_DB.get(eid))
        .map(|e| e.max_ranks as i32)
        .unwrap_or(1)
}

fn create_entry_info(lua: &Lua, (_config_id, entry_id): (i32, i32)) -> Result<Value> {
    use crate::traits::TRAIT_ENTRY_DB;
    let Some(entry) = TRAIT_ENTRY_DB.get(&(entry_id as u32)) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("entryID", entry_id)?;
    info.set("definitionID", entry.definition_id as i64)?;
    info.set("type", entry.entry_type as i32)?;
    info.set("maxRanks", entry.max_ranks as i32)?;
    info.set("subTreeID", entry.sub_tree_id as i64)?;
    info.set("isAvailable", true)?;
    info.set("conditionIDs", lua.create_table()?)?;
    Ok(Value::Table(info))
}

fn create_definition_info(lua: &Lua, def_id: i32) -> Result<Value> {
    use crate::traits::TRAIT_DEFINITION_DB;
    let Some(def) = TRAIT_DEFINITION_DB.get(&(def_id as u32)) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("spellID", def.spell_id as i64)?;
    info.set("overriddenSpellID", def.overrides_spell_id as i64)?;
    info.set("overrideIcon", def.override_icon as i64)?;
    info.set("visibleSpellID", def.visible_spell_id as i64)?;
    info.set("overrideName", def.override_name)?;
    info.set("overrideSubtext", def.override_subtext)?;
    info.set("overrideDescription", def.override_description)?;
    Ok(Value::Table(info))
}

fn create_condition_info(lua: &Lua, (_config_id, cond_id): (i32, i32)) -> Result<Value> {
    use crate::traits::TRAIT_COND_DB;
    let Some(cond) = TRAIT_COND_DB.get(&(cond_id as u32)) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("condID", cond_id)?;
    info.set("condType", cond.cond_type as i32)?;
    info.set("traitCurrencyID", cond.currency_id as i64)?;
    info.set("spentAmountRequired", cond.spent_amount as i32)?;
    info.set("specSetID", cond.spec_set_id as i32)?;
    info.set("questID", cond.quest_id as i64)?;
    info.set("achievementID", cond.achievement_id as i64)?;
    info.set("requiredLevel", cond.required_level as i32)?;
    info.set("traitNodeGroupID", cond.group_id as i64)?;
    info.set("traitNodeID", cond.node_id as i64)?;
    info.set("grantedRanks", cond.granted_ranks as i32)?;
    info.set("isMet", true)?;
    info.set("isSufficient", true)?;
    Ok(Value::Table(info))
}

fn create_sub_tree_info(lua: &Lua, (_config_id, sub_tree_id): (i32, i32)) -> Result<Value> {
    use crate::traits::TRAIT_SUBTREE_DB;
    let Some(st) = TRAIT_SUBTREE_DB.get(&(sub_tree_id as u32)) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("ID", sub_tree_id)?;
    info.set("name", st.name)?;
    info.set("description", st.description)?;
    info.set("traitTreeID", st.tree_id as i64)?;
    info.set("iconElementID", st.atlas_element_id as i64)?;
    info.set("isActive", true)?;
    Ok(Value::Table(info))
}
