//! Supplied player-owned HouseInfo records and queued list updates.

use super::current_house::{HouseInfo, encode_house_info};
use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_table, table_get, table_set_num,
};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let housing = ensure_namespace(state, "C_Housing")?;
    table_set_rust_fn_static(state, housing, "GetPlayerOwnedHouses", request_owned_houses)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetPlayerOwnedHouses", set_player_owned_houses)
}

fn request_owned_houses(state: &mut LuaState) -> LuaResult<u32> {
    let houses = borrow_state(state)?.housing.owned_houses.clone();
    borrow_state_mut(state)?
        .pending_owned_house_replies
        .push_back(houses);
    Ok(0)
}

fn set_player_owned_houses(state: &mut LuaState) -> LuaResult<u32> {
    let rows = stack_val(state, 1);
    let houses = supplied_houses(state, rows)?;
    borrow_state_mut(state)?.housing.owned_houses = houses;
    Ok(0)
}

fn supplied_houses(state: &mut LuaState, rows: Val) -> LuaResult<Vec<HouseInfo>> {
    let Val::Table(table) = rows else {
        return Err(runtime_error(
            "A_Admin.SetPlayerOwnedHouses: expected an array",
        ));
    };
    let supplied = state
        .gc
        .tables
        .get(table)
        .ok_or_else(|| runtime_error("A_Admin.SetPlayerOwnedHouses: invalid array"))?;
    let len = supplied.len(&state.gc.string_arena);
    let entries = (1..=len)
        .map(|index| supplied.get_int(index as i64))
        .collect::<Vec<_>>();
    entries
        .into_iter()
        .map(|entry| decode_house_info(state, entry))
        .collect()
}

fn decode_house_info(state: &mut LuaState, row: Val) -> LuaResult<HouseInfo> {
    if !matches!(row, Val::Table(_)) {
        return Err(runtime_error(
            "A_Admin.SetPlayerOwnedHouses: each entry must be a table",
        ));
    }
    Ok(HouseInfo {
        plot_id: field(state, row, "plotID")?,
        house_name: field(state, row, "houseName")?,
        owner_name: field(state, row, "ownerName")?,
        plot_cost: field(state, row, "plotCost")?,
        neighborhood_name: field(state, row, "neighborhoodName")?,
        move_out_time: field(state, row, "moveOutTime")?,
        plot_reserved: field(state, row, "plotReserved")?,
        neighborhood_guid: field(state, row, "neighborhoodGUID")?,
        house_guid: field(state, row, "houseGUID")?,
    })
}

fn field<T: FromStack>(state: &mut LuaState, row: Val, name: &str) -> LuaResult<T> {
    let value = table_get(state, row, name);
    let top = state.top;
    state.push(value);
    let result = T::from_stack(state, -1);
    state.top = top;
    result
}

pub(crate) fn encode_house_list(state: &mut LuaState, houses: Vec<HouseInfo>) -> Val {
    let Val::Table(table) = create_table(state) else {
        unreachable!("create_table returns a table");
    };
    for (index, house) in houses.into_iter().enumerate() {
        let value = encode_house_info(state, house);
        table_set_num(state, table, (index + 1) as f64, value);
    }
    Val::Table(table)
}
