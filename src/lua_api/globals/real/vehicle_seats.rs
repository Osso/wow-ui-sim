//! Vehicle seat and seat-indicator model. Retail/Forever UnitDocumentation declare
//! byte-identical blocks; the original consumer is
//! Blizzard_UIPanels_Game/Shared/VehicleSeatIndicator.lua (SetupVehicle, Update, OnClick).
//! Host inputs: `A_Admin.SetVehicleUIIndicator` (indicator art/layout) and
//! `A_Admin.EnterVehicle` (seats, the player's seat and the UNIT_ENTERED_VEHICLE payload).
//! INFERRED: seat switches and ejections apply locally (server-response stand-ins)
//! and notify VEHICLE_PASSENGERS_CHANGED, the event the indicator refreshes on (R9-5).

mod inputs;

use crate::lua_api::SimState;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, val_to_string};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};
use std::collections::HashMap;

pub(crate) use inputs::{admin_enter_vehicle, admin_set_vehicle_ui_indicator};

#[derive(Debug, Clone)]
pub struct VehicleSeat {
    /// Native control type string: "None", "Root" or "Child".
    pub control_type: String,
    /// Another occupant's (name, server); the player's seat is `VehicleSeatState::player_seat`.
    pub occupant: Option<(String, String)>,
    pub ejectable: bool,
    pub can_switch: bool,
}

#[derive(Debug, Clone)]
pub struct VehicleIndicator {
    pub background: u32,
    /// (virtual seat index, x, y) per indicator seat, in display order.
    pub seats: Vec<(u32, f64, f64)>,
}

#[derive(Debug, Default)]
pub struct VehicleSeatState {
    pub indicators: HashMap<u32, VehicleIndicator>,
    pub seats: Vec<VehicleSeat>,
    /// 1-based virtual seat the player occupies while in a vehicle.
    pub player_seat: Option<usize>,
    pub hides_pet_frame: bool,
}

impl VehicleSeatState {
    /// Leaving clears seats and the player's seat; indicator layouts are host catalog data.
    pub(crate) fn leave_vehicle(&mut self) {
        self.seats.clear();
        self.player_seat = None;
        self.hides_pet_frame = false;
    }
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let functions: [(&str, rilua::vm::closure::RustFn); 11] = [
        ("UnitVehicleSeatCount", unit_vehicle_seat_count),
        ("UnitVehicleSeatInfo", unit_vehicle_seat_info),
        ("GetVehicleUIIndicator", get_vehicle_ui_indicator),
        ("GetVehicleUIIndicatorSeat", get_vehicle_ui_indicator_seat),
        ("CanSwitchVehicleSeat", can_switch_vehicle_seat),
        ("UnitSwitchToVehicleSeat", unit_switch_to_vehicle_seat),
        ("CanEjectPassengerFromSeat", can_eject_passenger_from_seat),
        ("EjectPassengerFromSeat", eject_passenger_from_seat),
        ("UnitUsingVehicle", unit_using_vehicle),
        ("UnitInVehicleControlSeat", unit_in_vehicle_control_seat),
        (
            "UnitInVehicleHidesPetFrame",
            unit_in_vehicle_hides_pet_frame,
        ),
    ];
    for (name, function) in functions {
        LuaApiMut::register_function(lua, name, function)?;
    }
    Ok(())
}

/// Seats exist only while the player is in a vehicle; other unit tokens have none.
fn player_seats(sim: &SimState) -> &[VehicleSeat] {
    if sim.player.in_vehicle {
        &sim.vehicle_seats.seats
    } else {
        &[]
    }
}

fn unit_is_player(state: &mut LuaState, index: i32) -> bool {
    val_to_string(state, stack_val(state, index)).as_deref() == Some("player")
}

/// Nilable unit arguments read the player (INFERRED, R9-5).
fn optional_unit_is_player(state: &mut LuaState) -> bool {
    matches!(stack_val(state, 1), Val::Nil) || unit_is_player(state, 1)
}

