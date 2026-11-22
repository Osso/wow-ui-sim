//! Temporary `C_Ping` defaults.
//!
//! The simulator does not model ping target options yet. Keep the inert empty
//! option list in the temporary workaround layer until ping state exists.

const PING_DEFAULTS_LUA: &str = r#"
C_Ping = C_Ping or __wow_namespace()

if rawget(C_Ping, "GetDefaultPingOptions") == nil then
    function C_Ping.GetDefaultPingOptions()
        return {}
    end
end

if rawget(C_Ping, "SendMacroPing") == nil then
    function C_Ping.SendMacroPing()
        if Enum and Enum.PingResult and Enum.PingResult.FailedSilent then
            return Enum.PingResult.FailedSilent
        end
        return 0
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PING_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_empty_default_ping_options() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, String) = env
            .eval(
                r#"
                local options = C_Ping.GetDefaultPingOptions()
                return #options, type(C_Ping.SendMacroPing({ type = Enum.PingSubjectType.ActionReady }))
                "#,
            )
            .expect("default ping options should be queryable");

        assert_eq!(result, (0, "number".to_string()));
    }

    #[test]
    fn preserves_existing_ping_options_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_Ping.GetDefaultPingOptions()
                return { "existing" }
            end
            "#,
        )
        .expect("fixture should install existing ping provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let first_option: String = env
            .eval("local options = C_Ping.GetDefaultPingOptions(); return options[1]")
            .expect("existing ping provider should remain callable");

        assert_eq!(first_option, "existing");
    }
}
