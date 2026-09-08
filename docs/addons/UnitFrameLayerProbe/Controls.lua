local _, Probe = ...
local MAX_RUNS, SHOT_DELAY, SHOT_TIMEOUT = 3, 1.1, 10
local phases = { "created", "panel-hide-show", "panel-raise" }
local definitions = {
    { label = "1: LOW top-level parent / HIGH RED vs MEDIUM BLUE", strata = "HIGH", parent = true },
    { label = "2: independent HIGH RED vs MEDIUM BLUE", strata = "HIGH" },
    { label = "3: DIALOG RED vs MEDIUM BLUE", strata = "DIALOG" },
    { label = "4: plain TOOLTIP RED vs MEDIUM BLUE", strata = "TOOLTIP" },
    { label = "5: owned GameTooltip RED vs MEDIUM BLUE", strata = "TOOLTIP", tooltip = true },
}
local active, pendingShot

local function message(text)
    print("[UnitFrameLayerProbe] " .. text)
end

local function cleanup(run)
    run.record.cleanupErrors = {}
    for _, frame in ipairs(run.owned) do
        for _, method in ipairs({ "Hide", "ClearAllPoints", "SetParent" }) do
            local result = Probe.callMethod(frame, method)
            if result.status ~= "ok" then
                run.record.cleanupErrors[#run.record.cleanupErrors + 1] = { method = method, result = result }
            end
        end
    end
end

local function finish(run, status, detail)
    if active ~= run then return end
    active = nil
    run.record.status = status
    run.record.error = detail
    run.record.finishedAt = Probe.callGlobal("GetServerTime")
    cleanup(run)
    message("Control run " .. run.record.sequence .. ": " .. status .. "; /reload to save data.")
end

local function guarded(run, fn)
    if active ~= run then return end
    local ok, err = pcall(fn)
    if not ok then finish(run, "error", Probe.errorText(err)) end
end

local function later(run, delay, fn)
    C_Timer.After(delay, function() guarded(run, fn) end)
end

local function ownFrame(run, key, kind, parent, template)
    local name = kind == "GameTooltip" and ("UnitFrameLayerProbeTooltip" .. run.record.sequence) or nil
    local frame = CreateFrame(kind, name, parent, template)
    run.owned[#run.owned + 1] = frame
    run.frames[key] = frame
    return frame
end

local function position(frame, x, y, width, height)
    frame:SetSize(width, height)
    frame:SetPoint("TOPLEFT", UIParent, "CENTER", x, y)
end

local function color(frame, red, green, blue)
    local texture = frame:CreateTexture(nil, "OVERLAY")
    texture:SetAllPoints(frame)
    texture:SetColorTexture(red, green, blue, 1)
end

local function label(run, key, text, x, y, width)
    local frame = ownFrame(run, key, "Frame", UIParent)
    frame:SetFrameStrata("TOOLTIP")
    position(frame, x, y, width, 30)
    local font = frame:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    font:SetAllPoints(frame)
    font:SetTextColor(1, 1, 1, 1)
    font:SetText(text)
    return font
end

local function configureTooltip(case)
    case.red:SetOwner(case.blue, "ANCHOR_NONE")
    case.red:SetText("Owned GameTooltip RED")
    case.red:ClearAllPoints()
    position(case.red, case.x, case.y, 180, 100)
    case.red:Show()
end

local function buildCase(run, index, definition)
    local key = "case" .. index
    local x, y = -520 + ((index - 1) % 3) * 360, 180 - math.floor((index - 1) / 3) * 240
    local parent = UIParent
    if definition.parent then
        parent = ownFrame(run, key .. ".parent", "Frame", UIParent)
        parent:SetFrameStrata("LOW")
        parent:SetToplevel(true)
        position(parent, x, y, 180, 100)
    end
    local kind = definition.tooltip and "GameTooltip" or "Frame"
    local red = ownFrame(run, key .. ".red", kind, parent, definition.tooltip and "GameTooltipTemplate" or nil)
    red:SetFrameStrata(definition.strata)
    position(red, x, y, 180, 100)
    color(red, 1, 0, 0)
    local blue = ownFrame(run, key .. ".blue", "Frame", UIParent)
    blue:SetFrameStrata("MEDIUM")
    blue:SetToplevel(true)
    position(blue, x + 40, y - 30, 180, 100)
    color(blue, 0, 0, 1)
    local case = { red = red, blue = blue, tooltip = definition.tooltip, x = x, y = y }
    run.cases[#run.cases + 1] = case
    if case.tooltip then configureTooltip(case) end
    label(run, key .. ".label", definition.label, x, y + 38, 330)
end

local function buildFixtures(run)
    for i, definition in ipairs(definitions) do buildCase(run, i, definition) end
    run.header = label(run, "header", "", -520, 260, 1100)
end

local function capturePhase(run)
    local name = phases[run.phase]
    local now = GetServerTime()
    local header = "UnitFrameLayerProbe run " .. run.record.sequence .. " | " .. name .. " | " .. tostring(now)
    run.header:SetText(header)
    local sample = {
        name = name, header = header, build = Probe.callGlobal("GetBuildInfo"),
        timestamp = Probe.callGlobal("GetServerTime"), uptime = Probe.callGlobal("GetTime"), frames = {},
        screenshot = { status = "requested", requestedAt = Probe.callGlobal("GetTime") },
    }
    for key, frame in pairs(run.frames) do sample.frames[key] = Probe.captureFrame(frame) end
    sample.tooltipOwner = Probe.callMethod(run.cases[5].red, "GetOwner")
    run.record.phases[#run.record.phases + 1] = sample
    return sample
end

local function requestScreenshot(run)
    local sample = capturePhase(run)
    local request = { run = run, sample = sample }
    pendingShot = request
    local ok, err = pcall(Screenshot)
    if not ok then
        pendingShot = nil
        sample.screenshot.status = "error"
        error(err)
    end
    C_Timer.After(SHOT_TIMEOUT, function()
        if pendingShot ~= request then return end
        sample.screenshot.status = "timeout"
        finish(run, "timeout")
        -- Keep the outstanding request as a barrier: screenshot events have no request ID.
        -- A late completion drains it; otherwise /reload is required before another run.
    end)
end

local function applyPhase(run)
    for _, case in ipairs(run.cases) do
        if run.phase == 2 then
            case.blue:Hide()
            case.blue:Show()
            if case.tooltip then configureTooltip(case) end
        elseif run.phase == 3 then
            case.blue:Raise()
        end
    end
    later(run, 0, function() requestScreenshot(run) end)
end

local function screenshotFinished(event)
    local request = pendingShot
    if not request then return end
    pendingShot = nil
    local shot, run = request.sample.screenshot, request.run
    shot.event = event
    shot.completedAt = Probe.callGlobal("GetTime")
    if shot.status ~= "timeout" then shot.status = event == "SCREENSHOT_SUCCEEDED" and "succeeded" or "failed" end
    if active ~= run then return end
    if event == "SCREENSHOT_FAILED" then finish(run, "screenshot_failed"); return end
    if run.phase == #phases then finish(run, "complete"); return end
    later(run, SHOT_DELAY, function()
        run.phase = run.phase + 1
        applyPhase(run)
    end)
end

function Probe.startControls()
    if active then message("A control run is active; use /unitlayerprobe cancel."); return end
    if pendingShot then message("A screenshot is still outstanding; wait for completion or /reload."); return end
    local db = Probe.database()
    if not db then return end
    if db.controlRuns == nil then db.controlRuns = {} end
    if type(db.controlRuns) ~= "table" then message("Invalid control history; left untouched."); return end
    if #db.controlRuns >= MAX_RUNS then message("Control limit 3 reached; existing records preserved."); return end
    local record = {
        sequence = #db.controlRuns + 1, probeVersion = Probe.version, status = "running", phases = {},
        startedAt = Probe.callGlobal("GetServerTime"), build = Probe.callGlobal("GetBuildInfo"),
    }
    db.controlRuns[#db.controlRuns + 1] = record
    local run = { record = record, owned = {}, frames = {}, cases = {}, phase = 1 }
    active = run
    message("Starting controls. Close menus/SpellBook before running; only owned fixtures are changed.")
    guarded(run, function() buildFixtures(run); applyPhase(run) end)
end

function Probe.cancelControls()
    if active then finish(active, "cancelled") else message("No control run is active.") end
end

local listener = CreateFrame("Frame")
listener:RegisterEvent("SCREENSHOT_SUCCEEDED")
listener:RegisterEvent("SCREENSHOT_FAILED")
listener:SetScript("OnEvent", function(_, event)
    local request = pendingShot
    if not request then return end
    local ok, err = pcall(screenshotFinished, event)
    if not ok then finish(request.run, "error", Probe.errorText(err)) end
end)
