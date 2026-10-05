//! Lua boundary for opaque club membership and privilege-checked management.
use super::club_model::{Club, Invitation, Member, OWNER};
use super::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let table = ensure_namespace(state, "C_Club")?;
    for (name, function) in [
        (
            "GetSubscribedClubs",
            subscribed as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("GetClubInfo", club_info),
        ("GetClubMembers", members),
        ("GetMemberInfo", member_info),
        ("GetMemberInfoForSelf", self_info),
        ("GetClubPrivileges", privileges),
        ("GetAssignableRoles", assignable_roles),
        ("CreateClub", create_club),
        ("AssignMemberRole", assign_role),
        ("KickMember", kick),
        ("SetClubMemberNote", set_note),
        ("SendInvitation", send_invitation),
        ("RevokeInvitation", revoke_invitation),
        ("GetInvitationsForClub", invitations),
        ("RequestInvitationsForClub", request_invitations),
        ("IsRestricted", restriction),
        ("AreMembersReady", members_ready),
    ] {
        table_set_rust_fn_static(state, table, name, function)?;
    }
    Ok(())
}

pub(super) fn sync_guild(state: &LuaState) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let name = sim.world.guild_name.clone();
    let roster = sim.world.guild_members.clone();
    sim.clubs.sync_guild(name.as_deref(), &roster);
    Ok(())
}

fn string_arg(state: &mut LuaState, index: i32) -> LuaResult<String> {
    let value = unwrap_secret(state, stack_val(state, index))?;
    crate::lua_api::methods::val_to_string(state, value)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| runtime_error("C_Club: club/member ID must be a nonempty opaque string"))
}

pub(super) fn member_arg(state: &mut LuaState) -> LuaResult<Option<Member>> {
    let club_id = string_arg(state, 1)?;
    let member_id = string_arg(state, 2)?;
    sync_guild(state)?;
    Ok(borrow_state(state)?
        .clubs
        .clubs
        .get(&club_id)
        .and_then(|club| club.member(&member_id))
        .cloned())
}

fn read_club(state: &mut LuaState) -> LuaResult<Option<Club>> {
    let id = string_arg(state, 1)?;
    sync_guild(state)?;
    let sim = borrow_state(state)?;
    Ok(sim
        .clubs
        .initialized
        .then(|| sim.clubs.clubs.get(&id).cloned())
        .flatten())
}

fn authorize_mutation(state: &mut LuaState) -> LuaResult<bool> {
    for index in 1..=(state.top - state.base) as i32 {
        unwrap_secret(state, stack_val(state, index))?;
    }
    Ok(borrow_state(state)?.clubs.can_mutate())
}

fn push_optional_member(state: &mut LuaState, member: Option<&Member>, club_type: u8) -> u32 {
    let value = member
        .map(|member| build_member_info(state, member, club_type))
        .unwrap_or(Val::Nil);
    state.push(value);
    1
}

pub(super) fn build_member_info(state: &mut LuaState, member: &Member, club_type: u8) -> Val {
    let table = create_table(state);
    for (field, text) in [
        ("memberId", member.id.as_str()),
        ("name", member.name.as_str()),
        ("memberNote", member.note.as_str()),
    ] {
        let value = create_string(state, text);
        table_set(state, table, field, value);
    }
    table_set(state, table, "isSelf", Val::Bool(member.is_self));
    table_set(state, table, "role", Val::Num(member.role as f64));
    table_set(state, table, "presence", Val::Num(member.presence as f64));
    table_set(state, table, "clubType", Val::Num(club_type as f64));
    if let Some(rank) = member.guild_rank {
        table_set(state, table, "guildRankOrder", Val::Num(rank as f64));
    }
    table
}

fn build_club_info(state: &mut LuaState, club: &Club) -> Val {
    let table = create_table(state);
    for (field, text) in [
        ("clubId", &club.id),
        ("name", &club.name),
        ("description", &club.description),
    ] {
        let value = create_string(state, text);
        table_set(state, table, field, value);
    }
    let broadcast = create_string(state, "");
    table_set(state, table, "broadcast", broadcast);
    table_set(state, table, "clubType", Val::Num(club.club_type as f64));
    table_set(state, table, "avatarId", Val::Num(club.avatar_id as f64));
    table_set(
        state,
        table,
        "memberCount",
        Val::Num(club.members.len() as f64),
    );
    table
}

