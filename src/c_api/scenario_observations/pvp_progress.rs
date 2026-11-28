use super::project;
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
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation = read_input(state)?;
    borrow_state_mut(state)?.scenario_observations.pvp_progress = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
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
