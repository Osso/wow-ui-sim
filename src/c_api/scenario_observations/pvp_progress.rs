// C_PvP.GetBrawlSoloRBGMinItemLevel: PvpInfoDocumentation.lua:352.
// C_PvP.GetWeeklyChestInfo: PvpInfoDocumentation.lua:835.
use super::{borrow_state, project};
use super::{borrow_state_mut, read_input, register_methods};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PvpProgress {
    arena_skirmish_won_today: bool,
    training_arena_won_today: bool,
    training_bg_won_today: bool,
    active_match_registered: bool,
    war_mode_enabled: bool,
    weekly_reward_achieved: bool,
    last_week_reward_achieved: bool,
    last_week_reward_claimed: bool,
    tier_max_from_wins: u32,
    solo_rbg_brawl_min_item_level: u32,
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation = read_input(state)?;
    borrow_state_mut(state)?.scenario_observations.pvp_progress = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_reward_queries(state)?;
    register_methods(
        state,
        "C_PvP",
        &[
            // PvpInfoDocumentation.lua:859
            ("HasArenaSkirmishWinToday", arena_skirmish_won_today),
            // PvpInfoDocumentation.lua:878
            (
                "HasRandomTrainingGroundArenaWinToday",
                training_arena_won_today,
            ),
            // PvpInfoDocumentation.lua:887
            ("HasRandomTrainingGroundBGWinToday", training_bg_won_today),
            // PvpInfoDocumentation.lua:905
            ("IsActiveMatchRegistered", active_match_registered),
            // PvpInfoDocumentation.lua:1143
            ("IsWarModeFeatureEnabled", war_mode_enabled),
        ],
    )
}

fn arena_skirmish_won_today(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.pvp_progress.arena_skirmish_won_today)
    })
}

fn training_arena_won_today(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.pvp_progress.training_arena_won_today)
    })
}

fn training_bg_won_today(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| Val::Bool(o.pvp_progress.training_bg_won_today))
}

fn active_match_registered(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| Val::Bool(o.pvp_progress.active_match_registered))
}

fn war_mode_enabled(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| Val::Bool(o.pvp_progress.war_mode_enabled))
}

fn register_reward_queries(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_PvP",
        &[
            ("GetWeeklyChestInfo", weekly_chest),
            ("GetBrawlSoloRBGMinItemLevel", brawl_min_item_level),
        ],
    )
}

fn weekly_chest(state: &mut LuaState) -> LuaResult<u32> {
    let outputs = {
        let sim = borrow_state(state)?;
        let progress = &sim.scenario_observations.pvp_progress;
        [
            Val::Bool(progress.weekly_reward_achieved),
            Val::Bool(progress.last_week_reward_achieved),
            Val::Bool(progress.last_week_reward_claimed),
            Val::Num(f64::from(progress.tier_max_from_wins)),
        ]
    };
    for output in outputs {
        state.push(output);
    }
    Ok(4)
}

fn brawl_min_item_level(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Num(f64::from(o.pvp_progress.solo_rbg_brawl_min_item_level))
    })
}
