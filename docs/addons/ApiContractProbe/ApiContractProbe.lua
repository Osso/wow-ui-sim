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

local function curveStateMethod(curve, name, ...)
    local fn, ok = readField(curve, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, curve, ...))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for i = 2, math.min(values.n, 9) do result.values[i - 1] = scalar(values[i]) end
    if values.n > 9 then result.truncated = true end
    return result, values[2]
end

local function curveStateSnapshot(curve)
    local state = { curveType = curveStateMethod(curve, "GetType"),
        count = curveStateMethod(curve, "GetPointCount"), points = method(curve, "GetPoints"),
        hasSecretValues = curveStateMethod(curve, "HasSecretValues"), evaluations = {} }
    for _, x in ipairs({ -1, 0, 10, 15, 20, 30, 31 }) do
        state.evaluations[#state.evaluations + 1] = { x = x, result = curveStateMethod(curve, "Evaluate", x) }
    end
    return state
end

local function curveEditSnapshot(curve)
    return { count = curveStateMethod(curve, "GetPointCount"),
        points = method(curve, "GetPoints"),
        evaluationInput = 15, evaluation = curveStateMethod(curve, "Evaluate", 15) }
end

local function replaceCurvePoints(curve, inputs)
    if not accessible(CreateVector2D) or type(CreateVector2D) ~= "function" then
        return { status = "missing-vector-constructor" }
    end
    local points = {}
    for i, input in ipairs(inputs) do
        local ok, vector = pcall(CreateVector2D, input.x, input.y)
        if not ok then return { status = "vector-construction-error" } end
        if not accessible(vector) then return { status = "restricted-vector" } end
        if type(vector) ~= "table" and type(vector) ~= "userdata" then
            return { status = "invalid-vector-result" }
        end
        points[i] = vector
    end
    return curveStateMethod(curve, "SetPoints", points)
end

local function captureCurveEdit()
    local inputs = { { x = 30, y = 4 }, { x = 10, y = 7 }, { x = 20, y = 2 } }
    local result = { inputs = inputs, removals = {}, replacements = {} }
    for _, index in ipairs({ -1, 0, 1, 2, 3, 4 }) do
        local curve, failure = newCurve(inputs)
        local row = { index = index, status = failure or "observed" }
        if curve then
            row.before = curveEditSnapshot(curve)
            row.mutation = curveStateMethod(curve, "RemovePoint", index)
            row.after = curveEditSnapshot(curve)
        end
        result.removals[#result.removals + 1] = row
    end
    for _, replacement in ipairs({ {}, {
        { x = 30, y = 4 }, { x = 10, y = 7 }, { x = 20, y = 2 }, { x = 10, y = 9 },
    } }) do
        local curve, failure = newCurve(inputs)
        local row = { inputs = replacement, status = failure or "observed" }
        if curve then
            row.before = curveEditSnapshot(curve)
            row.mutation = replaceCurvePoints(curve, replacement)
            row.after = curveEditSnapshot(curve)
        end
        result.replacements[#result.replacements + 1] = row
    end
    return result
end

local function configureStateCurve(curve)
    local types, ok = readField(Enum, "LuaCurveType")
    if not ok then return { status = "missing-linear-type" } end
    local linear, found = readField(types, "Linear")
    if not found or not accessible(linear) or type(linear) ~= "number" then
        return { status = "missing-linear-type" }
    end
    return curveStateMethod(curve, "SetType", linear)
end

local function captureCurveReset(points)
    local curve, failure = newCurve(points)
    if not curve then return { status = failure } end
    local result = { status = "observed", setType = configureStateCurve(curve) }
    result.before = curveStateSnapshot(curve)
    result.reset = curveStateMethod(curve, "SetToDefaults")
    result.after = curveStateSnapshot(curve)
    return result
end

local function captureCurveCopy(points)
    local original, failure = newCurve(points)
    if not original then return { status = failure } end
    local result = { status = "observed", setType = configureStateCurve(original) }
    local copy
    result.result, copy = curveStateMethod(original, "Copy")
    if result.result.status ~= "observed" then return result end
    if not accessible(copy) then result.status = "restricted-copy"; return result end
    if type(copy) ~= "table" and type(copy) ~= "userdata" then result.status = "invalid-copy"; return result end
    result.before = curveStateSnapshot(copy)
    result.add = curveStateMethod(original, "AddPoint", 40, 11)
    result.originalAfterAdd, result.afterAdd = curveStateSnapshot(original), curveStateSnapshot(copy)
    result.clear = curveStateMethod(original, "ClearPoints")
    result.originalAfterClear, result.afterClear = curveStateSnapshot(original), curveStateSnapshot(copy)
    return result
end

local function captureCurveState()
    local points = { { x = 30, y = 4 }, { x = 10, y = 7 }, { x = 20, y = 2 }, { x = 10, y = 9 } }
    return { inputs = points, empty = captureCurveReset({}), populated = captureCurveReset(points),
        copy = captureCurveCopy(points) }
end

-- Only plain stored fields of returned colors/points are observed; userdata stays opaque.
local function colorObservation(value, depth)
    local result = scalar(value)
    if result.status ~= "observed" or result.kind ~= "table" then return result end
    if depth >= 3 then result.status = "depth-limit"; return result end
    result.fields = {}
    for _, key in ipairs({ "x", "y", "r", "g", "b", "a" }) do
        result.fields[key] = colorObservation(rawget(value, key), depth + 1)
    end
    if depth == 0 then
        result.entries = {}
        for index = 1, 4 do result.entries[index] = colorObservation(rawget(value, index), depth + 1) end
    end
    return result
end

local function colorMethod(curve, name, ...)
    local fn, ok = readField(curve, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, curve, ...))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do
        result.values[index - 1] = colorObservation(values[index], 0)
    end
    if values.n > 17 then result.truncated = true end
    return result
end

local function colorSnapshot(curve)
    local result = { curveType = colorMethod(curve, "GetType"), count = colorMethod(curve, "GetPointCount"),
        hasSecretValues = colorMethod(curve, "HasSecretValues"), points = colorMethod(curve, "GetPoints"),
        indices = {}, evaluations = {} }
    for _, index in ipairs({ -1, 0, 1, 2, 3, 4 }) do
        result.indices[#result.indices + 1] = { index = index, result = colorMethod(curve, "GetPoint", index) }
    end
    for _, x in ipairs({ -17, -16, 0, 16, 32, 48, 49 }) do
        result.evaluations[#result.evaluations + 1] = { x = x, packed = colorMethod(curve, "Evaluate", x),
            unpacked = colorMethod(curve, "EvaluateUnpacked", x) }
    end
    return result
end

local function createColorProbeCurve()
    local fn, ok = readField(C_CurveUtil, "CreateColorCurve")
    if not ok or not accessible(fn) or type(fn) ~= "function" then return nil, "missing-constructor" end
    if not accessible(CreateColor) or type(CreateColor) ~= "function" then return nil, "missing-constructor" end
    local success, curve = pcall(fn)
    if not success then return nil, "curve-construction-error" end
    if not accessible(curve) then return nil, "restricted-curve" end
    if type(curve) ~= "table" and type(curve) ~= "userdata" then return nil, "invalid-curve" end
    return curve
end

local function addColorProbePoint(curve, point)
    local ok, color = pcall(CreateColor, point.r, point.g, point.b, point.a)
    if not ok then return { status = "color-construction-error" } end
    if not accessible(color) then return { status = "restricted-color" } end
    if type(color) ~= "table" and type(color) ~= "userdata" then return { status = "invalid-color" } end
    return colorMethod(curve, "AddPoint", point.x, color)
end

local function copyColorProbe(curve, point)
    local fn, ok = readField(curve, "Copy")
    if not ok or not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, curve))
    if not values[1] then return { status = "call-error" } end
    local copy = values[2]
    local result = { status = "observed", n = values.n - 1, values = {} }
    -- A copied curve is an owned method target, never a returned point to inspect.
    for index = 2, math.min(values.n, 17) do result.values[index - 1] = scalar(values[index]) end
    if values.n > 17 then result.truncated = true end
    if not accessible(copy) then result.status = "restricted-copy"; return result end
    if type(copy) ~= "table" and type(copy) ~= "userdata" then result.status = "invalid-copy"; return result end
    result.before = { original = colorSnapshot(curve), copy = colorSnapshot(copy) }
    result.add = addColorProbePoint(copy, point)
    result.afterAdd = { original = colorSnapshot(curve), copy = colorSnapshot(copy) }
    result.clear = colorMethod(copy, "ClearPoints")
    result.afterClear = { original = colorSnapshot(curve), copy = colorSnapshot(copy) }
    return result
end

local function captureColorCurves()
    local inputs = {
        { x = 0, r = 0, g = 1, b = 0.25, a = 1 },
        { x = 32, r = 1, g = 0, b = 0.25, a = 0.75 },
        { x = -16, r = 0, g = 0.25, b = 1, a = 0.5 },
        { x = 48, r = 1, g = 0.75, b = 0, a = 0.25 },
    }
    local curve, failure = createColorProbeCurve()
    if not curve then return { status = failure, inputs = inputs } end
    local result = { status = "observed", inputs = inputs, empty = colorSnapshot(curve), additions = {} }
    for _, point in ipairs(inputs) do
        local added = addColorProbePoint(curve, point)
        result.additions[#result.additions + 1] = added
        if added.status ~= "observed" then result.status = added.status; return result end
    end
    result.populated = colorSnapshot(curve)
    result.copyInput = { x = 16, r = 0.5, g = 0.25, b = 0.75, a = 0.5 }
    result.copy = copyColorProbe(curve, result.copyInput)
    result.defaults = { before = colorSnapshot(curve), reset = colorMethod(curve, "SetToDefaults") }
    result.defaults.after = colorSnapshot(curve)
    return result
end

local function captureSex()
    local result = { units = {}, enums = {} }
    local unitSexEnum = readField(Enum, "UnitSex")
    for _, name in ipairs({ "Male", "Female", "None", "Both", "Neutral" }) do
        local value, ok = readField(unitSexEnum, name)
        result.enums[name] = ok and scalar(value) or { status = "field-error" }
    end
    for _, unit in ipairs({ "player", "target", "focus", "pet", "party1", "party2",
        "nonexistent", "invalid-unit-token", "" }) do
        result.units[unit] = { exists = observe(UnitExists, unit),
            legacy = observe(UnitSex, unit), base = observe(UnitSexBase, unit) }
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

local function observeUnitTargetDisplay(unit)
    if not accessible(unit) then return { status = "restricted-input" } end
    local fn = UnitShouldDisplaySpellTargetName
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Function guards may revoke the token before invocation.
    if not accessible(unit) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, unit))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do
        result.values[index - 1] = scalar(values[index])
    end
    if values.n > 17 then result.truncated = true end
    return result
