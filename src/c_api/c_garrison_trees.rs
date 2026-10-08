//! Server-input garrison tree catalog and active tree context.
//!
//! This models the three 7.2.5 tree queries, not tree progression/research or a
//! historical Legion catalog. Unknown class/type pairs return no values, matching
//! the current cached API documentation's MayReturnNothing contract.

use std::collections::BTreeMap;

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug)]
pub struct GarrisonTalentTree {
    pub garrison_type: i64,
    pub class_id: i64,
    pub friendship_faction_id: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct GarrisonTrees {
    pub current_tree_id: Option<i64>,
    /// IDs are returned in ascending order; no native catalog-order claim.
    pub catalog: BTreeMap<i64, GarrisonTalentTree>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Garrison")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCurrentGarrTalentTreeID",
        current_tree_id,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCurrentGarrTalentTreeFriendshipFactionID",
        current_friendship_faction_id,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetTalentTreeIDsByClassID",
        tree_ids_by_class,
    )?;
    Ok(())
}

fn current_tree_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.garrison_trees.current_tree_id;
    state.push(id.map_or(Val::Nil, |id| Val::Num(id as f64)));
    Ok(1)
}

fn current_friendship_faction_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = {
        let sim = borrow_state(state)?;
        let trees = &sim.garrison_trees;
        trees
            .current_tree_id
            .and_then(|id| trees.catalog.get(&id))
            .and_then(|tree| tree.friendship_faction_id)
    };
    state.push(id.map_or(Val::Nil, |id| Val::Num(id as f64)));
    Ok(1)
}

fn tree_ids_by_class(state: &mut LuaState) -> LuaResult<u32> {
    let garrison_type = i64::from_stack(state, 1)?;
    let class_id = i64::from_stack(state, 2)?;
    let ids: Vec<i64> = borrow_state(state)?
        .garrison_trees
        .catalog
        .iter()
        .filter(|(_, tree)| tree.garrison_type == garrison_type && tree.class_id == class_id)
        .map(|(&id, _)| id)
        .collect();
    if ids.is_empty() {
        return Ok(0);
    }
    let array = create_table(state);
    for (index, id) in ids.iter().enumerate() {
        set_table_array(state, array, index as i64 + 1, Val::Num(*id as f64));
    }
    state.push(array);
    Ok(1)
}
