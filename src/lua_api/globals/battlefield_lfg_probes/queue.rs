use super::{stack_bool, stack_i32};
use crate::lua_api::env::WowLuaAppData;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, create_table};
use crate::lua_api::state_types::{LfdDungeonInfo, PendingTimer};
use crate::lua_api::{next_timer_id, timer_layout};
use crate::lua_bridge::stack_val;
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table as RiluaTable;
use rilua::{LuaResult, Val, runtime_error};
use std::time::Duration;

/// `SetLFGDungeonEnabled(dungeonID, enabled)` persists the checkbox state
/// that Blizzard's LFD list stores through `LFGDungeonList_SetDungeonEnabled`.
pub(super) fn set_lfg_dungeon_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let dungeon_id = stack_i32(state, 1).unwrap_or(0);
    let enabled = stack_bool(state, 2);
    borrow_state_mut(state)?
        .lfd_enabled_dungeons
        .insert(dungeon_id, enabled);
    Ok(0)
}

/// `ClearAllLFGDungeons(category)` clears pending/active queued mode for that
/// category. Blizzard calls this immediately before selecting queue entries.
pub(super) fn clear_all_lfg_dungeons(state: &mut LuaState) -> LuaResult<u32> {
    let category = stack_i32(state, 1).unwrap_or(0);
    let mut sim = borrow_state_mut(state)?;
    clear_lfg_category(&mut sim, category);
    Ok(0)
}

fn clear_lfg_category(sim: &mut crate::lua_api::state::SimState, category: i32) {
    sim.lfg_active_categories.remove(&category);
    sim.lfg_queued_dungeons.remove(&category);
    sim.lfg_queue_pop_due_at = None;
    if sim
        .lfg_active_proposal
        .as_ref()
        .is_some_and(|proposal| proposal.category == category)
    {
        sim.lfg_active_proposal = None;
    }
}

/// `SetLFGDungeon(category, dungeonID)` selects a dungeon for the next
/// `JoinLFG`. The simulator does not yet expose selected ids, but it validates
/// known ids so bad data fails closed instead of creating phantom queues.
pub(super) fn set_lfg_dungeon(state: &mut LuaState) -> LuaResult<u32> {
    let category = stack_i32(state, 1).unwrap_or(0);
    let dungeon_id = stack_i32(state, 2).unwrap_or(0);
    let known = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .any(|d| d.dungeon_id == dungeon_id && dungeon_id > 0);
    if !known {
        return Ok(0);
    }
    borrow_state_mut(state)?
        .lfg_queued_dungeons
        .entry(category)
        .or_default()
        .insert(dungeon_id);
    Ok(0)
}

/// `JoinLFG(category)` marks the category as queued. Proposal/server matching
/// state is out of scope for now, but Blizzard panels can observe the queued
/// mode through `GetLFGMode`.
pub(super) fn join_lfg(state: &mut LuaState) -> LuaResult<u32> {
    let category = stack_i32(state, 1).unwrap_or(0);
    if category > 0 {
        let delay = {
            let mut sim = borrow_state_mut(state)?;
            sim.lfg_active_categories.insert(category);
            sim.lfg_active_proposal = None;
            sim.lfg_queue_pop_due_at =
                Some(sim.clock_now() + Duration::from_secs_f64(sim.lfg_queue_pop_delay_seconds));
            sim.lfg_queue_pop_delay_seconds
        };
        schedule_lfg_queue_pop(state, delay)?;
        dispatch_event_now(state, "LFG_UPDATE", &[])?;
        dispatch_event_now(state, "LFG_QUEUE_STATUS_UPDATE", &[])?;
    }
    Ok(0)
}

