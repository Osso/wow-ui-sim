//! Retail 12.1.0 unit secret predicates applied to already-pushed returns.
//!
//! - `SecretWhenUnitIdentityRestricted`: returns are secret when any queried
//!   unit's identity is secret (`unit_identity_is_secret`).
//! - `SecretWhenUnitPossessionRestricted`: returns are secret while auras are
//!   secret (`unit_auras_restricted`), except for `player`, `pet` and `vehicle`.
//! - `SecretWhenUnitNameIdentityRestricted`: identity rules, except a player
//!   unit queried during an active PvP match.
//!
//! Secret returns are host-wrapped for every caller, matching `UnitName`.

use crate::lua_api::methods::{borrow_state, val_to_string};
use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const RETAIL_12_1_0: bool = cfg!(feature = "retail-12-1-0");

/// Wrap the top `count` returns when any of `units` has a secret identity.
pub(crate) fn finish_identity_restricted(
    state: &mut LuaState,
    units: &[&str],
    count: u32,
) -> LuaResult<u32> {
    if RETAIL_12_1_0 && any_identity_secret(state, units)? {
        wrap_returns(state, count);
    }
    Ok(count)
}

/// Wrap the top `count` returns while auras are secret, except for units
/// under the player's direct control.
pub(crate) fn finish_possession_restricted(
    state: &mut LuaState,
    unit: &str,
    count: u32,
) -> LuaResult<u32> {
    let controlled = matches!(unit, "player" | "pet" | "vehicle");
    if RETAIL_12_1_0 && !controlled && borrow_state(state)?.unit_auras_restricted {
        wrap_returns(state, count);
    }
    Ok(count)
}

/// `UnitName` secrecy: identity rules minus players during an active PvP match.
/// INFERRED: "player" means the resolved GUID is a `Player-` GUID.
pub(crate) fn unit_name_identity_restricted(state: &mut LuaState, unit: &str) -> LuaResult<bool> {
    let secret = super::super::unit_misc::unit_identity_is_secret(state, unit)?;
    if !RETAIL_12_1_0 || !secret {
        return Ok(secret);
    }
    let sim = borrow_state(state)?;
    let player_in_pvp = sim.pvp_match_active
        && super::super::unit_misc::existing_guid_for_unit(&sim, unit)
            .is_some_and(|guid| guid.starts_with("Player-"));
    Ok(!player_in_pvp)
}

fn any_identity_secret(state: &mut LuaState, units: &[&str]) -> LuaResult<bool> {
    for unit in units {
        if super::super::unit_misc::unit_identity_is_secret(state, unit)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Replace each stack slot in place so every wrapper stays rooted.
fn wrap_returns(state: &mut LuaState, count: u32) {
    let first = state.top - count as usize;
    for slot in first..state.top {
        state.stack[slot] = secret_copy(state, state.stack[slot]);
    }
}

/// Absent values (`nil`) stay absent; `MayReturnNothing` APIs keep their arity.
fn secret_copy(state: &mut LuaState, value: Val) -> Val {
    match value {
        Val::Bool(flag) => wrap_host_secret_bool(state, flag),
        Val::Num(number) => wrap_host_secret_number(state, number),
        Val::Str(_) => {
            let text = val_to_string(state, value).unwrap_or_default();
            wrap_host_secret_string(state, &text)
        }
        other => other,
    }
}
