//! State-backed vehicle / possess / taxi globals consumed by
//! `Blizzard_ActionBar/Shared/VehicleLeaveButton.lua` and
//! `Blizzard_ActionBar/Shared/PossessActionBar.lua`.
//!
//! All flags live on `state.player`:
//!
//! - `UnitInVehicle(unit)`            → `player.in_vehicle`
//! - `UnitHasVehicleUI(unit)`         → `player.has_vehicle_ui`
//! - `UnitHasVehiclePlayerFrameUI(unit)` → `player.has_vehicle_ui`
//! - `UnitControllingVehicle(unit)`   → `player.controlling_vehicle`
//! - `UnitOnTaxi(unit)`               → `player.on_taxi`
//! - `UnitVehicleSkin(unit)`          → `player.vehicle_skin` (`""` when none)
//! - `CanExitVehicle()`               → `player.has_vehicle_ui || player.on_taxi`
//! - `VehicleExit()`                  → clears the three vehicle flags and
//!   fires `UNIT_EXITED_VEHICLE` with `"player"`.
//! - `TaxiRequestEarlyLanding()`      → latches
//!   `player.taxi_early_landing_requested = true`. Real WoW would send the
//!   landing request to the server; the simulator just records it so tests
//!   can assert the leave button click reached this branch.
//!
//! Unit tokens other than `"player"` always return `false` / nil — the
//! simulator does not model vehicles or taxi seats for party / target.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, val_to_string};
use crate::lua_api::script_helpers::fire_named_event_state;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::state::PossessActionSlot;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::FromStack;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

