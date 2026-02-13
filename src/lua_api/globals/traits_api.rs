//! C_Traits namespace - talent/loadout system (Dragonflight+).
//!
//! Backed by static data from `data/traits.rs` and runtime state in `TalentState`.

use crate::lua_api::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Build and return the C_Traits Lua table.
pub fn register_c_traits(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    register_c_traits_config(&t, lua, Rc::clone(&state))?;
    register_c_traits_tree(&t, lua, Rc::clone(&state))?;
    register_c_traits_node(&t, lua, state)?;
    Ok(t)
}

/// C_Traits config-level APIs.
fn register_c_traits_config(
    t: &mlua::Table, lua: &Lua, state: Rc<RefCell<SimState>>,
) -> Result<()> {
    register_config_stubs(t, lua)?;
    register_config_mutations(t, lua, state)?;
    Ok(())
}

/// Stateless config stubs.
fn register_config_stubs(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set("GenerateImportString", lua.create_function(|_, _id: i32| Ok("dummy_talent_string".to_string()))?)?;
    t.set("GetConfigIDBySystemID", lua.create_function(|_, _id: i32| Ok(1i32))?)?;
    t.set("GetConfigIDByTreeID", lua.create_function(|_, _id: i32| Ok(1i32))?)?;
    t.set("GetConfigInfo", lua.create_function(create_config_info)?)?;
    t.set("CanPurchaseRank", lua.create_function(|_, (_a, _b, _c): (i32, i32, i32)| Ok(false))?)?;
    t.set("GetLoadoutSerializationVersion", lua.create_function(|_, ()| Ok(2i32))?)?;
    t.set("CommitConfig", lua.create_function(|_, _id: i32| Ok(true))?)?;
    t.set("RollbackConfig", lua.create_function(|_, _id: i32| Ok(true))?)?;
    t.set("GetStagedChanges", lua.create_function(|lua, _id: i32| {
        Ok((lua.create_table()?, lua.create_table()?, lua.create_table()?))
    })?)?;
    t.set("GetStagedChangesCost", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("RefundAllRanks", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(false))?)?;
    t.set("CascadeRepurchaseRanks", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(false))?)?;
    t.set("ClearCascadeRepurchaseHistory", lua.create_function(|_, _id: i32| Ok(()))?)?;
    t.set("GenerateInspectImportString", lua.create_function(|_, _unit: String| Ok("".to_string()))?)?;
    t.set("GetTreeHash", lua.create_function(|_, _id: i32| Ok("0".to_string()))?)?;
    Ok(())
}

/// State-aware config mutations: PurchaseRank, RefundRank, SetSelection, Reset, etc.
fn register_config_mutations(
    t: &mlua::Table, lua: &Lua, state: Rc<RefCell<SimState>>,
) -> Result<()> {
    let st = Rc::clone(&state);
    t.set("PurchaseRank", lua.create_function(move |lua, (config_id, node_id): (i32, i32)| {
        purchase_rank(&st, lua, config_id, node_id as u32)
    })?)?;

    let st = Rc::clone(&state);
    t.set("RefundRank", lua.create_function(move |lua, (config_id, node_id): (i32, i32)| {
        refund_rank(&st, lua, config_id, node_id as u32)
    })?)?;

    let st = Rc::clone(&state);
    t.set("SetSelection", lua.create_function(move |lua, (config_id, node_id, entry_id): (i32, i32, Option<i32>)| {
        set_selection(&st, lua, config_id, node_id as u32, entry_id.map(|id| id as u32))
    })?)?;

    let st = Rc::clone(&state);
    t.set("ConfigHasStagedChanges", lua.create_function(move |_, _id: i32| {
        Ok(st.borrow().talents.node_ranks.values().any(|&r| r > 0))
    })?)?;

    let st = Rc::clone(&state);
    t.set("ResetTree", lua.create_function(move |lua, config_id: i32| {
        reset_tree(&st, lua, config_id)
    })?)?;

    let st = Rc::clone(&state);
    t.set("ResetTreeByCurrency", lua.create_function(move |lua, (config_id, currency_id): (i32, i32)| {
        reset_tree_by_currency(&st, lua, config_id, currency_id as u32)
    })?)?;

    Ok(())
}

fn purchase_rank(
    state: &Rc<RefCell<SimState>>, lua: &Lua, config_id: i32, node_id: u32,
) -> Result<bool> {
    use crate::traits::TRAIT_NODE_DB;
    let Some(node) = TRAIT_NODE_DB.get(&node_id) else { return Ok(false) };
    let max_ranks = super::traits_api_node::node_max_ranks(node);
    let mut s = state.borrow_mut();
    let current = *s.talents.node_ranks.get(&node_id).unwrap_or(&0);
    if current >= max_ranks as u32 {
        return Ok(false);
    }
    s.talents.node_ranks.insert(node_id, current + 1);
    drop(s);
    fire_trait_nodes_changed(lua)
}

