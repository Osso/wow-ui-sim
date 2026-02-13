//! Hero talent spec resolution.
//!
//! Computes which hero subtrees are available per spec by scanning
//! SubTreeSelection nodes (type 3) in the trait tree.
//!
//! Data flow:
//!   TraitNode (type=3, with spec condition) → entries → TraitNodeEntry.sub_tree_id
//!
//! Used by:
//! - `C_ClassTalents.GetHeroTalentSpecsForClassSpec` → returns subtree IDs + unlock level
//! - `C_Traits.GetSubTreeInfo` → returns `subTreeSelectionNodeIDs` per subtree

use crate::lua_api::SimState;
use crate::traits::{TRAIT_COND_DB, TRAIT_ENTRY_DB, TRAIT_NODE_DB, TRAIT_TREE_DB};
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

/// Hero spec unlock level (WoW requires level 71 to pick a hero spec).
pub const HERO_SPEC_UNLOCK_LEVEL: i32 = 71;

/// For each tree, the available hero subtree IDs per spec set.
/// Key: (tree_id, spec_set_id), Value: sorted Vec of subtree IDs.
static SPEC_TO_SUBTREES: LazyLock<HashMap<(u32, u32), Vec<u32>>> = LazyLock::new(compute_spec_to_subtrees);

/// For each subtree, the SubTreeSelection node IDs that reference it.
/// Key: subtree_id, Value: Vec of selection node IDs.
static SUBTREE_TO_SELECTION_NODES: LazyLock<HashMap<u32, Vec<u32>>> =
    LazyLock::new(compute_subtree_to_selection_nodes);

/// Get hero subtree IDs available for the given tree and spec set.
pub fn subtree_ids_for_spec(tree_id: u32, spec_set_id: u32) -> Option<&'static Vec<u32>> {
    SPEC_TO_SUBTREES.get(&(tree_id, spec_set_id))
}

/// Get SubTreeSelection node IDs that reference the given subtree.
pub fn selection_node_ids_for_subtree(subtree_id: u32) -> &'static [u32] {
    SUBTREE_TO_SELECTION_NODES
        .get(&subtree_id)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

/// Build map: (tree_id, spec_set_id) → [subtree_ids].
///
/// For each SubTreeSelection node (type 3), find its spec condition,
/// then collect unique subtree IDs from its entries.
fn compute_spec_to_subtrees() -> HashMap<(u32, u32), Vec<u32>> {
    let mut map: HashMap<(u32, u32), Vec<u32>> = HashMap::new();

    for tree in TRAIT_TREE_DB.values() {
        for &node_id in tree.node_ids {
            let Some(node) = TRAIT_NODE_DB.get(&node_id) else {
                continue;
            };
            if node.node_type != 3 {
                continue;
            }
            let spec_set_id = find_spec_set_condition(node.cond_ids);
            let subtree_ids = collect_entry_subtree_ids(node.entry_ids);
            let key = (tree.id, spec_set_id);
            let entry = map.entry(key).or_default();
            for st_id in subtree_ids {
                if !entry.contains(&st_id) {
                    entry.push(st_id);
                }
            }
        }
    }

    // Sort for deterministic output
    for v in map.values_mut() {
        v.sort();
    }
    map
}

/// Build map: subtree_id → [selection_node_ids].
fn compute_subtree_to_selection_nodes() -> HashMap<u32, Vec<u32>> {
    let mut map: HashMap<u32, Vec<u32>> = HashMap::new();

    for node in TRAIT_NODE_DB.values() {
        if node.node_type != 3 {
            continue;
        }
        for &entry_id in node.entry_ids {
            let Some(entry) = TRAIT_ENTRY_DB.get(&entry_id) else {
                continue;
            };
            if entry.sub_tree_id != 0 {
                let nodes = map.entry(entry.sub_tree_id).or_default();
                if !nodes.contains(&node.id) {
                    nodes.push(node.id);
                }
            }
        }
    }

    for v in map.values_mut() {
        v.sort();
    }
    map
}

/// Find the spec_set_id from a node's conditions (cond_type == 1).
/// Returns 0 if no spec condition found (visible to all specs).
fn find_spec_set_condition(cond_ids: &[u32]) -> u32 {
    for &cid in cond_ids {
        if let Some(cond) = TRAIT_COND_DB.get(&cid) {
            if cond.cond_type == 1 {
                return cond.spec_set_id;
            }
        }
    }
    0
}

