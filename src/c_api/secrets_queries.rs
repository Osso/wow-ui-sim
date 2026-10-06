//! C_Secrets describes current output policies, independently of caller taint.
//! Contract: cached SecretPredicateAPIDocumentation.lua (not SecretsDocumentation.lua).
//! Missing native spell-cast/power attributes and health/power/comparison/totem
//! restriction models are explicitly distinguished from modeled restrictions.

use crate::c_api::{c_spell, c_spell_book, charge_state, unit_aura_access};
use crate::lua_api::globals::unit_misc;
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const NEVER_SECRET: f64 = 0.0;
const CONTEXTUALLY_SECRET: f64 = 2.0;
type Query = fn(&mut LuaState) -> LuaResult<u32>;

const QUERIES: &[(&str, Query)] = &[
    #[cfg(feature = "retail-12-0-5")]
    ("CanCompareUnitTokens", can_compare_unit_tokens),
    #[cfg(feature = "retail-12-0-5")]
    (
        "ShouldUnitThreatStateBeSecret",
        should_unit_threat_state_be_secret,
    ),
    ("GetPowerTypeSecrecy", get_power_type_secrecy),
    ("GetSpellCastSecrecy", get_spell_cast_secrecy),
    ("GetSpellCooldownSecrecy", get_spell_cooldown_secrecy),
    ("HasSecretRestrictions", has_secret_restrictions),
    (
        "ShouldActionCooldownBeSecret",
        should_action_cooldown_be_secret,
    ),
    ("ShouldAurasBeSecret", should_auras_be_secret),
    ("ShouldSpellAuraBeSecret", should_spell_aura_be_secret),
    (
        "ShouldSpellBookItemCooldownBeSecret",
        should_spell_book_item_cooldown_be_secret,
    ),
    (
        "ShouldSpellCooldownBeSecret",
        should_spell_cooldown_be_secret,
    ),
    ("ShouldTotemSlotBeSecret", should_totem_slot_be_secret),
    ("ShouldTotemSpellBeSecret", should_totem_spell_be_secret),
    (
        "ShouldUnitAuraIndexBeSecret",
        should_unit_aura_index_be_secret,
    ),
    (
        "ShouldUnitAuraInstanceBeSecret",
        should_unit_aura_instance_be_secret,
    ),
    (
        "ShouldUnitAuraSlotBeSecret",
        should_unit_aura_slot_be_secret,
    ),
    (
        "ShouldUnitComparisonBeSecret",
        should_unit_comparison_be_secret,
    ),
    (
        "ShouldUnitHealthMaxBeSecret",
        should_unit_health_max_be_secret,
    ),
    ("ShouldUnitIdentityBeSecret", should_unit_identity_be_secret),
    ("ShouldUnitPowerBeSecret", should_unit_power_be_secret),
    (
        "ShouldUnitPowerMaxBeSecret",
        should_unit_power_max_be_secret,
    ),
    (
        "ShouldUnitSpellCastBeSecret",
        should_unit_spell_cast_be_secret,
    ),
];

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for &(name, query) in QUERIES {
        table_set_rust_fn_static(state, namespace, name, query)?;
    }
    Ok(())
}

/// Authenticate every selector before looking at context, including selectors
/// irrelevant to a currently-unmodeled aspect. Queries never clear caller taint.
fn authenticate_arguments(state: &LuaState, count: i32) -> LuaResult<()> {
    for position in 1..=count {
        unwrap_secret(state, stack_val(state, position))?;
    }
    Ok(())
}

fn read_unit(state: &LuaState, position: i32) -> LuaResult<String> {
    let value = unwrap_secret(state, stack_val(state, position))?;
    Ok(val_to_string(state, value).unwrap_or_default())
}

fn read_spell(state: &LuaState, position: i32) -> LuaResult<Option<u32>> {
    let value = unwrap_secret(state, stack_val(state, position))?;
    Ok(c_spell::numeric_spell_id_value(state, value))
}

fn push_bool(state: &mut LuaState, flag: bool) -> LuaResult<u32> {
    state.push(Val::Bool(flag));
    Ok(1)
}

fn push_level(state: &mut LuaState, level: f64) -> LuaResult<u32> {
    state.push(Val::Num(level));
    Ok(1)
}

fn has_secret_restrictions(state: &mut LuaState) -> LuaResult<u32> {
    // This is a build capability query, not a query of active restrictions.
    push_bool(state, cfg!(feature = "retail-12-0-0"))
}

fn should_auras_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = unit_aura_access::auras_restricted(state)?;
    push_bool(state, restricted)
}

