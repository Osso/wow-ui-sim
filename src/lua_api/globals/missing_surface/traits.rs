use super::{ensure_namespace, set_table_array};
use crate::lua_api::globals::hero_talents;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table,
    create_table_with_capacity, frame_ref, table_set,
};
use crate::lua_api::script_helpers::{get_dispatch_script, get_event_listeners};
use crate::lua_api::talent_state;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use crate::specializations;
use crate::spell_descriptions;
use crate::traits::{
    TRAIT_COND_DB, TRAIT_CURRENCY_DB, TRAIT_DEFINITION_DB, TRAIT_ENTRY_DB, TRAIT_NODE_DB,
    TRAIT_SUBTREE_DB, TRAIT_TREE_DB,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

mod c_traits;
mod class_talents;

type LuaTableRef = rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>;

const DELVES_COMPANION_CONFIG_ID: i32 = 9201;
const DELVES_COMPANION_TRAIT_TREE_ID: u32 = 9201;
const DELVES_COMPANION_NODE_IDS: [u32; 3] = [9301, 9302, 9303];
const ACTIVE_ENTRY_HASH_FIELDS: usize = 2;
const VISIBLE_EDGE_HASH_FIELDS: usize = 4;
const NODE_INFO_HASH_FIELDS: usize = 29;

pub(super) fn register_trait_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_traits::register_c_traits(state)?;
    class_talents::register_c_class_talents(state)?;
    Ok(())
}

fn fire_named_event_with_arg(state: &mut LuaState, event_name: &str, arg: Val) {
    for widget_id in get_event_listeners(state, event_name) {
        let Some(handler) = get_dispatch_script(state, widget_id, "OnEvent") else {
            continue;
        };
        let Ok(frame) = frame_ref(state, widget_id) else {
            continue;
        };
        let event_name_val = create_string(state, event_name);
        let _ = call_function_state(state, handler, &[frame, event_name_val, arg]);
    }
}

/// Fire `TRAIT_NODE_CHANGED` for `changed_node_id` and nodes that directly
/// depend on it through incoming edge sources.
fn fire_trait_node_changed_with_dependents(state: &mut LuaState, changed_node_id: u32) {
    let affected = affected_trait_node_ids(changed_node_id);
    for node_id in affected {
        fire_named_event_with_arg(state, "TRAIT_NODE_CHANGED", Val::Num(node_id as f64));
    }
}

fn affected_trait_node_ids(changed_node_id: u32) -> Vec<u32> {
    let mut affected = Vec::with_capacity(8);
    let mut seen = HashSet::with_capacity(8);
    affected.push(changed_node_id);
    seen.insert(changed_node_id);

    for node in TRAIT_NODE_DB.values() {
        let depends_on_changed_node = node
            .edges
            .iter()
            .any(|edge| edge.source_node_id == changed_node_id);
        if depends_on_changed_node && seen.insert(node.id) {
            affected.push(node.id);
        }
    }
    affected
}

/// Fire `TRAIT_TREE_CURRENCY_INFO_UPDATED` for the node's owning tree.
fn fire_trait_tree_currency_info_updated_for_node(state: &mut LuaState, node_id: u32) {
    let Some(tree_id) = TRAIT_NODE_DB.get(&node_id).map(|node| node.tree_id) else {
        return;
    };
    fire_named_event_with_arg(
        state,
        "TRAIT_TREE_CURRENCY_INFO_UPDATED",
        Val::Num(tree_id as f64),
    );
}

fn current_spec_id(state: &LuaState) -> Option<u32> {
    let sim = borrow_state(state).ok()?;
    current_spec_id_from_sim(&sim)
}

fn current_spec_id_from_sim(sim: &crate::lua_api::SimState) -> Option<u32> {
    sim.talents.view_spec_id.or_else(|| player_spec_id(sim))
}

fn player_spec_id(state: &crate::lua_api::SimState) -> Option<u32> {
    specializations::specs_for_class(state.player.class_index as u32)
        .nth(state.player.active_spec_index.max(1) as usize - 1)
        .map(|spec| spec.id)
}

