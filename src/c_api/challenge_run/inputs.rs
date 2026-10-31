use super::{ActiveRun, MapInfo};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn field<T: FromStack>(state: &mut LuaState, row: Val, name: &str) -> LuaResult<T> {
    let value = table_get(state, row, name);
    decode(state, value)
}

pub(super) fn decode<T: FromStack>(state: &mut LuaState, value: Val) -> LuaResult<T> {
    let top = state.top;
    state.push(value);
    let result = T::from_stack(state, -1);
    state.top = top;
    result
}

pub(super) fn array(state: &LuaState, value: Val) -> LuaResult<Vec<Val>> {
    let Val::Table(reference) = value else {
        return Err(rilua::runtime_error("expected a supplied array"));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| rilua::runtime_error("missing supplied array"))?;
    Ok((1..=table.len(&state.gc.string_arena))
        .map(|i| table.get_int(i as i64))
        .collect())
}

pub(super) fn nonnegative(value: f64) -> LuaResult<f64> {
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(rilua::runtime_error(
            "supplied seconds must be finite and nonnegative",
        ))
    }
}

fn map_error(name: &str, error: rilua::LuaError) -> rilua::LuaError {
    let message = error.to_string().replace(" at argument -1", "");
    rilua::runtime_error(format!("mapInfo.{name}: {message}"))
}

fn map_field<T: FromStack>(state: &mut LuaState, row: Val, name: &str) -> LuaResult<T> {
    field(state, row, name).map_err(|error| map_error(name, error))
}

pub(super) fn set_map(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let row = stack_val(state, 2);
    if !matches!(row, Val::Table(_)) {
        return Err(rilua::runtime_error(format!(
            "mapInfo: expected table, got {}",
            row.type_name()
        )));
    }
    let time_limit = map_field(state, row, "timeLimit")?;
    let time_limit = nonnegative(time_limit).map_err(|error| map_error("timeLimit", error))?;
    let map = MapInfo {
        name: map_field(state, row, "name")?,
        id: map_field(state, row, "id")?,
        time_limit,
        texture: map_field(state, row, "texture")?,
        background: map_field(state, row, "backgroundTexture")?,
        map_id: map_field(state, row, "mapID")?,
    };
    borrow_state_mut(state)?.challenge_run.maps.insert(id, map);
    Ok(0)
}

pub(super) fn start_run(state: &mut LuaState) -> LuaResult<u32> {
    if borrow_state(state)?.challenge_run.active.is_some() {
        return Err(rilua::runtime_error(
            "clear the current challenge run before starting another",
        ));
    }
    let row = stack_val(state, 1);
    let affixes = table_get(state, row, "affixIDs");
    let affixes = array(state, affixes)?
        .into_iter()
        .map(|v| decode(state, v))
        .collect::<LuaResult<Vec<i32>>>()?;
    let run = ActiveRun {
        map_id: field(state, row, "mapChallengeModeID")?,
        event_map_id: field(state, row, "eventMapID")?,
        level: field(state, row, "level")?,
        affixes,
        charged: field(state, row, "wasActiveKeystoneCharged")?,
        deaths: field(state, row, "numDeaths")?,
        time_lost: nonnegative(field(state, row, "timeLost")?)?,
    };
    let event_id = run.event_map_id;
    borrow_state_mut(state)?.challenge_run.active = Some(run);
    crate::c_api::restricted_actions::set_challenge_mode(state, true)?;
    dispatch_event_now(
        state,
        "CHALLENGE_MODE_START",
        &[Val::Num(f64::from(event_id))],
    )?;
    Ok(0)
}

fn remove_run(state: &mut LuaState) -> LuaResult<Option<i32>> {
    let (run, timers) = {
        let mut sim = borrow_state_mut(state)?;
        let run = sim.challenge_run.active.take();
        let timers = std::mem::take(&mut sim.challenge_run.timers);
        sim.challenge_run.criteria.clear();
        sim.scenario.steps.clear();
        sim.scenario.in_scenario = false;
        (run, timers)
    };
    for id in timers.keys() {
        dispatch_event_now(state, "WORLD_STATE_TIMER_STOP", &[Val::Num(f64::from(*id))])?;
    }
    Ok(run.map(|run| run.event_map_id))
}

pub(super) fn clear_run(state: &mut LuaState) -> LuaResult<u32> {
    if let Some(event_id) = remove_run(state)? {
        crate::c_api::restricted_actions::set_challenge_mode(state, false)?;
        dispatch_event_now(
            state,
            "CHALLENGE_MODE_RESET",
            &[Val::Num(f64::from(event_id))],
        )?;
    }
    Ok(0)
}

pub(super) fn complete_run(state: &mut LuaState) -> LuaResult<u32> {
    let row = stack_val(state, 1);
    let completion = decode_completion(state, row)?;
    borrow_state_mut(state)?.challenge_completion = completion;
    remove_run(state)?;
    crate::c_api::restricted_actions::set_challenge_mode(state, false)?;
    dispatch_event_now(state, "CHALLENGE_MODE_COMPLETED", &[])?;
    Ok(0)
}

fn decode_completion(
    state: &mut LuaState,
    row: Val,
) -> LuaResult<crate::c_api::c_challenge_mode::ChallengeCompletionInfo> {
    use crate::c_api::c_challenge_mode::{ChallengeCompletionInfo, ChallengeCompletionMember};
    let members = table_get(state, row, "members");
    let members = array(state, members)?
        .into_iter()
        .map(|member| {
            Ok(ChallengeCompletionMember {
                member_guid: field(state, member, "memberGUID")?,
                name: field(state, member, "name")?,
            })
        })
        .collect::<LuaResult<Vec<_>>>()?;
    Ok(ChallengeCompletionInfo {
        map_challenge_mode_id: field(state, row, "mapChallengeModeID")?,
        level: field(state, row, "level")?,
        time: nonnegative(field(state, row, "time")?)?,
        on_time: field(state, row, "onTime")?,
        keystone_upgrade_levels: field(state, row, "keystoneUpgradeLevels")?,
        practice_run: field(state, row, "practiceRun")?,
        old_overall_dungeon_score: field(state, row, "oldOverallDungeonScore")?,
        new_overall_dungeon_score: field(state, row, "newOverallDungeonScore")?,
        is_map_record: field(state, row, "isMapRecord")?,
        is_affix_record: field(state, row, "isAffixRecord")?,
        is_eligible_for_score: field(state, row, "isEligibleForScore")?,
        members,
    })
}
