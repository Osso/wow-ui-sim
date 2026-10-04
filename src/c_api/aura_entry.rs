//! Shared host-backed aura-ID entry transition. Native randomness is host-owned.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::SimState;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};
use std::collections::HashSet;

pub(crate) fn apply_entry_event(state: &mut LuaState, event: &str) -> LuaResult<()> {
    if !matches!(
        event,
        "ENCOUNTER_START" | "CHALLENGE_MODE_START" | "PVP_MATCH_ACTIVE"
    ) {
        return Ok(());
    }
    let mut sim = borrow_state_mut(state)?;
    rekey_aura_instance_ids(&mut sim)
}

fn collect_live_ids(sim: &SimState) -> Vec<i32> {
    sim.player
        .buffs
        .iter()
        .chain(
            sim.party_members
                .iter()
                .flat_map(|member| member.buffs.iter().chain(member.debuffs.iter())),
        )
        .map(|aura| aura.aura_instance_id)
        .collect()
}

fn validate_replacements(sim: &SimState, live_ids: &[i32]) -> LuaResult<()> {
    let Some(replacements) = &sim.aura_entry_ids.pending else {
        return Ok(());
    };
    if replacements.len() != live_ids.len() {
        return Err(runtime_error(
            "aura entry replacement count does not match live aura count",
        ));
    }
    let live: HashSet<i32> = live_ids.iter().copied().collect();
    let mut selected = HashSet::new();
    for id in replacements {
        let reused = live.contains(id) || sim.aura_entry_ids.retired.contains(id);
        if *id <= 0 || reused || !selected.insert(*id) {
            return Err(runtime_error(
                "aura entry requires distinct positive fresh replacement IDs",
            ));
        }
    }
    Ok(())
}

fn rekey_aura_instance_ids(sim: &mut SimState) -> LuaResult<()> {
    let live_ids = collect_live_ids(sim);
    // No staged host batch: the entry event is delivered with IDs unchanged.
    if sim.aura_entry_ids.pending.is_none() {
        return Ok(());
    }
    // Validate the entire batch before changing any ID or consuming host input.
    validate_replacements(sim, &live_ids)?;
    let replacements = sim
        .aura_entry_ids
        .pending
        .take()
        .expect("validated host batch");
    sim.aura_entry_ids.retired.extend(live_ids);
    let auras = sim.player.buffs.iter_mut().chain(
        sim.party_members
            .iter_mut()
            .flat_map(|member| member.buffs.iter_mut().chain(member.debuffs.iter_mut())),
    );
    for (aura, id) in auras.zip(replacements) {
        aura.aura_instance_id = id;
    }
    Ok(())
}
