//! Temporary raid-lock defaults until instance lock state is modeled.
//! Merchant inventory is owned by `c_api::c_merchant_frame`.

const MERCHANT_RAID_DEFAULTS_LUA: &str = r#"
C_RaidLocks = C_RaidLocks or __wow_namespace()

if rawget(C_RaidLocks, "IsEncounterComplete") == nil then
    function C_RaidLocks.IsEncounterComplete(_mapID, _encounterID)
        return false
    end
end

if rawget(C_RaidLocks, "RequestRaidInfo") == nil then
    function C_RaidLocks.RequestRaidInfo()
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(MERCHANT_RAID_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_raid_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(C_RaidLocks.IsEncounterComplete) ~= "function" then return "raid_method" end
                if C_RaidLocks.IsEncounterComplete(1, 2) ~= false then return "raid_complete" end
                if C_RaidLocks.RequestRaidInfo() ~= nil then return "raid_request" end
                return "ok"
                "#,
            )
            .expect("merchant/raid default probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_merchant_and_raid_providers() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_MerchantFrame = C_MerchantFrame or __wow_namespace()
            C_RaidLocks = C_RaidLocks or __wow_namespace()

            function C_MerchantFrame.GetItemInfo(_index)
                return { name = "existing", price = 7 }
            end
            function C_RaidLocks.IsEncounterComplete(_mapID, _encounterID)
                return true
            end
            function C_RaidLocks.RequestRaidInfo()
                return "requested"
            end
            "#,
        )
        .expect("fixture should install existing providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                local item = C_MerchantFrame.GetItemInfo(1)
                return item.name .. ":" .. item.price .. ":" ..
                    tostring(C_RaidLocks.IsEncounterComplete(1, 2)) .. ":" ..
                    C_RaidLocks.RequestRaidInfo()
                "#,
            )
            .expect("existing merchant/raid providers should remain callable");

        assert_eq!(result, "existing:7:true:requested");
    }
}
