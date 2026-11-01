//! `C_ReportSystem` probe surface backed by report state.
//!
//! Report tokens and pending reports live in `SimState.pending_player_reports`
//! so tests and addon flows can inspect or clear them deterministically.

use crate::c_api::helpers::ensure_namespace;
use crate::event::{Event, EventArg};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::PendingPlayerReport;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_report_system_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_ReportSystem")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "InitiateReportPlayer",
        initiate_report_player,
    )?;
    table_set_rust_fn_static(state, table_ref, "SendReportPlayer", send_report_player)?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        table_set_rust_fn_static(state, table_ref, "CanReportPlayer", can_report_player)?;
    }
    Ok(())
}

fn can_report_player(state: &mut LuaState) -> LuaResult<u32> {
    if !matches!(crate::lua_bridge::stack_val(state, 1), Val::Table(_)) {
        return Err(rilua::runtime_error(
            "playerLocation requires a PlayerLocation table",
        ));
    }
    let eligible = borrow_state(state)?.report_player_eligible;
    state.push(Val::Bool(eligible));
    Ok(1)
}

fn initiate_report_player(state: &mut LuaState) -> LuaResult<u32> {
    let report_type = String::from_stack(state, 1)?;
    let _location = crate::lua_bridge::stack_val(state, 2);
    let token = {
        let mut sim = borrow_state_mut(state)?;
        let token = sim.next_report_token;
        sim.next_report_token += 1;
        sim.pending_player_reports.insert(
            token,
            PendingPlayerReport {
                report_type: report_type.clone(),
                comment: None,
            },
        );
        token
    };

    state.push(Val::Num(token as f64));
    Ok(1)
}

fn send_report_player(state: &mut LuaState) -> LuaResult<u32> {
    let token = i64::from_stack(state, 1)?;
    let comment = Option::<String>::from_stack(state, 2)?;
    let report = {
        let mut sim = borrow_state_mut(state)?;
        let Some(mut report) = sim.pending_player_reports.remove(&token) else {
            return Ok(0);
        };
        report.comment = comment;
        report
    };

    queue_report_result(state, &report.report_type);
    Ok(0)
}

fn queue_report_result(state: &mut LuaState, report_type: &str) {
    let mut sim = match borrow_state_mut(state) {
        Ok(sim) => sim,
        Err(_) => return,
    };
    sim.events.push(Event {
        name: "REPORT_PLAYER_RESULT".to_string(),
        args: vec![
            EventArg::Number(0.0),
            EventArg::String(report_type.to_string()),
        ],
    });
}
