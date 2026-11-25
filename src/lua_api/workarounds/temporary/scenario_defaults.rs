//! Temporary `C_Scenario` not-in-scenario defaults.
//!
//! Scenario participation state is not modeled yet. Objective tracker callers
//! expect numeric zero counts rather than nils when the player is not in a
//! scenario, so keep that compatibility shape explicit here.

const SCENARIO_DEFAULTS_LUA: &str = r#"
C_Scenario = C_Scenario or __wow_namespace()

local function installScenarioDefault(name, fn)
    if rawget(C_Scenario, name) == nil then
        C_Scenario[name] = fn
    end
end

installScenarioDefault("GetInfo", function()
    return nil, 0, 0, 0, nil, nil, nil, 0, 0, 0, nil, "evergreen-scenario", 0
end)

installScenarioDefault("IsInScenario", function()
    return false
end)

installScenarioDefault("GetStepInfo", function()
    return nil, 0, 0, false, false, 0, 0, 0, 0, false, false
end)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SCENARIO_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_scenario_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local scenarioName, currentStage, numStages, flags, _, _, _, xp, money, scenarioType, _, textureKit, scenarioID = C_Scenario.GetInfo()
                local title, currentCriteria, numCriteria, _, _, _, _, _, _, _, _ = C_Scenario.GetStepInfo()

                if scenarioName ~= nil then return "scenarioName" end
                if currentStage ~= 0 then return "currentStage" end
                if numStages ~= 0 then return "numStages" end
                if flags ~= 0 then return "flags" end
                if xp ~= 0 then return "xp" end
                if money ~= 0 then return "money" end
                if scenarioType ~= 0 then return "scenarioType" end
                if textureKit ~= "evergreen-scenario" then return "textureKit" end
                if scenarioID ~= 0 then return "scenarioID" end
                if C_Scenario.IsInScenario() ~= false then return "isInScenario" end
                if title ~= nil then return "stepTitle" end
                if currentCriteria ~= 0 then return "currentCriteria" end
                if numCriteria ~= 0 then return "numCriteria" end

                return "ok"
                "#,
            )
            .expect("scenario defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_scenario_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_Scenario = C_Scenario or __wow_namespace()

            function C_Scenario.IsInScenario()
                return true
            end
            function C_Scenario.GetInfo()
                return "existing", 2
            end
            "#,
        )
        .expect("fixture should install existing C_Scenario providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (bool, String, i32) = env
            .eval(
                r#"
                local name, stage = C_Scenario.GetInfo()
                return C_Scenario.IsInScenario(), name, stage
                "#,
            )
            .expect("existing C_Scenario providers should remain callable");

        assert_eq!(result, (true, "existing".to_string(), 2));
    }
}
