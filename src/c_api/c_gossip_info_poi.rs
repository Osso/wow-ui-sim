//! INFERRED (R88cz; native evidence pending): per-map gossip POI records.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set_static,
};
use crate::lua_api::sim_substates::GossipPoiInfo;
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_GossipInfo")?;
    table_set_rust_fn_static(state, namespace, "GetPoiForUiMapID", get_poi_for_ui_map)?;
    table_set_rust_fn_static(state, namespace, "GetPoiInfo", get_poi_info)?;
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetGossipPoi", set_gossip_poi)
}

fn get_poi_for_ui_map(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let poi_id = borrow_state(state)?
        .gossip
        .pois
        .get(&ui_map_id)
        .map(|poi| poi.poi_id);
    state.push(poi_id.map_or(Val::Nil, |id| Val::Num(id as f64)));
    Ok(1)
}

fn get_poi_info(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let poi_id = i32::from_stack(state, 2)?;
    let poi = borrow_state(state)?
        .gossip
        .pois
        .get(&ui_map_id)
        .filter(|poi| poi.poi_id == poi_id)
        .cloned();
    let Some(poi) = poi else {
        return Ok(0);
    };
    let info = push_poi_info(state, &poi);
    state.push(info);
    Ok(1)
}

fn push_poi_info(state: &mut LuaState, poi: &GossipPoiInfo) -> Val {
    let info = create_table(state);
    let name = create_string(state, &poi.name);
    table_set_static(state, info, "name", name);
    table_set_static(
        state,
        info,
        "textureIndex",
        Val::Num(poi.texture_index as f64),
    );
    let atlas_name = optional_string(state, poi.atlas_name.as_deref());
    table_set_static(state, info, "atlasName", atlas_name);
    table_set_static(state, info, "inBattleMap", Val::Bool(false));
    let position = push_position(state, poi.position);
    table_set_static(state, info, "position", position);
    info
}

fn push_position(state: &mut LuaState, (x, y): (f64, f64)) -> Val {
    let position = create_table(state);
    table_set_static(state, position, "x", Val::Num(x));
    table_set_static(state, position, "y", Val::Num(y));
    let Val::Table(position) = position else {
        unreachable!("create_table must return a table");
    };
    table_set_rust_fn_static(state, position, "GetXY", position_get_xy)
        .expect("gossip POI position getter registration");
    Val::Table(position)
}

fn position_get_xy(state: &mut LuaState) -> LuaResult<u32> {
    let position = stack_val(state, 1);
    for key in ["x", "y"] {
        let value = table_get(state, position, key);
        state.push(value);
    }
    Ok(2)
}

fn optional_string(state: &mut LuaState, value: Option<&str>) -> Val {
    value.map_or(Val::Nil, |value| create_string(state, value))
}

fn set_gossip_poi(state: &mut LuaState) -> LuaResult<u32> {
    let poi = GossipPoiInfo {
        poi_id: i32::from_stack(state, 2)?,
        name: String::from_stack(state, 3)?,
        position: (coordinate(state, 4)?, coordinate(state, 5)?),
        atlas_name: Option::<String>::from_stack(state, 6)?,
        texture_index: i32::from_stack(state, 7)?,
    };
    let ui_map_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?.gossip.pois.insert(ui_map_id, poi);
    Ok(0)
}

fn coordinate(state: &mut LuaState, index: usize) -> LuaResult<f64> {
    let value = f64::from_stack(state, index)?;
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(runtime_error(
            "A_Admin.SetGossipPoi: position must be between 0 and 1",
        ));
    }
    Ok(value)
}