fn subscribed(state: &mut LuaState) -> LuaResult<u32> {
    sync_guild(state)?;
    let clubs: Vec<_> = borrow_state(state)?.clubs.clubs.values().cloned().collect();
    let table = create_table(state);
    if borrow_state(state)?.clubs.initialized {
        for (index, club) in clubs.iter().enumerate() {
            let value = build_club_info(state, club);
            set_table_array(state, table, index as i64 + 1, value);
        }
    }
    state.push(table);
    Ok(1)
}

fn club_info(state: &mut LuaState) -> LuaResult<u32> {
    let club = read_club(state)?;
    let value = club
        .as_ref()
        .map(|club| build_club_info(state, club))
        .unwrap_or(Val::Nil);
    state.push(value);
    Ok(1)
}

fn members(state: &mut LuaState) -> LuaResult<u32> {
    let club = read_club(state)?;
    let table = create_table(state);
    if let Some(club) = club {
        for (index, member) in club.members.iter().enumerate() {
            let id = create_string(state, &member.id);
            set_table_array(state, table, index as i64 + 1, id);
        }
    }
    state.push(table);
    Ok(1)
}

fn member_info(state: &mut LuaState) -> LuaResult<u32> {
    let club = read_club(state)?;
    let id = string_arg(state, 2)?;
    let member = club.as_ref().and_then(|club| club.member(&id));
    Ok(push_optional_member(
        state,
        member,
        club.as_ref().map(|club| club.club_type).unwrap_or(0),
    ))
}

fn self_info(state: &mut LuaState) -> LuaResult<u32> {
    let club = read_club(state)?;
    let member = club.as_ref().and_then(Club::self_member);
    Ok(push_optional_member(
        state,
        member,
        club.as_ref().map(|club| club.club_type).unwrap_or(0),
    ))
}

const PRIVILEGES: &[&str] = &[
    "canDestroy",
    "canSetAttribute",
    "canSetName",
    "canSetDescription",
    "canSetAvatar",
    "canSetBroadcast",
    "canSetPrivacyLevel",
    "canSetOwnMemberAttribute",
    "canSetOtherMemberAttribute",
    "canSetOwnMemberNote",
    "canSetOtherMemberNote",
    "canSetOwnVoiceState",
    "canSetOwnPresenceLevel",
    "canUseVoice",
    "canVoiceMuteMemberForAll",
    "canGetInvitation",
    "canSendInvitation",
    "canSendGuestInvitation",
    "canRevokeOwnInvitation",
    "canRevokeOtherInvitation",
    "canGetBan",
    "canGetSuggestion",
    "canSuggestMember",
    "canGetTicket",
    "canCreateTicket",
    "canDestroyTicket",
    "canAddBan",
    "canRemoveBan",
    "canCreateStream",
    "canDestroyStream",
    "canSetStreamPosition",
    "canSetStreamAttribute",
    "canSetStreamName",
    "canSetStreamSubject",
    "canSetStreamAccess",
    "canSetStreamVoiceLevel",
    "canCreateMessage",
    "canDestroyOwnMessage",
    "canDestroyOtherMessage",
    "canEditOwnMessage",
    "canPinMessage",
];

fn build_role_array(state: &mut LuaState, roles: &[u8]) -> Val {
    let table = create_table(state);
    for (index, role) in roles.iter().enumerate() {
        set_table_array(state, table, index as i64 + 1, Val::Num(*role as f64));
    }
    table
}

fn privileges(state: &mut LuaState) -> LuaResult<u32> {
    let club = read_club(state)?;
    let table = create_table(state);
    for field in PRIVILEGES {
        table_set(
            state,
            table,
            field,
            Val::Bool(club.as_ref().is_some_and(|club| club.privilege(field))),
        );
    }
    let roles = club.as_ref().map(Club::kickable_roles).unwrap_or_default();
    let kickable = build_role_array(state, &roles);
    table_set(state, table, "kickableRoleIds", kickable);
    state.push(table);
    Ok(1)
}

fn assignable_roles(state: &mut LuaState) -> LuaResult<u32> {
    let club = read_club(state)?;
    let id = string_arg(state, 2)?;
    let roles = club
        .as_ref()
        .map(|club| club.assignable_roles(&id))
        .unwrap_or_default();
    let table = build_role_array(state, &roles);
    state.push(table);
    Ok(1)
}

