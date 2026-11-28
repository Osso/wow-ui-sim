//! A_Admin vehicle inputs. EnterVehicle dispatches UNIT_ENTERED_VEHICLE with the
//! VehicleDocumentation payload order: unit, showVehicleFrame, isControlSeat,
//! vehicleUIIndicatorID, vehicleGUID, mayChooseExit, hasPitch.

use super::{VehicleIndicator, VehicleSeat};
use crate::lua_api::globals::real::admin_fields as fields;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const INDICATOR_API: &str = "A_Admin.SetVehicleUIIndicator";
const ENTER_API: &str = "A_Admin.EnterVehicle";
const CONTROL_TYPES: [&str; 3] = ["None", "Root", "Child"];

/// `A_Admin.SetVehicleUIIndicator(id, {background=fileID, seats={{seat=, x=, y=}}})`.
pub(crate) fn admin_set_vehicle_ui_indicator(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let table = info_table(state, INDICATOR_API)?;
    let background = fields::unsigned(state, table, INDICATOR_API, "background")?;
    let mut seats = Vec::new();
    for row in fields::table_array(state, table, INDICATOR_API, "seats")? {
        seats.push((
            fields::unsigned(state, row, INDICATOR_API, "seat")?,
            fields::finite(state, row, INDICATOR_API, "x")?,
            fields::finite(state, row, INDICATOR_API, "y")?,
        ));
    }
    let indicator = VehicleIndicator { background, seats };
    borrow_state_mut(state)?
        .vehicle_seats
        .indicators
        .insert(id, indicator);
    Ok(0)
}

fn info_table(state: &LuaState, api: &str) -> LuaResult<Val> {
    match stack_val(state, 2) {
        table @ Val::Table(_) => Ok(table),
        _ => Err(runtime_error(format!("{api} expects an info table"))),
    }
}

fn read_seat(state: &mut LuaState, row: Val) -> LuaResult<VehicleSeat> {
    let control_type = fields::string(state, row, ENTER_API, "controlType")?;
    if !CONTROL_TYPES.contains(&control_type.as_str()) {
        return Err(runtime_error(format!(
            "{ENTER_API} controlType must be None, Root or Child"
        )));
    }
    let occupant = fields::optional_string(state, row, ENTER_API, "occupant")?;
    let server = fields::optional_string(state, row, ENTER_API, "server")?;
    Ok(VehicleSeat {
        control_type,
        occupant: occupant.map(|name| (name, server.unwrap_or_default())),
        ejectable: fields::boolean(state, row, ENTER_API, "ejectable")?,
        can_switch: fields::boolean(state, row, ENTER_API, "canSwitch")?,
    })
}

struct Entry {
    seats: Vec<VehicleSeat>,
    player_seat: usize,
    indicator: u32,
    guid: String,
    show_frame: bool,
    may_choose_exit: bool,
    has_pitch: bool,
    hides_pet_frame: bool,
}

fn read_entry(state: &mut LuaState, table: Val) -> LuaResult<Entry> {
    let mut seats = Vec::new();
    for row in fields::table_array(state, table, ENTER_API, "seats")? {
        seats.push(read_seat(state, row)?);
    }
    let player_seat = fields::unsigned(state, table, ENTER_API, "playerSeat")? as usize;
    if !(1..=seats.len()).contains(&player_seat) || seats[player_seat - 1].occupant.is_some() {
        return Err(runtime_error(format!(
            "{ENTER_API} playerSeat must name an empty seat"
        )));
    }
    Ok(Entry {
        seats,
        player_seat,
        indicator: fields::unsigned(state, table, ENTER_API, "indicatorID")?,
        guid: fields::string(state, table, ENTER_API, "vehicleGUID")?,
        show_frame: fields::boolean(state, table, ENTER_API, "showVehicleFrame")?,
        may_choose_exit: fields::boolean(state, table, ENTER_API, "mayChooseExit")?,
        has_pitch: fields::boolean(state, table, ENTER_API, "hasPitch")?,
        hides_pet_frame: fields::boolean(state, table, ENTER_API, "hidesPetFrame")?,
    })
}

/// Seats the player, mirrors the vehicle flags and dispatches UNIT_ENTERED_VEHICLE.
pub(crate) fn admin_enter_vehicle(state: &mut LuaState) -> LuaResult<u32> {
    let table = match stack_val(state, 1) {
        table @ Val::Table(_) => table,
        _ => return Err(runtime_error(format!("{ENTER_API} expects an info table"))),
    };
    let entry = read_entry(state, table)?;
    let control = entry.seats[entry.player_seat - 1].control_type == "Root";
    {
        let mut sim = borrow_state_mut(state)?;
        sim.player.in_vehicle = true;
        sim.player.has_vehicle_ui = entry.show_frame;
        sim.player.controlling_vehicle = control;
        sim.vehicle_seats.seats = entry.seats;
        sim.vehicle_seats.player_seat = Some(entry.player_seat);
        sim.vehicle_seats.hides_pet_frame = entry.hides_pet_frame;
    }
    let unit = create_string(state, "player");
    let guid = create_string(state, &entry.guid);
    let payload = [
        unit,
        Val::Bool(entry.show_frame),
        Val::Bool(control),
        Val::Num(f64::from(entry.indicator)),
        guid,
        Val::Bool(entry.may_choose_exit),
        Val::Bool(entry.has_pitch),
    ];
    dispatch_event_now(state, "UNIT_ENTERED_VEHICLE", &payload)?;
    Ok(0)
}
