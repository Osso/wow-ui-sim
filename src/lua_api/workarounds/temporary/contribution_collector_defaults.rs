//! Temporary `C_ContributionCollector` empty/default surface.
//!
//! Contribution collectors are not modeled yet. These methods preserve the
//! empty return shapes Blizzard startup code expects until backing state exists.

const CONTRIBUTION_COLLECTOR_DEFAULTS_LUA: &str = r#"
C_ContributionCollector = C_ContributionCollector or __wow_namespace()

local function emptyTable()
    return {}
end

local function emptyString()
    return ""
end

local function falseValue()
    return false
end

local function noResult()
end

local function enumValue(enumName, variantName, fallback)
    if Enum and Enum[enumName] and Enum[enumName][variantName] ~= nil then
        return Enum[enumName][variantName]
    end

    return fallback
end

local function defaultColor()
    if type(CreateColor) == "function" then
        return CreateColor(1, 1, 1, 1)
    end

    return {
        GetRGB = function()
            return 1, 1, 1
        end,
        GetRGBA = function()
            return 1, 1, 1, 1
        end,
    }
end

local function contributionAppearance()
    return {
        stateName = "",
        stateColor = defaultColor(),
        tooltipLine = "",
        tooltipUseTimeRemaining = false,
        statusBarAtlas = "",
        borderAtlas = "",
        bannerAtlas = "",
    }
end

local function contributionResult()
    return enumValue("ContributionResult", "Success", 0)
end

local function contributionOrderIndex(contributionID)
    return tonumber(contributionID) or 0
end

local function rewardQuestID()
    return 0
end

local function contributionState()
    return enumValue("ContributionState", "None", 0), 0, nil, 0
end

local defaults = {
    Close = noResult,
    Contribute = noResult,
    GetActive = noResult,
    HasPendingContribution = falseValue,
    IsAwaitingRewardQuestData = falseValue,
    GetAtlases = emptyTable,
    GetBuffs = noResult,
    GetContributionAppearance = contributionAppearance,
    GetContributionCollectorsForMap = emptyTable,
    GetContributionResult = contributionResult,
    GetDescription = emptyString,
    GetManagedContributionsForCreatureID = noResult,
    GetName = emptyString,
    GetOrderIndex = contributionOrderIndex,
    GetState = contributionState,
    GetRequiredContributionCurrency = noResult,
    GetRequiredContributionItem = noResult,
    GetRewardQuestID = rewardQuestID,
}

for name, provider in pairs(defaults) do
    if rawget(C_ContributionCollector, name) == nil then
        C_ContributionCollector[name] = provider
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CONTRIBUTION_COLLECTOR_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_contribution_collector_empty_default_shapes() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: (bool, String, bool, bool, i32, i32, bool) = env
            .eval(
                r#"
                local appearance = C_ContributionCollector.GetContributionAppearance(1)
                local r, g, b, a = appearance.stateColor:GetRGBA()
                local state, _, itemName, _ = C_ContributionCollector.GetState(1)
                return
                    type(C_ContributionCollector.GetAtlases(1)) == "table",
                    appearance.stateName,
                    appearance.tooltipUseTimeRemaining,
                    r == 1 and g == 1 and b == 1 and a == 1,
                    C_ContributionCollector.GetOrderIndex(17),
                    C_ContributionCollector.GetRewardQuestID(1),
                    itemName == nil and state == Enum.ContributionState.None
                "#,
            )
            .expect("contribution collector defaults should be readable");

        assert_eq!(result, (true, String::new(), false, true, 17, 0, true));
    }

    #[test]
    fn reports_no_active_or_pending_contributions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: (bool, bool, bool, bool) = env
            .eval(
                r#"
                return
                    C_ContributionCollector.GetActive() == nil,
                    C_ContributionCollector.GetRequiredContributionCurrency(1) == nil,
                    C_ContributionCollector.HasPendingContribution(1),
                    C_ContributionCollector.IsAwaitingRewardQuestData(1)
                "#,
            )
            .expect("contribution collector no-op methods should be readable");

        assert_eq!(result, (true, true, false, false));
    }

    #[test]
    fn preserves_existing_contribution_collector_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_ContributionCollector = C_ContributionCollector or __wow_namespace()

            function C_ContributionCollector.GetActive()
                return 44
            end

            function C_ContributionCollector.GetName(_contributionID)
                return "Existing"
            end
            "#,
        )
        .expect("fixture should install existing contribution collector provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, String, bool) = env
            .eval(
                r#"
                return C_ContributionCollector.GetActive(),
                       C_ContributionCollector.GetName(1),
                       type(C_ContributionCollector.GetAtlases(1)) == "table"
                "#,
            )
            .expect("existing contribution collector provider should remain callable");

        assert_eq!(result, (44, "Existing".to_string(), true));
    }
}
