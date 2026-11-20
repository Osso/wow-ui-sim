//! Temporary `C_SocialQueue` defaults.
//!
//! Other client profiles still use empty-result compatibility behavior.
//! Retail and WoW Forever provide modeled, host-supplied Quick Join state.

const SOCIAL_QUEUE_DEFAULTS_LUA: &str = r#"
C_SocialQueue = C_SocialQueue or __wow_namespace()

local function installSocialQueueDefault(name, fn)
    if rawget(C_SocialQueue, name) == nil then
        C_SocialQueue[name] = fn
    end
end

if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100 then
    installSocialQueueDefault("IsSystemEnabled", function()
        return false
    end)

    installSocialQueueDefault("IsSystemSupported", function()
        return false
    end)
end

installSocialQueueDefault("GetAllGroups", function()
    return {}
end)

installSocialQueueDefault("GetConfig", function()
    return {}
end)

installSocialQueueDefault("GetGroupForPlayer", function()
    return nil
end)

installSocialQueueDefault("GetGroupInfo", function()
    return nil
end)

installSocialQueueDefault("GetGroupMembers", function()
    return {}
end)

installSocialQueueDefault("GetGroupQueues", function()
    return {}
end)

installSocialQueueDefault("RequestToJoin", function()
end)

installSocialQueueDefault("SignalToastDisplayed", function()
end)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SOCIAL_QUEUE_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_empty_social_queue_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, i32, bool, bool, i32, i32) = env
            .eval(
                r#"
                C_SocialQueue.RequestToJoin(1)
                C_SocialQueue.SignalToastDisplayed(1)
                return #C_SocialQueue.GetAllGroups(),
                    #C_SocialQueue.GetConfig(),
                    C_SocialQueue.GetGroupForPlayer("Player") == nil,
                    C_SocialQueue.GetGroupInfo(1) == nil,
                    #C_SocialQueue.GetGroupMembers(1),
                    #C_SocialQueue.GetGroupQueues(1)
                "#,
            )
            .expect("social queue defaults should be callable");

        assert_eq!(result, (0, 0, true, true, 0, 0));
    }

    #[test]
    fn preserves_existing_social_queue_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_SocialQueue.GetAllGroups()
                return { "existing" }
            end
            "#,
        )
        .expect("fixture should install existing social queue provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let first_group: String = env
            .eval("local groups = C_SocialQueue.GetAllGroups(); return groups[1]")
            .expect("existing social queue provider should remain callable");

        assert_eq!(first_group, "existing");
    }
}
