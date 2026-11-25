//! Host inputs for `GetInventoryItemsForSlot` candidate publication.

use crate::lua_api::globals::real::inventory_candidates::{
    CandidateLocation, InventoryCandidate, InventoryCandidates,
};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::table_security::{TableAccess, check_table_access, unwrap_secret};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};
use std::collections::HashMap;

const USAGE: &str =
    "Usage: A_Admin.SetInventorySlotCandidates(slot, isTransmog, candidates, options)";

type Candidates = Vec<InventoryCandidate>;

#[derive(Default)]
struct Options {
    slot_names: Option<HashMap<String, u32>>,
    carried_enabled: Option<bool>,
    carried_count: Option<u32>,
    interaction_guid_nonzero: Option<bool>,
    transmog_ready: Option<bool>,
}

pub(super) fn register(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetInventorySlotCandidates", set_inventory_slot_candidates)
}

fn set_inventory_slot_candidates(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_public_slot(state)?;
    let is_transmog = bool::from_stack(state, 2)?;
    let candidates = read_candidates(state, stack_val(state, 3))?;
    let options = read_options(state, stack_val(state, 4))?;
    let mut sim = borrow_state_mut(state)?;
    sim.inventory_candidates
        .accepted
        .insert((slot, is_transmog), candidates);
    options.apply(&mut sim.inventory_candidates, slot);
    Ok(0)
}

fn read_public_slot(state: &LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    if (1..=30).contains(&slot) {
        Ok(slot - 1)
    } else {
        Err(runtime_error(USAGE))
    }
}

fn read_candidates(state: &mut LuaState, value: Val) -> LuaResult<Candidates> {
    let list = read_table(state, value)?;
    check_table_access(state, list, None, TableAccess::GetLength)?;
    let len = state
        .gc
        .tables
        .get(list)
        .ok_or_else(|| runtime_error(USAGE))?
        .len(&state.gc.string_arena);
    (1..=len)
        .map(|index| read_candidate(state, list, index))
        .collect()
}

fn read_candidate(
    state: &mut LuaState,
    list: GcRef<Table>,
    index: usize,
) -> LuaResult<InventoryCandidate> {
    let key = Val::Num(index as f64);
    check_table_access(state, list, Some(key), TableAccess::Index)?;
    let value = state
        .gc
        .tables
        .get(list)
        .ok_or_else(|| runtime_error(USAGE))?
        .get(key, &state.gc.string_arena);
    let table = read_table(state, value)?;
    Ok(InventoryCandidate {
        location: read_location(state, table)?,
        link: read_optional_string(state, table, "link")?,
    })
}

