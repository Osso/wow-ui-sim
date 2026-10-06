//! Current location snapshots; ownership and tracked-house selection remain distinct.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug, Default)]
pub struct LocationState {
    pub inside_plot: bool,
    pub on_neighborhood_map: bool,
    pub current_neighborhood_guid: Option<String>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let housing = ensure_namespace(state, "C_Housing")?;
    for (name, function) in [
        ("IsInsidePlot", inside_plot as rilua::RustFn),
        ("IsOnNeighborhoodMap", on_map),
        ("GetCurrentNeighborhoodGUID", neighborhood),
        ("SetTrackedHouseGuid", track_house),
    ] {
        table_set_rust_fn_static(state, housing, name, function)?;
    }
    let decor = ensure_namespace(state, "C_HousingDecor")?;
    table_set_rust_fn_static(state, decor, "IsHouseExteriorDoorHovered", door_hovered)?;
    let layout = ensure_namespace(state, "C_HousingLayout")?;
    table_set_rust_fn_static(state, layout, "GetNumActiveRooms", room_count)
}

fn inside_plot(state: &mut LuaState) -> LuaResult<u32> {
    let inside = {
        let sim = borrow_state(state)?;
        // INFERRED subset relation: an owned plot is also a plot.
        sim.housing.location.inside_plot || sim.housing.inside_owned_plot
    };
    state.push(Val::Bool(inside));
    Ok(1)
}

fn on_map(state: &mut LuaState) -> LuaResult<u32> {
    let on_map = borrow_state(state)?.housing.location.on_neighborhood_map;
    state.push(Val::Bool(on_map));
    Ok(1)
}

fn neighborhood(state: &mut LuaState) -> LuaResult<u32> {
    let guid = borrow_state(state)?
        .housing
        .location
        .current_neighborhood_guid
        .clone();
    let value = guid.map_or(Val::Nil, |guid| create_string(state, &guid));
    state.push(value);
    Ok(1)
}

fn track_house(state: &mut LuaState) -> LuaResult<u32> {
    let guid = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?.housing.tracked_house_guid = guid;
    Ok(0)
}

fn door_hovered(state: &mut LuaState) -> LuaResult<u32> {
    let hovered = borrow_state(state)?.housing.exterior.entry_door_hovered;
    state.push(Val::Bool(hovered));
    Ok(1)
}

fn room_count(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED: existing room/floor records represent active rooms, not templates.
    let count = borrow_state(state)?.housing.base_room_floors.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}
