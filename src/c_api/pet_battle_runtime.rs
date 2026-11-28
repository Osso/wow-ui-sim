//! INFERRED local command boundary; requests are not authoritative server replies.
use super::pet_battle_runtime_model::{PetBattleRequest, PetBattleRuntime};
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
    table_set_rust_fn_static(state, ns, "GetAllEffectNames", get_all_effect_names)?;
    table_set_rust_fn_static(state, ns, "IsTrapAvailable", is_trap_available)?;
    table_set_rust_fn_static(state, ns, "IsPlayerNPC", is_player_npc)?;
    table_set_rust_fn_static(state, ns, "ShouldShowPetSelect", should_show_pet_select)?;
    Ok(())
}

/// INFERRED R88g9 (native evidence pending): consumer at SharedPetBattleTemplates.lua:381 expects string returns.
fn get_all_effect_names(state: &mut LuaState) -> LuaResult<u32> {
    let names = borrow_state(state)?
        .pet_battles
        .runtime
        .effect_names
        .clone();
    for name in &names {
        let value = create_string(state, name);
        state.push(value);
    }
    Ok(names.len() as u32)
}

/// INFERRED R88f.D.44, R88g9 (native evidence pending): evaluate supplied trap inputs.
fn is_trap_available(state: &mut LuaState) -> LuaResult<u32> {
    let (available, error) = {
        let sim = borrow_state(state)?;
        let battle = &sim.pet_battles;
        let runtime = &battle.runtime;
        let available = trap_is_available(battle.battle_state, runtime);
        let error = if available { 0 } else { runtime.trap_error };
        (available, error)
    };
    state.push(Val::Bool(available));
    state.push(Val::Num(f64::from(error)));
    Ok(2)
}

fn trap_is_available(battle_state: i32, runtime: &PetBattleRuntime) -> bool {
    if battle_state != 2 {
        return false;
    }
    if runtime.player_trap_state != 1 {
        return false;
    }
    if !runtime.opponent_is_npc {
        return false;
    }
    if !runtime.action_available {
        return false;
    }
    !runtime.local_team_present || !runtime.local_team_skip_blocking
}

/// INFERRED R88f.D.45, R88g9 (native evidence pending): read the opposite-team NPC marker.
fn is_player_npc(state: &mut LuaState) -> LuaResult<u32> {
    let is_npc = borrow_state(state)?.pet_battles.runtime.opponent_is_npc;
    state.push(Val::Bool(is_npc));
    Ok(1)
}

/// INFERRED R88f.D.46, R88g9 (native evidence pending): phases 1 and 3 expose local selection.
fn should_show_pet_select(state: &mut LuaState) -> LuaResult<u32> {
    let sim = borrow_state(state)?;
    let battle = &sim.pet_battles;
    let runtime = &battle.runtime;
    let selection_phase = matches!(battle.battle_state, 1 | 3);
    let should_show =
        selection_phase && runtime.local_team_present && runtime.local_team_should_select;
    drop(sim);
    state.push(Val::Bool(should_show));
    Ok(1)
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

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    fn runtime_members_read_supplied_state() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            "A_Admin.SetPetBattleEffectName('Supplied effect'); \
             A_Admin.SetPetBattleTeamState(true, false, true, true); \
             A_Admin.SetPetBattleState(1)",
        )
        .expect("pet-battle state inputs");
        let result: String = env
            .eval(
                r##"
                local effects = {C_PetBattles.GetAllEffectNames()}
                if effects[1] ~= "Supplied effect" then return "effects" end
                if C_PetBattles.IsPlayerNPC(2) ~= true then return "player-npc" end
                if C_PetBattles.ShouldShowPetSelect() ~= true then return "pet-select" end
                A_Admin.SetPetBattleState(2)
                A_Admin.SetPetBattleAvailability(1, true, 4)
                local available, reason = C_PetBattles.IsTrapAvailable()
                if not available or reason ~= 0 then return "trap-available" end
                A_Admin.SetPetBattleAvailability(0, true, 4)
                available, reason = C_PetBattles.IsTrapAvailable()
                if available or reason ~= 4 then return "trap-unavailable" end
                return "ok"
                "##,
            )
            .expect("Rust-backed pet-battle inputs should be callable");
        assert_eq!(result, "ok");
    }
}
