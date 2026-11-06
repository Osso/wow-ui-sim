//! Modeled external-event promotion state.
//!
//! Pinned `Blizzard_APIDocumentationGenerated/ExternalEventURLDocumentation.lua:5`
//! declares the `C_ExternalEventURL` namespace on Retail, PTR, Mists, Era and
//! Anniversary (`HasURL` at :11, `IsNew` at :20, `LaunchURL` at :29; PTR also
//! declares `CanShow` at :11). Wrath publishes no such documentation file.
//! WowForever publishes its own modeled namespace from
//! `c_api::forever_startup` and is not registered here.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Host-supplied promotion state. The default describes no promoted event in the
/// simulator scenario; it is not evidence of a native default.
#[derive(Debug, Clone, Default)]
pub struct ExternalEventUrlState {
    /// Promotion URL the host made available, if any.
    pub url: Option<String>,
    /// Host-supplied newness for the supplied promotion.
    pub is_new: bool,
    /// Host-supplied visibility for the PTR `CanShow` consumer.
    pub can_show: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail
            | ClientProfile::Ptr
            | ClientProfile::Mists
            | ClientProfile::Era
            | ClientProfile::Anniversary
    ) {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_ExternalEventURL")?;
    table_set_rust_fn_static(state, namespace, "HasURL", has_url)?;
    table_set_rust_fn_static(state, namespace, "IsNew", is_new)?;
    table_set_rust_fn_static(state, namespace, "LaunchURL", launch_url)?;
    if ACTIVE == ClientProfile::Ptr {
        table_set_rust_fn_static(state, namespace, "CanShow", can_show)?;
    }
    Ok(())
}

fn has_url(state: &mut LuaState) -> LuaResult<u32> {
    let supplied = borrow_state(state)?.external_event_url.url.is_some();
    state.push(Val::Bool(supplied));
    Ok(1)
}

fn is_new(state: &mut LuaState) -> LuaResult<u32> {
    let is_new = borrow_state(state)?.external_event_url.is_new;
    state.push(Val::Bool(is_new));
    Ok(1)
}

fn can_show(state: &mut LuaState) -> LuaResult<u32> {
    let can_show = borrow_state(state)?.external_event_url.can_show;
    state.push(Val::Bool(can_show));
    Ok(1)
}

/// Simulator policy: record the supplied URL in `SimState.last_launched_url`,
/// the same way the `LaunchURL` global does. No browser is opened, and an empty
/// event state records nothing. Native launch and failure behavior is unknown.
fn launch_url(state: &mut LuaState) -> LuaResult<u32> {
    let url = borrow_state(state)?.external_event_url.url.clone();
    if let Some(url) = url {
        borrow_state_mut(state)?.last_launched_url = Some(url);
    }
    Ok(0)
}
