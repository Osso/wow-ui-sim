//! Guild invitations answered by `AcceptGuild` / `DeclineGuild`.
//!
//! Declarations: Retail and Forever `PlayerScriptDocumentation.lua` (no
//! arguments or returns) and `GuildInfoDocumentation.lua` `GUILD_INVITE_REQUEST`
//! (inviter, guildName, guildAchievementPoints, oldGuildName, isNewGuild).
//! Consumer: `Blizzard_FrameXML/GuildInviteFrame.{lua,xml}` shows the invite,
//! its Join button calls `AcceptGuild` and its `OnHide` calls `DeclineGuild`
//! unless the invite was accepted.
//! INFERRED (R-ag10-2): acceptance applies the server's membership (supplied
//! with the invite) and fires `PLAYER_GUILD_UPDATE`, like `A_Admin.JoinGuild`;
//! a decline clears the invite without a client event.

use super::{Arg, fire};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, runtime_error};

const USAGE: &str = "Usage: A_Admin.ReceiveGuildInvite(inviter, guildName, \
                     achievementPoints, oldGuildName, isNewGuild, joinRank, numMembers)";

/// A pending invitation and the membership the server grants on acceptance.
#[derive(Clone, Debug, PartialEq)]
pub struct GuildInvite {
    pub inviter: String,
    pub guild_name: String,
    pub join_rank: String,
    pub num_members: i32,
}

fn receive_guild_invite(state: &mut LuaState) -> LuaResult<u32> {
    let inviter = String::from_stack(state, 1)?;
    let guild_name = String::from_stack(state, 2)?;
    let points = i32::from_stack(state, 3)?;
    let old_guild_name = String::from_stack(state, 4)?;
    let is_new_guild = bool::from_stack(state, 5)?;
    let join_rank = String::from_stack(state, 6)?;
    let num_members = i32::from_stack(state, 7)?;
    if inviter.is_empty() || guild_name.is_empty() || points < 0 || num_members < 1 {
        return Err(runtime_error(USAGE));
    }
    borrow_state_mut(state)?.interaction_dialogs.guild_invite = Some(GuildInvite {
        inviter: inviter.clone(),
        guild_name: guild_name.clone(),
        join_rank,
        num_members,
    });
    let args = vec![
        Arg::Str(inviter),
        Arg::Str(guild_name),
        Arg::Num(f64::from(points)),
        Arg::Str(old_guild_name),
        Arg::Bool(is_new_guild),
    ];
    fire(state, "GUILD_INVITE_REQUEST", args)?;
    Ok(0)
}

fn accept_guild(state: &mut LuaState) -> LuaResult<u32> {
    let joined = {
        let mut sim = borrow_state_mut(state)?;
        let invite = sim.interaction_dialogs.guild_invite.take();
        invite.map(|invite| {
            sim.world.guild_name = Some(invite.guild_name);
            sim.world.guild_rank = Some(invite.join_rank);
            sim.world.guild_num_members = invite.num_members;
        })
    };
    if joined.is_some() {
        fire(state, "PLAYER_GUILD_UPDATE", Vec::new())?;
    }
    Ok(0)
}

fn decline_guild(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.interaction_dialogs.guild_invite = None;
    Ok(0)
}

pub(super) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "AcceptGuild", accept_guild)?;
    LuaApiMut::register_function(lua, "DeclineGuild", decline_guild)
}

pub(super) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("ReceiveGuildInvite", receive_guild_invite)
}
