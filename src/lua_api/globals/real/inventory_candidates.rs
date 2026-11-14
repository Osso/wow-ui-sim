//! R88k.1 synchronous supplied-table publication. Unknown filters and link
//! builders are explicit host inputs, empty by default; no items are invented.
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::stack_val;
use rilua::table_security::{TableAccess, check_table_access, unwrap_secret};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};
use std::collections::{HashMap, HashSet};

const USAGE: &str = "Usage: GetInventoryItemsForSlot(slot, returnTable, [\"transmogrify\"])";

#[derive(Debug, Clone, Default)]
pub struct InventoryCandidates {
    pub slot_names: HashMap<String, u32>,
    pub carried_enabled: bool,
    pub carried_count: u32,
    pub interaction_guid_nonzero: bool,
    /// Slots having both selected item and required record for transmog mode.
    pub transmog_ready_slots: HashSet<u32>,
    /// Accepted candidates supplied by the host, keyed by requested internal slot
    /// and mode. The partially known native eligibility predicate stays UNKNOWN.
    pub accepted: HashMap<(u32, bool), Vec<InventoryCandidate>>,
}

#[derive(Debug, Clone)]
pub struct InventoryCandidate {
    pub location: CandidateLocation,
    /// None is the verified selected-builder no-text result (key removal).
    pub link: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum CandidateLocation {
    Equipment(u32),
    Carried(u32),
    Container { id: u32, slot: u32 },
}

impl InventoryCandidates {
    fn key(&self, location: CandidateLocation, transmog: bool) -> Option<u64> {
        let packed = match location {
            CandidateLocation::Equipment(slot) if slot < 19 => 1048576 + u64::from(slot),
            CandidateLocation::Carried(slot)
                if self.carried_enabled && slot >= 35 && slot - 35 < self.carried_count =>
            {
                3145728 + u64::from(slot - 35)
            }
            CandidateLocation::Container { id, slot } if id < 4 => {
                3145728 + u64::from(id + 1) * 256 + u64::from(slot)
            }
            CandidateLocation::Container { id, slot }
                if (5..=10).contains(&id) && (self.interaction_guid_nonzero || transmog) =>
            {
                6291456 + u64::from(id - 4) * 256 + u64::from(slot)
            }
            _ => return None,
        };
        Some(packed + 1)
    }
}

fn read_slot(state: &LuaState) -> LuaResult<Option<u32>> {
    let value = unwrap_secret(state, stack_val(state, 1)).unwrap_or(Val::Nil);
    let text = if let Val::Str(r) = value {
        state.gc.string_arena.get(r).map(|s| s.data())
    } else {
        None
    };
    let number = match value {
        Val::Num(n) => Some(n),
        _ => text.and_then(crate::c_api::parse_numeric_string),
    };
    if let Some(n) = number {
        let slot = (n - 1.0).trunc() as i64 as u32;
        return Ok((slot < 30).then_some(slot));
    }
    if let Some(name) = text {
        let name = String::from_utf8_lossy(name);
        let sim = borrow_state(state)?;
        if let Some(&id) = sim
            .inventory_candidates
            .slot_names
            .iter()
            .find_map(|(key, id)| key.eq_ignore_ascii_case(&name).then_some(id))
        {
            if id != 0 {
                return Ok(Some(id - 1));
            }
        }
    }
    Err(runtime_error(USAGE))
}

fn transmog_mode(state: &LuaState) -> bool {
    let Ok(Val::Str(r)) = unwrap_secret(state, stack_val(state, 3)) else {
        return false;
    };
    state.gc.string_arena.get(r).is_some_and(|s| {
        s.data()
            .split(|&c| c == 0)
            .next()
            .unwrap_or_default()
            .eq_ignore_ascii_case(b"transmogrify")
    })
}

fn get_inventory_items_for_slot(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = read_slot(state)? else {
        return Ok(0);
    };
    let Val::Table(table) = unwrap_secret(state, stack_val(state, 2)).unwrap_or(Val::Nil) else {
        return Err(runtime_error(USAGE));
    };
    let mode = transmog_mode(state);
    let sim = borrow_state(state)?;
    let inputs = &sim.inventory_candidates;
    if !sim.player.present || (mode && !inputs.transmog_ready_slots.contains(&slot)) {
        return Ok(0);
    }
    let writes: Vec<_> = inputs
        .accepted
        .get(&(slot, mode))
        .into_iter()
        .flatten()
        .filter_map(|candidate| {
            inputs
                .key(candidate.location, mode)
                .map(|key| (key, candidate.link.clone()))
        })
        .collect();
    drop(sim);
    write_candidates(state, table, writes)?;
    state.push(Val::Table(table));
    Ok(1)
}

fn write_candidates(
    state: &mut LuaState,
    table: GcRef<Table>,
    writes: Vec<(u64, Option<String>)>,
) -> LuaResult<()> {
    for (key, text) in writes {
        let key = Val::Num(key as f64);
        check_table_access(state, table, Some(key), TableAccess::IndexedAssignment)?;
        if state.gc.tables.is_frozen(table)
            || state.gc.tables.get(table).is_some_and(|t| t.is_read_only())
        {
            return Err(runtime_error(
                "attempted to perform indexed assignment on a frozen table",
            ));
        }
        let value = text
            .as_deref()
            .map(|s| create_string(state, s))
            .unwrap_or(Val::Nil);
        state
            .gc
            .tables
            .get_mut(table)
            .ok_or_else(|| runtime_error("table has been collected"))?
            .raw_set(key, value, &state.gc.string_arena)?;
        state.gc.barrier_back(table);
    }
    Ok(())
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    if cfg!(feature = "client-retail") {
        LuaApiMut::register_function(
            lua,
            "GetInventoryItemsForSlot",
            get_inventory_items_for_slot,
        )?;
    }
    Ok(())
}
