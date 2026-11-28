use super::borrow_state;
use super::create_string;
use super::project;
use super::{borrow_state_mut, read_input, register_methods};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct HousingObservation {
    basic_exterior_hovered: bool,
    basic_hovered_decor_guid: Option<String>,
    cleanup_hovered_decor_guid: Option<String>,
    customize_hovered_decor_guid: Option<String>,
    expert_exterior_hovered: bool,
    expert_hovered_decor_guid: Option<String>,
    selected_room_guid: Option<String>,
    selected_stairwell_room_count: u32,
    preview_dye_item_count: u32,
    in_other_players_plot: bool,
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation: HousingObservation = read_input(state)?;
    if [
        &observation.basic_hovered_decor_guid,
        &observation.cleanup_hovered_decor_guid,
        &observation.customize_hovered_decor_guid,
        &observation.expert_hovered_decor_guid,
        &observation.selected_room_guid,
    ]
    .into_iter()
    .any(|value| value.as_ref().is_some_and(String::is_empty))
    {
        return Err(rilua::runtime_error(
            "housing observation GUID must not be empty",
        ));
    }
    borrow_state_mut(state)?
        .scenario_observations
        .housing_observation = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_housingbasicmode(state)?;
    register_housingcleanupmode(state)?;
    register_housingcustomizemode(state)?;
    register_housingexpertmode(state)?;
    register_housinglayout(state)?;
    register_housingneighborhood(state)?;
    Ok(())
}

fn register_housingbasicmode(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_HousingBasicMode",
        &[
            // HousingBasicModeUIDocumentation.lua:88
            ("IsHouseExteriorHovered", basic_exterior_hovered),
            // HousingBasicModeUIDocumentation.lua:108
            ("IsHoveringDecor", basic_hovered_decor_guid),
        ],
    )
}

fn register_housingcleanupmode(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_HousingCleanupMode",
        &[
            // HousingCleanupModeUIDocumentation.lua:20
            ("IsHoveringDecor", cleanup_hovered_decor_guid),
        ],
    )
}

fn register_housingcustomizemode(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_HousingCustomizeMode",
        &[
            // HousingCustomizeModeUIDocumentation.lua:267
            ("IsHoveringDecor", customize_hovered_decor_guid),
            // HousingCustomizeModeUIDocumentation.lua:134
            ("GetNumDyesToSpendOnSelectedDecor", preview_dye_item_count),
        ],
    )
}

fn register_housingexpertmode(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_HousingExpertMode",
        &[
            // HousingExpertModeUIDocumentation.lua:91
            ("IsHouseExteriorHovered", expert_exterior_hovered),
            // HousingExpertModeUIDocumentation.lua:111
            ("IsHoveringDecor", expert_hovered_decor_guid),
        ],
    )
}

fn register_housinglayout(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_HousingLayout",
        &[
            // HousingLayoutUIDocumentation.lua:152
            ("GetSelectedRoom", selected_room_guid),
            // HousingLayoutUIDocumentation.lua:163
            (
                "GetSelectedStairwellRoomCount",
                selected_stairwell_room_count,
            ),
        ],
    )
}

fn register_housingneighborhood(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_HousingNeighborhood",
        &[
            // HousingNeighborhoodUIDocumentation.lua:183
            ("IsPlayerInOtherPlayersPlot", in_other_players_plot),
        ],
    )
}

fn basic_exterior_hovered(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.housing_observation.basic_exterior_hovered)
    })
}

fn basic_hovered_decor_guid(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.housing_observation.basic_hovered_decor_guid.is_some())
    })
}

fn cleanup_hovered_decor_guid(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.housing_observation.cleanup_hovered_decor_guid.is_some())
    })
}

fn customize_hovered_decor_guid(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.housing_observation.customize_hovered_decor_guid.is_some())
    })
}

fn expert_exterior_hovered(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.housing_observation.expert_exterior_hovered)
    })
}

fn expert_hovered_decor_guid(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.housing_observation.expert_hovered_decor_guid.is_some())
    })
}

fn selected_room_guid(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .scenario_observations
        .housing_observation
        .selected_room_guid
        .clone();
    let Some(value) = value else { return Ok(0) };
    let result = create_string(state, &value);
    state.push(result);
    Ok(1)
}

fn selected_stairwell_room_count(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Num(f64::from(
            o.housing_observation.selected_stairwell_room_count,
        ))
    })
}

fn preview_dye_item_count(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Num(f64::from(o.housing_observation.preview_dye_item_count))
    })
}

fn in_other_players_plot(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.housing_observation.in_other_players_plot)
    })
}
