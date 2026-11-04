//! Retail shared cursor clear and placement (WM46, WM54, WM44-A, WM44-B).
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
pub(crate) const MODE_EQUIPMENT_SET: u8 = 12;
pub(crate) const MODE_CURRENCY: u8 = 13;
pub(crate) const MODE_OUTFIT: u8 = 21;

/// Native cursor storage that outlives one payload.
#[derive(Debug, Clone)]
pub struct CursorStorage {
    /// Base and displayed cursor selectors; these are separate from held content.
    pub presentation_base: u8,
    pub presentation_display: u8,
    /// WM86's two native gates have unknown meaning; supplied independently.
    pub presentation_gate_a: bool,
    pub presentation_gate_b: bool,
    /// Supplied refresh and notification controls; native defaults are unknown.
    pub presentation_refresh: bool,
    pub presentation_notifications: bool,
    /// Shared item-ID storage S, reported as the previous stored ID.
    pub stored_item_id: u32,
    /// Grid flag: set by grid-showing pickups, cleared by the shared clear.
    pub grid_shown: bool,
}

impl Default for CursorStorage {
    fn default() -> Self {
        Self {
            presentation_base: 1,
            presentation_display: 1,
            presentation_gate_a: false,
            presentation_gate_b: false,
            presentation_refresh: false,
            presentation_notifications: false,
            stored_item_id: 0,
            grid_shown: false,
        }
    }
}

/// Whether this profile uses the native cursor model.
pub(crate) fn modeled() -> bool {
    ACTIVE == ClientProfile::Retail
}

/// Native UICursorType of a payload. Talent cursors have none: WM54's Flyout 14
/// or Spell 3 selection needs talent records, spell relationships and flyout
/// records that the simulator lacks.
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
        CursorInfo::EquipmentSet { .. } => Some(MODE_EQUIPMENT_SET),
        CursorInfo::Currency { .. } => Some(MODE_CURRENCY),
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

/// Whether a successful pickup sets the grid flag. Owned items (mode 1) use
/// the WM54 Q4 predicate on their item ID.
fn shows_grid(cursor: &CursorInfo, mode: u8) -> bool {
    match cursor {
        CursorInfo::Item { item_id, .. } if mode == MODE_ITEM => {
            crate::c_api::item_spell::owned_item_shows_grid(*item_id)
        }
        _ => matches!(mode, 3 | 4 | 6 | 7 | 12 | 13 | 14 | 16 | 17 | 21),
    }
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
    clear_with_restoration(state, true)
}

/// An equip request keeps reason 1 when it clears the staged cursor (WM57).
pub(crate) fn clear_retaining_equip_lock(state: &mut LuaState) -> LuaResult<()> {
    clear_with_restoration(state, false)
}

fn clear_with_restoration(state: &mut LuaState, restore: bool) -> LuaResult<()> {
    let (old, saved_id, grid_shown, unlocked) = {
        let mut sim = borrow_state_mut(state)?;
        let old = sim.cursor_item.take();
        if !modeled() {
            return Ok(());
        }
        let unlocked = if restore {
            crate::c_api::item_spell::item_equip::restore_staged_lock(&mut sim)
        } else {
            sim.item_equip.staged = None;
            None
        };
        let storage = &mut sim.cursor_storage;
        let saved_id = storage.stored_item_id;
        let old = old.map(|cursor| native_mode(&cursor));
        if old.flatten().is_some_and(clear_zeroes_stored_id) {
            storage.stored_item_id = 0;
        }
        (
            old,
            saved_id,
            std::mem::take(&mut storage.grid_shown),
            unlocked,
        )
    };
    if let Some(slot) = unlocked {
        crate::c_api::item_spell::item_equip::dispatch_lock_transition(state, slot, false)?;
    }
    if grid_shown {
        dispatch_event_now(state, "ACTIONBAR_HIDEGRID", &[])?;
    }
    match old {
        None => Ok(()),
        Some(Some(mode)) => notify(state, mode, saved_id),
        // Talent cursors keep the simulator's argument-less event.
        Some(None) => dispatch_event_now(state, "CURSOR_CHANGED", &[]),
    }
}

/// Every setter runs the shared clear, even from an empty cursor (WM54 Q2),
/// then installs its payload. The set-side event reports a previous mode and
/// a stored ID that depend on the setter (WM46, WM54 Q1):
/// - typed setters: the mode saved before the clear, and 0;
/// - owned item (1): the mode saved after the clear, and 0;
/// - item-ID setters (5, 6): the mode and S saved after the clear.
///
/// It fires only when that previous mode differs from the new one, so a
/// same-mode typed replacement reports only the clear.
pub(crate) fn place(state: &mut LuaState, cursor: CursorInfo) -> LuaResult<()> {
    place_with_staged(state, cursor, None)
}

/// Install an owned item after the caller cleared and re-resolved its identity.
pub(crate) fn place_staged_after_clear(
    state: &mut LuaState,
    cursor: CursorInfo,
    staged: crate::c_api::item_spell::item_equip::StagedEquip,
) -> LuaResult<()> {
    let post_clear_mode = current_mode(state)?;
    install_cursor(state, cursor, Some(staged), post_clear_mode)
}

fn place_with_staged(
    state: &mut LuaState,
    cursor: CursorInfo,
    staged: Option<crate::c_api::item_spell::item_equip::StagedEquip>,
) -> LuaResult<()> {
    if !modeled() {
        borrow_state_mut(state)?.cursor_item = Some(cursor);
        return Ok(());
    }
    let pre_clear_mode = current_mode(state)?;
    clear(state)?;
    install_cursor(state, cursor, staged, pre_clear_mode)
}

fn install_cursor(
    state: &mut LuaState,
    cursor: CursorInfo,
    staged: Option<crate::c_api::item_spell::item_equip::StagedEquip>,
    pre_clear_mode: u8,
) -> LuaResult<()> {
    let post_clear_mode = current_mode(state)?;
    let mode = native_mode(&cursor);
    let grid = mode.is_some_and(|mode| shows_grid(&cursor, mode));
    let post_clear_id = {
        let mut sim = borrow_state_mut(state)?;
        let post_clear_id = sim.cursor_storage.stored_item_id;
        if let Some(item_id) = stored_item_id(&cursor) {
            sim.cursor_storage.stored_item_id = item_id;
        }
        sim.cursor_item = Some(cursor);
        sim.item_equip.staged = staged;
        post_clear_id
    };
    let Some(mode) = mode else {
        return Ok(());
    };
    if grid {
        dispatch_event_now(state, "ACTIONBAR_SHOWGRID", &[])?;
        borrow_state_mut(state)?.cursor_storage.grid_shown = true;
    }
    let (previous, reported_id) = match mode {
        MODE_ITEM => (post_clear_mode, 0),
        MODE_MERCHANT | MODE_ACTION_BAR => (post_clear_mode, post_clear_id),
        _ => (pre_clear_mode, 0),
    };
    if previous == mode {
        return Ok(());
    }
    notify(state, previous, reported_id)
}

/// CURSOR_CHANGED with its first two arguments read from the cursor at
/// notification time, so callback re-entry is reflected (WM54 Q1).
fn notify(state: &mut LuaState, previous_mode: u8, stored_id: u32) -> LuaResult<()> {
    let (empty, mode) = {
        let sim = borrow_state(state)?;
        let cursor = sim.cursor_item.as_ref();
        (cursor.is_none(), cursor.and_then(native_mode).unwrap_or(0))
    };
    let payload = changed_payload(empty, mode, previous_mode, stored_id);
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