fn refund_rank(
    state: &Rc<RefCell<SimState>>, lua: &Lua, _config_id: i32, node_id: u32,
) -> Result<bool> {
    let mut s = state.borrow_mut();
    let current = *s.talents.node_ranks.get(&node_id).unwrap_or(&0);
    if current == 0 {
        return Ok(false);
    }
    if current == 1 {
        s.talents.node_ranks.remove(&node_id);
        s.talents.node_selections.remove(&node_id);
    } else {
        s.talents.node_ranks.insert(node_id, current - 1);
    }
    drop(s);
    fire_trait_nodes_changed(lua)
}

fn set_selection(
    state: &Rc<RefCell<SimState>>, lua: &Lua,
    _config_id: i32, node_id: u32, entry_id: Option<u32>,
) -> Result<bool> {
    let mut s = state.borrow_mut();
    match entry_id {
        Some(eid) => {
            // Select an entry: set selection and ensure rank >= 1.
            s.talents.node_selections.insert(node_id, eid);
            let current = *s.talents.node_ranks.get(&node_id).unwrap_or(&0);
            if current == 0 {
                s.talents.node_ranks.insert(node_id, 1);
            }
        }
        None => {
            // nil entry_id = deselect/refund the selection node.
            let current = *s.talents.node_ranks.get(&node_id).unwrap_or(&0);
            if current == 0 {
                return Ok(false);
            }
            s.talents.node_ranks.remove(&node_id);
            s.talents.node_selections.remove(&node_id);
        }
    }
    drop(s);
    fire_trait_nodes_changed(lua)
}

fn reset_tree(
    state: &Rc<RefCell<SimState>>, lua: &Lua, config_id: i32,
) -> Result<bool> {
    let mut s = state.borrow_mut();
    s.talents.node_ranks.clear();
    s.talents.node_selections.clear();
    drop(s);
    fire_trait_config_updated(lua, config_id)
}

fn reset_tree_by_currency(
    state: &Rc<RefCell<SimState>>, lua: &Lua, config_id: i32, currency_id: u32,
) -> Result<bool> {
    let mut s = state.borrow_mut();
    let nodes_to_clear: Vec<u32> = s.talents.node_ranks.keys()
        .filter(|nid| s.talents.node_currency_map.get(nid) == Some(&currency_id))
        .copied()
        .collect();
    for nid in &nodes_to_clear {
        s.talents.node_ranks.remove(nid);
        s.talents.node_selections.remove(nid);
    }
    drop(s);
    fire_trait_config_updated(lua, config_id)
}

/// Fire events for staging operations (PurchaseRank, RefundRank, SetSelection):
/// - TRAIT_NODE_CHANGED for each node (invalidates nodeInfo cache)
/// - TRAIT_TREE_CURRENCY_INFO_UPDATED for the tree (refreshes point display)
/// Does NOT fire TRAIT_CONFIG_UPDATED — that only happens on CommitConfig.
fn fire_trait_nodes_changed(lua: &Lua) -> Result<bool> {
    fire_node_changed_events(lua)?;
    fire_currency_updated_event(lua)?;
    Ok(true)
}

/// Fire all events after a config commit or full reset:
/// - TRAIT_NODE_CHANGED + TRAIT_TREE_CURRENCY_INFO_UPDATED (via fire_trait_nodes_changed)
/// - TRAIT_CONFIG_UPDATED for the config (triggers tree reload in the UI)
fn fire_trait_config_updated(lua: &Lua, config_id: i32) -> Result<bool> {
    fire_trait_nodes_changed(lua)?;
    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    fire.call::<()>((
        lua.create_string("TRAIT_CONFIG_UPDATED")?,
        config_id as i64,
    ))?;
    Ok(true)
}

fn fire_node_changed_events(lua: &Lua) -> Result<()> {
    use crate::traits::TRAIT_TREE_DB;
    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    let event = lua.create_string("TRAIT_NODE_CHANGED")?;
    if let Some(tree) = TRAIT_TREE_DB.get(&790) {
        for &nid in tree.node_ids {
            fire.call::<()>((event.clone(), nid as i64))?;
        }
    }
    Ok(())
}

fn fire_currency_updated_event(lua: &Lua) -> Result<()> {
    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    fire.call::<()>((
        lua.create_string("TRAIT_TREE_CURRENCY_INFO_UPDATED")?,
        790i64,
    ))?;
    Ok(())
}

