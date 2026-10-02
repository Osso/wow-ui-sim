//! Retail 12.0.5 action counts over explicit slot inputs and existing charges.
//! Selection, strict domains and empty/zero privacy policies are inferred.

use super::charge_state::{cooldowns_are_restricted, read_charge_input};
use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, create_string, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::table_security::{unwrap_secret, wrap_host_secret_number, wrap_host_secret_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const DISPLAY_API: &str = "C_ActionBar.GetActionDisplayCount";
const USE_API: &str = "C_ActionBar.GetActionUseCount";

pub(crate) fn get_action_use_count(state: &mut LuaState) -> LuaResult<u32> {
    let argument = authenticate_argument(state, USE_API, 1)?;
    let slot = parse_slot(argument, USE_API)?;
    let (quantity, _, restricted) = read_count_snapshot(state, slot)?;
    let number = f64::from(quantity.unwrap_or(0));
    let result = if restricted {
        wrap_host_secret_number(state, number)
    } else {
        Val::Num(number)
    };
    state.push(result);
    Ok(1)
}

pub(crate) fn get_action_display_count(state: &mut LuaState) -> LuaResult<u32> {
    // Authenticate all original stack roots before parsing or copying strings.
    let slot_value = authenticate_argument(state, DISPLAY_API, 1)?;
    let maximum_value = authenticate_argument(state, DISPLAY_API, 2)?;
    let replacement_value = authenticate_argument(state, DISPLAY_API, 3)?;
    let slot = parse_slot(slot_value, DISPLAY_API)?;
    let maximum = parse_maximum(maximum_value)?;
    let replacement = parse_replacement(state, replacement_value)?;
    let (_, quantity, restricted) = read_count_snapshot(state, slot)?;
    let display = format_display_count(quantity, maximum, &replacement);
    let result = if restricted {
        wrap_host_secret_string(state, &display)
    } else {
        create_string(state, &display)
    };
    state.push(result);
    Ok(1)
}

fn authenticate_argument(state: &LuaState, api: &str, position: i32) -> LuaResult<Val> {
    unwrap_secret(state, stack_val(state, position)).map_err(|_| {
        runtime_error(format!(
            "{api}: argument {position} requires an untainted caller"
        ))
    })
}

fn parse_slot(value: Val, api: &str) -> LuaResult<u32> {
    if let Val::Num(number) = value {
        if is_positive_u32(number) {
            return Ok(number as u32);
        }
    }
    Err(runtime_error(format!(
        "{api}: argument 1 requires a finite integral positive u32 number"
    )))
}

fn is_positive_u32(number: f64) -> bool {
    number.is_finite() && number > 0.0 && f64::from(number as u32) == number
}

fn parse_maximum(value: Val) -> LuaResult<f64> {
    match value {
        Val::Nil => Ok(9999.0),
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(runtime_error(format!(
            "{DISPLAY_API}: argument 2 requires a finite number or nil"
        ))),
    }
}

fn parse_replacement(state: &LuaState, value: Val) -> LuaResult<String> {
    if value == Val::Nil {
        return Ok("*".to_owned());
    }
    val_to_string(state, value)
        .filter(|text| !text.contains('\0'))
        .ok_or_else(|| {
            runtime_error(format!(
                "{DISPLAY_API}: argument 3 requires a UTF-8 NUL-free string or nil"
            ))
        })
}

fn matching_use_quantity(sim: &SimState, slot: u32, spell_id: Option<u32>) -> Option<u32> {
    let spell_id = spell_id?;
    sim.action_use_counts
        .get(&slot)
        .filter(|input| input.spell_id == spell_id)
        .map(|input| input.count)
}

fn read_count_snapshot(state: &LuaState, slot: u32) -> LuaResult<(Option<u32>, Option<u32>, bool)> {
    // One immutable borrow; no pruning or VM allocations while it is held.
    let sim = borrow_state(state)?;
    let spell_id = sim.action_bars.get(&slot).copied();
    let use_quantity = matching_use_quantity(&sim, slot, spell_id);
    let display_quantity = match read_charge_input(&sim, spell_id) {
        Some(charge) => Some(charge.current_charges),
        None => use_quantity,
    };
    Ok((
        use_quantity,
        display_quantity,
        cooldowns_are_restricted(&sim),
    ))
}

pub(crate) fn format_display_count(
    quantity: Option<u32>,
    maximum: f64,
    replacement: &str,
) -> String {
    let Some(quantity) = quantity else {
        return String::new();
    };
    if f64::from(quantity) > maximum {
        return replacement.to_owned();
    }
    quantity.to_string()
}
