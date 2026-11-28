use super::project;
use super::{borrow_state_mut, read_input, register_methods};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct MinimapPresentation {
    rotation_ignored: bool,
    tracking_battle_pets: bool,
    quest_poi_world_effect_count: u32,
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation = read_input(state)?;
    borrow_state_mut(state)?
        .scenario_observations
        .minimap_presentation = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_Minimap",
        &[
            // MinimapDocumentation.lua:166
            ("IsRotateMinimapIgnored", rotation_ignored),
            // MinimapDocumentation.lua:184
            ("IsTrackingBattlePets", tracking_battle_pets),
            // MinimapDocumentation.lua:52
            ("GetNumQuestPOIWorldEffects", quest_poi_world_effect_count),
        ],
    )
}

fn rotation_ignored(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.minimap_presentation.rotation_ignored)
    })
}

fn tracking_battle_pets(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.minimap_presentation.tracking_battle_pets)
    })
}

fn quest_poi_world_effect_count(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Num(f64::from(
            o.minimap_presentation.quest_poi_world_effect_count,
        ))
    })
}
