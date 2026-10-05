//! Temporary Retail 12.0.7 `C_Club` member-management defaults.
//!
//! Missing backing system: club member roles, member notes, invitations and the
//! privileges that gate them. The guild-club model grants the player no club
//! privileges (`GetClubPrivileges` is all false), so these privilege-checked
//! calls change nothing and no role is assignable. Replace with a club
//! membership model that owns roles, notes, invitations and privileges.

const CLUB_MANAGEMENT_DEFAULTS_LUA: &str = r#"
C_Club = C_Club or __wow_namespace()

if rawget(C_Club, "GetAssignableRoles") == nil then
    function C_Club.GetAssignableRoles(_clubId, _memberId)
        return {}
    end
end

for _, name in ipairs({
    "AssignMemberRole", "KickMember", "RevokeInvitation", "SendInvitation", "SetClubMemberNote",
}) do
    if rawget(C_Club, name) == nil then
        C_Club[name] = function() end
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CLUB_MANAGEMENT_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn privilege_checked_management_calls_change_nothing() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            local clubId = C_Club.GetGuildClubId()
            local before = #C_Club.GetClubMembers(clubId)
            local privileges = C_Club.GetClubPrivileges(clubId)
            assert(not privileges.canSendInvitation and not privileges.canSetOtherMemberNote)
            assert(#C_Club.GetAssignableRoles(clubId, 2) == 0)
            assert(select('#', C_Club.KickMember(clubId, 2)) == 0)
            C_Club.AssignMemberRole(clubId, 2, 3)
            C_Club.SetClubMemberNote(clubId, 2, "officer alt")
            C_Club.SendInvitation(clubId, 2)
            C_Club.RevokeInvitation(clubId, 2)
            assert(#C_Club.GetClubMembers(clubId) == before)
            "#,
        )
        .expect("club management calls are inert without privileges");
    }
}
