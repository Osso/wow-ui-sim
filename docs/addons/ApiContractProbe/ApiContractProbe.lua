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
        if #value > 256 then result.truncated = true end
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

-- Cast results are scalar observations only: never inspect returned objects.
local function observeCast(fn, ...)
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, ...))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do
        result.values[index - 1] = scalar(values[index])
    end
    if values.n > 17 then result.truncated = true end
    return result
end

local function resourceCurve()
    local inputs = { { x = 0, y = 0 }, { x = 0.5, y = 10 }, { x = 1, y = 20 },
        { x = 25, y = 30 }, { x = 50, y = 40 }, { x = 100, y = 50 } }
    local kindTable = readField(Enum, "LuaCurveType")
    local linear, ok = readField(kindTable, "Linear")
    if not ok or not accessible(linear) or type(linear) ~= "number" then
        return nil, { inputs = inputs, status = "missing-linear-type" }
    end
    local curve, failure = newCurve(inputs)
    if not curve then return nil, { inputs = inputs, status = failure } end
    local setting = method(curve, "SetType", linear)
    if setting.status ~= "observed" then return nil, { inputs = inputs, status = "set-type-error" } end
    return curve, { inputs = inputs, status = "constructed", interpolation = "Linear" }
end

local function resourcePower(unit, unmodified, curve)
    return {
        current = observeCast(UnitPower, unit, nil, unmodified),
        maximum = observeCast(UnitPowerMax, unit, nil, unmodified),
        percent = observeCast(UnitPowerPercent, unit, nil, unmodified),
        nilCurve = observeCast(UnitPowerPercent, unit, nil, unmodified, nil),
        curved = curve and observeCast(UnitPowerPercent, unit, nil, unmodified, curve)
            or { status = "curve-unavailable" },
    }
end

local function captureResources()
    local curve, definition = resourceCurve()
    local result = { curve = definition, units = {} }
    for _, unit in ipairs({ "player", "target", "focus", "pet", "nonexistent" }) do
        result.units[unit] = {
            health = {
                current = observeCast(UnitHealth, unit), maximum = observeCast(UnitHealthMax, unit),
                default = observeCast(UnitHealthPercent, unit),
                percent = observeCast(UnitHealthPercent, unit, false),
                nilCurve = observeCast(UnitHealthPercent, unit, false, nil),
                curved = curve and observeCast(UnitHealthPercent, unit, false, curve)
                    or { status = "curve-unavailable" },
            },
            power = {
                type = observeCast(UnitPowerType, unit),
                current = observeCast(UnitPower, unit), maximum = observeCast(UnitPowerMax, unit),
                default = observeCast(UnitPowerPercent, unit),
                modified = resourcePower(unit, false, curve), unmodified = resourcePower(unit, true, curve),
            },
        }
    end
    return result
end

local function captureCasts()
    local result = { units = {} }
    for _, unit in ipairs({ "player", "target", "focus", "party1", "nonexistent", "invalid-unit-token", "" }) do
        result.units[unit] = { casting = observeCast(UnitCastingInfo, unit),
            channel = observeCast(UnitChannelInfo, unit) }
    end
    return result
end

local function captureHyperlinks()
    local inputs = {
        "plain ASCII", "é漢字🙂", "", "a|nb", "a\nb",
        "|Hitem:19019|h[Item]|h", "|cffff0000red|r", "|A:atlas:16:16|a",
        "|Ttexture:16:16|t", "|cffff0000|Hitem:19019|h[Item]|h|r |A:atlas:16:16|a |Ttexture:16:16|t",
        "||", "|Hitem:19019|h[Item]", "|cffff0000red", "|A:atlas:16:16", "|Ttexture:16:16", "|h",
    }
    local variants = {
        { name = "omitted", flags = {} },
        { name = "false", flags = { false, false, false, false, false } },
        { name = "maintainColor", flags = { true, false, false, false, false } },
        { name = "maintainBrackets", flags = { false, true, false, false, false } },
        { name = "stripNewlines", flags = { false, false, true, false, false } },
        { name = "maintainAtlases", flags = { false, false, false, true, false } },
        { name = "maintainTextures", flags = { false, false, false, false, true } },
        { name = "consumer", flags = { false, true, false, true, true } },
        { name = "true", flags = { true, true, true, true, true } },
    }
    local fn = readField(C_StringUtil, "StripHyperlinks")
    local rows = {}
    for _, input in ipairs(inputs) do
        for _, variant in ipairs(variants) do
            rows[#rows + 1] = { input = input, variant = variant.name, flags = variant.flags,
                argumentCount = 1 + #variant.flags,
                result = observe(fn, input, unpack(variant.flags)) }
        end
    end
    return rows
