//! A_Admin death/release inputs: corpse recovery delay, resurrect offers, resurrection
//! sickness text, area spirit healer range and cemetery selection availability.

use crate::lua_api::globals::real::admin_fields as fields;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state_mut, create_string, val_to_string};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::time::Duration;

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetCorpseRecoveryDelay", set_corpse_recovery_delay)?
        .set_function("OfferResurrect", offer_resurrect)?
        .set_function("SetResSicknessDuration", set_res_sickness_duration)?
        .set_function("SetAreaSpiritHealer", set_area_spirit_healer)?
        .set_function("SetCemeterySelectionAvailable", |state| {
            let available = bool::from_stack(state, 1)?;
            borrow_state_mut(state)?
                .death_release
                .cemetery_selection_available = available;
            Ok(0)
        })
}

fn seconds_arg(state: &LuaState, api: &str) -> LuaResult<Duration> {
    match stack_val(state, 1) {
        Val::Num(seconds) if seconds.is_finite() && seconds >= 0.0 => {
            Ok(Duration::from_secs_f64(seconds))
        }
        _ => Err(runtime_error(format!(
            "{api} expects finite nonnegative seconds"
        ))),
    }
}

/// `A_Admin.SetCorpseRecoveryDelay(seconds)`: 0 clears the delay.
fn set_corpse_recovery_delay(state: &mut LuaState) -> LuaResult<u32> {
    let delay = seconds_arg(state, "A_Admin.SetCorpseRecoveryDelay")?;
    let mut sim = borrow_state_mut(state)?;
    sim.death_release.corpse_recovery_until = (!delay.is_zero()).then(|| sim.clock_now() + delay);
    Ok(0)
}

/// `A_Admin.OfferResurrect({offerer=, hasSickness=, hasTimer=})`: dispatches RESURRECT_REQUEST.
fn offer_resurrect(state: &mut LuaState) -> LuaResult<u32> {
    const API: &str = "A_Admin.OfferResurrect";
    let table = match stack_val(state, 1) {
        table @ Val::Table(_) => table,
        _ => return Err(runtime_error(format!("{API} expects an info table"))),
    };
    let offerer = fields::string(state, table, API, "offerer")?;
    let sickness = fields::boolean(state, table, API, "hasSickness")?;
    let timer = fields::boolean(state, table, API, "hasTimer")?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.pending_resurrect = Some(offerer.clone());
        sim.death_release.offer_has_sickness = sickness;
        sim.death_release.offer_has_timer = timer;
    }
    let inviter = create_string(state, &offerer);
    dispatch_event_now(state, "RESURRECT_REQUEST", &[inviter])?;
    Ok(0)
}

/// `A_Admin.SetResSicknessDuration(text|nil)`.
fn set_res_sickness_duration(state: &mut LuaState) -> LuaResult<u32> {
    let text = match stack_val(state, 1) {
        Val::Nil => None,
        value @ Val::Str(_) => val_to_string(state, value),
        _ => {
            return Err(runtime_error(
                "A_Admin.SetResSicknessDuration expects a string or nil",
            ));
        }
    };
    borrow_state_mut(state)?.death_release.res_sickness_duration = text;
    Ok(0)
}

/// `A_Admin.SetAreaSpiritHealer(secondsToWave|nil)`: entering or leaving a healer's
/// range raises AREA_SPIRIT_HEALER_IN_RANGE / AREA_SPIRIT_HEALER_OUT_OF_RANGE.
fn set_area_spirit_healer(state: &mut LuaState) -> LuaResult<u32> {
    let wave = match stack_val(state, 1) {
        Val::Nil => None,
        _ => Some(seconds_arg(state, "A_Admin.SetAreaSpiritHealer")?),
    };
    let event = {
        let mut sim = borrow_state_mut(state)?;
        let deadline = wave.map(|wave| sim.clock_now() + wave);
        sim.death_release.area_heal_wave = deadline;
        if deadline.is_none() {
            sim.death_release.area_heal_queued = false;
        }
        match deadline {
            Some(_) => "AREA_SPIRIT_HEALER_IN_RANGE",
            None => "AREA_SPIRIT_HEALER_OUT_OF_RANGE",
        }
    };
    dispatch_event_now(state, event, &[])?;
    Ok(0)
}
