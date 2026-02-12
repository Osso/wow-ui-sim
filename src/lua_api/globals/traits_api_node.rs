//! Node/entry/definition/condition info builders for C_Traits.
//!
//! Split from traits_api.rs — these are the read-side data accessors.

use crate::lua_api::SimState;
use crate::traits::{TraitNodeInfo, TRAIT_COND_DB, TRAIT_NODE_DB};
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_node_info(
    lua: &Lua, state: &Rc<RefCell<SimState>>, _config_id: Value, node_id: Value,
) -> Result<Value> {
    let node_id = match &node_id {
        Value::Integer(n) => *n as i32,
        Value::Number(n) => *n as i32,
        _ => return build_empty_node_info(lua, 0),
    };
    let Some(node) = TRAIT_NODE_DB.get(&(node_id as u32)) else {
        return build_empty_node_info(lua, node_id);
    };
    let info = lua.create_table()?;
    set_node_static_fields(lua, &info, node, node_id)?;
    set_node_dynamic_fields(lua, &info, node, node_id as u32, state)?;
    Ok(Value::Table(info))
}

/// Static node fields that don't depend on talent state.
fn set_node_static_fields(
    lua: &Lua, info: &mlua::Table, node: &TraitNodeInfo, node_id: i32,
) -> Result<()> {
    info.set("ID", node_id)?;
    info.set("posX", node.pos_x)?;
    info.set("posY", node.pos_y)?;
    info.set("type", node.node_type as i32)?;
    info.set("flags", node.flags as i32)?;
    if node.sub_tree_id != 0 {
        info.set("subTreeID", node.sub_tree_id as i64)?;
    }
    build_node_entry_ids(lua, info, node)?;
    build_node_cond_ids(lua, info, node)?;
    build_node_group_ids(lua, info, node)?;
    info.set("isCascadeRepurchasable", false)?;
    Ok(())
}

/// Dynamic node fields that depend on talent purchase state.
fn set_node_dynamic_fields(
    lua: &Lua, info: &mlua::Table, node: &TraitNodeInfo,
    node_id: u32, state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let max_ranks = node_max_ranks(node);
    let s = state.borrow();

    // SubTreeSelection nodes (type 3) are not instantiated by the Blizzard UI.
    if node.node_type == 3 {
        info.set("isVisible", false)?;
        return set_empty_ranks(info, lua, max_ranks);
    }

    // Hero subtree nodes stay fully talented.
    if node.sub_tree_id != 0 {
        return set_fully_talented(lua, info, node, max_ranks);
    }

    let ranks_purchased = *s.talents.node_ranks.get(&node_id).unwrap_or(&0) as i32;
    let is_available = check_node_available(node, &s);
    let meets_edges = check_edge_requirements(node, &s);
    let has_currency = check_has_currency(node_id, &s);
    let can_purchase = ranks_purchased < max_ranks && is_available && meets_edges && has_currency;

    info.set("currentRank", ranks_purchased)?;
    info.set("activeRank", ranks_purchased)?;
    info.set("ranksPurchased", ranks_purchased)?;
    info.set("maxRanks", max_ranks)?;
    info.set("isVisible", true)?;
    info.set("isAvailable", is_available)?;
    info.set("canPurchaseRank", can_purchase)?;
    info.set("canRefundRank", ranks_purchased > 0)?;
    info.set("meetsEdgeRequirements", meets_edges)?;
    build_node_edges_dynamic(lua, info, node, &s)?;
    build_active_entry(lua, info, node, node_id, ranks_purchased, &s)?;
    Ok(())
}

fn set_fully_talented(
    lua: &Lua, info: &mlua::Table, node: &TraitNodeInfo, max_ranks: i32,
) -> Result<()> {
    info.set("currentRank", max_ranks)?;
    info.set("activeRank", max_ranks)?;
    info.set("ranksPurchased", max_ranks)?;
    info.set("maxRanks", max_ranks)?;
    info.set("isVisible", true)?;
    info.set("isAvailable", true)?;
    info.set("canPurchaseRank", false)?;
    info.set("canRefundRank", false)?;
    info.set("meetsEdgeRequirements", true)?;
    // Edges all active for hero nodes, filtered to same subtree.
    let edges = lua.create_table()?;
    let mut idx = 0i64;
    for edge in node.edges.iter() {
        if !should_show_edge(node.sub_tree_id, edge.source_node_id) {
            continue;
        }
        idx += 1;
        let e = lua.create_table()?;
        e.set("targetNode", edge.source_node_id as i64)?;
        e.set("type", edge.edge_type as i32)?;
        e.set("visualStyle", edge.visual_style as i32)?;
        e.set("isActive", true)?;
        edges.set(idx, e)?;
    }
    info.set("visibleEdges", edges)?;
    let active_entry = lua.create_table()?;
    active_entry.set("entryID", node.entry_ids.first().copied().unwrap_or(0) as i64)?;
    active_entry.set("rank", max_ranks)?;
    info.set("activeEntry", active_entry)?;
    Ok(())
}

