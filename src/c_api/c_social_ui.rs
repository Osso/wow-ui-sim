//! VERIFIED: pr46-49-decomp-answers.md, R88ba ba1-ba4 (Retail 12.1.5).
//! No arguments; one non-nil Boolean. The native CVar callback publishes the status event.
//! INFERRED: the image's feature-policy byte is represented by the constant below.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// INFERRED simulator mapping: the native feature-policy byte is fixed at one.
const SOCIAL_UI_FEATURE_POLICY_ENABLED: bool = true;

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_SocialUI")?;
    table_set_rust_fn_static(state, namespace, "IsSystemEnabled", is_system_enabled)
}

pub(crate) fn cvar_set_callback(state: &mut LuaState) -> LuaResult<()> {
    dispatch_event_now(state, "SOCIAL_UI_SYSTEM_STATUS_UPDATED", &[])
}

fn is_system_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let cvar_enabled = borrow_state(state)?
        .cvars
        .cached_integer("socialUIEnabledClient")
        .unwrap_or(0)
        != 0;
    state.push(Val::Bool(cvar_enabled && SOCIAL_UI_FEATURE_POLICY_ENABLED));
    Ok(1)
}
