use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn club_opaque_ids_survive_roster_reordering() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        ids = C_Club.GetClubMembers('guild-0')
        assert(type(ids[1]) == 'string' and type(ids[2]) == 'string')
        assert(ids[1] ~= ids[2])
        assert(C_Club.GetMemberInfo('guild-0', ids[2]).memberId == ids[2])
        secondName = C_Club.GetMemberInfo('guild-0', ids[2]).name
        secondGuid = C_Club.GetMemberInfo('guild-0', ids[2]).guid
        assert(type(secondGuid) == 'string')
    "#).unwrap();
    env.state().borrow_mut().world.guild_members.reverse();
    env.exec(r#"
        local reordered = C_Club.GetClubMembers('guild-0')
        assert(reordered[1] == ids[2] and reordered[2] == ids[1])
        assert(C_Club.GetMemberInfo('guild-0', ids[2]).name == secondName)
        assert(C_Club.GetMemberInfo('guild-0', ids[2]).guid == secondGuid)
        local ranges = C_Club.GetMessageRanges('guild-0', 1)
        local message = C_Club.GetMessageInfo('guild-0', 1, ranges[1].oldestMessageId)
        assert(message.author.memberId == ids[1])
    "#).unwrap();
}

#[test]
fn club_creation_grants_owner_privileges_and_own_note_events() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CLUB_ADDED')
        listener:RegisterEvent('CLUB_MEMBER_UPDATED')
        local clubId, updatedId
        listener:SetScript('OnEvent', function(_, event, club, member)
            if event == 'CLUB_ADDED' then clubId = club else updatedId = member end
        end)
        assert(select('#', C_Club.CreateClub('Raid Team', 'Raid', 'Friday raids', 1, 0)) == 0)
        assert(clubId, 'CreateClub must publish created club')
        local info = C_Club.GetClubInfo(clubId)
        assert(info.name == 'Raid Team' and info.clubType == 1 and info.memberCount == 1)
        local owner = C_Club.GetMemberInfoForSelf(clubId)
        assert(type(owner.memberId) == 'string' and owner.role == 1)
        local privileges = C_Club.GetClubPrivileges(clubId)
        assert(privileges.canSendInvitation and privileges.canSetOwnMemberNote)
        C_Club.SetClubMemberNote(clubId, owner.memberId, 'Raid organizer')
        assert(C_Club.GetMemberInfo(clubId, owner.memberId).memberNote == 'Raid organizer')
        assert(updatedId == owner.memberId)
    "#).unwrap();
}