fn config_name(config_id: i32) -> &'static str {
    match config_id {
        DELVES_COMPANION_CONFIG_ID => "Delves Companion",
        101 => "Holy Mythic+",
        102 => "Holy Raid",
        201 => "Protection Raid",
        202 => "Protection Mythic+",
        301 => "Retribution Raid",
        302 => "Retribution Mythic+",
        _ => "Default Loadout",
    }
}

fn config_spec_id(config_id: i32) -> Option<u32> {
    match config_id {
        101 | 102 => Some(65),
        201 | 202 => Some(66),
        301 | 302 => Some(70),
        _ => None,
    }
}

fn trait_node_spec_set(node: &crate::traits::TraitNodeInfo) -> u32 {
    node.cond_ids
        .iter()
        .chain(node.group_cond_ids.iter())
        .filter_map(|cond_id| TRAIT_COND_DB.get(cond_id))
        .find(|cond| cond.cond_type == 1)
        .map(|cond| cond.spec_set_id)
        .unwrap_or(0)
}

fn trait_node_is_visible_for_spec(node_id: u32, spec_id: Option<u32>) -> bool {
    let Some(node) = TRAIT_NODE_DB.get(&node_id) else {
        return true;
    };
    let required_spec_set = trait_node_spec_set(node);
    spec_set_contains_spec(required_spec_set, spec_id)
}

fn push_u32_array(state: &mut LuaState, values: impl IntoIterator<Item = u32>) -> Val {
    let table = create_table(state);
    for (index, value) in values.into_iter().enumerate() {
        set_table_array(state, table, index as i64 + 1, Val::Num(value as f64));
    }
    table
}

fn push_i32_array(state: &mut LuaState, values: impl IntoIterator<Item = i32>) -> Val {
    let table = create_table(state);
    for (index, value) in values.into_iter().enumerate() {
        set_table_array(state, table, index as i64 + 1, Val::Num(value as f64));
    }
    table
}

const TREE_HASH_VALUE_MIXER: u32 = 0x9E37_79B9;
const TREE_HASH_LANE_MIXER: u32 = 0x85EB_CA6B;
const TREE_HASH_INITIAL_LANES: [u32; 4] = [0xC2B2_AE35, 0x27D4_EB2F, 0x1656_67B1, 0x85EB_CA77];

fn mix_tree_hash_lane(lane: &mut u32, value: u32) {
    *lane ^= value.wrapping_mul(TREE_HASH_VALUE_MIXER);
    *lane = lane.rotate_left(7).wrapping_mul(TREE_HASH_LANE_MIXER);
}

fn trait_tree_hash_bytes(tree: &crate::traits::TraitTreeInfo) -> [u8; 16] {
    let mut lanes = TREE_HASH_INITIAL_LANES;
    mix_tree_hash_lane(&mut lanes[0], tree.id);
    mix_tree_hash_lane(&mut lanes[1], tree.first_node_id);
    mix_tree_hash_lane(&mut lanes[2], tree.flags);
    mix_tree_hash_lane(&mut lanes[3], tree.node_ids.len() as u32);
    mix_tree_hash_lane(&mut lanes[0], tree.currency_ids.len() as u32);

    let lane_count = lanes.len();
    for (index, &node_id) in tree.node_ids.iter().enumerate() {
        mix_tree_hash_lane(&mut lanes[index % lane_count], node_id);
    }
    for (index, &currency_id) in tree.currency_ids.iter().enumerate() {
        mix_tree_hash_lane(&mut lanes[(index + 1) % lane_count], currency_id);
    }

    let mut bytes = [0u8; 16];
    for (index, lane) in lanes.into_iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&lane.to_le_bytes());
    }
    bytes
}

fn node_max_ranks(node: &crate::traits::TraitNodeInfo) -> u32 {
    node.entry_ids
        .first()
        .and_then(|entry_id| TRAIT_ENTRY_DB.get(entry_id))
        .map(|entry| entry.max_ranks)
        .unwrap_or(1)
}

