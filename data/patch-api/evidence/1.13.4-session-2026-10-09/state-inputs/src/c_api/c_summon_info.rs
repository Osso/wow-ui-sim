//! `C_SummonInfo` and `C_IncomingSummon` probe surface backed by
//! `SimState.summon_request`.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::stack_val;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_summon_info_surface(state: &mut LuaState) -> LuaResult<()> {
    register_summon_info_methods(state)?;
    register_incoming_summon_methods(state)
}

fn register_summon_info_methods(state: &mut LuaState) -> LuaResult<()> {
    let summon_info = ensure_namespace(state, "C_SummonInfo")?;
    table_set_rust_fn_static(
        state,
        summon_info,
        "GetSummonReason",
        c_summon_info_get_reason,
    )?;
    table_set_rust_fn_static(
        state,
        summon_info,
        "GetSummonConfirmTimeLeft",
        c_summon_info_get_confirm_time_left,
    )?;
    table_set_rust_fn_static(
        state,
        summon_info,
        "IsSummonSkippingStartExperience",
        c_summon_info_is_skipping_start_experience,
    )?;
    Ok(())
}

fn register_incoming_summon_methods(state: &mut LuaState) -> LuaResult<()> {
    let incoming = ensure_namespace(state, "C_IncomingSummon")?;
    table_set_rust_fn_static(
        state,
        incoming,
        "HasIncomingSummon",
        c_incoming_summon_has_incoming_summon,
    )?;
    table_set_rust_fn_static(
        state,
        incoming,
        "IncomingSummonStatus",
        c_incoming_summon_status,
    )?;
    Ok(())
}

fn c_summon_info_get_reason(state: &mut LuaState) -> LuaResult<u32> {
    let reason = {
        let sim = borrow_state(state)?;
        sim.summon_request
            .active
            .then_some(sim.summon_request.reason as f64)
    };
    if let Some(reason) = reason {
        state.push(Val::Num(reason));
    } else {
        state.push(Val::Nil);
    }
    Ok(1)
}

fn c_summon_info_get_confirm_time_left(state: &mut LuaState) -> LuaResult<u32> {
    let ms = borrow_state(state)?.summon_request.time_left_ms;
    state.push(Val::Num(ms as f64));
    Ok(1)
}

fn c_summon_info_is_skipping_start_experience(state: &mut LuaState) -> LuaResult<u32> {
    let flag = borrow_state(state)?.summon_request.skips_start_experience;
    state.push(Val::Bool(flag));
    Ok(1)
}

fn c_incoming_summon_has_incoming_summon(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    let active = borrow_state(state)?.summon_request.active;
    state.push(Val::Bool(active && unit == "player"));
    Ok(1)
}

fn c_incoming_summon_status(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    let active = borrow_state(state)?.summon_request.active;
    let status = if active && unit == "player" { 1.0 } else { 0.0 };
    state.push(Val::Num(status));
    Ok(1)
}
