//! C_Traits namespace - talent/loadout system (Dragonflight+).
//!
//! Backed by static data from `data/traits.rs`.

use mlua::{Lua, Result, Value};

/// Build and return the C_Traits Lua table.
pub fn register_c_traits(lua: &Lua) -> Result<mlua::Table> {
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
    t.set("ConfigHasStagedChanges", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("CommitConfig", lua.create_function(|_, _id: i32| Ok(true))?)?;
    t.set("RollbackConfig", lua.create_function(|_, _id: i32| Ok(true))?)?;
    t.set("GetStagedChanges", lua.create_function(|lua, _id: i32| {
        Ok((lua.create_table()?, lua.create_table()?, lua.create_table()?))
    })?)?;
    t.set("GetStagedChangesCost", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("PurchaseRank", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(false))?)?;
    t.set("RefundRank", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(false))?)?;
    t.set("RefundAllRanks", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(false))?)?;
    t.set("SetSelection", lua.create_function(|_, (_a, _b, _c): (i32, i32, i32)| Ok(false))?)?;
    t.set("CascadeRepurchaseRanks", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(false))?)?;
    t.set("ClearCascadeRepurchaseHistory", lua.create_function(|_, _id: i32| Ok(()))?)?;
    t.set("ResetTree", lua.create_function(|_, _id: i32| Ok(true))?)?;
    t.set("ResetTreeByCurrency", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(true))?)?;
    t.set("GenerateInspectImportString", lua.create_function(|_, _unit: String| Ok("".to_string()))?)?;
    t.set("GetTreeHash", lua.create_function(|_, _id: i32| Ok("0".to_string()))?)?;
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
    t.set("GetNodeCost", lua.create_function(|lua, (_cfg, _node): (i32, i32)| lua.create_table())?)?;
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
    info.set("minZoom", 0.75)?;
    info.set("maxZoom", 1.2)?;
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

fn create_node_info(lua: &Lua, (_config_id, node_id): (Value, Value)) -> Result<Value> {
    use crate::traits::TRAIT_NODE_DB;
    let node_id = match &node_id {
        Value::Integer(n) => *n as i32,
        Value::Number(n) => *n as i32,
        _ => return build_empty_node_info(lua, 0),
    };
    let Some(node) = TRAIT_NODE_DB.get(&(node_id as u32)) else {
        return build_empty_node_info(lua, node_id);
    };
    let info = lua.create_table()?;
    info.set("ID", node_id)?;
    info.set("posX", node.pos_x)?;
    info.set("posY", node.pos_y)?;
    info.set("type", node.node_type as i32)?;
    info.set("flags", node.flags as i32)?;
    // subTreeID must be nil (not 0) when absent — Lua treats 0 as truthy
    if node.sub_tree_id != 0 {
        info.set("subTreeID", node.sub_tree_id as i64)?;
    }

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
    let active_entry = lua.create_table()?;
    active_entry.set("entryID", node.entry_ids.first().copied().unwrap_or(0) as i64)?;
    active_entry.set("rank", max_ranks)?;
    info.set("activeEntry", active_entry)?;
    info.set("isVisible", true)?;
    info.set("isAvailable", true)?;
    info.set("canPurchaseRank", false)?;
    info.set("canRefundRank", false)?;
    info.set("meetsEdgeRequirements", true)?;
    info.set("isCascadeRepurchasable", false)?;
    Ok(Value::Table(info))
}

/// Build a minimal nodeInfo for nodes not in the trait DB (e.g. Delves companion nodes).
/// WoW always returns a struct, so callers don't guard against nil.
fn build_empty_node_info(lua: &Lua, node_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    info.set("ID", node_id)?;
    info.set("posX", 0)?;
    info.set("posY", 0)?;
    info.set("type", 0)?;
    info.set("flags", 0)?;
    // subTreeID omitted (nil) — Lua treats 0 as truthy
    info.set("entryIDs", lua.create_table()?)?;
    info.set("visibleEdges", lua.create_table()?)?;
    info.set("conditionIDs", lua.create_table()?)?;
    info.set("groupIDs", lua.create_table()?)?;
    info.set("currentRank", 0)?;
    info.set("activeRank", 0)?;
    info.set("ranksPurchased", 0)?;
    info.set("maxRanks", 0)?;
    let active_entry = lua.create_table()?;
    active_entry.set("entryID", 0i64)?;
    active_entry.set("rank", 0)?;
    info.set("activeEntry", active_entry)?;
    info.set("isVisible", false)?;
    info.set("isAvailable", false)?;
    info.set("canPurchaseRank", false)?;
    info.set("canRefundRank", false)?;
    info.set("meetsEdgeRequirements", false)?;
    info.set("isCascadeRepurchasable", false)?;
    Ok(Value::Table(info))
}

fn build_node_entry_ids(lua: &Lua, info: &mlua::Table, node: &crate::traits::TraitNodeInfo) -> Result<()> {
    let entry_ids = lua.create_table()?;
    for (i, &eid) in node.entry_ids.iter().enumerate() {
        entry_ids.set(i as i64 + 1, eid as i64)?;
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
    if entry.sub_tree_id != 0 {
        info.set("subTreeID", entry.sub_tree_id as i64)?;
    }
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
    info.set("spellID", if def.spell_id != 0 { Value::Integer(def.spell_id as i64) } else { Value::Nil })?;
    info.set("overriddenSpellID", if def.overrides_spell_id != 0 { Value::Integer(def.overrides_spell_id as i64) } else { Value::Nil })?;
    info.set("overrideIcon", if def.override_icon != 0 { Value::Integer(def.override_icon as i64) } else { Value::Nil })?;
    info.set("visibleSpellID", if def.visible_spell_id != 0 { Value::Integer(def.visible_spell_id as i64) } else { Value::Nil })?;
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
    info.set("posX", 0)?;
    info.set("posY", 0)?;
    Ok(Value::Table(info))
}
