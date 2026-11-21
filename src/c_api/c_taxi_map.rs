//! INFERRED (R88cp): taxi-map queries return supplied nodes for the requested UI map.
//! `ShouldMapShowTaxiNodes` is true when the simulator has at least one node for that map.

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table_with_capacity, table_get,
    table_set_static, val_to_string,
};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const SET_NODES_ERROR: &str = "A_Admin.SetTaxiMapNodes: expected an array of node tables";

#[derive(Clone, Debug, PartialEq)]
pub struct TaxiMapNode {
    pub map_id: i32,
    pub node_id: u32,
    pub name: String,
    pub atlas_name: String,
    pub texture_kit: String,
    pub faction: u8,
    pub x: f64,
    pub y: f64,
    pub state: u8,
    pub slot_index: u32,
    pub undiscovered: bool,
    pub use_special_icon: bool,
    pub special_icon_cost_string: Option<String>,
    pub map_layer_transition: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TaxiMapState {
    pub nodes: Vec<TaxiMapNode>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_TaxiMap")?;
    table_set_rust_fn_static(state, namespace, "GetAllTaxiNodes", get_all_taxi_nodes)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetTaxiNodesForMap",
        get_taxi_nodes_for_map,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "ShouldMapShowTaxiNodes",
        should_map_show_nodes,
    )?;
    let admin = ensure_namespace(state, "A_Admin")?;
    table_set_rust_fn_static(state, admin, "SetTaxiMapNodes", set_taxi_map_nodes)?;
    Ok(())
}

fn get_all_taxi_nodes(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    let nodes = nodes_for_map(state, map_id)?;
    let result = create_table_with_capacity(state, nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        let value = taxi_node_info(state, node)?;
        set_table_array(state, result, index as i64 + 1, value);
    }
    state.push(result);
    Ok(1)
}

fn get_taxi_nodes_for_map(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    let nodes = nodes_for_map(state, map_id)?;
    let result = create_table_with_capacity(state, nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        let value = map_taxi_node_info(state, node)?;
        set_table_array(state, result, index as i64 + 1, value);
    }
    state.push(result);
    Ok(1)
}

fn should_map_show_nodes(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    let should_show = borrow_state(state)?
        .taxi_map
        .nodes
        .iter()
        .any(|node| node.map_id == map_id);
    state.push(Val::Bool(should_show));
    Ok(1)
}

