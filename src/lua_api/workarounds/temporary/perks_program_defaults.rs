//! Temporary `C_PerksProgram` empty-catalog defaults.
//!
//! Trader's Tender vendor/catalog state is not modeled yet. These defaults keep
//! Trading Post and Monthly Activities startup paths well-typed until real
//! catalog, currency, and pending reward state exists.

const PERKS_PROGRAM_DEFAULTS_LUA: &str = r#"
C_PerksProgram = C_PerksProgram or __wow_namespace()

if rawget(C_PerksProgram, "GetAvailableVendorItemIDs") == nil then
    function C_PerksProgram.GetAvailableVendorItemIDs()
        return {}
    end
end

if rawget(C_PerksProgram, "GetCurrencyAmount") == nil then
    function C_PerksProgram.GetCurrencyAmount()
        return 0
    end
end

if rawget(C_PerksProgram, "GetPendingChestRewards") == nil then
    function C_PerksProgram.GetPendingChestRewards()
        return {}
    end
end

if rawget(C_PerksProgram, "RequestPendingChestRewards") == nil then
    function C_PerksProgram.RequestPendingChestRewards()
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PERKS_PROGRAM_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_perks_program_empty_catalog_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, i32, String, i32, bool) = env
            .eval(
                r##"
                local vendorItems = C_PerksProgram.GetAvailableVendorItemIDs()
                local rewards = C_PerksProgram.GetPendingChestRewards()
                local requestOK = pcall(C_PerksProgram.RequestPendingChestRewards)
                return #vendorItems,
                       C_PerksProgram.GetCurrencyAmount(),
                       type(rewards),
                       #rewards,
                       requestOK
                "##,
            )
            .expect("perks program defaults should be callable");

        assert_eq!(result, (0, 0, "table".to_string(), 0, true));
    }

    #[test]
    fn preserves_existing_perks_program_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_PerksProgram = C_PerksProgram or __wow_namespace()

            function C_PerksProgram.GetCurrencyAmount()
                return 777
            end

            function C_PerksProgram.GetAvailableVendorItemIDs()
                return { 11, 22 }
            end
            "#,
        )
        .expect("fixture should install existing perks provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32) = env
            .eval(
                r#"
                return C_PerksProgram.GetCurrencyAmount(),
                       #C_PerksProgram.GetAvailableVendorItemIDs()
                "#,
            )
            .expect("existing perks provider should remain callable");

        assert_eq!(result, (777, 2));
    }
}