end

local function captureNames()
    local result = { units = {} }
    for _, unit in ipairs({
        "player", "party1", "party2", "party3", "party4", "target",
        "nonexistent", "invalid-unit-token", "",
    }) do
        result.units[unit] = { name = observe(UnitName, unit),
            unmodified = observe(UnitNameUnmodified, unit) }
    end
    return result
end

local function captureNumbers()
    local floor, floorOK = readField(C_StringUtil, "FloorToNearestString")
    local round, roundOK = readField(C_StringUtil, "RoundToNearestString")
    if not floorOK then floor = nil end
    if not roundOK then round = nil end
    local result = { locale = observe(GetLocale), samples = {} }
    for _, input in ipairs({
        -2.5, -1.5, -0.5, 0, 0.5, 1.5, 2.5,
        -0.500001, -0.499999, 0.499999, 0.500001,
        -1, 1, -100, 100, -0.1, 0.1, -0.9, 0.9,
        -1234.5678, 1234.5678, -1e6, 1e6, -1e12, 1e12,
    }) do
        result.samples[#result.samples + 1] = { input = input,
            floor = observe(floor, input), round = observe(round, input) }
    end
    return result
end

local function observeAction(fn, fields, ...)
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, ...))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do
        local value = values[index]
        local item
        if fields == false then
            item = accessible(value) and { status = "observed", kind = type(value) } or { status = "restricted" }
        else
            item = scalar(value)
        end
        if fields and item.status == "observed" and item.kind == "table" then
            item.fields = {}
            for _, key in ipairs(fields) do item.fields[key] = scalar(rawget(value, key)) end
        end
        result.values[index - 1] = item
    end
    if values.n > 17 then result.truncated = true end
    return result
end

local function actionFunction(name)
    if not accessible(C_ActionBar) or type(C_ActionBar) ~= "table" then return nil end
    return rawget(C_ActionBar, name)
end

local function captureActions(slot)
    local display = actionFunction("GetActionDisplayCount")
    local result = { slot = slot, identity = observeAction(GetActionInfo, nil, slot),
        display = { default = observeAction(display, nil, slot), formats = {} },
        charges = observeAction(actionFunction("GetActionCharges"), {
            "currentCharges", "maxCharges", "cooldownStartTime", "cooldownDuration", "chargeModRate",
        }, slot),
        duration = observeAction(actionFunction("GetActionChargeDuration"), false, slot) }
    for _, threshold in ipairs({ 0, 1, 9999 }) do
        result.display.formats[#result.display.formats + 1] = { threshold = threshold,
            replacement = "*", result = observeAction(display, nil, slot, threshold, "*") }
    end
    return result
end

local function parseActionSlot(input)
    local token, label = string.match(input, "^(%S+)%s*(.-)%s*$")
    if not token or not string.match(token, "^%-?%d+$") then return nil end
    local slot = tonumber(token)
    -- Command syntax only: no claim about native slot validation or coercion.
    if not slot or slot < -9007199254740991 or slot > 9007199254740991 then return nil end
    return slot, label
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

local callbackSession
local function callbackPayload(...)
    local values = pack(...)
    local result = { n = values.n, values = {} }
    for index = 1, math.min(values.n, 16) do result.values[index] = scalar(values[index]) end
    if values.n > 16 then result.truncated = true end
    return result
end

local function callbackCall(fn, cb, unit)
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values
    if unit then values = pack(pcall(fn, "UNIT_HEALTH", cb, unit))
    else values = pack(pcall(fn, "UNIT_HEALTH", cb)) end
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do result.values[index - 1] = scalar(values[index]) end
    if values.n > 17 then result.truncated = true end
    return result
end

local function callbackOutcome(result)
    if result.status ~= "observed" then return result.status end
    if result.truncated then return "uncertain" end
    for _, value in ipairs(result.values) do
        if value.status ~= "observed" then return "uncertain" end
    end
    local first = result.values[1]
    if first and first.kind == "boolean" and first.value == false then return "refused" end
    return "accepted"
end

