-- Runs the selected original Audio.lua polling callback unhooked, so the real
-- InitVoiceSettings reaches the device catalogs. No vendor callee is replaced.
function RunOriginalVoiceInitialization()
    local ok, err = pcall(OriginalAudioCallback)
    if not ok then error(err, 0) end
    return true
end

local function FindInitializer(setting)
    local name, category, layout
    for i = 1, 20 do
        name, layout = debug.getupvalue(OriginalAudioCallback, i)
        if not name then break end
        if name == 'layout' then break end
    end
    assert(name == 'layout' and layout, 'original Audio layout upvalue')
    for _, initializer in ipairs(layout:GetInitializers()) do
        if initializer.GetSetting and initializer:GetSetting() == setting then
            return initializer
        end
    end
    error('device dropdown initializer missing')
end

-- Returns the original setting's live value, its init-time default and the
-- labels/values produced by the original dropdown option builder.
function ObserveOriginalDeviceSetting(kind)
    local setting = Settings.GetSetting('PROXY_VOICE_' .. kind .. '_DEVICE')
    assert(setting, 'original device setting registered')
    local options = FindInitializer(setting):GetOptions()()
    local rendered = {}
    for _, option in ipairs(options) do
        rendered[#rendered + 1] = tostring(option.value) .. '=' .. tostring(option.label)
    end
    return setting:GetValue(), setting:GetDefaultValue(), table.concat(rendered, ';')
end
