//! Retail `C_Item.DropItemOnUnit` trade-slot branch (WM34-B, WM68).
//!
//! Staging a new trade, the special trade slot and the give branch have
//! UNKNOWN conditions; only a drop on the current trade target is modeled.

use super::cursor_slot_compatibility::cursor_owned_item;
use crate::lua_api::globals::real::cursor_model;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::SimState;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Trade slots 1–6 are ordinary; slot 7 is the separate special slot.
const ORDINARY_TRADE_SLOTS: usize = 6;

pub(super) fn drop_item_on_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let placed = place_in_trade_slot(&mut *borrow_state_mut(state)?, &unit);
    let Some(slot) = placed else {
        return Ok(0);
    };
    dispatch_event_now(state, "TRADE_PLAYER_ITEM_CHANGED", &[Val::Num(slot as f64)])?;
    cursor_model::clear(state)?;
    Ok(0)
}

/// Store the owned cursor item in the first free ordinary slot (INFERRED order)
/// and return its one-based slot. A full trade retains the cursor.
fn place_in_trade_slot(sim: &mut SimState, unit: &str) -> Option<usize> {
    let (item_id, _) = cursor_owned_item(sim)?;
    let unit_guid = existing_guid_for_unit(sim, unit)?;
    let trade_target = sim.active_trade.as_ref()?.target.clone();
    if existing_guid_for_unit(sim, &trade_target)? != unit_guid {
        return None;
    }
    let slots = &mut sim.active_trade.as_mut()?.player_slots;
    let index = slots[..ORDINARY_TRADE_SLOTS]
        .iter()
        .position(|&slot_item| slot_item == 0)?;
    slots[index] = item_id;
    Some(index + 1)
}
