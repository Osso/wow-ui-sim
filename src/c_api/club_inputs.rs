//! Explicit host inputs for club member events; no fabricated live server.
use super::club_members::{fire_member_event, sync_guild};
use super::club_model::{GUILD_ID, MEMBER, Member, OWNER};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::state::{GuildMember, SimState};
use rilua::{LuaResult, runtime_error};

type MemberEvent = (&'static str, Option<u8>);

/// Apply a server/fixture member snapshot before synchronous listeners run.
/// INFERRED: identical snapshots are silent; changed presence/role publish their
/// specific events, other changed fields publish MEMBER_UPDATED. Invitations
/// disappear on membership arrival, never merely because they were sent.
pub fn receive_member(env: &WowLuaEnv, club_id: &str, mut member: Member) -> crate::Result<()> {
    validate_member(&member)?;
    let mut lua = env.lua.borrow_mut();
    sync_guild(lua.state_mut())?;
    let previous = store_member_snapshot(&mut env.state().borrow_mut(), club_id, &mut member)?;
    for (event, detail) in classify_changes(previous.as_ref(), &member) {
        fire_member_event(lua.state_mut(), event, club_id, &member.id, detail);
    }
    Ok(())
}

fn store_member_snapshot(
    sim: &mut SimState,
    club_id: &str,
    member: &mut Member,
) -> LuaResult<Option<Member>> {
    let club = sim
        .clubs
        .clubs
        .get_mut(club_id)
        .ok_or_else(|| runtime_error(format!("C_Club member input: unknown club {club_id}")))?;
    validate_identity(club, member)?;
    let previous = club.member(&member.id).cloned();
    if club_id == GUILD_ID {
        // INFERRED: absent guild rank retains previous rank, initially 1.
        member.guild_rank = Some(
            member
                .guild_rank
                .or_else(|| previous.as_ref().and_then(|entry| entry.guild_rank))
                .unwrap_or(1),
        );
    }
    if let Some(existing) = club.member_mut(&member.id) {
        *existing = member.clone();
    } else {
        club.members.push(member.clone());
    }
    club.invitations
        .retain(|invitation| invitation.invitee.id != member.id);
    club.candidates.remove(&member.id);
    if club_id == GUILD_ID {
        update_guild_roster(&mut sim.world.guild_members, previous.as_ref(), member);
    }
    Ok(previous)
}

/// Apply a member departure. Unknown IDs are idempotent, unknown clubs are errors.
pub fn receive_member_removal(
    env: &WowLuaEnv,
    club_id: &str,
    member_id: &str,
) -> crate::Result<()> {
    let mut lua = env.lua.borrow_mut();
    sync_guild(lua.state_mut())?;
    let removed = remove_member_snapshot(&mut env.state().borrow_mut(), club_id, member_id)?;
    if removed.is_some() {
        fire_member_event(
            lua.state_mut(),
            "CLUB_MEMBER_REMOVED",
            club_id,
            member_id,
            None,
        );
    }
    Ok(())
}

fn remove_member_snapshot(
    sim: &mut SimState,
    club_id: &str,
    member_id: &str,
) -> LuaResult<Option<Member>> {
    let club =
        sim.clubs.clubs.get_mut(club_id).ok_or_else(|| {
            runtime_error(format!("C_Club removal input: unknown club {club_id}"))
        })?;
    let removed = club.member(member_id).cloned();
    if club.club_type != 2 && removed.as_ref().is_some_and(|member| member.role == OWNER) {
        return Err(runtime_error(
            "C_Club removal input: transfer required owner before removal",
        ));
    }
    club.members.retain(|member| member.id != member_id);
    if club_id == GUILD_ID
        && let Some(member) = &removed
    {
        sim.world
            .guild_members
            .retain(|entry| entry.name != member.name);
    }
    Ok(removed)
}

fn validate_member(member: &Member) -> LuaResult<()> {
    if member.id.is_empty() || !(OWNER..=MEMBER).contains(&member.role) || member.presence > 5 {
        return Err(runtime_error(
            "C_Club member input: nonempty opaque ID, role 1..4 and presence 0..5 required",
        ));
    }
    Ok(())
}

fn validate_identity(club: &super::club_model::Club, member: &Member) -> LuaResult<()> {
    let other_self = club
        .members
        .iter()
        .any(|entry| entry.id != member.id && entry.is_self);
    if member.is_self && other_self {
        return Err(runtime_error(
            "C_Club member input: duplicate self identity",
        ));
    }
    if club.club_type == 2 {
        return Ok(());
    }
    let other_owner = club
        .members
        .iter()
        .any(|entry| entry.id != member.id && entry.role == OWNER);
    let demotes_owner = club
        .member(&member.id)
        .is_some_and(|entry| entry.role == OWNER)
        && member.role != OWNER;
    if (member.role == OWNER && other_owner) || (demotes_owner && !other_owner) {
        return Err(runtime_error(
            "C_Club member input: required unique owner must be transferred atomically",
        ));
    }
    Ok(())
}

fn update_guild_roster(roster: &mut Vec<GuildMember>, previous: Option<&Member>, member: &Member) {
    let previous_name = previous
        .map(|entry| entry.name.as_str())
        .unwrap_or(&member.name);
    let entry = GuildMember {
        name: member.name.clone(),
        rank_index: member.guild_rank.unwrap_or(1),
        online: member.presence != 3 && member.presence != 0,
    };
    if let Some(existing) = roster.iter_mut().find(|entry| entry.name == previous_name) {
        *existing = entry;
    } else {
        roster.push(entry);
    }
}

fn classify_changes(previous: Option<&Member>, member: &Member) -> Vec<MemberEvent> {
    let Some(previous) = previous else {
        return vec![("CLUB_MEMBER_ADDED", None)];
    };
    let mut events = Vec::new();
    if previous.presence != member.presence {
        events.push(("CLUB_MEMBER_PRESENCE_UPDATED", Some(member.presence)));
    }
    if previous.role != member.role {
        events.push(("CLUB_MEMBER_ROLE_UPDATED", Some(member.role)));
    }
    let attributes_changed = previous.name != member.name
        || previous.note != member.note
        || previous.is_self != member.is_self
        || previous.guild_rank != member.guild_rank;
    if attributes_changed {
        events.push(("CLUB_MEMBER_UPDATED", None));
    }
    events
}
