//! Explicit spell-keyed charge input; no automatic charge progression.

use crate::lua_api::SimState;
use crate::lua_api::globals::lua_duration_object::push_timed_duration_object_with_rate;
use crate::lua_api::methods::{borrow_state, create_table_with_capacity, table_set_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpellChargeState {
    pub current_charges: u32,
    pub max_charges: u32,
    pub recharge_start: f64,
    pub recharge_duration: f64,
    pub charge_mod_rate: f64,
}

pub(crate) fn read_charge_input(sim: &SimState, spell_id: Option<u32>) -> Option<SpellChargeState> {
    sim.spell_charges
        .get(&spell_id?)
        .filter(|charge| charge.max_charges > 0)
        .copied()
}

pub(crate) fn cooldowns_are_restricted(sim: &SimState) -> bool {
    cfg!(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    )) && sim.cooldowns_restricted
}

/// Charge action/book selectors use AllowedWhenUntainted, not output policy.
/// Public values remain accessible to tainted callers; secret values use the VM guard.
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub(crate) fn read_charge_selector_number(state: &LuaState, index: i32) -> LuaResult<Option<f64>> {
    let value =
        rilua::table_security::unwrap_secret(state, crate::lua_bridge::stack_val(state, index))?;
    Ok(match value {
        Val::Num(number) => Some(number),
        _ => None,
    })
}

pub(crate) fn push_charge_info(state: &mut LuaState, spell_id: Option<u32>) -> LuaResult<u32> {
    let (charge, restricted) = {
        let sim = borrow_state(state)?;
        (
            read_charge_input(&sim, spell_id),
            cooldowns_are_restricted(&sim),
        )
    };
    let Some(charge) = charge else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let info = create_table_with_capacity(state, 6);
    // Keep the result reachable while host-secret wrappers allocate.
    state.push(info);
    table_set_static(
        state,
        info,
        "maxCharges",
        Val::Num(charge.max_charges as f64),
    );
    for (name, number) in [
        ("currentCharges", charge.current_charges as f64),
        ("cooldownStartTime", charge.recharge_start),
        ("cooldownDuration", charge.recharge_duration),
        ("chargeModRate", charge.charge_mod_rate),
    ] {
        let value = if restricted {
            rilua::table_security::wrap_host_secret_number(state, number)
        } else {
            Val::Num(number)
        };
        table_set_static(state, info, name, value);
    }
    #[cfg(feature = "retail-12-0-5")]
    table_set_static(state, info, "isActive", Val::Bool(charge_is_active(charge)));
    Ok(1)
}

fn charge_is_active(charge: SpellChargeState) -> bool {
    let recharging = charge.max_charges > 1 && charge.current_charges < charge.max_charges;
    let has_interval = charge.recharge_start > 0.0 && charge.recharge_duration > 0.0;
    recharging && has_interval
}

fn select_recharge_times(charge: SpellChargeState, now: f64) -> Option<(f64, f64, f64)> {
    let active = if cfg!(feature = "retail-12-0-5") {
        charge_is_active(charge)
    } else {
        charge.current_charges < charge.max_charges
    };
    if active {
        return Some((
            charge.recharge_start,
            charge.recharge_duration,
            charge.charge_mod_rate,
        ));
    }
    // 12.0.5 specifies zero-span at max; earlier active-only behavior is inferred.
    cfg!(feature = "retail-12-0-5").then_some((now, 0.0, charge.charge_mod_rate))
}

pub(crate) fn push_charge_duration(state: &mut LuaState, spell_id: Option<u32>) -> LuaResult<u32> {
    let timing = {
        let sim = borrow_state(state)?;
        let now = sim.start_time.elapsed().as_secs_f64();
        read_charge_input(&sim, spell_id).and_then(|charge| select_recharge_times(charge, now))
    };
    let Some((start, seconds, rate)) = timing else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_timed_duration_object_with_rate(state, start, seconds, rate)
}
