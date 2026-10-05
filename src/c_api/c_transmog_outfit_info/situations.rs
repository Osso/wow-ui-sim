//! Per-viewed-outfit situation selection and pending overlay.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

use super::model::SituationKey;

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, handler) in [
        (
            "UpdatePendingSituation",
            update_pending as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("GetOutfitSituation", get_situation),
        ("HasPendingOutfitSituations", has_pending),
        ("ClearAllPendingSituations", clear_pending),
        ("CommitPendingSituations", commit_pending),
        ("ResetOutfitSituations", reset_situations),
    ] {
        table_set_rust_fn_static(state, namespace, name, handler)?;
    }
    Ok(())
}

fn read_situation(state: &mut LuaState) -> LuaResult<SituationKey> {
    let option = stack_val(state, 1);
    if !matches!(option, Val::Table(_)) {
        return Err(rilua::runtime_error("situation option requires a table"));
    }
    let mut values = [0; 4];
    for (index, key) in ["situationID", "specID", "loadoutID", "equipmentSetID"]
        .iter()
        .enumerate()
    {
        let Val::Num(number) = table_get(state, option, key) else {
            return Err(rilua::runtime_error(format!(
                "situation option.{key} requires a number"
            )));
        };
        values[index] = number as i32;
    }
    Ok((values[0], values[1], values[2], values[3]))
}

fn update_pending(state: &mut LuaState) -> LuaResult<u32> {
    let key = read_situation(state)?;
    let value = bool::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .transmog_outfits
        .pending_situations
        .insert(key, value);
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SITUATIONS_CHANGED", &[])?;
    Ok(0)
}

fn get_situation(state: &mut LuaState) -> LuaResult<u32> {
    let key = read_situation(state)?;
    let value = {
        let sim = borrow_state(state)?;
        let saved = sim
            .viewed_transmog_outfit_id
            .and_then(|id| sim.transmog_outfits.saved.get(&id));
        sim.transmog_outfits
            .pending_situations
            .get(&key)
            .copied()
            .or_else(|| saved.and_then(|outfit| outfit.situations.get(&key).copied()))
            .unwrap_or(false)
    };
    state.push(Val::Bool(value));
    Ok(1)
}

fn has_pending(state: &mut LuaState) -> LuaResult<u32> {
    let pending = !borrow_state(state)?
        .transmog_outfits
        .pending_situations
        .is_empty();
    state.push(Val::Bool(pending));
    Ok(1)
}

fn clear_pending(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .transmog_outfits
        .pending_situations
        .clear();
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SITUATIONS_CHANGED", &[])?;
    Ok(0)
}

pub(super) fn commit_situations(state: &LuaState) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let id = sim
        .viewed_transmog_outfit_id
        .ok_or_else(|| rilua::runtime_error("no viewed outfit"))?;
    let pending = std::mem::take(&mut sim.transmog_outfits.pending_situations);
    sim.transmog_outfits
        .saved
        .entry(id)
        .or_default()
        .situations
        .extend(pending);
    Ok(())
}

fn commit_pending(state: &mut LuaState) -> LuaResult<u32> {
    commit_situations(state)?;
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SITUATIONS_CHANGED", &[])?;
    dispatch_event_now(state, "TRANSMOG_OUTFITS_CHANGED", &[])?;
    Ok(0)
}

fn reset_situations(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        let id = sim
            .viewed_transmog_outfit_id
            .ok_or_else(|| rilua::runtime_error("no viewed outfit"))?;
        // INFERRED: reset immediately clears saved and pending selections.
        sim.transmog_outfits
            .saved
            .entry(id)
            .or_default()
            .situations
            .clear();
        sim.transmog_outfits.pending_situations.clear();
    }
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SITUATIONS_CHANGED", &[])?;
    Ok(0)
}