/// Retail behavior is VERIFIED by R88ea; the same projection is INFERRED for Forever.
/// Selectors 1 and 2 return the native slot shapes; other selectors return no values.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_possess_info(state: &mut LuaState) -> LuaResult<u32> {
    let selector = i32::from_stack(state, 1)?;
    let Some(index) = selector
        .checked_sub(1)
        .and_then(|value| usize::try_from(value).ok())
        .filter(|_| matches!(selector, 1 | 2))
    else {
        return Ok(0);
    };
    let possess_slot = {
        let sim = borrow_state(state)?;
        sim.action_bar_state.possess_slots[index].clone()
    };
    push_possess_info(state, selector, possess_slot)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn push_possess_info(
    state: &mut LuaState,
    selector: i32,
    slot: Option<PossessActionSlot>,
) -> LuaResult<u32> {
    match selector {
        1 => push_first_possess_slot(state, slot.as_ref()),
        2 => push_second_possess_slot(state, slot.as_ref()),
        _ => Ok(0),
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn push_first_possess_slot(
    state: &mut LuaState,
    slot: Option<&PossessActionSlot>,
) -> LuaResult<u32> {
    if let Some(slot) = slot {
        let texture = create_string(state, &slot.texture);
        state.push(texture);
        state.push(Val::Num(slot.spell_id as f64));
    } else {
        state.push(Val::Nil);
        state.push(Val::Nil);
    }
    state.push(Val::Bool(true));
    Ok(3)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn push_second_possess_slot(
    state: &mut LuaState,
    slot: Option<&PossessActionSlot>,
) -> LuaResult<u32> {
    state.push(Val::Num(136190.0));
    state.push(slot.map_or(Val::Nil, |slot| Val::Num(slot.spell_id as f64)));
    state.push(Val::Bool(slot.is_some_and(|slot| slot.enabled)));
    Ok(3)
}

fn unit_arg_is_player(state: &mut LuaState, index: i32) -> bool {
    val_to_string(state, stack_val(state, index)).as_deref() == Some("player")
}

fn push_player_flag<F>(state: &mut LuaState, read: F) -> LuaResult<u32>
where
    F: FnOnce(&crate::lua_api::SimState) -> bool,
{
    let value = if unit_arg_is_player(state, 1) {
        read(&*borrow_state(state)?)
    } else {
        false
    };
    state.push(Val::Bool(value));
    Ok(1)
}

/// `UnitHasVehicleUI(unit)` — true when the override action bar is currently
/// shown. Only `"player"` is modeled.
fn unit_has_vehicle_ui(state: &mut LuaState) -> LuaResult<u32> {
    push_player_flag(state, |sim| sim.player.has_vehicle_ui)
}

/// `UnitInVehicle(unit)` — true when the player is currently seated in a
/// vehicle. Only `"player"` is modeled.
fn unit_in_vehicle(state: &mut LuaState) -> LuaResult<u32> {
    push_player_flag(state, |sim| sim.player.in_vehicle)
}

/// `UnitHasVehiclePlayerFrameUI(unit)` — true when PlayerFrame should swap to
/// vehicle art. Only `"player"` is modeled.
fn unit_has_vehicle_player_frame_ui(state: &mut LuaState) -> LuaResult<u32> {
    push_player_flag(state, |sim| sim.player.has_vehicle_ui)
}

/// `UnitControllingVehicle(unit)` — true when the player is the controlling
/// occupant of the vehicle. Only `"player"` is modeled.
fn unit_controlling_vehicle(state: &mut LuaState) -> LuaResult<u32> {
    push_player_flag(state, |sim| sim.player.controlling_vehicle)
}

/// `UnitOnTaxi(unit)` — true when the player is on a taxi route. Only
/// `"player"` is modeled.
fn unit_on_taxi(state: &mut LuaState) -> LuaResult<u32> {
    push_player_flag(state, |sim| sim.player.on_taxi)
}

/// `UnitVehicleSkin(unit)` — skin/style identifier for the unit's current
/// vehicle UI. Returns the empty string when the unit is not in a skinned
/// vehicle. `ActionBarController_UpdateAll` uses the empty-string sentinel
/// to decide whether the skinned override bar should display.
fn unit_vehicle_skin(state: &mut LuaState) -> LuaResult<u32> {
    let skin = if unit_arg_is_player(state, 1) {
        borrow_state(state)?
            .player
            .vehicle_skin
            .clone()
            .unwrap_or_default()
    } else {
        String::new()
    };
    let value = create_string(state, &skin);
    state.push(value);
    Ok(1)
}

/// `CanExitVehicle()` — true when the leave button should be shown. The
/// real client gates this on additional vehicle-controller flags; the
/// simulator collapses it to "has vehicle UI or on taxi".
fn can_exit_vehicle(state: &mut LuaState) -> LuaResult<u32> {
    let can_exit = {
        let sim = borrow_state(state)?;
        sim.player.has_vehicle_ui || sim.player.on_taxi
    };
    state.push(Val::Bool(can_exit));
    Ok(1)
}

/// `VehicleExit()` — clears the three vehicle flags and fires
/// `UNIT_EXITED_VEHICLE` for `"player"`. No-op when the player is not in a
/// vehicle (matches Blizzard: clicking the leave button without a vehicle
/// is harmless).
fn vehicle_exit(state: &mut LuaState) -> LuaResult<u32> {
    let was_in_vehicle = {
        let mut sim = borrow_state_mut(state)?;
        let was = sim.player.in_vehicle || sim.player.has_vehicle_ui;
        sim.player.in_vehicle = false;
        sim.player.controlling_vehicle = false;
        sim.player.has_vehicle_ui = false;
        was
    };
    if was_in_vehicle {
        let player = create_string(state, "player");
        fire_named_event_state(state, "UNIT_EXITED_VEHICLE", &[player]);
    }
    Ok(0)
}

/// `TaxiRequestEarlyLanding()` — latches `taxi_early_landing_requested`.
/// Tests assert the flag flipped after the leave button's click handler.
fn taxi_request_early_landing(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.player.taxi_early_landing_requested = true;
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    LuaApiMut::register_function(lua, "GetPossessInfo", get_possess_info)?;
    LuaApiMut::register_function(lua, "UnitInVehicle", unit_in_vehicle)?;
    LuaApiMut::register_function(lua, "UnitHasVehicleUI", unit_has_vehicle_ui)?;
    LuaApiMut::register_function(
        lua,
        "UnitHasVehiclePlayerFrameUI",
        unit_has_vehicle_player_frame_ui,
    )?;
    LuaApiMut::register_function(lua, "UnitControllingVehicle", unit_controlling_vehicle)?;
    LuaApiMut::register_function(lua, "UnitOnTaxi", unit_on_taxi)?;
    LuaApiMut::register_function(lua, "UnitVehicleSkin", unit_vehicle_skin)?;
    LuaApiMut::register_function(lua, "CanExitVehicle", can_exit_vehicle)?;
    LuaApiMut::register_function(lua, "VehicleExit", vehicle_exit)?;
    LuaApiMut::register_function(lua, "TaxiRequestEarlyLanding", taxi_request_early_landing)?;
    Ok(())
}
