use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

/// Covers the Lua scenario fallback. Retail 12.0.5+ and Forever publish the
/// supplied-scenario model instead (4347778476, INFERRED, R88dh pending).
#[cfg(not(any(feature = "retail-12-0-5", feature = "client-wowforever")))]
#[test]
fn scenario_defaults_match_not_in_scenario_state() {
    let env = env();
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
        .expect("C_Scenario default surface should be callable");
    assert_eq!(result, "ok");
}
