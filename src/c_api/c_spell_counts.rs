//! Retail 12.1.5 cast counts use the Spell catalog and ordinary reagent limits.
//! Older profiles retain explicit count inputs; property and eligibility details
//! remain unmodeled. Output restriction is separate from inventory arithmetic.

#[cfg(feature = "retail-12-0-5")]
use super::c_action_bar_counts::format_display_count;
use super::c_spell::read_public_spell_identifier_at;
use super::c_spell::read_spell_identifier_value;
use super::charge_state::cooldowns_are_restricted;
#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
use super::charge_state::read_charge_input;
#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
use super::helpers::parse_replacement;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
use crate::lua_api::methods::{create_string, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::runtime_error;
use rilua::table_security::wrap_host_secret_number;
#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
use rilua::table_security::{unwrap_secret, wrap_host_secret_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CAST_API: &str = "C_Spell.GetSpellCastCount";
#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
const CAST_USAGE: &str = "Usage: local castCount = C_Spell.GetSpellCastCount(spellIdentifier)";
#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
const DISPLAY_API: &str = "C_Spell.GetSpellDisplayCount";
#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
const DISPLAY_USAGE: &str = concat!(
    "Usage: local displayCount = C_Spell.GetSpellDisplayCount(",
    "spellIdentifier [, maxDisplayCount, replacementString])"
);
#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
const DEFAULT_DISPLAY_MAXIMUM: f64 = 9999.0;
#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
const SPELL_REAGENT_PAIR_LIMIT: usize = 8;

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
fn native_count_contract_enabled() -> bool {
    // INFERRED: PTR and Forever declaration parity with VERIFIED Retail R88p/R88k/R88r.
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
}

fn secret_count_input_enabled() -> bool {
    matches!(ACTIVE, ClientProfile::WowForever)
        || cfg!(all(
            feature = "retail-12-0-5",
            any(feature = "profile-retail", feature = "client-ptr")
        ))
}

fn count_outputs_are_restricted(sim: &crate::lua_api::SimState) -> bool {
    // INFERRED Forever count-only output policy; shared charge policy is unchanged.
    if ACTIVE == ClientProfile::WowForever {
        sim.cooldowns_restricted
    } else {
        cfg!(feature = "retail-12-0-5") && cooldowns_are_restricted(sim)
    }
}

fn read_count_identifier(state: &LuaState, api: &str) -> LuaResult<Option<u32>> {
    let value = stack_val(state, 1);
    let enabled = secret_count_input_enabled();
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
    #[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
    if native_count_contract_enabled() {
        let value = stack_val(state, 1);
        if rilua::table_security::secret_payload_kind(state, value)
            == Some(rilua::table_security::SecretPayloadKind::Number)
        {
            // Preserve the existing cast numeric-secret access policy. R88r.r1
            // changes conversion only, after authentic host inspection succeeds.
            return rilua::table_security::inspect_host_secret_number(state, value, |number| {
                read_native_count_identifier(state, Val::Num(number), CAST_API, CAST_USAGE)
            });
        }
        if !rilua::table_security::is_secret_value(state, value) {
            return read_native_count_identifier(state, value, CAST_API, CAST_USAGE);
        }
    }
    // Other secret kinds and older-profile readers keep their existing errors.
    read_count_identifier(state, CAST_API)
}

#[cfg(feature = "retail-12-1-5")]
fn secret_spell_identifier_fails_metadata_gate(state: &LuaState, value: Val) -> LuaResult<bool> {
    let is_retail = ACTIVE == ClientProfile::Retail;
    let is_secret_number = rilua::table_security::secret_payload_kind(state, stack_val(state, 1))
        == Some(rilua::table_security::SecretPayloadKind::Number);
    if !is_retail || !is_secret_number {
        return Ok(false);
    }

    let source = borrow_state(state)?.spell_identifier_metadata_source;
    Ok(source.is_some_and(|source| source != 0)
        && matches!(value, Val::Num(number) if number <= 1.0))
}

#[cfg(all(feature = "client-wowforever", not(feature = "retail-12-1-5")))]
fn secret_spell_identifier_fails_metadata_gate(_state: &LuaState, _value: Val) -> LuaResult<bool> {
    Ok(false)
}

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
fn read_native_count_identifier(
    state: &LuaState,
    value: Val,
    api: &str,
    usage: &str,
) -> LuaResult<Option<u32>> {
    // VERIFIED (pr46-49-decomp-answers.md §R88r.r1): a nonzero source rejects
    // accepted secret numeric selectors <=1 with the required Usage error.
    if secret_spell_identifier_fails_metadata_gate(state, value)? {
        return Err(super::helpers::bad_argument(1, api, usage));
    }
    // Accepted secret numbers then use ordinary signed conversion.
    // Do not substitute caller taint, wrapper presence or allocation identity.
    let ordinary_or_number = !rilua::table_security::is_secret_value(state, stack_val(state, 1))
        || matches!(value, Val::Num(_));
    if ordinary_or_number && unrepresented_count_number(state, value)? {
        // UNKNOWN FP exception configuration: no represented identity is supplied.
        // This is not the maximum reader's range-error path or a wrap/clamp claim.
        return Ok(None);
    }
    if let Some(identifier) = read_numeric_or_link_count_string(state, value, api, usage)? {
        return Ok(identifier);
    }
    // INFERRED host name-resolution/override seam: existing spell_id_aliases is empty
    // by default. No native candidate-map contents or overridden value are asserted.
    super::c_spell::read_retail_spell_identifier(state, value, api, usage)
}

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
fn read_numeric_or_link_count_string(
    state: &LuaState,
    value: Val,
    api: &str,
    usage: &str,
) -> LuaResult<Option<Option<u32>>> {
    let Val::Str(reference) = value else {
        return Ok(None);
    };
    let Some(text) = state.gc.string_arena.get(reference) else {
        return Ok(None);
    };
    // VERIFIED R88r.r2: both string paths return before name replacement.
    if let Some(number) = super::helpers::parse_numeric_string(text.data()) {
        return super::c_spell::read_retail_spell_identifier(state, Val::Num(number), api, usage)
            .map(Some);
    }
    let parsed = std::str::from_utf8(text.data())
        .ok()
        .and_then(|text| super::item_spell::parse_prefixed_id(text, "spell"));
    Ok(parsed.map(Some))
}

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
fn unrepresented_count_number(state: &LuaState, value: Val) -> LuaResult<bool> {
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
    #[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
    if native_count_contract_enabled() {
        return spell_id.map_or(0.0, |id| retail_spell_cast_count(sim, id) as f64);
    }
    #[cfg(feature = "retail-12-0-0")]
    return f64::from(
        spell_id
            .and_then(|id| sim.spell_cast_counts.get(&id).copied())
            .unwrap_or(0),
    );
    #[cfg(not(feature = "retail-12-0-0"))]
    unreachable!("Forever must select the native count contract")
}

pub(crate) fn get_spell_cast_count(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_cast_count_identifier(state)?;
    let (number, restricted) = {
        let sim = borrow_state(state)?;
        (
            cast_count_number(&sim, spell_id),
            count_outputs_are_restricted(&sim),
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

#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
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

#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
fn read_display_identifier(state: &LuaState, identifier: Val) -> LuaResult<Option<u32>> {
    #[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
    if native_count_contract_enabled() {
        return read_native_count_identifier(state, identifier, DISPLAY_API, DISPLAY_USAGE);
    }
    validate_identifier(state, identifier)?;
    read_spell_identifier_value(state, identifier)
}

#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
fn read_display_text(
    state: &LuaState,
    spell_id: Option<u32>,
    maximum: f64,
    replacement: &str,
) -> LuaResult<(String, bool)> {
    #[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
    if native_count_contract_enabled() {
        return read_retail_display_text(state, spell_id, maximum, replacement);
    }
    #[cfg(feature = "retail-12-0-5")]
    {
        let (quantity, restricted) = read_display_snapshot(state, spell_id)?;
        return Ok((
            format_display_count(quantity, maximum, replacement),
            restricted,
        ));
    }
    #[cfg(not(feature = "retail-12-0-5"))]
    unreachable!("Forever must select the native count contract")
}

#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
fn parse_display_maximum(state: &LuaState, value: Val) -> LuaResult<f64> {
    if value == Val::Nil {
        return Ok(DEFAULT_DISPLAY_MAXIMUM);
    }
    // VERIFIED (pr46-49-decomp-answers.md §R88r.r1): signed-32 conversion
    // accepts secret 1, 0 and -1; it has no identifier <=1 gate.
    super::helpers::read_untainted_signed32_argument(state, 2, DISPLAY_API, DISPLAY_USAGE)
        .map(f64::from)
}

#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
fn parse_spell_replacement(state: &LuaState, value: Val) -> LuaResult<String> {
    // VERIFIED R88k.5 line 11622: ordinary numeric replacements convert to text.
    #[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
    if native_count_contract_enabled() {
        if let Val::Num(number) = value
            && !rilua::table_security::is_secret_value(state, stack_val(state, 3))
        {
            return Ok(Val::Num(number).to_string());
        }
    }
    parse_replacement(state, value, DISPLAY_API)
}

#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
fn authenticate_argument(state: &LuaState, position: i32) -> LuaResult<Val> {
    unwrap_secret(state, stack_val(state, position)).map_err(|_| {
        #[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
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

#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
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

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
fn read_retail_display_text(
    state: &LuaState,
    spell_id: Option<u32>,
    maximum: f64,
    replacement: &str,
) -> LuaResult<(String, bool)> {
    let sim = borrow_state(state)?;
    let uses_cast_count = spell_id.is_some_and(|id| has_reagent_display_mode(&sim, id));
    let restricted = count_outputs_are_restricted(&sim);
    let display = if uses_cast_count {
        let quantity = spell_id.map_or(0, |id| retail_spell_cast_count(&sim, id));
        format_retail_cast_display(quantity, maximum, replacement)
    } else {
        read_retail_charge_display(&sim, spell_id)
    };
    Ok((display, restricted))
}

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
fn read_retail_charge_display(sim: &crate::lua_api::SimState, spell_id: Option<u32>) -> String {
    let existing_spell = spell_id.filter(|id| crate::spells::get_spell(*id).is_some());
    let charge = existing_spell.and_then(|id| read_charge_input(sim, Some(id)));
    let multi_charge = charge.filter(|charge| charge.max_charges > 1);
    multi_charge
        .map(|charge| charge.current_charges.to_string())
        .unwrap_or_default()
}

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
fn has_reagent_display_mode(sim: &crate::lua_api::SimState, spell_id: u32) -> bool {
    sim.spell_reagent_pairs.get(&spell_id).is_some_and(|pairs| {
        pairs
            .iter()
            .take(SPELL_REAGENT_PAIR_LIMIT)
            .any(|(reagent_id, quantity)| *reagent_id != 0 && (*quantity as i16) > 0)
    })
}

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
fn format_retail_cast_display(count: i64, maximum: f64, replacement: &str) -> String {
    if count > maximum as i64 {
        replacement.to_owned()
    } else {
        count.to_string()
    }
}

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
fn retail_spell_cast_count(sim: &crate::lua_api::SimState, spell_id: u32) -> i64 {
    if crate::spells::get_spell(spell_id).is_none() || super::c_spell::spell_context_is_null(sim) {
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

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
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

#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
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
