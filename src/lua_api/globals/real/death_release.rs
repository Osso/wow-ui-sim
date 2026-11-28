//! Spirit release, resurrect offers, area spirit healers and cemetery preference.
//! Retail/Forever PlayerScriptDocumentation blocks are byte-identical. Original
//! consumers: Blizzard_StaticPopup_Game DEATH / RESURRECT* / AREA_SPIRIT_HEAL dialogs,
//! Blizzard_Game EventImplementation (HandleResurrectRequest, area spirit healer
//! handlers) and Blizzard_SharedMapDataProviders SelectableGraveyardDataProvider.
//! Host inputs live in `inputs.rs`. Native timing and events are INFERRED (R9-6).

mod inputs;

use crate::lua_api::SimState;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};
use std::time::Instant;

pub(crate) use inputs::register_admin;

#[derive(Debug, Default)]
pub struct DeathReleaseState {
    pub is_ghost: bool,
    pub corpse_recovery_until: Option<Instant>,
    /// Attributes of the pending resurrect offer (`pending_resurrect`).
    pub offer_has_sickness: bool,
    pub offer_has_timer: bool,
    pub res_sickness_duration: Option<String>,
    /// Next area spirit healer wave while the player is in a healer's range.
    pub area_heal_wave: Option<Instant>,
    pub area_heal_queued: bool,
    pub cemetery_selection_available: bool,
    pub cemetery_preference: u32,
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let functions: [(&str, rilua::vm::closure::RustFn); 12] = [
        ("RepopMe", repop_me),
        ("GetCorpseRecoveryDelay", get_corpse_recovery_delay),
        ("ResurrectHasSickness", resurrect_has_sickness),
        ("ResurrectHasTimer", resurrect_has_timer),
        ("GetResSicknessDuration", get_res_sickness_duration),
        ("TimeoutResurrect", timeout_resurrect),
        ("AcceptAreaSpiritHeal", accept_area_spirit_heal),
        ("CancelAreaSpiritHeal", cancel_area_spirit_heal),
        ("GetAreaSpiritHealerTime", get_area_spirit_healer_time),
        (
            "IsCemeterySelectionAvailable",
            is_cemetery_selection_available,
        ),
        ("GetCemeteryPreference", get_cemetery_preference),
        ("SetCemeteryPreference", set_cemetery_preference),
    ];
    for (name, function) in functions {
        LuaApiMut::register_function(lua, name, function)?;
    }
    Ok(())
}

/// Whole seconds until `deadline` on the simulation clock, rounded up; 0 when unset.
fn seconds_until(sim: &SimState, deadline: Option<Instant>) -> f64 {
    deadline.map_or(0.0, |deadline| {
        deadline
            .saturating_duration_since(sim.clock_now())
            .as_secs_f64()
            .ceil()
    })
}

fn push_bool(state: &mut LuaState, read: fn(&SimState) -> bool) -> LuaResult<u32> {
    let value = read(&*borrow_state(state)?);
    state.push(Val::Bool(value));
    Ok(1)
}

fn push_number(state: &mut LuaState, read: fn(&SimState) -> f64) -> LuaResult<u32> {
    let value = read(&*borrow_state(state)?);
    state.push(Val::Num(value));
    Ok(1)
}

/// INFERRED: releasing turns the dead player into a ghost, ends the release timer,
/// leaves the corpse behind and fires PLAYER_ALIVE (the event a release raises).
fn repop_me(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        if sim.player.health > 0 || sim.death_release.is_ghost {
            return Ok(0);
        }
        sim.death_release.is_ghost = true;
        sim.release_timer = None;
        sim.corpse_available = true;
    }
    dispatch_event_now(state, "PLAYER_ALIVE", &[])?;
    Ok(0)
}

fn get_corpse_recovery_delay(state: &mut LuaState) -> LuaResult<u32> {
    push_number(state, |sim| {
        seconds_until(sim, sim.death_release.corpse_recovery_until)
    })
}

fn resurrect_has_sickness(state: &mut LuaState) -> LuaResult<u32> {
    push_bool(state, |sim| {
        sim.pending_resurrect.is_some() && sim.death_release.offer_has_sickness
    })
}

fn resurrect_has_timer(state: &mut LuaState) -> LuaResult<u32> {
    push_bool(state, |sim| {
        sim.pending_resurrect.is_some() && sim.death_release.offer_has_timer
    })
}

fn get_res_sickness_duration(state: &mut LuaState) -> LuaResult<u32> {
    let text = borrow_state(state)?
        .death_release
        .res_sickness_duration
        .clone();
    let value = text.map_or(Val::Nil, |text| create_string(state, &text));
    state.push(value);
    Ok(1)
}

/// INFERRED: an expired offer is withdrawn like a declined one.
fn timeout_resurrect(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.pending_resurrect = None;
    Ok(0)
}

/// Queues for the next wave while dead or a ghost within a healer's range.
fn accept_area_spirit_heal(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let eligible = sim.player.health <= 0 && sim.death_release.area_heal_wave.is_some();
    sim.death_release.area_heal_queued |= eligible;
    Ok(0)
}

fn cancel_area_spirit_heal(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.death_release.area_heal_queued = false;
    Ok(0)
}

/// Seconds until the queued wave; 0 while not queued (R9-6).
fn get_area_spirit_healer_time(state: &mut LuaState) -> LuaResult<u32> {
    push_number(state, |sim| {
        let wave = sim.death_release.area_heal_wave;
        let queued = sim.death_release.area_heal_queued;
        if queued {
            seconds_until(sim, wave)
        } else {
            0.0
        }
    })
}

fn is_cemetery_selection_available(state: &mut LuaState) -> LuaResult<u32> {
    push_bool(state, |sim| sim.death_release.cemetery_selection_available)
}

fn get_cemetery_preference(state: &mut LuaState) -> LuaResult<u32> {
    push_number(state, |sim| {
        f64::from(sim.death_release.cemetery_preference)
    })
}

/// INFERRED: a changed preference raises CEMETERY_PREFERENCE_UPDATED.
fn set_cemetery_preference(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let changed = sim.death_release.cemetery_preference != id;
        sim.death_release.cemetery_preference = id;
        changed
    };
    if changed {
        dispatch_event_now(state, "CEMETERY_PREFERENCE_UPDATED", &[])?;
    }
    Ok(0)
}

/// Resurrection or corpse recovery ends ghost form; true when the player was a ghost.
pub(crate) fn leave_ghost_form(sim: &mut SimState) -> bool {
    std::mem::take(&mut sim.death_release.is_ghost)
}