fn total_node_max_ranks(node: &crate::traits::TraitNodeInfo) -> u32 {
    let total: u32 = node
        .entry_ids
        .iter()
        .filter_map(|entry_id| TRAIT_ENTRY_DB.get(entry_id))
        .map(|entry| entry.max_ranks)
        .sum();
    if total > 0 {
        total
    } else {
        node_max_ranks(node)
    }
}

fn spec_set_contains_spec(spec_set_id: u32, spec_id: Option<u32>) -> bool {
    spec_set_id == 0 || spec_id.map(hero_talents::spec_id_to_spec_set) == Some(spec_set_id)
}

fn check_spec_conditions_met(
    node: &crate::traits::TraitNodeInfo,
    state: &crate::lua_api::SimState,
) -> bool {
    check_spec_conditions_met_for_spec(node, current_spec_id_from_sim(state).or(Some(66)))
}

fn check_spec_conditions_met_for_spec(
    node: &crate::traits::TraitNodeInfo,
    spec_id: Option<u32>,
) -> bool {
    let mut saw_spec_condition = false;
    let mut has_matching_spec = false;
    for &cond_id in node.cond_ids.iter().chain(node.group_cond_ids.iter()) {
        let Some(cond) = TRAIT_COND_DB.get(&cond_id) else {
            continue;
        };
        if cond.cond_type != 1 {
            continue;
        }
        saw_spec_condition = true;
        if spec_set_contains_spec(cond.spec_set_id, spec_id) {
            has_matching_spec = true;
        }
    }
    !saw_spec_condition || has_matching_spec
}

fn check_node_available(
    node: &crate::traits::TraitNodeInfo,
    state: &crate::lua_api::SimState,
) -> bool {
    for &cond_id in node.cond_ids.iter() {
        let Some(cond) = TRAIT_COND_DB.get(&cond_id) else {
            continue;
        };
        if cond.cond_type != 0 || cond.currency_id == 0 {
            continue;
        }
        if state.talents.spent_for_currency(cond.currency_id) < cond.spent_amount {
            return false;
        }
    }
    true
}

fn check_edge_requirements(
    node: &crate::traits::TraitNodeInfo,
    state: &crate::lua_api::SimState,
) -> bool {
    let mut has_sufficient = false;
    let mut any_sufficient_met = false;
    for edge in node.edges {
        let purchased = state
            .talents
            .node_ranks
            .get(&edge.source_node_id)
            .copied()
            .unwrap_or(0)
            > 0;
        match edge.edge_type {
            2 => {
                has_sufficient = true;
                if purchased {
                    any_sufficient_met = true;
                }
            }
            3 if !purchased => return false,
            _ => {}
        }
    }
    !has_sufficient || any_sufficient_met
}

fn max_points_for_currency(currency_id: u32) -> u32 {
    let Some(currency) = TRAIT_CURRENCY_DB.get(&currency_id) else {
        return 0;
    };
    match currency.flags {
        4 => 31,
        8 => 30,
        _ => 0,
    }
}

const HERO_TALENT_POINT_BUDGET: u32 = 11;

fn has_unspent_talent_points(state: &crate::lua_api::SimState) -> bool {
    let Some(tree) = TRAIT_TREE_DB.get(&790) else {
        return false;
    };
    tree.currency_ids.iter().any(|&currency_id| {
        let max_points = max_points_for_currency(currency_id);
        max_points > 0 && state.talents.spent_for_currency(currency_id) < max_points
    })
}

fn class_talent_tree_for_spec(spec_id: u32) -> Option<u32> {
    hero_talents::class_tree_for_spec(spec_id)
}