fn community() -> (WowLuaEnv, String) {
    let env = WowLuaEnv::new().unwrap();
    let club: String = env.eval(r#"
        C_Club.CreateClub('Friday Raiders', 'Raid', 'Progression', 1, 42)
        for _, club in ipairs(C_Club.GetSubscribedClubs()) do
            if club.name == 'Friday Raiders' then return club.clubId end
        end
    "#).unwrap();
    env.exec(&format!("clubId = '{club}'")).unwrap();
    (env, club)
}

#[test]
fn club_roles_notes_kicks_have_opaque_payloads_and_updated_state() {
    use wow_ui_sim::c_api::club_model::Member;
    let (env, club) = community();
    env.state().borrow_mut().clubs.clubs.get_mut(&club).unwrap().members.push(
        Member::new("character:Jaina-99", "Jaina", 4, false));
    env.exec(r#"
        local id = 'character:Jaina-99'
        local seen = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CLUB_MEMBER_ROLE_UPDATED')
        listener:RegisterEvent('CLUB_MEMBER_UPDATED')
        listener:RegisterEvent('CLUB_MEMBER_REMOVED')
        listener:SetScript('OnEvent', function(_, event, club, member, role)
            assert(club == clubId and member == id)
            seen[#seen + 1] = event
            if event == 'CLUB_MEMBER_REMOVED' then
                assert(C_Club.GetMemberInfo(club, member) == nil)
            elseif event == 'CLUB_MEMBER_ROLE_UPDATED' then
                assert(role == 3 and C_Club.GetMemberInfo(club, member).role == role)
            else
                assert(C_Club.GetMemberInfo(club, member).memberNote == 'Healer alt')
            end
        end)
        local roles = C_Club.GetAssignableRoles(clubId, id)
        assert(#roles == 4 and roles[1] == 1 and roles[4] == 4)
        assert(select('#', C_Club.AssignMemberRole(clubId, id, 3)) == 0)
        assert(seen[1] == 'CLUB_MEMBER_ROLE_UPDATED')
        C_Club.AssignMemberRole(clubId, id, 3)
        assert(#seen == 1)
        C_Club.SetClubMemberNote(clubId, id, 'Healer alt')
        assert(seen[2] == 'CLUB_MEMBER_UPDATED')
        C_Club.SetClubMemberNote(clubId, id, 'Healer alt')
        assert(#seen == 2)
        C_Club.KickMember(clubId, id)
        assert(seen[3] == 'CLUB_MEMBER_REMOVED' and #C_Club.GetClubMembers(clubId) == 1)
        C_Club.KickMember(clubId, id)
        assert(#seen == 3)
        assert(not pcall(C_Club.GetMemberInfo, clubId, 2))
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn club_invitations_are_pending_and_revoke_uses_member_identity() {
    use wow_ui_sim::c_api::club_model::Member;
    let (env, club) = community();
    let candidate = Member::new("account:Khadgar-84", "Khadgar", 4, false);
    env.state().borrow_mut().clubs.clubs.get_mut(&club).unwrap()
        .candidates.insert(candidate.id.clone(), candidate);
    env.exec(r#"
        local changed = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CLUB_INVITATIONS_RECEIVED_FOR_CLUB')
        listener:SetScript('OnEvent', function(_, _, club)
            assert(club == clubId)
            changed = changed + 1
        end)
        local id = 'account:Khadgar-84'
        C_Club.SendInvitation(clubId, 'not-a-candidate')
        assert(changed == 0)
        C_Club.SendInvitation(clubId, id)
        local pending = C_Club.GetInvitationsForClub(clubId)
        assert(changed == 1 and #pending == 1)
        assert(type(pending[1].invitationId) == 'string' and pending[1].isMyInvitation)
        assert(pending[1].invitee.memberId == id and pending[1].invitee.name == 'Khadgar')
        assert(C_Club.GetMemberInfo(clubId, id) == nil and #C_Club.GetClubMembers(clubId) == 1)
        C_Club.SendInvitation(clubId, id)
        assert(changed == 1)
        C_Club.RequestInvitationsForClub(clubId)
        assert(changed == 2)
        C_Club.RevokeInvitation(clubId, id)
        assert(changed == 3 and #C_Club.GetInvitationsForClub(clubId) == 0)
        C_Club.RevokeInvitation(clubId, id)
        assert(changed == 3)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn club_owner_transfer_is_atomic_and_required_owner_cannot_be_removed() {
    use wow_ui_sim::c_api::club_model::Member;
    let (env, club) = community();
    env.state().borrow_mut().clubs.clubs.get_mut(&club).unwrap().members.push(
        Member::new("member:Uther", "Uther", 4, false));
    env.exec(r#"
        local self = C_Club.GetMemberInfoForSelf(clubId)
        local count = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CLUB_MEMBER_ROLE_UPDATED')
        listener:SetScript('OnEvent', function()
            count = count + 1
            assert(C_Club.GetMemberInfo(clubId, 'member:Uther').role == 1)
            assert(C_Club.GetMemberInfoForSelf(clubId).role == 2)
        end)
        assert(#C_Club.GetAssignableRoles(clubId, self.memberId) == 0)
        C_Club.AssignMemberRole(clubId, self.memberId, 4)
        C_Club.KickMember(clubId, self.memberId)
        assert(C_Club.GetMemberInfoForSelf(clubId).role == 1 and count == 0)
        C_Club.AssignMemberRole(clubId, 'member:Uther', 1)
        assert(count == 2)
        assert(#C_Club.GetAssignableRoles(clubId, 'member:Uther') == 0)
        C_Club.KickMember(clubId, 'member:Uther')
        C_Club.AssignMemberRole(clubId, 'member:Uther', 4)
        assert(C_Club.GetMemberInfo(clubId, 'member:Uther').role == 1 and count == 2)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn club_denied_management_has_no_side_effects() {
    use wow_ui_sim::c_api::club_model::Member;
    let (env, club) = community();
    {
        let mut sim = env.state().borrow_mut();
        let club = sim.clubs.clubs.get_mut(&club).unwrap();
        club.members[0].role = 4;
        club.members.push(Member::new("member:Uther", "Uther", 1, false));
        let candidate = Member::new("candidate:Jaina", "Jaina", 4, false);
        club.candidates.insert(candidate.id.clone(), candidate);
    }
    env.exec(r#"
        local count = 0
        local listener = CreateFrame('Frame')
        for _, event in ipairs({'CLUB_MEMBER_ROLE_UPDATED', 'CLUB_MEMBER_REMOVED',
            'CLUB_MEMBER_UPDATED', 'CLUB_INVITATIONS_RECEIVED_FOR_CLUB'}) do listener:RegisterEvent(event) end
        listener:SetScript('OnEvent', function() count = count + 1 end)
        local p = C_Club.GetClubPrivileges(clubId)
        assert(not p.canSendInvitation and not p.canSetOtherMemberNote and #p.kickableRoleIds == 0)
        C_Club.AssignMemberRole(clubId, 'member:Uther', 4)
        C_Club.KickMember(clubId, 'member:Uther')
        C_Club.SetClubMemberNote(clubId, 'member:Uther', 'Denied')
        C_Club.SendInvitation(clubId, 'candidate:Jaina')
        C_Club.RevokeInvitation(clubId, 'candidate:Jaina')
        assert(count == 0 and #C_Club.GetClubMembers(clubId) == 2)
        assert(C_Club.GetMemberInfo(clubId, 'member:Uther').role == 1)
        assert(C_Club.GetMemberInfo(clubId, 'member:Uther').memberNote == '')
        assert(#C_Club.GetInvitationsForClub(clubId) == 0)
        local self = C_Club.GetMemberInfoForSelf(clubId)
        C_Club.SetClubMemberNote(clubId, self.memberId, 'Own note allowed')
        assert(count == 1 and C_Club.GetMemberInfoForSelf(clubId).memberNote == 'Own note allowed')
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn club_restrictions_and_initialization_deny_privileged_mutations() {
    use wow_ui_sim::c_api::club_model::Member;
    let (env, club) = community();
    {
        let mut sim = env.state().borrow_mut();
        sim.clubs.clubs.get_mut(&club).unwrap().members.push(Member::new("member:Jaina", "Jaina", 4, false));
        let candidate = Member::new("candidate:Uther", "Uther", 4, false);
        sim.clubs.clubs.get_mut(&club).unwrap().candidates.insert(candidate.id.clone(), candidate);
        sim.clubs.restriction_reason = 1;
    }
    env.exec(r#"
        events = 0
        local listener = CreateFrame('Frame')
        for _, event in ipairs({'CLUB_ADDED', 'CLUB_MEMBER_ROLE_UPDATED', 'CLUB_MEMBER_REMOVED',
            'CLUB_MEMBER_UPDATED', 'CLUB_INVITATIONS_RECEIVED_FOR_CLUB'}) do listener:RegisterEvent(event) end
        listener:SetScript('OnEvent', function() events = events + 1 end)
        function attemptMutations()
            C_Club.AssignMemberRole(clubId, 'member:Jaina', 3)
            C_Club.KickMember(clubId, 'member:Jaina')
            C_Club.SetClubMemberNote(clubId, 'member:Jaina', 'Denied')
            C_Club.SendInvitation(clubId, 'candidate:Uther')
            C_Club.RevokeInvitation(clubId, 'candidate:Uther')
            C_Club.CreateClub('Denied', nil, '', 1, 0)
        end
        assert(C_Club.IsRestricted() == 1)
        attemptMutations()
        assert(events == 0 and #C_Club.GetClubMembers(clubId) == 2)
        assert(C_Club.GetMemberInfo(clubId, 'member:Jaina').role == 4)
        assert(C_Club.GetMemberInfo(clubId, 'member:Jaina').memberNote == '')
        assert(#C_Club.GetInvitationsForClub(clubId) == 0)
    "#).unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.clubs.restriction_reason = 0;
        sim.clubs.initialized = false;
    }
    env.exec("attemptMutations(); assert(events == 0 and not C_Club.AreMembersReady(clubId))").unwrap();
    let sim = env.state().borrow();
    let club = sim.clubs.clubs.get(&club).unwrap();
    assert_eq!(club.members.len(), 2);
    assert_eq!(club.members[1].role, 4);
    assert!(club.members[1].note.is_empty());
    assert!(club.invitations.is_empty());
    assert!(sim.lua_errors.is_empty());
}


#[test]
fn club_host_inputs_publish_added_presence_updated_and_removed_with_same_id() {
    use wow_ui_sim::c_api::{c_club, club_model::Member};
    let (env, club) = community();
    env.exec(r#"
        received = {}
        local listener = CreateFrame('Frame')
        for _, event in ipairs({'CLUB_MEMBER_ADDED', 'CLUB_MEMBER_PRESENCE_UPDATED',
            'CLUB_MEMBER_UPDATED', 'CLUB_MEMBER_ROLE_UPDATED', 'CLUB_MEMBER_REMOVED'}) do
            listener:RegisterEvent(event)
        end
        listener:SetScript('OnEvent', function(_, event, club, id, detail)
            assert(club == clubId and id == 'opaque:server/Jaina-4')
            local info = C_Club.GetMemberInfo(club, id)
            if event == 'CLUB_MEMBER_REMOVED' then assert(info == nil) else
                assert(info.memberId == id)
                if event == 'CLUB_MEMBER_PRESENCE_UPDATED' then assert(info.presence == detail) end
                if event == 'CLUB_MEMBER_ROLE_UPDATED' then assert(info.role == detail) end
            end
            received[#received + 1] = {event, id, detail}
        end)
    "#).unwrap();
    let mut member = Member::new("opaque:server/Jaina-4", "Jaina", 4, false);
    c_club::receive_member(&env, &club, member.clone()).unwrap();
    env.exec("assert(#received == 1 and received[1][1] == 'CLUB_MEMBER_ADDED')").unwrap();
    c_club::receive_member(&env, &club, member.clone()).unwrap();
    env.exec("assert(#received == 1)").unwrap();
    member.presence = 2;
    c_club::receive_member(&env, &club, member.clone()).unwrap();
    env.exec("assert(#received == 2 and received[2][1] == 'CLUB_MEMBER_PRESENCE_UPDATED' and received[2][3] == 2)").unwrap();
    member.name = "Jaina-Proudmoore".into();
    member.note = "Frost mage".into();
    c_club::receive_member(&env, &club, member.clone()).unwrap();
    env.exec(r#"
        assert(#received == 3 and received[3][1] == 'CLUB_MEMBER_UPDATED')
        local info = C_Club.GetMemberInfo(clubId, received[3][2])
        assert(info.name == 'Jaina-Proudmoore' and info.memberNote == 'Frost mage')
    "#).unwrap();
    member.role = 3;
    c_club::receive_member(&env, &club, member.clone()).unwrap();
    env.exec("assert(#received == 4 and received[4][1] == 'CLUB_MEMBER_ROLE_UPDATED' and received[4][3] == 3)").unwrap();
    c_club::receive_member_removal(&env, &club, &member.id).unwrap();
    c_club::receive_member_removal(&env, &club, &member.id).unwrap();
    env.exec("assert(#received == 5 and received[5][1] == 'CLUB_MEMBER_REMOVED')").unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn club_host_join_retires_pending_invitation_before_added_callback() {
    use wow_ui_sim::c_api::{c_club, club_model::Member};
    let (env, club) = community();
    let candidate = Member::new("server:Khadgar", "Khadgar", 4, false);
    env.state().borrow_mut().clubs.clubs.get_mut(&club).unwrap()
        .candidates.insert(candidate.id.clone(), candidate.clone());
    env.exec(r#"
        C_Club.SendInvitation(clubId, 'server:Khadgar')
        assert(#C_Club.GetInvitationsForClub(clubId) == 1)
        joined = false
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CLUB_MEMBER_ADDED')
        listener:SetScript('OnEvent', function(_, _, club, member)
            assert(member == 'server:Khadgar' and #C_Club.GetInvitationsForClub(club) == 0)
            assert(C_Club.GetMemberInfo(club, member).name == 'Khadgar')
            joined = true
        end)
    "#).unwrap();
    c_club::receive_member(&env, &club, candidate).unwrap();
    env.exec("assert(joined and #C_Club.GetClubMembers(clubId) == 2)").unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn club_host_invalid_input_preserves_members_and_sends_no_event() {
    use wow_ui_sim::c_api::{c_club, club_model::Member};
    let (env, club) = community();
    env.exec(r#"
        received = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CLUB_MEMBER_ADDED')
        listener:SetScript('OnEvent', function() received = received + 1 end)
    "#).unwrap();
    assert!(c_club::receive_member(&env, &club, Member::new("", "Invalid", 4, false)).is_err());
    assert!(c_club::receive_member(&env, "missing-club", Member::new("missing:1", "Invalid", 4, false)).is_err());
    assert!(c_club::receive_member(&env, &club, Member::new("invalid:1", "Invalid", 9, false)).is_err());
    env.exec("assert(received == 0 and #C_Club.GetClubMembers(clubId) == 1)").unwrap();
}


#[test]
fn club_guild_host_presence_and_departure_preserve_message_author_identity() {
    use wow_ui_sim::c_api::{c_club, club_model::Member};
    let env = WowLuaEnv::new().unwrap();
    let member_id: String = env.eval("return C_Club.GetClubMembers('guild-0')[2]").unwrap();
    env.exec(r#"
        guildEvents = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CLUB_MEMBER_PRESENCE_UPDATED')
        listener:RegisterEvent('CLUB_MEMBER_REMOVED')
        listener:SetScript('OnEvent', function(_, event, club, id, presence)
            guildEvents[#guildEvents + 1] = {event, id, presence}
            if event == 'CLUB_MEMBER_PRESENCE_UPDATED' then
                assert(C_Club.GetMemberInfo(club, id).presence == presence)
            else assert(C_Club.GetMemberInfo(club, id) == nil) end
        end)
        local range = C_Club.GetMessageRanges('guild-0', 1)[1]
        historical = C_Club.GetMessagesBefore('guild-0', 1, range.newestMessageId, 20)[3]
        assert(historical.author.name == 'Jaina')
    "#).unwrap();
    let mut member = Member::new(&member_id, "Jaina", 4, false);
    member.presence = 2;
    c_club::receive_member(&env, "guild-0", member.clone()).unwrap();
    c_club::receive_member(&env, "guild-0", member).unwrap();
    env.exec("assert(#guildEvents == 1 and guildEvents[1][3] == 2)").unwrap();
    c_club::receive_member_removal(&env, "guild-0", &member_id).unwrap();
    env.exec(r#"
        assert(#guildEvents == 2 and guildEvents[2][2] == historical.author.memberId)
        local message = C_Club.GetMessageInfo('guild-0', 1, historical.messageId)
        assert(message.author.memberId == historical.author.memberId and message.author.name == 'Jaina')
        assert(#C_Club.GetClubMembers('guild-0') == 1)
        C_Club.SendMessage('guild-0', 1, 'Still raiding')
        local range = C_Club.GetMessageRanges('guild-0', 1)[1]
        local latest = C_Club.GetMessageInfo('guild-0', 1, range.newestMessageId)
        assert(latest.author.memberId == C_Club.GetMemberInfoForSelf('guild-0').memberId)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn club_revocation_distinguishes_own_and_other_invitations() {
    use wow_ui_sim::c_api::club_model::{Invitation, Member};
    let (env, club) = community();
    let own_id = env.state().borrow().clubs.clubs.get(&club).unwrap().members[0].id.clone();
    {
        let mut sim = env.state().borrow_mut();
        let club = sim.clubs.clubs.get_mut(&club).unwrap();
        club.members[0].role = 4;
        club.members.push(Member::new("member:Owner", "Uther", 1, false));
        for (id, inviter) in [("candidate:Own", own_id.as_str()), ("candidate:Other", "member:Owner")] {
            club.invitations.push(Invitation { id: format!("invite:{id}"),
                invitee: Member::new(id, id, 4, false), inviter_id: inviter.into() });
        }
    }
    env.exec(r#"
        local p = C_Club.GetClubPrivileges(clubId)
        assert(p.canRevokeOwnInvitation and not p.canRevokeOtherInvitation)
        assert(#C_Club.GetInvitationsForClub(clubId) == 2)
        C_Club.RevokeInvitation(clubId, 'candidate:Other')
        assert(#C_Club.GetInvitationsForClub(clubId) == 2)
        C_Club.RevokeInvitation(clubId, 'candidate:Own')
        local remaining = C_Club.GetInvitationsForClub(clubId)
        assert(#remaining == 1 and not remaining[1].isMyInvitation)
        assert(remaining[1].invitee.memberId == 'candidate:Other')
    "#).unwrap();
}
