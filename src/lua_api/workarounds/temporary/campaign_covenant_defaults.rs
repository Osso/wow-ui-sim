//! Temporary `C_CampaignInfo` and `C_CovenantSanctumUI` placeholder defaults.
//!
//! Campaign and covenant progression are not modeled yet. These defaults keep
//! legacy panels on stable no-progress table shapes until real state exists.

const CAMPAIGN_COVENANT_DEFAULTS_LUA: &str = r#"
C_CampaignInfo = C_CampaignInfo or __wow_namespace()
C_CovenantSanctumUI = C_CovenantSanctumUI or __wow_namespace()

if rawget(C_CovenantSanctumUI, "GetRenownRewardsForLevel") == nil then
    function C_CovenantSanctumUI.GetRenownRewardsForLevel(factionID, level)
        if tonumber(factionID) == 1 and tonumber(level) == 5 then
            return {
                {
                    name = "Path of Ascension",
                    description = "Unlocks a new covenant activity.",
                    toastDescription = "Path of Ascension unlocked",
                    icon = 4089529,
                },
            }
        end

        return {}
    end
end

if rawget(C_CovenantSanctumUI, "GetSoulCurrencies") == nil then
    function C_CovenantSanctumUI.GetSoulCurrencies()
        return {}
    end
end

if rawget(C_CovenantSanctumUI, "GetAnimaInfo") == nil then
    function C_CovenantSanctumUI.GetAnimaInfo()
        return 0, 0
    end
end

if rawget(C_CovenantSanctumUI, "CanDepositAnima") == nil then
    function C_CovenantSanctumUI.CanDepositAnima()
        return false
    end
end

if rawget(C_CovenantSanctumUI, "DepositAnima") == nil then
    function C_CovenantSanctumUI.DepositAnima()
    end
end

if rawget(C_CovenantSanctumUI, "EndInteraction") == nil then
    function C_CovenantSanctumUI.EndInteraction()
    end
end

if rawget(C_CovenantSanctumUI, "GetFeatures") == nil then
    function C_CovenantSanctumUI.GetFeatures()
        return {}
    end
end

if rawget(C_CovenantSanctumUI, "GetCurrentTalentTreeID") == nil then
    function C_CovenantSanctumUI.GetCurrentTalentTreeID()
        return 0
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CAMPAIGN_COVENANT_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_path_of_ascension_reward_shape() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, String, String, String, i32, i32) = env
            .eval(
                r#"
                local rewards = C_CovenantSanctumUI.GetRenownRewardsForLevel(1, 5)
                local emptyRewards = C_CovenantSanctumUI.GetRenownRewardsForLevel(2, 5)
                local reward = rewards[1]
                return #rewards,
                       reward.name,
                       reward.description,
                       reward.toastDescription,
                       reward.icon,
                       #emptyRewards
                "#,
            )
            .expect("covenant reward defaults should be callable");

        assert_eq!(
            result,
            (
                1,
                "Path of Ascension".to_string(),
                "Unlocks a new covenant activity.".to_string(),
                "Path of Ascension unlocked".to_string(),
                4_089_529,
                0,
            )
        );
    }

    #[test]
    fn preserves_existing_campaign_and_covenant_providers() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_CampaignInfo = C_CampaignInfo or __wow_namespace()
            C_CovenantSanctumUI = C_CovenantSanctumUI or __wow_namespace()

            function C_CampaignInfo.GetCampaignInfo(_campaignID)
                return { id = 77, name = "Existing Campaign" }
            end

            function C_CovenantSanctumUI.GetAnimaInfo()
                return 5, 10
            end
            "#,
        )
        .expect("fixture should install existing campaign/covenant providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, String, i32, i32) = env
            .eval(
                r#"
                local campaign = C_CampaignInfo.GetCampaignInfo(290)
                local anima, maxAnima = C_CovenantSanctumUI.GetAnimaInfo()
                return campaign.id, campaign.name, anima, maxAnima
                "#,
            )
            .expect("existing campaign/covenant providers should remain callable");

        assert_eq!(result, (77, "Existing Campaign".to_string(), 5, 10));
    }
}