fn starter_build_candidate(
    state: &crate::lua_api::SimState,
    node: &crate::traits::TraitNodeInfo,
    active_hero_subtree: Option<u32>,
) -> Option<(u8, i32, i32, u32, u32)> {
    let ranks_purchased = state.talents.node_ranks.get(&node.id).copied().unwrap_or(0);
    let total_max_ranks = total_node_max_ranks(node);
    if ranks_purchased >= total_max_ranks || !check_has_currency(node.id, state) {
        return None;
    }

    let subtree_priority = match (node.sub_tree_id, active_hero_subtree) {
        (0, _) => 0,
        (subtree_id, Some(active)) if subtree_id == active => 1,
        _ => 2,
    };
    let entry_id = state
        .talents
        .node_selections
        .get(&node.id)
        .copied()
        .or_else(|| node.entry_ids.first().copied())
        .unwrap_or(0);

    Some((subtree_priority, node.pos_y, node.pos_x, node.id, entry_id))
}

fn starter_build_purchase_for_state(state: &crate::lua_api::SimState) -> Option<(u32, u32)> {
    sorted_starter_build_candidates(state)
        .into_iter()
        .next()
        .map(|(_, _, _, node_id, entry_id)| (node_id, entry_id))
}

fn sorted_starter_build_candidates(
    state: &crate::lua_api::SimState,
) -> Vec<(u8, i32, i32, u32, u32)> {
    let active_hero_subtree = state.talents.active_hero_subtree();
    let mut candidate_nodes = TRAIT_NODE_DB
        .values()
        .filter(|node| {
            check_spec_conditions_met(node, state)
                && check_node_available(node, state)
                && check_edge_requirements(node, state)
        })
        .filter_map(|node| starter_build_candidate(state, node, active_hero_subtree))
        .collect::<Vec<_>>();

    candidate_nodes.sort_unstable();
    candidate_nodes
}

fn check_has_currency(node_id: u32, state: &crate::lua_api::SimState) -> bool {
    let Some(&currency_id) = state.talents.node_currency_map.get(&node_id) else {
        return true;
    };
    let max_points = max_points_for_currency(currency_id);
    max_points == 0 || state.talents.spent_for_currency(currency_id) < max_points
}

fn push_node_active_entry(state: &mut LuaState, info: Val, lookup_node_id: Option<u32>) {
    let entry_id = borrow_state(state).ok().and_then(|sim| {
        lookup_node_id
            .and_then(|id| sim.talents.node_selections.get(&id).copied())
            .or_else(|| {
                lookup_node_id
                    .and_then(|id| TRAIT_NODE_DB.get(&id))
                    .and_then(|node| matches!(node.node_type, 0 | 1).then_some(node.entry_ids[0]))
            })
    });
    let Some(entry_id) = entry_id else {
        table_set(state, info, "activeEntry", Val::Nil);
        return;
    };

    let active_entry = create_table_with_capacity(state, ACTIVE_ENTRY_HASH_FIELDS);
    let rank = borrow_state(state)
        .ok()
        .and_then(|sim| lookup_node_id.and_then(|id| sim.talents.node_ranks.get(&id).copied()))
        .unwrap_or(0);
    table_set(state, active_entry, "entryID", Val::Num(entry_id as f64));
    table_set(state, active_entry, "rank", Val::Num(rank as f64));
    table_set(state, info, "activeEntry", active_entry);
}

fn push_node_rank_fields(state: &mut LuaState, info: Val, lookup_node_id: Option<u32>) {
    let ranks_purchased = borrow_state(state)
        .ok()
        .and_then(|sim| lookup_node_id.and_then(|id| sim.talents.node_ranks.get(&id).copied()))
        .unwrap_or(0);
    let rank_val = Val::Num(ranks_purchased as f64);
    table_set(state, info, "ranksPurchased", rank_val);
    table_set(state, info, "currentRank", rank_val);
    table_set(state, info, "activeRank", rank_val);
    table_set(state, info, "ranksIncreased", rank_val);
    let entry_ranks_increased = create_table(state);
    table_set(
        state,
        info,
        "entryIDToRanksIncreased",
        entry_ranks_increased,
    );
}

fn push_node_array_fields(state: &mut LuaState, info: Val, node: &crate::traits::TraitNodeInfo) {
    let entry_ids = push_u32_array(state, node.entry_ids.iter().copied());
    table_set(state, info, "entryIDs", entry_ids);

    let condition_ids = push_u32_array(state, node.cond_ids.iter().copied());
    table_set(state, info, "conditionIDs", condition_ids);

    let group_ids = push_u32_array(state, node.group_ids.iter().copied());
    table_set(state, info, "groupIDs", group_ids);
}

