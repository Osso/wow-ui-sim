//! Retail 12.1.5 cast counts use the Spell catalog and ordinary reagent limits.
//! Older profiles retain explicit count inputs; property and eligibility details
//! remain unmodeled. Output restriction is separate from inventory arithmetic.

#[cfg(feature = "retail-12-0-5")]
use super::c_action_bar_counts::{format_display_count, parse_replacement};
use super::c_spell::read_public_spell_identifier_at;
use super::c_spell::read_spell_identifier_value;
use super::charge_state::cooldowns_are_restricted;
#[cfg(feature = "retail-12-0-5")]
use super::charge_state::read_charge_input;
#[cfg(feature = "retail-12-1-5")]
use crate::client_profile::ClientProfile;
use crate::lua_api::methods::borrow_state;
#[cfg(feature = "retail-12-0-5")]
use crate::lua_api::methods::{create_string, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::runtime_error;
use rilua::table_security::wrap_host_secret_number;
#[cfg(feature = "retail-12-0-5")]
use rilua::table_security::{unwrap_secret, wrap_host_secret_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CAST_API: &str = "C_Spell.GetSpellCastCount";
#[cfg(feature = "retail-12-1-5")]
const CAST_USAGE: &str = "Usage: local castCount = C_Spell.GetSpellCastCount(spellIdentifier)";
#[cfg(feature = "retail-12-0-5")]
const DISPLAY_API: &str = "C_Spell.GetSpellDisplayCount";
#[cfg(feature = "retail-12-0-5")]
const DISPLAY_USAGE: &str = concat!(
    "Usage: local displayCount = C_Spell.GetSpellDisplayCount(",
    "spellIdentifier [, maxDisplayCount, replacementString])"
);
#[cfg(feature = "retail-12-0-5")]
const DEFAULT_DISPLAY_MAXIMUM: f64 = 9999.0;
#[cfg(feature = "retail-12-1-5")]
const SPELL_REAGENT_PAIR_LIMIT: usize = 8;

#[cfg(feature = "retail-12-1-5")]
fn native_count_contract_enabled() -> bool {
    // INFERRED: PTR applicability of the VERIFIED Retail R88p contract.
    matches!(
        crate::client_profile::ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr
    )
}

fn read_count_identifier(state: &LuaState, api: &str) -> LuaResult<Option<u32>> {
    let value = stack_val(state, 1);
    let enabled = cfg!(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ));
    if enabled
        && rilua::table_security::secret_payload_kind(state, value)
            == Some(rilua::table_security::SecretPayloadKind::Number)
    {
        return rilua::table_security::inspect_host_secret_number(state, value, |number| {
            if !is_unsigned_u32(number) {
                return Err(runtime_error(format!(
                    "{api}: argument 1 requires a finite integral u32 secret spell selector"
                )));
            }
            read_spell_identifier_value(state, Val::Num(number))
        });
    }
    read_public_spell_identifier_at(state, 1, api)
}

fn read_cast_count_identifier(state: &LuaState) -> LuaResult<Option<u32>> {
    #[cfg(feature = "retail-12-1-5")]
    if native_count_contract_enabled()
        && !rilua::table_security::is_secret_value(state, stack_val(state, 1))
    {
        return read_native_count_identifier(state, stack_val(state, 1), CAST_API, CAST_USAGE);
    }
    read_count_identifier(state, CAST_API)
}

#[cfg(feature = "retail-12-1-5")]
fn read_native_count_identifier(
    state: &LuaState,
    value: Val,
    api: &str,
    usage: &str,
) -> LuaResult<Option<u32>> {
    if !rilua::table_security::is_secret_value(state, stack_val(state, 1))
        && unrepresented_count_number(state, value)?
    {
        // UNKNOWN FP exception configuration: no represented identity is supplied.
        // This is not the maximum reader's range-error path or a wrap/clamp claim.
        return Ok(None);
    }
    super::c_spell::read_retail_spell_identifier(state, value, api, usage)
}

#[cfg(feature = "retail-12-1-5")]
fn unrepresented_count_number(state: &LuaState, value: Val) -> LuaResult<bool> {
    if matches!(value, Val::Str(_)) && read_spell_identifier_value(state, value)?.is_some() {
        return Ok(false); // Preserve the existing, not fully native-proven alias mapping.
    }
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|text| super::helpers::parse_numeric_string(text.data())),
        _ => None,
    };
    Ok(number.is_some_and(|n| !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&n)))
}

