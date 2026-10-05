//! Set/custom-set contents stage into the same pending slot overlay as manual edits.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table, table_set_num};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, handler) in [
        (
            "SetOutfitToCustomSet",
            import_custom_set as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("SetOutfitToSet", import_set),
        ("GetSourceIDsForSlot", get_source_ids),
        ("GetSetSourcesForSlot", get_sources),
    ] {
        table_set_rust_fn_static(state, namespace, name, handler)?;
    }
    Ok(())
}

fn read_source_ids(state: &LuaState) -> LuaResult<Vec<i64>> {
    let set = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    Ok(borrow_state(state)?
        .transmog_sets
        .slot_sources
        .get(&(set, slot))
        .cloned()
        .unwrap_or_default())
}

fn get_source_ids(state: &mut LuaState) -> LuaResult<u32> {
    let ids = read_source_ids(state)?;
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, id) in ids.iter().enumerate() {
        table_set_num(state, table, (index + 1) as f64, Val::Num(*id as f64));
    }
    state.push(array);
    Ok(1)
}

fn get_sources(state: &mut LuaState) -> LuaResult<u32> {
    let ids = read_source_ids(state)?;
    let rows = {
        let sim = borrow_state(state)?;
        ids.iter()
            .filter_map(|id| sim.transmog_appearance_sources.get(id).cloned())
            .collect::<Vec<_>>()
    };
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, source) in rows.iter().enumerate() {
        let row = super::super::c_transmog_collection::appearance_sources::build_source_table(
            state, source,
        );
        table_set_num(state, table, (index + 1) as f64, row);
    }
    state.push(array);
    Ok(1)
}

fn option_for_source(state: &LuaState, id: i64, slot: i32) -> LuaResult<i32> {
    if slot < 12 {
        return Ok(0);
    }
    let category = borrow_state(state)?
        .transmog_appearance_sources
        .get(&id)
        .map(|row| row.category);
    Ok(match category {
        Some(18) => 5,
        Some(19) => 4,
        Some(20..=24) => 2,
        Some(25..=27) => 3,
        _ => 1,
    })
}

fn stage_source(state: &LuaState, slot: i32, kind: i32, id: i64) -> LuaResult<()> {
    if id == 0 {
        return Ok(());
    }
    let option = option_for_source(state, id, slot)?;
    let row = super::pending::build_pending_record(state, id, 1)?;
    borrow_state_mut(state)?
        .transmog_outfits
        .pending_slots
        .insert((slot, kind, option), row);
    Ok(())
}

fn import_custom_set(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let items = super::super::c_transmog_collection::read_custom_set_items(state, id)?;
    // TransmogUtil consumes this list by one-based inventory slot, not outfit
    // enum slot. Zero IDs leave that slot untouched (INFERRED merge policy).
    for (index, item) in items.iter().enumerate() {
        let Some(slot) = super::slots::outfit_slot_from_inventory(index as i32) else {
            continue;
        };
        stage_source(state, slot, 0, i64::from(item.appearance_id))?;
        if slot == 1 {
            stage_source(state, 2, 0, i64::from(item.secondary_appearance_id))?;
        }
        if matches!(slot, 12 | 13) {
            stage_source(state, slot, 1, i64::from(item.illusion_id))?;
        }
    }
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SLOT_REFRESH", &[])?;
    Ok(0)
}

fn import_set(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let sources = {
        let sim = borrow_state(state)?;
        // INFERRED: prefer the first collected alternative for each slot.
        sim.transmog_sets
            .slot_sources
            .iter()
            .filter(|((set, _), _)| *set == id)
            .filter_map(|((_, slot), ids)| {
                ids.iter()
                    .find(|source| {
                        sim.transmog_appearance_sources
                            .get(source)
                            .is_some_and(|row| row.is_collected)
                    })
                    .map(|source| (*slot, *source))
            })
            .collect::<Vec<_>>()
    };
    for (slot, source) in sources {
        stage_source(state, slot, 0, source)?;
    }
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SLOT_REFRESH", &[])?;
    Ok(0)
}
