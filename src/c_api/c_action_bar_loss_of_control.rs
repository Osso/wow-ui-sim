//! Retail 12.0.5 action LoC snapshots over existing slot and spell inputs.
//! Strict slots, inactive misses and verbatim flags are inferred policies.

use super::charge_state::cooldowns_are_restricted;
use crate::lua_api::LossOfControlInfo;
use crate::lua_api::methods::{borrow_state, create_table_with_capacity, table_set_static};
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const API_NAME: &str = "C_ActionBar.GetActionLossOfControlCooldownInfo";

pub(crate) fn get_action_loss_of_control_cooldown_info(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_action_slot(state)?;
    let (info, restricted) = {
        let sim = borrow_state(state)?;
        let info = sim
            .action_bars
            .get(&slot)
            .and_then(|spell| sim.spell_loss_of_control.get(spell))
            .cloned()
            .unwrap_or(LossOfControlInfo {
                start_time: 0.0,
                duration: 0.0,
                mod_rate: 1.0,
                is_active: false,
                should_replace_normal_cooldown: false,
            });
        (info, cooldowns_are_restricted(&sim))
    };
    let table = create_table_with_capacity(state, 5);
    // Root the ordinary result before keys or host-secret wrappers allocate.
    state.push(table);
    for (key, number) in [
        ("startTime", info.start_time),
        ("duration", info.duration),
        ("modRate", f64::from(info.mod_rate)),
    ] {
        let value = if restricted {
            rilua::table_security::wrap_host_secret_number(state, number)
        } else {
            Val::Num(number)
        };
        table_set_static(state, table, key, value);
    }
    table_set_static(state, table, "isActive", Val::Bool(info.is_active));
    table_set_static(
        state,
        table,
        "shouldReplaceNormalCooldown",
        Val::Bool(info.should_replace_normal_cooldown),
    );
    Ok(1)
}

fn read_action_slot(state: &LuaState) -> LuaResult<u32> {
    // Authenticate the original rooted argument before type checks or model access.
    let value = unwrap_secret(state, stack_val(state, 1))
        .map_err(|error| rilua::runtime_error(format!("{API_NAME}: argument 1: {error}")))?;
    if let Val::Num(number) = value {
        if is_positive_u32(number) {
            return Ok(number as u32);
        }
    }
    Err(rilua::runtime_error(format!(
        "{API_NAME}: argument 1 requires a finite integral positive u32 number"
    )))
}

fn is_positive_u32(number: f64) -> bool {
    // The cast round trip excludes nonfinite, fractional and overflow values.
    number > 0.0 && f64::from(number as u32) == number
}