pub(super) fn fire_member_event(
    state: &mut LuaState,
    event: &str,
    club_id: &str,
    member_id: &str,
    detail: Option<u8>,
) {
    let club = create_string(state, club_id);
    let member = create_string(state, member_id);
    let mut args = vec![club, member];
    if let Some(detail) = detail {
        args.push(Val::Num(detail as f64));
    }
    fire_named_event_state(state, event, &args);
}

fn fire_club_event(state: &mut LuaState, event: &str, club_id: &str) {
    let club = create_string(state, club_id);
    fire_named_event_state(state, event, &[club]);
}

fn create_club(state: &mut LuaState) -> LuaResult<u32> {
    if !authorize_mutation(state)? {
        return Ok(0);
    }
    let name = String::from_stack(state, 1)?;
    let description = String::from_stack(state, 3)?;
    let club_type = i32::from_stack(state, 4)?;
    let avatar_id = u32::from_stack(state, 5)?;
    if ![0, 1].contains(&club_type) {
        return Err(runtime_error(
            "C_Club.CreateClub: expected BattleNet or Character club",
        ));
    }
    let id = {
        let mut sim = borrow_state_mut(state)?;
        let id = sim.clubs.allocate_id("club");
        let member_id = sim.clubs.allocate_id("member");
        let owner = Member::new(&member_id, &sim.player.name, OWNER, true);
        // INFERRED: local completion is synchronous; owner role is required/unique.
        sim.clubs.clubs.insert(
            id.clone(),
            Club {
                id: id.clone(),
                name,
                description,
                club_type: club_type as u8,
                avatar_id,
                members: vec![owner],
                candidates: Default::default(),
                invitations: Vec::new(),
            },
        );
        id
    };
    fire_club_event(state, "CLUB_ADDED", &id);
    Ok(0)
}

fn assign_role(state: &mut LuaState) -> LuaResult<u32> {
    if !authorize_mutation(state)? {
        return Ok(0);
    }
    let Some(club) = read_club(state)? else {
        return Ok(0);
    };
    let id = string_arg(state, 2)?;
    let role = i32::from_stack(state, 3)?;
    let Ok(role) = u8::try_from(role) else {
        return Ok(0);
    };
    if !club.assignable_roles(&id).contains(&role) {
        return Ok(0);
    }
    if club.member(&id).is_some_and(|member| member.role == role) {
        return Ok(0);
    }
    let previous_owner = update_role(state, &club.id, &id, role)?;
    if let Some(owner_id) = previous_owner {
        fire_member_event(
            state,
            "CLUB_MEMBER_ROLE_UPDATED",
            &club.id,
            &owner_id,
            Some(super::club_model::LEADER),
        );
    }
    fire_member_event(state, "CLUB_MEMBER_ROLE_UPDATED", &club.id, &id, Some(role));
    Ok(0)
}

fn update_role(state: &LuaState, club_id: &str, id: &str, role: u8) -> LuaResult<Option<String>> {
    let mut sim = borrow_state_mut(state)?;
    let club = sim.clubs.clubs.get_mut(club_id).expect("validated club");
    // INFERRED: assigning Owner transfers ownership atomically, demoting old owner
    // to Leader. Direct demotion/removal of the sole owner is never allowed.
    let previous_owner = if role == OWNER {
        club.members
            .iter_mut()
            .find(|member| member.role == OWNER)
            .map(|owner| {
                owner.role = super::club_model::LEADER;
                owner.id.clone()
            })
    } else {
        None
    };
    club.member_mut(id).expect("validated member").role = role;
    Ok(previous_owner)
}

fn kick(state: &mut LuaState) -> LuaResult<u32> {
    if !authorize_mutation(state)? {
        return Ok(0);
    }
    let Some(club) = read_club(state)? else {
        return Ok(0);
    };
    let id = string_arg(state, 2)?;
    let Some(member) = club.member(&id) else {
        return Ok(0);
    };
    if member.is_self || !club.kickable_roles().contains(&member.role) {
        return Ok(0);
    }
    borrow_state_mut(state)?
        .clubs
        .clubs
        .get_mut(&club.id)
        .expect("validated club")
        .members
        .retain(|member| member.id != id);
    fire_member_event(state, "CLUB_MEMBER_REMOVED", &club.id, &id, None);
    Ok(0)
}

