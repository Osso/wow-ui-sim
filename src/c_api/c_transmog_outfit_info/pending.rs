//! Viewed pending appearance overlay and atomic outfit application.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

use super::{ViewedOutfitSlotInfo, model::SlotKey};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, handler) in [
        (
            "SetPendingTransmog",
            set_pending as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("RevertPendingTransmog", revert_pending),
        ("ClearAllPendingTransmogs", clear_pending),
        ("HasPendingOutfitTransmogs", has_pending),
        ("CommitAndApplyAllPending", commit_and_apply),
    ] {
        table_set_rust_fn_static(state, namespace, name, handler)?;
    }
    Ok(())
}

pub(super) fn read_slot_key(state: &LuaState) -> LuaResult<SlotKey> {
    let key = (
        i32::from_stack(state, 1)?,
        i32::from_stack(state, 2)?,
        i32::from_stack(state, 3)?,
    );
    if !(0..=14).contains(&key.0) || !(0..=1).contains(&key.1) || !(0..=11).contains(&key.2) {
        return Err(rilua::runtime_error("invalid transmog slot/type/option"));
    }
    Ok(key)
}

fn set_pending(state: &mut LuaState) -> LuaResult<u32> {
    let key = read_slot_key(state)?;
    let id = i64::from_stack(state, 4)?;
    let display = i32::from_stack(state, 5)?;
    if id < 0 || !(0..=4).contains(&display) {
        return Err(rilua::runtime_error("invalid transmog ID/display type"));
    }
    let record = build_pending_record(state, id, display)?;
    borrow_state_mut(state)?
        .transmog_outfits
        .pending_slots
        .insert(key, record);
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SLOT_REFRESH", &[])?;
    Ok(0)
}

fn build_pending_record(
    state: &LuaState,
    id: i64,
    display: i32,
) -> LuaResult<ViewedOutfitSlotInfo> {
    let sim = borrow_state(state)?;
    let source = sim.transmog_appearance_sources.get(&id);
    // INFERRED: non-assigned modes need no collected source. Illusions use
    // the illusion catalog rather than item modified appearance records.
    let illusion_collected = sim
        .transmog_illusions
        .iter()
        .any(|row| i64::from(row.source_id) == id && row.is_collected);
    let collected =
        display != 1 || source.is_some_and(|row| row.is_collected) || illusion_collected;
    Ok(ViewedOutfitSlotInfo {
        transmog_id: id,
        display_type: display,
        is_transmogrified: false,
        has_pending: true,
        is_pending_collected: collected,
        can_transmogrify: sim.transmog_enabled && collected,
        warning: 0,
        warning_text: String::new(),
        error: if collected { 0 } else { 8 },
        error_text: if collected {
            String::new()
        } else {
            "Appearance not collected".into()
        },
        texture: source.map(|row| row.icon),
        sheathe_category: 0,
    })
}

fn revert_pending(state: &mut LuaState) -> LuaResult<u32> {
    let key = read_slot_key(state)?;
    borrow_state_mut(state)?
        .transmog_outfits
        .pending_slots
        .remove(&key);
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SLOT_REFRESH", &[])?;
    Ok(0)
}

fn clear_pending(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .transmog_outfits
        .pending_slots
        .clear();
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SLOT_REFRESH", &[])?;
    Ok(0)
}

fn has_pending(state: &mut LuaState) -> LuaResult<u32> {
    let pending = !borrow_state(state)?
        .transmog_outfits
        .pending_slots
        .is_empty();
    state.push(Val::Bool(pending));
    Ok(1)
}

fn commit_and_apply(state: &mut LuaState) -> LuaResult<u32> {
    let _use_available_discount = bool::from_stack(state, 1)?;
    // INFERRED: no automatic discount without a pricing model input. Explicit
    // cost snapshots remain authoritative; otherwise use per-slot host price.
    let keys = commit_slots(state)?;
    super::situations::commit_situations(state)?;
    for (slot, kind, option) in keys {
        dispatch_event_now(
            state,
            "VIEWED_TRANSMOG_OUTFIT_SLOT_SAVE_SUCCESS",
            &[
                Val::Num(slot as f64),
                Val::Num(kind as f64),
                Val::Num(option as f64),
            ],
        )?;
    }
    dispatch_event_now(state, "TRANSMOG_DISPLAYED_OUTFIT_CHANGED", &[])?;
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SLOT_REFRESH", &[])?;
    Ok(0)
}

fn commit_slots(state: &LuaState) -> LuaResult<Vec<SlotKey>> {
    let mut sim = borrow_state_mut(state)?;
    let id = sim
        .viewed_transmog_outfit_id
        .ok_or_else(|| rilua::runtime_error("no viewed outfit"))?;
    if !sim.transmog_enabled
        || sim
            .transmog_outfits
            .pending_slots
            .values()
            .any(|row| !row.can_transmogrify)
    {
        return Err(rilua::runtime_error(
            "pending outfit contains unavailable transmogs",
        ));
    }
    let cost = sim.pending_transmog_cost.map(|row| row.cost).unwrap_or(
        sim.transmog_outfits.slot_cost * sim.transmog_outfits.pending_slots.len() as u64,
    );
    if cost > sim.player.money.max(0) as u64 {
        return Err(rilua::runtime_error("not enough money to apply outfit"));
    }
    sim.player.money -= cost as i64;
    let pending = std::mem::take(&mut sim.transmog_outfits.pending_slots);
    let keys = pending.keys().copied().collect();
    for (key, mut record) in pending {
        record.has_pending = false;
        record.is_transmogrified = record.display_type == 1;
        sim.viewed_outfit_slots.insert(key, record);
    }
    let slots = sim.viewed_outfit_slots.clone();
    sim.transmog_outfits.saved.entry(id).or_default().slots = slots;
    sim.active_transmog_outfit_id = Some(id);
    sim.pending_transmog_cost = None;
    Ok(keys)
}
