//! Guild membership predicates over the host guild roster (`A_Admin.SetGuildRoster`).
//! Retail/Forever UnitDocumentation declare byte-identical `UnitIsInMyGuild` and
//! `IsPlayerInGuildFromGUID` blocks. Membership is keyed by roster GUIDs; the player
//! counts as a member of their own guild. A roster row without a GUID never matches.
//! `AcceptGuild`/`DeclineGuild` answer the invite `A_Admin.OfferGuildInvite` raises
//! (GUILD_INVITE_REQUEST); original consumer Blizzard_FrameXML GuildInviteFrame.

use crate::lua_api::SimState;
use crate::lua_api::globals::real::admin_fields as fields;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

/// Guild details the host supplies with an invite; applied when it is accepted.
#[derive(Debug, Clone)]
pub struct PendingGuildInvite {
    pub guild_name: String,
    pub rank_name: String,
    pub num_members: i32,
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "UnitIsInMyGuild", unit_is_in_my_guild)?;
    LuaApiMut::register_function(lua, "IsPlayerInGuildFromGUID", is_player_in_guild_from_guid)?;
    LuaApiMut::register_function(lua, "AcceptGuild", accept_guild)?;
    LuaApiMut::register_function(lua, "DeclineGuild", decline_guild)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("OfferGuildInvite", offer_guild_invite)
}

/// `A_Admin.OfferGuildInvite({inviter=, guildName=, guildAchievementPoints=,
/// oldGuildName=, isNewGuild=, rankName=, numMembers=})`. INFERRED: with
/// auto-decline enabled the invite is refused before it reaches the client.
fn offer_guild_invite(state: &mut LuaState) -> LuaResult<u32> {
    const API: &str = "A_Admin.OfferGuildInvite";
    let table = match stack_val(state, 1) {
        table @ Val::Table(_) => table,
        _ => return Err(runtime_error(format!("{API} expects an info table"))),
    };
    let inviter = fields::string(state, table, API, "inviter")?;
    let guild_name = fields::string(state, table, API, "guildName")?;
    let points = fields::unsigned(state, table, API, "guildAchievementPoints")?;
    let old_guild = fields::string(state, table, API, "oldGuildName")?;
    let is_new = fields::boolean(state, table, API, "isNewGuild")?;
    let rank_name = fields::string(state, table, API, "rankName")?;
    let num_members = fields::unsigned(state, table, API, "numMembers")?;
    {
        let mut sim = borrow_state_mut(state)?;
        if sim.auto_decline_guild_invites {
            return Ok(0);
        }
        sim.pending_guild_invite = Some(PendingGuildInvite {
            guild_name: guild_name.clone(),
            rank_name,
            num_members: i32::try_from(num_members).unwrap_or(i32::MAX),
        });
    }
    let payload = [
        create_string(state, &inviter),
        create_string(state, &guild_name),
        Val::Num(f64::from(points)),
        create_string(state, &old_guild),
        Val::Bool(is_new),
    ];
    dispatch_event_now(state, "GUILD_INVITE_REQUEST", &payload)?;
    Ok(0)
}

fn accept_guild(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        let Some(invite) = sim.pending_guild_invite.take() else {
            return Ok(0);
        };
        sim.world.guild_name = Some(invite.guild_name);
        sim.world.guild_rank = Some(invite.rank_name);
        sim.world.guild_num_members = invite.num_members;
    }
    dispatch_event_now(state, "PLAYER_GUILD_UPDATE", &[])?;
    Ok(0)
}

fn decline_guild(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.pending_guild_invite = None;
    Ok(0)
}

fn guid_in_my_guild(sim: &SimState, guid: &str) -> bool {
    if sim.world.guild_name.is_none() {
        return false;
    }
    let player = existing_guid_for_unit(sim, "player");
    player.as_deref() == Some(guid)
        || sim
            .world
            .guild_members
            .iter()
            .any(|member| member.guid.as_deref() == Some(guid))
}

fn unit_is_in_my_guild(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let member = {
        let sim = borrow_state(state)?;
        existing_guid_for_unit(&sim, &unit).is_some_and(|guid| guid_in_my_guild(&sim, &guid))
    };
    state.push(Val::Bool(member));
    Ok(1)
}

fn is_player_in_guild_from_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let member = guid_in_my_guild(&*borrow_state(state)?, &guid);
    state.push(Val::Bool(member));
    Ok(1)
}
