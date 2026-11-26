//! INFERRED local command boundary; requests are not authoritative server replies.
use super::pet_battle_runtime_model::PetBattleRequest;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    super::pet_battle_runtime_queries::register_queries(state)?;
    let ns = super::ensure_namespace(state, "C_PetBattles")?;
    for (name, function) in [
        (
            "GetPVPMatchmakingInfo",
            matchmaking_info as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("CanAcceptQueuedPVPMatch", can_accept),
        ("StartPVPMatchmaking", start_matchmaking),
        ("AcceptQueuedPVPMatch", accept_match),
        ("UseAbility", use_ability),
        ("ChangePet", change_pet),
        ("UseTrap", use_trap),
        ("SkipTurn", skip_turn),
        ("StartPVPDuel", start_duel),
        ("AcceptPVPDuel", accept_duel),
        ("SetPendingReportBattlePetTarget", report_pet),
        ("SetPendingReportTargetFromUnit", report_unit),
        ("ForfeitGame", forfeit),
    ] {
        table_set_rust_fn_static(state, ns, name, function)?;
    }
    Ok(())
}

fn matchmaking_info(state: &mut LuaState) -> LuaResult<u32> {
    let queue = borrow_state(state)?.pet_battles.runtime.matchmaking.clone();
    // VERIFIED R88f.D.63 publication mapping; zero timer defaults are INFERRED.
    let status = match queue.status {
        0 => Val::Nil,
        1 => create_string(state, "queued"),
        2 => create_string(state, "suspended"),
        3 => create_string(state, "proposal"),
        4 => create_string(state, "entry"),
        _ => return Err(runtime_error("unsupported host pet matchmaking state")),
    };
    state.push(status);
    state.push(Val::Num(f64::from(queue.estimated_time)));
    state.push(Val::Num(f64::from(queue.queued_time_ms) * 0.001));
    Ok(3)
}

fn can_accept(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED supplied eligibility, not a native validation implementation.
    let ready = {
        let sim = borrow_state(state)?;
        let queue = &sim.pet_battles.runtime.matchmaking;
        queue.status == 3 && queue.can_accept
    };
    state.push(Val::Bool(ready));
    Ok(1)
}

fn start_matchmaking(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.pet_battles.is_matchmaking = true;
    sim.pet_battles
        .runtime
        .requests
        .push(PetBattleRequest::StartMatchmaking);
    Ok(0)
}

fn accept_match(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let runtime = &mut sim.pet_battles.runtime;
    // VERIFIED R88f.D.66 checked block: proposal state 3 transitions to 4.
    if runtime.matchmaking.status == 3 {
        runtime.requests.push(PetBattleRequest::AcceptMatch);
        runtime.matchmaking.status = 4;
        runtime.matchmaking.can_accept = false;
    }
    Ok(0)
}

fn select_action(state: &mut LuaState, action: i32, selector: Option<i32>) -> LuaResult<u32> {
    // INFERRED compatibility: local selection only, not transport or round rules.
    borrow_state_mut(state)?.pet_battles.runtime.selected_action = Some((action, selector));
    Ok(0)
}

fn use_ability(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<i32>::from_stack(state, 1)?;
    select_action(state, 1, index)
}

fn change_pet(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<i32>::from_stack(state, 1)?;
    select_action(state, 2, index)
}

fn use_trap(state: &mut LuaState) -> LuaResult<u32> {
    select_action(state, 3, None)
}
fn skip_turn(state: &mut LuaState) -> LuaResult<u32> {
    select_action(state, 4, None)
}

fn request(state: &mut LuaState, request: PetBattleRequest) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .pet_battles
        .runtime
        .requests
        .push(request);
    Ok(0)
}

fn start_duel(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let exact_match = Option::<bool>::from_stack(state, 2)?.unwrap_or(false);
    request(state, PetBattleRequest::StartDuel { unit, exact_match })
}

fn accept_duel(state: &mut LuaState) -> LuaResult<u32> {
    request(state, PetBattleRequest::AcceptDuel)
}

fn report_pet(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    request(state, PetBattleRequest::ReportPet(slot))
}

fn report_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    request(state, PetBattleRequest::ReportUnit(unit))
}

fn forfeit(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED local lifecycle, not server acknowledgement. Finished=6 is declared
    // in both pinned PetBattleConstantsDocumentation.lua:64.
    borrow_state_mut(state)?.pet_battles.battle_state = 6;
    Ok(0)
}
