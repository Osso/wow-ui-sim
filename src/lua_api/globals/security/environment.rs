//! Real secure/global environment helpers exposed by the WoW client.

use rilua::{LuaApiMut, LuaResult, Val, vm::state::LuaState};

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

if SwapToGlobalEnvironment == nil then
  function SwapToGlobalEnvironment()
    setfenv(2, _G)
  end
end
"#;

pub(crate) fn register_environment_helpers(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "IsInGlobalEnvironment", is_in_global_environment)?;
    lua.exec(ENVIRONMENT_HELPERS_LUA)
}

fn is_in_global_environment(state: &mut LuaState) -> LuaResult<u32> {
    // A Rust primitive observes the caller before Lua tail-call frame replacement.
    let caller_environment = state.call_stack[..=state.ci]
        .iter()
        .rev()
        .find_map(|frame| {
            let Val::Function(function) = state.stack_get(frame.func) else {
                return None;
            };
            match state.gc.closures.get(function) {
                Some(rilua::vm::closure::Closure::Lua(closure)) => Some(closure.env),
                _ => None,
            }
        });
    state.push(Val::Bool(caller_environment == Some(state.global)));
    Ok(1)
}

#[cfg(test)]
mod tests {
    const ENVIRONMENT_SWAP_SETUP: &str = r##"
            local globalEnv = GetGlobalEnvironment()
            globalEnv.CapturedSecureEnv = GetCurrentEnvironment()
            globalEnv.BeforeSwapWasSecure = globalEnv.CapturedSecureEnv ~= globalEnv
            SwapToGlobalEnvironmentReturnCount = select("#", SwapToGlobalEnvironment())
            AfterSwapIsGlobal = GetCurrentEnvironment() == GetGlobalEnvironment()
            SwapProbeValue = "global-after-swap"
            "##;
    const ENVIRONMENT_SWAP_PROBE: &str = r#"
                return BeforeSwapWasSecure,
                       AfterSwapIsGlobal,
                       rawget(_G, "SwapToGlobalEnvironmentReturnCount"),
                       tostring(rawget(_G, "SwapProbeValue")),
                       tostring(rawget(CapturedSecureEnv, "SwapProbeValue"))
                "#;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn environment_helpers_track_and_swap_caller_environment() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec_rilua_secure(ENVIRONMENT_SWAP_SETUP)
            .expect("secure chunk should execute");
        let (before_was_secure, after_is_global, return_count, global_value, secure_value): (
            bool,
            bool,
            i64,
            String,
            String,
        ) = env
            .eval(ENVIRONMENT_SWAP_PROBE)
            .expect("probe result should evaluate");
        assert!(before_was_secure);
        assert!(after_is_global);
        assert_eq!(return_count, 0);
        assert_eq!(global_value, "global-after-swap");
        assert_eq!(secure_value, "nil");
    }
}
