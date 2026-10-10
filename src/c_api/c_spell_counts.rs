//! Retail 12.0.0 cast counts and 12.0.5 display/max counts over explicit inputs.
//! Charge priority, formatting domains and empty/zero policies are inferred.
//! Cast secret-input permissions remain unmodeled; output restriction is separate.

#[cfg(feature = "retail-12-0-5")]
use super::c_action_bar_counts::{format_display_count, parse_maximum, parse_replacement};
use super::c_spell::read_public_spell_identifier_at;
#[cfg(feature = "retail-12-0-5")]
use super::c_spell::read_spell_identifier_value;
use super::charge_state::cooldowns_are_restricted;
#[cfg(feature = "retail-12-0-5")]
use super::charge_state::read_charge_input;
use crate::lua_api::methods::borrow_state;
#[cfg(feature = "retail-12-0-5")]
use crate::lua_api::methods::{create_string, val_to_string};
#[cfg(feature = "retail-12-0-5")]
use crate::lua_bridge::stack_val;
#[cfg(feature = "retail-12-0-5")]
use rilua::runtime_error;
use rilua::table_security::wrap_host_secret_number;
#[cfg(feature = "retail-12-0-5")]
use rilua::table_security::{unwrap_secret, wrap_host_secret_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CAST_API: &str = "C_Spell.GetSpellCastCount";
#[cfg(feature = "retail-12-0-5")]
const DISPLAY_API: &str = "C_Spell.GetSpellDisplayCount";

pub(crate) fn get_spell_cast_count(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_public_spell_identifier_at(state, 1, CAST_API)?;
    let (quantity, restricted) = {
        let sim = borrow_state(state)?;
        (
            spell_id.and_then(|id| sim.spell_cast_counts.get(&id).copied()),
            cfg!(feature = "retail-12-0-5") && cooldowns_are_restricted(&sim),
        )
    };
    let number = f64::from(quantity.unwrap_or(0));
    let result = if restricted {
        wrap_host_secret_number(state, number)
    } else {
        Val::Num(number)
    };
    state.push(result);
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
const MAX_APPLICATIONS_API: &str = "C_Spell.GetSpellMaxCumulativeAuraApplications";

/// Explicit spell-keyed maximum aura stacks; zero for an undeclared spell (INFERRED).
/// Secret only under the explicit unit-aura restriction input.
#[cfg(feature = "retail-12-0-5")]
pub(crate) fn get_spell_max_cumulative_aura_applications(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_public_spell_identifier_at(state, 1, MAX_APPLICATIONS_API)?;
    let (maximum, restricted) = {
        let sim = borrow_state(state)?;
        let maximum = spell_id
            .and_then(|id| sim.spell_max_cumulative_aura_applications.get(&id).copied())
            .unwrap_or(0);
        (f64::from(maximum), sim.unit_auras_restricted)
    };
    let result = if restricted {
        wrap_host_secret_number(state, maximum)
    } else {
        Val::Num(maximum)
    };
    state.push(result);
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
pub(crate) fn get_spell_display_count(state: &mut LuaState) -> LuaResult<u32> {
    // Authenticate every original stack root before parsing or model access.
    let identifier = authenticate_argument(state, 1)?;
    let maximum_value = authenticate_argument(state, 2)?;
    let replacement_value = authenticate_argument(state, 3)?;
    validate_identifier(state, identifier)?;
    let maximum = parse_maximum(maximum_value, DISPLAY_API)?;
    let replacement = parse_replacement(state, replacement_value, DISPLAY_API)?;
    let spell_id = read_spell_identifier_value(state, identifier)?;
    let (quantity, restricted) = read_display_snapshot(state, spell_id)?;
    let display = format_display_count(quantity, maximum, &replacement);
    let result = if restricted {
        wrap_host_secret_string(state, &display)
    } else {
        create_string(state, &display)
    };
    state.push(result);
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn authenticate_argument(state: &LuaState, position: i32) -> LuaResult<Val> {
    unwrap_secret(state, stack_val(state, position)).map_err(|_| {
        runtime_error(format!(
            "{DISPLAY_API}: argument {position} requires an untainted caller"
        ))
    })
}

#[cfg(feature = "retail-12-0-5")]
fn validate_identifier(state: &LuaState, value: Val) -> LuaResult<()> {
    match value {
        Val::Num(number) if is_unsigned_u32(number) => Ok(()),
        Val::Str(_) if val_to_string(state, value).is_some() => Ok(()),
        _ => Err(runtime_error(format!(
            "{DISPLAY_API}: argument 1 requires a UTF-8 string or finite integral u32 number"
        ))),
    }
}

#[cfg(feature = "retail-12-0-5")]
fn is_unsigned_u32(number: f64) -> bool {
    number.is_finite() && f64::from(number as u32) == number
}

#[cfg(feature = "retail-12-0-5")]
fn read_display_snapshot(
    state: &LuaState,
    spell_id: Option<u32>,
) -> LuaResult<(Option<u32>, bool)> {
    // Copy inputs under one immutable borrow; release before VM allocation.
    let sim = borrow_state(state)?;
    let cast_quantity = spell_id.and_then(|id| sim.spell_cast_counts.get(&id).copied());
    let quantity = match read_charge_input(&sim, spell_id) {
        Some(charge) => Some(charge.current_charges),
        None => cast_quantity,
    };
    Ok((quantity, cooldowns_are_restricted(&sim)))
}
