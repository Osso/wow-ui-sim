//! State-backed loot-method probe + refresh globals.
//!
//! Migrates 3 entries off `GLOBAL_ZERO_STUBS`:
//!
//! - `GetLootMethod()`             → `(method, partyMasterIndex,
//!   raidMasterIndex)` from `SimState.loot_method`.
//! - `GetMasterLooterThreshold()`  → `loot_method.threshold` (item
//!   quality enum, 0..=4).
//! - `RequestPartyLootMethod()`    → fires `PARTY_LOOT_METHOD_CHANGED`
//!   so listeners re-poll on demand.

use crate::event::Event;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

/// `GetLootMethod()` — retail returns `(method, partyMasterIndex,
/// raidMasterIndex)`. Master-looter indices stay 0 when `method` is
/// anything other than `"master"`.
fn get_loot_method(state: &mut LuaState) -> LuaResult<u32> {
    let (method, party_idx, raid_idx) = {
        let sim = borrow_state(state)?;
        (
            sim.loot_method.method.clone(),
            sim.loot_method.party_master_index,
            sim.loot_method.raid_master_index,
        )
    };
    let method_val = create_string(state, &method);
    state.push(method_val);
    state.push(Val::Num(party_idx as f64));
    state.push(Val::Num(raid_idx as f64));
    Ok(3)
}

/// `GetMasterLooterThreshold()` — retail returns the item quality
/// cutoff (enum value 0..=4).
fn get_master_looter_threshold(state: &mut LuaState) -> LuaResult<u32> {
    let threshold = borrow_state(state)?.loot_method.threshold;
    state.push(Val::Num(threshold as f64));
    Ok(1)
}

/// Loot opt-out selection; unsupplied state is false (not opted out).
#[cfg(any(feature = "client-wrath", feature = "client-retail"))]
fn get_opt_out_of_loot(state: &mut LuaState) -> LuaResult<u32> {
    let opt_out = borrow_state(state)?.loot_method.opt_out;
    state.push(Val::Bool(opt_out));
    Ok(1)
}

/// `RequestPartyLootMethod()` — retail asks the server to re-send the
/// party's current loot method, which fires `PARTY_LOOT_METHOD_CHANGED`.
/// The sim already holds authoritative state, so we just replay the
/// event for listeners.
fn request_party_loot_method(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.events.push(Event {
        name: "PARTY_LOOT_METHOD_CHANGED".to_string(),
        args: Vec::new(),
    });
    Ok(0)
}

/// `SetOptOutOfLoot(optOut)` — the Retail unit popup passes `1` or `nil`.
/// INFERRED (R-ag10-16): no event fires; the popup re-reads `GetOptOutOfLoot`.
#[cfg(feature = "client-retail")]
fn set_opt_out_of_loot(state: &mut LuaState) -> LuaResult<u32> {
    let opt_out = !matches!(
        crate::lua_bridge::stack_val(state, 1),
        Val::Nil | Val::Bool(false)
    );
    borrow_state_mut(state)?.loot_method.opt_out = opt_out;
    Ok(0)
}

/// `SetLootThreshold(quality)` — the group leader sets the group-loot item
/// quality threshold (the Retail popup offers 2 Uncommon .. 4 Epic).
/// INFERRED (R-ag10-16): ungrouped players, non-leaders and other qualities
/// change nothing; a change fires `PARTY_LOOT_METHOD_CHANGED`.
#[cfg(feature = "client-retail")]
fn set_loot_threshold(state: &mut LuaState) -> LuaResult<u32> {
    let quality = match crate::lua_bridge::stack_val(state, 1) {
        Val::Num(n) if matches!(n, 2.0 | 3.0 | 4.0) => n as i32,
        _ => return Ok(0),
    };
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let grouped = sim.party_group_active && !sim.party_members.is_empty();
        let leader = grouped && sim.party_leader_index.is_none();
        let changed = leader && sim.loot_method.threshold != quality;
        if changed {
            sim.loot_method.threshold = quality;
        }
        changed
    };
    if changed {
        crate::lua_api::globals::state_backed_queries::dispatch_event_now(
            state,
            "PARTY_LOOT_METHOD_CHANGED",
            &[],
        )?;
    }
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetLootMethod", get_loot_method)?;
    LuaApiMut::register_function(lua, "GetMasterLooterThreshold", get_master_looter_threshold)?;
    #[cfg(any(feature = "client-wrath", feature = "client-retail"))]
    LuaApiMut::register_function(lua, "GetLootThreshold", get_master_looter_threshold)?;
    #[cfg(any(feature = "client-wrath", feature = "client-retail"))]
    LuaApiMut::register_function(lua, "GetOptOutOfLoot", get_opt_out_of_loot)?;
    #[cfg(feature = "client-retail")]
    LuaApiMut::register_function(lua, "SetOptOutOfLoot", set_opt_out_of_loot)?;
    #[cfg(feature = "client-retail")]
    LuaApiMut::register_function(lua, "SetLootThreshold", set_loot_threshold)?;
    LuaApiMut::register_function(lua, "RequestPartyLootMethod", request_party_loot_method)?;
    Ok(())
}
