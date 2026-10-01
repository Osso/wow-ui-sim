//! Explicit DamageMeter inputs and independent out-of-combat query snapshots.
//! Combat publication is blocked until field-level secrecy has a modeled contract.

use std::collections::HashMap;

use super::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

mod snapshot;

const CURRENT_SESSION_TYPE: i32 = 1;

/// Empty/disabled default and explicit type bindings are simulator inferences.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DamageMeterInput {
    pub available: bool,
    pub failure_reason: String,
    pub available_sessions: Vec<DamageMeterAvailableCombatSession>,
    /// DamageMeterSessionType numeric value -> explicitly supplied session ID.
    pub session_ids_by_type: HashMap<i32, i64>,
    /// (session ID, DamageMeterType numeric value) -> aggregate snapshot input.
    pub sessions: HashMap<(i64, i32), DamageMeterCombatSession>,
    pub source_details: HashMap<(i64, i32, DamageMeterSourceKey), DamageMeterCombatSessionSource>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageMeterAvailableCombatSession {
    pub session_id: i64,
    pub name: String,
    pub duration_seconds: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DamageMeterCombatSession {
    pub combat_sources: Vec<DamageMeterCombatSource>,
    pub max_amount: f64,
    pub total_amount: f64,
    pub duration_seconds: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DamageMeterSourceKey {
    pub source_guid: Option<String>,
    pub source_creature_id: Option<i64>,
}

/// DamageMeterCombatSource: aggregate identity/metrics, never spell details.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageMeterCombatSource {
    pub source_guid: Option<String>,
    pub source_creature_id: Option<i64>,
    /// ConditionalSecret in cached documentation; not inherently readable.
    pub name: String,
    pub class_filename: String,
    pub spec_icon_id: i64,
    pub total_amount: f64,
    pub amount_per_second: f64,
    pub is_local_player: bool,
    pub death_recap_id: i64,
    pub death_time_seconds: f64,
    pub classification: String,
    pub source_display_type: i32,
    pub faction_group: Option<String>,
}

/// DamageMeterCombatSessionSource: direct-detail result, not an aggregate row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DamageMeterCombatSessionSource {
    pub combat_spells: Vec<DamageMeterCombatSpell>,
    pub max_amount: f64,
    pub total_amount: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageMeterCombatSpell {
    pub spell_id: i64,
    pub total_amount: f64,
    pub amount_per_second: f64,
    pub creature_name: String,
    pub overkill_amount: f64,
    pub is_avoidable: bool,
    pub is_deadly: bool,
    // Documentation declares one structure, not a sequence of target rows.
    pub combat_spell_details: DamageMeterCombatSpellUnitDetails,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageMeterCombatSpellUnitDetails {
    pub unit_name: String,
    pub unit_class_filename: String,
    pub classification: String,
    pub is_pet: bool,
    pub is_mob: bool,
    pub amount: f64,
    pub spec_icon_id: i64,
}

impl DamageMeterInput {
    fn bound_session_id(&self, session_type: i32) -> Option<i64> {
        self.session_ids_by_type.get(&session_type).copied()
    }

    fn find_source_details(
        &self,
        session_id: Option<i64>,
        meter: i32,
        selector: &DamageMeterSourceKey,
    ) -> Option<&DamageMeterCombatSessionSource> {
        let session_id = session_id?;
        if selector.source_guid.is_none() && selector.source_creature_id.is_none() {
            return None;
        }
        let mut matches = self.source_details.iter().filter(|((id, kind, key), _)| {
            *id == session_id && *kind == meter && selector.matches(key)
        });
        let (_, details) = matches.next()?;
        // Partial selectors must be unique; HashMap iteration never chooses a winner.
        if matches.next().is_some() {
            return None;
        }
        Some(details)
    }

    fn clear_sessions(&mut self) {
        self.available_sessions.clear();
        self.session_ids_by_type.clear();
        self.sessions.clear();
        self.source_details.clear();
    }
}

impl DamageMeterSourceKey {
    fn matches(&self, key: &Self) -> bool {
        let guid_matches = self.source_guid.is_none() || self.source_guid == key.source_guid;
        let creature_matches =
            self.source_creature_id.is_none() || self.source_creature_id == key.source_creature_id;
        guid_matches && creature_matches
    }

    fn from_arguments(state: &LuaState) -> LuaResult<Self> {
        Ok(Self {
            source_guid: Option::<String>::from_stack(state, 3)?,
            source_creature_id: Option::<i64>::from_stack(state, 4)?,
        })
    }
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_DamageMeter")?;
    let functions: &[(&str, rilua::RustFn)] = &[
        ("IsDamageMeterAvailable", is_available),
        ("GetAvailableCombatSessions", available_sessions),
        ("GetCurrentCombatSessionID", current_session_id),
        ("GetCombatSessionFromID", session_from_id),
        ("GetCombatSessionFromType", session_from_type),
        ("GetCombatSessionSourceFromID", source_from_id),
        ("GetCombatSessionSourceFromType", source_from_type),
        ("GetSessionDurationSeconds", duration_seconds),
        ("ResetAllCombatSessions", reset_sessions),
    ];
    for &(name, function) in functions {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    Ok(())
}

fn is_available(state: &mut LuaState) -> LuaResult<u32> {
    let (available, reason) = {
        let sim = borrow_state(state)?;
        (
            sim.damage_meter.available,
            sim.damage_meter.failure_reason.clone(),
        )
    };
    state.push(Val::Bool(available));
    let reason = create_string(state, &reason);
    state.push(reason);
    Ok(2)
}

fn available_sessions(state: &mut LuaState) -> LuaResult<u32> {
    let entries = borrow_state(state)?.damage_meter.available_sessions.clone();
    snapshot::push_available_sessions(state, &entries);
    Ok(1)
}

fn current_session_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?
        .damage_meter
        .bound_session_id(CURRENT_SESSION_TYPE);
    state.push(id.map_or(Val::Nil, |id| Val::Num(id as f64)));
    Ok(1)
}

fn duration_seconds(state: &mut LuaState) -> LuaResult<u32> {
    let session_type = i32::from_stack(state, 1)?;
    let duration = {
        let sim = borrow_state(state)?;
        let input = &sim.damage_meter;
        let id = input.bound_session_id(session_type);
        input
            .available_sessions
            .iter()
            .find(|entry| Some(entry.session_id) == id)
            .and_then(|entry| entry.duration_seconds)
    };
    state.push(duration.map_or(Val::Nil, Val::Num));
    Ok(1)
}

/// No automatic documentation annotation enforcement exists in registration or
/// C_Secrets. Fail explicitly rather than disclose plain input during combat.
/// Retire this block only after field-level secrecy and selector access are modeled.
fn require_out_of_combat(state: &LuaState) -> LuaResult<()> {
    if borrow_state(state)?.player.in_combat {
        return Err(rilua::runtime_error(
            "C_DamageMeter combat publication is unavailable: field secrecy is not modeled",
        ));
    }
    Ok(())
}

fn session_from_id(state: &mut LuaState) -> LuaResult<u32> {
    require_out_of_combat(state)?;
    let id = i64::from_stack(state, 1)?;
    let meter = i32::from_stack(state, 2)?;
    push_session(state, Some(id), meter)
}

fn session_from_type(state: &mut LuaState) -> LuaResult<u32> {
    require_out_of_combat(state)?;
    let session_type = i32::from_stack(state, 1)?;
    let meter = i32::from_stack(state, 2)?;
    let id = borrow_state(state)?
        .damage_meter
        .bound_session_id(session_type);
    push_session(state, id, meter)
}

fn push_session(state: &mut LuaState, id: Option<i64>, meter: i32) -> LuaResult<u32> {
    let input = {
        let sim = borrow_state(state)?;
        id.and_then(|id| sim.damage_meter.sessions.get(&(id, meter)))
            .cloned()
    };
    // Missing-selector empty wrappers are simulator policy, not stored records.
    snapshot::push_session(state, &input.unwrap_or_default());
    Ok(1)
}

fn source_from_id(state: &mut LuaState) -> LuaResult<u32> {
    require_out_of_combat(state)?;
    let id = i64::from_stack(state, 1)?;
    let meter = i32::from_stack(state, 2)?;
    let selector = DamageMeterSourceKey::from_arguments(state)?;
    push_source(state, Some(id), meter, &selector)
}

fn source_from_type(state: &mut LuaState) -> LuaResult<u32> {
    require_out_of_combat(state)?;
    let session_type = i32::from_stack(state, 1)?;
    let meter = i32::from_stack(state, 2)?;
    let selector = DamageMeterSourceKey::from_arguments(state)?;
    let id = borrow_state(state)?
        .damage_meter
        .bound_session_id(session_type);
    push_source(state, id, meter, &selector)
}

fn push_source(
    state: &mut LuaState,
    id: Option<i64>,
    meter: i32,
    selector: &DamageMeterSourceKey,
) -> LuaResult<u32> {
    let input = borrow_state(state)?
        .damage_meter
        .find_source_details(id, meter, selector)
        .cloned();
    snapshot::push_source_details(state, &input.unwrap_or_default());
    Ok(1)
}

fn reset_sessions(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.damage_meter.clear_sessions();
    Ok(0)
}