fn read_location(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<CandidateLocation> {
    let equipment_slot = read_optional_number(state, table, "equipmentSlot")?;
    let bag_id = read_optional_number(state, table, "bagID")?;
    let bag_slot = read_optional_number(state, table, "bagSlot")?;
    match (equipment_slot, bag_id, bag_slot) {
        (Some(slot), None, None) if (1..=19).contains(&slot) => {
            Ok(CandidateLocation::Equipment(slot - 1))
        }
        (None, bag, Some(slot)) if slot > 0 => {
            let bag_id = bag.unwrap_or(0);
            if bag_id < 4 || (5..=10).contains(&bag_id) {
                Ok(CandidateLocation::Container { id: bag_id, slot })
            } else {
                Err(runtime_error(USAGE))
            }
        }
        _ => Err(runtime_error("candidate needs equipmentSlot or bagSlot")),
    }
}

fn read_options(state: &mut LuaState, value: Val) -> LuaResult<Options> {
    if matches!(value, Val::Nil) {
        return Ok(Options::default());
    }
    let table = read_table(state, value)?;
    Ok(Options {
        slot_names: read_optional_slot_names(state, table)?,
        carried_enabled: read_optional_boolean(state, table, "carriedEnabled")?,
        carried_count: read_optional_number(state, table, "carriedCount")?,
        interaction_guid_nonzero: read_optional_boolean(state, table, "interactionGuidNonzero")?,
        transmog_ready: read_optional_boolean(state, table, "transmogReady")?,
    })
}

impl Options {
    fn apply(self, inputs: &mut InventoryCandidates, slot: u32) {
        if let Some(names) = self.slot_names {
            inputs.slot_names.extend(names);
        }
        if let Some(enabled) = self.carried_enabled {
            inputs.carried_enabled = enabled;
        }
        if let Some(count) = self.carried_count {
            inputs.carried_count = count;
        }
        if let Some(nonzero) = self.interaction_guid_nonzero {
            inputs.interaction_guid_nonzero = nonzero;
        }
        if let Some(ready) = self.transmog_ready {
            if ready {
                inputs.transmog_ready_slots.insert(slot);
            } else {
                inputs.transmog_ready_slots.remove(&slot);
            }
        }
    }
}

fn read_optional_slot_names(
    state: &mut LuaState,
    table: GcRef<Table>,
) -> LuaResult<Option<HashMap<String, u32>>> {
    let value = read_field(state, table, "slotNames")?;
    if matches!(value, Val::Nil) {
        return Ok(None);
    }
    let names_table = read_table(state, value)?;
    check_table_access(state, names_table, None, TableAccess::Iterate)?;
    read_slot_names(state, names_table).map(Some)
}

fn read_slot_names(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<HashMap<String, u32>> {
    let mut names = HashMap::new();
    let mut key = Val::Nil;
    while let Some((next, value)) = state
        .gc
        .tables
        .get(table)
        .ok_or_else(|| runtime_error(USAGE))?
        .next(key, &state.gc.string_arena)?
    {
        key = next.clone();
        let (name, slot) = read_slot_name(state, next, value)?;
        names.insert(name, slot);
    }
    Ok(names)
}

fn read_slot_name(state: &LuaState, key: Val, value: Val) -> LuaResult<(String, u32)> {
    let Val::Str(name) = key else {
        return Err(runtime_error("slotNames keys must be strings"));
    };
    let name = state
        .gc
        .string_arena
        .get(name)
        .map(|s| String::from_utf8_lossy(s.data()).into_owned())
        .ok_or_else(|| runtime_error(USAGE))?;
    let slot = read_number(unwrap_secret(state, value)?)?;
    if !(1..=30).contains(&slot) {
        return Err(runtime_error(USAGE));
    }
    Ok((name, slot))
}

fn read_optional_number(
    state: &mut LuaState,
    table: GcRef<Table>,
    name: &str,
) -> LuaResult<Option<u32>> {
    let value = read_field(state, table, name)?;
    match unwrap_secret(state, value)? {
        Val::Nil => Ok(None),
        value => read_number(value).map(Some),
    }
}

fn read_optional_boolean(
    state: &mut LuaState,
    table: GcRef<Table>,
    name: &str,
) -> LuaResult<Option<bool>> {
    let value = read_field(state, table, name)?;
    match unwrap_secret(state, value)? {
        Val::Nil => Ok(None),
        Val::Bool(value) => Ok(Some(value)),
        _ => Err(runtime_error(USAGE)),
    }
}

fn read_optional_string(
    state: &mut LuaState,
    table: GcRef<Table>,
    name: &str,
) -> LuaResult<Option<String>> {
    let value = read_field(state, table, name)?;
    match unwrap_secret(state, value)? {
        Val::Nil => Ok(None),
        Val::Str(value) => state
            .gc
            .string_arena
            .get(value)
            .map(|s| Some(String::from_utf8_lossy(s.data()).into_owned()))
            .ok_or_else(|| runtime_error(USAGE)),
        _ => Err(runtime_error(USAGE)),
    }
}

fn read_field(state: &mut LuaState, table: GcRef<Table>, name: &str) -> LuaResult<Val> {
    let key = state.gc.intern_string(name.as_bytes());
    check_table_access(state, table, Some(Val::Str(key)), TableAccess::Index)?;
    Ok(state
        .gc
        .tables
        .get(table)
        .ok_or_else(|| runtime_error(USAGE))?
        .get_str(key, &state.gc.string_arena))
}

fn read_table(state: &LuaState, value: Val) -> LuaResult<GcRef<Table>> {
    match unwrap_secret(state, value)? {
        Val::Table(table) => Ok(table),
        _ => Err(runtime_error(USAGE)),
    }
}

fn read_number(value: Val) -> LuaResult<u32> {
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= 0.0
                && number <= f64::from(u32::MAX) =>
        {
            Ok(number as u32)
        }
        _ => Err(runtime_error(USAGE)),
    }
}
