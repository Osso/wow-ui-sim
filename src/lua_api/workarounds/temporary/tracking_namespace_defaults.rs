//! Temporary defaults for unmodeled neighborhood-initiative members.
//!
//! Retail content-tracking queries and available-house XP are backed by c_api.

const TRACKING_NAMESPACE_DEFAULTS_LUA: &str = r#"
C_ContentTracking = C_ContentTracking or __wow_namespace()
C_NeighborhoodInitiative = C_NeighborhoodInitiative or __wow_namespace()

local function installNeighborhoodInitiativeDefault(name, fn)
    if rawget(C_NeighborhoodInitiative, name) == nil then
        C_NeighborhoodInitiative[name] = fn
    end
end

installNeighborhoodInitiativeDefault("IsInitiativeEnabled", function()
    return false
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

    modern_engine_namespace_fixture! {
    #[test]
    fn keeps_unmodeled_neighborhood_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(C_ContentTracking) ~= "table" then return "content-tracking-namespace" end
                if C_NeighborhoodInitiative.IsInitiativeEnabled() ~= false then return "enabled" end
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
