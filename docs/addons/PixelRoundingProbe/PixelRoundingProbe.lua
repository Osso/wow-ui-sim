local addonName = ...
local EXPECTED_BUILD = "69594"
local CAPTURE_DELAY = 0

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
    point.relativeTo = value(point, "relativeTo", relative and relative.GetName, relative)
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

local function captureCase(sample, label, setup)
    local case = { label = label, errors = {} }
    sample.cases[#sample.cases + 1] = case

    local parent = CreateFrame("Frame", nil, UIParent)
    parent:Hide()
    parent:SetSize(301.25, 179.75)
    parent:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 123.375, 87.625)

    local frame = CreateFrame("Frame", nil, parent)
    frame:Hide()
    local texture = frame:CreateTexture(nil, "ARTWORK")
    local fontString = frame:CreateFontString(nil, "OVERLAY", "GameFontNormal")

    case.before = captureObject(frame, "frame")
    setup(case, parent, frame, texture, fontString)
    case.after = captureObject(frame, "frame")
    case.texture = captureObject(texture, "texture")
    case.fontString = captureObject(fontString, "fontString")
    if next(case.errors) == nil then
        case.errors = nil
    end
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
    end)
    captureCase(sample, "stretch-two-anchors", function(case, _, frame)
        configurePoint(case, frame, "BOTTOMLEFT", "BOTTOMLEFT", 0.375, -0.625)
        configurePoint(case, frame, "TOPRIGHT", "TOPRIGHT", -0.875, 0.125)
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
        setRoundLayout(case, texture, true)
        setRoundLayout(case, fontString, true)
    end)
end

local function capture(label)
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
    sample.physicalScreen = {
        width = value(sample, "physicalWidth", GetPhysicalScreenWidth),
        height = value(sample, "physicalHeight", GetPhysicalScreenHeight),
    }
    sample.uiParent = captureObject(UIParent, "frame")
    captureCases(sample)
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
            capture(label .. "+settled")
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
