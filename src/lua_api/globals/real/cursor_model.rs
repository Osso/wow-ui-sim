//! Retail shared cursor clear and placement (WM46, WM44-A, WM44-B).
//!
//! Other profiles keep their historical behavior: the payload is assigned or
//! dropped without native modes, stored item IDs or grid events.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state::{CursorInfo, CursorItemOrigin};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) const MODE_ITEM: u8 = 1;
pub(crate) const MODE_MONEY: u8 = 2;
pub(crate) const MODE_SPELL: u8 = 3;
pub(crate) const MODE_PET_ACTION: u8 = 4;
pub(crate) const MODE_MERCHANT: u8 = 5;
pub(crate) const MODE_ACTION_BAR: u8 = 6;
pub(crate) const MODE_MACRO: u8 = 7;
pub(crate) const MODE_OUTFIT: u8 = 21;

/// Native cursor storage that outlives one payload.
#[derive(Debug, Default, Clone)]
pub struct CursorStorage {
    /// Shared item-ID storage S, reported as the previous stored ID.
    pub stored_item_id: u32,
    /// Grid flag: set by grid-showing pickups, cleared by the shared clear.
    pub grid_shown: bool,
}

/// Whether this profile uses the native cursor model.
pub(crate) fn modeled() -> bool {
    ACTIVE == ClientProfile::Retail
}

/// Native UICursorType of a payload. Talent cursors have none: native
/// selection between Flyout 14 and Spell 3 is not established.
pub(crate) fn native_mode(cursor: &CursorInfo) -> Option<u8> {
    match cursor {
        CursorInfo::Item {
            origin: CursorItemOrigin::Merchant { .. },
            ..
        } => Some(MODE_MERCHANT),
        CursorInfo::Item { .. } => Some(MODE_ITEM),
        CursorInfo::Money { .. } => Some(MODE_MONEY),
        CursorInfo::Spell { .. } | CursorInfo::Action { .. } => Some(MODE_SPELL),
        CursorInfo::PetAction { .. } => Some(MODE_PET_ACTION),
        CursorInfo::ActionBar { .. } => Some(MODE_ACTION_BAR),
        CursorInfo::Macro { .. } => Some(MODE_MACRO),
        CursorInfo::TransmogOutfit { .. } => Some(MODE_OUTFIT),
        CursorInfo::Talent { .. } => None,
    }
}

/// Item ID the payload's setter writes to S (merchant and ActionBar).
fn stored_item_id(cursor: &CursorInfo) -> Option<u32> {
    match cursor {
        CursorInfo::Item {
            item_id,
            origin: CursorItemOrigin::Merchant { .. },
            ..
        }
        | CursorInfo::ActionBar { item_id } => Some(*item_id),
        _ => None,
    }
}

/// Modes whose clear zeros S; every other mode keeps a stale value.
fn clear_zeroes_stored_id(mode: u8) -> bool {
    matches!(mode, 5 | 6 | 10 | 18 | 19 | 20)
}

/// Pickup paths that set the grid flag. Mode 1's resolution and eligibility
/// predicates are not modelled, so owned items never show the grid.
fn shows_grid(mode: u8) -> bool {
    matches!(mode, 3 | 4 | 6 | 7 | 12 | 13 | 14 | 16 | 17 | 21)
}

/// Current native mode; zero for an empty cursor.
pub(crate) fn current_mode(state: &LuaState) -> LuaResult<u8> {
    let sim = borrow_state(state)?;
    Ok(sim.cursor_item.as_ref().and_then(native_mode).unwrap_or(0))
}

/// Shared clear: HIDEGRID when flagged, then CURSOR_CHANGED(true, 0, oldMode,
/// S before the clear) for a nonempty cursor. Item-upgrade observations stay
/// with the caller, which may have swapped them into a slot.
pub(crate) fn clear(state: &mut LuaState) -> LuaResult<()> {
    let (old, saved_id, grid_shown) = {
        let mut sim = borrow_state_mut(state)?;
        let old = sim.cursor_item.take();
        if !modeled() {
            return Ok(());
        }
        let storage = &mut sim.cursor_storage;
        let saved_id = storage.stored_item_id;
        let old = old.map(|cursor| native_mode(&cursor));
        if old.flatten().is_some_and(clear_zeroes_stored_id) {
            storage.stored_item_id = 0;
        }
        (old, saved_id, std::mem::take(&mut storage.grid_shown))
    };
    if grid_shown {
        dispatch_event_now(state, "ACTIONBAR_HIDEGRID", &[])?;
    }
    match old {
        None => Ok(()),
        Some(Some(mode)) => {
            let payload = changed_payload(true, 0, mode, saved_id);
            dispatch_event_now(state, "CURSOR_CHANGED", &payload)
        }
        // Talent cursors keep the simulator's argument-less event.
        Some(None) => dispatch_event_now(state, "CURSOR_CHANGED", &[]),
    }
}

/// Shared clear, then the payload. Item setters report S after the clear;
/// typed setters report 0.
pub(crate) fn place(state: &mut LuaState, cursor: CursorInfo) -> LuaResult<()> {
    if !modeled() {
        borrow_state_mut(state)?.cursor_item = Some(cursor);
        return Ok(());
    }
    clear(state)?;
    let mode = native_mode(&cursor);
    let reported_id = {
        let mut sim = borrow_state_mut(state)?;
        let post_clear_id = sim.cursor_storage.stored_item_id;
        let written = stored_item_id(&cursor);
        if let Some(item_id) = written {
            sim.cursor_storage.stored_item_id = item_id;
        }
        sim.cursor_item = Some(cursor);
        written.map_or(0, |_| post_clear_id)
    };
    let Some(mode) = mode else {
        return Ok(());
    };
    if shows_grid(mode) {
        dispatch_event_now(state, "ACTIONBAR_SHOWGRID", &[])?;
        borrow_state_mut(state)?.cursor_storage.grid_shown = true;
    }
    let payload = changed_payload(false, mode, 0, reported_id);
    dispatch_event_now(state, "CURSOR_CHANGED", &payload)
}

/// The simulator's argument-less CURSOR_CHANGED, kept for unmodelled profiles.
pub(crate) fn legacy_changed(state: &mut LuaState) {
    if !modeled() {
        fire_named_event_state(state, "CURSOR_CHANGED", &[]);
    }
}

fn changed_payload(empty: bool, mode: u8, previous_mode: u8, stored_id: u32) -> [Val; 4] {
    [
        Val::Bool(empty),
        Val::Num(f64::from(mode)),
        Val::Num(f64::from(previous_mode)),
        Val::Num(f64::from(stored_id)),
    ]
}