/// `LeaveLFG([category])` clears queued and proposal state. Blizzard has
/// several no-argument callers, so a missing category leaves every active
/// simulator queue.
pub(super) fn leave_lfg(state: &mut LuaState) -> LuaResult<u32> {
    let had_proposal = {
        let mut sim = borrow_state_mut(state)?;
        if let Some(category) = stack_i32(state, 1).filter(|category| *category > 0) {
            let had_proposal = sim
                .lfg_active_proposal
                .as_ref()
                .is_some_and(|proposal| proposal.category == category);
            clear_lfg_category(&mut sim, category);
            had_proposal
        } else {
            let had_proposal = sim.lfg_active_proposal.is_some();
            sim.lfg_active_categories.clear();
            sim.lfg_queued_dungeons.clear();
            sim.lfg_queue_pop_due_at = None;
            sim.lfg_active_proposal = None;
            had_proposal
        }
    };
    if had_proposal {
        dispatch_event_now(state, "LFG_PROPOSAL_DONE", &[])?;
    }
    dispatch_event_now(state, "LFG_UPDATE", &[])?;
    dispatch_event_now(state, "LFG_QUEUE_STATUS_UPDATE", &[])?;
    Ok(0)
}

fn schedule_lfg_queue_pop(state: &mut LuaState, delay: f64) -> LuaResult<()> {
    let callback = Val::Function(state.gc.alloc_closure(Closure::Rust(RustClosure::new(
        pop_lfg_queue,
        "LFG.QueuePop",
    ))));
    let id = next_timer_id(state)?;
    timer_layout::store_timer_callback(state, id, callback);

    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    let owner_addon = {
        let sim = app.sim_state.borrow();
        sim.loading_addon_index.or(sim.executing_addon_index)
    };
    let now = app.sim_state.borrow().clock_now();
    app.sim_state
        .borrow_mut()
        .rilua_timers
        .push_back(PendingTimer {
            id,
            fire_at: now + Duration::from_secs_f64(delay),
            interval: None,
            remaining: None,
            cancelled: false,
            owner_addon,
            callback_receives_timer: false,
            callback_arg: None,
        });
    Ok(())
}

fn pop_lfg_queue(state: &mut LuaState) -> LuaResult<u32> {
    let should_dispatch = {
        let mut sim = borrow_state_mut(state)?;
        let Some(due_at) = sim.lfg_queue_pop_due_at else {
            return Ok(0);
        };
        if sim.clock_now() < due_at {
            return Ok(0);
        }
        let Some((category, dungeon_id)) = first_active_queue(&sim) else {
            sim.lfg_queue_pop_due_at = None;
            return Ok(0);
        };
        sim.lfg_active_categories.remove(&category);
        sim.lfg_queued_dungeons.remove(&category);
        sim.lfg_queue_pop_due_at = None;
        sim.lfg_active_proposal = Some(crate::lua_api::state::LfgProposalState {
            category,
            dungeon_id,
        });
        true
    };
    if should_dispatch {
        dispatch_event_now(state, "LFG_PROPOSAL_SHOW", &[])?;
        dispatch_event_now(state, "LFG_PROPOSAL_UPDATE", &[])?;
        dispatch_event_now(state, "LFG_QUEUE_STATUS_UPDATE", &[])?;
    }
    Ok(0)
}

fn first_active_queue(sim: &crate::lua_api::state::SimState) -> Option<(i32, i32)> {
    sim.lfg_queued_dungeons
        .iter()
        .filter(|(category, ids)| sim.lfg_active_categories.contains(category) && !ids.is_empty())
        .map(|(category, ids)| (*category, *ids.iter().next().expect("ids is not empty")))
        .min_by_key(|(category, dungeon_id)| (*category, *dungeon_id))
}

fn is_category_queued(state: &LuaState, category: i32) -> LuaResult<bool> {
    Ok(borrow_state(state)?
        .lfg_active_categories
        .contains(&category))
}

fn first_queued_lfg_id(state: &LuaState, category: i32) -> LuaResult<Option<i32>> {
    Ok(borrow_state(state)?
        .lfg_queued_dungeons
        .get(&category)
        .and_then(|ids| ids.iter().next().copied()))
}