end

local function captureUnitTargetDisplay()
    local result = { units = {} }
    for _, unit in ipairs({ "player", "target", "focus", "party1", "nonexistent", "invalid-unit-token", "" }) do
        local row = { unit = scalar(unit), observations = {} }
        for index = 1, 2 do row.observations[index] = observeUnitTargetDisplay(unit) end
        result.units[#result.units + 1] = row
    end
    return result
end

local function observePublicQuery(namespace, name)
    local fn, ok = readField(namespace, name)
    if not ok then return { status = "field-error" } end
    return observeCast(fn)
end

local function capturePublicQueries()
    local result = { personalResourceDisplay = {}, omittedCompanion = {},
        housingMarketShopEnabled = {}, encounterTimelineCurrentTime = {},
        encounterLimitingResurrections = {}, encounterSuppressingRelease = {}, showTimelineForEncounter = {},
        activeOutfitID = {}, houseExteriorDoorHovered = {}, spellDiminishSystemSupported = {} }
    for index = 1, 2 do
        result.personalResourceDisplay[index] = observePublicQuery(C_GameRules, "IsPersonalResourceDisplayEnabled")
        result.omittedCompanion[index] = observePublicQuery(C_DelvesUI, "GetLockedTextForCompanion")
        result.housingMarketShopEnabled[index] = observePublicQuery(C_Housing, "IsHousingMarketShopEnabled")
        result.encounterTimelineCurrentTime[index] = observePublicQuery(C_EncounterTimeline, "GetCurrentTime")
        result.encounterLimitingResurrections[index] = observePublicQuery(C_InstanceEncounter, "IsEncounterLimitingResurrections")
        result.encounterSuppressingRelease[index] = observePublicQuery(C_InstanceEncounter, "IsEncounterSuppressingRelease")
        result.showTimelineForEncounter[index] = observePublicQuery(C_InstanceEncounter, "ShouldShowTimelineForEncounter")
        result.activeOutfitID[index] = observePublicQuery(C_TransmogOutfitInfo, "GetActiveOutfitID")
        result.houseExteriorDoorHovered[index] = observePublicQuery(C_HousingCustomizeMode, "IsHouseExteriorDoorHovered")
        result.spellDiminishSystemSupported[index] = observePublicQuery(C_SpellDiminish, "IsSystemSupported")
    end
    return result
end

local function mapTuple(...)
    local values = pack(...)
    local result = { n = values.n, values = {} }
    for index = 1, math.min(values.n, 16) do result.values[index] = scalar(values[index]) end
    if values.n > 16 then result.truncated = true end
    return result
end

local function inspectCurrentAura(entry)
    local result = scalar(entry)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    result.fields = {}
    for _, key in ipairs({ "auraInstanceID", "spellId", "applications" }) do
        local value, ok = readField(entry, key)
        result.fields[key] = ok and scalar(value) or { status = "field-error" }
    end
    return result
end

local function observeCurrentUnitAuras(...)
    local fn, ok = readField(C_UnitAuras, "GetUnitAuras")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, ...))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    local first = result.values[1]
    if first and first.status == "observed" and first.kind == "table" then
        first.entries = {}
        for index = 1, 8 do
            local entry, entryOK = readField(values[2], index)
            first.entries[index] = entryOK and inspectCurrentAura(entry) or { status = "field-error" }
        end
    end
    return result
