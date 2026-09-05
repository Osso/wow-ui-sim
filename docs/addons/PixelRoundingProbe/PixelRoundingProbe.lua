local addonName = ...
local EXPECTED_BUILD = "69594"
local CAPTURE_DELAY = 0
local contexts = {}
local caseOrder = {}
local objectLabels = { [UIParent] = "UIParent" }

PixelRoundingProbeDB = {
    addon = addonName,
    expectedBuild = EXPECTED_BUILD,
    samples = {},
}

local function recordError(record, key, errorMessage)
    record.errors = record.errors or {}
    record.errors[key] = tostring(errorMessage)
end

local function value(record, key, fn, ...)
    local ok, result = pcall(fn, ...)
    if not ok then
        recordError(record, key, result)
        return nil
    end
    if issecretvalue and issecretvalue(result) then
        recordError(record, key, "secret value")
        return nil
    end
    local kind = type(result)
    if result == nil or kind == "string" or kind == "number" or kind == "boolean" then
        return result
    end
    recordError(record, key, "non-scalar " .. kind)
    return nil
end

local function method(record, key, object, name, ...)
    local fn = object and object[name]
    if type(fn) ~= "function" then
        recordError(record, key, name .. " is unavailable")
        return nil
    end
    return value(record, key, fn, object, ...)
end

local function capturePoint(record, object, index)
    local point = { index = index }
    local ok, anchor, relative, relativePoint, x, y = pcall(object.GetPoint, object, index)
    if not ok then
        recordError(point, "getPoint", anchor)
        return point
    end
    point.point = value(point, "point", function() return anchor end)
    point.relativeTo = value(point, "relativeTo", function()
        return relative and (objectLabels[relative] or relative:GetName())
    end)
    point.relativePoint = value(point, "relativePoint", function() return relativePoint end)
    point.x = value(point, "x", function() return x end)
    point.y = value(point, "y", function() return y end)
    return point
end

local function captureObject(object, kind)
    local record = { kind = kind, errors = {} }
    record.roundLayout = method(record, "roundLayout", object, "GetRoundLayoutToNearestPixel")
    record.scale = method(record, "scale", object, "GetScale")
    record.effectiveScale = method(record, "effectiveScale", object, "GetEffectiveScale")
    record.width = method(record, "width", object, "GetWidth")
    record.height = method(record, "height", object, "GetHeight")
    local sizeOK, sizeWidth, sizeHeight = pcall(object.GetSize, object)
    if sizeOK then
        record.size = {
            width = value(record, "sizeWidth", function() return sizeWidth end),
            height = value(record, "sizeHeight", function() return sizeHeight end),
        }
    else
        recordError(record, "getSize", sizeWidth)
    end
    local ok, left, bottom, width, height = pcall(object.GetRect, object)
    if ok then
        record.rect = {
            left = value(record, "left", function() return left end),
            bottom = value(record, "bottom", function() return bottom end),
            width = value(record, "width", function() return width end),
            height = value(record, "height", function() return height end),
        }
    else
        recordError(record, "getRect", left)
    end
    local pointCount = method(record, "pointCount", object, "GetNumPoints")
    if type(pointCount) == "number" then
        record.points = {}
        for index = 1, pointCount do
            record.points[index] = capturePoint(record, object, index)
        end
    end
    if next(record.errors) == nil then
        record.errors = nil
    end
    return record
end

local function setRoundLayout(record, object, enabled)
    local fn = object and object.SetRoundLayoutToNearestPixel
    if type(fn) ~= "function" then
        recordError(record, "setRoundLayout", "SetRoundLayoutToNearestPixel is unavailable")
        return
    end
    local ok, err = pcall(fn, object, enabled)
    if not ok then
        recordError(record, "setRoundLayout", err)
    end
end

