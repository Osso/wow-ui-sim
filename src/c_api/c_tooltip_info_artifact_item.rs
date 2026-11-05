//! WM94 §8: `GetArtifactItem` composes the Artifact UI's current item identity.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:43` declare the getter.
//! WM94 establishes the source (`C_ArtifactUI.GetArtifactItemID`, that is the
//! viewed artifact) and that no equipped-item fallback exists. The artifact
//! request's own descriptor-preparation branch and its additional/suppressed
//! lines are UNKNOWN, so the producer publishes the ordinary item lines and
//! does not add artifact-only content.

use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetArtifactItem", get_artifact_item)
}

fn get_artifact_item(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = borrow_state(state)?
        .viewed_artifact
        .info
        .as_ref()
        .map(|info| info.item_id)
        .filter(|item_id| *item_id > 0)
        .and_then(|item_id| u32::try_from(item_id).ok());
    let item = item_id.and_then(crate::items::get_item);
    let Some((item_id, item)) = item_id.zip(item) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = tooltip_for_item_context(state, item_id, Some(item));
    state.push(tooltip);
    Ok(1)
}
