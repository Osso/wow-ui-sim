//! Actual completion-tier snapshots, separate from repeated vault threshold rows.
use crate::lua_api::methods::{borrow_state, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityTierProgress {
    pub activity_tier_id: u32,
    pub difficulty: u32,
    pub num_points: u32,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_WeeklyRewards")?;
    table_set_rust_fn_static(state, ns, "GetSortedProgressForActivity", progress)
}

fn sort_progress(rows: &[ActivityTierProgress], combine: bool) -> Vec<(u32, u32, u64)> {
    let mut result = if combine {
        let mut groups = BTreeMap::<u32, (u32, u64)>::new();
        for row in rows {
            let group = groups
                .entry(row.difficulty)
                .or_insert((row.activity_tier_id, 0));
            // INFERRED representative tier ID: lowest ID at this difficulty.
            group.0 = group.0.min(row.activity_tier_id);
            group.1 += u64::from(row.num_points);
        }
        groups
            .into_iter()
            .map(|(difficulty, (id, points))| (id, difficulty, points))
            .collect::<Vec<_>>()
    } else {
        rows.iter()
            .map(|r| (r.activity_tier_id, r.difficulty, u64::from(r.num_points)))
            .collect()
    };
    // Cached AddWorldRunsToTooltip requires descending difficulty; ties use ID (INFERRED).
    result.sort_by_key(|(id, difficulty, _)| (std::cmp::Reverse(*difficulty), *id));
    result
}
fn progress(state: &mut LuaState) -> LuaResult<u32> {
    let kind = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(kind) = kind else {
        return Err(rilua::runtime_error("activity type must be a number"));
    };
    if !kind.is_finite() || kind.fract() != 0.0 || kind < i32::MIN as f64 || kind > i32::MAX as f64
    {
        return Err(rilua::runtime_error("activity type must be an i32 integer"));
    }
    let combine = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    let Val::Bool(combine) = combine else {
        return Err(rilua::runtime_error(
            "combineSharedDifficulty must be a boolean",
        ));
    };
    let rows = {
        let sim = borrow_state(state)?;
        sort_progress(
            sim.weekly_reward_progress
                .get(&(kind as i32))
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            combine,
        )
    };
    let result = create_table(state);
    state.push(result);
    for (index, (id, difficulty, points)) in rows.into_iter().enumerate() {
        let row = create_table(state);
        state.push(row);
        table_set(state, row, "activityTierID", Val::Num(f64::from(id)));
        table_set(state, row, "difficulty", Val::Num(f64::from(difficulty)));
        table_set(state, row, "numPoints", Val::Num(points as f64));
        super::helpers::set_table_array(state, result, index as i64 + 1, row);
        state.pop();
    }
    Ok(1)
}