local function startCallbackLane(session, name, register, unit)
    local lane = { unit = unit }
    local record = {}
    session.record[name] = record
    lane.callback = function(...)
        if not session.receiving then return end
        local saved = session.record
        if #saved.events >= 128 then saved.dropped = saved.dropped + 1; return end
        saved.events[#saved.events + 1] = { lane = name, label = saved.label,
            time = observeCast(GetTime), payload = callbackPayload(...) }
    end
    -- Retain identity even after a call error: the API may have registered before throwing.
    lane.pending = accessible(register) and type(register) == "function"
    record.registration = callbackCall(register, lane.callback, unit)
    local outcome = callbackOutcome(record.registration)
    record.status = outcome == "accepted" and "registered" or outcome
    session[name] = lane
    return outcome == "accepted"
end

local function stopCallbackLane(session, name, unregister)
    local lane, record = session[name], session.record[name]
    if not lane.pending then return true end
    record.removal = callbackCall(unregister, lane.callback, lane.unit)
    local outcome = callbackOutcome(record.removal)
    if outcome ~= "accepted" then record.status = "cleanup-" .. outcome; return false end
    lane.pending, record.status = false, "removed"
    return true
end

local function controlCallbacks(mode, label)
    local db = database()
    db.callbackSessions = db.callbackSessions or {}
    if mode == "callbacks-stop" then
        if not callbackSession then db.callbackStatus = "not-running"; return end
        callbackSession.receiving = false
        local globalOK = stopCallbackLane(callbackSession, "global", UnregisterEventCallback)
        local unitOK = stopCallbackLane(callbackSession, "unit", UnregisterUnitEventCallback)
        db.callbackStatus = globalOK and unitOK and "stopped" or "cleanup-incomplete"
        callbackSession.record.status = db.callbackStatus
        if globalOK and unitOK then callbackSession = nil end
        return
    end
    if callbackSession then db.callbackStatus = "already-running"; return end
    if type(issecretvalue) ~= "function" or type(canaccessvalue) ~= "function" then
        db.callbackStatus = "missing-access-api"; return
    end
    if #db.callbackSessions >= 10 then db.callbackStatus = "session-limit"; return end
    local record = { label = label, client = observeCast(GetBuildInfo), time = observeCast(GetTime),
        events = {}, dropped = 0 }
    local session = { record = record, receiving = true }
    db.callbackSessions[#db.callbackSessions + 1] = record
    callbackSession = session
    local globalOK = startCallbackLane(session, "global", RegisterEventCallback)
    local unitOK = startCallbackLane(session, "unit", RegisterUnitEventCallback, "player")
    record.status = globalOK and unitOK and "recording" or "registration-incomplete"
    db.callbackStatus = record.status
end

SLASH_APICONTRACTPROBE1 = "/apicontract"
SlashCmdList.APICONTRACTPROBE = function(input)
    local mode, label = string.match(input or "", "^%s*(%S*)%s*(.-)%s*$")
    if mode == "" then mode = "all" end
    if mode == "callbacks-start" or mode == "callbacks-stop" then
        controlCallbacks(mode, string.sub(label, 1, 128)); return
    end
    if mode == "events-start" or mode == "events-stop" then
        controlEvents(mode, string.sub(label, 1, 128)); return
    end
    local slot
    if mode == "actions" then
        slot, label = parseActionSlot(label)
        if slot == nil then print("Usage: /apicontract actions <integer-slot> <label>"); return end
    end
    if mode ~= "resources" and mode ~= "hyperlinks" and mode ~= "actions" and mode ~= "all" and mode ~= "curves" and mode ~= "sex" and mode ~= "names" and mode ~= "numbers" and mode ~= "casts" and mode ~= "publication" then
        print("Usage: /apicontract [all|curves|sex|names|numbers|casts|resources|hyperlinks|publication|events-start|events-stop|callbacks-start|callbacks-stop] [label]")
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
        if mode == "resources" then record.resources = captureResources() end
        if mode == "actions" then record.actions = captureActions(slot) end
        if mode == "hyperlinks" then record.hyperlinks = captureHyperlinks() end
        if mode == "all" or mode == "curves" then record.curves = captureCurves() end
        if mode == "all" or mode == "sex" then record.sex = captureSex() end
        if mode == "all" or mode == "names" then record.names = captureNames() end
        if mode == "all" or mode == "numbers" then record.numbers = captureNumbers() end
        if mode == "all" or mode == "casts" then record.casts = captureCasts() end
        if mode == "all" or mode == "publication" then record.publication = capturePublication() end
        record.status = "observed"
    end
    db.captures[#db.captures + 1] = record
end
