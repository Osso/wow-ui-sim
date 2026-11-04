//! Recipe cooldown projection from supplied spell history and charge state.

use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail
        || !cfg!(feature = "retail-12-1-5")
    {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_TradeSkillUI")?;
    table_set_rust_fn_static(state, namespace, "GetRecipeCooldown", get_recipe_cooldown)
}

fn recipe_id(state: &LuaState) -> LuaResult<i32> {
    const USAGE: &str = "Usage: GetRecipeCooldown(recipeID)";
    let original = stack_val(state, 1);
    let marked = rilua::table_security::is_secret_value(state, original);
    let value =
        rilua::table_security::unwrap_secret(state, original).map_err(|_| runtime_error(USAGE))?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) if !marked => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| super::helpers::parse_numeric_string(string.data())),
        _ => None,
    }
    .ok_or_else(|| runtime_error(USAGE))?;
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(runtime_error(format!(
            "integer overflow attempting to store {number:.6}"
        )));
    }
    Ok(number.trunc() as i32)
}

fn milliseconds(seconds: f64) -> u32 {
    (seconds * 1000.0) as u64 as u32
}

fn remaining_seconds(start: f64, duration: f64, now: f64) -> Option<f64> {
    let remaining = milliseconds(start)
        .wrapping_add(milliseconds(duration))
        .wrapping_sub(milliseconds(now));
    let seconds = wrapped_milliseconds_to_seconds(remaining);
    (seconds > 0.0).then_some(seconds)
}

fn wrapped_milliseconds_to_seconds(remaining: u32) -> f64 {
    // UNKNOWN (WM85b): signedness before float conversion, especially for stale records.
    f64::from((remaining as f32) * 0.001_f32)
}

fn is_day_cooldown() -> bool {
    // UNKNOWN: category flag 8 is not yet supplied; false is an EMULATED default.
    false
}

fn get_recipe_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let id = recipe_id(state)?;
    if profession_data::get_recipe(id).is_none() {
        return Ok(0);
    }
    let (remaining, charges, maximum) = {
        let sim = borrow_state(state)?;
        let now = sim.clock_elapsed().as_secs_f64();
        let spell_id = id as u32;
        let mut remaining = sim
            .spell_cooldowns
            .get(&spell_id)
            .and_then(|cooldown| remaining_seconds(cooldown.start, cooldown.duration, now));
        let (charges, maximum) = match sim.spell_charges.get(&spell_id) {
            Some(charge) => {
                let maximum = charge.max_charges.min(255);
                if charge.current_charges < maximum {
                    remaining =
                        remaining_seconds(charge.recharge_start, charge.recharge_duration, now);
                }
                (charge.current_charges, maximum)
            }
            None => (0, 0),
        };
        (remaining, charges, maximum)
    };
    state.push(remaining.map_or(Val::Nil, Val::Num));
    state.push(Val::Bool(is_day_cooldown()));
    state.push(Val::Num(f64::from(charges)));
    state.push(Val::Num(f64::from(maximum)));
    Ok(4)
}
