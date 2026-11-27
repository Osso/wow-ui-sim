//! `C_Cursor` namespace — cursor-state queries used by drag/drop UIs.
//!
//! Currently exposes only `GetCursorItem`, the call surfaced by
//! `Blizzard_AzeriteRespecUI`'s slot OnClick/OnReceiveDrag handlers
//! (`Blizzard_AzeriteRespecUI.lua:153,163`). Returns an `ItemLocation`-shaped
//! table — `{bagID, slotIndex}` for items picked up from a bag, or
//! `{equipmentSlotIndex}` for items picked up from a worn slot — with
//! `ItemLocationMixin` methods (`IsBagAndSlot`, `IsEquipmentSlot`,
//! `GetBagAndSlot`, `GetEquipmentSlot`, `IsEqualTo`, …) merged in when the
//! mixin global is loaded. Returns nil when the cursor is empty, holds a
//! non-item payload (spell/talent/macro/etc), or when the item came from a
//! non-locatable origin (merchant pickup, unknown source) — matching
//! Blizzard's contract that nil means "no `ItemLocation` to act on".
//!
//! The mixin attach is intentionally lazy: at registration time
//! `ItemLocationMixin` may not yet be loaded (Blizzard_ObjectAPI hasn't run),
//! but by the time a user interacts with a slot the addon has loaded it.

use crate::c_api::helpers::{ensure_namespace, global_val};
use crate::lua_api::methods::{borrow_state, create_table_with_fields};
use crate::lua_api::state::{CursorInfo, CursorItemOrigin};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_cursor_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_Cursor")?;
    table_set_rust_fn_static(state, ns, "GetCursorItem", get_cursor_item)?;
    Ok(())
}

fn get_cursor_item(state: &mut LuaState) -> LuaResult<u32> {
    let Some(origin) = locatable_cursor_origin(state)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let location = item_location_val(state, origin);
    state.push(location);
    Ok(1)
}

/// An `ItemLocation`-shaped table for a bag or equipment origin, with the
/// `ItemLocationMixin` methods merged in when that mixin is loaded.
pub(crate) fn item_location_val(state: &mut LuaState, origin: CursorItemOrigin) -> Val {
    let location = build_location_table(state, origin);
    attach_item_location_mixin(state, location);
    location
}

/// Reads the cursor's current item origin and returns it only when it
/// resolves to an `ItemLocation` (bag-slot or equipment-slot). Merchant and
/// Unknown origins return `None` so callers see nil — those origins don't
/// have a stable `ItemLocation`-style identity.
fn locatable_cursor_origin(state: &mut LuaState) -> LuaResult<Option<CursorItemOrigin>> {
    let sim = borrow_state(state)?;
    let origin = match sim.cursor_item.as_ref() {
        Some(CursorInfo::Item { origin, .. }) => *origin,
        _ => return Ok(None),
    };
    Ok(match origin {
        CursorItemOrigin::Bag { .. } | CursorItemOrigin::Equipped { .. } => Some(origin),
        CursorItemOrigin::Merchant { .. } | CursorItemOrigin::Unknown => None,
    })
}

fn build_location_table(state: &mut LuaState, origin: CursorItemOrigin) -> Val {
    match origin {
        CursorItemOrigin::Bag { bag, slot } => create_table_with_fields(
            state,
            &[
                ("bagID", Val::Num(bag as f64)),
                ("slotIndex", Val::Num(slot as f64)),
            ],
        ),
        CursorItemOrigin::Equipped { slot } => {
            create_table_with_fields(state, &[("equipmentSlotIndex", Val::Num(slot as f64))])
        }
        CursorItemOrigin::Merchant { .. } | CursorItemOrigin::Unknown => {
            unreachable!("filtered by locatable_cursor_origin")
        }
    }
}

/// Copy `ItemLocationMixin`'s methods (`IsBagAndSlot`, `IsEquipmentSlot`,
/// `GetBagAndSlot`, `GetEquipmentSlot`, `IsEqualTo`, etc.) onto the location
/// table so the addon can call `loc:IsEquipmentSlot()` directly. Mirrors
/// what the Lua `Mixin(t, ItemLocationMixin)` helper does. Silently skips
/// when the mixin global hasn't been loaded yet — the data fields still
/// satisfy direct field reads.
fn attach_item_location_mixin(state: &mut LuaState, location: Val) {
    let Val::Table(target_ref) = location else {
        return;
    };
    let Val::Table(mixin_ref) = global_val(state, "ItemLocationMixin") else {
        return;
    };
    let entries = match state.gc.tables.get(mixin_ref) {
        Some(table) => table.hash_entries(),
        None => return,
    };
    for (key, value) in entries {
        if let Some(target) = state.gc.tables.get_mut(target_ref) {
            let _ = target.raw_set(key, value, &state.gc.string_arena);
        }
    }
    state.gc.barrier_back(target_ref);
}
