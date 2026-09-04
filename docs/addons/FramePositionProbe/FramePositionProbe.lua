local addonName = ...

local frameNames = {
    "UIParent",
    "RaidWarningFrame",
    "PrivateRaidBossEmoteFrameAnchor",
    "DeadlyDebuffFrame",
    "RightManagedFrameContainer",
    "UIParentRightManagedFrameContainer",
    "ObjectiveTrackerFrame",
}

local function recordError(target, key, value)
    local ok, secret = pcall(issecretvalue, value)
    target.errors[key] = ok and not secret and tostring(value) or "<unreadable error>"
end

local function secretValue(target, key, value)
    if type(issecretvalue) ~= "function" then
        recordError(target, key .. "Secret", "issecretvalue is unavailable")
        return true, "<unavailable>"
    end

    local ok, secret = pcall(issecretvalue, value)
    if not ok then
        recordError(target, key .. "Secret", secret)
        return true, "<unavailable>"
    end
    if secret then
        target.errors[key] = "secret value"
        return true, "<secret>"
    end
    return false
end

local function scalar(target, key, value)
    local restricted, marker = secretValue(target, key, value)
    if restricted then
        return marker, true
    end

    local valueType = type(value)
    if valueType == "string" or valueType == "number" or valueType == "boolean" or value == nil then
        return value, false
    end
    recordError(target, key, "unexpected " .. valueType)
    return nil, false
end

local function callScalar(target, key, fn, ...)
    if type(fn) ~= "function" then
        recordError(target, key, "function is unavailable")
        return nil
    end

    local ok, value = pcall(fn, ...)
    if not ok then
        recordError(target, key, value)
        return nil
    end
    return scalar(target, key, value)
end

local function frameName(target, key, frame)
    if frame == nil then
        return nil
    end
    if secretValue(target, key, frame) then
        return nil
    end
    return callScalar(target, key, frame.GetName, frame)
end

local function capturePoint(frameRecord, frame, index)
    local pointRecord = { index = index, errors = {} }
    local ok, point, relative, relativePoint, x, y = pcall(frame.GetPoint, frame, index)
    if not ok then
        recordError(pointRecord, "getPoint", point)
        return pointRecord
    end

    pointRecord.point = scalar(pointRecord, "point", point)
    pointRecord.relativeTo = frameName(pointRecord, "relativeTo", relative)
    pointRecord.relativePoint = scalar(pointRecord, "relativePoint", relativePoint)
    pointRecord.x = scalar(pointRecord, "x", x)
    pointRecord.y = scalar(pointRecord, "y", y)
    if next(pointRecord.errors) then
        frameRecord.errors["point" .. index] = pointRecord.errors
        pointRecord.errors = nil
    end
    return pointRecord
end

local function captureFrame(sample, name)
    local frameRecord = { name = name, errors = {} }
    sample.frames[name] = frameRecord

    local frame = rawget(_G, name)
    if frame == nil then
        frameRecord.present = false
        return frameRecord
    end
    if secretValue(frameRecord, "frame", frame) then
        frameRecord.present = true
        frameRecord.secret = true
        return frameRecord
    end

    frameRecord.present = true
    frameRecord.actualName = frameName(frameRecord, "name", frame)
    local parentOk, parent = pcall(frame.GetParent, frame)
    if parentOk then
        frameRecord.parent = frameName(frameRecord, "parent", parent)
    else
        recordError(frameRecord, "getParent", parent)
    end
    frameRecord.shown = callScalar(frameRecord, "shown", frame.IsShown, frame)
    frameRecord.scale = callScalar(frameRecord, "scale", frame.GetScale, frame)
    frameRecord.effectiveScale = callScalar(frameRecord, "effectiveScale", frame.GetEffectiveScale, frame)

    local ok, left, bottom, width, height = pcall(frame.GetRect, frame)
    if ok then
        frameRecord.rect = {
            left = scalar(frameRecord, "left", left),
            bottom = scalar(frameRecord, "bottom", bottom),
            width = scalar(frameRecord, "width", width),
            height = scalar(frameRecord, "height", height),
        }
    else
        recordError(frameRecord, "getRect", left)
    end

    local pointCount = callScalar(frameRecord, "pointCount", frame.GetNumPoints, frame)
    if type(pointCount) == "number" then
        frameRecord.points = {}
        for index = 1, pointCount do
            frameRecord.points[index] = capturePoint(frameRecord, frame, index)
        end
    end

    return frameRecord
end

