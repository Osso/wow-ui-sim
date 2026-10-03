//! Immutable aura duration queries over explicit per-environment recast metadata.
//! Formula, eligibility and validation policies are INFERRED, not native-verified.

use crate::lua_api::game_data::AuraInfo;
use crate::lua_api::globals::auras::collect_filtered_unit_auras;
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

    fn refresh_duration(self, aura: &AuraInfo, now: f64) -> Option<f64> {
        // Inferred eligibility: permanent active records have no refresh total.
        if aura.duration == 0.0 || aura.expiration_time == 0.0 {
            return None;
        }
        let remaining = (aura.expiration_time - now).max(0.0);
        let duration = self.base_duration_seconds + remaining.min(self.max_carryover_seconds);
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
    table_set_rust_fn_static(
        state,
        namespace,
        "GetRefreshExtendedDuration",
        get_refresh_extended_duration,
    )
}

fn does_aura_have_expiration_time(state: &mut LuaState) -> LuaResult<u32> {
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
    push_duration(state, false)
}

fn get_refresh_extended_duration(state: &mut LuaState) -> LuaResult<u32> {
    push_duration(state, true)
}

fn push_duration(state: &mut LuaState, extended: bool) -> LuaResult<u32> {
    let duration = query_duration(state, extended)?;
    state.push(duration.map_or(Val::Nil, Val::Num));
    Ok(1)
}

fn query_duration(state: &mut LuaState, extended: bool) -> LuaResult<Option<f64>> {
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
    Ok(if extended {
        metadata.refresh_duration(&aura, sim.start_time.elapsed().as_secs_f64())
    } else {
        Some(metadata.base_duration_seconds)
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
    // Reject all secret inputs before conversion, resolution or unknown results.
    // This does not implement native AllowedWhenTainted/output-secret parity.
    for (index, name) in [(1, "unit"), (2, "aura instance"), (3, "spell identifier")] {
        if is_secret_value(state, stack_val(state, index)) {
            return Err(runtime_error(format!(
                "C_UnitAuras: secret {name} access is not modeled"
            )));
        }
    }
    let unit = match stack_val(state, 1) {
        Val::Nil => None,
        Val::Str(_) => Some(String::from_stack(state, 1)?),
        _ => {
            return Err(runtime_error(
                "C_UnitAuras: unit must be a public string or nil",
            ));
        }
    };
    let instance_id = match stack_val(state, 2) {
        Val::Num(number) if number.is_finite() => number,
        _ => {
            return Err(runtime_error(
                "C_UnitAuras: aura instance must be a public finite number",
            ));
        }
    };
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