fn cast_count_number(sim: &crate::lua_api::SimState, spell_id: Option<u32>) -> f64 {
    #[cfg(feature = "retail-12-1-5")]
    if native_count_contract_enabled() {
        return spell_id.map_or(0.0, |id| retail_spell_cast_count(sim, id) as f64);
    }
    f64::from(
        spell_id
            .and_then(|id| sim.spell_cast_counts.get(&id).copied())
            .unwrap_or(0),
    )
}

pub(crate) fn get_spell_cast_count(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_cast_count_identifier(state)?;
    let (number, restricted) = {
        let sim = borrow_state(state)?;
        (
            cast_count_number(&sim, spell_id),
            cfg!(feature = "retail-12-0-5") && cooldowns_are_restricted(&sim),
        )
    };
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
    let spell_id = read_count_identifier(state, MAX_APPLICATIONS_API)?;
    let (maximum, restricted) = {
        let sim = borrow_state(state)?;
        let maximum = spell_id
            .and_then(|id| sim.spell_max_cumulative_aura_applications.get(&id).copied())
            .unwrap_or(0);
        (
            f64::from(maximum),
            super::unit_aura_access::aura_secrecy_policy_active(&sim),
        )
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
    let spell_id = read_display_identifier(state, identifier)?;
    let maximum = parse_display_maximum(state, maximum_value)?;
    let replacement = parse_spell_replacement(state, replacement_value)?;
    let (display, restricted) = read_display_text(state, spell_id, maximum, &replacement)?;
    let result = if restricted {
        wrap_host_secret_string(state, &display)
    } else {
        create_string(state, &display)
    };
    state.push(result);
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn read_display_identifier(state: &LuaState, identifier: Val) -> LuaResult<Option<u32>> {
    #[cfg(feature = "retail-12-1-5")]
    if native_count_contract_enabled() {
        return read_native_count_identifier(state, identifier, DISPLAY_API, DISPLAY_USAGE);
    }
    validate_identifier(state, identifier)?;
    read_spell_identifier_value(state, identifier)
}

#[cfg(feature = "retail-12-0-5")]
fn read_display_text(
    state: &LuaState,
    spell_id: Option<u32>,
    maximum: f64,
    replacement: &str,
) -> LuaResult<(String, bool)> {
    #[cfg(feature = "retail-12-1-5")]
    if native_count_contract_enabled() {
        return read_retail_display_text(state, spell_id, maximum, replacement);
    }
    let (quantity, restricted) = read_display_snapshot(state, spell_id)?;
    Ok((
        format_display_count(quantity, maximum, replacement),
        restricted,
    ))
}

#[cfg(feature = "retail-12-0-5")]
fn parse_display_maximum(state: &LuaState, value: Val) -> LuaResult<f64> {
    if value == Val::Nil {
        return Ok(DEFAULT_DISPLAY_MAXIMUM);
    }
    super::helpers::read_untainted_signed32_argument(state, 2, DISPLAY_API, DISPLAY_USAGE)
        .map(f64::from)
}

#[cfg(feature = "retail-12-0-5")]
fn parse_spell_replacement(state: &LuaState, value: Val) -> LuaResult<String> {
    // VERIFIED R88k.5 line 11622: ordinary numeric replacements convert to text.
    #[cfg(feature = "retail-12-1-5")]
    if native_count_contract_enabled() {
        if let Val::Num(number) = value
            && !rilua::table_security::is_secret_value(state, stack_val(state, 3))
        {
            return Ok(Val::Num(number).to_string());
        }
    }
    parse_replacement(state, value, DISPLAY_API)
}

#[cfg(feature = "retail-12-0-5")]
fn authenticate_argument(state: &LuaState, position: i32) -> LuaResult<Val> {
    unwrap_secret(state, stack_val(state, position)).map_err(|_| {
        #[cfg(feature = "retail-12-1-5")]
        if native_count_contract_enabled() {
            // VERIFIED R88p.p2: modeled mode 1 denies before conversion.
            let reason = format!(
                "{DISPLAY_USAGE}. Secret values are only allowed during untainted execution for this argument."
            );
            return super::helpers::bad_argument(position, DISPLAY_API, &reason);
        }
        runtime_error(format!("{DISPLAY_API}: argument {position} requires an untainted caller"))
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

fn is_unsigned_u32(number: f64) -> bool {
    number.is_finite() && f64::from(number as u32) == number
}

#[cfg(feature = "retail-12-1-5")]
fn read_retail_display_text(
    state: &LuaState,
    spell_id: Option<u32>,
    maximum: f64,
    replacement: &str,
) -> LuaResult<(String, bool)> {
    let sim = borrow_state(state)?;
    let uses_cast_count = spell_id.is_some_and(|id| has_reagent_display_mode(&sim, id));
    let restricted = cooldowns_are_restricted(&sim);
    let display = if uses_cast_count {
        let quantity = spell_id.map_or(0, |id| retail_spell_cast_count(&sim, id));
        format_retail_cast_display(quantity, maximum, replacement)
    } else {
        read_retail_charge_display(&sim, spell_id)
    };
    Ok((display, restricted))
}

#[cfg(feature = "retail-12-1-5")]
fn read_retail_charge_display(sim: &crate::lua_api::SimState, spell_id: Option<u32>) -> String {
    let existing_spell = spell_id.filter(|id| crate::spells::SPELL_DB.get(id).is_some());
    let charge = existing_spell.and_then(|id| read_charge_input(sim, Some(id)));
    let multi_charge = charge.filter(|charge| charge.max_charges > 1);
    multi_charge
        .map(|charge| charge.current_charges.to_string())
        .unwrap_or_default()
}

#[cfg(feature = "retail-12-1-5")]
fn has_reagent_display_mode(sim: &crate::lua_api::SimState, spell_id: u32) -> bool {
    sim.spell_reagent_pairs.get(&spell_id).is_some_and(|pairs| {
        pairs
            .iter()
            .take(SPELL_REAGENT_PAIR_LIMIT)
            .any(|(reagent_id, quantity)| *reagent_id != 0 && (*quantity as i16) > 0)
    })
}

#[cfg(feature = "retail-12-1-5")]
fn format_retail_cast_display(count: i64, maximum: f64, replacement: &str) -> String {
    if count > maximum as i64 {
        replacement.to_owned()
    } else {
        count.to_string()
    }
}

#[cfg(feature = "retail-12-1-5")]
fn retail_spell_cast_count(sim: &crate::lua_api::SimState, spell_id: u32) -> i64 {
    if crate::spells::SPELL_DB.get(&spell_id).is_none()
        || super::c_spell::spell_context_is_null(sim)
    {
        return 0;
    }
    let Some(reagents) = sim.spell_reagent_pairs.get(&spell_id) else {
        return 0;
    };
    // VERIFIED R88p.p3: eight pairs, signed16 quantities and ordinary limits.
    // UNKNOWN: alternate-property backing and active-record limits stay absent.
    let limited_reagents = reagents.iter().take(SPELL_REAGENT_PAIR_LIMIT);
    let cast_limits =
        limited_reagents.filter_map(|(id, quantity)| reagent_cast_limit(sim, *id, *quantity));
    let minimum_count = cast_limits.min();
    minimum_count.map(|count| count.max(0)).unwrap_or(0)
}

#[cfg(feature = "retail-12-1-5")]
fn reagent_cast_limit(
    sim: &crate::lua_api::SimState,
    reagent_id: i32,
    quantity: i32,
) -> Option<i64> {
    let quantity = quantity as i16;
    if reagent_id == 0 || reagent_id == -2 || quantity <= 0 {
        return None;
    }
    if let Some(limit) = super::spell_reagent_inventory::alternate_limit(sim, reagent_id, quantity)
    {
        return Some(limit);
    }
    Some(matching_reagent_count(sim, reagent_id) / i64::from(quantity))
}

#[cfg(feature = "retail-12-1-5")]
fn matching_reagent_count(sim: &crate::lua_api::SimState, reagent_id: i32) -> i64 {
    super::spell_reagent_inventory::matching_count(sim, reagent_id)
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