local function captureBuild(sample)
    if type(GetBuildInfo) ~= "function" then
        recordError(sample, "getBuildInfo", "GetBuildInfo is unavailable")
        return
    end

    local ok, version, build, date, interface = pcall(GetBuildInfo)
    if not ok then
        recordError(sample, "getBuildInfo", version)
        return
    end
    sample.build = {
        version = scalar(sample, "buildVersion", version),
        build = scalar(sample, "buildNumber", build),
        date = scalar(sample, "buildDate", date),
        interface = scalar(sample, "interface", interface),
    }
end

local function captureSystemAnchors(target, systems)
    target.systemAnchors = {}
    if secretValue(target, "systems", systems) then
        return
    end
    if type(systems) ~= "table" then
        recordError(target, "systems", "layout systems are unavailable")
        return
    end
    target.systemCount = #systems
    for _, systemInfo in ipairs(systems) do
        local record = { errors = {} }
        if not secretValue(record, "systemInfo", systemInfo) then
            record.system = scalar(record, "system", systemInfo.system)
            record.systemIndex = scalar(record, "systemIndex", systemInfo.systemIndex)
            if record.system == Enum.EditModeSystem.RaidWarning
                or record.system == Enum.EditModeSystem.ObjectiveTracker then
                local anchor = systemInfo.anchorInfo
                if not secretValue(record, "anchorInfo", anchor) and type(anchor) == "table" then
                    record.anchorInfo = {}
                    for _, key in ipairs({ "point", "relativeTo", "relativePoint", "offsetX", "offsetY" }) do
                        record.anchorInfo[key] = scalar(record, key, anchor[key])
                    end
                end
                target.systemAnchors[#target.systemAnchors + 1] = record
            end
        end
        if next(record.errors) then
            target.errors["system" .. tostring(record.system)] = record.errors
        end
    end
end

local function captureEditMode(sample)
    sample.editMode = { errors = {} }
    local manager = rawget(_G, "EditModeManagerFrame")
    if manager == nil then
        sample.editMode.present = false
        return
    end
    if secretValue(sample.editMode, "manager", manager) then
        sample.editMode.present = true
        sample.editMode.secret = true
        return
    end

    sample.editMode.present = true
    sample.editMode.active = callScalar(sample.editMode, "active", manager.IsEditModeActive, manager)
    sample.editMode.activeLayout = callScalar(sample.editMode, "activeLayout", function()
        return C_EditMode.GetLayouts().activeLayout
    end)
    local ok, layout = pcall(manager.GetActiveLayoutInfo, manager)
    if not ok then
        recordError(sample.editMode, "getActiveLayoutInfo", layout)
        return
    end
    if layout == nil then
        sample.editMode.layoutPresent = false
        return
    end
    if secretValue(sample.editMode, "layout", layout) then
        sample.editMode.layoutPresent = true
        sample.editMode.layoutSecret = true
        return
    end

    sample.editMode.layoutPresent = true
    sample.editMode.layoutName = scalar(sample.editMode, "layoutName", layout.layoutName)
    sample.editMode.layoutType = scalar(sample.editMode, "layoutType", layout.layoutType)
    captureSystemAnchors(sample.editMode, layout.systems)
end

local function captureNonBlizzardAddons(sample)
    sample.loadedNonBlizzardAddons = {}
    local addons = rawget(_G, "C_AddOns")
    if type(addons) ~= "table" then
        recordError(sample, "cAddOns", "C_AddOns is unavailable")
        return
    end

    local count = callScalar(sample, "addonCount", addons.GetNumAddOns)
    if type(count) ~= "number" then
        return
    end

    for index = 1, count do
        local info = { index = index, errors = {} }
        local ok, name = pcall(addons.GetAddOnInfo, index)
        if not ok then
            recordError(info, "getAddOnInfo", name)
        else
            info.name = scalar(info, "name", name)
            local loaded = callScalar(info, "loaded", addons.IsAddOnLoaded, index)
            if loaded == true and type(info.name) == "string" and not info.name:match("^Blizzard_") then
                table.insert(sample.loadedNonBlizzardAddons, info.name)
            end
        end
        if next(info.errors) then
            sample.errors["addon" .. index] = info.errors
        end
    end
end

local function captureRaidState(sample)
    local raidWarning = rawget(_G, "RaidWarningFrame")
    sample.raidWarning = { errors = {} }
    if raidWarning == nil then
        sample.raidWarning.present = false
        return
    end

    sample.raidWarning.present = true
    sample.raidWarning.editMode = scalar(sample.raidWarning, "editMode", raidWarning.isInEditMode)
    local ok, lowest = pcall(raidWarning.GetLowestMessage, raidWarning)
    if not ok then
        recordError(sample.raidWarning, "getLowestMessage", lowest)
        return
    end
    local restricted, marker = secretValue(sample.raidWarning, "lowestMessage", lowest)
    if restricted then
        sample.raidWarning.lowestMessage = marker
        sample.raidWarning.lowestMessagePresent = marker
    else
        sample.raidWarning.lowestMessage = frameName(sample.raidWarning, "lowestMessage", lowest)
        sample.raidWarning.lowestMessagePresent = lowest ~= nil
    end
end

local function captureSample(kind, label)
    local sample = {
        kind = kind,
        label = label,
        errors = {},
        frames = {},
    }
    FramePositionProbeDB.samples[#FramePositionProbeDB.samples + 1] = sample

    sample.time = callScalar(sample, "time", GetTime)
    sample.capturedAt = callScalar(sample, "capturedAt", date, "%Y-%m-%dT%H:%M:%S")
    sample.inCombat = callScalar(sample, "inCombat", InCombatLockdown)
    sample.playerClass = { errors = {} }
    if type(UnitClass) == "function" then
        local ok, localized, english, classID = pcall(UnitClass, "player")
        if ok then
            sample.playerClass.localized = scalar(sample.playerClass, "localized", localized)
            sample.playerClass.english = scalar(sample.playerClass, "english", english)
            sample.playerClass.id = scalar(sample.playerClass, "id", classID)
        else
            recordError(sample.playerClass, "unitClass", localized)
        end
    else
        recordError(sample.playerClass, "unitClass", "UnitClass is unavailable")
    end

    local physicalWidth, physicalHeight = nil, nil
    if type(GetPhysicalScreenSize) == "function" then
        local ok, width, height = pcall(GetPhysicalScreenSize)
        if ok then
            physicalWidth = scalar(sample, "physicalWidth", width)
            physicalHeight = scalar(sample, "physicalHeight", height)
        else
            recordError(sample, "physicalScreenSize", width)
        end
    else
        recordError(sample, "physicalScreenSize", "GetPhysicalScreenSize is unavailable")
    end
    sample.screen = {
        width = callScalar(sample, "screenWidth", GetScreenWidth),
        height = callScalar(sample, "screenHeight", GetScreenHeight),
        physicalWidth = physicalWidth,
        physicalHeight = physicalHeight,
        uiScaleCVar = callScalar(sample, "uiScale", GetCVar, "uiScale"),
        useUiScaleCVar = callScalar(sample, "useUiScale", GetCVar, "useUiScale"),
    }

    captureBuild(sample)
    for key, capture in pairs({ editMode = captureEditMode, raidWarning = captureRaidState, addons = captureNonBlizzardAddons }) do
        local ok, errorMessage = pcall(capture, sample)
        if not ok then
            recordError(sample, key, errorMessage)
        end
    end
    for _, name in ipairs(frameNames) do
        local ok, errorMessage = pcall(captureFrame, sample, name)
        if not ok then
            recordError(sample, name, errorMessage)
        end
    end
end

local function scheduleDelayedSamples()
    if type(C_Timer) ~= "table" or type(C_Timer.After) ~= "function" then
        FramePositionProbeDB.errors.timer = "C_Timer.After is unavailable"
        return
    end

    for _, delay in ipairs({ 0, 2, 5 }) do
        local ok, errorMessage = pcall(C_Timer.After, delay, function()
            captureSample("delayed", "world+" .. delay)
        end)
        if not ok then
            FramePositionProbeDB.errors["timer" .. delay] = tostring(errorMessage)
        end
    end
end

local function onEvent(_, event)
    if event == "PLAYER_LOGIN" then
        captureSample("PLAYER_LOGIN")
    elseif event == "PLAYER_ENTERING_WORLD" then
        captureSample("PLAYER_ENTERING_WORLD")
        scheduleDelayedSamples()
    end
end

FramePositionProbeDB = {
    schemaVersion = 1,
    addonName = addonName,
    errors = {},
    samples = {},
}

SlashCmdList.FRAMEPOSITIONPROBE = function(command)
    local label = command:match("^%s*(.-)%s*$")
    if label == "" then
        label = "manual"
    end
    captureSample("manual", label)
    print("FramePositionProbe captured '" .. label .. "'; /reload or logout to flush SavedVariables")
end
SLASH_FRAMEPOSITIONPROBE1 = "/fpprobe"

local eventFrame = CreateFrame("Frame")
eventFrame:RegisterEvent("PLAYER_LOGIN")
eventFrame:RegisterEvent("PLAYER_ENTERING_WORLD")
eventFrame:SetScript("OnEvent", onEvent)