/// `GetLFGInfoServer(category[, lfgID])` feeds Blizzard's Lua `GetLFGMode`.
/// The simulator models only "queued for this category" and leaves server
/// proposal/listing details inert.
pub(super) fn get_lfg_info_server(state: &mut LuaState) -> LuaResult<u32> {
    let category = stack_i32(state, 1).unwrap_or(0);
    let queued = is_category_queued(state, category)?;
    let roles = borrow_state(state)?.lfg_roles.clone();
    state.push(Val::Bool(false)); // inParty
    state.push(Val::Bool(false)); // joined
    state.push(Val::Bool(queued)); // queued
    state.push(Val::Bool(false)); // noPartialClear
    state.push(Val::Nil); // achievements
    let comment = create_string(state, "");
    state.push(comment); // lfgComment
    state.push(Val::Num(0.0)); // slotCount
    state.push(Val::Nil); // reserved
    state.push(Val::Bool(roles.leader)); // leader
    state.push(Val::Bool(roles.tank)); // tank
    state.push(Val::Bool(roles.healer)); // healer
    state.push(Val::Bool(roles.dps)); // dps
    Ok(12)
}

/// `GetLFGQueuedList(category, queuedList?)` wipes and fills a Lua map of
/// queued ids. QueueStatusFrame uses it to decide which LFG entry to display.
pub(super) fn get_lfg_queued_list(state: &mut LuaState) -> LuaResult<u32> {
    let category = stack_i32(state, 1).unwrap_or(0);
    let ids = borrow_state(state)?
        .lfg_queued_dungeons
        .get(&category)
        .map(|set| set.iter().copied().collect::<Vec<_>>())
        .unwrap_or_default();
    let result = match stack_val(state, 2) {
        Val::Table(table_ref) => {
            clear_table(state, table_ref)?;
            Val::Table(table_ref)
        }
        _ => create_table(state),
    };
    if let Val::Table(table_ref) = result {
        for id in ids {
            if let Some(table) = state.gc.tables.get_mut(table_ref) {
                let _ = table.raw_set(Val::Num(id as f64), Val::Bool(true), &state.gc.string_arena);
            }
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(result);
    Ok(1)
}

fn clear_table(state: &mut LuaState, table_ref: GcRef<RiluaTable>) -> LuaResult<()> {
    let mut keys = Vec::new();
    if let Some(table) = state.gc.tables.get(table_ref) {
        let mut key = Val::Nil;
        while let Some((next_key, _)) = table.next(key, &state.gc.string_arena)? {
            keys.push(next_key);
            key = next_key;
        }
    }

    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        for key in keys {
            let _ = table.raw_set(key, Val::Nil, &state.gc.string_arena);
        }
    }
    state.gc.barrier_back(table_ref);
    Ok(())
}

/// `GetLFGQueueStats(category[, queueID])` returns the queue-status display
/// tuple. The sim has no server wait estimates, so timing values are zero.
pub(super) fn get_lfg_queue_stats(state: &mut LuaState) -> LuaResult<u32> {
    let category = stack_i32(state, 1).unwrap_or(0);
    let queue_id = stack_i32(state, 2)
        .filter(|id| *id > 0)
        .or(first_queued_lfg_id(state, category)?);
    let dungeon = {
        let sim = borrow_state(state)?;
        queue_id.and_then(|id| {
            sim.lfd_dungeons
                .iter()
                .find(|d| d.dungeon_id == id)
                .cloned()
        })
    };
    let Some(dungeon) = dungeon else {
        for _ in 0..18 {
            state.push(Val::Nil);
        }
        return Ok(18);
    };
    push_lfg_queue_stats(state, &dungeon);
    Ok(18)
}

fn push_lfg_queue_stats(state: &mut LuaState, dungeon: &LfdDungeonInfo) {
    state.push(Val::Bool(true)); // hasData
    for value in [
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        1.0,
        3.0,
        dungeon.type_id as f64,
        dungeon.subtype_id as f64,
    ] {
        state.push(Val::Num(value));
    }
    let name = create_string(state, &dungeon.name);
    state.push(name); // instanceName
    for _ in 0..6 {
        state.push(Val::Num(0.0));
    }
    state.push(Val::Num(dungeon.dungeon_id as f64)); // activeID
}

fn active_lfg_proposal(state: &LuaState) -> LuaResult<Option<(i32, i32, LfdDungeonInfo)>> {
    let sim = borrow_state(state)?;
    let Some(proposal) = sim.lfg_active_proposal.as_ref() else {
        return Ok(None);
    };
    let dungeon = sim
        .lfd_dungeons
        .iter()
        .find(|dungeon| dungeon.dungeon_id == proposal.dungeon_id)
        .cloned();
    Ok(dungeon.map(|dungeon| (proposal.category, proposal.dungeon_id, dungeon)))
}

/// `GetLFGProposal()` returns the active proposal tuple consumed by
/// `LFGDungeonReadyPopup_Update`, or the full inert no-proposal shape.
pub(super) fn get_lfg_proposal(state: &mut LuaState) -> LuaResult<u32> {
    let Some((category, dungeon_id, dungeon)) = active_lfg_proposal(state)? else {
        push_inactive_lfg_proposal(state);
        return Ok(15);
    };
    let role = {
        let sim = borrow_state(state)?;
        proposal_player_role(&sim)
    };

    state.push(Val::Bool(true)); // proposalExists
    state.push(Val::Num(dungeon_id as f64)); // id
    state.push(Val::Num(dungeon.type_id as f64)); // typeID
    state.push(Val::Num(dungeon.subtype_id as f64)); // subtypeID
    let name = create_string(state, &dungeon.name);
    state.push(name);
    let background = create_string(state, &dungeon.texture_filename);
    state.push(background);
    let role = create_string(state, role);
    state.push(role);
    state.push(Val::Bool(false)); // hasResponded
    state.push(Val::Num(3.0)); // totalEncounters
    state.push(Val::Num(0.0)); // completedEncounters
    state.push(Val::Num(5.0)); // numMembers
    state.push(Val::Bool(true)); // isLeader
    state.push(Val::Bool(dungeon.is_holiday)); // isHoliday
    state.push(Val::Num(category as f64)); // proposalCategory
    state.push(Val::Bool(false)); // isSilent
    Ok(15)
}

fn push_inactive_lfg_proposal(state: &mut LuaState) {
    state.push(Val::Bool(false));
    for _ in 0..3 {
        state.push(Val::Num(0.0));
    }
    for value in ["", "", ""] {
        let value = create_string(state, value);
        state.push(value);
    }
    state.push(Val::Bool(false));
    for _ in 0..3 {
        state.push(Val::Num(0.0));
    }
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    state.push(Val::Nil);
    state.push(Val::Bool(false));
}

/// `GetLFGProposalMember(index)` returns role/acceptance data for proposal
/// slots. The first member is the simulated player; remaining slots are
/// deterministic fillers so Blizzard status rows can render.
pub(super) fn get_lfg_proposal_member(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(0);
    if active_lfg_proposal(state)?.is_none() || !matches!(index, 1..=5) {
        for _ in 0..7 {
            state.push(Val::Nil);
        }
        return Ok(7);
    }

    let (role, level, name, class) = {
        let sim = borrow_state(state)?;
        proposal_member_data(&sim, index)
    };
    state.push(Val::Bool(index == 1)); // isLeader
    let role = create_string(state, &role);
    state.push(role);
    state.push(Val::Num(level as f64));
    state.push(Val::Bool(false)); // responded
    state.push(Val::Bool(false)); // accepted
    let name = create_string(state, &name);
    state.push(name);
    let class = create_string(state, &class);
    state.push(class);
    Ok(7)
}

fn proposal_member_data(
    sim: &crate::lua_api::state::SimState,
    index: i32,
) -> (String, i32, String, String) {
    let role = if index == 1 {
        proposal_player_role(sim).to_string()
    } else {
        match index {
            2 => "TANK",
            3 => "HEALER",
            _ => "DAMAGER",
        }
        .to_string()
    };
    let name = if index == 1 && !sim.player.name.is_empty() {
        sim.player.name.clone()
    } else {
        format!("ProposalMember{index}")
    };
    let class = class_token(sim.player.class_index).to_string();
    (role, sim.player.level, name, class)
}

fn proposal_player_role(sim: &crate::lua_api::state::SimState) -> &'static str {
    if sim.lfg_roles.tank {
        "TANK"
    } else if sim.lfg_roles.healer {
        "HEALER"
    } else if sim.lfg_roles.dps {
        "DAMAGER"
    } else {
        "NONE"
    }
}