end

local function captureCurrentUnitAuras()
    local result = {}
    -- Required filter intentionally omitted: negative case, not default semantics.
    result.omittedFilterNegative = observeCurrentUnitAuras("player")
    result.helpful = observeCurrentUnitAuras("player", "HELPFUL", 8)
    result.harmful = observeCurrentUnitAuras("player", "HARMFUL", 8)
    return result
end

local function auraNumberStatus(value)
    if not accessible(value) then return "restricted-input" end
    if type(value) ~= "number" or value ~= value or value == math.huge or value == -math.huge then
        return "unavailable-input"
    end
end

local function observeAuraCall(name, input, ...)
    if name ~= "GetAuraSlots" then
        local status = auraNumberStatus(input)
        if status then return { status = status } end
    end
    local fn, ok = readField(C_UnitAuras, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if name ~= "GetAuraSlots" then
        local status = auraNumberStatus(input)
        if status then return { status = status } end
    end
    local values = pack(pcall(fn, ...))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values
end

local function readAuraID(aura)
    if not accessible(aura) then return nil, "restricted-input" end
    if type(aura) ~= "table" and type(aura) ~= "userdata" then return nil, "unavailable-input" end
    local id, ok = readField(aura, "auraInstanceID")
    if not ok then return nil, "field-error" end
    local status = auraNumberStatus(id)
    if status then return nil, status end
    return id
end

local function observeAuraSlot(slot)
    local row = { slot = scalar(slot), queries = {} }
    local status = auraNumberStatus(slot)
    local values
    if status then row.producer = { status = status }
    else row.producer, values = observeAuraCall("GetAuraDataBySlot", slot, "player", slot) end
    local id
    status = "unavailable-input"
    if values then id, status = readAuraID(values[2]) end
    row.id = status and { status = status } or scalar(id)
    return row, id, status
end

local function captureAuraSlot(slot)
    local row, id, status = observeAuraSlot(slot)
    local variants = { {}, { 1 }, { 2 }, { 2, 5 }, { 1, 1 } }
    for index, args in ipairs(variants) do
        if status then row.queries[index] = { status = status }
        else
            row.queries[index] = observeAuraCall("GetAuraApplicationDisplayCount", id,
                "player", id, unpack(args))
        end
    end
    return row
end

local inspectDuration

local function captureAuraTimeSlot(slot)
    local row, id, status = observeAuraSlot(slot)
    for _, name in ipairs({ "DoesAuraHaveExpirationTime", "GetAuraBaseDuration",
        "GetRefreshExtendedDuration", "GetAuraDuration" }) do
        if status then row.queries[name] = { status = status }
        else
            local observation, values = observeAuraCall(name, id, "player", id)
            row.queries[name] = observation
            if name == "GetAuraDuration" and values then
                for index = 2, math.min(values.n, 17) do
                    observation.values[index - 1] = inspectDuration(values[index])
                end
            end
        end
    end
    return row
end

local function captureAuraPage(captureSlot)
    local producer, values = observeAuraCall("GetAuraSlots", nil, "player", "HELPFUL", 8)
    local result = { producer = producer, slots = {} }
    if not values then return result end
    -- First page only: continuation is recorded in the tuple, never followed.
    for index = 3, math.min(values.n, 10) do
        result.slots[#result.slots + 1] = captureSlot(values[index])
    end
    return result
end

local function observeSpellProducer(slot)
    local fn = GetActionInfo
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, slot))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values[2], values[3]
end