/// Minimal rank fields for non-instantiated nodes (SubTreeSelection).
fn set_empty_ranks(info: &mlua::Table, lua: &Lua, max_ranks: i32) -> Result<()> {
    info.set("currentRank", 0)?;
    info.set("activeRank", 0)?;
    info.set("ranksPurchased", 0)?;
    info.set("maxRanks", max_ranks)?;
    info.set("isAvailable", false)?;
    info.set("canPurchaseRank", false)?;
    info.set("canRefundRank", false)?;
    info.set("meetsEdgeRequirements", false)?;
    info.set("visibleEdges", lua.create_table()?)?;
    let ae = lua.create_table()?;
    ae.set("entryID", 0i64)?;
    ae.set("rank", 0)?;
    info.set("activeEntry", ae)?;
    Ok(())
}

fn build_active_entry(
    lua: &Lua, info: &mlua::Table, node: &TraitNodeInfo,
    node_id: u32, ranks_purchased: i32, state: &SimState,
) -> Result<()> {
    let active_entry = lua.create_table()?;
    let entry_id = if node.entry_ids.len() > 1 {
        // Choice node: use selected entry or first.
        state.talents.node_selections.get(&node_id)
            .copied()
            .unwrap_or_else(|| node.entry_ids.first().copied().unwrap_or(0))
    } else {
        node.entry_ids.first().copied().unwrap_or(0)
    };
    active_entry.set("entryID", entry_id as i64)?;
    active_entry.set("rank", ranks_purchased)?;
    info.set("activeEntry", active_entry)?;
    Ok(())
}

/// Check all gate conditions (cond_type==0) are met for this node.
fn check_node_available(node: &TraitNodeInfo, state: &SimState) -> bool {
    for &cid in node.cond_ids {
        let Some(cond) = TRAIT_COND_DB.get(&cid) else { continue };
        if cond.cond_type != 0 { continue }
        if cond.currency_id == 0 { continue }
        let spent = state.talents.spent_for_currency(cond.currency_id);
        if spent < cond.spent_amount {
            return false;
        }
    }
    true
}

/// Check all required edges (type > 0) have their source node purchased.
fn check_edge_requirements(node: &TraitNodeInfo, state: &SimState) -> bool {
    for edge in node.edges {
        if edge.edge_type == 0 { continue }
        let source_rank = *state.talents.node_ranks.get(&edge.source_node_id).unwrap_or(&0);
        if source_rank == 0 {
            return false;
        }
    }
    true
}

/// Check the node's currency has remaining points.
fn check_has_currency(node_id: u32, state: &SimState) -> bool {
    let Some(&cid) = state.talents.node_currency_map.get(&node_id) else {
        return true; // No currency mapped → free (hero nodes, etc.)
    };
    let max_pts = super::traits_api::max_points_for_currency(cid);
    if max_pts == 0 { return true }
    state.talents.spent_for_currency(cid) < max_pts
}

/// Build edges with dynamic isActive based on source node purchase state.
/// Filters out cross-subtree edges: non-hero nodes only show edges to other
/// non-hero nodes, hero nodes only show edges within the same subtree.
fn build_node_edges_dynamic(
    lua: &Lua, info: &mlua::Table, node: &TraitNodeInfo, state: &SimState,
) -> Result<()> {
    let edges = lua.create_table()?;
    let mut idx = 0i64;
    for edge in node.edges.iter() {
        if !should_show_edge(node.sub_tree_id, edge.source_node_id) {
            continue;
        }
        idx += 1;
        let e = lua.create_table()?;
        e.set("targetNode", edge.source_node_id as i64)?;
        e.set("type", edge.edge_type as i32)?;
        e.set("visualStyle", edge.visual_style as i32)?;
        let is_active = if edge.edge_type == 0 {
            true
        } else {
            *state.talents.node_ranks.get(&edge.source_node_id).unwrap_or(&0) > 0
        };
        e.set("isActive", is_active)?;
        edges.set(idx, e)?;
    }
    info.set("visibleEdges", edges)?;
    Ok(())
}

