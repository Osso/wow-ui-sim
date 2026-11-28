use super::project;
use super::{borrow_state_mut, read_input, register_methods};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WeeklyRewardStatus {
    rewards_generated: bool,
    weekly_chest_retired: bool,
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation = read_input(state)?;
    borrow_state_mut(state)?
        .scenario_observations
        .weekly_reward_status = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_WeeklyRewards",
        &[
            // WeeklyRewardsDocumentation.lua:205
            ("HasGeneratedRewards", rewards_generated),
            // WeeklyRewardsDocumentation.lua:223
            ("IsWeeklyChestRetired", weekly_chest_retired),
        ],
    )
}

fn rewards_generated(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.weekly_reward_status.rewards_generated)
    })
}

fn weekly_chest_retired(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.weekly_reward_status.weekly_chest_retired)
    })
}
