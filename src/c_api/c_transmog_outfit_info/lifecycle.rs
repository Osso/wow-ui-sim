//! Catalog creation, metadata edits and displayed selection.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

use super::{OutfitEntry, model::valid_name};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, handler) in [
        (
            "AddNewOutfit",
            add_new_outfit as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("CommitOutfitInfo", commit_outfit_info),
        ("IsValidTransmogOutfitName", is_valid_name),
        ("ChangeDisplayedOutfit", change_displayed_outfit),
        ("ClearDisplayedOutfit", clear_displayed_outfit),
        ("IsEquippedGearOutfitDisplayed", is_equipped_displayed),
        ("GetMaxNumberOfTotalOutfitsForSource", max_for_source),
        ("GetNumberOfOutfitsUnlockedForSource", unlocked_for_source),
        ("GetMaxNumberOfUsableOutfits", max_usable),
        ("GetNextOutfitCost", next_cost),
    ] {
        table_set_rust_fn_static(state, namespace, name, handler)?;
    }
    Ok(())
}

fn read_metadata(state: &LuaState, index: i32) -> LuaResult<(String, i64)> {
    let name = String::from_stack(state, index)?;
    let icon = i64::from_stack(state, index + 1)?;
    if !valid_name(&name) || icon < 0 {
        return Err(rilua::runtime_error("invalid outfit name or icon"));
    }
    Ok((name, icon))
}

fn add_new_outfit(state: &mut LuaState) -> LuaResult<u32> {
    let (name, icon) = read_metadata(state, 1)?;
    let id = insert_outfit(state, name, icon)?;
    dispatch_event_now(state, "TRANSMOG_OUTFITS_CHANGED", &[Val::Num(id as f64)])?;
    Ok(0)
}

fn insert_outfit(state: &LuaState, name: String, icon: i64) -> LuaResult<i64> {
    let mut sim = borrow_state_mut(state)?;
    let capacity: u32 = sim.transmog_outfits.unlocked_by_source.iter().sum();
    if sim.transmog_outfit_catalog.entries.len() >= capacity as usize {
        return Err(rilua::runtime_error("no unlocked outfit slots available"));
    }
    let entries = &mut sim.transmog_outfit_catalog.entries;
    let id = entries
        .iter()
        .map(|entry| entry.outfit_id)
        .max()
        .unwrap_or(0)
        + 1;
    let index = entries
        .iter()
        .map(|entry| entry.player_facing_outfit_index)
        .max()
        .unwrap_or(0)
        + 1;
    entries.push(OutfitEntry {
        outfit_id: id,
        name,
        icon,
        situation_categories: vec![],
        is_event_outfit: false,
        is_disabled: false,
        player_facing_outfit_index: index,
    });
    sim.transmog_outfits.saved.insert(id, Default::default());
    Ok(id)
}

fn commit_outfit_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = i64::from_stack(state, 1)?;
    let (name, icon) = read_metadata(state, 2)?;
    {
        let mut sim = borrow_state_mut(state)?;
        let entry = sim
            .transmog_outfit_catalog
            .entries
            .iter_mut()
            .find(|entry| entry.outfit_id == id)
            .ok_or_else(|| rilua::runtime_error(format!("unknown outfit ID {id}")))?;
        entry.name = name;
        entry.icon = icon;
    }
    dispatch_event_now(state, "TRANSMOG_OUTFITS_CHANGED", &[])?;
    Ok(0)
}

fn is_valid_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    state.push(Val::Bool(valid_name(&name)));
    Ok(1)
}

fn change_displayed_outfit(state: &mut LuaState) -> LuaResult<u32> {
    let id = i64::from_stack(state, 1)?;
    let _trigger = i32::from_stack(state, 2)?;
    let toggle_lock = bool::from_stack(state, 3)?;
    let allow_remove = bool::from_stack(state, 4)?;
    {
        let mut sim = borrow_state_mut(state)?;
        if !sim
            .transmog_outfit_catalog
            .entries
            .iter()
            .any(|entry| entry.outfit_id == id)
        {
            return Err(rilua::runtime_error(format!("unknown outfit ID {id}")));
        }
        let remove = allow_remove && sim.active_transmog_outfit_id == Some(id);
        sim.active_transmog_outfit_id = if remove { None } else { Some(id) };
        // INFERRED: toggleLock flips the selected outfit lock, independent of
        // whether this request removes it. Trigger identifies caller only.
        if toggle_lock && !sim.transmog_outfit_locks.remove(&id) {
            sim.transmog_outfit_locks.insert(id);
        }
    }
    dispatch_event_now(state, "TRANSMOG_DISPLAYED_OUTFIT_CHANGED", &[])?;
    Ok(0)
}

fn clear_displayed_outfit(state: &mut LuaState) -> LuaResult<u32> {
    let _trigger = i32::from_stack(state, 1)?;
    let toggle_lock = bool::from_stack(state, 2)?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.active_transmog_outfit_id = None;
        if toggle_lock {
            sim.equipped_outfit_locked = !sim.equipped_outfit_locked;
        }
    }
    dispatch_event_now(state, "TRANSMOG_DISPLAYED_OUTFIT_CHANGED", &[])?;
    Ok(0)
}

fn is_equipped_displayed(state: &mut LuaState) -> LuaResult<u32> {
    let equipped = borrow_state(state)?.active_transmog_outfit_id.is_none();
    state.push(Val::Bool(equipped));
    Ok(1)
}

fn read_source(state: &LuaState) -> LuaResult<usize> {
    let source = i32::from_stack(state, 1)?;
    if !(0..=2).contains(&source) {
        return Err(rilua::runtime_error("invalid outfit source"));
    }
    Ok(source as usize)
}

fn max_for_source(state: &mut LuaState) -> LuaResult<u32> {
    let source = read_source(state)?;
    let count = borrow_state(state)?.transmog_outfits.max_outfits_by_source[source];
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn unlocked_for_source(state: &mut LuaState) -> LuaResult<u32> {
    let source = read_source(state)?;
    let count = borrow_state(state)?.transmog_outfits.unlocked_by_source[source];
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn max_usable(state: &mut LuaState) -> LuaResult<u32> {
    let count: u32 = borrow_state(state)?
        .transmog_outfits
        .unlocked_by_source
        .iter()
        .sum();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn next_cost(state: &mut LuaState) -> LuaResult<u32> {
    let cost = borrow_state(state)?.transmog_outfits.next_outfit_cost;
    state.push(Val::Num(cost as f64));
    Ok(1)
}
