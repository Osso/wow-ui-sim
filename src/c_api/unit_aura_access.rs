//! Retail 12.1.0 aura-secret context over the explicit `unit_auras_restricted`
//! input (secret during combat, encounters, M+ and PvP matches).
//!
//! - `RequiresUnitAuraAccess` APIs (index, slot and instance-ID access) raise a
//!   Lua error for tainted callers while auras are secret.
//! - AuraData returned while auras are secret is a secret-wrapped table: secure
//!   code indexes it as usual, tainted code cannot read it.
//! - Spell-keyed (`RequiresNonSecretAura`) APIs stay callable; auras flagged
//!   never-secret by `C_Secrets.GetSpellAuraSecrecy` return plain data.
//! - `UNIT_AURA` update payloads are secret-wrapped while auras are secret.
//! - Script objects carrying `DenyTaintedAccessWhenAurasAreSecret` (Blizzard
//!   applies it to AuraButtons after `initializeFrame`, deferred to
//!   PLAYER_ENTERING_WORLD before login) are forbidden to tainted callers only
//!   while auras are secret; they become accessible again when auras do not.

use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::SimState;
use crate::widget::Frame;
use rilua::table_security::wrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn auras_restricted(state: &LuaState) -> LuaResult<bool> {
    Ok(cfg!(feature = "retail-12-1-0") && borrow_state(state)?.unit_auras_restricted)
}

/// `Enum.ScriptObjectAccessRestriction.DenyTaintedAccessWhenAurasAreSecret`.
pub(crate) const DENY_TAINTED_ACCESS_WHEN_AURAS_SECRET: u32 = 1;

/// Whether `frame`'s aura access restriction is in force right now.
pub(crate) fn aura_access_restriction_active(sim: &SimState, frame: &Frame) -> bool {
    cfg!(feature = "retail-12-1-0")
        && sim.unit_auras_restricted
        && frame.access_restrictions & DENY_TAINTED_ACCESS_WHEN_AURAS_SECRET != 0
}

/// Native script-object resolution: tainted code cannot reach an object whose
/// aura access restriction is in force. The stack-taint walk runs only for
/// restricted objects while auras are secret.
/// INFERRED: error wording follows the live forbidden-object message; the
/// 12.1.0 page gives no text contract.
pub(crate) fn ensure_script_object_accessible(state: &LuaState, id: u64) -> LuaResult<()> {
    if !cfg!(feature = "retail-12-1-0") {
        return Ok(());
    }
    let restricted = {
        let sim = borrow_state(state)?;
        sim.unit_auras_restricted
            && sim
                .widgets
                .get(id)
                .is_some_and(|frame| aura_access_restriction_active(&sim, frame))
    };
    if !restricted || rilua::api::state_is_secure(state) {
        return Ok(());
    }
    let taint = state.call_stack[..=state.ci]
        .iter()
        .find_map(|frame| frame.taint.clone())
        .unwrap_or_default();
    Err(runtime_error(format!(
        "Attempt to access forbidden object from code tainted by '{taint}'"
    )))
}

/// Enforce `RequiresUnitAuraAccess` before argument parsing or lookup.
pub(crate) fn require_unit_aura_access(state: &LuaState, api_name: &str) -> LuaResult<()> {
    if auras_restricted(state)? && !rilua::api::state_is_secure(state) {
        return Err(runtime_error(format!(
            "{api_name}: aura data cannot be accessed by addons while auras are secret"
        )));
    }
    Ok(())
}

/// AuraData is fully secret while auras are secret. Only secure callers reach
/// this under restriction; `wrap_secret` rejects any other caller.
pub(crate) fn finish_aura_data(state: &mut LuaState, aura_data: Val) -> LuaResult<Val> {
    if auras_restricted(state)? {
        return wrap_secret(state, aura_data);
    }
    Ok(aura_data)
}

/// `RequiresNonSecretAura`: never-secret spells stay plain for every caller.
/// INFERRED: other auras return nil to tainted callers and secret AuraData to
/// secure callers while auras are secret.
pub(crate) fn finish_spell_keyed_aura_data(
    state: &mut LuaState,
    spell_id: i32,
    aura_data: Val,
) -> LuaResult<Val> {
    if !spell_keyed_aura_is_secret(state, spell_id)? {
        return Ok(aura_data);
    }
    if !rilua::api::state_is_secure(state) {
        return Ok(Val::Nil);
    }
    wrap_secret(state, aura_data)
}

/// Shared by spell-keyed outputs and the prospective C_Secrets query.
/// Preserve the existing context-first policy, including its known limitation:
/// always-secret spell flags do not yet force secrecy outside this context.
pub(crate) fn spell_keyed_aura_is_secret(state: &LuaState, spell_id: i32) -> LuaResult<bool> {
    Ok(auras_restricted(state)? && !is_never_secret_aura(spell_id)?)
}

#[cfg(feature = "retail-12-1-0")]
fn is_never_secret_aura(spell_id: i32) -> LuaResult<bool> {
    super::c_secrets::is_never_secret_aura(spell_id)
}

#[cfg(not(feature = "retail-12-1-0"))]
fn is_never_secret_aura(_spell_id: i32) -> LuaResult<bool> {
    unreachable!("auras are never restricted before 12.1.0")
}

/// `UNIT_AURA` update info while auras are secret. INFERRED: the unit token
/// stays public because it routes `RegisterUnitEvent` delivery.
pub(crate) fn unit_aura_event_payload(state: &mut LuaState, update_info: Val) -> LuaResult<Val> {
    finish_aura_data(state, update_info)
}
