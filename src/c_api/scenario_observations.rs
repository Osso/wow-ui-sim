//! Typed host observations for otherwise unmodeled client services.
//! See docs/admin-api/scenario-observations.md for the input contracts.
//! The empty observation is not a claim about native login state.
use super::{ensure_namespace, helpers::set_table_array};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, create_table};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, RustFn, Val, runtime_error};
use serde::de::DeserializeOwned;
mod collection_observation;
mod content_availability;
mod gamepad_configuration;
mod group_activity;
mod housing_observation;
mod minimap_presentation;
mod player_features;
mod pvp_progress;
mod weekly_reward_status;

#[derive(Debug, Default)]
pub(crate) struct Observations {
    player_features: player_features::PlayerFeatures,
    group_activity: group_activity::GroupActivity,
    pvp_progress: pvp_progress::PvpProgress,
    weekly_reward_status: weekly_reward_status::WeeklyRewardStatus,
    content_availability: content_availability::ContentAvailability,
    minimap_presentation: minimap_presentation::MinimapPresentation,
    housing_observation: housing_observation::HousingObservation,
    collection_observation: collection_observation::CollectionObservation,
    gamepad_configuration_ids: Vec<gamepad_configuration::ConfigId>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    player_features::register(state)?;
    group_activity::register(state)?;
    pvp_progress::register(state)?;
    weekly_reward_status::register(state)?;
    content_availability::register(state)?;
    minimap_presentation::register(state)?;
    housing_observation::register(state)?;
    collection_observation::register(state)?;
    gamepad_configuration::register(state)?;
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    let builder = builder.set_function("SetPlayerFeatureState", player_features::supply)?;
    let builder = builder.set_function("SetGroupActivityState", group_activity::supply)?;
    let builder = builder.set_function("SetPvpProgressState", pvp_progress::supply)?;
    let builder = builder.set_function("SetWeeklyRewardStatus", weekly_reward_status::supply)?;
    let builder =
        builder.set_function("SetWorldContentAvailability", content_availability::supply)?;
    let builder =
        builder.set_function("SetMinimapPresentationState", minimap_presentation::supply)?;
    let builder =
        builder.set_function("SetHousingEditorObservation", housing_observation::supply)?;
    register_catalog_admin(builder)
}

fn register_catalog_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetCollectionObservation", collection_observation::supply)?
        .set_function("SetGamePadConfigurationIDs", gamepad_configuration::supply)
}

fn read_input<T: DeserializeOwned>(state: &mut LuaState) -> LuaResult<T> {
    let json = String::from_stack(state, 1)?;
    serde_json::from_str(&json)
        .map_err(|error| runtime_error(format!("invalid scenario observation: {error}")))
}

fn register_methods(
    state: &mut LuaState,
    namespace: &str,
    methods: &[(&str, RustFn)],
) -> LuaResult<()> {
    let ns = ensure_namespace(state, namespace)?;
    for &(name, handler) in methods {
        table_set_rust_fn_static(state, ns, name, handler)?;
    }
    Ok(())
}

fn project(state: &mut LuaState, value: impl FnOnce(&Observations) -> Val) -> LuaResult<u32> {
    let result = value(&borrow_state(state)?.scenario_observations);
    state.push(result);
    Ok(1)
}

fn numbers(state: &mut LuaState, values: Vec<u32>) -> LuaResult<u32> {
    let result = create_table(state);
    for (index, value) in values.into_iter().enumerate() {
        set_table_array(state, result, index as i64 + 1, Val::Num(f64::from(value)));
    }
    state.push(result);
    Ok(1)
}

#[cfg(test)]
mod tests;
