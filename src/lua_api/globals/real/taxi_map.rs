//! INFERRED (R88fx; native evidence pending): `GetTaxiMapID` reads the supplied taxi-map ID.
//!
//! Legacy slot globals (`retail-12-1-5-70688.csv` `_G` rows) read the nodes that
//! `A_Admin.SetTaxiMapNodes` supplies to `C_TaxiMap`, addressed by `slotIndex`.
//! INFERRED R88g9 (native evidence pending): the slot count is the highest
//! supplied slot; `TaxiNodeGetType` maps `Enum.FlightPathState` Current,
//! Reachable and Unreachable to the `TaxiFrame.lua` type names and an empty
//! slot to `"NONE"`; a direct flight is a route with one hop; hop coordinates
//! are the supplied node positions.

use crate::c_api::c_taxi_map::TaxiMapNode;
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetTaxiMapID", get_taxi_map_id)?;
    for (name, handler) in [
        ("NumTaxiNodes", num_taxi_nodes as rilua::vm::closure::RustFn),
        ("TaxiNodeGetType", taxi_node_get_type),
        ("TaxiNodeName", taxi_node_name),
        ("TaxiNodePosition", taxi_node_position),
        ("TaxiNodeCost", taxi_node_cost),
        ("GetNumRoutes", get_num_routes),
        ("TaxiIsDirectFlight", taxi_is_direct_flight),
        ("TaxiGetNodeSlot", taxi_get_node_slot),
        ("TaxiGetSrcX", taxi_get_src_x),
        ("TaxiGetSrcY", taxi_get_src_y),
        ("TaxiGetDestX", taxi_get_dest_x),
        ("TaxiGetDestY", taxi_get_dest_y),
    ] {
        LuaApiMut::register_function(lua, name, handler)?;
    }
    Ok(())
}

fn get_taxi_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = borrow_state(state)?.taxi_map.ui_map_id;
    if map_id == 0 {
        return Ok(0);
    }
    state.push(Val::Num(f64::from(map_id)));
    Ok(1)
}

fn node_at_slot(state: &LuaState, slot: u32) -> LuaResult<Option<TaxiMapNode>> {
    Ok(borrow_state(state)?
        .taxi_map
        .nodes
        .iter()
        .find(|node| node.slot_index == slot)
        .cloned())
}

fn slot_node(state: &LuaState) -> LuaResult<Option<TaxiMapNode>> {
    node_at_slot(state, u32::from_stack(state, 1)?)
}

fn num_taxi_nodes(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .taxi_map
        .nodes
        .iter()
        .map(|node| node.slot_index)
        .max()
        .unwrap_or(0);
    state.push(Val::Num(f64::from(count)));
    Ok(1)
}

fn taxi_node_get_type(state: &mut LuaState) -> LuaResult<u32> {
    let kind = match slot_node(state)?.map(|node| node.state) {
        Some(0) => "CURRENT",
        Some(1) => "REACHABLE",
        Some(_) => "UNREACHABLE",
        None => "NONE",
    };
    let value = create_string(state, kind);
    state.push(value);
    Ok(1)
}

fn taxi_node_name(state: &mut LuaState) -> LuaResult<u32> {
    let Some(node) = slot_node(state)? else {
        return Ok(0);
    };
    let value = create_string(state, &node.name);
    state.push(value);
    Ok(1)
}

fn taxi_node_position(state: &mut LuaState) -> LuaResult<u32> {
    let Some(node) = slot_node(state)? else {
        return Ok(0);
    };
    state.push(Val::Num(node.x));
    state.push(Val::Num(node.y));
    Ok(2)
}

fn taxi_node_cost(state: &mut LuaState) -> LuaResult<u32> {
    let cost = slot_node(state)?.map_or(0, |node| node.cost);
    state.push(Val::Num(cost as f64));
    Ok(1)
}

fn get_num_routes(state: &mut LuaState) -> LuaResult<u32> {
    let count = slot_node(state)?.map_or(0, |node| node.routes.len());
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn taxi_is_direct_flight(state: &mut LuaState) -> LuaResult<u32> {
    let direct = slot_node(state)?.is_some_and(|node| node.routes.len() == 1);
    state.push(Val::Bool(direct));
    Ok(1)
}

fn route_hop(state: &LuaState) -> LuaResult<Option<(u32, u32)>> {
    let route = usize::try_from(i64::from_stack(state, 2)?).unwrap_or(0);
    let Some(node) = slot_node(state)? else {
        return Ok(None);
    };
    Ok(route
        .checked_sub(1)
        .and_then(|index| node.routes.get(index).copied()))
}

fn taxi_get_node_slot(state: &mut LuaState) -> LuaResult<u32> {
    let is_source = bool::from_stack(state, 3)?;
    let Some((source, destination)) = route_hop(state)? else {
        return Ok(0);
    };
    let slot = if is_source { source } else { destination };
    state.push(Val::Num(f64::from(slot)));
    Ok(1)
}

fn push_hop_coordinate(state: &mut LuaState, source: bool, use_x: bool) -> LuaResult<u32> {
    let Some((from, to)) = route_hop(state)? else {
        return Ok(0);
    };
    let Some(node) = node_at_slot(state, if source { from } else { to })? else {
        return Ok(0);
    };
    state.push(Val::Num(if use_x { node.x } else { node.y }));
    Ok(1)
}

fn taxi_get_src_x(state: &mut LuaState) -> LuaResult<u32> {
    push_hop_coordinate(state, true, true)
}

fn taxi_get_src_y(state: &mut LuaState) -> LuaResult<u32> {
    push_hop_coordinate(state, true, false)
}

fn taxi_get_dest_x(state: &mut LuaState) -> LuaResult<u32> {
    push_hop_coordinate(state, false, true)
}

fn taxi_get_dest_y(state: &mut LuaState) -> LuaResult<u32> {
    push_hop_coordinate(state, false, false)
}
