local suffix = 'Blizzard_SettingsDefinitions_Shared/Audio.lua'
local function IsOriginalAudio(func)
    local info = debug.getinfo(func, 'S')
    local path = info.source:gsub('\\', '/')
    return info.what == 'Lua' and path:sub(-#suffix) == suffix
end

local register
for _, candidate in ipairs(SettingsRegistrar.registrants) do
    if IsOriginalAudio(candidate) then
        assert(not register, 'one selected original Audio registrant')
        register = candidate
    end
end
assert(register, 'original Audio registrant was not selected')
register()

local function FindOriginalCallback(func, seen)
    if type(func) ~= 'function' or seen[func] then return end
    seen[func] = true
    if IsOriginalAudio(func) then return func end
    local i = 1
    while true do
        local name, value = debug.getupvalue(func, i)
        if not name then return end
        local found = FindOriginalCallback(value, seen)
        if found then return found end
        i = i + 1
    end
end

local callback
for _, stored in pairs(debug.getregistry().__rilua_timer_callbacks) do
    local original = FindOriginalCallback(stored, {})
    if original then
        assert(not callback, 'one original Audio ticker')
        callback = original
    end
end
assert(callback, 'original Audio polling callback missing')
OriginalAudioCallback = callback
local init
for i = 1, 20 do
    local name, value = debug.getupvalue(callback, i)
    if not name then break end
    if name == 'InitVoiceSettings' then init = value end
end
assert(init and IsOriginalAudio(init), 'original initialization closure')
local info = debug.getinfo(callback, 'S')
print('voice settings original callback', info.source, info.linedefined)

function ObserveOriginalVoiceSettings()
    local entered = false
    -- Observe entry to the original initialization branch. Stop before device
    -- catalogs/settings APIs, which are separate contracts. No callee is replaced.
    debug.sethook(function()
        local caller = debug.getinfo(2, 'f')
        if caller.func == init then
            entered = true
            debug.sethook()
            error('voice-settings-init-observed')
        end
    end, 'c')
    local ok, err = pcall(callback)
    debug.sethook()
    if entered then
        assert(not ok and tostring(err):find('voice-settings-init-observed', 1, true))
        return true
    end
    if not ok then error(err) end
    return false
end