local function spellInputStatus(kind, id)
    if not accessible(kind) or not accessible(id) then return "restricted-input" end
    if type(kind) ~= "string" or kind ~= "spell" or type(id) ~= "number" then
        return "unavailable-input"
    end
    if id ~= id or id == math.huge or id == -math.huge then return "unavailable-input" end
end

local function observeSpellbookMetadata(kind, id, name)
    local status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local fn, ok = readField(C_SpellBook, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Lookup and function guards can revoke the original producer values.
    status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function captureSpellbookMetadata(slot)
    local identity, kind, id = observeSpellProducer(slot)
    local result = { slot = slot, identity = identity, queries = {} }
    for _, name in ipairs({ "FindBaseSpellByID", "FindFlyoutSlotBySpellID", "FindSpellOverrideByID" }) do
        result.queries[name] = identity.status == "observed" and observeSpellbookMetadata(kind, id, name)
            or { status = "unavailable-input" }
    end
    return result
end

local function observeSpellMetadata(kind, id, name)
    local status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local fn, ok = readField(C_Spell, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Lookup and function guards can revoke the original producer values.
    status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function inspectWeeklyProgressEntry(entry)
    local result = scalar(entry)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    result.fields = {}
    for _, key in ipairs({ "activityTierID", "difficulty", "numPoints" }) do
        local value, ok = readField(entry, key)
        result.fields[key] = ok and scalar(value) or { status = "field-error" }
    end
    return result
end

local function observeWeeklyProgress(name, combine)
    local enums, enumOK = readField(Enum, "WeeklyRewardChestThresholdType")
    local value, valueOK
    if enumOK then value, valueOK = readField(enums, name) end
    local input = valueOK and scalar(value) or { status = "unavailable-enum" }
    if input.status ~= "observed" or input.kind ~= "number" then
        return { status = "unavailable-enum" }
    end
    local fn, ok = readField(C_WeeklyRewards, "GetSortedProgressForActivity")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(value) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, value, combine))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status, result.input = "observed", input
    local first = result.values[1]
    if first and first.status == "observed" and first.kind == "table" then
        first.entries = {}
        for index = 1, 8 do
            local entry, entryOK = readField(values[2], index)
            first.entries[index] = entryOK and inspectWeeklyProgressEntry(entry) or { status = "field-error" }
        end
    end
    return result
end

