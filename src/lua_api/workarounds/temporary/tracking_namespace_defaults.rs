//! Temporary tracking namespace defaults.
//!
//! Content tracking and neighborhood initiatives are queried by objective
//! tracker startup code. Until their backing state exists, expose empty
//! tracked-list shapes from the workaround layer.

const TRACKING_NAMESPACE_DEFAULTS_LUA: &str = r#"
C_ContentTracking = C_ContentTracking or __wow_namespace()
C_NeighborhoodInitiative = C_NeighborhoodInitiative or __wow_namespace()

local function installContentTrackingDefault(name, fn)
    if rawget(C_ContentTracking, name) == nil then
        C_ContentTracking[name] = fn
    end
end

local function installNeighborhoodInitiativeDefault(name, fn)
    if rawget(C_NeighborhoodInitiative, name) == nil then
        C_NeighborhoodInitiative[name] = fn
    end
end

installContentTrackingDefault("GetTrackedIDs", function()
    return {}
end)

installContentTrackingDefault("IsTracking", function()
    return false
end)

installNeighborhoodInitiativeDefault("IsInitiativeEnabled", function()
    return false
end)

installNeighborhoodInitiativeDefault("GetAvailableHouseXP", function()
    return 0
end)

installNeighborhoodInitiativeDefault("GetInitiativeTaskInfo", function()
    return nil
end)

"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(TRACKING_NAMESPACE_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_empty_tracking_namespace_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local tracked = C_ContentTracking.GetTrackedIDs()
                if type(tracked) ~= "table" or #tracked ~= 0 then return "tracked" end
                if C_ContentTracking.IsTracking(42) ~= false then return "tracking" end
                if C_NeighborhoodInitiative.IsInitiativeEnabled() ~= false then return "enabled" end
                if C_NeighborhoodInitiative.GetAvailableHouseXP() ~= 0 then return "xp" end
                local tasks = C_NeighborhoodInitiative.GetTrackedInitiativeTasks()
                if type(tasks) ~= "table" then return "tasks" end
                if type(tasks.trackedIDs) ~= "table" or #tasks.trackedIDs ~= 0 then return "ids" end
                if C_NeighborhoodInitiative.GetInitiativeTaskInfo(1) ~= nil then return "info" end
                if C_NeighborhoodInitiative.RemoveTrackedInitiativeTask(1) ~= nil then return "remove" end
                if C_NeighborhoodInitiative.AddTrackedInitiativeTask(1) ~= nil then return "add" end
                return "ok"
                "#,
            )
            .expect("tracking namespace defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_tracking_namespace_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_ContentTracking.GetTrackedIDs()
                return { 7 }
            end
            function C_NeighborhoodInitiative.GetAvailableHouseXP()
                return 123
            end
            "#,
        )
        .expect("fixture should install existing tracking providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32) = env
            .eval(
                r#"
                return C_ContentTracking.GetTrackedIDs()[1],
                    C_NeighborhoodInitiative.GetAvailableHouseXP()
                "#,
            )
            .expect("existing tracking providers should remain callable");

        assert_eq!(result, (7, 123));
    }
}
