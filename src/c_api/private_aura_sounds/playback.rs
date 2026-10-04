//! Aura-change detection for registered aura sounds and an observable playback log.
//!
//! INFERRED: native detection lives in the aura system; the simulator diffs each
//! registered unit's auras once per tick, so every aura mutation path is covered.
//! A unit's first observation (at registration or first tick) is a silent baseline.

use super::AuraSoundRegistration;
use crate::lua_api::globals::auras::collect_all_unit_auras;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use rilua::vm::state::LuaState;
use std::collections::{BTreeSet, HashMap};

pub const TRIGGER_ADDED: u32 = 0;
pub const TRIGGER_APPLICATIONS_INCREASED: u32 = 1;
pub const TRIGGER_REMOVED: u32 = 2;

/// One played aura sound, in playback order.
#[derive(Clone, Debug, PartialEq)]
pub struct AuraSoundPlayback {
    pub aura_sound_id: u32,
    pub trigger: u32,
    pub unit_token: String,
    pub spell_id: u32,
    pub aura_instance_id: i32,
    pub sound_file_name: Option<String>,
    pub sound_file_id: Option<u32>,
    pub output_channel: Option<String>,
}

/// aura instance id -> (spell id, applications)
pub(super) type UnitAuraSnapshot = HashMap<i32, (u32, i32)>;

struct AuraChange {
    trigger: u32,
    spell_id: u32,
    aura_instance_id: i32,
}

fn snapshot_unit(state: &mut LuaState, unit: &str) -> UnitAuraSnapshot {
    collect_all_unit_auras(state, unit)
        .into_iter()
        .map(|aura| {
            let spell_id = u32::try_from(aura.spell_id).unwrap_or(0);
            (aura.aura_instance_id, (spell_id, aura.applications))
        })
        .collect()
}

/// Record a silent baseline for `unit` unless it is already observed.
pub(super) fn observe_unit(state: &mut LuaState, unit: &str) {
    let observed = borrow_state(state).is_ok_and(|sim| {
        sim.private_aura_sound_registrations
            .observed
            .contains_key(unit)
    });
    if observed {
        return;
    }
    let snapshot = snapshot_unit(state, unit);
    if let Ok(mut sim) = borrow_state_mut(state) {
        sim.private_aura_sound_registrations
            .observed
            .insert(unit.to_string(), snapshot);
    }
}

fn diff_snapshots(previous: &UnitAuraSnapshot, current: &UnitAuraSnapshot) -> Vec<AuraChange> {
    let mut instance_ids: BTreeSet<i32> = previous.keys().copied().collect();
    instance_ids.extend(current.keys().copied());
    instance_ids
        .into_iter()
        .filter_map(|aura_instance_id| {
            let (trigger, spell_id) = match (
                previous.get(&aura_instance_id),
                current.get(&aura_instance_id),
            ) {
                (None, Some(&(spell_id, _))) => (TRIGGER_ADDED, spell_id),
                (Some(&(spell_id, _)), None) => (TRIGGER_REMOVED, spell_id),
                (Some(&(_, before)), Some(&(spell_id, after))) if after > before => {
                    (TRIGGER_APPLICATIONS_INCREASED, spell_id)
                }
                _ => return None,
            };
            Some(AuraChange {
                trigger,
                spell_id,
                aura_instance_id,
            })
        })
        .collect()
}

fn playbacks_for_change(
    registrations: &HashMap<u32, AuraSoundRegistration>,
    unit: &str,
    change: &AuraChange,
) -> Vec<AuraSoundPlayback> {
    let mut ids: Vec<u32> = registrations
        .iter()
        .filter(|(_, registration)| {
            registration.unit_token == unit
                && registration.spell_id == change.spell_id
                && registration.trigger == change.trigger
        })
        .map(|(id, _)| *id)
        .collect();
    ids.sort_unstable();
    ids.into_iter()
        .map(|id| {
            let registration = &registrations[&id];
            AuraSoundPlayback {
                aura_sound_id: id,
                trigger: change.trigger,
                unit_token: unit.to_string(),
                spell_id: change.spell_id,
                aura_instance_id: change.aura_instance_id,
                sound_file_name: registration.sound_file_name.clone(),
                sound_file_id: registration.sound_file_id,
                output_channel: registration.output_channel.clone(),
            }
        })
        .collect()
}

fn registered_units(state: &LuaState) -> Vec<String> {
    let Ok(sim) = borrow_state(state) else {
        return Vec::new();
    };
    let units: BTreeSet<String> = sim
        .private_aura_sound_registrations
        .registrations
        .values()
        .map(|registration| registration.unit_token.clone())
        .collect();
    units.into_iter().collect()
}

/// Per-tick pass: diff every registered unit and play matching sounds.
pub(crate) fn tick(state: &mut LuaState) {
    let units = registered_units(state);
    if let Ok(mut sim) = borrow_state_mut(state) {
        sim.private_aura_sound_registrations
            .observed
            .retain(|unit, _| units.contains(unit));
    }
    for unit in units {
        let current = snapshot_unit(state, &unit);
        let Ok(mut sim) = borrow_state_mut(state) else {
            return;
        };
        let sounds = &mut sim.private_aura_sound_registrations;
        let Some(previous) = sounds.observed.insert(unit.clone(), current.clone()) else {
            continue;
        };
        let playbacks: Vec<AuraSoundPlayback> = diff_snapshots(&previous, &current)
            .iter()
            .flat_map(|change| playbacks_for_change(&sounds.registrations, &unit, change))
            .collect();
        for playback in playbacks {
            if let (Some(path), Some(manager)) = (
                playback.sound_file_name.as_deref(),
                sim.sound_manager.as_mut(),
            ) {
                let _ = manager.play_sound_file(path);
            }
            sim.private_aura_sound_registrations
                .playbacks
                .push(playback);
        }
    }
}
