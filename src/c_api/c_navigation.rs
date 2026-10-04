//! Retail 12.0.5 caller-authorized navigation query over explicit host selection.

use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Navigation")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetNearestPartyMemberToken",
        get_nearest_party_member_token,
    )
}

fn authorize_navigation_caller(state: &LuaState) -> LuaResult<()> {
    let active_depth = state.ci.saturating_add(1);
    let addon_caller = state
        .call_stack
        .iter()
        .take(active_depth)
        .any(|info| info.taint.is_some());
    if addon_caller {
        return Err(runtime_error(
            "C_Navigation.GetNearestPartyMemberToken cannot be called by addons",
        ));
    }
    Ok(())
}

fn get_nearest_party_member_token(state: &mut LuaState) -> LuaResult<u32> {
    authorize_navigation_caller(state)?;
    // INFERRED: absent host selection is an explicit failure, not a nil/default token.
    let token = borrow_state(state)?
        .nearest_party_member_token
        .clone()
        .ok_or_else(|| {
            runtime_error(
                "C_Navigation.GetNearestPartyMemberToken has no host-selected party token",
            )
        })?;
    let result = create_string(state, &token);
    state.push(result);
    Ok(1)
}
