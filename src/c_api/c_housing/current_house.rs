//! Supplied HouseInfo shape; native acquisition and nil-selection policy are unknown.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// HousingUISharedDocumentation.lua in the pinned Retail/PTR caches.
/// No default row is fabricated: plotID is required; every other field is nullable.
#[derive(Clone, Debug)]
pub struct HouseInfo {
    pub plot_id: f64,
    pub house_name: Option<String>,
    pub owner_name: Option<String>,
    pub plot_cost: Option<f64>,
    pub neighborhood_name: Option<String>,
    pub move_out_time: Option<i64>,
    pub plot_reserved: Option<bool>,
    pub neighborhood_guid: Option<String>,
    pub house_guid: Option<String>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let housing = ensure_namespace(state, "C_Housing")?;
    table_set_rust_fn_static(state, housing, "GetCurrentHouseInfo", current_house_info)
}

fn current_house_info(state: &mut LuaState) -> LuaResult<u32> {
    let info = borrow_state(state)?.housing.current_house_info.clone();
    let value = info.map_or(Val::Nil, |info| encode_house_info(state, info));
    state.push(value);
    Ok(1)
}

pub(crate) fn encode_house_info(state: &mut LuaState, info: HouseInfo) -> Val {
    let table = create_table(state);
    table_set(state, table, "plotID", Val::Num(info.plot_id));
    for (key, value) in [
        ("houseName", info.house_name),
        ("ownerName", info.owner_name),
        ("neighborhoodName", info.neighborhood_name),
        ("neighborhoodGUID", info.neighborhood_guid),
        ("houseGUID", info.house_guid),
    ] {
        if let Some(text) = value {
            let value = create_string(state, &text);
            table_set(state, table, key, value);
        }
    }
    if let Some(cost) = info.plot_cost {
        table_set(state, table, "plotCost", Val::Num(cost));
    }
    if let Some(time) = info.move_out_time {
        table_set(state, table, "moveOutTime", Val::Num(time as f64));
    }
    if let Some(reserved) = info.plot_reserved {
        table_set(state, table, "plotReserved", Val::Bool(reserved));
    }
    table
}
