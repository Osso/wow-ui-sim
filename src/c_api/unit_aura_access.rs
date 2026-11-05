//! Retail 12.1.0 aura-secret context.
//!
//! WM105 defines the gate in this order: an execution context, a recorded taint
//! source (a null source permits), the generated-secrecy-policy flag, and any
//! of restriction entries 0..3 (`Combat`, `Encounter`, `ChallengeMode`,
//! `PvPMatch`) exactly `Active`. `Map` and `Chat` are never read, and the gate
//! addresses the same six-entry array as `GetAddOnRestrictionState`.
//!
//! - `RequiresUnitAuraAccess` APIs raise
//!   `<ApiName>(): <aura report> Lua Taint: <source>` for a denied caller.
//! - Restricted AuraData uses the bounded WM61 S3 per-field publication policy.
//! - Spell-keyed (`RequiresNonSecretAura`) APIs stay callable; auras flagged
//!   never-secret by `C_Secrets.GetSpellAuraSecrecy` return plain data.
//! - `UNIT_AURA` update payloads are secret-wrapped while auras are secret.
//! - Script objects carrying `DenyTaintedAccessWhenAurasAreSecret` (Blizzard
//!   applies it to AuraButtons after `initializeFrame`, deferred to
//!   PLAYER_ENTERING_WORLD before login) are forbidden to tainted callers only
//!   while auras are secret; they become accessible again when auras do not.

use crate::lua_api::methods::{borrow_state, table_get, table_set};
use crate::lua_api::state::SimState;
use crate::widget::Frame;
use rilua::table_security::wrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// EMULATED (WM105 §2): the read-only generated-secrecy-policy byte is `1` in
/// this build, and any nonzero value enables the state tests. No runtime writer
/// was recovered, so the shipped value is the modelled one.
pub(crate) const GENERATED_SECRECY_POLICY_ENABLED: bool = true;

/// `AddOnRestrictionType` entries the aura gate reads (WM105 §1).
const AURA_RESTRICTION_TYPES: usize = 4;
/// `AddOnRestrictionState.Active` (WM105 §1); `Activating` is 1 and does not match.
const ACTIVE: u8 = 2;

/// WM105 §5: one rule shared by the aura gate, the S8 ContextuallySecret
/// classifier and the accessor publication class.
pub(crate) fn aura_secrecy_policy_active(sim: &SimState) -> bool {
    GENERATED_SECRECY_POLICY_ENABLED
        && sim.restricted_actions[..AURA_RESTRICTION_TYPES]
            .iter()
            .any(|&state| state == ACTIVE)
}

pub(crate) fn auras_restricted(state: &LuaState) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    Ok(cfg!(feature = "retail-12-1-0") && aura_secrecy_policy_active(&sim))
}

/// `Enum.ScriptObjectAccessRestriction.DenyTaintedAccessWhenAurasAreSecret`.
pub(crate) const DENY_TAINTED_ACCESS_WHEN_AURAS_SECRET: u32 = 1;

/// Whether `frame`'s aura access restriction is in force right now.
pub(crate) fn aura_access_restriction_active(sim: &SimState, frame: &Frame) -> bool {
    cfg!(feature = "retail-12-1-0")
        && aura_secrecy_policy_active(sim)
        && frame.access_restrictions & DENY_TAINTED_ACCESS_WHEN_AURAS_SECRET != 0
}

/// Native script-object resolution: tainted code cannot reach an object whose
/// aura access restriction is in force, checked only for restricted objects
/// while auras are secret.
/// The text is the client's forbidden-object template, which names no addon
/// (S9 (e)); that this path uses it is best-guess.
pub(crate) fn ensure_script_object_accessible(state: &LuaState, id: u64) -> LuaResult<()> {
    if !cfg!(feature = "retail-12-1-0") {
        return Ok(());
    }
    let restricted = {
        let sim = borrow_state(state)?;
        aura_secrecy_policy_active(&sim)
            && sim
                .widgets
                .get(id)
                .is_some_and(|frame| aura_access_restriction_active(&sim, frame))
    };
    if !restricted || rilua::api::state_is_secure(state) {
        return Ok(());
    }
    Err(runtime_error(
        "Attempt to access forbidden object from code tainted by an AddOn",
    ))
}

