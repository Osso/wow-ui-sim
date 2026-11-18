//! VERIFIED: pr46-49-decomp-answers.md, R88ap part B §6 (Retail 12.1.5).
//! No system-ID argument; one Boolean. UNKNOWN writers/events remain host inputs.
//! INFERRED (simulator mapping): one optional Boolean stands in for the native no-owner
//! default-plus-policy-bytes path and the absent-subordinate false path; `None` yields false.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_SocialUI")?;
    table_set_rust_fn_static(state, namespace, "IsSystemEnabled", is_system_enabled)
}

fn is_system_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?
        .social_system_policy_enabled
        .unwrap_or(false);
    state.push(Val::Bool(enabled));
    Ok(1)
}