fn push_node_visible_edges(
    state: &mut LuaState,
    info: Val,
    node: &crate::traits::TraitNodeInfo,
    active_source_nodes: &[u32],
) {
    let visible_edges = create_table(state);
    for (index, edge) in node.edges.iter().enumerate() {
        let edge_info = create_table_with_capacity(state, VISIBLE_EDGE_HASH_FIELDS);
        table_set(
            state,
            edge_info,
            "targetNode",
            Val::Num(edge.source_node_id as f64),
        );
        table_set(state, edge_info, "type", Val::Num(edge.edge_type as f64));
        table_set(
            state,
            edge_info,
            "visualStyle",
            Val::Num(edge.visual_style as f64),
        );
        let is_active = active_source_nodes.contains(&edge.source_node_id);
        table_set(state, edge_info, "isActive", Val::Bool(is_active));
        set_table_array(state, visible_edges, index as i64 + 1, edge_info);
    }
    table_set(state, info, "visibleEdges", visible_edges);
}

struct NodeDbRuntimeFields {
    ranks_purchased: u32,
    active_entry_id: Option<u32>,
    active_hero_subtree: Option<u32>,
    is_spec_visible: bool,
    is_available: bool,
    meets_edge_requirements: bool,
    has_currency: bool,
    active_source_nodes: Vec<u32>,
}

fn spec_id_for_config_query(config_id: i32, sim: &crate::lua_api::SimState) -> Option<u32> {
    config_spec_id(config_id).or_else(|| current_spec_id_from_sim(sim))
}

fn node_db_runtime_fields(
    state: &LuaState,
    lookup_node_id: Option<u32>,
    node: &crate::traits::TraitNodeInfo,
    config_id: i32,
) -> NodeDbRuntimeFields {
    match borrow_state(state).ok() {
        Some(sim) => {
            let spec_id = spec_id_for_config_query(config_id, &sim);
            NodeDbRuntimeFields {
                ranks_purchased: lookup_node_id
                    .and_then(|id| sim.talents.node_ranks.get(&id).copied())
                    .unwrap_or(0),
                active_entry_id: lookup_node_id
                    .and_then(|id| sim.talents.node_selections.get(&id).copied()),
                active_hero_subtree: sim.talents.active_hero_subtree(),
                is_spec_visible: check_spec_conditions_met_for_spec(node, spec_id),
                is_available: check_node_available(node, &sim),
                meets_edge_requirements: check_edge_requirements(node, &sim),
                has_currency: lookup_node_id
                    .is_none_or(|node_id| check_has_currency(node_id, &sim)),
                active_source_nodes: ranked_node_ids(&sim),
            }
        }
        None => NodeDbRuntimeFields {
            ranks_purchased: 0,
            active_entry_id: None,
            active_hero_subtree: None,
            is_spec_visible: true,
            is_available: false,
            meets_edge_requirements: false,
            has_currency: true,
            active_source_nodes: Vec::new(),
        },
    }
}

fn ranked_node_ids(sim: &crate::lua_api::SimState) -> Vec<u32> {
    sim.talents
        .node_ranks
        .iter()
        .filter_map(|(&node_id, &rank)| (rank > 0).then_some(node_id))
        .collect()
}

fn push_node_db_fields(
    state: &mut LuaState,
    info: Val,
    config_id: i32,
    lookup_node_id: Option<u32>,
) {
    let Some(node) = lookup_node_id.and_then(|id| TRAIT_NODE_DB.get(&id)) else {
        push_missing_node_db_fields(state, info);
        return;
    };

    let runtime = node_db_runtime_fields(state, lookup_node_id, node, config_id);
    push_existing_node_db_fields(state, info, config_id, lookup_node_id, node, runtime);
}

