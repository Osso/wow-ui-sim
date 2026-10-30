//! Immutable aura duration queries over explicit per-environment recast metadata.
//! Formula, eligibility and validation policies are INFERRED, not native-verified.

use super::unit_aura_access::require_unit_aura_access;
use crate::lua_api::game_data::AuraInfo;
use crate::lua_api::globals::auras::collect_filtered_unit_auras;
use crate::lua_api::globals::lua_duration_object::push_timed_duration_object;
use crate::lua_api::methods::borrow_state;
use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::table_security::is_secret_value;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// Recast metadata, independent of an active aura's current duration.
/// No production records are seeded; the explicit cap is an inferred policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpellAuraDuration {
    pub base_duration_seconds: f64,
    pub max_carryover_seconds: f64,
}

impl SpellAuraDuration {
    fn is_valid(self) -> bool {
        self.base_duration_seconds.is_finite()
            && self.base_duration_seconds >= 0.0
            && self.max_carryover_seconds.is_finite()
            && self.max_carryover_seconds >= 0.0
    }

    fn carryover(self, aura: &AuraInfo, now: f64) -> Option<f64> {
        // User-approved default, not native evidence.
        if aura.duration == 0.0 || aura.expiration_time == 0.0 {
            return None;
        }
        Some(
            (aura.expiration_time - now)
                .max(0.0)
                .min(self.max_carryover_seconds),
        )
    }

    fn refresh_duration(self, aura: &AuraInfo, now: f64) -> Option<f64> {
        let duration = self.base_duration_seconds + self.carryover(aura, now)?;
        duration.is_finite().then_some(duration)
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAuraBaseDuration",
        get_aura_base_duration,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "DoesAuraHaveExpirationTime",
        does_aura_have_expiration_time,
    )?;
    table_set_rust_fn_static(state, namespace, "GetAuraDuration", get_aura_duration)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetRefreshExtendedDuration",
        get_refresh_extended_duration,
    )?;
    #[cfg(feature = "retail-12-1-5")]
    table_set_rust_fn_static(
        state,
        namespace,
        "GetRefreshCarryOverDuration",
        get_refresh_carryover_duration,
    )?;
    Ok(())
}

/// Timed snapshot of a stored aura. INFERRED: an unknown unit or instance errors
/// (`RequiresValidUnitAuraInstance`); a permanent aura yields a zero-span object.
fn get_aura_duration(state: &mut LuaState) -> LuaResult<u32> {
    require_unit_aura_access(state, "C_UnitAuras.GetAuraDuration")?;
    let (unit, instance_id) = read_expiration_arguments(state)?;
    let aura = unit
        .as_deref()
        .and_then(|unit| find_public_aura(state, unit, instance_id))
        .ok_or_else(|| runtime_error("C_UnitAuras.GetAuraDuration: no such aura instance"))?;
    push_timed_duration_object(state, aura.expiration_time - aura.duration, aura.duration)
}

fn does_aura_have_expiration_time(state: &mut LuaState) -> LuaResult<u32> {
    require_unit_aura_access(state, "C_UnitAuras.DoesAuraHaveExpirationTime")?;
    let (unit, instance_id) = read_expiration_arguments(state)?;
    let expires = unit
        .as_deref()
        .and_then(|unit| find_public_aura(state, unit, instance_id))
        .is_some_and(|aura| aura.expiration_time != 0.0);
    state.push(Val::Bool(expires));
    Ok(1)
}

/// 12.0.5 `AllowedWhenUntainted`: the VM unwraps secrets for untainted callers
/// and denies tainted ones. Both arguments are authenticated before either is
/// validated. Aura access is unmodeled.
fn read_expiration_arguments(state: &LuaState) -> LuaResult<(Option<String>, f64)> {
    let unit = unwrap_secret(state, stack_val(state, 1))?;
    let instance_id = unwrap_secret(state, stack_val(state, 2))?;
    let unit = match unit {
        Val::Nil => None,
        Val::Str(_) => Some(
            val_to_string(state, unit)
                .ok_or_else(|| runtime_error("C_UnitAuras: unit must be a UTF-8 string or nil"))?,
        ),
        _ => return Err(runtime_error("C_UnitAuras: unit must be a string or nil")),
    };
    match instance_id {
        Val::Num(instance_id) if instance_id.is_finite() => Ok((unit, instance_id)),
        _ => Err(runtime_error(
            "C_UnitAuras: aura instance must be a finite number",
        )),
    }
}

fn get_aura_base_duration(state: &mut LuaState) -> LuaResult<u32> {
    require_unit_aura_access(state, "C_UnitAuras.GetAuraBaseDuration")?;
    push_duration(state, DurationQuery::Base)
}

