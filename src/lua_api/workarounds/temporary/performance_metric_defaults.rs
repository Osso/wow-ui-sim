//! Temporary framerate and performance metric globals.
//!
//! The simulator does not model addon CPU/memory accounting yet. These globals
//! are safe startup defaults for Blizzard performance UI callers.

const PERFORMANCE_METRIC_DEFAULTS_LUA: &str = r#"
if GetFramerate == nil then
  function GetFramerate()
    return 60
  end
end

if UpdateAddOnMemoryUsage == nil then
  function UpdateAddOnMemoryUsage()
    return 0
  end
end

if UpdateAddOnCPUUsage == nil then
  function UpdateAddOnCPUUsage()
    return 0
  end
end

if ResetCPUUsage == nil then
  function ResetCPUUsage()
    return 0
  end
end

if GetAddOnMemoryUsage == nil then
  function GetAddOnMemoryUsage(_name)
    return 0
  end
end

if GetAddOnCPUUsage == nil then
  function GetAddOnCPUUsage(_name)
    return 0
  end
end

if GetFrameCPUUsage == nil then
  function GetFrameCPUUsage(_frame, _includeChildren)
    return 0, 0
  end
end


if GetDownloadedPercentage == nil then
  function GetDownloadedPercentage()
    return 1
  end
end

if GetMovieDownloadProgress == nil then
  function GetMovieDownloadProgress(_movieID)
    return false, 0, 0
  end
end

MAINMENUBAR_COMMUNICATION_PROTOCOL_LABEL = MAINMENUBAR_COMMUNICATION_PROTOCOL_LABEL or "Protocol: %s (home) %s (world)"

if GetProtocolTypes == nil then
  function GetProtocolTypes()
    return 1, 1
  end
end
"#;

#[cfg(not(feature = "retail-12-0-7"))]
const LEGACY_PROFILING_DEFAULTS_LUA: &str = r#"
if GetEventCPUUsage == nil then
  function GetEventCPUUsage(_event)
    return 0
  end
end

if GetFunctionCPUUsage == nil then
  function GetFunctionCPUUsage(_fn, _includeSubroutines)
    return 0, 0
  end
end

if GetScriptCPUUsage == nil then
  function GetScriptCPUUsage(_frame, _script, _includeChildren)
    return 0, 0
  end
end

"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PERFORMANCE_METRIC_DEFAULTS_LUA)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    lua.exec(LEGACY_PROFILING_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_performance_metric_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let script_arity = if cfg!(feature = "retail-12-0-7") {
            1
        } else {
            2
        };
        env.exec(&format!("ExpectedScriptArity = {script_arity}"))
            .expect("set profile-specific script metric arity");

        let result: String = env
            .eval(
                r#"
                if GetFramerate() ~= 60 then return "framerate" end
                if UpdateAddOnMemoryUsage() ~= 0 then return "update_memory" end
                if UpdateAddOnCPUUsage() ~= 0 then return "update_cpu" end
                if ResetCPUUsage() ~= 0 then return "reset_cpu" end
                if GetAddOnMemoryUsage("missing") ~= 0 then return "addon_memory" end
                if GetAddOnCPUUsage("missing") ~= 0 then return "addon_cpu" end
                local frame, children = GetFrameCPUUsage(UIParent, true)
                if frame ~= 0 or children ~= 0 then return "frame_cpu" end
                if GetEventCPUUsage("PLAYER_LOGIN") ~= 0 then return "event_cpu" end
                local fnUsage, fnSubroutines = GetFunctionCPUUsage(function() end, true)
                if fnUsage ~= 0 or fnSubroutines ~= 0 then return "function_cpu" end
                local scriptUsage, scriptChildren = GetScriptCPUUsage(UIParent, "OnShow", true)
                if scriptUsage ~= 0 or select('#', GetScriptCPUUsage(UIParent, "OnShow", true)) ~= ExpectedScriptArity then return "script_cpu" end
                if GetDownloadedPercentage() ~= 1 then return "downloaded" end
                local inProgress, downloaded, total = GetMovieDownloadProgress(1)
                if inProgress or downloaded ~= 0 or total ~= 0 then return "movie_download" end
                local protocolHome, protocolWorld = GetProtocolTypes()
                if protocolHome ~= 1 or protocolWorld ~= 1 then return "protocol_types" end
                if type(MAINMENUBAR_COMMUNICATION_PROTOCOL_LABEL) ~= "string" then return "protocol_label" end
                return "ok"
                "#,
            )
            .expect("performance metric defaults probe should run");

        assert_eq!(result, "ok");
    }
}