local function contextForCase(label)
    if contexts[label] then
        return contexts[label]
    end
    local parent = CreateFrame("Frame", nil, UIParent)
    local frame = CreateFrame("Frame", nil, parent)
    parent:Hide()
    frame:Hide()
    local context = {
        parent = parent,
        frame = frame,
        texture = frame:CreateTexture(nil, "ARTWORK"),
        fontString = frame:CreateFontString(nil, "OVERLAY", "GameFontNormal"),
        defaultFlags = {},
        defaultQueries = {},
    }
    for _, kind in ipairs({ "parent", "frame", "texture", "fontString" }) do
        local object = context[kind]
        objectLabels[object] = label .. ":" .. kind
        context.defaultFlags[kind] = method(context.defaultQueries, kind, object, "GetRoundLayoutToNearestPixel")
    end
    contexts[label] = context
    caseOrder[#caseOrder + 1] = label
    return context
end

local function resetContext(case, context)
    for _, kind in ipairs({ "parent", "frame", "texture", "fontString" }) do
        local object = context[kind]
        if context.defaultFlags[kind] ~= nil then
            setRoundLayout(case, object, context.defaultFlags[kind])
        end
        method(case, kind .. "ClearPoints", object, "ClearAllPoints")
        method(case, kind .. "ClearSize", object, "SetSize", 0, 0)
    end
    method(case, "parentScale", context.parent, "SetScale", 1)
    method(case, "frameScale", context.frame, "SetScale", 1)
    method(case, "parentSize", context.parent, "SetSize", 301.25, 179.75)
    method(case, "parentPoint", context.parent, "SetPoint", "BOTTOMLEFT", UIParent, "BOTTOMLEFT", 123.375, 87.625)
end

local function captureCaseAfter(case, context)
    case.after = captureObject(context.frame, "frame")
    case.parent = captureObject(context.parent, "parent")
    case.texture = captureObject(context.texture, "texture")
    case.fontString = captureObject(context.fontString, "fontString")
    if next(case.errors) == nil then
        case.errors = nil
    end
end

local function captureCase(sample, label, setup)
    local case = { label = label, id = label, errors = {} }
    sample.cases[#sample.cases + 1] = case
    local context = contextForCase(label)
    resetContext(case, context)
    case.defaultQueries = context.defaultQueries
    case.before = captureObject(context.frame, "frame")
    case.parentBefore = captureObject(context.parent, "parent")
    local ok, err = pcall(setup, case, context.parent, context.frame, context.texture, context.fontString)
    if not ok then
        recordError(case, "setup", err)
    end
    captureCaseAfter(case, context)
end

local function configurePoint(case, frame, point, relativePoint, x, y)
    local ok, err = pcall(frame.SetPoint, frame, point, frame:GetParent(), relativePoint, x, y)
    if not ok then
        recordError(case, "setPoint", err)
    end
end

local function configureSize(case, frame, width, height)
    local ok, err = pcall(frame.SetSize, frame, width, height)
    if not ok then
        recordError(case, "setSize", err)
    end
end

local function captureCases(sample)
    captureCase(sample, "default", function() end)
    captureCase(sample, "bottomleft-fractional", function(case, _, frame)
        configureSize(case, frame, 101.375, 40.625)
        configurePoint(case, frame, "BOTTOMLEFT", "BOTTOMLEFT", 0.375, -0.625)
    end)
    captureCase(sample, "center-negative", function(case, _, frame)
        configureSize(case, frame, 101.375, 40.625)
        configurePoint(case, frame, "CENTER", "CENTER", -0.375, 0.625)
        case.beforeFlag = captureObject(frame, "frame")
        setRoundLayout(case, frame, true)
    end)
    captureCase(sample, "stretch-two-anchors", function(case, _, frame)
        configurePoint(case, frame, "BOTTOMLEFT", "BOTTOMLEFT", 0.375, -0.625)
        configurePoint(case, frame, "TOPRIGHT", "TOPRIGHT", -0.875, 0.125)
        case.beforeFlag = captureObject(frame, "frame")
        setRoundLayout(case, frame, true)
    end)
    for _, scale in ipairs({ 0.8, 1, 1.25 }) do
        captureCase(sample, "own-scale-" .. tostring(scale), function(case, _, frame)
            frame:SetScale(scale)
            configureSize(case, frame, 101.375, 40.625)
            configurePoint(case, frame, "BOTTOMLEFT", "BOTTOMLEFT", 0.375, -0.625)
            setRoundLayout(case, frame, true)
        end)
    end
    captureCase(sample, "round-before-size-and-point", function(case, _, frame)
        setRoundLayout(case, frame, true)
        case.afterFlag = captureObject(frame, "frame")
        configureSize(case, frame, 101.375, 40.625)
        configurePoint(case, frame, "BOTTOMLEFT", "BOTTOMLEFT", 0.375, -0.625)
    end)
    captureCase(sample, "round-after-size-and-point", function(case, _, frame)
        configureSize(case, frame, 101.375, 40.625)
        configurePoint(case, frame, "BOTTOMLEFT", "BOTTOMLEFT", 0.375, -0.625)
        case.beforeFlag = captureObject(frame, "frame")
        setRoundLayout(case, frame, true)
    end)
    captureCase(sample, "round-toggle-off", function(case, _, frame)
        configureSize(case, frame, 101.375, 40.625)
        configurePoint(case, frame, "CENTER", "CENTER", -0.375, 0.625)
        setRoundLayout(case, frame, true)
        case.enabled = captureObject(frame, "frame")
        setRoundLayout(case, frame, false)
    end)
    captureCase(sample, "parent-scale-and-reposition", function(case, parent, frame)
        configureSize(case, frame, 101.375, 40.625)
        configurePoint(case, frame, "BOTTOMLEFT", "BOTTOMLEFT", 0.375, -0.625)
        setRoundLayout(case, frame, true)
        parent:SetScale(1.25)
        parent:ClearAllPoints()
        parent:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 123.875, 87.125)
        case.parentAfter = captureObject(parent, "parent")
    end)
    captureCase(sample, "regions-round-layout", function(case, _, frame, texture, fontString)
        configureSize(case, frame, 101.375, 40.625)
        configurePoint(case, frame, "BOTTOMLEFT", "BOTTOMLEFT", 0.375, -0.625)
        texture:SetAllPoints(frame)
        fontString:SetPoint("CENTER", frame, "CENTER", -0.375, 0.625)
        fontString:SetSize(33.375, 14.625)
        fontString:SetText("Probe")
        setRoundLayout(case, texture, true)
        setRoundLayout(case, fontString, true)
    end)
end

local function captureBootstrapState()
    local record = {
        clickBindingModeType = type(InClickBindingMode),
        collectionsToggleType = type(ToggleCollectionsJournal),
    }
    for _, name in ipairs({ "Blizzard_ClickBindingUI", "Blizzard_Collections" }) do
        local addon = {}
        local ok, loaded, finished = pcall(C_AddOns.IsAddOnLoaded, name)
        if ok then
            addon.loaded = value(addon, "loaded", function() return loaded end)
            addon.finished = value(addon, "finished", function() return finished end)
        else
            recordError(addon, "loadState", loaded)
        end
        record[name] = addon
    end
    return record
end

local function capture(label, settled)
    local sample = { label = label, errors = {}, cases = {} }
    PixelRoundingProbeDB.samples[#PixelRoundingProbeDB.samples + 1] = sample
    local version, build, date, interface = GetBuildInfo()
    sample.build = {
        version = value(sample, "version", function() return version end),
        build = value(sample, "build", function() return build end),
        date = value(sample, "date", function() return date end),
        interface = value(sample, "interface", function() return interface end),
        expectedBuild = EXPECTED_BUILD,
        matchesExpectedBuild = build == EXPECTED_BUILD,
    }
    sample.physicalScreen = {}
    local physicalOK, physicalWidth, physicalHeight = pcall(GetPhysicalScreenSize)
    if physicalOK then
        sample.physicalScreen.width = value(sample, "physicalWidth", function() return physicalWidth end)
        sample.physicalScreen.height = value(sample, "physicalHeight", function() return physicalHeight end)
    else
        recordError(sample, "physicalScreenSize", physicalWidth)
    end
    sample.uiParent = captureObject(UIParent, "frame")
    sample.bootstrap = captureBootstrapState()
    if settled then
        for _, caseLabel in ipairs(caseOrder) do
            local case = { label = caseLabel, id = caseLabel, errors = {} }
            sample.cases[#sample.cases + 1] = case
            captureCaseAfter(case, contexts[caseLabel])
        end
    else
        captureCases(sample)
    end
    if next(sample.errors) == nil then
        sample.errors = nil
    end
    print(addonName .. " captured " .. label .. " for build " .. tostring(build))
end

local function scheduleCapture(label)
    if InCombatLockdown and InCombatLockdown() then
        PixelRoundingProbeDB.skipped = { label = label, reason = "in combat" }
        print(addonName .. " skipped capture: in combat")
        return
    end
    C_Timer.After(CAPTURE_DELAY, function()
        capture(label .. "+next-tick")
        C_Timer.After(CAPTURE_DELAY, function()
            capture(label .. "+settled", true)
        end)
    end)
end

SLASH_PIXELROUNDINGPROBE1 = "/pixelprobe"
SlashCmdList.PIXELROUNDINGPROBE = function(label)
    scheduleCapture(label ~= "" and label or "manual")
end

local eventFrame = CreateFrame("Frame")
eventFrame:RegisterEvent("PLAYER_ENTERING_WORLD")
eventFrame:SetScript("OnEvent", function()
    scheduleCapture("world")
end)
