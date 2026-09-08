local ADDON = ...
local VERSION, SCHEMA, MAX_CAPTURES, MAX_PARENTS = "1.0.0", 1, 30, 16

local paths = {
    "UIParent", "PlayerFrame", "PlayerFrame.noPortraitMode", "PlayerFrame.bbfName",
    "PlayerFrame.noPortraitMode.Texture",
    "PlayerFrame.PlayerFrameContent.PlayerFrameContentMain.HealthBarsContainer.HealthBar.BBFPixelBorder",
    "PlayerFrame.PlayerFrameContent.PlayerFrameContentMain.ManaBar.BBFPixelBorder",
    "PlayerSpellsFrame", "PlayerSpellsFrame.SpellBookFrame", "SpellBookFrame",
}
local methods = {
    "GetName", "GetObjectType", "GetFrameStrata", "GetFrameLevel", "GetRaisedFrameLevel",
    "HasFixedFrameStrata", "HasFixedFrameLevel", "IsToplevel", "IsShown", "IsVisible",
    "GetAlpha", "GetEffectiveAlpha", "GetScale", "GetEffectiveScale", "GetRect",
    "GetDrawLayer", "GetParent",
}
local addons = {
    "BetterBlizzFrames", "ClickableRaidBuffs", "DandersFrames", "BlizzMove",
    "EnhanceQoLMover", "Blizzard_PlayerSpells",
}

local function pack(...)
    return { n = select("#", ...), ... }
end

local function errorText(value)
    local ok, text = pcall(tostring, value)
    return ok and text or "unprintable error"
end

local function describe(value)
    local kind = type(value)
    if type(issecretvalue) == "function" then
        local ok, secret = pcall(issecretvalue, value)
        if not ok then return { kind = kind, status = "error", message = errorText(secret) } end
        if secret then return { kind = kind, status = "secret" } end
    end
    if kind == "nil" then return { kind = kind } end
    if kind == "string" or kind == "boolean" then return { kind = kind, value = value } end
    if kind == "number" then
        if value == value and value ~= math.huge and value ~= -math.huge then
            return { kind = kind, value = value }
        end
        return { kind = kind, status = "nonfinite", text = errorText(value) }
    end
    local ok, text = pcall(tostring, value)
    if not ok then return { kind = kind, status = "error", message = errorText(text) } end
    return { kind = kind, identity = text }
end

local function invoke(fn, ...)
    if type(fn) ~= "function" then return { status = "missing", kind = type(fn) } end
    local result = pack(pcall(fn, ...))
    if not result[1] then return { status = "error", message = errorText(result[2]) } end
    local values = { n = result.n - 1 }
    for i = 2, result.n do values[i - 1] = describe(result[i]) end
    return { status = "ok", values = values }, result[2]
end

local function resolve(path)
    local object = _G
    for key in path:gmatch("[^.]+") do
        if object == nil then return nil, { status = "missing", at = key } end
        local ok, value = pcall(function() return object[key] end)
        if not ok then return nil, { status = "error", message = errorText(value), at = key } end
        object = value
    end
    if object == nil then return nil, { status = "missing" } end
    return object, { status = "ok" }
end

local function callGlobal(path, ...)
    local fn, lookup = resolve(path)
    if lookup.status ~= "ok" then return lookup end
    return invoke(fn, ...)
end

local function callMethod(object, name, ...)
    local ok, fn = pcall(function() return object[name] end)
    if not ok then return { status = "error", message = errorText(fn), phase = "lookup" } end
    return invoke(fn, object, ...)
end

local function snapshot(object)
    local result = { status = "ok", object = describe(object), methods = {} }
    local parent
    for _, method in ipairs(methods) do
        local observation, value = callMethod(object, method)
        result.methods[method] = observation
        if method == "GetParent" then parent = value end
    end
    return result, parent
end

