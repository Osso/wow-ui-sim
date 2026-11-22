//! Social restrictions supplied by this simulator environment, not account policy.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let consumer_profile = matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever);
    let squelch_profile = matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::Mists
    );
    if !consumer_profile && !squelch_profile {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_SocialRestrictions")?;
    if consumer_profile {
        table_set_rust_fn_static(state, namespace, "IsMuted", is_muted)?;
        table_set_rust_fn_static(
            state,
            namespace,
            "IsAgeVerificationRestricted",
            is_age_verification_restricted,
        )?;
        table_set_rust_fn_static(state, namespace, "IsChatDisabled", is_chat_disabled)?;
    }
    if squelch_profile {
        table_set_rust_fn_static(state, namespace, "IsSquelched", is_squelched)?;
    }
    #[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
    if ACTIVE == ClientProfile::Retail {
        table_set_rust_fn_static(state, namespace, "IsFriendsDisabled", is_friends_disabled)?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        builder.set_function("SetChatDisabled", set_chat_disabled)
    } else {
        Ok(builder)
    }
}

// INFERRED R88ek (native evidence pending): supply this host scenario, not account policy.
fn set_chat_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let disabled = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.social_restrictions.chat_disabled = disabled;
    Ok(0)
}

fn is_muted(state: &mut LuaState) -> LuaResult<u32> {
    let muted = borrow_state(state)?.social_restrictions.is_muted;
    state.push(Val::Bool(muted));
    Ok(1)
}

fn is_age_verification_restricted(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = borrow_state(state)?
        .social_restrictions
        .age_verification_restricted;
    state.push(Val::Bool(restricted));
    Ok(1)
}

fn is_chat_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let disabled = borrow_state(state)?.social_restrictions.chat_disabled;
    state.push(Val::Bool(disabled));
    Ok(1)
}

fn is_squelched(state: &mut LuaState) -> LuaResult<u32> {
    let squelched = borrow_state(state)?.social_restrictions.is_squelched;
    state.push(Val::Bool(squelched));
    Ok(1)
}

/// VERIFIED R88ap part A §4: one Boolean from settings byte, initially zero.
/// INFERRED (simulator mapping): `friends_disabled_byte` is the host stand-in for that byte.
#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn is_friends_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let disabled = borrow_state(state)?
        .social_restrictions
        .friends_disabled_byte
        != 0;
    state.push(Val::Bool(disabled));
    Ok(1)
}