/// Filter cross-subtree edges: only show edges between nodes in the same
/// subtree (both hero or both non-hero).
fn should_show_edge(this_sub_tree: u32, target_node_id: u32) -> bool {
    let target_sub_tree = TRAIT_NODE_DB.get(&target_node_id)
        .map(|n| n.sub_tree_id)
        .unwrap_or(0);
    match (this_sub_tree, target_sub_tree) {
        (0, 0) => true,                          // both non-hero
        (a, b) if a != 0 && a == b => true,       // same hero subtree
        _ => false,                                // cross-subtree
    }
}

/// Build a minimal nodeInfo for nodes not in the trait DB.
pub fn build_empty_node_info(lua: &Lua, node_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    info.set("ID", node_id)?;
    info.set("posX", 0)?;
    info.set("posY", 0)?;
    info.set("type", 0)?;
    info.set("flags", 0)?;
    info.set("entryIDs", lua.create_table()?)?;
    info.set("visibleEdges", lua.create_table()?)?;
    info.set("conditionIDs", lua.create_table()?)?;
    info.set("groupIDs", lua.create_table()?)?;
    set_empty_node_state(lua, &info)?;
    Ok(Value::Table(info))
}

fn set_empty_node_state(lua: &Lua, info: &mlua::Table) -> Result<()> {
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
    Ok(())
}

fn build_node_entry_ids(lua: &Lua, info: &mlua::Table, node: &TraitNodeInfo) -> Result<()> {
    let entry_ids = lua.create_table()?;
    for (i, &eid) in node.entry_ids.iter().enumerate() {
        entry_ids.set(i as i64 + 1, eid as i64)?;
    }
    info.set("entryIDs", entry_ids)?;
    Ok(())
}

fn build_node_cond_ids(lua: &Lua, info: &mlua::Table, node: &TraitNodeInfo) -> Result<()> {
    let cond_ids = lua.create_table()?;
    for (i, &cid) in node.cond_ids.iter().enumerate() {
        cond_ids.set(i as i64 + 1, cid as i64)?;
    }
    info.set("conditionIDs", cond_ids)?;
    Ok(())
}

fn build_node_group_ids(lua: &Lua, info: &mlua::Table, node: &TraitNodeInfo) -> Result<()> {
    let group_ids = lua.create_table()?;
    for (i, &gid) in node.group_ids.iter().enumerate() {
        group_ids.set(i as i64 + 1, gid as i64)?;
    }
    info.set("groupIDs", group_ids)?;
    Ok(())
}

/// Get max ranks for a node from its first entry.
pub fn node_max_ranks(node: &TraitNodeInfo) -> i32 {
    use crate::traits::TRAIT_ENTRY_DB;
    node.entry_ids.first()
        .and_then(|eid| TRAIT_ENTRY_DB.get(eid))
        .map(|e| e.max_ranks as i32)
        .unwrap_or(1)
}

pub fn create_entry_info(lua: &Lua, (_config_id, entry_id): (i32, i32)) -> Result<Value> {
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

pub fn create_definition_info(lua: &Lua, def_id: i32) -> Result<Value> {
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

/// Dynamic condition info — isMet depends on talent state.
pub fn create_condition_info(
    lua: &Lua, state: &Rc<RefCell<SimState>>, cond_id: i32,
) -> Result<Value> {
    let Some(cond) = TRAIT_COND_DB.get(&(cond_id as u32)) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    set_condition_static_fields(&info, cond, cond_id)?;
    let s = state.borrow();
    let is_met = evaluate_condition(cond, &s);
    info.set("isMet", is_met)?;
    info.set("isSufficient", is_met)?;
    Ok(Value::Table(info))
}

fn set_condition_static_fields(
    info: &mlua::Table, cond: &crate::traits::TraitCondInfo, cond_id: i32,
) -> Result<()> {
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
    Ok(())
}

/// Evaluate whether a trait condition is met based on current talent state.
fn evaluate_condition(cond: &crate::traits::TraitCondInfo, state: &SimState) -> bool {
    match cond.cond_type {
        0 => { // Gate: check spent amount for currency
            if cond.currency_id == 0 { return true }
            state.talents.spent_for_currency(cond.currency_id) >= cond.spent_amount
        }
        1 => true,  // Spec set: always correct spec
        2 => cond.required_level <= 80, // Level check: simulated level 80
        _ => true,  // Granted ranks, misc: always met
    }
}

pub fn create_sub_tree_info(lua: &Lua, (_config_id, sub_tree_id): (i32, i32)) -> Result<Value> {
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
