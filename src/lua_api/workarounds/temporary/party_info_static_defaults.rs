//! Temporary `C_PartyInfo` static defaults.
//!
//! Pending invite confirmation and Torghast/Jailer's Tower/walk-in state are
//! not modeled yet. These defaults preserve the inert startup baseline while
//! group membership and loot-method probes remain state-backed in Rust.

const PARTY_INFO_STATIC_DEFAULTS_LUA: &str = r#"
C_PartyInfo = C_PartyInfo or __wow_namespace()

if rawget(C_PartyInfo, "AllowedToDoPartyConversion") == nil then
    function C_PartyInfo.AllowedToDoPartyConversion()
        return false
    end
end

if rawget(C_PartyInfo, "IsPartyInJailersTower") == nil then
    function C_PartyInfo.IsPartyInJailersTower()
        return false
    end
end

if rawget(C_PartyInfo, "IsPartyWalkIn") == nil then
    function C_PartyInfo.IsPartyWalkIn()
        return false
    end
end

"#;

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const PARTY_INFO_LEGACY_INVITE_DEFAULT_LUA: &str = r#"
C_PartyInfo = C_PartyInfo or __wow_namespace()
if rawget(C_PartyInfo, "GetInviteConfirmationInfo") == nil then
    function C_PartyInfo.GetInviteConfirmationInfo(_guid)
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PARTY_INFO_STATIC_DEFAULTS_LUA)?;
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    lua.exec(PARTY_INFO_LEGACY_INVITE_DEFAULT_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_party_info_static_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (can_convert, in_jailers_tower, walk_in, invite_results): (bool, bool, bool, i32) = env
            .eval(
                r##"
                return C_PartyInfo.AllowedToDoPartyConversion(),
                       C_PartyInfo.IsPartyInJailersTower(),
                       C_PartyInfo.IsPartyWalkIn(),
                       select("#", C_PartyInfo.GetInviteConfirmationInfo("Player-1234-ABCDEF"))
                "##,
            )
            .expect("party info static defaults should be callable");

        assert!(!can_convert);
        assert!(!in_jailers_tower);
        assert!(!walk_in);
        assert_eq!(invite_results, 0);
    }

    #[test]
    fn preserves_existing_party_info_static_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_PartyInfo = C_PartyInfo or __wow_namespace()

            function C_PartyInfo.AllowedToDoPartyConversion()
                return true
            end

            function C_PartyInfo.GetInviteConfirmationInfo(_guid)
                return "invite"
            end
            "#,
        )
        .expect("fixture should install existing party info provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (bool, String) = env
            .eval(
                r#"
                return C_PartyInfo.AllowedToDoPartyConversion(),
                       C_PartyInfo.GetInviteConfirmationInfo("Player-1234-ABCDEF")
                "#,
            )
            .expect("existing party info provider should remain callable");

        assert_eq!(result, (true, "invite".to_string()));
    }
}