local function captureWeeklyProgress()
    local result = { modes = {} }
    for _, name in ipairs({ "Raid", "Activities", "World", "RankedPvP", "Concession" }) do
        local row = { name = name, observations = {} }
        row.observations[1] = observeWeeklyProgress(name, false)
        row.observations[2] = observeWeeklyProgress(name, true)
        result.modes[#result.modes + 1] = row
    end
    return result
end

local function observeHousingPreviewMode(name)
    local enums, enumOK = readField(Enum, "HouseEditorMode")
    local value, valueOK
    if enumOK then value, valueOK = readField(enums, name) end
    local input = valueOK and scalar(value) or { status = "unavailable-enum" }
    if input.status ~= "observed" or input.kind ~= "number" then
        return { status = "unavailable-enum" }
    end
    local fn, ok = readField(C_HousingDecor, "IsModeDisabledForPreviewState")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Namespace lookup and function guards can revoke the published value.
    if not accessible(value) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, value))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status, result.input = "observed", input
    return result
end

local function captureHousingPreviewModes()
    local result = { modes = {} }
    for _, name in ipairs({ "BasicDecor", "ExpertDecor", "Customize", "Cleanup", "Layout", "ExteriorCustomization" }) do
        local row = { name = name, observations = {} }
        for index = 1, 2 do row.observations[index] = observeHousingPreviewMode(name) end
        result.modes[#result.modes + 1] = row
    end
    return result
end

local function observeSpellVisibility(kind, id, name)
    local status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local enums, enumOK = readField(Enum, "SpellAuraVisibilityType")
    local value, valueOK
    if enumOK then value, valueOK = readField(enums, name) end
    local input = valueOK and scalar(value) or { status = "unavailable-enum" }
    if input.status ~= "observed" or input.kind ~= "number" then
        return { status = "unavailable-enum" }
    end
    local fn, ok = readField(C_Spell, "GetVisibilityInfo")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Enum lookup, serialization and function guards can revoke producer inputs.
    status = spellInputStatus(kind, id)
    if status then return { status = status } end
    if not accessible(value) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, id, value))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status, result.input = "observed", input
    return result
end

local function observeAuraDefensive(kind, id)
    local status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local fn, ok = readField(C_UnitAuras, "AuraIsBigDefensive")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Namespace lookup and function guards can revoke the original inputs.
    status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function captureSpellMetadata(slot)
    local identity, kind, id = observeSpellProducer(slot)
    local result = { slot = slot, identity = identity, queries = {}, visibility = {} }
    for _, name in ipairs({ "GetSpellDisplayCount", "GetSpellMaxCumulativeAuraApplications",
        "IsConsumableSpell", "IsExternalDefensive", "IsPriorityAura", "IsSpellCrowdControl", "IsSpellImportant" }) do
        result.queries[name] = identity.status == "observed" and observeSpellMetadata(kind, id, name)
            or { status = "unavailable-input" }
    end
    for _, name in ipairs({ "RaidInCombat", "RaidOutOfCombat", "EnemyTarget" }) do
        result.visibility[name] = identity.status == "observed" and observeSpellVisibility(kind, id, name)
            or { status = "unavailable-input" }
    end
    result.auraQueries = {
        AuraIsBigDefensive = identity.status == "observed" and observeAuraDefensive(kind, id)
            or { status = "unavailable-input" },
    }
    return result
end

local function observeItemProducer(slot)
    local fn = GetInventoryItemLink
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, "player", slot))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values[2]
end

local function observeItemBinding(link)
    if not accessible(link) then return { status = "restricted-input" } end
    if type(link) ~= "string" then return { status = "unavailable-input" } end
    local fn, ok = readField(C_Item, "IsItemBindToAccount")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Namespace lookup and function guards may revoke input access.
    if not accessible(link) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, link))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function captureItemBinding()
    local result = { slots = {} }
    for slot = 1, 19 do
        local producer, link = observeItemProducer(slot)
        local binding = { status = "unavailable-input" }
        if producer.status == "observed" then binding = observeItemBinding(link) end
        result.slots[slot] = { slot = slot, producer = producer, binding = binding }
    end
    return result
end

local function observeFillMethod(object, name, ...)
    local fn, ok = readField(object, name)
    if not ok then return { status = "field-error" } end
    if not accessible(object) then return { status = "restricted-object" } end
    return observeCast(fn, object, ...)
end

