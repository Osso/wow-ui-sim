//! VERIFIED: pr46-49-decomp-answers.md, R88ba ba1-ba4 (Retail 12.1.5).
//! No arguments; one non-nil Boolean. The native CVar callback publishes the status event.
//! INFERRED: the image's feature-policy byte is represented by the constant below.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

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
    crate::event::dispatch_event_now(state, "SOCIAL_UI_SYSTEM_STATUS_UPDATED", &[])
}

fn is_system_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let cvar = borrow_state(state)?
        .cvars
        .cached_integer("socialUIEnabledClient");
    state.push(Val::Bool(cvar.unwrap_or(0) != 0));
    Ok(1)
}