fn class_token(class_index: i32) -> &'static str {
    match class_index {
        1 => "WARRIOR",
        2 => "PALADIN",
        3 => "HUNTER",
        4 => "ROGUE",
        5 => "PRIEST",
        6 => "DEATHKNIGHT",
        7 => "SHAMAN",
        8 => "MAGE",
        9 => "WARLOCK",
        10 => "MONK",
        11 => "DRUID",
        12 => "DEMONHUNTER",
        13 => "EVOKER",
        _ => "PALADIN",
    }
}

/// `GetLFGProposalEncounter(index)` returns boss rows for the proposal
/// tooltip. The simulator tracks only encounter count, so names are stable
/// placeholders and every boss is alive.
pub(super) fn get_lfg_proposal_encounter(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(0);
    let has_encounter = active_lfg_proposal(state)?.is_some() && (1..=3).contains(&index);
    let name = if has_encounter {
        format!("Encounter {index}")
    } else {
        String::new()
    };
    let name = create_string(state, &name);
    state.push(name);
    let texture = create_string(state, "");
    state.push(texture);
    state.push(Val::Bool(false));
    Ok(3)
}

/// `AcceptProposal()` consumes the active proposal and reports success.
pub(super) fn accept_proposal(state: &mut LuaState) -> LuaResult<u32> {
    clear_active_lfg_proposal(state, "LFG_PROPOSAL_SUCCEEDED")
}

