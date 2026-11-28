// C_TransmogCollection.HasFavorites: Retail TransmogItemsDocumentation.lua:690; Forever TransmogItemsDocumentation.lua:700.
// C_ArtifactUI.DoesEquippedArtifactHaveAnyRelicsSlotted: ArtifactUIDocumentation.lua:120.
use super::{borrow_state_mut, project, read_input, register_methods};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::BTreeSet;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CollectionObservation {
    equipped_artifact_relic_item_ids: BTreeSet<u32>,
    favorite_appearance_ids: BTreeSet<u32>,
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation = read_input(state)?;
    borrow_state_mut(state)?
        .scenario_observations
        .collection_observation = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_ArtifactUI",
        &[("DoesEquippedArtifactHaveAnyRelicsSlotted", relics_slotted)],
    )?;
    register_methods(
        state,
        "C_TransmogCollection",
        &[("HasFavorites", has_favorites)],
    )
}

fn relics_slotted(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(
            !o.collection_observation
                .equipped_artifact_relic_item_ids
                .is_empty(),
        )
    })
}

fn has_favorites(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(!o.collection_observation.favorite_appearance_ids.is_empty())
    })
}
