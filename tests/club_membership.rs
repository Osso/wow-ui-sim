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
    "#).unwrap();
    env.state().borrow_mut().world.guild_members.reverse();
    env.exec(r#"
        local reordered = C_Club.GetClubMembers('guild-0')
        assert(reordered[1] == ids[2] and reordered[2] == ids[1])
        assert(C_Club.GetMemberInfo('guild-0', ids[2]).name == secondName)
        local ranges = C_Club.GetMessageRanges('guild-0', 1)
        local message = C_Club.GetMessageInfo('guild-0', 1, ranges[1].oldestMessageId)
        assert(type(message.author.memberId) == 'string')
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
