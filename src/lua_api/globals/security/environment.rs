//! Real secure/global environment helpers exposed by the WoW client.

const ENVIRONMENT_HELPERS_LUA: &str = r#"
if GetGlobalEnvironment == nil then
  function GetGlobalEnvironment()
    return _G
  end
end

if GetCurrentEnvironment == nil then
  function GetCurrentEnvironment()
    return getfenv(2)
  end
end

if IsInGlobalEnvironment == nil then
  function IsInGlobalEnvironment()
    return getfenv(2) == _G
  end
end

if SwapToGlobalEnvironment == nil then
  function SwapToGlobalEnvironment()
    setfenv(2, _G)
    return _G
  end
end
"#;

pub(crate) fn register_environment_helpers(lua: &mut rilua::Lua) -> rilua::LuaResult<()> {
    lua.exec(ENVIRONMENT_HELPERS_LUA)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn environment_helpers_track_and_swap_caller_environment() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        env.exec_rilua_secure(
            r#"
            local globalEnv = GetGlobalEnvironment()
            globalEnv.CapturedSecureEnv = GetCurrentEnvironment()
            globalEnv.BeforeSwapWasSecure = globalEnv.CapturedSecureEnv ~= globalEnv
            SwapToGlobalEnvironment()
            AfterSwapIsGlobal = GetCurrentEnvironment() == GetGlobalEnvironment()
            SwapProbeValue = "global-after-swap"
            "#,
        )
        .expect("secure chunk should execute");

        let (before_was_secure, after_is_global, global_value, secure_value): (
            bool,
            bool,
            String,
            String,
        ) = env
            .eval(
                r#"
                return BeforeSwapWasSecure,
                       AfterSwapIsGlobal,
                       tostring(rawget(_G, "SwapProbeValue")),
                       tostring(rawget(CapturedSecureEnv, "SwapProbeValue"))
                "#,
            )
            .expect("probe result should evaluate");

        assert!(before_was_secure);
        assert!(after_is_global);
        assert_eq!(global_value, "global-after-swap");
        assert_eq!(secure_value, "nil");
    }
}