/// Enforce `RequiresUnitAuraAccess` before argument parsing or lookup.
///
/// WM105: the accessor raises `<ApiName>(): <aura report> Lua Taint: <source>`.
/// The report is `Auras cannot be accessed when secret while tainted by
/// '<source>'`; the wrapper appends `Lua Taint: <source>` (or `No Lua Taint`
/// when no source is recorded, which permits and therefore never reaches here).
/// The suffix separator is best-guess, native UNKNOWN.
pub(crate) fn require_unit_aura_access(state: &LuaState, api_name: &str) -> LuaResult<()> {
    if !auras_restricted(state)? {
        return Ok(());
    }
    let Some(origin) = rilua::api::stack_taint_origin(state) else {
        return Ok(());
    };
    Err(runtime_error(format!(
        "{api_name}(): Auras cannot be accessed when secret while tainted by '{origin}' Lua Taint: {origin}"
    )))
}

/// Apply WM61 S3 field publication while preserving the outer table value.
pub(crate) fn finish_aura_data(state: &mut LuaState, aura_data: Val) -> LuaResult<Val> {
    if !auras_restricted(state)? || matches!(aura_data, Val::Nil) {
        return Ok(aura_data);
    }
    if let Val::Num(spell_id) = table_get(state, aura_data, "spellId")
        && !spell_keyed_aura_is_secret(state, spell_id as i32)?
    {
        return Ok(aura_data);
    }
    state.push(aura_data);
    let result = mark_aura_fields(state, aura_data);
    state.pop();
    result?;
    Ok(aura_data)
}

fn mark_aura_fields(state: &mut LuaState, aura_data: Val) -> LuaResult<()> {
    for field in [
        "auraInstanceID",
        "spellId",
        "icon",
        "applications",
        "name",
        "duration",
        "expirationTime",
        "sourceUnit",
        "dispelName",
        "isHelpful",
        "isHarmful",
        "points",
    ] {
        let value = table_get(state, aura_data, field);
        if !matches!(value, Val::Nil) {
            let secret = wrap_secret(state, value)?;
            table_set(state, aura_data, field, secret);
        }
    }
    Ok(())
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
    finish_aura_data(state, aura_data)
}

/// WM61 S8 pre-lookup gate for the ID-based spell queries.
pub(crate) fn reject_spell_keyed_lookup(state: &LuaState, spell_id: u32) -> LuaResult<bool> {
    if !auras_restricted(state)? || rilua::api::state_is_secure(state) {
        return Ok(false);
    }
    #[cfg(feature = "retail-12-1-0")]
    {
        Ok(super::c_secrets::classify_aura_secrecy(spell_id)? != super::c_secrets::NEVER_SECRET)
    }
    #[cfg(not(feature = "retail-12-1-0"))]
    {
        let _ = spell_id;
        Ok(false)
    }
}

/// WM105 §5: the S8 `ContextuallySecret` outcome reads the same policy flag and
/// the same four restriction entries as the aura gate. `AlwaysSecret` is secret
/// for every caller; `NeverSecret` stays public.
pub(crate) fn spell_keyed_aura_is_secret(state: &LuaState, spell_id: i32) -> LuaResult<bool> {
    #[cfg(feature = "retail-12-1-0")]
    {
        let Ok(spell_id) = u32::try_from(spell_id) else {
            return Ok(false);
        };
        Ok(match super::c_secrets::classify_aura_secrecy(spell_id)? {
            super::c_secrets::NEVER_SECRET => false,
            super::c_secrets::ALWAYS_SECRET => true,
            _ => auras_restricted(state)?,
        })
    }
    #[cfg(not(feature = "retail-12-1-0"))]
    {
        let _ = spell_id;
        Ok(false)
    }
}

/// `UNIT_AURA` update info while auras are secret. INFERRED: the unit token
/// stays public because it routes `RegisterUnitEvent` delivery.
pub(crate) fn unit_aura_event_payload(state: &mut LuaState, update_info: Val) -> LuaResult<Val> {
    if auras_restricted(state)? {
        return wrap_secret(state, update_info);
    }
    Ok(update_info)
}