fn push_existing_node_db_fields(
    state: &mut LuaState,
    info: Val,
    config_id: i32,
    lookup_node_id: Option<u32>,
    node: &crate::traits::TraitNodeInfo,
    runtime: NodeDbRuntimeFields,
) {
    let total_max_ranks = total_node_max_ranks(node);
    let spec_id = borrow_state(state)
        .ok()
        .and_then(|sim| spec_id_for_config_query(config_id, &sim));
    let is_visible = lookup_node_id.is_some_and(|id| trait_node_is_visible_for_spec(id, spec_id))
        && runtime.is_spec_visible;
    let can_purchase_rank = runtime.ranks_purchased < total_max_ranks
        && runtime.is_available
        && runtime.meets_edge_requirements
        && runtime.has_currency;

    table_set(state, info, "posX", Val::Num(node.pos_x as f64));
    table_set(state, info, "posY", Val::Num(node.pos_y as f64));
    table_set(state, info, "type", Val::Num(node.node_type as f64));
    table_set(state, info, "flags", Val::Num(node.flags as f64));
    push_node_array_fields(state, info, node);
    push_committed_rank_entry_ids(state, info, &runtime);
    push_node_rank_limits(state, info, node, total_max_ranks);
    push_node_availability_fields(state, info, is_visible, can_purchase_rank, &runtime);
    push_node_subtree_fields(state, info, node.sub_tree_id, runtime.active_hero_subtree);
    push_node_visible_edges(state, info, node, &runtime.active_source_nodes);
}

fn push_committed_rank_entry_ids(state: &mut LuaState, info: Val, runtime: &NodeDbRuntimeFields) {
    let entry_ids = create_table(state);
    if let Some(active_entry_id) = runtime
        .active_entry_id
        .filter(|_| runtime.ranks_purchased > 0)
    {
        set_table_array(state, entry_ids, 1, Val::Num(active_entry_id as f64));
    }
    table_set(state, info, "entryIDsWithCommittedRanks", entry_ids);
}

fn push_node_rank_limits(
    state: &mut LuaState,
    info: Val,
    node: &crate::traits::TraitNodeInfo,
    total_max_ranks: u32,
) {
    table_set(
        state,
        info,
        "maxRanks",
        Val::Num(node_max_ranks(node) as f64),
    );
    table_set(
        state,
        info,
        "totalMaxRanks",
        Val::Num(total_max_ranks as f64),
    );
}

fn push_node_availability_fields(
    state: &mut LuaState,
    info: Val,
    is_visible: bool,
    can_purchase_rank: bool,
    runtime: &NodeDbRuntimeFields,
) {
    table_set(state, info, "isVisible", Val::Bool(is_visible));
    table_set(state, info, "isAvailable", Val::Bool(runtime.is_available));
    table_set(state, info, "isDisplayError", Val::Bool(false));
    table_set(state, info, "canPurchaseRank", Val::Bool(can_purchase_rank));
    table_set(
        state,
        info,
        "canRefundRank",
        Val::Bool(runtime.ranks_purchased > 0),
    );
    table_set(
        state,
        info,
        "meetsEdgeRequirements",
        Val::Bool(runtime.meets_edge_requirements),
    );
    table_set(state, info, "isCascadeRepurchasable", Val::Bool(false));
    table_set(state, info, "cascadeRepurchaseEntryID", Val::Nil);
}

fn push_node_subtree_fields(
    state: &mut LuaState,
    info: Val,
    sub_tree_id: u32,
    active_hero_subtree: Option<u32>,
) {
    if sub_tree_id == 0 {
        table_set(state, info, "subTreeID", Val::Nil);
        table_set(state, info, "subTreeActive", Val::Nil);
        return;
    }

    table_set(state, info, "subTreeID", Val::Num(sub_tree_id as f64));
    let sub_tree_active = active_hero_subtree == Some(sub_tree_id);
    table_set(state, info, "subTreeActive", Val::Bool(sub_tree_active));
}

