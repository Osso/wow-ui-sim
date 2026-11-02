//! `C_Item.PickupItem` places an item ID on the native ActionBar cursor
//! (mode 6), per WM44-A and its WM46 erratum.

use crate::c_api::item_upgrade_eligibility;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::CursorInfo;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const MODE_EMPTY: f64 = 0.0;
const MODE_OWNED_ITEM: f64 = 1.0;
const MODE_ACTION_BAR: f64 = 6.0;

/// Cursor removed by the clear step. Only modes 1 and 6 have a known native
/// mode number and stored ID; other cursor kinds await WM46.
enum ClearedCursor {
    Empty,
    Modeled {
        mode: f64,
        stored_id: u32,
        grid_shown: bool,
    },
    Unmodeled,
}

fn take_cursor(state: &mut LuaState) -> LuaResult<ClearedCursor> {
    let mut sim = borrow_state_mut(state)?;
    item_upgrade_eligibility::discard_cursor(&mut sim);
    Ok(match sim.cursor_item.take() {
        None => ClearedCursor::Empty,
        // Mode 6 always shows the grid and stores its item ID.
        Some(CursorInfo::ActionBar { item_id }) => ClearedCursor::Modeled {
            mode: MODE_ACTION_BAR,
            stored_id: item_id,
            grid_shown: true,
        },
        // No simulator path shows the grid or stores an ID for mode 1.
        Some(CursorInfo::Item { .. }) => ClearedCursor::Modeled {
            mode: MODE_OWNED_ITEM,
            stored_id: 0,
            grid_shown: false,
        },
        Some(_) => ClearedCursor::Unmodeled,
    })
}

fn dispatch_clear_events(state: &mut LuaState, cleared: ClearedCursor) -> LuaResult<()> {
    match cleared {
        ClearedCursor::Empty => Ok(()),
        ClearedCursor::Modeled {
            mode,
            stored_id,
            grid_shown,
        } => {
            if grid_shown {
                dispatch_event_now(state, "ACTIONBAR_HIDEGRID", &[])?;
            }
            let payload = [
                Val::Bool(true),
                Val::Num(MODE_EMPTY),
                Val::Num(mode),
                Val::Num(f64::from(stored_id)),
            ];
            dispatch_event_now(state, "CURSOR_CHANGED", &payload)
        }
        // Known gap until WM46: keep the simulator's argument-less event.
        ClearedCursor::Unmodeled => dispatch_event_now(state, "CURSOR_CHANGED", &[]),
    }
}

pub(super) fn pickup_item(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let item = super::parse_item_id_key(state, value)
        .ok_or_else(|| runtime_error("C_Item.PickupItem: expected itemInfo"))?;
    let cleared = take_cursor(state)?;
    dispatch_clear_events(state, cleared)?;
    if item == 0 {
        return Ok(0);
    }
    borrow_state_mut(state)?.cursor_item = Some(CursorInfo::ActionBar { item_id: item });
    dispatch_event_now(state, "ACTIONBAR_SHOWGRID", &[])?;
    // The fourth argument is the stored ID after clearing, which is zero in
    // every modeled flow: clearing mode 6 zeros it and nothing sets it otherwise.
    let payload = [
        Val::Bool(false),
        Val::Num(MODE_ACTION_BAR),
        Val::Num(MODE_EMPTY),
        Val::Num(0.0),
    ];
    dispatch_event_now(state, "CURSOR_CHANGED", &payload)?;
    Ok(0)
}
