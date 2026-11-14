//! Retail 12.1.5 cast counts use the Spell catalog and ordinary reagent limits.
//! Older profiles retain explicit count inputs; unsupported Retail metadata modes
//! and inventory inclusion rules remain unmodeled. Output restriction is separate.

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

pub(crate) fn get_spell_cast_count(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-1-5")]
    let spell_id = if crate::client_profile::ACTIVE == ClientProfile::Retail
        && !rilua::table_security::is_secret_value(state, stack_val(state, 1))
    {
        super::c_spell::read_retail_spell_identifier(
            state,
            stack_val(state, 1),
            CAST_API,
            CAST_USAGE,
        )?
    } else {
        read_count_identifier(state, CAST_API)?
    };
    #[cfg(not(feature = "retail-12-1-5"))]
    let spell_id = read_count_identifier(state, CAST_API)?;
    let (number, restricted) = {
        let sim = borrow_state(state)?;
        #[cfg(feature = "retail-12-1-5")]
        let number = if crate::client_profile::ACTIVE == ClientProfile::Retail {
            spell_id.map_or(0.0, |id| retail_spell_cast_count(&sim, id) as f64)
        } else {
            f64::from(
                spell_id
                    .and_then(|id| sim.spell_cast_counts.get(&id).copied())
                    .unwrap_or(0),
            )
        };
        #[cfg(not(feature = "retail-12-1-5"))]
        let number = f64::from(
            spell_id
                .and_then(|id| sim.spell_cast_counts.get(&id).copied())
                .unwrap_or(0),
        );
        (
            number,
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
    #[cfg(feature = "retail-12-1-5")]
    let spell_id = if crate::client_profile::ACTIVE == ClientProfile::Retail {
        super::c_spell::read_retail_spell_identifier(state, identifier, DISPLAY_API, DISPLAY_USAGE)?
    } else {
        validate_identifier(state, identifier)?;
        read_spell_identifier_value(state, identifier)?
    };
    #[cfg(not(feature = "retail-12-1-5"))]
    let spell_id = {
        validate_identifier(state, identifier)?;
        read_spell_identifier_value(state, identifier)?
    };
    let maximum = parse_display_maximum(state, maximum_value)?;
    let replacement = parse_replacement(state, replacement_value, DISPLAY_API)?;
    #[cfg(feature = "retail-12-1-5")]
    let (display, restricted) = if crate::client_profile::ACTIVE == ClientProfile::Retail {
        read_retail_display_text(state, spell_id, maximum, &replacement)?
    } else {
        let (quantity, restricted) = read_display_snapshot(state, spell_id)?;
        (
            format_display_count(quantity, maximum, &replacement),
            restricted,
        )
    };
    #[cfg(not(feature = "retail-12-1-5"))]
    let (display, restricted) = {
        let (quantity, restricted) = read_display_snapshot(state, spell_id)?;
        (
            format_display_count(quantity, maximum, &replacement),
            restricted,
        )
    };
    let result = if restricted {
        wrap_host_secret_string(state, &display)
    } else {
        create_string(state, &display)
    };
    state.push(result);
    Ok(1)
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
        spell_id
            .filter(|id| crate::spells::SPELL_DB.get(id).is_some())
            .and_then(|id| read_charge_input(&sim, Some(id)))
            .filter(|charge| charge.max_charges > 1)
            .map(|charge| charge.current_charges.to_string())
            .unwrap_or_default()
    };
    Ok((display, restricted))
}

#[cfg(feature = "retail-12-1-5")]
fn has_reagent_display_mode(sim: &crate::lua_api::SimState, spell_id: u32) -> bool {
    sim.spell_reagent_pairs.get(&spell_id).is_some_and(|pairs| {
        pairs
            .iter()
            .take(SPELL_REAGENT_PAIR_LIMIT)
            .any(|(reagent_id, quantity)| *reagent_id != 0 && *quantity > 0)
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
    // Only the ordinary inventory path is modeled. Metadata masks 2 and 32,
    // alternate inventory counting, inclusion rules, and active-record limits
    // have no matching SimState inputs and remain UNKNOWN.
    reagents
        .iter()
        .take(SPELL_REAGENT_PAIR_LIMIT)
        .filter_map(|(reagent_id, quantity)| {
            if *reagent_id == 0 || *reagent_id == -2 || *quantity <= 0 {
                return None;
            }
            let inventory_count = sim
                .bag_items
                .values()
                .filter(|item| i64::from(item.item_id) == i64::from(*reagent_id))
                .map(|item| i64::from(item.stack_count))
                .sum::<i64>();
            Some(inventory_count / i64::from(*quantity))
        })
        .min()
        .map(|count| count.max(0))
        .unwrap_or(0)
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