/// C_Traits tree-level APIs.
fn register_c_traits_tree(
    t: &mlua::Table, lua: &Lua, state: Rc<RefCell<SimState>>,
) -> Result<()> {
    t.set("InitializeViewLoadout", lua.create_function(|_, (_a, _b): (i32, i32)| Ok(true))?)?;
    t.set("GetTreeInfo", lua.create_function(create_tree_info)?)?;
    t.set("GetTreeNodes", lua.create_function(create_tree_nodes)?)?;
    t.set("GetAllTreeIDs", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetTraitSystemFlags", lua.create_function(|_, _id: i32| Ok(0))?)?;

    let st = Rc::clone(&state);
    t.set("GetTreeCurrencyInfo", lua.create_function(move |lua, (_config_id, tree_id): (i32, i32)| {
        create_tree_currency_info(lua, &st, tree_id)
    })?)?;

    Ok(())
}

/// C_Traits node/entry/definition-level APIs.
fn register_c_traits_node(
    t: &mlua::Table, lua: &Lua, state: Rc<RefCell<SimState>>,
) -> Result<()> {
    let st = Rc::clone(&state);
    t.set("GetNodeInfo", lua.create_function(move |lua, (cfg, nid): (Value, Value)| {
        super::traits_api_node::create_node_info(lua, &st, cfg, nid)
    })?)?;

    t.set("GetEntryInfo", lua.create_function(super::traits_api_node::create_entry_info)?)?;
    t.set("GetDefinitionInfo", lua.create_function(super::traits_api_node::create_definition_info)?)?;

    let st = Rc::clone(&state);
    t.set("GetConditionInfo", lua.create_function(move |lua, (_cfg, cid): (i32, i32)| {
        super::traits_api_node::create_condition_info(lua, &st, cid)
    })?)?;

    t.set("GetSubTreeInfo", lua.create_function(super::traits_api_node::create_sub_tree_info)?)?;

    let st = Rc::clone(&state);
    t.set("GetNodeCost", lua.create_function(move |lua, (_cfg, node_id): (i32, i32)| {
        create_node_cost(lua, &st, node_id as u32)
    })?)?;

    Ok(())
}

fn create_config_info(lua: &Lua, _config_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
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

/// Max points for a currency, derived from currency flags.
/// flags=4 → class (31 points), flags=8 → spec (30 points).
pub(crate) fn max_points_for_currency(currency_id: u32) -> u32 {
    use crate::traits::TRAIT_CURRENCY_DB;
    let Some(c) = TRAIT_CURRENCY_DB.get(&currency_id) else { return 0 };
    match c.flags {
        4 => 31,
        8 => 30,
        _ => 0,
    }
}

fn create_tree_currency_info(
    lua: &Lua, state: &Rc<RefCell<SimState>>, tree_id: i32,
) -> Result<Value> {
    use crate::traits::{TRAIT_CURRENCY_DB, TRAIT_TREE_DB};
    let Some(tree) = TRAIT_TREE_DB.get(&(tree_id as u32)) else {
        return Ok(Value::Nil);
    };
    let s = state.borrow();
    let arr = lua.create_table()?;
    for (i, &cid) in tree.currency_ids.iter().enumerate() {
        let entry = lua.create_table()?;
        entry.set("traitCurrencyID", cid as i64)?;
        let max_pts = max_points_for_currency(cid);
        let spent = s.talents.spent_for_currency(cid);
        let quantity = max_pts.saturating_sub(spent);
        entry.set("quantity", quantity as i64)?;
        entry.set("maxQuantity", max_pts as i64)?;
        entry.set("spent", spent as i64)?;
        let flags = TRAIT_CURRENCY_DB.get(&cid).map(|c| c.flags).unwrap_or(0);
        entry.set("flags", flags as i64)?;
        arr.set(i as i64 + 1, entry)?;
    }
    Ok(Value::Table(arr))
}

fn create_node_cost(
    lua: &Lua, state: &Rc<RefCell<SimState>>, node_id: u32,
) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    let s = state.borrow();
    if let Some(&cid) = s.talents.node_currency_map.get(&node_id) {
        let cost = lua.create_table()?;
        cost.set("ID", cid as i64)?;
        cost.set("amount", 1)?;
        t.set(1, cost)?;
    }
    Ok(t)
}

/// Check if `HasUnspentTalentPoints` — any class/spec currency has remaining points.
pub fn has_unspent_talent_points(state: &SimState) -> bool {
    use crate::traits::TRAIT_TREE_DB;
    let Some(tree) = TRAIT_TREE_DB.get(&790) else { return false };
    tree.currency_ids.iter().any(|&cid| {
        let max_pts = max_points_for_currency(cid);
        max_pts > 0 && state.talents.spent_for_currency(cid) < max_pts
    })
}