local function captureObject(path)
    local object, lookup = resolve(path)
    if lookup.status ~= "ok" then return lookup end
    local result, parent = snapshot(object)
    local chain = { entries = {} }
    result.parents = chain
    local seen, current = { [object] = true }, result
    while current.methods.GetParent.status == "ok" and parent ~= nil do
        if seen[parent] then chain.status = "cycle"; return result end
        if #chain.entries == MAX_PARENTS then chain.status = "limit"; return result end
        seen[parent] = true
        current, parent = snapshot(parent)
        chain.entries[#chain.entries + 1] = current
    end
    chain.status = current.methods.GetParent.status
    if chain.status == "ok" then chain.status = "root" end
    return result
end

local function database()
    if UnitFrameLayerProbeDB == nil then
        UnitFrameLayerProbeDB = {
            schemaVersion = SCHEMA, probeVersion = VERSION, captures = {},
            nextSequence = 1, skippedCaptures = 0,
        }
    end
    local db = UnitFrameLayerProbeDB
    if type(db) ~= "table" or db.schemaVersion ~= SCHEMA or type(db.captures) ~= "table"
        or type(db.nextSequence) ~= "number" or type(db.skippedCaptures) ~= "number" then
        print("[UnitFrameLayerProbe] Unsupported saved schema; capture skipped, saved data untouched.")
        return
    end
    return db
end

local function captureAddons()
    local result = {}
    for _, name in ipairs(addons) do
        local loaded = callGlobal("C_AddOns.IsAddOnLoaded", name)
        local version = callGlobal("C_AddOns.GetAddOnMetadata", name, "Version")
        result[name] = { loaded = loaded, version = version }
    end
    return result
end

local function captureConfig()
    local result = {}
    for _, key in ipairs({ "noPortraitModes", "noPortraitPixelBorder" }) do
        local value, lookup = resolve("BetterBlizzFramesDB." .. key)
        if lookup.status ~= "ok" then result[key] = lookup
        elseif type(value) == "boolean" then result[key] = { status = "ok", value = value }
        else result[key] = { status = "non_boolean", kind = type(value) } end
    end
    return result
end

local hookState = { status = "not_checked" }
local function capture(reason)
    local db = database()
    if not db then return end
    if #db.captures >= MAX_CAPTURES then
        db.skippedCaptures = db.skippedCaptures + 1
        print("[UnitFrameLayerProbe] Capture limit 30 reached; existing captures preserved.")
        return
    end
    local sample = {
        sequence = db.nextSequence, reason = reason, probeVersion = VERSION,
        timestamp = callGlobal("GetServerTime"), uptime = callGlobal("GetTime"),
        build = callGlobal("GetBuildInfo"), metrics = {}, frames = {},
        addons = captureAddons(), noPortraitConfig = captureConfig(), hooks = hookState,
    }
    for _, name in ipairs({ "GetPhysicalScreenSize", "GetScreenWidth", "GetScreenHeight" }) do
        sample.metrics[name] = callGlobal(name)
    end
    for _, path in ipairs(paths) do sample.frames[path] = captureObject(path) end
    db.captures[#db.captures + 1] = sample
    db.nextSequence = db.nextSequence + 1
    print("[UnitFrameLayerProbe] Captured #" .. sample.sequence .. " " .. reason .. "; /reload to save.")
end

local function deferredCapture(reason)
    local result = callGlobal("C_Timer.After", 0, function() capture(reason) end)
    if result.status ~= "ok" then
        print("[UnitFrameLayerProbe] Deferred capture failed: " .. result.status .. " " .. (result.message or ""))
    end
end

local hooked = setmetatable({}, { __mode = "k" })
local function installBookHooks()
    local book, lookup = resolve("PlayerSpellsFrame.SpellBookFrame")
    if lookup.status ~= "ok" then hookState = lookup; return end
    local state = hooked[book] or {}
    for _, event in ipairs({ "OnShow", "OnHide" }) do
        if not state[event] or state[event].status ~= "ok" then
            state[event] = callMethod(book, "HookScript", event, function()
                deferredCapture("SpellBook." .. event)
            end)
        end
    end
    hooked[book] = state
    hookState = { object = describe(book), OnShow = state.OnShow, OnHide = state.OnHide }
end

local function checkLoadedBook()
    local observation, loaded = callGlobal("C_AddOns.IsAddOnLoaded", "Blizzard_PlayerSpells")
    if observation.status == "ok" and loaded == true then installBookHooks()
    else hookState = { loaded = observation } end
end

SLASH_UNITFRAMELAYERPROBE1 = "/unitlayerprobe"
SlashCmdList.UNITFRAMELAYERPROBE = function() capture("manual") end
local listener = CreateFrame("Frame")
listener:RegisterEvent("ADDON_LOADED")
listener:RegisterEvent("PLAYER_LOGIN")
listener:SetScript("OnEvent", function(_, event, name)
    if event == "PLAYER_LOGIN" then
        checkLoadedBook()
        deferredCapture("PLAYER_LOGIN")
    elseif name == "Blizzard_PlayerSpells" then
        installBookHooks()
    elseif name == ADDON then
        checkLoadedBook()
    end
end)
checkLoadedBook()
