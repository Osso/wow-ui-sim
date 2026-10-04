//! Catalog-backed viewed outfit selection, separate from applied/pending state.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCurrentlyViewedOutfitID",
        get_currently_viewed_outfit_id,
    )?;
    table_set_rust_fn_static(state, namespace, "ChangeViewedOutfit", change_viewed_outfit)
}

fn get_currently_viewed_outfit_id(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED: numeric zero represents initial absence, like the active query.
    let id = borrow_state(state)?.viewed_transmog_outfit_id.unwrap_or(0);
    state.push(Val::Num(id as f64));
    Ok(1)
}

fn change_viewed_outfit(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(id) = value else {
        return Err(rilua::runtime_error(
            "ChangeViewedOutfit requires a numeric outfitID",
        ));
    };
    if select_viewed_outfit(state, id)? {
        // INFERRED association: each valid request refreshes the viewed outfit,
        // including repeat selection when the vendor first opens the frame.
        // Declaration specifies synchronous delivery and no payload.
        dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_CHANGED", &[])?;
    }
    Ok(0)
}

fn select_viewed_outfit(state: &mut LuaState, requested_id: f64) -> LuaResult<bool> {
    let mut sim = borrow_state_mut(state)?;
    let id = sim
        .transmog_outfit_catalog
        .entries
        .iter()
        .find(|entry| entry.outfit_id as f64 == requested_id)
        .map(|entry| entry.outfit_id);
    // INFERRED: catalog misses preserve state and emit no refresh. Like catalog
    // lookup, first matching ID wins; isDisabled is not an eligibility policy.
    let Some(id) = id else { return Ok(false) };
    // INFERRED: changing viewed selection does not discard pending snapshots.
    sim.viewed_transmog_outfit_id = Some(id);
    Ok(true)
}
