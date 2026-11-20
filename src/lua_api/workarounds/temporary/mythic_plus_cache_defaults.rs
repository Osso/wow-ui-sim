//! Temporary `C_MythicPlus` cache/request defaults.
//!
//! The core Mythic+ query surface is backed by `SimState.mythic_plus`. Weekly
//! chest cache state and server refresh requests are not modeled yet, so keep
//! those inert compatibility defaults explicit here.

const MYTHIC_PLUS_CACHE_DEFAULTS_LUA: &str = r#"
C_MythicPlus = C_MythicPlus or __wow_namespace()

local function installMythicPlusDefault(name, fn)
    if rawget(C_MythicPlus, name) == nil then
        C_MythicPlus[name] = fn
    end
end

installMythicPlusDefault("GetLastWeeklyChest", function()
end)

installMythicPlusDefault("RequestCurrentAffixes", function()
end)

installMythicPlusDefault("RequestMapInfo", function()
end)

installMythicPlusDefault("RequestRewards", function()
end)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(MYTHIC_PLUS_CACHE_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_mythic_plus_cache_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r##"
                local function countReturns(...)
                    return select("#", ...)
                end

                if countReturns(C_MythicPlus.GetLastWeeklyChest()) ~= 0 then
                    return "weekly_chest"
                end
                if countReturns(C_MythicPlus.RequestCurrentAffixes()) ~= 0 then
                    return "current_affixes"
                end
                if countReturns(C_MythicPlus.RequestMapInfo()) ~= 0 then
                    return "map_info"
                end
                if countReturns(C_MythicPlus.RequestRewards()) ~= 0 then
                    return "rewards"
                end
                return "ok"
                "##,
            )
            .expect("Mythic+ cache defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn preserves_existing_mythic_plus_cache_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_MythicPlus = C_MythicPlus or __wow_namespace()

            function C_MythicPlus.GetLastWeeklyChest()
                return "existing"
            end
            function C_MythicPlus.RequestRewards()
                return "requested"
            end
            "#,
        )
        .expect("fixture should install existing C_MythicPlus providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (String, String) = env
            .eval(
                r#"
                return C_MythicPlus.GetLastWeeklyChest(),
                    C_MythicPlus.RequestRewards()
                "#,
            )
            .expect("existing C_MythicPlus providers should remain callable");

        assert_eq!(result, ("existing".to_string(), "requested".to_string()));
    }
}
