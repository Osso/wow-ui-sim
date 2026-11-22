//! Temporary C_PartyInfo instance-abandon vote defaults.
//!
//! Active abandon votes, per-player responses, and shutdown timers are not
//! modeled yet. Keep the inert "no vote in progress" shape explicit until
//! backing vote state exists.

const PARTY_INFO_INSTANCE_ABANDON_DEFAULTS_LUA: &str = r#"
C_PartyInfo = C_PartyInfo or __wow_namespace()

if rawget(C_PartyInfo, "GetInstanceAbandonVoteTime") == nil then
    function C_PartyInfo.GetInstanceAbandonVoteTime()
        return 0, 0
    end
end

if rawget(C_PartyInfo, "GetInstanceAbandonShutdownTime") == nil then
    function C_PartyInfo.GetInstanceAbandonShutdownTime()
        return 0, 0
    end
end

if rawget(C_PartyInfo, "GetInstanceAbandonVoteResponse") == nil then
    function C_PartyInfo.GetInstanceAbandonVoteResponse()
        return nil
    end
end

if rawget(C_PartyInfo, "SetInstanceAbandonVoteResponse") == nil then
    function C_PartyInfo.SetInstanceAbandonVoteResponse(_response)
    end
end

if rawget(C_PartyInfo, "GetNumInstanceAbandonGroupVoteResponses") == nil then
    function C_PartyInfo.GetNumInstanceAbandonGroupVoteResponses()
        return 0
    end
end

if rawget(C_PartyInfo, "CanStartInstanceAbandonVote") == nil then
    function C_PartyInfo.CanStartInstanceAbandonVote()
        return false
    end
end

if rawget(C_PartyInfo, "StartInstanceAbandonVote") == nil then
    function C_PartyInfo.StartInstanceAbandonVote()
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PARTY_INFO_INSTANCE_ABANDON_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_party_info_instance_abandon_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, i32, i32, i32, bool, bool, bool, i32, bool) = env
            .eval(
                r#"
                local voteDuration, voteTimeLeft = C_PartyInfo.GetInstanceAbandonVoteTime()
                local shutdownDuration, shutdownTimeLeft = C_PartyInfo.GetInstanceAbandonShutdownTime()
                local response = C_PartyInfo.GetInstanceAbandonVoteResponse()
                local setOK = pcall(C_PartyInfo.SetInstanceAbandonVoteResponse, true)
                local startOK = pcall(C_PartyInfo.StartInstanceAbandonVote)

                return voteDuration, voteTimeLeft,
                       shutdownDuration, shutdownTimeLeft,
                       response == nil,
                       setOK,
                       startOK,
                       C_PartyInfo.GetNumInstanceAbandonGroupVoteResponses(),
                       C_PartyInfo.CanStartInstanceAbandonVote()
                "#,
            )
            .expect("party info instance-abandon defaults should be callable");

        assert_eq!(result, (0, 0, 0, 0, true, true, true, 0, false));
    }

    #[test]
    fn preserves_existing_party_info_instance_abandon_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_PartyInfo = C_PartyInfo or __wow_namespace()

            function C_PartyInfo.GetInstanceAbandonVoteTime()
                return 9, 8
            end

            function C_PartyInfo.CanStartInstanceAbandonVote()
                return true
            end
            "#,
        )
        .expect("fixture should install existing party info provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32, bool) = env
            .eval(
                r#"
                local voteDuration, voteTimeLeft = C_PartyInfo.GetInstanceAbandonVoteTime()
                return voteDuration, voteTimeLeft, C_PartyInfo.CanStartInstanceAbandonVote()
                "#,
            )
            .expect("existing party info provider should remain callable");

        assert_eq!(result, (9, 8, true));
    }
}