fn get_refresh_extended_duration(state: &mut LuaState) -> LuaResult<u32> {
    require_unit_aura_access(state, "C_UnitAuras.GetRefreshExtendedDuration")?;
    push_duration(state, DurationQuery::Extended)
}

#[cfg(feature = "retail-12-1-5")]
fn get_refresh_carryover_duration(state: &mut LuaState) -> LuaResult<u32> {
    require_unit_aura_access(state, "C_UnitAuras.GetRefreshCarryOverDuration")?;
    push_duration(state, DurationQuery::Carryover)
}

#[derive(Clone, Copy)]
enum DurationQuery {
    Base,
    Extended,
    #[cfg(feature = "retail-12-1-5")]
    Carryover,
}

fn push_duration(state: &mut LuaState, query: DurationQuery) -> LuaResult<u32> {
    let duration = query_duration(state, query)?;
    state.push(duration.map_or(Val::Nil, Val::Num));
    Ok(1)
}

fn query_duration(state: &mut LuaState, query: DurationQuery) -> LuaResult<Option<f64>> {
    let (unit, instance_id) = read_public_arguments(state)?;
    let Some(unit) = unit else {
        return Ok(None);
    };
    let Some(aura) = find_public_aura(state, &unit, instance_id) else {
        return Ok(None);
    };
    let spell_id = if matches!(stack_val(state, 3), Val::Nil) {
        Some(aura.spell_id as u32)
    } else {
        // Explicit identifiers alone use the existing alias-first resolver.
        super::c_spell::read_spell_identifier_at(state, 3)?
    };
    let Some(spell_id) = spell_id else {
        return Ok(None);
    };
    let sim = borrow_state(state)?;
    let Some(metadata) = sim.spell_aura_durations.get(&spell_id).copied() else {
        return Ok(None);
    };
    if !metadata.is_valid() {
        return Ok(None);
    }
    Ok(match query {
        DurationQuery::Base => Some(metadata.base_duration_seconds),
        DurationQuery::Extended => {
            metadata.refresh_duration(&aura, sim.clock_elapsed().as_secs_f64())
        }
        #[cfg(feature = "retail-12-1-5")]
        DurationQuery::Carryover => metadata.carryover(&aura, sim.clock_elapsed().as_secs_f64()),
    })
}

fn find_public_aura(state: &mut LuaState, unit: &str, instance_id: f64) -> Option<AuraInfo> {
    for filter in ["HELPFUL", "HARMFUL"] {
        if let Some(aura) = collect_filtered_unit_auras(state, unit, filter)
            .into_iter()
            .find(|aura| f64::from(aura.aura_instance_id) == instance_id)
        {
            return Some(aura);
        }
    }
    None
}

fn read_public_arguments(state: &LuaState) -> LuaResult<(Option<String>, f64)> {
    reject_unsupported_duration_secrets(state)?;
    let unit = match stack_val(state, 1) {
        Val::Nil => None,
        Val::Str(_) => Some(String::from_stack(state, 1)?),
        _ => {
            return Err(runtime_error(
                "C_UnitAuras: unit must be a public string or nil",
            ));
        }
    };
    let instance_id = read_duration_instance(state)?;
    match stack_val(state, 3) {
        Val::Nil | Val::Str(_) => {}
        Val::Num(number) if number.is_finite() => {}
        _ => {
            return Err(runtime_error(
                "C_UnitAuras: spell identifier must be nil or a public finite number or string",
            ));
        }
    }
    Ok((unit, instance_id))
}

fn duration_instance_is_numeric_secret(state: &LuaState) -> bool {
    cfg!(any(feature = "profile-retail", feature = "client-ptr"))
        && rilua::table_security::secret_payload_kind(state, stack_val(state, 2))
            == Some(rilua::table_security::SecretPayloadKind::Number)
}

fn reject_unsupported_duration_secrets(state: &LuaState) -> LuaResult<()> {
    // Only numeric instance selectors are admitted; units and optional spells
    // retain their existing policy before conversion or unknown results.
    for (index, name) in [(1, "unit"), (2, "aura instance"), (3, "spell identifier")] {
        if index == 2 && duration_instance_is_numeric_secret(state) {
            continue;
        }
        if is_secret_value(state, stack_val(state, index)) {
            return Err(runtime_error(format!(
                "C_UnitAuras: secret {name} access is not modeled"
            )));
        }
    }
    Ok(())
}

fn read_duration_instance(state: &LuaState) -> LuaResult<f64> {
    let value = if duration_instance_is_numeric_secret(state) {
        rilua::table_security::inspect_host_secret_number(state, stack_val(state, 2), |number| {
            Ok(Val::Num(number))
        })?
    } else {
        stack_val(state, 2)
    };
    match value {
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(runtime_error(
            "C_UnitAuras: aura instance must be a finite number",
        )),
    }
}
