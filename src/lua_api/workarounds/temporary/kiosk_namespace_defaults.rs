//! Temporary `Kiosk` namespace defaults.
//!
//! Kiosk sessions and credential injection are not modeled. These defaults keep
//! glue startup inert until simulator-backed kiosk state replaces this workaround.

const KIOSK_NAMESPACE_DEFAULTS_LUA: &str = r#"
if type(Kiosk) ~= "table" then
    Kiosk = {}
end

if Kiosk.IsEnabled == nil then
    function Kiosk.IsEnabled()
        return false
    end
end

if Kiosk.IsCompetitiveModeEnabled == nil then
    function Kiosk.IsCompetitiveModeEnabled()
        return false
    end
end

if Kiosk.GetKioskLoginInfo == nil then
    function Kiosk.GetKioskLoginInfo()
        return nil, nil, nil
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(KIOSK_NAMESPACE_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_inert_kiosk_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(Kiosk) ~= "table" then
                    return "missing_namespace"
                end
                if Kiosk.IsEnabled() ~= false then
                    return "enabled"
                end
                if Kiosk.IsCompetitiveModeEnabled() ~= false then
                    return "competitive"
                end
                if type(Kiosk.GetKioskLoginInfo) ~= "function" then
                    return "missing_login_info"
                end
                local accountName, password, realmAddress = Kiosk.GetKioskLoginInfo()
                if accountName ~= nil or password ~= nil or realmAddress ~= nil then
                    return "unexpected_login_info"
                end
                return "ok"
                "#,
            )
            .expect("kiosk defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_kiosk_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            Kiosk = {
                IsEnabled = function() return true end,
                GetKioskLoginInfo = function() return "account", "password", 123 end,
                ExistingMember = 42,
            }
            "#,
        )
        .expect("fixture should install existing Kiosk table");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("kiosk defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if Kiosk.IsEnabled() ~= true then
                    return "overwrote_existing"
                end
                if Kiosk.ExistingMember ~= 42 then
                    return "lost_member"
                end
                local accountName, password, realmAddress = Kiosk.GetKioskLoginInfo()
                if accountName ~= "account" or password ~= "password" or realmAddress ~= 123 then
                    return "overwrote_login_info"
                end
                if Kiosk.IsCompetitiveModeEnabled() ~= false then
                    return "missing_default"
                end
                return "ok"
                "#,
            )
            .expect("kiosk preservation probe should run");

        assert_eq!(result, "ok");
    }
}
