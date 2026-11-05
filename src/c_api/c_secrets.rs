//! Spell aura classification and explicit stat/cooldown output policies.

#[cfg(any(feature = "aura-containers", feature = "retail-12-0-0"))]
use super::ensure_namespace;
#[cfg(feature = "retail-12-0-0")]
#[path = "secrets_queries.rs"]
mod queries;
#[cfg(feature = "retail-12-0-5")]
use crate::lua_api::methods::borrow_state;
#[cfg(any(feature = "aura-containers", feature = "retail-12-0-5"))]
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[cfg(feature = "aura-containers")]
use sim_data_profile::spell_aura_secrecy as attributes;

#[cfg(feature = "aura-containers")]
pub(crate) const NEVER_SECRET: i32 = 0;
#[cfg(feature = "aura-containers")]
pub(crate) const ALWAYS_SECRET: i32 = 1;
#[cfg(feature = "aura-containers")]
const CONTEXTUALLY_SECRET: i32 = 2;

#[cfg(any(feature = "aura-containers", feature = "retail-12-0-0"))]
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Secrets")?;
    #[cfg(feature = "retail-12-0-0")]
    queries::register(state, namespace)?;
    #[cfg(feature = "client-wowforever")]
    {
        table_set_rust_fn_static(
            state,
            namespace,
            "ShouldUnitIdentityBeSecret",
            forever_unit_identity_is_secret,
        )?;
        table_set_rust_fn_static(
            state,
            namespace,
            "ShouldAurasBeSecret",
            should_auras_be_secret,
        )?;
    }
    #[cfg(feature = "aura-containers")]
    table_set_rust_fn_static(
        state,
        namespace,
        "GetSpellAuraSecrecy",
        get_spell_aura_secrecy,
    )?;
    #[cfg(feature = "retail-12-0-5")]
    table_set_rust_fn_static(
        state,
        namespace,
        "ShouldUnitStatsBeSecret",
        should_unit_stats_be_secret,
    )?;
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    table_set_rust_fn_static(
        state,
        namespace,
        "ShouldCooldownsBeSecret",
        should_cooldowns_be_secret,
    )?;
    Ok(())
}

#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
fn should_auras_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = super::unit_aura_access::auras_restricted(state)?;
    state.push(Val::Bool(restricted));
    Ok(1)
}

#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
fn should_cooldowns_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = {
        let sim = borrow_state(state)?;
        super::charge_state::cooldowns_are_restricted(&sim)
    };
    state.push(Val::Bool(restricted));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn forever_unit_identity_is_secret(state: &mut LuaState) -> LuaResult<u32> {
    let value =
        rilua::table_security::unwrap_secret(state, crate::lua_bridge::stack_val(state, 1))?;
    let unit = crate::lua_api::methods::val_to_string(state, value).unwrap_or_default();
    let secret = crate::lua_api::globals::unit_misc::unit_identity_is_secret(state, &unit)?;
    state.push(Val::Bool(secret));
    Ok(1)
}

/// Push only a concrete Rust-computed stat result, never an arbitrary Lua value.
/// Host production preserves caller taint; profiles without this policy stay plain.
pub(crate) fn push_stat_number(state: &mut LuaState, number: f64) -> LuaResult<()> {
    #[cfg(feature = "retail-12-0-5")]
    let restricted = borrow_state(state)?.unit_stats_restricted;
    #[cfg(feature = "retail-12-0-5")]
    let value = if restricted {
        rilua::table_security::wrap_host_secret_number(state, number)
    } else {
        Val::Num(number)
    };
    #[cfg(not(feature = "retail-12-0-5"))]
    let value = Val::Num(number);
    state.push(value);
    Ok(())
}

#[cfg(feature = "retail-12-0-5")]
fn should_unit_stats_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = borrow_state(state)?.unit_stats_restricted;
    state.push(Val::Bool(restricted));
    Ok(1)
}

#[cfg(feature = "aura-containers")]
fn get_spell_aura_secrecy(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = super::c_spell::numeric_spell_id(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let secrecy = classify_aura_secrecy(spell_id)?;
    state.push(Val::Num(f64::from(secrecy)));
    Ok(1)
}

#[cfg(feature = "aura-containers")]
pub(crate) fn classify_aura_secrecy(spell_id: u32) -> LuaResult<i32> {
    let flags = attributes::aura_flags(spell_id);
    Ok(if flags & attributes::AURA_ALWAYS_SECRET != 0 {
        ALWAYS_SECRET
    } else if flags & attributes::AURA_NEVER_SECRET != 0 {
        NEVER_SECRET
    } else {
        CONTEXTUALLY_SECRET
    })
}
