//! Supplied challenge lifecycle and catalogue. Pinned declarations:
//! docs/specs/challenge-mode-run.md. No native dungeon policy is inferred.
mod inputs;
mod scenario;
pub(crate) mod timers;

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_string, create_table};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::BTreeMap;

/// Client affix array capacity: the getter fills a fixed four-slot array and
/// stops at the first zero slot (decomp answer, 12.1.5).
const ACTIVE_KEYSTONE_AFFIX_SLOTS: usize = 4;

/// Client keystone globals behind `GetActiveKeystoneInfo`: the key level, a
/// fixed four-slot affix array and the charged byte. The client has exactly
/// three writers for them and no reader derives one from another, so they are
/// stored apart from the run bookkeeping. A run start applies the decoded
/// values; two complementary reset routines store zero; nothing else writes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActiveKeystone {
    pub level: i32,
    pub affixes: [i32; ACTIVE_KEYSTONE_AFFIX_SLOTS],
    pub charged: bool,
}

impl ActiveKeystone {
    /// Applicator: copy the decoded run values into the four client slots.
    pub fn applied(level: i32, affixes: &[i32], charged: bool) -> Self {
        let mut slots = [0; ACTIVE_KEYSTONE_AFFIX_SLOTS];
        for (slot, id) in slots.iter_mut().zip(affixes) {
            *slot = *id;
        }
        Self {
            level,
            affixes: slots,
            charged,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ActiveRun {
    pub map_id: i32,
    pub event_map_id: i32,
    pub level: i32,
    pub affixes: Vec<i32>,
    pub charged: bool,
    pub deaths: i32,
    pub time_lost: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct MapInfo {
    pub name: String,
    pub id: i32,
    pub time_limit: f64,
    pub texture: Option<f64>,
    pub background: f64,
    pub map_id: i32,
}

#[derive(Debug, Default)]
pub(crate) struct ChallengeState {
    pub active: Option<ActiveRun>,
    pub active_keystone: ActiveKeystone,
    pub maps: BTreeMap<i32, MapInfo>,
    pub timers: BTreeMap<i32, timers::ElapsedTimer>,
    pub criteria: BTreeMap<i32, Vec<scenario::Criterion>>,
    /// Supplied scenario criteria visibility; `false` (default) keeps criteria shown.
    pub criteria_hidden: bool,
}

fn supports_run() -> bool {
    ACTIVE != ClientProfile::Wrath
}

fn supports_keystone() -> bool {
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !supports_run() {
        return Ok(());
    }
    let ns = super::ensure_namespace(state, "C_ChallengeMode")?;
    table_set_rust_fn_static(state, ns, "GetActiveChallengeMapID", active_map)?;
    table_set_rust_fn_static(state, ns, "IsChallengeModeActive", is_active)?;
    if supports_keystone() {
        table_set_rust_fn_static(state, ns, "GetActiveKeystoneInfo", active_keystone)?;
        table_set_rust_fn_static(state, ns, "GetDeathCount", death_count)?;
        super::c_challenge_mode::register(state)?;
    }
    scenario::register(state)
}

pub(crate) fn register_maps(
    state: &mut LuaState,
    ns: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "GetMapTable", map_table)?;
    table_set_rust_fn_static(state, ns, "GetMapUIInfo", map_info)
}

pub(crate) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    if !supports_run() {
        return Ok(b);
    }
    let b = b
        .set_function("SetChallengeModeMapInfo", inputs::set_map)?
        .set_function("StartChallengeRun", inputs::start_run)?
        .set_function("ClearChallengeRun", inputs::clear_run)?
        .set_function("CompleteChallengeRun", inputs::complete_run)?
        .set_function("SetScenarioCriteria", scenario::set_criteria)?
        .set_function("SetScenarioCriteriaShown", scenario::set_criteria_shown)?
        .set_function("StartWorldElapsedTimer", timers::start)?
        .set_function("SetWorldElapsedTime", timers::set)?
        .set_function("AdvanceWorldElapsedTime", timers::advance)?;
    if !supports_keystone() {
        return Ok(b);
    }
    b.set_function("ResetActiveKeystoneInfo", inputs::reset_active_keystone)
}

fn active_map(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?
        .challenge_run
        .active
        .as_ref()
        .map(|run| run.map_id);
    state.push(id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}

fn is_active(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.challenge_run.active.is_some();
    state.push(Val::Bool(active));
    Ok(1)
}

/// The client getter has no "is a run active" test: it publishes the keystone
/// globals verbatim and never returns nothing. A fresh session holds level 0, an
/// empty affix list and `false`. The affix list fills four slots and stops at
/// the first zero slot. See docs/specs/challenge-mode-run.md, keystone globals.
fn active_keystone(state: &mut LuaState) -> LuaResult<u32> {
    let keystone = borrow_state(state)?.challenge_run.active_keystone.clone();
    state.push(Val::Num(f64::from(keystone.level)));
    let affixes = create_table(state);
    state.push(affixes);
    for (index, id) in keystone
        .affixes
        .iter()
        .take_while(|id| **id != 0)
        .enumerate()
    {
        super::helpers::set_table_array(state, affixes, index as i64 + 1, Val::Num(f64::from(*id)));
    }
    state.push(Val::Bool(keystone.charged));
    Ok(3)
}

fn death_count(state: &mut LuaState) -> LuaResult<u32> {
    let run = borrow_state(state)?.challenge_run.active.clone();
    let Some(run) = run else { return Ok(0) };
    state.push(Val::Num(f64::from(run.deaths)));
    state.push(Val::Num(run.time_lost));
    Ok(2)
}

fn map_table(state: &mut LuaState) -> LuaResult<u32> {
    let ids: Vec<_> = borrow_state(state)?
        .challenge_run
        .maps
        .keys()
        .copied()
        .collect();
    let result = create_table(state);
    state.push(result);
    for (index, id) in ids.iter().enumerate() {
        super::helpers::set_table_array(state, result, index as i64 + 1, Val::Num(f64::from(*id)));
    }
    Ok(1)
}

fn map_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let map = borrow_state(state)?.challenge_run.maps.get(&id).cloned();
    let Some(map) = map else { return Ok(0) };
    let name = create_string(state, &map.name);
    state.push(name);
    for value in [
        Some(f64::from(map.id)),
        Some(map.time_limit),
        map.texture,
        Some(map.background),
        Some(f64::from(map.map_id)),
    ] {
        state.push(value.map_or(Val::Nil, Val::Num));
    }
    Ok(6)
}