/// Map specID → specSetID for Paladin.
///
/// Matches the mapping in `traits_api_node.rs`:
///   SpecSet 27 → Spec 65 (Holy)
///   SpecSet 28 → Spec 66 (Protection)
///   SpecSet 29 → Spec 70 (Retribution)
pub fn spec_id_to_spec_set(spec_id: u32) -> u32 {
    match spec_id {
        65 => 27,
        66 => 28,
        70 => 29,
        _ => 0,
    }
}

/// Collect unique subtree IDs from a node's entries.
fn collect_entry_subtree_ids(entry_ids: &[u32]) -> Vec<u32> {
    let mut ids = Vec::new();
    for &eid in entry_ids {
        if let Some(entry) = TRAIT_ENTRY_DB.get(&eid) {
            if entry.sub_tree_id != 0 && !ids.contains(&entry.sub_tree_id) {
                ids.push(entry.sub_tree_id);
            }
        }
    }
    ids
}

/// Get the active hero subtree ID from talent state, or None.
///
/// Checks SubTreeSelection nodes for the active spec (Protection, spec_set=28)
/// in tree 790. If one has a selection, returns the selected entry's subtree ID.
pub fn get_active_hero_subtree(state: &SimState) -> Value {
    let spec_set = 28u32; // Protection
    let tree_id = 790u32;
    let Some(subtree_ids) = subtree_ids_for_spec(tree_id, spec_set) else {
        return Value::Nil;
    };
    for &st_id in subtree_ids {
        for &node_id in selection_node_ids_for_subtree(st_id) {
            if let Some(&entry_id) = state.talents.node_selections.get(&node_id) {
                if let Some(entry) = TRAIT_ENTRY_DB.get(&entry_id) {
                    if entry.sub_tree_id != 0 {
                        return Value::Integer(entry.sub_tree_id as i64);
                    }
                }
            }
        }
    }
    Value::Nil
}

/// C_ClassTalents namespace — class talent configuration and hero spec APIs.
pub fn register_c_class_talents(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let t = lua.create_table()?;
    register_config_stubs(&t, lua)?;
    register_hero_spec_apis(&t, lua, &state)?;
    let st = Rc::clone(&state);
    t.set("HasUnspentTalentPoints", lua.create_function(move |_, ()| {
        Ok(super::traits_api::has_unspent_talent_points(&st.borrow()))
    })?)?;
    t.set("HasUnspentHeroTalentPoints", lua.create_function(|_, ()| Ok(false))?)?;
    let st = state;
    t.set("GetActiveHeroTalentSpec", lua.create_function(move |_, ()| {
        Ok(get_active_hero_subtree(&st.borrow()))
    })?)?;
    lua.globals().set("C_ClassTalents", t)?;
    Ok(())
}

fn register_config_stubs(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set("GetActiveConfigID", lua.create_function(|_, ()| Ok(1i32))?)?;
    t.set("GetConfigIDsBySpecID", lua.create_function(|lua, _spec_id: Option<i32>| {
        let t = lua.create_table()?;
        t.set(1, 1i32)?;
        Ok(t)
    })?)?;
    t.set("CanEditTalents", lua.create_function(|_, ()| Ok((true, Value::Nil)))?)?;
    t.set("GetStarterBuildActive", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetHasStarterBuild", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetLastSelectedSavedConfigID", lua.create_function(|_, _spec_id: Option<i32>| Ok(Value::Nil))?)?;
    t.set("CanChangeTalents", lua.create_function(|_, ()| Ok((true, false)))?)?;
    Ok(())
}

fn register_hero_spec_apis(t: &mlua::Table, lua: &Lua, _state: &Rc<RefCell<SimState>>) -> Result<()> {
    t.set("GetHeroTalentSpecsForClassSpec", lua.create_function(|lua, (_cfg, spec_id): (Option<i32>, Option<i32>)| {
        let spec = spec_id.unwrap_or(66) as u32;
        let spec_set = spec_id_to_spec_set(spec);
        match subtree_ids_for_spec(790, spec_set) {
            Some(ids) if !ids.is_empty() => {
                let t = lua.create_table()?;
                for (i, &id) in ids.iter().enumerate() {
                    t.set(i as i64 + 1, id as i64)?;
                }
                Ok((Value::Table(t), Value::Integer(HERO_SPEC_UNLOCK_LEVEL as i64)))
            }
            _ => Ok((Value::Nil, Value::Integer(HERO_SPEC_UNLOCK_LEVEL as i64))),
        }
    })?)?;
    Ok(())
}
