//! Latest collected transmog appearance for the current session.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Declared by `Blizzard_APIDocumentationGenerated/TransmogItemsDocumentation.lua`
/// on Retail/PTR (`:553`), WowForever (`:554`) and Mists/Era/Anniversary (`:524`).
/// The Wrath runtime cache ships no `Blizzard_APIDocumentationGenerated`, so the
/// getter stays unpublished there.
fn publishes_latest_appearance() -> bool {
    matches!(
        ACTIVE,
        ClientProfile::Retail
            | ClientProfile::Ptr
            | ClientProfile::WowForever
            | ClientProfile::Mists
            | ClientProfile::Era
            | ClientProfile::Anniversary
    )
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !publishes_latest_appearance() {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_TransmogCollection")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetLatestAppearance",
        get_latest_appearance,
    )
}

fn get_latest_appearance(state: &mut LuaState) -> LuaResult<u32> {
    let latest = borrow_state(state)?.latest_transmog_appearance;
    if let Some((visual_id, category_id)) = latest {
        state.push(Val::Num(visual_id as f64));
        state.push(Val::Num(category_id as f64));
        Ok(2)
    } else {
        Ok(0)
    }
}
