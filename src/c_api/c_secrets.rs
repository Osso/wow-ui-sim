//! Spell aura classification and explicit stat/cooldown output policies.

#[cfg(any(feature = "aura-containers", feature = "retail-12-0-5"))]
use super::ensure_namespace;
#[cfg(feature = "retail-12-0-5")]
use crate::lua_api::methods::borrow_state;
#[cfg(any(feature = "aura-containers", feature = "retail-12-0-5"))]
use crate::lua_bridge::table_set_rust_fn_static;
#[cfg(feature = "aura-containers")]
use rilua::runtime_error;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[cfg(feature = "aura-containers")]
#[path = "../../data/spell_aura_secrecy.rs"]
mod attributes;

#[cfg(feature = "aura-containers")]
const NEVER_SECRET: i32 = 0;
#[cfg(feature = "aura-containers")]
const ALWAYS_SECRET: i32 = 1;
#[cfg(feature = "aura-containers")]
const CONTEXTUALLY_SECRET: i32 = 2;

#[cfg(any(feature = "aura-containers", feature = "retail-12-0-5"))]
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Secrets")?;
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

#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
fn should_cooldowns_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = borrow_state(state)?.cooldowns_restricted;
    state.push(Val::Bool(restricted));
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
fn classify_aura_secrecy(spell_id: u32) -> LuaResult<i32> {
    match attributes::aura_flags(spell_id) {
        0 => Ok(CONTEXTUALLY_SECRET),
        attributes::AURA_ALWAYS_SECRET => Ok(ALWAYS_SECRET),
        attributes::AURA_NEVER_SECRET => Ok(NEVER_SECRET),
        flags => Err(runtime_error(format!(
            "C_Secrets.GetSpellAuraSecrecy: ambiguous aura secrecy flags {flags:#x} for spell {spell_id}; native precedence is unverified"
        ))),
    }
}