fn set_note(state: &mut LuaState) -> LuaResult<u32> {
    if !authorize_mutation(state)? {
        return Ok(0);
    }
    let Some(club) = read_club(state)? else {
        return Ok(0);
    };
    let id = string_arg(state, 2)?;
    let note = String::from_stack(state, 3)?;
    let Some(member) = club.member(&id) else {
        return Ok(0);
    };
    let field = if member.is_self {
        "canSetOwnMemberNote"
    } else {
        "canSetOtherMemberNote"
    };
    if !club.privilege(field) || member.note == note {
        return Ok(0);
    }
    borrow_state_mut(state)?
        .clubs
        .clubs
        .get_mut(&club.id)
        .expect("validated club")
        .member_mut(&id)
        .expect("validated member")
        .note = note;
    // INFERRED: changed notes publish MEMBER_UPDATED; identical writes are silent.
    fire_member_event(state, "CLUB_MEMBER_UPDATED", &club.id, &id, None);
    Ok(0)
}

fn send_invitation(state: &mut LuaState) -> LuaResult<u32> {
    if !authorize_mutation(state)? {
        return Ok(0);
    }
    let Some(club) = read_club(state)? else {
        return Ok(0);
    };
    let id = string_arg(state, 2)?;
    if !club.privilege("canSendInvitation")
        || club.member(&id).is_some()
        || club
            .invitations
            .iter()
            .any(|invitation| invitation.invitee.id == id)
    {
        return Ok(0);
    }
    let Some(invitee) = club.candidates.get(&id).cloned() else {
        return Ok(0);
    };
    {
        let mut sim = borrow_state_mut(state)?;
        let invitation_id = sim.clubs.allocate_id("invitation");
        let invitation = Invitation {
            id: invitation_id,
            invitee,
            inviter_id: club.self_member().expect("privileged member").id.clone(),
        };
        sim.clubs
            .clubs
            .get_mut(&club.id)
            .expect("validated club")
            .invitations
            .push(invitation);
    }
    // INFERRED: local invitation changes publish the documented club refresh event.
    fire_club_event(state, "CLUB_INVITATIONS_RECEIVED_FOR_CLUB", &club.id);
    Ok(0)
}

fn revoke_invitation(state: &mut LuaState) -> LuaResult<u32> {
    if !authorize_mutation(state)? {
        return Ok(0);
    }
    let Some(club) = read_club(state)? else {
        return Ok(0);
    };
    let id = string_arg(state, 2)?;
    let Some(invitation) = club
        .invitations
        .iter()
        .find(|invitation| invitation.invitee.id == id)
    else {
        return Ok(0);
    };
    let own = club
        .self_member()
        .is_some_and(|member| member.id == invitation.inviter_id);
    let privilege = if own {
        "canRevokeOwnInvitation"
    } else {
        "canRevokeOtherInvitation"
    };
    if !club.privilege(privilege) {
        return Ok(0);
    }
    borrow_state_mut(state)?
        .clubs
        .clubs
        .get_mut(&club.id)
        .expect("validated club")
        .invitations
        .retain(|invitation| invitation.invitee.id != id);
    fire_club_event(state, "CLUB_INVITATIONS_RECEIVED_FOR_CLUB", &club.id);
    Ok(0)
}

fn invitations(state: &mut LuaState) -> LuaResult<u32> {
    let club = read_club(state)?;
    let table = create_table(state);
    if let Some(club) = club.filter(|club| club.privilege("canGetInvitation")) {
        for (index, invitation) in club.invitations.iter().enumerate() {
            let entry = create_table(state);
            let id = create_string(state, &invitation.id);
            let invitee = build_member_info(state, &invitation.invitee, club.club_type);
            let own = club
                .self_member()
                .is_some_and(|member| member.id == invitation.inviter_id);
            table_set(state, entry, "invitationId", id);
            table_set(state, entry, "isMyInvitation", Val::Bool(own));
            table_set(state, entry, "invitee", invitee);
            set_table_array(state, table, index as i64 + 1, entry);
        }
    }
    state.push(table);
    Ok(1)
}

fn request_invitations(state: &mut LuaState) -> LuaResult<u32> {
    if let Some(club) = read_club(state)?.filter(|club| club.privilege("canGetInvitation")) {
        fire_club_event(state, "CLUB_INVITATIONS_RECEIVED_FOR_CLUB", &club.id);
    }
    Ok(0)
}

fn restriction(state: &mut LuaState) -> LuaResult<u32> {
    let reason = borrow_state(state)?.clubs.restriction_reason;
    state.push(Val::Num(reason as f64));
    Ok(1)
}

fn members_ready(state: &mut LuaState) -> LuaResult<u32> {
    let ready = read_club(state)?.is_some();
    state.push(Val::Bool(ready));
    Ok(1)
}