local function captureFillStyles(object)
    local result = {}
    for _, name in ipairs({ "Standard", "StandardNoRangeFill", "Center", "Reverse" }) do
        local enums, enumOK = readField(Enum, "StatusBarFillStyle")
        local value, valueOK
        if enumOK then value, valueOK = readField(enums, name) end
        local input = valueOK and scalar(value) or { status = "field-error" }
        local row = { name = name, input = input }
        result[#result + 1] = row
        if input.status == "observed" and (input.kind == "number" or input.kind == "string"
            or input.kind == "boolean") and accessible(value) then
            row.setter = observeFillMethod(object, "SetFillStyle", value)
            row.getters = {}
            for index = 1, 2 do row.getters[index] = observeFillMethod(object, "GetFillStyle") end
        end
    end
    return result
end

local function captureStatusbarFill()
    local fn, parent = CreateFrame, UIParent
    if not accessible(fn) or not accessible(parent) then
        return { constructor = { status = "restricted" } }
    end
    if type(fn) ~= "function" then return { constructor = { status = "missing-api" } } end
    local values = pack(pcall(fn, "StatusBar", nil, parent))
    if not values[1] then return { constructor = { status = "call-error" } } end
    local object = values[2]
    -- Hide before recording constructor results or inspecting fill state.
    local hidden = observeFillMethod(object, "Hide")
    local constructor = mapTuple(unpack(values, 2, values.n))
    constructor.status = "observed"
    local row = { constructor = constructor, hide = hidden }
    if hidden.status ~= "observed" then
        row.status = "visibility-unconfirmed"
        return row
    end
    row.default = observeFillMethod(object, "GetFillStyle")
    row.styles = captureFillStyles(object)
    return row
end

local function captureHealCalculatorObject()
    local fn = CreateUnitHealPredictionCalculator
    if not accessible(fn) then return { constructor = { status = "restricted" } } end
    if type(fn) ~= "function" then return { constructor = { status = "missing-api" } } end
    local values = pack(pcall(fn))
    if not values[1] then return { constructor = { status = "call-error" } } end
    local observation = mapTuple(unpack(values, 2, values.n))
    observation.status = "observed"
    local row = { constructor = observation }
    local object = values[2]
    if not accessible(object) then return row end
    if type(object) ~= "table" and type(object) ~= "userdata" then return row end
    row.methods = {}
    for _, name in ipairs({ "GetHealAbsorbMode", "GetHealAbsorbClampMode",
        "GetDamageAbsorbClampMode", "GetHealAbsorbs", "GetDamageAbsorbs" }) do
        local samples = {}
        row.methods[name] = samples
        for index = 1, 2 do
            local getter, ok = readField(object, name)
            samples[index] = ok and observeCast(getter, object) or { status = "field-error" }
        end
    end
    return row
end

local function captureHealCalculator()
    local result = { objects = {} }
    for index = 1, 2 do result.objects[index] = captureHealCalculatorObject() end
    return result
end

local function mapCallbackResult(mode)
    if mode == "multiple" then return "first", nil, "third" end
    if mode == "nil" then return nil end
    if mode == "zero" then return end
    if mode == "throw" then error({}) end
    return "mapped"
end

local function captureMapValues()
    local fn = mapvalues
    if not accessible(fn) then return { status = "restricted" } end
    if type(fn) ~= "function" then return { status = "missing-api" } end
    local cases = {
        { id = "zero-inputs", inputs = pack(), callback = "single" },
        { id = "one-input", inputs = pack(17), callback = "single" },
        { id = "multiple-inputs", inputs = pack(17, "two", false), callback = "single" },
        { id = "interior-nil", inputs = pack(17, nil, "tail"), callback = "single" },
        { id = "trailing-nils", inputs = pack(17, nil, nil), callback = "single" },
        { id = "multiple-returns", inputs = pack(17, "two"), callback = "multiple" },
        { id = "nil-return", inputs = pack(17, "two"), callback = "nil" },
        { id = "zero-returns", inputs = pack(17, "two"), callback = "zero" },
        { id = "opaque-throw", inputs = pack(17), callback = "throw" },
    }
    local result = { status = "observed", cases = {}, invocationCount = 0 }
    for _, case in ipairs(cases) do
        local row = { id = case.id, callback = case.callback, invocations = {},
            inputs = mapTuple(unpack(case.inputs, 1, case.inputs.n)) }
        local active = true
        local function callback(...)
            if not active then error({}) end
            if result.invocationCount >= 32 then
                result.status = "invocation-limit"
                error({})
            end
            result.invocationCount = result.invocationCount + 1
            row.invocations[#row.invocations + 1] = mapTuple(...)
            -- Return only fixed ordinary literals, never compute from callback arguments.
            return mapCallbackResult(case.callback)
        end
        row.output = observeCast(fn, callback, unpack(case.inputs, 1, case.inputs.n))
        active = false
        result.cases[#result.cases + 1] = row
        if result.status == "invocation-limit" then break end
    end
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

local durationMethods = {
    "GetTotalDuration", "GetElapsedDuration", "GetRemainingDuration", "GetElapsedPercent",
    "GetRemainingPercent", "GetStartTime", "GetEndTime", "GetClockTime", "GetModRate", "HasExpired",
}
local previousDurations = {}
local durationCapture = 0

local function observeDurationMethod(object, name)
    local fn, ok = readField(object, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(object) then return { status = "restricted-object" } end
    local values = pack(pcall(fn, object))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

inspectDuration = function(object)
    local result = scalar(object)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    -- Controlled producer-object experiment, never a passive payload observer.
    result.methods = {}
    for _, name in ipairs(durationMethods) do
        result.methods[name] = observeDurationMethod(object, name)
    end
    return result
end

local function observeSpellDuration(kind, id, name)
    local status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local fn, ok = readField(C_Spell, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Recheck original inputs after namespace lookup and function access checks.
    status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do
        result.values[index - 1] = inspectDuration(values[index])
    end
    if values.n > 17 then result.truncated = true end
    return result
end

local function observeSpellbookPair(kind, id)
    local status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local fn, ok = readField(C_SpellBook, "FindSpellBookSlotForSpell")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    status = spellInputStatus(kind, id)
    if status then return { status = status } end
    local values = pack(pcall(fn, id, false, true, true, true))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values[2], values[3]
end

local function spellbookPairStatus(slot, bank)
    if not accessible(slot) or not accessible(bank) then return "restricted-input" end
    if type(slot) ~= "number" or type(bank) ~= "number" then return "unavailable-input" end
    if slot ~= slot or slot == math.huge or slot == -math.huge
        or bank ~= bank or bank == math.huge or bank == -math.huge then
        return "unavailable-input"
    end
end

local function observeSpellbookDuration(slot, bank, name)
    local status = spellbookPairStatus(slot, bank)
    if status then return { status = status } end
    local fn, ok = readField(C_SpellBook, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    status = spellbookPairStatus(slot, bank)
    if status then return { status = status } end
    local values
    if name == "GetSpellBookItemCooldownDuration" then values = pack(pcall(fn, slot, bank, false))
    else values = pack(pcall(fn, slot, bank)) end
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do
        result.values[index - 1] = inspectDuration(values[index])
    end
    if values.n > 17 then result.truncated = true end
    return result
end

local function captureSpellbookDuration(slot)
    local identity, kind, id = observeSpellProducer(slot)
    local result = { slot = slot, identity = identity, queries = {} }
    local bookSlot, bank
    if identity.status == "observed" then result.producer, bookSlot, bank = observeSpellbookPair(kind, id)
    else result.producer = { status = "unavailable-input" } end
    for _, name in ipairs({ "GetSpellBookItemChargeDuration", "GetSpellBookItemCooldownDuration",
        "GetSpellBookItemLossOfControlCooldownDuration" }) do
        result.queries[name] = result.producer.status == "observed" and observeSpellbookDuration(bookSlot, bank, name)
            or { status = "unavailable-input" }
    end
    return result
end

local function captureSpellDuration(slot)
    local identity, kind, id = observeSpellProducer(slot)
    local result = { slot = slot, identity = identity, queries = {} }
    for _, name in ipairs({ "GetSpellChargeDuration", "GetSpellLossOfControlCooldownDuration" }) do
        result.queries[name] = identity.status == "observed" and observeSpellDuration(kind, id, name)
            or { status = "unavailable-input" }
    end
    return result
end

local function queryDuration(fn, retained, counters, ...)
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn, ...))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do
        local object = values[index]
        local item = inspectDuration(object)
        result.values[index - 1] = item
        if item.methods then
            counters.objects = counters.objects + 1
            item.observationRef = durationCapture .. ":" .. counters.objects
            if #retained < 28 then
                retained[#retained + 1] = { object = object, observationRef = item.observationRef }
                item.retention = "next-capture"
            else
                counters.dropped = counters.dropped + 1
                item.retention = "limit"
            end
        end
    end
    if values.n > 17 then result.truncated = true end
    return result
end

local function captureCastDurations()
    durationCapture = durationCapture + 1
    local result = { capture = durationCapture, previous = {}, units = {} }
    local old = previousDurations
    previousDurations = {}
    for _, previous in ipairs(old) do
        result.previous[#result.previous + 1] = { observationRef = previous.observationRef,
            observation = inspectDuration(previous.object) }
    end
    local counters = { objects = 0, dropped = 0 }
    for _, unit in ipairs({ "player", "target", "focus", "party1", "nonexistent", "invalid-unit-token", "" }) do
        result.units[unit] = {
            casting = queryDuration(UnitCastingDuration, previousDurations, counters, unit),
            channel = queryDuration(UnitChannelDuration, previousDurations, counters, unit),
            empowerDefault = queryDuration(UnitEmpoweredChannelDuration, previousDurations, counters, unit),
            empowerFalse = queryDuration(UnitEmpoweredChannelDuration, previousDurations, counters, unit, false),
            empowerTrue = queryDuration(UnitEmpoweredChannelDuration, previousDurations, counters, unit, true),
        }
    end
    result.retainedCount, result.retentionDropped = #previousDurations, counters.dropped
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

local numericInputs = {
    -2.5, -1.5, -0.5, 0, 0.5, 1.5, 2.5,
    -0.500001, -0.499999, 0.499999, 0.500001,
    -1, 1, -100, 100, -0.1, 0.1, -0.9, 0.9,
    -1234.5678, 1234.5678, -1e6, 1e6, -1e12, 1e12,
}

local function captureRaidMarkers()
    local result = { units = {}, markers = {}, enabled = {} }
    for _, unit in ipairs({ "player", "target", "focus", "pet", "party1", "party2",
        "nonexistent", "invalid-unit-token", "" }) do
        local row = { unit = unit, observations = {} }
        for attempt = 1, 2 do
            row.observations[attempt] = observeCast(CanBeRaidTarget, unit)
        end
        result.units[#result.units + 1] = row
    end
    for index = 1, 8 do
        local row = { index = index, observations = {} }
        for attempt = 1, 2 do
            row.observations[attempt] = observeCast(IsRaidMarkerActive, index)
        end
        result.markers[index] = row
    end
    for attempt = 1, 2 do
        result.enabled[attempt] = observeCast(IsRaidMarkerSystemEnabled)
    end
    return result
end

local function captureAbbreviations()
    local result = { locale = observeCast(GetLocale), samples = {} }
    for _, input in ipairs(numericInputs) do
        result.samples[#result.samples + 1] = { input = input,
            large = observeCast(AbbreviateLargeNumbers, input),
            small = observeCast(AbbreviateNumbers, input) }
    end
    return result
end

local function captureNumbers()
    local floor, floorOK = readField(C_StringUtil, "FloorToNearestString")
    local round, roundOK = readField(C_StringUtil, "RoundToNearestString")
    if not floorOK then floor = nil end
    if not roundOK then round = nil end
    local result = { locale = observe(GetLocale), samples = {} }
    for _, input in ipairs(numericInputs) do
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
    if mode == "actions" or mode == "spell-metadata" or mode == "spell-duration" or mode == "spellbook-metadata" or mode == "spellbook-duration" then
        slot, label = parseActionSlot(label)
        if slot == nil then print("Usage: /apicontract " .. mode .. " <integer-slot> <label>"); return end
    end
    if mode ~= "weekly-progress" and mode ~= "housing-preview-modes" and mode ~= "spellbook-duration" and mode ~= "spellbook-metadata" and mode ~= "unit-target-display" and mode ~= "unit-auras-current" and mode ~= "aura-time" and mode ~= "aura-display-count" and mode ~= "spell-duration" and mode ~= "spell-metadata" and mode ~= "public-queries" and mode ~= "item-binding" and mode ~= "statusbar-fill" and mode ~= "raid-markers" and mode ~= "abbreviations" and mode ~= "heal-calculator" and mode ~= "mapvalues" and mode ~= "cast-durations" and mode ~= "color-curves" and mode ~= "curve-edit" and mode ~= "curve-state" and mode ~= "resources" and mode ~= "hyperlinks" and mode ~= "actions" and mode ~= "all" and mode ~= "curves" and mode ~= "sex" and mode ~= "names" and mode ~= "numbers" and mode ~= "casts" and mode ~= "publication" then
        print("Usage: /apicontract [all|curves|curve-state|curve-edit|color-curves|sex|names|numbers|casts|cast-durations|resources|hyperlinks|mapvalues|heal-calculator|abbreviations|raid-markers|statusbar-fill|item-binding|public-queries|aura-display-count|aura-time|unit-auras-current|unit-target-display|spellbook-metadata|spellbook-duration|housing-preview-modes|weekly-progress|publication|events-start|events-stop|callbacks-start|callbacks-stop] [label]")
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
        if mode == "weekly-progress" then record.weeklyProgress = captureWeeklyProgress() end
        if mode == "housing-preview-modes" then record.housingPreviewModes = captureHousingPreviewModes() end
        if mode == "unit-target-display" then record.unitTargetDisplay = captureUnitTargetDisplay() end
        if mode == "unit-auras-current" then record.unitAurasCurrent = captureCurrentUnitAuras() end
        if mode == "aura-display-count" then record.auraDisplayCount = captureAuraPage(captureAuraSlot) end
        if mode == "aura-time" then record.auraTime = captureAuraPage(captureAuraTimeSlot) end
        if mode == "spell-duration" then record.spellDuration = captureSpellDuration(slot) end
        if mode == "spell-metadata" then record.spellMetadata = captureSpellMetadata(slot) end
        if mode == "spellbook-metadata" then record.spellbookMetadata = captureSpellbookMetadata(slot) end
        if mode == "spellbook-duration" then record.spellbookDuration = captureSpellbookDuration(slot) end
        if mode == "public-queries" then record.publicQueries = capturePublicQueries() end
        if mode == "item-binding" then record.itemBinding = captureItemBinding() end
        if mode == "statusbar-fill" then record.statusbarFill = captureStatusbarFill() end
        if mode == "raid-markers" then record.raidMarkers = captureRaidMarkers() end
        if mode == "abbreviations" then record.abbreviations = captureAbbreviations() end
        if mode == "heal-calculator" then record.healCalculator = captureHealCalculator() end
        if mode == "mapvalues" then record.mapvalues = captureMapValues() end
        if mode == "cast-durations" then record.castDurations = captureCastDurations() end
        if mode == "color-curves" then record.colorCurves = captureColorCurves() end
        if mode == "curve-edit" then record.curveEdit = captureCurveEdit() end
        if mode == "curve-state" then record.curveState = captureCurveState() end
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