fn set_taxi_map_nodes(state: &mut LuaState) -> LuaResult<u32> {
    let nodes = read_taxi_map_nodes(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.taxi_map.nodes = nodes;
    Ok(0)
}

fn read_taxi_map_nodes(state: &mut LuaState, value: Val) -> LuaResult<Vec<TaxiMapNode>> {
    let Val::Table(array_ref) = value else {
        return Err(runtime_error(SET_NODES_ERROR));
    };
    let length = state
        .gc
        .tables
        .get(array_ref)
        .ok_or_else(|| runtime_error(SET_NODES_ERROR))?
        .len(&state.gc.string_arena);
    (1..=length)
        .map(|index| {
            let entry = state
                .gc
                .tables
                .get(array_ref)
                .map(|array| array.get_int(index))
                .unwrap_or(Val::Nil);
            read_taxi_map_node(state, entry)
        })
        .collect()
}

fn read_taxi_map_node(state: &mut LuaState, value: Val) -> LuaResult<TaxiMapNode> {
    let Val::Table(table) = value else {
        return Err(runtime_error(SET_NODES_ERROR));
    };
    Ok(TaxiMapNode {
        map_id: read_integer(state, table, "mapID", i32::MIN.into(), i32::MAX.into())? as i32,
        node_id: read_integer(state, table, "nodeID", 0.0, u32::MAX as f64)? as u32,
        name: read_string(state, table, "name")?,
        atlas_name: read_string(state, table, "atlasName")?,
        texture_kit: read_string(state, table, "textureKit")?,
        faction: read_integer(state, table, "faction", 0.0, 2.0)? as u8,
        x: read_number(state, table, "x")?,
        y: read_number(state, table, "y")?,
        state: read_integer(state, table, "state", 0.0, 2.0)? as u8,
        slot_index: read_integer(state, table, "slotIndex", 0.0, u32::MAX as f64)? as u32,
        undiscovered: read_bool(state, table, "isUndiscovered")?,
        use_special_icon: read_bool(state, table, "useSpecialIcon")?,
        special_icon_cost_string: read_optional_string(state, table, "specialIconCostString")?,
        map_layer_transition: read_bool(state, table, "isMapLayerTransition")?,
    })
}

fn read_number(state: &mut LuaState, table: GcRef, key: &str) -> LuaResult<f64> {
    match table_get(state, Val::Table(table), key) {
        Val::Num(value) if value.is_finite() => Ok(value),
        _ => Err(invalid_field(key)),
    }
}

fn read_integer(
    state: &mut LuaState,
    table: GcRef,
    key: &str,
    min: f64,
    max: f64,
) -> LuaResult<i64> {
    let value = read_number(state, table, key)?;
    if value.fract() == 0.0 && (min..=max).contains(&value) {
        Ok(value as i64)
    } else {
        Err(invalid_field(key))
    }
}

fn read_string(state: &mut LuaState, table: GcRef, key: &str) -> LuaResult<String> {
    let value = table_get(state, Val::Table(table), key);
    val_to_string(state, value).ok_or_else(|| invalid_field(key))
}

fn read_optional_string(
    state: &mut LuaState,
    table: GcRef,
    key: &str,
) -> LuaResult<Option<String>> {
    let value = table_get(state, Val::Table(table), key);
    match value {
        Val::Nil => Ok(None),
        value => val_to_string(state, value)
            .map(Some)
            .ok_or_else(|| invalid_field(key)),
    }
}

fn read_bool(state: &mut LuaState, table: GcRef, key: &str) -> LuaResult<bool> {
    match table_get(state, Val::Table(table), key) {
        Val::Bool(value) => Ok(value),
        _ => Err(invalid_field(key)),
    }
}

fn invalid_field(key: &str) -> rilua::LuaError {
    runtime_error(format!("A_Admin.SetTaxiMapNodes: invalid {key}"))
}

fn nodes_for_map(state: &LuaState, map_id: i32) -> LuaResult<Vec<TaxiMapNode>> {
    Ok(borrow_state(state)?
        .taxi_map
        .nodes
        .iter()
        .filter(|node| node.map_id == map_id)
        .cloned()
        .collect())
}

fn taxi_node_info(state: &mut LuaState, node: &TaxiMapNode) -> LuaResult<Val> {
    let position = super::c_map::create_world_position_vector(state, node.x, node.y)?;
    let info = create_table_with_capacity(state, 10);
    set_node_identity_fields(state, info, node, position);
    table_set_static(state, info, "state", Val::Num(f64::from(node.state)));
    table_set_static(
        state,
        info,
        "slotIndex",
        Val::Num(f64::from(node.slot_index)),
    );
    table_set_static(
        state,
        info,
        "useSpecialIcon",
        Val::Bool(node.use_special_icon),
    );
    table_set_static(
        state,
        info,
        "isMapLayerTransition",
        Val::Bool(node.map_layer_transition),
    );
    if let Some(value) = &node.special_icon_cost_string {
        let value = create_string(state, value);
        table_set_static(state, info, "specialIconCostString", value);
    }
    Ok(info)
}

fn map_taxi_node_info(state: &mut LuaState, node: &TaxiMapNode) -> LuaResult<Val> {
    let position = super::c_map::create_world_position_vector(state, node.x, node.y)?;
    let info = create_table_with_capacity(state, 7);
    set_node_identity_fields(state, info, node, position);
    let atlas_name = create_string(state, &node.atlas_name);
    table_set_static(state, info, "atlasName", atlas_name);
    table_set_static(state, info, "faction", Val::Num(f64::from(node.faction)));
    table_set_static(state, info, "isUndiscovered", Val::Bool(node.undiscovered));
    Ok(info)
}

fn set_node_identity_fields(state: &mut LuaState, info: Val, node: &TaxiMapNode, position: Val) {
    let name = create_string(state, &node.name);
    let texture_kit = create_string(state, &node.texture_kit);
    table_set_static(state, info, "nodeID", Val::Num(f64::from(node.node_id)));
    table_set_static(state, info, "position", position);
    table_set_static(state, info, "name", name);
    table_set_static(state, info, "textureKit", texture_kit);
}
