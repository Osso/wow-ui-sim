//! Social restrictions supplied by this simulator environment, not account policy.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::Mists
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_SocialRestrictions")?;
    #[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
    table_set_rust_fn_static(state, namespace, "IsFriendsDisabled", is_friends_disabled)?;
    table_set_rust_fn_static(state, namespace, "IsSquelched", is_squelched)
}

fn is_squelched(state: &mut LuaState) -> LuaResult<u32> {
    let squelched = borrow_state(state)?.social_restrictions.is_squelched;
    state.push(Val::Bool(squelched));
    Ok(1)
}

/// VERIFIED R88ap part A §4: one Boolean from settings byte, initially zero.
#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn is_friends_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let disabled = borrow_state(state)?
        .social_restrictions
        .friends_disabled_byte
        != 0;
    state.push(Val::Bool(disabled));
    Ok(1)
}
