//! Temporary `C_MajorFactions` display-policy defaults.
//!
//! Renown faction data, IDs, levels, and reward rows remain state-backed in Rust.
//! Expansion-page visibility, journey/reward-track policy, current renown level,
//! and NPC faction mapping are not modeled yet.

const MAJOR_FACTION_DISPLAY_DEFAULTS_LUA: &str = r#"
C_MajorFactions = C_MajorFactions or __wow_namespace()

if rawget(C_MajorFactions, "IsMajorFactionHiddenFromExpansionPage") == nil then
    function C_MajorFactions.IsMajorFactionHiddenFromExpansionPage(_factionID)
        return false
    end
end

if rawget(C_MajorFactions, "ShouldDisplayMajorFactionAsJourney") == nil then
    function C_MajorFactions.ShouldDisplayMajorFactionAsJourney(_factionID)
        return false
    end
end

if rawget(C_MajorFactions, "HasMaximumRenown") == nil then
    function C_MajorFactions.HasMaximumRenown(_factionID)
        return false
    end
end

if rawget(C_MajorFactions, "GetCurrentRenownLevel") == nil then
    function C_MajorFactions.GetCurrentRenownLevel(_factionID)
        return 1
    end
end

if rawget(C_MajorFactions, "ShouldUseJourneyRewardTrack") == nil then
    function C_MajorFactions.ShouldUseJourneyRewardTrack(_factionID)
        return false
    end
end

if rawget(C_MajorFactions, "GetRenownNPCFactionID") == nil then
    function C_MajorFactions.GetRenownNPCFactionID(_factionID)
        return 0
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(MAJOR_FACTION_DISPLAY_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_major_faction_display_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, bool, bool, i32, i32, bool, i32) = env
            .eval(
                r#"
                return C_MajorFactions.IsMajorFactionHiddenFromExpansionPage(2507),
                       C_MajorFactions.ShouldDisplayMajorFactionAsJourney(2507),
                       C_MajorFactions.HasMaximumRenown(2507),
                       C_MajorFactions.GetCurrentRenownLevel(2507),
                       #C_MajorFactions.GetRenownRewardsForLevel(2507, 1),
                       C_MajorFactions.ShouldUseJourneyRewardTrack(2507),
                       C_MajorFactions.GetRenownNPCFactionID(2507)
                "#,
            )
            .expect("major faction display defaults should be callable");

        assert_eq!(result, (false, false, false, 1, 0, false, 0));
    }

    #[test]
    fn preserves_existing_major_faction_display_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_MajorFactions = C_MajorFactions or __wow_namespace()

            function C_MajorFactions.GetCurrentRenownLevel(_factionID)
                return 9
            end

            function C_MajorFactions.GetRenownRewardsForLevel(_factionID, _level)
                return { { itemID = 1 } }
            end
            "#,
        )
        .expect("fixture should install existing major faction display provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32) = env
            .eval(
                r#"
                return C_MajorFactions.GetCurrentRenownLevel(2507),
                       #C_MajorFactions.GetRenownRewardsForLevel(2507, 1)
                "#,
            )
            .expect("existing major faction display provider should remain callable");

        assert_eq!(result, (9, 1));
    }
}