/// `RejectProposal()` consumes the active proposal and returns to idle state.
pub(super) fn reject_proposal(state: &mut LuaState) -> LuaResult<u32> {
    clear_active_lfg_proposal(state, "LFG_PROPOSAL_DONE")
}

fn clear_active_lfg_proposal(state: &mut LuaState, event_name: &str) -> LuaResult<u32> {
    let had_proposal = {
        let mut sim = borrow_state_mut(state)?;
        let had_proposal = sim.lfg_active_proposal.is_some();
        sim.lfg_active_proposal = None;
        sim.lfg_queue_pop_due_at = None;
        had_proposal
    };
    if had_proposal {
        dispatch_event_now(state, event_name, &[])?;
        if event_name != "LFG_PROPOSAL_DONE" {
            dispatch_event_now(state, "LFG_PROPOSAL_DONE", &[])?;
        }
        dispatch_event_now(state, "LFG_UPDATE", &[])?;
        dispatch_event_now(state, "LFG_QUEUE_STATUS_UPDATE", &[])?;
    }
    Ok(0)
}

/// `GetLFGRoles()` → `(leader, tank, healer, dps)`.
pub(super) fn get_lfg_roles(state: &mut LuaState) -> LuaResult<u32> {
    let roles = borrow_state(state)?.lfg_roles.clone();
    state.push(Val::Bool(roles.leader));
    state.push(Val::Bool(roles.tank));
    state.push(Val::Bool(roles.healer));
    state.push(Val::Bool(roles.dps));
    Ok(4)
}

/// `SetLFGRoles(leader, tank, healer, dps)`.
pub(super) fn set_lfg_roles(state: &mut LuaState) -> LuaResult<u32> {
    let leader = stack_bool(state, 1);
    let tank = stack_bool(state, 2);
    let healer = stack_bool(state, 3);
    let dps = stack_bool(state, 4);
    let mut sim = borrow_state_mut(state)?;
    sim.lfg_roles.leader = leader;
    sim.lfg_roles.tank = tank;
    sim.lfg_roles.healer = healer;
    sim.lfg_roles.dps = dps;
    Ok(0)
}
