-- Observations only. No native result is normalized into a guessed contract.
local function accessible(value)
    if type(issecretvalue) ~= "function" or type(canaccessvalue) ~= "function" then return false end
    local ok, secret = pcall(issecretvalue, value)
    if not ok or secret then return false end
    local accessOK, access = pcall(canaccessvalue, value)
    return accessOK and access == true
end

local function pack(...)
    return { n = select("#", ...), ... }
end

local function readField(object, key)
    if not accessible(object) then return nil, false end
    local ok, value = pcall(function() return object[key] end)
    return value, ok
end

local function scalar(value)
    if not accessible(value) then return { status = "restricted" } end
    local kind = type(value)
    local result = { kind = kind, status = "observed" }
    if kind == "number" then
        if value ~= value or value == math.huge or value == -math.huge then
            result.status = "nonfinite"
        else
            result.value = value
        end
    elseif kind == "string" then
        result.value = string.sub(value, 1, 256)
    elseif kind == "boolean" then
        result.value = value
    end
    return result
end

local function summarize(value, depth, invokeMethods)
    local result = scalar(value)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    if depth >= 2 then result.status = "depth-limit"; return result end
    -- Passive event payloads must not execute __index or object methods.
    if not invokeMethods and result.kind == "userdata" then return result end
    local lookup = invokeMethods and readField or function(object, key)
        return rawget(object, key), true
    end
    result.fields = {}
    for _, key in ipairs({ "x", "y" }) do
        local field, ok = lookup(value, key)
        result.fields[key] = ok and scalar(field) or { status = "field-error" }
    end
    local getXY, methodOK = lookup(value, "GetXY")
    if invokeMethods and methodOK and accessible(getXY) and type(getXY) == "function" then
        local values = pack(pcall(getXY, value))
        result.xy = { status = values[1] and "observed" or "call-error" }
        if values[1] then
            result.xy.n, result.xy.values = values.n - 1, {}
            for index = 2, math.min(values.n, 5) do
                result.xy.values[index - 1] = scalar(values[index])
            end
        end
    end
    if result.kind == "table" then
        result.entries = {}
        for index = 1, 4 do
            local entry, ok = lookup(value, index)
            result.entries[index] = ok and summarize(entry, depth + 1, invokeMethods) or { status = "field-error" }
        end
    end
    return result
end

local function observe(fn, ...)
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, ...))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 9) do
        result.values[index - 1] = summarize(values[index], 0, true)
    end
    if values.n > 9 then result.truncated = true end
    return result
end

local function method(object, name, ...)
    local fn, ok = readField(object, name)
    if not ok then return { status = "field-error" } end
    return observe(fn, object, ...)
end

local function newCurve(points)
    local create, ok = readField(C_CurveUtil, "CreateCurve")
    if not ok or not accessible(create) or type(create) ~= "function" then return nil, "missing-api" end
    local success, curve = pcall(create)
    if not success then return nil, "construction-error" end
    if not accessible(curve) then return nil, "restricted-curve" end
    for _, point in ipairs(points) do
        local result = method(curve, "AddPoint", point.x, point.y)
        if result.status ~= "observed" then return nil, "add-point-error" end
    end
    return curve
end

local function rawPoint(curve)
    local fn, ok = readField(curve, "GetPoint")
    if not ok or not accessible(fn) or type(fn) ~= "function" then return nil, false end
    local success, point = pcall(fn, curve, 1)
    if not success or not accessible(point) then return nil, false end
    return point, type(point) == "table" or type(point) == "userdata"
end

