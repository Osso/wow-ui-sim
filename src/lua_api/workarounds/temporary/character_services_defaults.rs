//! Temporary `C_CharacterServices` service/display defaults.
//!
//! Active boost and class-trial type probes are state-backed in Rust. Service
//! entitlement, VAS distribution, display metadata, and assignment state are not
//! modeled yet, so these functions preserve the inert startup-compatible shape.

const CHARACTER_SERVICES_DEFAULTS_LUA: &str = r#"
C_CharacterServices = C_CharacterServices or __wow_namespace()

if rawget(C_CharacterServices, "HasRequiredServiceForCharacterUpgrade") == nil then
    function C_CharacterServices.HasRequiredServiceForCharacterUpgrade()
        return false
    end
end

if rawget(C_CharacterServices, "HasRequiredBoostForClassTrial") == nil then
    function C_CharacterServices.HasRequiredBoostForClassTrial()
        return false
    end
end

if rawget(C_CharacterServices, "GetCharacterServiceDisplayInfo") == nil then
    function C_CharacterServices.GetCharacterServiceDisplayInfo()
        return {}
    end
end

if rawget(C_CharacterServices, "GetVASDistributions") == nil then
    function C_CharacterServices.GetVASDistributions()
        return {}
    end
end

if rawget(C_CharacterServices, "GetCharacterServiceDisplayData") == nil then
    function C_CharacterServices.GetCharacterServiceDisplayData()
        return {
            boostLevel = 80,
            flowTitle = "Character Upgrade",
            popupInfo = {
                textureKit = "characterupdate",
            },
        }
    end
end

if rawget(C_CharacterServices, "AssignUpgradeDistribution") == nil then
    function C_CharacterServices.AssignUpgradeDistribution()
    end
end

if rawget(C_CharacterServices, "AssignPCTDistribution") == nil then
    function C_CharacterServices.AssignPCTDistribution()
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CHARACTER_SERVICES_DEFAULTS_LUA)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        // User-approved default until queued character upgrades are modeled.
        lua.exec("function C_CharacterServices.HasQueuedUpgrade() return false end")?;
        // User-approved, not native evidence: class-trial boosts are not modeled.
        lua.exec("function C_CharacterServices.IsTrialBoostEnabled() return false end")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_character_services_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, bool, i32, i32, bool, bool) = env
            .eval(
                r#"
                C_CharacterServices.AssignUpgradeDistribution()
                C_CharacterServices.AssignPCTDistribution()
                local data = C_CharacterServices.GetCharacterServiceDisplayData()
                return C_CharacterServices.HasRequiredServiceForCharacterUpgrade(),
                       C_CharacterServices.HasRequiredBoostForClassTrial(),
                       #C_CharacterServices.GetCharacterServiceDisplayInfo(),
                       #C_CharacterServices.GetVASDistributions(),
                       data.boostLevel == 80
                           and data.flowTitle == "Character Upgrade"
                           and data.popupInfo.textureKit == "characterupdate",
                       true
                "#,
            )
            .expect("character service defaults should be callable");

        assert_eq!(result, (false, false, 0, 0, true, true));
    }

    #[test]
    fn preserves_existing_character_services_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_CharacterServices = C_CharacterServices or __wow_namespace()

            function C_CharacterServices.HasRequiredServiceForCharacterUpgrade()
                return true
            end

            function C_CharacterServices.GetCharacterServiceDisplayData()
                return { boostLevel = 70, flowTitle = "Existing" }
            end
            "#,
        )
        .expect("fixture should install existing character service provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (bool, i32, String) = env
            .eval(
                r#"
                local data = C_CharacterServices.GetCharacterServiceDisplayData()
                return C_CharacterServices.HasRequiredServiceForCharacterUpgrade(),
                       data.boostLevel,
                       data.flowTitle
                "#,
            )
            .expect("existing character service provider should remain callable");

        assert_eq!(result, (true, 70, "Existing".to_string()));
    }
}