fn unit_vehicle_seat_count(state: &mut LuaState) -> LuaResult<u32> {
    let count = if unit_is_player(state, 1) {
        player_seats(&*borrow_state(state)?).len()
    } else {
        0
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn seat_index(state: &LuaState, index: i32) -> LuaResult<usize> {
    Ok(u32::from_stack(state, index)? as usize)
}

/// (controlType, occupant name, server, ejectable, canSwitch) of an existing seat.
fn seat_info(
    sim: &SimState,
    seat: usize,
) -> Option<(String, Option<(String, String)>, bool, bool)> {
    let info = player_seats(sim).get(seat.checked_sub(1)?)?;
    let occupant = if sim.vehicle_seats.player_seat == Some(seat) {
        Some((sim.player.name.clone(), String::new()))
    } else {
        info.occupant.clone()
    };
    Some((
        info.control_type.clone(),
        occupant,
        info.ejectable,
        info.can_switch,
    ))
}

fn unit_vehicle_seat_info(state: &mut LuaState) -> LuaResult<u32> {
    let seat = seat_index(state, 2)?;
    if !unit_is_player(state, 1) {
        return Ok(0);
    }
    let Some((control, occupant, ejectable, can_switch)) = seat_info(&*borrow_state(state)?, seat)
    else {
        return Ok(0);
    };
    let control = create_string(state, &control);
    state.push(control);
    for text in occupant.map_or([None, None], |(name, server)| [Some(name), Some(server)]) {
        let value = text.map_or(Val::Nil, |text| create_string(state, &text));
        state.push(value);
    }
    state.push(Val::Bool(ejectable));
    state.push(Val::Bool(can_switch));
    Ok(5)
}

fn get_vehicle_ui_indicator(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let Some((background, count)) = borrow_state(state)?
        .vehicle_seats
        .indicators
        .get(&id)
        .map(|indicator| (indicator.background, indicator.seats.len()))
    else {
        return Ok(0);
    };
    state.push(Val::Num(f64::from(background)));
    state.push(Val::Num(count as f64));
    Ok(2)
}

fn get_vehicle_ui_indicator_seat(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let index = seat_index(state, 2)?;
    let seat = borrow_state(state)?
        .vehicle_seats
        .indicators
        .get(&id)
        .and_then(|indicator| indicator.seats.get(index.checked_sub(1)?).copied());
    let Some((virtual_seat, x, y)) = seat else {
        return Ok(0);
    };
    state.push(Val::Num(f64::from(virtual_seat)));
    state.push(Val::Num(x));
    state.push(Val::Num(y));
    Ok(3)
}

/// A seat accepts the player when it exists, is empty, allows switching and is not theirs.
fn seat_accepts_player(sim: &SimState, seat: usize) -> bool {
    sim.vehicle_seats.player_seat != Some(seat)
        && seat_info(sim, seat).is_some_and(|(_, occupant, _, can)| occupant.is_none() && can)
}

/// INFERRED (R9-5): switching is possible while another seat accepts the player.
fn can_switch(sim: &SimState) -> bool {
    (1..=player_seats(sim).len()).any(|seat| seat_accepts_player(sim, seat))
}

fn can_switch_vehicle_seat(state: &mut LuaState) -> LuaResult<u32> {
    let can = can_switch(&*borrow_state(state)?);
    state.push(Val::Bool(can));
    Ok(1)
}

fn notify_passengers_changed(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "VEHICLE_PASSENGERS_CHANGED", &[])?;
    Ok(0)
}

fn unit_switch_to_vehicle_seat(state: &mut LuaState) -> LuaResult<u32> {
    let seat = seat_index(state, 2)?;
    if !unit_is_player(state, 1) || !seat_accepts_player(&*borrow_state(state)?, seat) {
        return Ok(0);
    }
    {
        let mut sim = borrow_state_mut(state)?;
        sim.vehicle_seats.player_seat = Some(seat);
        sim.player.controlling_vehicle = sim.vehicle_seats.seats[seat - 1].control_type == "Root";
    }
    notify_passengers_changed(state)
}

/// INFERRED (R9-5): only the controlling (Root) occupant ejects ejectable passengers.
fn can_eject(sim: &SimState, seat: usize) -> bool {
    let controls = sim
        .vehicle_seats
        .player_seat
        .and_then(|own| seat_info(sim, own))
        .is_some_and(|(control, ..)| control == "Root");
    let passenger = sim.vehicle_seats.player_seat != Some(seat)
        && seat_info(sim, seat)
            .is_some_and(|(_, occupant, ejectable, _)| occupant.is_some() && ejectable);
    controls && passenger
}

fn can_eject_passenger_from_seat(state: &mut LuaState) -> LuaResult<u32> {
    let seat = seat_index(state, 1)?;
    let can = can_eject(&*borrow_state(state)?, seat);
    state.push(Val::Bool(can));
    Ok(1)
}

fn eject_passenger_from_seat(state: &mut LuaState) -> LuaResult<u32> {
    let seat = seat_index(state, 1)?;
    if !can_eject(&*borrow_state(state)?, seat) {
        return Ok(0);
    }
    borrow_state_mut(state)?.vehicle_seats.seats[seat - 1].occupant = None;
    notify_passengers_changed(state)
}

fn push_player_vehicle_flag(
    state: &mut LuaState,
    is_player: bool,
    read: fn(&SimState) -> bool,
) -> LuaResult<u32> {
    let value = is_player && {
        let sim = borrow_state(state)?;
        sim.player.in_vehicle && read(&sim)
    };
    state.push(Val::Bool(value));
    Ok(1)
}

fn unit_using_vehicle(state: &mut LuaState) -> LuaResult<u32> {
    let is_player = unit_is_player(state, 1);
    push_player_vehicle_flag(state, is_player, |_| true)
}

fn unit_in_vehicle_control_seat(state: &mut LuaState) -> LuaResult<u32> {
    let is_player = optional_unit_is_player(state);
    push_player_vehicle_flag(state, is_player, |sim| sim.player.controlling_vehicle)
}

fn unit_in_vehicle_hides_pet_frame(state: &mut LuaState) -> LuaResult<u32> {
    let is_player = optional_unit_is_player(state);
    push_player_vehicle_flag(state, is_player, |sim| sim.vehicle_seats.hides_pet_frame)
}
