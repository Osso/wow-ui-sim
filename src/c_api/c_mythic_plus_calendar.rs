//! B22 live CalendarTime snapshots. Cached declarations may postdate 12.0.7.

use crate::c_api::helpers::set_table_array;
use crate::c_api::patch_12_0_7_inputs::{authenticate_arguments, integer_selector};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_api::state::{MythicPlusRun, MythicPlusWeeklyBest};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

/// Host-supplied fields; no timezone, date arithmetic or index conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarTime {
    pub month_day: i32,
    pub month: i32,
    pub weekday: i32,
    pub year: i32,
    pub hour: i32,
    pub minute: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MythicPlusMember {
    pub name: Option<String>,
    pub spec_id: i32,
    pub class_id: i32,
}

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetRunHistory", history)?;
    table_set_rust_fn_static(state, namespace, "GetWeeklyBestForMap", weekly)?;
    table_set_rust_fn_static(state, namespace, "GetSeasonBestForMap", season)
}

fn push_calendar(state: &mut LuaState, date: CalendarTime) -> Val {
    let table = create_table(state);
    state.push(table);
    for (key, value) in [
        ("monthDay", date.month_day),
        ("month", date.month),
        ("weekday", date.weekday),
        ("year", date.year),
        ("hour", date.hour),
        ("minute", date.minute),
    ] {
        table_set(state, table, key, Val::Num(f64::from(value)));
    }
    table
}

fn push_affixes(state: &mut LuaState, ids: &[i32]) -> Val {
    let array = create_table(state);
    state.push(array);
    for (index, id) in ids.iter().enumerate() {
        set_table_array(state, array, index as i64 + 1, Val::Num(f64::from(*id)));
    }
    array
}

fn push_members(state: &mut LuaState, members: &[MythicPlusMember]) -> Val {
    let array = create_table(state);
    state.push(array);
    for (index, member) in members.iter().enumerate() {
        let entry = create_table(state);
        state.push(entry);
        if let Some(name) = &member.name {
            let name = create_string(state, name);
            table_set(state, entry, "name", name);
        }
        table_set(state, entry, "specID", Val::Num(f64::from(member.spec_id)));
        table_set(
            state,
            entry,
            "classID",
            Val::Num(f64::from(member.class_id)),
        );
        set_table_array(state, array, index as i64 + 1, entry);
        state.pop();
    }
    array
}

fn push_run(state: &mut LuaState, run: &MythicPlusRun, date: CalendarTime) -> Val {
    let entry = create_table(state);
    state.push(entry);
    for (key, value) in [
        (
            "mapChallengeModeID",
            Val::Num(f64::from(run.map_challenge_mode_id)),
        ),
        ("level", Val::Num(f64::from(run.level))),
        ("completed", Val::Bool(run.completed)),
        ("season", Val::Num(f64::from(run.season))),
        ("runScore", Val::Num(run.run_score)),
        ("thisWeek", Val::Bool(run.this_week)),
        ("durationSec", Val::Num(f64::from(run.duration_sec))),
    ] {
        table_set(state, entry, key, value);
    }
    let calendar = push_calendar(state, date);
    table_set(state, entry, "completionDate", calendar);
    state.pop();
    entry
}

fn history(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state)?;
    // Existing history filter behavior is unchanged: flags are not modeled.
    let runs = borrow_state(state)?.mythic_plus.run_history.clone();
    let array = create_table(state);
    state.push(array);
    let dated_runs = runs
        .iter()
        .filter_map(|run| Some((run, run.completion_date?)));
    // INFERRED: undated host records are data gaps, omitted rather than fabricated.
    for (index, (run, date)) in dated_runs.enumerate() {
        let entry = push_run(state, run, date);
        set_table_array(state, array, index as i64 + 1, entry);
        state.pop();
    }
    Ok(1)
}

fn weekly(state: &mut LuaState) -> LuaResult<u32> {
    let args = authenticate_arguments(state)?;
    let Some(map) = integer_selector(args.first().copied().unwrap_or(Val::Nil)) else {
        return Ok(0);
    };
    let best = borrow_state(state)?
        .mythic_plus
        .weekly_best_per_map
        .get(&map)
        .cloned();
    // INFERRED: missing/undated map records return no values.
    let Some((best, date)) = best.and_then(|best| {
        let date = best.completion_date?;
        Some((best, date))
    }) else {
        return Ok(0);
    };
    state.push(Val::Num(f64::from(best.duration_sec)));
    state.push(Val::Num(f64::from(best.level)));
    push_calendar(state, date);
    push_affixes(state, &best.affix_ids);
    push_members(state, &best.members);
    state.push(Val::Num(best.score));
    Ok(6)
}

fn push_best(state: &mut LuaState, best: Option<&MythicPlusWeeklyBest>) {
    // INFERRED: an undated side is unavailable independently of the other side.
    let Some((best, date)) = best.and_then(|best| Some((best, best.completion_date?))) else {
        state.push(Val::Nil);
        return;
    };
    let table = create_table(state);
    state.push(table);
    for (key, number) in [
        ("durationSec", f64::from(best.duration_sec)),
        ("level", f64::from(best.level)),
        ("dungeonScore", best.score),
    ] {
        table_set(state, table, key, Val::Num(number));
    }
    let calendar = push_calendar(state, date);
    table_set(state, table, "completionDate", calendar);
    state.pop();
    let affixes = push_affixes(state, &best.affix_ids);
    table_set(state, table, "affixIDs", affixes);
    state.pop();
    let members = push_members(state, &best.members);
    table_set(state, table, "members", members);
    state.pop();
}

fn season(state: &mut LuaState) -> LuaResult<u32> {
    let args = authenticate_arguments(state)?;
    let map = integer_selector(args.first().copied().unwrap_or(Val::Nil));
    let best = {
        let sim = borrow_state(state)?;
        map.and_then(|map| sim.mythic_plus.season_best_per_map.get(&map).cloned())
    };
    let (intime, overtime) = best.unwrap_or_default();
    push_best(state, intime.as_ref());
    push_best(state, overtime.as_ref());
    Ok(2)
}
