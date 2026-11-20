//! Guild verbs that mutate `WorldState.guild_*` and dispatch
//! `GUILD_ROSTER_UPDATE` / `GUILD_MOTD` / `GUILD_CHALLENGE_UPDATED`.
//!
//! Migrates 8 entries off `GLOBAL_NIL_STUBS`:
//!
//! - `GuildInvite(name)` — appends a GuildMember at the lowest rank.
//!   Fires `GUILD_ROSTER_UPDATE`.
//! - `GuildUninvite(name)`         — remove by name. Fires `GUILD_ROSTER_UPDATE`.
//! - `GuildKick(name)`             — alias of `GuildUninvite`.
//! - `GuildLeave()` — clears guild identity + roster.
//!   Fires `GUILD_ROSTER_UPDATE`.
//! - `GuildPromote(name)` — decrement member's rank_index (minimum 1).
//!   Fires `GUILD_ROSTER_UPDATE`.
//! - `GuildSetMOTD(text)`          — write `world.guild_motd`. Fires `GUILD_MOTD`.
//! - `RequestGuildRoster()`        — refresh signal. Fires `GUILD_ROSTER_UPDATE`.
//! - `RequestGuildChallengeInfo()` — fires `GUILD_CHALLENGE_UPDATED`.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::event::Event;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state_types::GuildMember;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

fn push_event(state: &mut LuaState, name: &str) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: name.to_string(),
        args: Vec::new(),
    });
    Ok(())
}

fn required_string(state: &mut LuaState, index: i32) -> Option<String> {
    Option::<String>::from_stack(state, index)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
}

fn lowest_rank_index(world: &crate::lua_api::state_types::WorldState) -> i32 {
    if world.guild_ranks.is_empty() {
        1
    } else {
        world.guild_ranks.len() as i32
    }
}

/// `GuildInvite(name)` — append a GuildMember at the lowest rank.
pub(crate) fn guild_invite(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        return Ok(0);
    };
    {
        let mut st = borrow_state_mut(state)?;
        let rank = lowest_rank_index(&st.world);
        st.world.guild_members.push(GuildMember {
            name,
            guid: None,
            rank_index: rank,
            online: true,
        });
        st.world.guild_num_members = st.world.guild_num_members.saturating_add(1);
    }
    push_event(state, "GUILD_ROSTER_UPDATE")?;
    Ok(0)
}

/// `GuildUninvite(name)` — remove by name. Silent no-op when missing.
pub(crate) fn guild_uninvite(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        return Ok(0);
    };
    let removed = {
        let mut st = borrow_state_mut(state)?;
        let before = st.world.guild_members.len();
        st.world.guild_members.retain(|m| m.name != name);
        if before != st.world.guild_members.len() {
            st.world.guild_num_members = st.world.guild_num_members.saturating_sub(1);
            true
        } else {
            false
        }
    };
    if removed {
        push_event(state, "GUILD_ROSTER_UPDATE")?;
    }
    Ok(0)
}

/// `GuildKick(name)` — alias of `GuildUninvite`.
fn guild_kick(state: &mut LuaState) -> LuaResult<u32> {
    guild_uninvite(state)
}

/// `GuildLeave()` — clear guild identity + roster, fire roster update.
pub(crate) fn guild_leave(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut st = borrow_state_mut(state)?;
        st.world.guild_name = None;
        st.world.guild_rank = None;
        st.world.guild_num_members = 0;
        st.world.guild_members.clear();
        st.world.guild_motd.clear();
        st.world.guild_club_id = None;
        st.world.guild_is_officer = false;
    }
    push_event(state, "GUILD_ROSTER_UPDATE")?;
    Ok(0)
}

/// `GuildPromote(name)` — bump the named member up one rank (minimum 1 =
/// Guild Master). Silent no-op for unknown names.
pub(crate) fn guild_promote(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        return Ok(0);
    };
    let promoted = {
        let mut st = borrow_state_mut(state)?;
        let Some(member) = st.world.guild_members.iter_mut().find(|m| m.name == name) else {
            return Ok(0);
        };
        if member.rank_index <= 1 {
            return Ok(0);
        }
        member.rank_index -= 1;
        true
    };
    if promoted {
        push_event(state, "GUILD_ROSTER_UPDATE")?;
    }
    Ok(0)
}

/// `GuildSetMOTD(text)` — write `world.guild_motd`. Fires `GUILD_MOTD`
/// even on identical text (matches retail: server round-trip always fires).
fn guild_set_motd(state: &mut LuaState) -> LuaResult<u32> {
    let text = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    borrow_state_mut(state)?.world.guild_motd = text;
    push_event(state, "GUILD_MOTD")?;
    Ok(0)
}

/// `RequestGuildRoster()` — server refresh signal.
fn request_guild_roster(state: &mut LuaState) -> LuaResult<u32> {
    push_event(state, "GUILD_ROSTER_UPDATE")?;
    Ok(0)
}

/// `RequestGuildChallengeInfo()` — fire `GUILD_CHALLENGE_UPDATED` synchronously.
/// `CommunitiesGuildInfoFrame_OnLoad` calls this immediately after registering
/// the event, expecting the OnEvent handler to run before the panel is shown.
/// Queueing the event leaves the four challenge rows visible with empty labels
/// because `_UpdateChallenges` never runs in time.
fn request_guild_challenge_info(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "GUILD_CHALLENGE_UPDATED", &[])?;
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GuildInvite", guild_invite)?;
    LuaApiMut::register_function(lua, "GuildUninvite", guild_uninvite)?;
    LuaApiMut::register_function(lua, "GuildKick", guild_kick)?;
    LuaApiMut::register_function(lua, "GuildLeave", guild_leave)?;
    LuaApiMut::register_function(lua, "GuildPromote", guild_promote)?;
    LuaApiMut::register_function(lua, "GuildSetMOTD", guild_set_motd)?;
    LuaApiMut::register_function(lua, "RequestGuildRoster", request_guild_roster)?;
    LuaApiMut::register_function(
        lua,
        "RequestGuildChallengeInfo",
        request_guild_challenge_info,
    )?;
    Ok(())
}