fn should_spell_aura_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    let spell = read_spell(state, 1)?;
    let secret = match spell.and_then(|id| i32::try_from(id).ok()) {
        Some(spell) => unit_aura_access::spell_keyed_aura_is_secret(state, spell)?,
        None => false,
    };
    push_bool(state, secret)
}

fn should_unit_aura_index_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 3)?;
    should_auras_be_secret(state)
}

fn should_unit_aura_instance_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 2)?;
    // INFERRED: report the access policy even for absent selectors, because
    // RequiresUnitAuraAccess runs before lookup in the corresponding API.
    should_auras_be_secret(state)
}

fn should_unit_aura_slot_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    should_unit_aura_instance_be_secret(state)
}

fn should_spell_cooldown_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    read_spell(state, 1)?;
    let restricted = read_cooldown_restriction(state)?;
    push_bool(state, restricted)
}

fn should_action_cooldown_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 1)?;
    // Existing action output wraps even the zero-duration empty-slot DTO.
    let restricted = read_cooldown_restriction(state)?;
    push_bool(state, restricted)
}

fn read_cooldown_restriction(state: &LuaState) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    Ok(charge_state::cooldowns_are_restricted(&sim))
}

fn should_spell_book_item_cooldown_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    let slot = unwrap_secret(state, stack_val(state, 1))?;
    let bank = unwrap_secret(state, stack_val(state, 2))?;
    let present = c_spell_book::cooldown_spell_for_book_entry(slot, bank).is_some();
    let restricted = read_cooldown_restriction(state)?;
    push_bool(state, present && restricted)
}

fn should_unit_identity_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_unit(state, 1)?;
    let secret = unit_misc::unit_identity_is_secret(state, &unit)?;
    push_bool(state, secret)
}

fn get_spell_cooldown_secrecy(state: &mut LuaState) -> LuaResult<u32> {
    read_spell(state, 1)?;
    // INFERRED: no per-spell cooldown secrecy attributes are modeled. Every
    // current cooldown uses the shared contextual restriction, including zeros.
    push_level(state, CONTEXTUALLY_SECRET)
}

fn get_power_type_secrecy(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 1)?;
    // INFERRED simulator default: no native power secrecy attribute dataset or
    // restriction policy exists; UnitPower and UnitPowerMax currently stay public.
    // NeverSecret describes those current outputs, not the native power dataset.
    push_level(state, NEVER_SECRET)
}

fn get_spell_cast_secrecy(state: &mut LuaState) -> LuaResult<u32> {
    read_spell(state, 1)?;
    // INFERRED simulator default: only public player cast/channel data exists.
    // Native per-spell cast secrecy attributes and foreign casts are unmodeled.
    push_level(state, NEVER_SECRET)
}

fn should_unit_health_max_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 1)?;
    // INFERRED simulator default: UnitHealthMax has no health-max secrecy model.
    push_bool(state, false)
}

fn should_unit_power_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 2)?;
    // INFERRED simulator default: power-type secrecy and power restrictions
    // have no backing state. Do not borrow the unrelated unit-stat restriction.
    push_bool(state, false)
}

fn should_unit_power_max_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    should_unit_power_be_secret(state)
}

#[cfg(feature = "retail-12-0-5")]
fn can_compare_unit_tokens(state: &mut LuaState) -> LuaResult<u32> {
    let lhs = read_unit(state, 1)?;
    let rhs = read_unit(state, 2)?;
    push_bool(
        state,
        super::unit_comparison::permitted(Some(&lhs), Some(&rhs)),
    )
}

#[cfg(feature = "retail-12-0-5")]
fn should_unit_threat_state_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 2)?;
    // The same host policy controls UnitThreatLeadSituation secret returns.
    let restricted = borrow_state(state)?
        .plain_global_inputs
        .threat_state_restricted;
    push_bool(state, restricted)
}

fn should_unit_comparison_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 2)?;
    // INFERRED simulator default: UnitIsUnit implements comparability but not
    // SecretWhenUnitComparisonRestricted. Identity secrecy is a different policy.
    push_bool(state, false)
}

fn should_unit_spell_cast_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 2)?;
    // INFERRED simulator default: existing player-only cast readers stay public.
    push_bool(state, false)
}

fn should_totem_slot_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    authenticate_arguments(state, 1)?;
    // Empty GetTotemInfo is the only current totem surface. Active totem slots,
    // their associated spells and SecretWhenTotemSlotSecret are unmodeled.
    push_bool(state, false)
}

fn should_totem_spell_be_secret(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED simulator default: no active totem spell can yield secret data.
    should_totem_slot_be_secret(state)
}
