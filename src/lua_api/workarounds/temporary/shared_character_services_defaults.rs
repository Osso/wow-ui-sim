//! Temporary `C_SharedCharacterServices` defaults.
//!
//! Shared upgrade-distribution state is not modeled yet. Return an empty list
//! so character-service UI code sees no available shared upgrades until that
//! state exists.

const SHARED_CHARACTER_SERVICES_DEFAULTS_LUA: &str = r#"
C_SharedCharacterServices = C_SharedCharacterServices or __wow_namespace()

if rawget(C_SharedCharacterServices, "GetUpgradeDistributions") == nil then
    function C_SharedCharacterServices.GetUpgradeDistributions()
        return {}
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SHARED_CHARACTER_SERVICES_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_upgrade_distribution_empty_list_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let count: i32 = env
            .eval(
                r#"
                local distributions = C_SharedCharacterServices.GetUpgradeDistributions()
                return #distributions
                "#,
            )
            .expect("upgrade distributions should be queryable");

        assert_eq!(count, 0);
    }

    #[test]
    fn preserves_existing_upgrade_distribution_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_SharedCharacterServices = C_SharedCharacterServices or __wow_namespace()

            function C_SharedCharacterServices.GetUpgradeDistributions()
                return { { distributionID = 42 } }
            end
            "#,
        )
        .expect("fixture should install existing C_SharedCharacterServices provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let distribution_id: i32 = env
            .eval(
                r#"
                local distributions = C_SharedCharacterServices.GetUpgradeDistributions()
                return distributions[1].distributionID
                "#,
            )
            .expect("existing upgrade distribution provider should remain callable");

        assert_eq!(distribution_id, 42);
    }
}