local function captureCurves()
    local points = { { x = 30, y = 4 }, { x = 10, y = 7 }, { x = 20, y = 2 } }
    local empty, failure = newCurve({})
    if not empty then return { status = failure } end
    local result = { status = "observed", inputs = points, empty = method(empty, "GetPoints") }
    local curve
    curve, failure = newCurve(points)
    if not curve then result.status = failure; return result end
    result.points, result.indices = method(curve, "GetPoints"), {}
    for _, index in ipairs({ 0, 1, 2, 3, -1, 4 }) do
        result.indices[#result.indices + 1] = { index = index, result = method(curve, "GetPoint", index) }
    end
    local first, firstOK = rawPoint(curve)
    local second, secondOK = rawPoint(curve)
    result.identity = { status = "inconclusive" }
    if firstOK and secondOK then
        local ok, equal = pcall(function() return first == second end)
        result.identity = ok and scalar(equal) or { status = "comparison-error" }
    end
    -- A separate curve prevents mutation from contaminating preceding observations.
    local mutationCurve
    mutationCurve, failure = newCurve(points)
    if not mutationCurve then result.mutation = { status = failure }; return result end
    result.mutation = { before = method(mutationCurve, "GetPoint", 1) }
    local point, safe = rawPoint(mutationCurve)
    if not safe then result.mutation.status = "inconclusive"; return result end
    local changed = pcall(function() point.x = 91 end)
    result.mutation.status = changed and "write-accepted" or "write-error"
    result.mutation.after = method(mutationCurve, "GetPoint", 1)
    return result
end

local function captureSex()
    local result = { units = {}, enums = {} }
    local unitSexEnum = readField(Enum, "UnitSex")
    for _, name in ipairs({ "Male", "Female", "None", "Both", "Neutral" }) do
        local value, ok = readField(unitSexEnum, name)
        result.enums[name] = ok and scalar(value) or { status = "field-error" }
    end
    for _, unit in ipairs({ "player", "target", "focus", "pet" }) do
        local exists = observe(UnitExists, unit)
        local value = exists.values and exists.values[1]
        if not value or value.status ~= "observed" or value.kind ~= "boolean" then
            result.units[unit] = { status = "restricted-or-error" }
        elseif not value.value then
            result.units[unit] = { status = "absent" }
        else
            result.units[unit] = { status = "observed", legacy = observe(UnitSex, unit),
                base = observe(UnitSexBase, unit) }
        end
    end
    return result
end

local function capturePublication()
    local rows = {}
    for index, target in ipairs(ApiContractProbeTargets.publication) do
        if index > 256 then break end
        local row = { id = target.id, plan = target.plan, owner = target.owner }
        if target.owner == "cvar" then
            local getter = readField(C_CVar, "GetCVar")
            local defaultGetter = readField(C_CVar, "GetCVarDefault")
            row.current, row.default = observe(getter, target.path), observe(defaultGetter, target.path)
        else
            local keys = {}
            for key in string.gmatch(target.path, "[^.]+") do keys[#keys + 1] = key end
            local parent = _G
            for part = 1, #keys - 1 do
                local value, ok = readField(parent, keys[part])
                if not ok or not accessible(value) then
                    row.status = "restricted-or-error-parent"; break
                end
                if value == nil then row.status = "missing-parent"; break end
                parent = value
            end
            if not row.status then
                row.parent = scalar(parent)
                local value, ok = readField(parent, keys[#keys])
                row.ordinary = ok and scalar(value) or { status = "lookup-error" }
                if accessible(parent) and type(parent) == "table" then
                    local rawOK, raw = pcall(rawget, parent, keys[#keys])
                    row.raw = rawOK and scalar(raw) or { status = "raw-lookup-error" }
                else
                    row.raw = { status = "not-accessible-table" }
                end
            end
        end
        rows[#rows + 1] = row
    end
    return rows
end

local eventFrame
local function database()
    if ApiContractProbeDB == nil then
        ApiContractProbeDB = { schema = 1, captures = {}, dropped = 0 }
    end
    local db = ApiContractProbeDB
    db.events = db.events or {}
    db.registrations = db.registrations or {}
    db.droppedEvents = db.droppedEvents or 0
    return db
end

local function recordEvent(_, name, ...)
    local db = database()
    if #db.events >= 256 then db.droppedEvents = db.droppedEvents + 1; return end
    local values = pack(...)
    local payload = { n = values.n, values = {} }
    for index = 1, math.min(values.n, 8) do payload.values[index] = summarize(values[index], 0) end
    if values.n > 8 then payload.truncated = true end
    local safeName = scalar(name)
    db.events[#db.events + 1] = { name = safeName.value, nameStatus = safeName.status,
        time = observe(GetTime), payload = payload, label = db.eventLabel }
end

local function controlEvents(mode, label)
    local db = database()
    if mode == "events-stop" then
        if eventFrame then
            local stopped = method(eventFrame, "UnregisterAllEvents")
            db.eventStatus = stopped.status == "observed" and "stopped" or "stop-error"
            if db.eventStatus == "stopped" then eventFrame = nil end
        else
            db.eventStatus = "not-running"
        end
        return
    end
    if eventFrame then db.eventStatus = "already-running"; return end
    if type(issecretvalue) ~= "function" or type(canaccessvalue) ~= "function" then
        db.eventStatus = "missing-access-api"; return
    end
    if type(CreateFrame) ~= "function" then db.eventStatus = "missing-frame-api"; return end
    local ok, frame = pcall(CreateFrame, "Frame")
    if not ok or not accessible(frame) then db.eventStatus = "frame-error"; return end
    local script = method(frame, "SetScript", "OnEvent", recordEvent)
    if script.status ~= "observed" then db.eventStatus = "script-error"; return end
    eventFrame, db.eventLabel = frame, label
    db.registrations = {}
    for index, target in ipairs(ApiContractProbeTargets.events) do
        if index > 256 then break end
        local registration = method(frame, "RegisterEvent", target.event)
        db.registrations[#db.registrations + 1] = { id = target.id, plan = target.plan,
            status = registration.status == "observed" and "registered" or "registration-error" }
    end
    db.eventStatus, db.eventClient = "recording", observe(GetBuildInfo)
    db.sourceHash = ApiContractProbeTargets.sourceHash
end

SLASH_APICONTRACTPROBE1 = "/apicontract"
SlashCmdList.APICONTRACTPROBE = function(input)
    local mode, label = string.match(input or "", "^%s*(%S*)%s*(.-)%s*$")
    if mode == "" then mode = "all" end
    if mode == "events-start" or mode == "events-stop" then
        controlEvents(mode, string.sub(label, 1, 128)); return
    end
    if mode ~= "all" and mode ~= "curves" and mode ~= "sex" and mode ~= "publication" then
        print("Usage: /apicontract [all|curves|sex|publication|events-start|events-stop] [label]")
        return
    end
    local db = database()
    if #db.captures >= 10 then db.dropped = db.dropped + 1; return end
    local record = { mode = mode, label = string.sub(label, 1, 128),
        sourceHash = ApiContractProbeTargets.sourceHash }
    if type(issecretvalue) ~= "function" or type(canaccessvalue) ~= "function" then
        record.status = "missing-access-api"
    else
        record.client, record.time = observe(GetBuildInfo), observe(time)
        if mode == "all" or mode == "curves" then record.curves = captureCurves() end
        if mode == "all" or mode == "sex" then record.sex = captureSex() end
        if mode == "all" or mode == "publication" then record.publication = capturePublication() end
        record.status = "observed"
    end
    db.captures[#db.captures + 1] = record
end
