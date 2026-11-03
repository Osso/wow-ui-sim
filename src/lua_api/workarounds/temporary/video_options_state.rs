//! Temporary `C_VideoOptions` state surface.
//!
//! Video option queries are currently a lightweight simulator fixture. Keep the
//! mutable Lua-visible state explicit as temporary compatibility behavior until
//! display configuration is backed by a real subsystem.

const VIDEO_OPTIONS_STATE_LUA: &str = r#"
if type(C_VideoOptions) ~= "table" then
    C_VideoOptions = {}
end

local state = rawget(_G, "__wow_video_options_state")
if type(state) ~= "table" then
    state = {
        defaultGameWindowSize = { x = 1920, y = 1080 },
        currentGameWindowSize = { x = 1920, y = 1080 },
        availableGameWindowSizes = {},
        gxAdapterInfo = {},
        setGameWindowSizeCount = 0,
        lastSetWindowSize = nil,
    }
    rawset(_G, "__wow_video_options_state", state)
end

local function CopyTable(source)
    local copied = {}
    if type(source) ~= "table" then
        return copied
    end
    for key, value in pairs(source) do
        copied[key] = value
    end
    return copied
end

local function VideoOptionsState()
    if type(state.defaultGameWindowSize) ~= "table" then
        state.defaultGameWindowSize = { x = 1920, y = 1080 }
    end
    if type(state.currentGameWindowSize) ~= "table" then
        state.currentGameWindowSize = CopyTable(state.defaultGameWindowSize)
    end
    if type(state.availableGameWindowSizes) ~= "table" then
        state.availableGameWindowSizes = {}
    end
    if type(state.gxAdapterInfo) ~= "table" then
        state.gxAdapterInfo = {}
    end
    return state
end

local function CopyWindowSize(size)
    if type(size) ~= "table" then
        return { x = 0, y = 0 }
    end
    return {
        x = tonumber(size.x) or 0,
        y = tonumber(size.y) or 0,
    }
end

local function CopyWindowSizes(sizes)
    local copied = {}
    if type(sizes) ~= "table" then
        return copied
    end
    for index, size in ipairs(sizes) do
        copied[index] = CopyWindowSize(size)
    end
    return copied
end

local function CopyAdapterInfo(adapters)
    local copied = {}
    if type(adapters) ~= "table" then
        return copied
    end
    for index, adapter in ipairs(adapters) do
        copied[index] = CopyTable(adapter)
    end
    return copied
end

C_VideoOptions._state = VideoOptionsState()

if rawget(C_VideoOptions, "GetDefaultGameWindowSize") == nil then
    function C_VideoOptions.GetDefaultGameWindowSize()
        return CopyWindowSize(VideoOptionsState().defaultGameWindowSize)
    end
end

if rawget(C_VideoOptions, "GetCurrentGameWindowSize") == nil then
    function C_VideoOptions.GetCurrentGameWindowSize()
        return CopyWindowSize(VideoOptionsState().currentGameWindowSize)
    end
end

if rawget(C_VideoOptions, "GetGameWindowSizes") == nil then
    function C_VideoOptions.GetGameWindowSizes()
        return CopyWindowSizes(VideoOptionsState().availableGameWindowSizes)
    end
end

if rawget(C_VideoOptions, "GetGxAdapterInfo") == nil then
    function C_VideoOptions.GetGxAdapterInfo()
        return CopyAdapterInfo(VideoOptionsState().gxAdapterInfo)
    end
end

if rawget(C_VideoOptions, "IsSpellVisualDensitySystemSupported") == nil then
    function C_VideoOptions.IsSpellVisualDensitySystemSupported()
        return false
    end
end

if rawget(C_VideoOptions, "SetGameWindowSize") == nil then
    function C_VideoOptions.SetGameWindowSize(width, height)
        local currentState = VideoOptionsState()
        currentState.currentGameWindowSize = {
            x = tonumber(width) or 0,
            y = tonumber(height) or 0,
        }
        currentState.lastSetWindowSize = CopyWindowSize(currentState.currentGameWindowSize)
        currentState.setGameWindowSizeCount = (tonumber(currentState.setGameWindowSizeCount) or 0) + 1
        return true
    end
end
"#;

// User-approved offline capability, not a native platform measurement. Replace
// this fixture flag with the optional texture-pack/launcher capability model.
const FOREVER_VIDEO_CAPABILITIES_LUA: &str = r#"
local state = C_VideoOptions._state
if state.highResTexturesAvailable == nil then
    state.highResTexturesAvailable = false
end

if rawget(C_VideoOptions, "AreHighResTexturesAvailable") == nil then
    function C_VideoOptions.AreHighResTexturesAvailable()
        return state.highResTexturesAvailable == true
    end
end

-- The UI renderer uses single-sample targets. Replace this unsupported capability
-- when multisample render targets and MSAA mode enumeration are implemented.
if rawget(_G, "MultiSampleAntiAliasingSupported") == nil then
    function MultiSampleAntiAliasingSupported()
        return false
    end
end

-- The simulator renders 2D UI, without a 3D world renderer. These unsupported
-- water, object-outline and secondary-lighting capabilities are not host GPU probes.
-- Native world linear-lighting/Bloom startup mode is also absent; this does not
-- describe the color space used to composite the simulator's 2D UI.
for _, method in ipairs({
    "IsPBRWaterSupported",
    "IsOutlineModeSupported",
    "IsSecondaryLightingSupported",
    "IsLinearEnabledOnStart",
}) do
    if rawget(C_VideoOptions, method) == nil then
        C_VideoOptions[method] = function() return false end
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(VIDEO_OPTIONS_STATE_LUA)?;
    if cfg!(feature = "client-wowforever") {
        lua.exec(FOREVER_VIDEO_CAPABILITIES_LUA)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_window_size_state_and_returns_copies() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local defaultSize = C_VideoOptions.GetDefaultGameWindowSize()
                if defaultSize.x ~= 1920 or defaultSize.y ~= 1080 then
                    return "bad_default"
                end
                C_VideoOptions.SetGameWindowSize(1280, 720)
                local currentSize = C_VideoOptions.GetCurrentGameWindowSize()
                if currentSize.x ~= 1280 or currentSize.y ~= 720 then
                    return "bad_current"
                end
                currentSize.x = 1
                if C_VideoOptions.GetCurrentGameWindowSize().x ~= 1280 then
                    return "leaked_current_copy"
                end
                C_VideoOptions._state.availableGameWindowSizes = {
                    { x = 800, y = 600 },
                }
                local sizes = C_VideoOptions.GetGameWindowSizes()
                sizes[1].x = 1
                if C_VideoOptions.GetGameWindowSizes()[1].x ~= 800 then
                    return "leaked_sizes_copy"
                end
                C_VideoOptions._state.gxAdapterInfo = {
                    { name = "adapter", index = 1 },
                }
                local adapters = C_VideoOptions.GetGxAdapterInfo()
                adapters[1].name = "mutated"
                if C_VideoOptions.GetGxAdapterInfo()[1].name ~= "adapter" then
                    return "leaked_adapter_copy"
                end
                if C_VideoOptions.IsSpellVisualDensitySystemSupported() then
                    return "bad_density"
                end
                if C_VideoOptions._state.setGameWindowSizeCount ~= 1 then
                    return "bad_counter"
                end
                return "ok"
                "#,
            )
            .expect("video options probe should run");

        assert_eq!(result, "ok");
    }
}
