//! Rilua A_Admin handlers — Encounter.
//!
//! Extracted from rilua_admin_world.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

mod unit_status;

use crate::lua_api::globals::admin::opt_string_stack;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::BagItem;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// ── Encounter ─────────────────────────────────────────────────────────────────

pub(super) fn simulate_boss_kill(state: &mut LuaState) -> LuaResult<u32> {
    let encounter_id = i32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let difficulty_id = i32::from_stack(state, 3)?;
    let group_size = i32::from_stack(state, 4)?;
    let name_val = crate::lua_api::methods::create_string(state, &name);
    let mut payload = vec![
        Val::Num(encounter_id as f64),
        name_val,
        Val::Num(difficulty_id as f64),
        Val::Num(group_size as f64),
        Val::Num(1.0),
    ];
    if cfg!(feature = "retail-12-0-7") {
        payload.push(unit_status::copy_status_input(state)?);
    }
    dispatch_encounter_end(state, &payload)?;
    let name_val = crate::lua_api::methods::create_string(state, &name);
    dispatch_event_now(
        state,
        "BOSS_KILL",
        &[Val::Num(encounter_id as f64), name_val],
    )?;
    Ok(0)
}

fn dispatch_encounter_end(state: &mut LuaState, payload: &[Val]) -> LuaResult<()> {
    // Keep the copied status list alive across every listener, including callbacks
    // which collect garbage before a later listener receives the snapshot.
    let saved_top = state.top;
    for value in payload {
        state.push(*value);
    }
    let result = dispatch_event_now(state, "ENCOUNTER_END", payload);
    state.top = saved_top;
    result
}

pub(super) fn start_loot_roll(state: &mut LuaState) -> LuaResult<u32> {
    let roll_id = i32::from_stack(state, 1)?;
    let roll_time = f64::from_stack(state, 2)?;
    let info = build_loot_roll_info(state, roll_id, roll_time);
    let mut st = borrow_state_mut(state)?;
    st.world.loot_rolls.insert(roll_id, info);
    drop(st);
    dispatch_event_now(
        state,
        "START_LOOT_ROLL",
        &[Val::Num(roll_id as f64), Val::Num(roll_time)],
    )?;
    Ok(0)
}

fn build_loot_roll_info(
    state: &LuaState,
    roll_id: i32,
    roll_time: f64,
) -> crate::lua_api::state::LootRollInfo {
    use crate::lua_bridge::stack_val;
    let item_quality = match stack_val(state, 5) {
        Val::Num(n) => n as i32,
        _ => 4,
    };
    let item_level = match stack_val(state, 6) {
        Val::Num(n) => n as i32,
        _ => 0,
    };
    crate::lua_api::state::LootRollInfo {
        roll_id,
        roll_time,
        texture: opt_string_stack(state, 4, ""),
        name: opt_string_stack(state, 3, ""),
        count: 1,
        quality: item_quality,
        bind_on_pickup: true,
        can_need: true,
        can_greed: true,
        can_disenchant: false,
        disenchant_level: 0,
        item_level,
        item_link: opt_string_stack(state, 7, ""),
    }
}

pub(super) fn end_loot_roll(state: &mut LuaState) -> LuaResult<u32> {
    let roll_id = i32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    st.world.loot_rolls.remove(&roll_id);
    drop(st);
    dispatch_event_now(state, "LOOT_ROLLS_COMPLETE", &[Val::Num(roll_id as f64)])?;
    Ok(0)
}

pub(super) fn add_loot_item(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let stack_count = Option::<i32>::from_stack(state, 2)?.unwrap_or(1);
    let hyperlink = opt_string_stack(state, 3, "");
    let hyperlink = (!hyperlink.is_empty()).then_some(hyperlink);
    let mut sim = borrow_state_mut(state)?;
    sim.loot_slots.push(BagItem {
        item_id,
        stack_count,
        hyperlink,
    });
    sim.loot_frame_open = true;
    Ok(0)
}

pub(super) fn clear_loot(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.loot_slots.clear();
    sim.loot_frame_open = false;
    Ok(0)
}

pub(super) fn get_last_loot_roll_choice(state: &mut LuaState) -> LuaResult<u32> {
    let choice = crate::lua_api::methods::borrow_state(state)?.last_loot_roll_choice;
    match choice {
        Some(choice) => state.push(Val::Num(choice as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}
