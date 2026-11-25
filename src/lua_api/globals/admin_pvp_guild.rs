//! Rilua A_Admin handlers — PvP & guild.
//!
//! Extracted from rilua_admin_world.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::{BattlefieldStatus, GuildPerk};
use crate::lua_bridge::FromStack;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

// ── PvP & guild ───────────────────────────────────────────────────────────────

pub(super) fn set_pvp_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let v = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.player.pvp_enabled = v;
    Ok(0)
}

pub(super) fn set_honor_level(state: &mut LuaState) -> LuaResult<u32> {
    let level = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?.player.honor_level = level;
    Ok(0)
}

pub(super) fn set_battlefield_max_id(state: &mut LuaState) -> LuaResult<u32> {
    let max_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?.max_battlefield_id = max_id;
    Ok(0)
}

pub(super) fn set_battlefield_queue(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let queued = bool::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    sim.battlefield_queue.status = if queued {
        BattlefieldStatus::Queued
    } else {
        BattlefieldStatus::None
    };
    sim.battlefield_queue.index = if queued { index } else { 0 };
    sim.battlefield_queue.name = if queued {
        format!("Battleground {index}")
    } else {
        String::new()
    };
    Ok(0)
}

pub(super) fn set_battlefield_arena(state: &mut LuaState) -> LuaResult<u32> {
    let active = bool::from_stack(state, 1)?;
    let record_flag = bool::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    sim.world.battlefield_arena = active;
    sim.world.current_arena_record_flag = record_flag;
    Ok(0)
}

pub(super) fn set_player_present(state: &mut LuaState) -> LuaResult<u32> {
    let present = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.player.present = present;
    Ok(0)
}

pub(super) fn set_pvp_timer(state: &mut LuaState) -> LuaResult<u32> {
    let running = bool::from_stack(state, 1)?;
    let deadline_ms = u32::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    sim.pvp_timer_running = running;
    sim.pvp_timer_deadline_ms = deadline_ms;
    Ok(0)
}

pub(super) fn set_guild_perks(state: &mut LuaState) -> LuaResult<u32> {
    let argument_count = state.top.saturating_sub(state.base);
    if argument_count % 2 != 0 {
        return Err(rilua::LuaError::Runtime(rilua::RuntimeError {
            message: "A_Admin.SetGuildPerks expects spellID, level pairs".into(),
            level: 0,
            traceback: vec![],
        }));
    }
    let perks = (1..=argument_count)
        .step_by(2)
        .map(|index| {
            Ok(GuildPerk {
                spell_id: u32::from_stack(state, index as i32)?,
                level: u32::from_stack(state, (index + 1) as i32)?,
            })
        })
        .collect::<LuaResult<Vec<_>>>()?;
    borrow_state_mut(state)?.world.guild_perks = perks;
    Ok(0)
}

pub(super) fn set_guild_info(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let rank = String::from_stack(state, 2)?;
    let num_members = i32::from_stack(state, 3)?;
    let mut st = borrow_state_mut(state)?;
    st.world.guild_name = Some(name);
    st.world.guild_rank = Some(rank);
    st.world.guild_num_members = num_members;
    Ok(0)
}

pub(super) fn join_guild(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let rank = String::from_stack(state, 2)?;
    let num_members = i32::from_stack(state, 3)?;
    let mut st = borrow_state_mut(state)?;
    st.world.guild_name = Some(name);
    st.world.guild_rank = Some(rank);
    st.world.guild_num_members = num_members;
    drop(st);
    dispatch_event_now(state, "PLAYER_GUILD_UPDATE", &[])?;
    Ok(0)
}

pub(super) fn clear_guild(state: &mut LuaState) -> LuaResult<u32> {
    let mut st = borrow_state_mut(state)?;
    st.world.guild_name = None;
    st.world.guild_rank = None;
    st.world.guild_num_members = 0;
    Ok(0)
}

pub(super) fn leave_guild(state: &mut LuaState) -> LuaResult<u32> {
    let mut st = borrow_state_mut(state)?;
    st.world.guild_name = None;
    st.world.guild_rank = None;
    st.world.guild_num_members = 0;
    drop(st);
    dispatch_event_now(state, "PLAYER_GUILD_UPDATE", &[])?;
    Ok(0)
}