fn push_missing_node_db_fields(state: &mut LuaState, info: Val) {
    table_set(state, info, "posX", Val::Num(0.0));
    table_set(state, info, "posY", Val::Num(0.0));
    table_set(state, info, "type", Val::Num(0.0));
    table_set(state, info, "flags", Val::Num(0.0));
    push_empty_node_id_arrays(state, info);
    table_set(state, info, "maxRanks", Val::Num(0.0));
    table_set(state, info, "totalMaxRanks", Val::Num(0.0));
    push_missing_node_status_fields(state, info);
    table_set(state, info, "subTreeID", Val::Nil);
    table_set(state, info, "subTreeActive", Val::Nil);
}

fn push_empty_node_id_arrays(state: &mut LuaState, info: Val) {
    let entry_ids = create_table(state);
    table_set(state, info, "entryIDs", entry_ids);
    let entry_ids_with_committed_ranks = create_table(state);
    table_set(
        state,
        info,
        "entryIDsWithCommittedRanks",
        entry_ids_with_committed_ranks,
    );
    let condition_ids = create_table(state);
    table_set(state, info, "conditionIDs", condition_ids);
    let group_ids = create_table(state);
    table_set(state, info, "groupIDs", group_ids);
    let visible_edges = create_table(state);
    table_set(state, info, "visibleEdges", visible_edges);
}

fn push_missing_node_status_fields(state: &mut LuaState, info: Val) {
    table_set(state, info, "isVisible", Val::Bool(false));
    table_set(state, info, "isAvailable", Val::Bool(false));
    table_set(state, info, "isDisplayError", Val::Bool(false));
    table_set(state, info, "canPurchaseRank", Val::Bool(false));
    table_set(state, info, "canRefundRank", Val::Bool(false));
    table_set(state, info, "meetsEdgeRequirements", Val::Bool(false));
    table_set(state, info, "isCascadeRepurchasable", Val::Bool(false));
    table_set(state, info, "cascadeRepurchaseEntryID", Val::Nil);
}

fn push_node_info(state: &mut LuaState, config_id: i32, node_id: i32) -> Val {
    let info = create_table_with_capacity(state, NODE_INFO_HASH_FIELDS);
    table_set(state, info, "ID", Val::Num(node_id as f64));
    table_set(state, info, "id", Val::Num(node_id as f64));
    let lookup_node_id = u32::try_from(node_id).ok();
    push_node_active_entry(state, info, lookup_node_id);
    push_node_rank_fields(state, info, lookup_node_id);
    push_node_db_fields(state, info, config_id, lookup_node_id);
    info
}

fn trait_tree_id_for_config(state: &LuaState, config_id: i32) -> Option<u32> {
    if config_id == DELVES_COMPANION_CONFIG_ID {
        return Some(DELVES_COMPANION_TRAIT_TREE_ID);
    }

    if config_id == 1 {
        return Some(672);
    }

    config_spec_id(config_id)
        .and_then(class_talent_tree_for_spec)
        .or_else(|| {
            borrow_state(state).ok().and_then(|sim| {
                sim.talents
                    .is_active_config(config_id)
                    .then(|| class_talent_tree_for_spec(sim.talents.active_spec_id))
                    .flatten()
            })
        })
}

fn config_id_for_tree_id(state: &LuaState, tree_id: u32) -> Option<i32> {
    if tree_id == DELVES_COMPANION_TRAIT_TREE_ID {
        return Some(DELVES_COMPANION_CONFIG_ID);
    }

    if tree_id == 672 {
        return Some(1);
    }

    borrow_state(state).ok().and_then(|sim| {
        let active_config_id = sim.talents.active_config_id;
        (trait_tree_id_for_config(state, active_config_id) == Some(tree_id))
            .then_some(active_config_id)
    })
}

fn config_id_for_system_id(state: &LuaState, system_id: i32) -> Option<i32> {
    match system_id {
        1 => borrow_state(state)
            .ok()
            .map(|sim| sim.talents.active_config_id),
        _ => None,
    }
}

fn trait_system_flags_for_config(state: &LuaState, config_id: i32) -> u32 {
    trait_tree_id_for_config(state, config_id)
        .and_then(|tree_id| TRAIT_TREE_DB.get(&tree_id))
        .map(|tree| tree.flags)
        .unwrap_or(0)
}
