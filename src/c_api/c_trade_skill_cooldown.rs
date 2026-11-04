//! Recipe cooldown projection from supplied spell history and charge state.

use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[path = "recipe_day_cooldown_data.rs"]
mod day_cooldown_data;

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

fn end_milliseconds(start: f64, duration: f64) -> u32 {
    milliseconds(start).wrapping_add(milliseconds(duration))
}

fn wrapped_milliseconds_to_seconds(remaining: u32) -> Option<f64> {
    // WM85b: the wrapped u32 is zero-extended before conversion to f32.
    let seconds = f64::from((remaining as f32) * 0.001_f32);
    (seconds > 0.0).then_some(seconds)
}

fn ordinary_end(start: f64, duration: f64, now: u32) -> Option<u32> {
    let end = end_milliseconds(start, duration);
    ((end.wrapping_sub(now) as i32) > 0).then_some(end)
}

fn charge_end(start: f64, duration: f64, now: u32) -> u32 {
    let end = end_milliseconds(start, duration);
    if end != 0 && (end.wrapping_sub(now) as i32) < 0 {
        now
    } else {
        end
    }
}

fn is_day_cooldown(recipe_id: i32) -> bool {
    u32::try_from(recipe_id).ok().is_some_and(|id| {
        day_cooldown_data::DAY_COOLDOWN_SPELLS
            .binary_search(&id)
            .is_ok()
    })
}

fn get_recipe_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let id = recipe_id(state)?;
    if profession_data::get_recipe(id).is_none() {
        return Ok(0);
    }
    let (remaining, charges, maximum) = {
        let sim = borrow_state(state)?;
        let now = milliseconds(sim.clock_elapsed().as_secs_f64());
        let spell_id = id as u32;
        let mut end = sim
            .spell_cooldowns
            .get(&spell_id)
            .and_then(|cooldown| ordinary_end(cooldown.start, cooldown.duration, now));
        let (charges, maximum) = match sim.spell_charges.get(&spell_id) {
            Some(charge) => {
                let maximum = charge.max_charges.min(255);
                if charge.current_charges < maximum {
                    end = Some(charge_end(
                        charge.recharge_start,
                        charge.recharge_duration,
                        now,
                    ));
                }
                (charge.current_charges, maximum)
            }
            None => (0, 0),
        };
        let final_now = milliseconds(sim.clock_elapsed().as_secs_f64());
        let remaining =
            end.and_then(|end| wrapped_milliseconds_to_seconds(end.wrapping_sub(final_now)));
        (remaining, charges, maximum)
    };
    state.push(remaining.map_or(Val::Nil, Val::Num));
    state.push(Val::Bool(is_day_cooldown(id)));
    state.push(Val::Num(f64::from(charges)));
    state.push(Val::Num(f64::from(maximum)));
    Ok(4)
}

#[cfg(test)]
mod tests {
    use super::wrapped_milliseconds_to_seconds;

    #[test]
    fn wrapped_high_bit_publishes_unsigned_seconds() {
        let seconds = wrapped_milliseconds_to_seconds(0x8000_0000).unwrap();
        assert!(seconds > 2_147_000.0 && seconds < 2_148_000.0);
    }

    #[test]
    fn zero_delta_publishes_nil() {
        assert_eq!(wrapped_milliseconds_to_seconds(0), None);
    }
}
