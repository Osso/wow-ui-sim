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

local function observeUnitRolePredicate(unit, name)
    if not accessible(unit) then return { status = "restricted-input" } end
    local fn, ok = readField(_G, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Lookup and function guards may revoke the token before invocation.
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

local inspectDuration

local function observeEmpoweredStages(unit, name, ...)
    if not accessible(unit) then return { status = "restricted-input" } end
    local fn, ok = readField(_G, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(unit) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, unit, ...))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do
        result.values[index - 1] = scalar(values[index])
    end
    if values.n > 17 then result.truncated = true end
    local list = values[2]
    if accessible(list) and type(list) == "table" then
        result.entries = {}
        for index = 1, 8 do
            local value, entryOK = readField(list, index)
            if not entryOK then result.entries[index] = { status = "field-error" }
            elseif name == "UnitEmpoweredStageDurations" then result.entries[index] = inspectDuration(value)
            else result.entries[index] = scalar(value) end
        end
    end
    return result
end

local function captureEmpoweredStages()
    local result = { units = {} }
    for _, unit in ipairs({ "player", "target", "focus", "party1", "nonexistent", "invalid-unit-token", "" }) do
        local row = { unit = scalar(unit), queries = {} }
        row.queries.durations = observeEmpoweredStages(unit, "UnitEmpoweredStageDurations")
        row.queries.percentagesWithoutHold = observeEmpoweredStages(unit, "UnitEmpoweredStagePercentages", false)
        row.queries.percentagesWithHold = observeEmpoweredStages(unit, "UnitEmpoweredStagePercentages", true)
        result.units[#result.units + 1] = row
    end
    return result
end

local function captureFullNames()
    local result = { units = {} }
    for _, unit in ipairs({ "player", "target", "focus", "pet", "party1", "nonexistent", "invalid-unit-token", "" }) do
        result.units[#result.units + 1] = {
            unit = scalar(unit), result = observeUnitRolePredicate(unit, "UnitFullName"),
        }
    end
    return result
end

local function captureUnitRolePredicates()
    local function observeOmission(explicitNil)
        if explicitNil and not accessible(nil) then return { status = "restricted-input" } end
        local fn, ok = readField(_G, "UnitIsNPCAsPlayer")
        if not ok then return { status = "field-error" } end
        if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
        -- Function guards may revoke the explicit nil before forwarding.
        if explicitNil and not accessible(nil) then return { status = "restricted-input" } end
        local values
        if explicitNil then values = pack(pcall(fn, nil)) else values = pack(pcall(fn)) end
        if not values[1] then return { status = "call-error" } end
        local observation = { status = "observed", n = values.n - 1, values = {} }
        for index = 2, math.min(values.n, 17) do
            observation.values[index - 1] = scalar(values[index])
        end
        if values.n > 17 then observation.truncated = true end
        return observation
    end
    local result = { units = {} }
    for _, unit in ipairs({ "player", "target", "focus", "pet", "party1", "nonexistent", "invalid-unit-token", "" }) do
        local row = { unit = scalar(unit), queries = {} }
        for _, name in ipairs({ "UnitIsLieutenant", "UnitIsMinion", "UnitIsNPCAsPlayer" }) do
            row.queries[name] = observeUnitRolePredicate(unit, name)
        end
        result.units[#result.units + 1] = row
    end
    result.omittedInput = {}
    result.omittedInput.omitted = observeOmission(false)
    result.omittedInput.explicitNil = observeOmission(true)
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

local function inspectTrainingGround(object)
    local result = scalar(object)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    result.fields = {}
    for _, key in ipairs({ "name", "icon", "gameType", "shortDescription", "longDescription", "mapDescription",
        "maxPlayers", "battlegroundID", "lfgDungeonID", "mapID", "isHoliday", "isRandom", "canEnter", "isTrainingGround" }) do
        local value, ok = readField(object, key)
        result.fields[key] = ok and scalar(value) or { status = "field-error" }
    end
    return result
end

local function observeTrainingGrounds()
    local fn, ok = readField(C_PvP, "GetTrainingGrounds")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn))
    if not values[1] then return { status = "call-error" } end
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 17) do result.values[index - 1] = scalar(values[index]) end
    if values.n > 17 then result.truncated = true end
    local first = result.values[1]
    if first and first.status == "observed" and first.kind == "table" then
        first.entries = {}
        for index = 1, 8 do
            local entry, entryOK = readField(values[2], index)
            first.entries[index] = entryOK and inspectTrainingGround(entry) or { status = "field-error" }
        end
    end
    return result
end

local function captureTrainingGroundsStructures()
    local grounds = observeTrainingGrounds()
    -- Five declared returns: honor, experience, item rewards, currency rewards, role bonus.
    -- observePublicQuery keeps the three nested reward values opaque.
    local rewards = observePublicQuery(C_PvP, "GetRandomTrainingGroundRewards")
    return { grounds = grounds, rewards = rewards }
end

local function captureDeathRecapCurrent()
    local result = {}
    for _, name in ipairs({ "GetRecapEvents", "GetRecapLink", "HasRecapEvents" }) do
        result[name] = {}
        for index = 1, 2 do
            result[name][index] = observePublicQuery(C_DeathRecap, name)
        end
    end
    return result
end

local function capturePlayerStateQueries()
    local result = {}
    for _, name in ipairs({ "GetCollapsingStarCost", "ShowingCloak", "ShowingHelm" }) do
        result[name] = {}
        for index = 1, 2 do
            result[name][index] = observePublicQuery(_G, name)
        end
    end
    return result
end

do
    -- Keep the lock in the addon session, not in resettable SavedVariables.
    local transitionBlocked = false
    local captureReadOnly = capturePlayerStateQueries

    local function usableBoolean(value)
        return accessible(value) and type(value) == "boolean"
    end

    local function invoke(fn, ...)
        local values = pack(pcall(fn, ...))
        if not values[1] then return { status = "call-error" } end
        local result = { status = "observed", n = values.n - 1, values = {} }
        for index = 2, math.min(values.n, 17) do
            result.values[index - 1] = scalar(values[index])
        end
        if values.n > 17 then result.truncated = true end
        return result, values[2]
    end

    local function readState(name)
        local fn, ok = readField(_G, name)
        if not ok then return { status = "field-error" } end
        if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
        return invoke(fn)
    end

    local function setState(name, value, baseline)
        if not usableBoolean(baseline) then return { status = "restricted-baseline" }, false end
        local fn, ok = readField(_G, name)
        if not ok then return { status = "field-error" }, false end
        if not accessible(fn) or type(fn) ~= "function" then
            return { status = "missing-api" }, false
        end
        -- Lookup/function/argument guards may revoke the original restoration value.
        if not usableBoolean(value) then return { status = "restricted-argument" }, false end
        if not usableBoolean(baseline) then return { status = "restricted-baseline" }, false end
        local result = invoke(fn, value)
        return result, true
    end

    local function restoreState(getter, setter, baseline)
        local result = {}
        local attempted
        result.setter, attempted = setState(setter, baseline, baseline)
        local final
        result.read, final = readState(getter)
        if not attempted then result.status = "restoration-skipped"
        elseif result.setter.status == "call-error" then result.status = "restoration-error"
        else
            result.status = "restoration-unconfirmed"
            if result.read.status == "observed" and usableBoolean(final) and usableBoolean(baseline)
                and accessible(final) and accessible(baseline) and final == baseline then
                -- Equality is a captured observation, not a native restoration guarantee.
                result.status = "confirmed-by-observation"
            end
        end
        return result
    end

    local function captureLane(getter, setter)
        local row = { steps = {}, restoration = { status = "not-needed" } }
        local baseline
        row.baseline, baseline = readState(getter)
        if row.baseline.status ~= "observed" or not usableBoolean(baseline) then
            row.status = "transition-unavailable"
            return row, false
        end
        local attempted = false
        for index, value in ipairs({ false, true }) do
            local step, invoked = {}, false
            step.setter, invoked = setState(setter, value, baseline)
            attempted = attempted or invoked
            step.read = invoked and readState(getter) or { status = "skipped" }
            row.steps[index] = step
        end
        if attempted then
            -- Native setter errors are contained; cleanup still runs after every attempted sequence.
            row.restoration = restoreState(getter, setter, baseline)
        end
        local unconfirmed = attempted and row.restoration.status ~= "confirmed-by-observation"
        row.status = unconfirmed and "restoration-unconfirmed" or "observed"
        return row, unconfirmed
    end

    capturePlayerStateQueries = function(mode)
        if mode ~= "cloak-helm-transition" then return captureReadOnly() end
        if transitionBlocked then return { status = "blocked-restoration-unconfirmed" } end
        local result = {}
        local cloakUnconfirmed, helmUnconfirmed
        result.cloak, cloakUnconfirmed = captureLane("ShowingCloak", "ShowCloak")
        result.helm, helmUnconfirmed = captureLane("ShowingHelm", "ShowHelm")
        transitionBlocked = cloakUnconfirmed or helmUnconfirmed
        result.status = transitionBlocked and "restoration-unconfirmed" or "observed"
        return result
    end
end

local function captureStableBonusSlot()
    local result = { IsBonusPetSlotAvailable = {} }
    for index = 1, 2 do
        result.IsBonusPetSlotAvailable[index] = observePublicQuery(C_StableInfo, "IsBonusPetSlotAvailable")
    end
    return result
end

local function captureTrainingGroundsState()
    local result = {}
    for _, name in ipairs({ "AreTrainingGroundsEnabled", "CanPlayerUseTrainingGroundsUI",
        "HasRandomTrainingGroundWinToday" }) do
        result[name] = {}
        for index = 1, 2 do
            result[name][index] = observePublicQuery(C_PvP, name)
        end
    end
    return result
end

local function captureNeighborhoodState()
    local result = {}
    for _, name in ipairs({ "GetActiveNeighborhood", "GetRequiredLevel", "IsInitiativeEnabled",
        "IsPlayerInNeighborhoodGroup", "IsViewingActiveNeighborhood", "PlayerHasInitiativeAccess",
        "PlayerMeetsRequiredLevel" }) do
        result[name] = {}
        for index = 1, 2 do
            result[name][index] = observePublicQuery(C_NeighborhoodInitiative, name)
        end
    end
    return result
end

local function captureOutfitState()
    local result = {}
    for _, name in ipairs({ "GetCurrentlyViewedOutfitID", "GetMaxNumberOfUsableOutfits", "GetNextOutfitCost",
        "GetPendingTransmogCost", "HasPendingOutfitSituations", "IsEquippedGearOutfitDisplayed",
        "IsEquippedGearOutfitLocked" }) do
        result[name] = {}
        for index = 1, 2 do
            result[name][index] = observePublicQuery(C_TransmogOutfitInfo, name)
        end
    end
    return result
end

local function capturePublicQueries(mode)
    if mode == "expansion-audio-fields" then
        local function observeExpansion(name)
            local expansion, published = readField(_G, name)
            local input = published and scalar(expansion) or { status = "unavailable-input" }
            if input.status ~= "observed" or input.kind ~= "number" then
                return { status = "unavailable-input" }
            end
            local fn, ok = readField(_G, "GetExpansionDisplayInfo")
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            -- Global lookup and function guards can revoke the original published value.
            if not accessible(expansion) then return { status = "restricted-input" } end
            local values = pack(pcall(fn, expansion))
            if not values[1] then return { status = "call-error" } end
            local result = { status = "observed", input = input, n = values.n - 1, values = {} }
            for index = 2, math.min(values.n, 17) do
                result.values[index - 1] = scalar(values[index])
            end
            if values.n > 17 then result.truncated = true end
            local first = result.values[1]
            if first and first.status == "observed" and (first.kind == "table" or first.kind == "userdata") then
                first.fields = {}
                for _, key in ipairs({ "glueAmbianceSoundKit", "glueCreditsSoundKit", "glueMusicSoundKit" }) do
                    local value, fieldOK = readField(values[2], key)
                    first.fields[key] = fieldOK and scalar(value) or { status = "field-error" }
                end
            end
            return result
        end
        local result = { expansions = {} }
        for _, name in ipairs({ "LE_EXPANSION_CLASSIC", "LE_EXPANSION_LEVEL_CURRENT" }) do
            local row = { name = name, observations = {} }
            for attempt = 1, 2 do row.observations[attempt] = observeExpansion(name) end
            result.expansions[#result.expansions + 1] = row
        end
        return result
    end
    if mode == "combat-audio-settings-read" then
        local function observeSetting(api, enumName, name)
            local value, input
            if enumName then
                local enums = readField(_G, "Enum")
                local members = readField(enums, enumName)
                local ok
                value, ok = readField(members, name)
                input = ok and scalar(value) or { status = "unavailable-enum" }
                if input.status ~= "observed" or input.kind ~= "number" then
                    return { status = "unavailable-enum" }
                end
            end
            local namespace = readField(_G, "C_CombatAudioAlert")
            local fn, ok = readField(namespace, api)
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            local values
            if enumName then
                -- Lookup and function guards may revoke the original published value.
                if not accessible(value) then return { status = "restricted-input" } end
                values = pack(pcall(fn, value))
            else
                values = pack(pcall(fn))
            end
            if not values[1] then return { status = "call-error" } end
            local result = { status = "observed", n = values.n - 1, values = {}, input = input }
            for index = 2, math.min(values.n, 17) do result.values[index - 1] = scalar(values[index]) end
            if values.n > 17 then result.truncated = true end
            return result
        end
        local result = { IsEnabled = {}, specSettings = {}, throttles = {} }
        for index = 1, 2 do result.IsEnabled[index] = observeSetting("IsEnabled") end
        local groups = {
            { api = "GetSpecSetting", enum = "CombatAudioAlertSpecSetting", rows = result.specSettings,
                names = { "Resource1Percent", "Resource1Format", "Resource1Voice", "Resource1Volume",
                    "Resource2Percent", "Resource2Format", "Resource2Voice", "Resource2Volume", "SayIfTargeted" } },
            { api = "GetThrottle", enum = "CombatAudioAlertThrottle", rows = result.throttles,
                names = { "Sample", "PlayerHealth", "TargetHealth", "PlayerCast", "TargetCast",
                    "PlayerResource1", "PlayerResource2", "PlayerHealthSamePercent", "TargetHealthSamePercent",
                    "PlayerResource1SamePercent", "PlayerResource2SamePercent" } },
        }
        for _, group in ipairs(groups) do
            for _, name in ipairs(group.names) do
                local row = { name = name, observations = {} }
                for index = 1, 2 do row.observations[index] = observeSetting(group.api, group.enum, name) end
                group.rows[#group.rows + 1] = row
            end
        end
        return result
    end
    if mode == "encounter-warning-state" then
        local result = { IsFeatureAvailable = {}, IsFeatureEnabled = {} }
        for index = 1, 2 do
            result.IsFeatureAvailable[index] = observePublicQuery(C_EncounterWarnings, "IsFeatureAvailable")
            result.IsFeatureEnabled[index] = observePublicQuery(C_EncounterWarnings, "IsFeatureEnabled")
        end
        return result
    end
    if mode == "ping-enabled" then
        local result = { IsPingSystemEnabled = {} }
        for index = 1, 2 do
            result.IsPingSystemEnabled[index] = observePublicQuery(C_Ping, "IsPingSystemEnabled")
        end
        return result
    end
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

local function observeGuidIdentity(name, input)
    if not accessible(input) then return { status = "restricted-input" } end
    if type(input) ~= "string" then return { status = "unavailable-input" } end
    local fn, ok = readField(_G, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Global lookup and function guards may revoke the original token or GUID.
    if not accessible(input) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, input))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values[2]
end

local function captureGuidIdentity()
    local result = { units = {} }
    for _, unit in ipairs({ "player", "target", "party1" }) do
        local row = { unit = scalar(unit), queries = {} }
        local guid
        row.producer, guid = observeGuidIdentity("UnitGUID", unit)
        for _, name in ipairs({ "UnitClassFromGUID", "UnitNameFromGUID" }) do
            row.queries[name] = observeGuidIdentity(name, guid)
        end
        result.units[#result.units + 1] = row
    end
    return result
end

local function customSetInputStatus(value, expected)
    if not accessible(value) then return "restricted-input" end
    if type(value) ~= expected then return "unavailable-input" end
    if expected == "number" and (value ~= value or value == math.huge or value == -math.huge) then
        return "unavailable-input"
    end
end

local function observeCustomSetCall(name, input, expected)
    if expected then
        local status = customSetInputStatus(input, expected)
        if status then return { status = status } end
    end
    local fn, ok = readField(C_TransmogCollection, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values
    if expected then
        local status = customSetInputStatus(input, expected)
        if status then return { status = status } end
        values = pack(pcall(fn, input))
    else values = pack(pcall(fn)) end
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values
end

local function inspectHousingCatalogFields(object, keys)
    local result = scalar(object)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    result.fields = {}
    for _, key in ipairs(keys) do
        local value, ok = readField(object, key)
        result.fields[key] = ok and scalar(value) or { status = "field-error" }
    end
    return result
end

local function observeCooldownViewerCall(name, input, categoryQuery)
    local status = customSetInputStatus(input, "number")
    if status then return { status = status } end
    local fn, ok = readField(C_CooldownViewer, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    status = customSetInputStatus(input, "number")
    if status then return { status = status } end
    local values
    if categoryQuery then values = pack(pcall(fn, input, false))
    else values = pack(pcall(fn, input)) end
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values
end

local function inspectCooldownViewerAlerts(object)
    local result = scalar(object)
    if result.status ~= "observed" or result.kind ~= "table" then return result end
    result.entries = {}
    for index = 1, 8 do
        local value, ok = readField(object, index)
        result.entries[index] = ok and scalar(value) or { status = "field-error" }
    end
    return result
end

local function captureCooldownViewerCategory(name)
    local result = { name = name, items = {} }
    local enums, enumOK = readField(Enum, "CooldownViewerCategory")
    local category, ok
    if enumOK then category, ok = readField(enums, name) end
    if not ok or customSetInputStatus(category, "number") then
        result.producer = { status = "unavailable-enum" }
        return result
    end
    local values
    result.producer, values = observeCooldownViewerCall("GetCooldownViewerCategorySet", category, true)
    if not values then return result end
    local list = values[2]
    if not accessible(list) or type(list) ~= "table" then return result end
    for index = 1, 8 do
        local id, idOK = readField(list, index)
        local item = { id = idOK and scalar(id) or { status = "field-error" } }
        result.items[index] = item
        if not idOK then
            item.info, item.alerts = { status = "field-error" }, { status = "field-error" }
        else
            local info, alerts
            item.info, info = observeCooldownViewerCall("GetCooldownViewerCooldownInfo", id)
            if info then item.info.object = inspectHousingCatalogFields(info[2], { "cooldownID", "category" }) end
            item.alerts, alerts = observeCooldownViewerCall("GetValidAlertTypes", id)
            if alerts then
                local observation = inspectCooldownViewerAlerts(alerts[2])
                item.alerts.object = observation
                item.alerts.entries = observation.entries
            end
        end
    end
    return result
end

local function captureCooldownViewerRead()
    local result = { categories = {} }
    for _, name in ipairs({ "Essential", "Utility", "TrackedBuff", "TrackedBar", "GroupBuff",
        "SpecAgnosticEssential", "SpecAgnosticTracked", "EquipSlotEssential", "EquipSlotTracked" }) do
        result.categories[#result.categories + 1] = captureCooldownViewerCategory(name)
    end
    return result
end

local function observeCatalogShopCall(name, id, productQuery)
    if productQuery then
        local status = customSetInputStatus(id, "number")
        if status then return { status = status } end
    end
    local fn, ok = readField(C_CatalogShop, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values
    if productQuery then
        local status = customSetInputStatus(id, "number")
        if status then return { status = status } end
        values = pack(pcall(fn, id))
    else values = pack(pcall(fn)) end
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    if productQuery and values.n > 1 then
        result.values[1] = inspectHousingCatalogFields(values[2], {
            "ID", "displayName", "iconTexture", "linkTag", "isDisabled", "showPersistentRefundButton",
        })
    end
    return result, values
end

local function observeCategoryProducts(categoryValues)
    if not categoryValues then return { status = "unavailable-input" } end
    local category = categoryValues[2]
    if not accessible(category) then return { status = "restricted-input" } end
    if type(category) ~= "table" and type(category) ~= "userdata" then
        return { status = "unavailable-input" }
    end
    -- Existing category-field observations can revoke this original receiver or ID.
    local id, ok = readField(category, "ID")
    if not ok then return { status = "field-error" } end
    local status = customSetInputStatus(id, "number")
    if status then return { status = status } end
    local fn, found = readField(C_CatalogShop, "GetProductIDsForCategory")
    if not found then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    status = customSetInputStatus(id, "number")
    if status then return { status = status } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    local first = result.values[1]
    if first and first.status == "observed" and first.kind == "table" then
        first.entries = {}
        for index = 1, 8 do
            local value, readable = readField(values[2], index)
            first.entries[index] = readable and scalar(value) or { status = "field-error" }
        end
    end
    return result
end

local function captureCatalogProducts()
    local result, values = observeCatalogShopCall("GetNewProducts")
    local first = result.values and result.values[1]
    if not first or first.status ~= "observed" or first.kind ~= "table" then return result end
    first.entries = {}
    for index = 1, 8 do
        local id, ok = readField(values[2], index)
        local row = { status = ok and "observed" or "field-error" }
        if ok then
            row.id = scalar(id)
            local categoryValues
            row.category, categoryValues = observeCatalogShopCall("GetFirstCategoryByProductID", id, true)
            row.categoryProducts = observeCategoryProducts(categoryValues)
        end
        first.entries[index] = row
    end
    return result
end

local function captureRefundableDecors()
    -- The nilable productIdFilterOpt is omitted, not explicitly passed as nil.
    local result, values = observeCatalogShopCall("GetRefundableDecors")
    local first = result.values and result.values[1]
    if not first or first.status ~= "observed" or first.kind ~= "table" then return result end
    first.entries = {}
    for index = 1, 8 do
        local entry, ok = readField(values[2], index)
        first.entries[index] = ok and inspectHousingCatalogFields(entry, {
            "decorGUID", "timeRemainingSeconds", "name", "price",
        }) or { status = "field-error" }
    end
    return result
end

local function observeCatalogCurrency(name)
    local constants, ok = readField(Constants, "CatalogShopVirtualCurrencyConstants")
    if not ok then return { status = "unavailable-input" } end
    local value, valueOK = readField(constants, name)
    if not valueOK then return { status = "unavailable-input" } end
    if not accessible(value) then return { status = "restricted-input" } end
    if type(value) ~= "string" then return { status = "unavailable-input" } end
    local input = scalar(value)
    local fn, functionOK = readField(C_CatalogShop, "GetVirtualCurrencyBalance")
    if not functionOK then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    -- Publication observations and function guards can revoke the original string.
    if not accessible(value) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, value))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status, result.input = "observed", input
    return result
end

local function captureHousingCatalog()
    local result = { featured = {} }
    for index = 1, 2 do
        result.featured[index] = observePublicQuery(C_HousingCatalog, "HasFeaturedEntries")
    end
    result.products = captureCatalogProducts()
    result.refundable = captureRefundableDecors()
    result.currencies = {}
    for _, name in ipairs({ "HEARTHSTEEL_VC_CURRENCY_CODE", "TRADERS_TENDER_VC_CURRENCY_CODE" }) do
        result.currencies[name] = observeCatalogCurrency(name)
    end
    return result
end

local function captureCustomSetNames(mode)
    if mode == "transmog-source-validity" then
        local function countStatus(count)
            local status = customSetInputStatus(count, "number")
            if status then return status end
            if count < 0 or count % 1 ~= 0 then return "unavailable-input" end
        end
        local function observeIndex(count, index)
            local status = countStatus(count)
            if status then return { status = status } end
            if not accessible(index) then return { status = "restricted-input" } end
            local input = scalar(index)
            local fn, ok = readField(C_TransmogCollection, "IsValidTransmogSource")
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            -- Recheck the original producer count, not the truncated loop bound.
            -- Checking the derived index can also revoke access to that count.
            if not accessible(index) then return { status = "restricted-input" } end
            status = countStatus(count)
            if status then return { status = status } end
            -- The count guard can revoke the previously checked index.
            if not accessible(index) then return { status = "restricted-input" } end
            local values = pack(pcall(fn, index))
            if not values[1] then return { status = "call-error" } end
            local result = mapTuple(unpack(values, 2, values.n))
            result.status, result.input = "observed", input
            return result
        end
        local producer, values = observeCustomSetCall("GetNumTransmogSources")
        local result = { producer = producer, sources = {} }
        local count = values and values[2]
        result.status = countStatus(count) or "observed"
        if result.status ~= "observed" then return result end
        -- Vendor source filters enumerate 1..count; these are not appearance IDs.
        for index = 1, math.min(count, 8) do
            result.sources[#result.sources + 1] = observeIndex(count, index)
        end
        return result
    end
    local maximum = {}
    for index = 1, 2 do
        maximum[index] = observePublicQuery(C_TransmogCollection, "GetNumMaxCustomSets")
    end
    local sets, values = observeCustomSetCall("GetCustomSets")
    local result = { maximum = maximum, sets = sets }
    local first = sets.values and sets.values[1]
    if not first or first.status ~= "observed" or first.kind ~= "table" then return result end
    first.entries = {}
    for index = 1, 4 do
        local id, ok = readField(values[2], index)
        local row = { status = ok and "observed" or "field-error" }
        if ok then
            row.id = scalar(id)
            local infoValues
            row.info, infoValues = observeCustomSetCall("GetCustomSetInfo", id, "number")
            local name = infoValues and infoValues[2]
            row.validation = observeCustomSetCall("IsValidCustomSetName", name, "string")
        end
        first.entries[index] = row
    end
    return result
end

local neighborhoodInfoFields = { "isLoaded", "neighborhoodGUID", "initiativeID", "currentCycleID",
    "progressRequired", "currentProgress", "playerTotalContribution", "duration", "tasks", "milestones",
    "title", "description" }
local neighborhoodTaskFields = { "ID", "taskName", "description", "progressContributionAmount", "tracked",
    "supersedes", "timesCompleted", "completed", "inProgress", "taskType", "sortOrder", "rewardQuestID",
    "requirementsList", "criteriaList" }

local function inspectNeighborhoodFields(object, keys, nestedKey)
    local result = scalar(object)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    local nested
    result.fields = {}
    for _, key in ipairs(keys) do
        local value, ok = readField(object, key)
        result.fields[key] = ok and scalar(value) or { status = "field-error" }
        if ok and key == nestedKey then nested = value end
    end
    return result, nested
end

local function observeNeighborhoodCall(name, id, taskQuery)
    if taskQuery then
        local input = scalar(id)
        if input.status == "restricted" then return { status = "restricted-input" } end
        if input.status ~= "observed" or input.kind ~= "number" then
            return { status = "unavailable-input" }
        end
    end
    local fn, ok = readField(C_NeighborhoodInitiative, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values
    if taskQuery then
        -- Namespace lookup and function guards can revoke a producer's original ID.
        if not accessible(id) then return { status = "restricted-input" } end
        values = pack(pcall(fn, id))
    else values = pack(pcall(fn)) end
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values
end

local function observeQuestFavor(id, clamp)
    local input = scalar(id)
    if input.status == "restricted" then return { status = "restricted-input" } end
    if input.status ~= "observed" or input.kind ~= "number" then
        return { status = "unavailable-input" }
    end
    local fn, ok = readField(C_QuestInfoSystem, "GetQuestLogRewardFavor")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(id) then return { status = "restricted-input" } end
    local values
    if clamp then values = pack(pcall(fn, id, true))
    else values = pack(pcall(fn, id)) end
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function captureQuestFavor()
    local producer, values = observeNeighborhoodCall("GetNeighborhoodInitiativeInfo")
    local result = { producer = producer, entries = {} }
    local function observeOmission(explicitFalse)
        if explicitFalse and (not accessible(nil) or not accessible(false)) then
            return { status = "restricted-input" }
        end
        local fn, fieldOK = readField(C_QuestInfoSystem, "GetQuestLogRewardFavor")
        if not fieldOK then return { status = "field-error" } end
        if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
        -- Lookup and function guards can revoke the explicit optional arguments.
        if explicitFalse and (not accessible(nil) or not accessible(false)) then
            return { status = "restricted-input" }
        end
        local returns
        if explicitFalse then returns = pack(pcall(fn, nil, false))
        else returns = pack(pcall(fn)) end
        if not returns[1] then return { status = "call-error" } end
        local observation = mapTuple(unpack(returns, 2, returns.n))
        observation.status = "observed"
        return observation
    end
    local function finish()
        result.omissions = { omitted = observeOmission(false) }
        result.omissions.unclamped = observeOmission(true)
        return result
    end
    if not values then return finish() end
    local tasks, ok = readField(values[2], "tasks")
    if not ok then result.status = "field-error"; return finish() end
    local observed = scalar(tasks)
    if observed.status ~= "observed" or observed.kind ~= "table" then
        result.status = "unavailable-input"
        return finish()
    end
    for index = 1, 4 do
        local entry, entryOK = readField(tasks, index)
        local id, idOK
        if entryOK then id, idOK = readField(entry, "rewardQuestID") end
        local row = { status = "field-error" }
        if idOK then
            row = { id = scalar(id), omitted = observeQuestFavor(id, false) }
            row.clamped = observeQuestFavor(id, true)
        end
        result.entries[index] = row
    end
    return finish()
end

local function inspectNeighborhoodActivity(object)
    local result, list = inspectNeighborhoodFields(object,
        { "isLoaded", "neighborhoodGUID", "nextUpdateTime", "taskActivity" }, "taskActivity")
    local field = result.fields and result.fields.taskActivity
    if not field or field.status ~= "observed" or field.kind ~= "table" then return result end
    field.entries = {}
    for index = 1, 8 do
        local entry, ok = readField(list, index)
        field.entries[index] = ok and inspectNeighborhoodFields(entry,
            { "taskID", "playerName", "taskName", "completionTime", "amount" }) or { status = "field-error" }
    end
    return result
end

local function inspectNeighborhoodTracked(object)
    local result, ids = inspectNeighborhoodFields(object, { "trackedIDs" }, "trackedIDs")
    local field = result.fields and result.fields.trackedIDs
    if not field or field.status ~= "observed" or field.kind ~= "table" then return result end
    field.entries = {}
    for index = 1, 4 do
        local id, ok = readField(ids, index)
        local row = { status = ok and "observed" or "field-error" }
        if ok then
            row.id = scalar(id)
            local values
            row.info, values = observeNeighborhoodCall("GetInitiativeTaskInfo", id, true)
            if values and values.n > 1 then
                row.info.values[1] = inspectNeighborhoodFields(values[2], neighborhoodTaskFields)
            end
            row.chatLink = observeNeighborhoodCall("GetInitiativeTaskChatLink", id, true)
        end
        field.entries[index] = row
    end
    return result
end

local function inspectNeighborhoodEntries(list, inspectEntry)
    local result = scalar(list)
    if result.status ~= "observed" or result.kind ~= "table" then return result end
    result.entries = {}
    for index = 1, 4 do
        local entry, ok = readField(list, index)
        result.entries[index] = ok and inspectEntry(entry) or { status = "field-error" }
    end
    return result
end

local function inspectNeighborhoodTask(object)
    return inspectNeighborhoodFields(object, neighborhoodTaskFields)
end

local function inspectNeighborhoodReward(object)
    return inspectNeighborhoodFields(object,
        { "title", "description", "decorID", "decorQuantity", "favor", "money", "rewardQuestID" })
end

local function inspectNeighborhoodMilestone(object)
    local result, rewards = inspectNeighborhoodFields(object,
        { "milestoneOrderIndex", "requiredContributionAmount", "rewards" }, "rewards")
    if result.fields then
        result.rewardEntries = result.fields.rewards.status == "field-error" and { status = "field-error" }
            or inspectNeighborhoodEntries(rewards, inspectNeighborhoodReward)
    end
    return result
end

local function inspectNeighborhoodInitiative(object)
    local result = scalar(object)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    local tasks, milestones, tasksOK, milestonesOK
    result.fields = {}
    for _, key in ipairs(neighborhoodInfoFields) do
        local value, ok = readField(object, key)
        result.fields[key] = ok and scalar(value) or { status = "field-error" }
        if key == "tasks" then tasks, tasksOK = value, ok end
        if key == "milestones" then milestones, milestonesOK = value, ok end
    end
    -- Keep opaque field observations intact; nested captures never feed task APIs.
    result.taskEntries = tasksOK and inspectNeighborhoodEntries(tasks, inspectNeighborhoodTask)
        or { status = "field-error" }
    result.milestoneEntries = milestonesOK and inspectNeighborhoodEntries(milestones, inspectNeighborhoodMilestone)
        or { status = "field-error" }
    return result
end

local function inspectPerksCriteriaList(list, fields)
    local result = scalar(list)
    if result.status ~= "observed" or result.kind ~= "table" then return result end
    result.entries = {}
    for index = 1, 4 do
        local entry, ok = readField(list, index)
        result.entries[index] = ok and inspectHousingCatalogFields(entry, fields) or { status = "field-error" }
    end
    return result
end

local function inspectPerksRoot(root)
    local result = scalar(root)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then return result end
    local list, ok = readField(root, "activities")
    local activities = ok and scalar(list) or { status = "field-error" }
    result.fields = { activities = activities }
    if activities.status ~= "observed" or activities.kind ~= "table" then return result end
    activities.entries = {}
    for index = 1, 8 do
        local entry, entryOK = readField(list, index)
        local activity = entryOK and inspectHousingCatalogFields(entry, { "ID" }) or { status = "field-error" }
        activities.entries[index] = activity
        if activity.fields then
            local criteria, criteriaOK = readField(entry, "criteriaList")
            activity.fields.criteriaList = criteriaOK and inspectPerksCriteriaList(criteria, { "criteriaID", "requiredValue" })
                or { status = "field-error" }
            local requirements, requirementsOK = readField(entry, "requirementsList")
            activity.fields.requirementsList = requirementsOK and inspectPerksCriteriaList(requirements, { "completed", "requirementText" })
                or { status = "field-error" }
        end
    end
    return result
end

local function inspectHouseExteriorOptions(object, selectedKey, fields)
    local result, options = inspectNeighborhoodFields(object, { selectedKey, "options" }, "options")
    if result.fields then
        -- Keep the opaque options field separate from the bounded entry observations.
        result.optionEntries = result.fields.options.status == "field-error" and { status = "field-error" }
            or inspectPerksCriteriaList(options, fields)
    end
    return result
end

local function captureNeighborhoodStructures(mode)
    local result = {}
    if mode == "lfg-title-match-read" then
        local function inputStatus(args, nilFrom)
            for index = 1, args.n do
                local value = args[index]
                if not accessible(value) then return "restricted-input" end
                if nilFrom and index >= nilFrom then
                    if type(value) ~= "nil" then return "unavailable-input" end
                elseif type(value) ~= "number" or value ~= value or value == math.huge or value == -math.huge then
                    return "unavailable-input"
                end
            end
        end
        local function query(name, context, ...)
            local args = pack(...)
            local nilFrom = name == "DoesEntryTitleMatchPrebuiltTitle" and 3 or nil
            local status = inputStatus(context) or inputStatus(args, nilFrom)
            if status then return { status = status } end
            local fn, ok = readField(C_LFGList, name)
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            -- Retain and recheck original ancestry as well as this call's arguments.
            status = inputStatus(context) or inputStatus(args, nilFrom)
            if status then return { status = status } end
            local values = pack(pcall(fn, unpack(args, 1, args.n)))
            if not values[1] then return { status = "call-error" } end
            local observation = mapTuple(unpack(values, 2, values.n))
            observation.status = "observed"
            return observation, values
        end
        local function entries(values, inspectEntry)
            local rows = {}
            local list = values and values[2]
            if not accessible(list) or type(list) ~= "table" then return rows end
            for index = 1, 2 do
                local value, ok = readField(list, index)
                rows[index] = ok and inspectEntry(value) or { status = "field-error" }
            end
            return rows
        end
        local container, ok = readField(Enum, "LFGListFilter")
        local filter
        if ok then filter = readField(container, "PvE") end
        result.filter, result.experiment = scalar(filter), "explicit-nil-playstyles"
        local values
        result.producer, values = query("GetAvailableCategories", pack(filter), filter)
        result.categories = entries(values, function(categoryID)
            local category = { categoryID = scalar(categoryID) }
            local groupValues
            category.producer, groupValues = query("GetAvailableActivityGroups",
                pack(filter, categoryID), categoryID, filter)
            category.groups = entries(groupValues, function(groupID)
                local group = { groupID = scalar(groupID) }
                local activityValues
                group.producer, activityValues = query("GetAvailableActivities",
                    pack(filter, categoryID, groupID), categoryID, groupID, filter)
                group.activities = entries(activityValues, function(activityID)
                    local row = { activityID = scalar(activityID) }
                    -- Declared nullable inputs: an explicit-nil experiment, not caller defaults.
                    row.match = query("DoesEntryTitleMatchPrebuiltTitle",
                        pack(filter, categoryID, groupID), activityID, groupID, nil, nil)
                    return row
                end)
                return group
            end)
            return category
        end)
        return result
    end
    if mode == "lfg-playstyle-format" then
        local function published(enumName, key)
            local container, ok = readField(Enum, enumName)
            if not ok then return nil end
            local value, memberOK = readField(container, key)
            if memberOK then return value end
        end
        local function inputStatus(args, objectLast)
            for index = 1, args.n do
                local value = args[index]
                if not accessible(value) then return "restricted-input" end
                if objectLast and index == args.n then
                    if type(value) ~= "table" and type(value) ~= "userdata" then return "unavailable-input" end
                elseif type(value) ~= "number" or value ~= value or value == math.huge or value == -math.huge then
                    return "unavailable-input"
                end
            end
        end
        local function query(name, objectLast, ...)
            local args = pack(...)
            local status = inputStatus(args, objectLast)
            if status then return { status = status } end
            local fn, ok = readField(C_LFGList, name)
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            status = inputStatus(args, objectLast)
            if status then return { status = status } end
            local values = pack(pcall(fn, unpack(args, 1, args.n)))
            if not values[1] then return { status = "call-error" } end
            local observation = mapTuple(unpack(values, 2, values.n))
            observation.status = "observed"
            return observation, values
        end
        local function entries(values, inspectEntry)
            local rows = {}
            local list = values and values[2]
            if not accessible(list) or type(list) ~= "table" then return rows end
            for index = 1, 2 do
                local value, ok = readField(list, index)
                rows[index] = ok and inspectEntry(value) or { status = "field-error" }
            end
            return rows
        end
        local function activity(id)
            local row = { activityID = scalar(id) }
            local values
            row.info, values = query("GetActivityInfoTable", false, id)
            local info = values and values[2]
            local playstyle = published("LFGEntryPlaystyle", "None")
            local general = published("LFGEntryGeneralPlaystyle", "None")
            row.formatted = query("GetPlaystyleString", true, playstyle, general, info)
            return row
        end
        local filter = published("LFGListFilter", "PvE")
        result.filter = scalar(filter)
        local values
        result.producer, values = query("GetAvailableCategories", false, filter)
        result.categories = entries(values, function(id)
            local row = { categoryID = scalar(id) }
            local activities
            -- Vendor's selectedFilters=0 branch passes group 0, not an enum fallback.
            row.producer, activities = query("GetAvailableActivities", false, id, 0, filter)
            row.activities = entries(activities, activity)
            return row
        end)
        return result
    end
    if mode == "crafting-schematic-read" then
        local function inspectReagent(object)
            return inspectHousingCatalogFields(object, { "itemID", "currencyID" })
        end
        local function inspectVariable(object)
            local observation = inspectHousingCatalogFields(object, { "quantity" })
            if observation.fields then
                local reagent, ok = readField(object, "reagent")
                observation.fields.reagent = ok and inspectReagent(reagent) or { status = "field-error" }
            end
            return observation
        end
        local function inspectSlot(object)
            local observation = inspectHousingCatalogFields(object,
                { "quantityRequired", "dataSlotIndex", "slotIndex", "reagentType" })
            if observation.fields then
                local reagents, reagentsOK = readField(object, "reagents")
                observation.fields.reagents = reagentsOK and inspectNeighborhoodEntries(reagents, inspectReagent)
                    or { status = "field-error" }
                local variables, variablesOK = readField(object, "variableQuantities")
                observation.fields.variableQuantities = variablesOK and inspectNeighborhoodEntries(variables, inspectVariable)
                    or { status = "field-error" }
            end
            return observation
        end
        local function inspectSchematic(object)
            local observation = inspectHousingCatalogFields(object, { "recipeID", "productQuality" })
            if observation.fields then
                local slots, ok = readField(object, "reagentSlotSchematics")
                observation.fields.reagentSlotSchematics = ok and inspectNeighborhoodEntries(slots, inspectSlot)
                    or { status = "field-error" }
            end
            return observation
        end
        local function observeRecipe(id, query)
            if query then
                local status = customSetInputStatus(id, "number")
                if status then return { status = status } end
            end
            local name = query and "GetRecipeSchematic" or "GetRecipesTracked"
            local fn, ok = readField(C_TradeSkillUI, name)
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            -- Function guards may revoke the original recipe ID or literal arguments.
            if not accessible(false) or (query and not accessible(nil)) then return { status = "restricted-input" } end
            local values
            if query then
                local status = customSetInputStatus(id, "number")
                if status then return { status = status } end
                values = pack(pcall(fn, id, false, nil))
            else values = pack(pcall(fn, false)) end
            if not values[1] then return { status = "call-error" } end
            local observation = mapTuple(unpack(values, 2, values.n))
            observation.status = "observed"
            if query and values.n > 1 then observation.values[1] = inspectSchematic(values[2]) end
            return observation, values
        end
        local producer, values = observeRecipe(nil, false)
        result = { producer = producer, entries = {} }
        local first = producer.values and producer.values[1]
        if first and first.status == "observed" and first.kind == "table" then
            for index = 1, 4 do
                local id, ok = readField(values[2], index)
                result.entries[index] = {
                    id = ok and scalar(id) or { status = "field-error" },
                    schematic = ok and observeRecipe(id, true) or { status = "field-error" },
                }
            end
        end
        return result
    end
    if mode == "timeline-track-info" then
        local function observeTrack(name)
            local tracks, tracksOK = readField(Enum, "EncounterTimelineTrack")
            local value, valueOK
            if tracksOK then value, valueOK = readField(tracks, name) end
            local input = valueOK and scalar(value) or { status = "unavailable-enum" }
            if input.status ~= "observed" or input.kind ~= "number" then
                return { status = "unavailable-enum" }
            end
            local fn, ok = readField(C_EncounterTimeline, "GetTrackInfo")
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            -- Publication observations, API lookup and function guards can revoke the original value.
            if not accessible(value) then return { status = "restricted-input" } end
            local values = pack(pcall(fn, value))
            if not values[1] then return { status = "call-error" } end
            local observation = mapTuple(unpack(values, 2, values.n))
            observation.status, observation.input = "observed", input
            observation.info = inspectNeighborhoodFields(values[2], {
                "id", "type", "minimumDuration", "maximumDuration", "minimumEventIntroDuration",
                "minimumEventGapDuration", "maximumEventCount", "sortDirection",
            })
            return observation
        end
        result.tracks = {}
        for _, name in ipairs({ "Queued", "Short", "Medium", "Long", "Indeterminate" }) do
            result.tracks[#result.tracks + 1] = { name = name, observation = observeTrack(name) }
        end
        return result
    end
    if mode == "timeline-source-counts" then
        local function observeSourceCount(name)
            local sources, sourcesOK = readField(Enum, "EncounterTimelineEventSource")
            local value, valueOK
            if sourcesOK then value, valueOK = readField(sources, name) end
            local input = valueOK and scalar(value) or { status = "unavailable-enum" }
            if input.status ~= "observed" or input.kind ~= "number" then
                return { status = "unavailable-enum" }
            end
            local fn, ok = readField(C_EncounterTimeline, "GetEventCountBySource")
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            -- Publication observations, API lookup and function guards can revoke the original value.
            if not accessible(value) then return { status = "restricted-input" } end
            local values = pack(pcall(fn, value))
            if not values[1] then return { status = "call-error" } end
            local observation = mapTuple(unpack(values, 2, values.n))
            observation.status, observation.input = "observed", input
            return observation
        end
        result.sources = {}
        for _, name in ipairs({ "Encounter", "Script", "EditMode" }) do
            local row = { name = name, observations = {} }
            for attempt = 1, 2 do row.observations[attempt] = observeSourceCount(name) end
            result.sources[#result.sources + 1] = row
        end
        return result
    end
    if mode == "timeline-lifecycle-read" then
        local function observeLifecycle(name, id)
            local query = name ~= "GetEventList" and name ~= "GetEventHighlightTime"
            if query then
                local input = scalar(id)
                if input.status == "restricted" then return { status = "restricted-input" } end
                if input.status ~= "observed" or input.kind ~= "number" then
                    return { status = "unavailable-input" }
                end
            end
            local fn, ok = readField(C_EncounterTimeline, name)
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            local values
            if query then
                -- Lookup and function guards can revoke the original event ID.
                if not accessible(id) then return { status = "restricted-input" } end
                values = pack(pcall(fn, id))
            else values = pack(pcall(fn)) end
            if not values[1] then return { status = "call-error" } end
            local observation = mapTuple(unpack(values, 2, values.n))
            observation.status = "observed"
            if name == "GetEventTimer" then
                for index = 2, math.min(values.n, 17) do
                    observation.values[index - 1] = inspectDuration(values[index])
                end
            end
            return observation, values
        end
        local producer, values = observeLifecycle("GetEventList")
        result = { producer = producer, entries = {}, highlight = {} }
        local first = producer.values and producer.values[1]
        if first and first.status == "observed" and first.kind == "table" then
            for index = 1, 8 do
                local id, ok = readField(values[2], index)
                local row = { id = ok and scalar(id) or { status = "field-error" }, queries = {} }
                for _, name in ipairs({ "GetEventState", "GetEventTimeElapsed", "GetEventTimeRemaining", "GetEventTimer" }) do
                    row.queries[name] = ok and observeLifecycle(name, id) or { status = "field-error" }
                end
                result.entries[index] = row
            end
        end
        -- Highlights remain independent even when the list or every ID is unavailable.
        for attempt = 1, 2 do result.highlight[attempt] = observeLifecycle("GetEventHighlightTime") end
        return result
    end
    if mode == "timeline-current-events" then
        local function observeTimeline(name, id)
            local query = name ~= "GetEventList"
            if query then
                local input = scalar(id)
                if input.status == "restricted" then return { status = "restricted-input" } end
                if input.status ~= "observed" or input.kind ~= "number" then
                    return { status = "unavailable-input" }
                end
            end
            local fn, ok = readField(C_EncounterTimeline, name)
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            local values
            if query then
                -- Namespace lookup and function guards can revoke the original event ID.
                if not accessible(id) then return { status = "restricted-input" } end
                values = pack(pcall(fn, id))
            else values = pack(pcall(fn)) end
            if not values[1] then return { status = "call-error" } end
            local observation = mapTuple(unpack(values, 2, values.n))
            observation.status = "observed"
            if name == "GetEventInfo" and values.n > 1 then
                observation.values[1] = inspectNeighborhoodFields(values[2], {
                    "id", "source", "spellName", "spellID", "iconFileID", "duration",
                    "maxQueueDuration", "icons", "severity", "isApproximate",
                })
            end
            return observation, values
        end
        local producer, values = observeTimeline("GetEventList")
        result = { producer = producer, entries = {} }
        local first = producer.values and producer.values[1]
        if not first or first.status ~= "observed" or first.kind ~= "table" then return result end
        for index = 1, 8 do
            local id, ok = readField(values[2], index)
            local row = { id = ok and scalar(id) or { status = "field-error" } }
            row.info = ok and observeTimeline("GetEventInfo", id) or { status = "field-error" }
            -- Color remains opaque, and overrideTrigger is deliberately omitted.
            row.color = ok and observeTimeline("GetEventColor", id) or { status = "field-error" }
            result.entries[index] = row
        end
        return result
    end
    if mode == "item-interaction-flags" then
        for attempt = 1, 2 do
            local fn, ok = readField(C_ItemInteraction, "GetItemInteractionInfo")
            local observation
            if not ok then observation = { status = "field-error" }
            elseif not accessible(fn) or type(fn) ~= "function" then observation = { status = "missing-api" }
            else
                local values = pack(pcall(fn))
                if not values[1] then observation = { status = "call-error" }
                else
                    observation = mapTuple(unpack(values, 2, values.n))
                    observation.status = "observed"
                    if values.n > 1 then
                        observation.values[1] = inspectNeighborhoodFields(values[2], { "flags" })
                    end
                end
            end
            result[attempt] = observation
        end
        return result
    end
    if mode == "house-exterior-options" then
        for _, spec in ipairs({
            { "GetCurrentHouseExteriorType" },
            { "GetHouseExteriorSizeOptions", "selectedSize", { "size", "name", "isLocked" } },
            { "GetHouseExteriorTypeOptions", "selectedExteriorType",
                { "houseExteriorTypeID", "name", "isLocked", "isInvalid", "reasonString" } },
        }) do
            local fn, ok = readField(C_HouseExterior, spec[1])
            local observation
            if not ok then observation = { status = "field-error" }
            elseif not accessible(fn) or type(fn) ~= "function" then observation = { status = "missing-api" }
            else
                local values = pack(pcall(fn))
                if not values[1] then observation = { status = "call-error" }
                else
                    observation = mapTuple(unpack(values, 2, values.n))
                    observation.status = "observed"
                    if spec[2] and values.n > 1 then
                        observation.values[1] = inspectHouseExteriorOptions(values[2], spec[2], spec[3])
                    end
                end
            end
            result[spec[1]] = observation
        end
        return result
    end
    if mode == "perks-criteria" then
        for attempt = 1, 2 do
            local fn, ok = readField(C_PerksActivities, "GetPerksActivitiesInfo")
            local observation
            if not ok then observation = { status = "field-error" }
            elseif not accessible(fn) or type(fn) ~= "function" then observation = { status = "missing-api" }
            else
                local values = pack(pcall(fn))
                if not values[1] then observation = { status = "call-error" }
                else
                    observation = mapTuple(unpack(values, 2, values.n))
                    observation.status = "observed"
                    if values.n > 1 then observation.values[1] = inspectPerksRoot(values[2]) end
                end
            end
            result[attempt] = observation
        end
        return result
    end
    for _, name in ipairs({ "GetNeighborhoodInitiativeInfo", "GetInitiativeActivityLogInfo", "GetTrackedInitiativeTasks" }) do
        local observation, values = observeNeighborhoodCall(name)
        if values and values.n > 1 then
            if name == "GetNeighborhoodInitiativeInfo" then
                observation.values[1] = inspectNeighborhoodInitiative(values[2])
            elseif name == "GetInitiativeActivityLogInfo" then
                observation.values[1] = inspectNeighborhoodActivity(values[2])
            else observation.values[1] = inspectNeighborhoodTracked(values[2]) end
        end
        result[name] = observation
    end
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

local function majorFactionInputStatus(id)
    if not accessible(id) then return "restricted-input" end
    if type(id) ~= "number" or id ~= id or id == math.huge or id == -math.huge then
        return "unavailable-input"
    end
end

local function observeRenownCall(name, factionID, level)
    local function inputStatus()
        local status = majorFactionInputStatus(factionID)
        if status then return status end
        if name == "GetRenownRewardsForLevel" then return majorFactionInputStatus(level) end
    end
    local status = inputStatus()
    if status then return { status = status } end
    local fn, ok = readField(C_MajorFactions, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    status = inputStatus()
    if status then return { status = status } end
    local values
    if name == "GetRenownRewardsForLevel" then values = pack(pcall(fn, factionID, level))
    else values = pack(pcall(fn, factionID)) end
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values[2]
end

local function captureRenownRewards(factionID, level)
    local result, list = observeRenownCall("GetRenownRewardsForLevel", factionID, level)
    if not accessible(list) or type(list) ~= "table" then return result end
    result.entries = {}
    for index = 1, 4 do
        local entry, ok = readField(list, index)
        result.entries[index] = ok and inspectNeighborhoodFields(entry, {
            "renownRewardID", "uiOrder", "isAccountUnlock", "itemID", "spellID", "mountID",
            "transmogID", "transmogSetID", "titleMaskID", "transmogIllusionSourceID",
            "icon", "name", "description", "toastDescription", "rewardType",
        }) or { status = "field-error" }
    end
    return result
end

local function captureRenownLevels(factionID)
    local result, list = observeRenownCall("GetRenownLevels", factionID)
    if not accessible(list) or type(list) ~= "table" then return result end
    result.entries = {}
    for index = 1, 4 do
        local entry, ok = readField(list, index)
        local row = { status = "field-error" }
        if ok then
            local level
            row, level = inspectNeighborhoodFields(entry,
                { "factionID", "level", "locked", "isMilestone", "isCapstone" }, "level")
            -- Never substitute the entry's factionID for the original producer ID.
            row.rewards = captureRenownRewards(factionID, level)
        end
        result.entries[index] = row
    end
    return result
end

local function captureMajorFactionRenownRewards()
    local result = { entries = {} }
    local fn, ok = readField(C_MajorFactions, "GetMajorFactionIDs")
    if not ok then result.producer = { status = "field-error" }; return result end
    if not accessible(fn) or type(fn) ~= "function" then
        result.producer = { status = "missing-api" }; return result
    end
    local values = pack(pcall(fn, nil))
    if not values[1] then result.producer = { status = "call-error" }; return result end
    result.producer = mapTuple(unpack(values, 2, values.n))
    result.producer.status = "observed"
    local list = values[2]
    if not accessible(list) or type(list) ~= "table" then return result end
    for index = 1, 8 do
        local id, idOK = readField(list, index)
        local row = { status = "field-error" }
        if idOK then
            row = { id = scalar(id) }
            row.levels = captureRenownLevels(id)
        end
        result.entries[index] = row
    end
    return result
end

local function inspectMajorFactionData(object)
    local result, highlights = inspectNeighborhoodFields(object,
        { "description", "playerCompanionID", "highlights" }, "highlights")
    if not result.fields then return result end
    result.highlights = scalar(highlights)
    if result.highlights.status ~= "observed" or result.highlights.kind ~= "table" then return result end
    result.highlights.entries = {}
    for index = 1, 4 do
        local entry, ok = readField(highlights, index)
        result.highlights.entries[index] = ok and inspectNeighborhoodFields(entry,
            { "title", "description", "level" }) or { status = "field-error" }
    end
    return result
end

local function observeMajorFactionJourney(id, name)
    local status = majorFactionInputStatus(id)
    if status then return { status = status } end
    local fn, ok = readField(C_MajorFactions, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    status = majorFactionInputStatus(id)
    if status then return { status = status } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    if name == "GetMajorFactionData" then result.data = inspectMajorFactionData(values[2]) end
    return result
end

local function captureMajorFactionJourney()
    local result = { entries = {} }
    local fn, ok = readField(C_MajorFactions, "GetMajorFactionIDs")
    if not ok then result.producer = { status = "field-error" }; return result end
    if not accessible(fn) or type(fn) ~= "function" then
        result.producer = { status = "missing-api" }; return result
    end
    -- One explicit nil is the documented nilable expansion filter.
    local values = pack(pcall(fn, nil))
    if not values[1] then result.producer = { status = "call-error" }; return result end
    result.producer = mapTuple(unpack(values, 2, values.n))
    result.producer.status = "observed"
    local list = values[2]
    if not accessible(list) or type(list) ~= "table" then return result end
    for index = 1, 8 do
        local id, idOK = readField(list, index)
        local row = { queries = {} }
        if not idOK then row.status = "field-error"
        else
            row.id = scalar(id)
            for _, name in ipairs({ "ShouldDisplayMajorFactionAsJourney", "ShouldUseJourneyRewardTrack" }) do
                row.queries[name] = observeMajorFactionJourney(id, name)
            end
            row.queries.GetMajorFactionData = observeMajorFactionJourney(id, "GetMajorFactionData")
        end
        result.entries[index] = row
    end
    return result
end

local function preyNumberAccessible(value)
    return accessible(value) and type(value) == "number" and value == value
        and value ~= math.huge and value ~= -math.huge
end

local function observePreyVisualization(id, kind)
    if not preyNumberAccessible(id) or not preyNumberAccessible(kind) then return { status = "unavailable-input" } end
    local enums, ok = readField(Enum, "UIWidgetVisualizationType")
    local discriminator
    if ok then discriminator, ok = readField(enums, "PreyHuntProgress") end
    if not ok or not preyNumberAccessible(discriminator) then return { status = "unavailable-enum" } end
    if not preyNumberAccessible(id) or not preyNumberAccessible(kind) then return { status = "restricted-input" } end
    if not accessible(kind) or not accessible(discriminator) then return { status = "restricted-input" } end
    if kind ~= discriminator then return { status = "wrong-widget-type" } end
    local fn
    fn, ok = readField(C_UIWidgetManager, "GetPreyHuntProgressWidgetVisualizationInfo")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not preyNumberAccessible(id) or not preyNumberAccessible(kind)
        or not preyNumberAccessible(discriminator) then return { status = "restricted-input" } end
    if not accessible(kind) or not accessible(discriminator) then return { status = "restricted-input" } end
    if kind ~= discriminator then return { status = "wrong-widget-type" } end
    if not preyNumberAccessible(id) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status, result.fields = "observed", {}
    for _, key in ipairs({ "shownState", "progressState", "tooltip", "tooltipLoc", "widgetSizeSetting",
        "textureKit", "frameTextureKit", "hasTimer", "orderIndex", "widgetTag", "inAnimType",
        "outAnimType", "widgetScale", "layoutDirection", "modelSceneLayer", "scriptedAnimationEffectID" }) do
        local value, fieldOK = readField(values[2], key)
        result.fields[key] = fieldOK and scalar(value) or { status = "field-error" }
    end
    return result
end

local function observePreyWidgetSet(id)
    if not preyNumberAccessible(id) then return { status = "unavailable-input" } end
    local fn, ok = readField(C_UIWidgetManager, "GetAllWidgetsBySetID")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not preyNumberAccessible(id) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status, result.entries = "observed", {}
    for index = 1, 4 do
        local entry, entryOK = readField(values[2], index)
        local row = { status = entryOK and "observed" or "field-error" }
        result.entries[index] = row
        if entryOK then
            local widgetID, idOK = readField(entry, "widgetID")
            local kind, kindOK = readField(entry, "widgetType")
            row.widgetID = idOK and scalar(widgetID) or { status = "field-error" }
            row.widgetType = kindOK and scalar(kind) or { status = "field-error" }
            row.visualization = idOK and kindOK and observePreyVisualization(widgetID, kind)
                or { status = "unavailable-input" }
        end
    end
    return result
end

local function observePreyWidget(id, name)
    if not preyNumberAccessible(id) then return { status = "unavailable-input" } end
    local enums, ok = readField(Enum, "MapIconUIWidgetSetType")
    local value
    if ok then value, ok = readField(enums, name) end
    if not ok or not preyNumberAccessible(value) then return { status = "unavailable-enum" } end
    local fn
    fn, ok = readField(C_TaskQuest, "GetQuestUIWidgetSetByType")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not preyNumberAccessible(id) or not preyNumberAccessible(value) then
        return { status = "restricted-input" }
    end
    local values = pack(pcall(fn, id, value))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values[2]
end

local function capturePreyQuestWidgets()
    local result = { queries = {}, details = {} }
    local fn, ok = readField(C_QuestLog, "GetActivePreyQuest")
    local values
    if not ok then result.producer = { status = "field-error" }
    elseif not accessible(fn) or type(fn) ~= "function" then
        result.producer = { status = "missing-api" }
    else
        values = pack(pcall(fn))
        if values[1] then
            result.producer = mapTuple(unpack(values, 2, values.n))
            result.producer.status = "observed"
        else result.producer = { status = "call-error" }; values = nil end
    end
    for _, name in ipairs({ "Tooltip", "BehindIcon", "AdventureMapDetails" }) do
        local setID
        if values then result.queries[name], setID = observePreyWidget(values[2], name)
        else result.queries[name] = { status = "unavailable-input" } end
        result.details[name] = observePreyWidgetSet(setID)
    end
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

local function inspectDiminishCategory(entry)
    local result = scalar(entry)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    result.fields = {}
    for _, key in ipairs({ "category", "name", "icon" }) do
        local value, ok = readField(entry, key)
        result.fields[key] = ok and scalar(value) or { status = "field-error" }
    end
    return result
end

local function observeDiminishCategory(enumName, name, apiName, isList)
    local enums, enumOK = readField(Enum, enumName)
    local value, valueOK
    if enumOK then value, valueOK = readField(enums, name) end
    local input = valueOK and scalar(value) or { status = "unavailable-enum" }
    if input.status ~= "observed" or input.kind ~= "number" then
        return { status = "unavailable-enum" }
    end
    local fn, ok = readField(C_SpellDiminish, apiName)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(value) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, value))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status, result.input = "observed", input
    if not isList then
        if values.n > 1 then result.values[1] = inspectDiminishCategory(values[2]) end
        return result
    end
    local first = result.values[1]
    if first and first.status == "observed" and first.kind == "table" then
        first.entries = {}
        for index = 1, 8 do
            local entry, entryOK = readField(values[2], index)
            first.entries[index] = entryOK and inspectDiminishCategory(entry) or { status = "field-error" }
        end
    end
    return result
end

local function captureDiminishCategories()
    local result = { rulesets = {}, categories = {} }
    for _, name in ipairs({ "None", "PvE", "PvP" }) do
        result.rulesets[#result.rulesets + 1] = { name = name,
            observation = observeDiminishCategory("SpellDiminishRuleset", name, "GetAllSpellDiminishCategories", true) }
    end
    for _, name in ipairs({ "Root", "Taunt", "Stun", "AoEKnockback", "Incapacitate", "Disorient", "Silence", "Disarm" }) do
        result.categories[#result.categories + 1] = { name = name,
            observation = observeDiminishCategory("SpellDiminishCategory", name, "GetSpellDiminishCategoryInfo", false) }
    end
    return result
end

local function observeOutfitSlotQuery(slot, name)
    local status = auraNumberStatus(slot)
    if status then return { status = status } end
    local fn, ok = readField(C_TransmogOutfitInfo, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    status = auraNumberStatus(slot)
    if status then return { status = status } end
    local values = pack(pcall(fn, slot))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function inspectOutfitSlot(entry, querySlot)
    local result = scalar(entry)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    result.fields = {}
    local slot
    for _, key in ipairs({ "slot", "type", "collectionType", "slotName", "isSecondary" }) do
        local value, ok = readField(entry, key)
        result.fields[key] = ok and scalar(value) or { status = "field-error" }
        if key == "slot" and ok then slot = value end
    end
    if querySlot then
        result.queries = {}
        for _, name in ipairs({ "GetEquippedSlotOptionFromTransmogSlot", "GetUnassignedAtlasForSlot" }) do
            result.queries[name] = observeOutfitSlotQuery(slot, name)
        end
    end
    return result
end

local function inspectOutfitSlotList(list, querySlot)
    local result = scalar(list)
    if result.status ~= "observed" or result.kind ~= "table" then return result end
    result.entries = {}
    for index = 1, 8 do
        local entry, ok = readField(list, index)
        result.entries[index] = ok and inspectOutfitSlot(entry, querySlot) or { status = "field-error" }
    end
    return result
end

local function inspectOutfitSlotGroups(groups)
    local result = scalar(groups)
    if result.status ~= "observed" or result.kind ~= "table" then return result end
    result.entries = {}
    for index = 1, 8 do
        local entry, ok = readField(groups, index)
        local row = ok and scalar(entry) or { status = "field-error" }
        if row.status == "observed" and (row.kind == "table" or row.kind == "userdata") then
            row.fields = {}
            for _, key in ipairs({ "position", "appearanceSlotInfo", "illusionSlotInfo" }) do
                local value, fieldOK = readField(entry, key)
                if not fieldOK then row.fields[key] = { status = "field-error" }
                elseif key == "position" then row.fields[key] = scalar(value)
                else row.fields[key] = inspectOutfitSlotList(value, false) end
            end
        end
        result.entries[index] = row
    end
    return result
end

local function observeOutfitSlotProducer(name, locations)
    local fn, ok = readField(C_TransmogOutfitInfo, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    if locations then
        for index = 2, math.min(values.n, 3) do
            result.values[index - 1] = inspectOutfitSlotList(values[index], true)
        end
    elseif values.n > 1 then result.values[1] = inspectOutfitSlotGroups(values[2]) end
    return result
end

local function captureOutfitSlots()
    local locations = observeOutfitSlotProducer("GetAllSlotLocationInfo", true)
    local groups = observeOutfitSlotProducer("GetSlotGroupInfo", false)
    return { locations = locations, groups = groups }
end

local function inspectOutfitCategories(categories)
    local result = scalar(categories)
    if result.status ~= "observed" or result.kind ~= "table" then return result end
    result.entries = {}
    for index = 1, 8 do
        local value, ok = readField(categories, index)
        result.entries[index] = ok and scalar(value) or { status = "field-error" }
    end
    return result
end

local function inspectOutfitEntry(entry)
    local result = scalar(entry)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    local id
    result.fields = {}
    for _, key in ipairs({ "outfitID", "name", "icon", "isEventOutfit", "isDisabled",
        "playerFacingOutfitIndex", "situationCategories" }) do
        local value, ok = readField(entry, key)
        if not ok then result.fields[key] = { status = "field-error" }
        elseif key == "situationCategories" then result.fields[key] = inspectOutfitCategories(value)
        else result.fields[key] = scalar(value) end
        if key == "outfitID" and ok then id = value end
    end
    return result, id
end

local function observeOutfitInfo(id)
    if not accessible(id) then return { status = "restricted-input" } end
    local input = scalar(id)
    if input.status ~= "observed" or input.kind ~= "number" then
        return { status = "unavailable-input" }
    end
    local fn, ok = readField(C_TransmogOutfitInfo, "GetOutfitInfo")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(id) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    if values.n > 1 then result.values[1] = inspectOutfitEntry(values[2]) end
    return result
end

local function observeOutfitTooltip(id)
    if not accessible(id) then return { status = "restricted-input" } end
    local input = scalar(id)
    if input.status ~= "observed" or input.kind ~= "number" then
        return { status = "unavailable-input" }
    end
    local fn, ok = readField(C_TooltipInfo, "GetOutfit")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(id) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, id))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function captureOutfitTooltip()
    local fn, ok = readField(C_TransmogOutfitInfo, "GetOutfitsInfo")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    local first = result.values[1]
    if not first or first.status ~= "observed" or first.kind ~= "table" then return result end
    first.entries = {}
    for index = 1, 4 do
        local entry, entryOK = readField(values[2], index)
        local id, idOK
        if entryOK then id, idOK = readField(entry, "outfitID") end
        local row = { id = idOK and scalar(id) or { status = "field-error" } }
        row.query = idOK and observeOutfitTooltip(id) or { status = "unavailable-input" }
        first.entries[index] = row
    end
    return result
end

local function captureOutfitCatalog()
    local fn, ok = readField(C_TransmogOutfitInfo, "GetOutfitsInfo")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    local first = result.values[1]
    if not first or first.status ~= "observed" or first.kind ~= "table" then return result end
    first.entries = {}
    for index = 1, 8 do
        local entry, entryOK = readField(values[2], index)
        local row, id
        if entryOK then row, id = inspectOutfitEntry(entry)
        else row = { status = "field-error" } end
        row.query = observeOutfitInfo(id)
        first.entries[index] = row
    end
    return result
end

local function inspectSetCatalogEntry(entry)
    local result = scalar(entry)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    result.fields = {}
    for _, key in ipairs({ "setID", "name", "collected", "favorite", "validForCharacter", "grantAsPrecedingVariant" }) do
        local value, ok = readField(entry, key)
        result.fields[key] = ok and scalar(value) or { status = "field-error" }
    end
    return result
end

local function observeAvailableSets()
    local fn, ok = readField(C_TransmogSets, "GetAvailableSets")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    local values = pack(pcall(fn))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    local first = result.values[1]
    if first and first.status == "observed" and first.kind == "table" then
        first.entries = {}
        for index = 1, 8 do
            local entry, entryOK = readField(values[2], index)
            first.entries[index] = entryOK and inspectSetCatalogEntry(entry) or { status = "field-error" }
        end
    end
    return result
end

local function captureSetsCatalog()
    local result = { available = observeAvailableSets(), filters = {} }
    for index = 1, 2 do
        result.filters[index] = observePublicQuery(C_TransmogSets, "IsUsingDefaultSetsFilters")
    end
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

local function observeNameplateInsets(name)
    local enums, enumOK = readField(Enum, "NamePlateType")
    local value, valueOK
    if enumOK then value, valueOK = readField(enums, name) end
    local input = valueOK and scalar(value) or { status = "unavailable-enum" }
    if input.status ~= "observed" or input.kind ~= "number" then
        return { status = "unavailable-enum" }
    end
    local fn, ok = readField(C_NamePlateManager, "GetNamePlateHitTestInsets")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(value) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, value))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status, result.input = "observed", input
    return result
end

local function captureNameplateMetrics()
    local result = { size = {}, modes = {} }
    for index = 1, 2 do
        result.size[index] = observePublicQuery(C_NamePlate, "GetNamePlateSize")
    end
    for _, name in ipairs({ "Friendly", "Enemy" }) do
        local row = { name = name, observations = {} }
        for index = 1, 2 do row.observations[index] = observeNameplateInsets(name) end
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

local function inspectCraftingQuality(info)
    local result = scalar(info)
    if result.status ~= "observed" or (result.kind ~= "table" and result.kind ~= "userdata") then
        return result
    end
    result.fields = {}
    for _, name in ipairs({ "quality", "icon", "iconSmall", "iconInventory", "iconMixed",
        "iconAppear", "iconDissolve", "barFill", "barBackground", "barBackgroundCap",
        "barHighlight", "iconChat", "iconQuestObjective" }) do
        local value, ok = readField(info, name)
        result.fields[name] = ok and scalar(value) or { status = "field-error" }
    end
    return result
end

local function observeItemQuality(link, name)
    if not accessible(link) then return { status = "restricted-input" } end
    if type(link) ~= "string" then return { status = "unavailable-input" } end
    local fn, ok = readField(C_TradeSkillUI, name)
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(link) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, link))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    result.info = inspectCraftingQuality(values[2])
    return result
end

local function captureTradeskillItemQuality()
    local result = { slots = {} }
    for slot = 1, 19 do
        local producer, link = observeItemProducer(slot)
        local row = { slot = slot, producer = producer, queries = {} }
        for _, name in ipairs({ "GetItemCraftedQualityInfo", "GetItemReagentQualityInfo" }) do
            row.queries[name] = producer.status == "observed" and observeItemQuality(link, name)
                or { status = "unavailable-input" }
        end
        result.slots[slot] = row
    end
    return result
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

local function observeEquippedItemInfo(link)
    if not accessible(link) then return { status = "restricted-input" } end
    if type(link) ~= "string" then return { status = "unavailable-input" } end
    local fn, ok = readField(C_Item, "GetItemInfo")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(link) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, link))
    if not values[1] then return { status = "call-error" } end
    -- GetItemInfo declares eighteen returns; other observers retain their sixteen-position bound.
    local result = { status = "observed", n = values.n - 1, values = {} }
    for index = 2, math.min(values.n, 19) do
        result.values[index - 1] = scalar(values[index])
    end
    if values.n > 19 then result.truncated = true end
    return result
end

local function observeEquipmentLocation(slot)
    local receiver = ItemLocation
    local fn, ok = readField(receiver, "CreateFromEquipmentSlot")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(receiver) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, receiver, slot))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result, values[2]
end

local function observeTransmogEligibility(location)
    if not accessible(location) then return { status = "restricted-input" } end
    if type(location) ~= "table" and type(location) ~= "userdata" then
        return { status = "unavailable-input" }
    end
    local fn, ok = readField(C_Item, "CanItemTransmogAppearance")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(location) then return { status = "restricted-input" } end
    local values = pack(pcall(fn, location))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function captureItemBinding(itemInfo)
    local result = { slots = {} }
    if itemInfo == "transmog-eligibility" then
        for slot = 1, 19 do
            local producer, location = observeEquipmentLocation(slot)
            result.slots[slot] = { slot = slot, producer = producer,
                eligibility = producer.status == "observed" and observeTransmogEligibility(location)
                    or { status = "unavailable-input" } }
        end
        return result
    end
    for slot = 1, 19 do
        local producer, link = observeItemProducer(slot)
        local observation = { status = "unavailable-input" }
        if producer.status == "observed" then
            observation = itemInfo and observeEquippedItemInfo(link) or observeItemBinding(link)
        end
        local row = { slot = slot, producer = producer }
        if itemInfo then row.itemInfo = observation else row.binding = observation end
        result.slots[slot] = row
    end
    return result
end

-- Keep the new producer chain scoped: the main chunk and slash handler have Lua 5.1 local/upvalue limits.
do
    local captureEquipment = captureItemBinding
    local function objectStatus(object, tableOnly)
        if not accessible(object) then return "restricted-input" end
        local kind = type(object)
        if kind ~= "table" and (tableOnly or kind ~= "userdata") then return "unavailable-input" end
    end

    local function factoryInputStatus(descriptor, transmogType, secondary)
        if not accessible(descriptor) or not accessible(transmogType) or not accessible(secondary) then
            return "restricted-input"
        end
        if type(descriptor) ~= "string" or type(secondary) ~= "boolean" then return "unavailable-input" end
        return customSetInputStatus(transmogType, "number")
    end

    local function recordCall(fn, ...)
        local values = pack(pcall(fn, ...))
        if not values[1] then return { status = "call-error" } end
        local result = mapTuple(unpack(values, 2, values.n))
        result.status = "observed"
        return result, values[2]
    end

    local function createLocation(descriptor, secondary)
        local enum, enumOK = readField(Enum, "TransmogType")
        local transmogType, valueOK
        if enumOK then transmogType, valueOK = readField(enum, "Appearance") end
        if not valueOK then return { status = "field-error" } end
        local status = factoryInputStatus(descriptor, transmogType, secondary)
        if status then return { status = status } end
        local fn, ok = readField(TransmogUtil, "CreateTransmogLocation")
        if not ok then return { status = "field-error" } end
        if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
        status = factoryInputStatus(descriptor, transmogType, secondary)
        if status then return { status = status } end
        -- Vendor factory is a dot call; no TransmogUtil receiver is passed.
        return recordCall(fn, descriptor, transmogType, secondary)
    end

    local function getLocationData(location)
        local status = objectStatus(location)
        if status then return { status = status } end
        local fn, ok = readField(location, "GetData")
        if not ok then return { status = "field-error" } end
        if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
        status = objectStatus(location)
        if status then return { status = status } end
        return recordCall(fn, location)
    end

    local function dataStatus(data)
        local status = objectStatus(data, true)
        if status then return status end
        local fields = {}
        for index, key in ipairs({ "slotID", "type", "modification" }) do
            local value, ok = readField(data, key)
            if not ok then return "field-error" end
            fields[index] = value
            status = customSetInputStatus(value, "number")
            if status then return status end
        end
        -- A later field lookup may revoke an earlier value. Validate all three again; never rebuild data.
        for index = 1, 3 do
            status = customSetInputStatus(fields[index], "number")
            if status then return status end
        end
        return objectStatus(data, true)
    end

    local function getAppearanceSource(object, key)
        local status = objectStatus(object)
        if status then return { status = status } end
        -- Reread the original field after visual serialization, never its saved scalar copy.
        local id, fieldOK = readField(object, key)
        if not fieldOK then return { status = "field-error" } end
        status = customSetInputStatus(id, "number")
        if status then return { status = status } end
        local fn, ok = readField(C_TransmogCollection, "GetAppearanceSourceInfo")
        if not ok then return { status = "field-error" } end
        if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
        status = customSetInputStatus(id, "number")
        if status then return { status = status } end
        local result, info = recordCall(fn, id)
        if result.status == "observed" then
            result.object = inspectHousingCatalogFields(info, { "category", "itemAppearanceID",
                "canHaveIllusion", "icon", "isCollected", "itemLink", "transmoglink", "sourceType",
                "itemSubclass", "ignoreModelAttachmentChecksForIllusion" })
        end
        return result
    end

    local function getVisualInfo(data)
        local status = dataStatus(data)
        if status then return { status = status } end
        local fn, ok = readField(C_Transmog, "GetSlotVisualInfo")
        if not ok then return { status = "field-error" } end
        if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
        status = dataStatus(data)
        if status then return { status = status } end
        local result, object = recordCall(fn, data)
        if result.status == "observed" then
            result.object = inspectHousingCatalogFields(object, { "baseSourceID", "baseVisualID",
                "appliedSourceID", "appliedVisualID", "pendingSourceID", "pendingVisualID",
                "hasUndo", "isHideVisual", "itemSubclass" })
        end
        return result, object
    end

    captureItemBinding = function(mode)
        if mode ~= "slot-visual-info" then return captureEquipment(mode) end
        local result = { cases = {} }
        for _, descriptor in ipairs({ "HEADSLOT", "SHOULDERSLOT" }) do
            for _, secondary in ipairs({ false, true }) do
                local row = { descriptor = scalar(descriptor), secondary = scalar(secondary) }
                local location, data, visualObject
                row.factory, location = createLocation(descriptor, secondary)
                row.data = { status = "unavailable-input" }
                row.visual = { status = "unavailable-input" }
                if row.factory.status == "observed" then row.data, data = getLocationData(location) end
                if row.data.status == "observed" then row.visual, visualObject = getVisualInfo(data) end
                row.sources = {}
                for _, key in ipairs({ "baseSourceID", "appliedSourceID" }) do
                    row.sources[key] = row.visual.status == "observed" and getAppearanceSource(visualObject, key)
                        or { status = "unavailable-input" }
                end
                result.cases[#result.cases + 1] = row
            end
        end
        return result
    end
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

local function captureHealCalculator(mode)
    if mode == "modes" then
        local function objectStatus(object)
            if not accessible(object) then return "restricted-input" end
            if type(object) ~= "table" and type(object) ~= "userdata" then return "unavailable-input" end
        end
        local function enumStatus(value)
            if not accessible(value) then return "restricted-input" end
            if type(value) ~= "number" or value ~= value or value == math.huge or value == -math.huge then
                return "unavailable-input"
            end
        end
        local function call(object, name, hasValue, value)
            local status = objectStatus(object)
            if status then return { status = status } end
            if hasValue then
                status = enumStatus(value)
                if status then return { status = status } end
            end
            local fn, ok = readField(object, name)
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            status = objectStatus(object)
            if status then return { status = status } end
            if hasValue then
                status = enumStatus(value)
                if status then return { status = status } end
            end
            if not accessible(object) then return { status = "restricted-input" } end
            if hasValue and not accessible(value) then return { status = "restricted-input" } end
            local values
            if hasValue then values = pack(pcall(fn, object, value))
            else values = pack(pcall(fn, object)) end
            if not values[1] then return { status = "call-error" } end
            local result = mapTuple(unpack(values, 2, values.n))
            result.status = "observed"
            return result
        end
        local getters = { "GetHealAbsorbMode", "GetHealAbsorbClampMode", "GetDamageAbsorbClampMode" }
        local function snapshot(object)
            local result = {}
            for _, name in ipairs(getters) do result[name] = call(object, name) end
            return result
        end
        local fn, ok = readField(_G, "CreateUnitHealPredictionCalculator")
        if not ok then return { constructor = { status = "field-error" } } end
        if not accessible(fn) or type(fn) ~= "function" then
            return { constructor = { status = "missing-api" } }
        end
        local values = pack(pcall(fn))
        if not values[1] then return { constructor = { status = "call-error" } } end
        local row = { constructor = mapTuple(unpack(values, 2, values.n)) }
        row.constructor.status = "observed"
        local object = values[2]
        row.status = objectStatus(object)
        if row.status then return row end
        row.baseline, row.modes, row.resets = snapshot(object), {}, {}
        for _, group in ipairs({
            { "UnitHealAbsorbMode", "SetHealAbsorbMode", getters[1], { "ReducedByIncomingHeals", "Total" } },
            { "UnitHealAbsorbClampMode", "SetHealAbsorbClampMode", getters[2], { "CurrentHealth", "MaximumHealth" } },
            { "UnitDamageAbsorbClampMode", "SetDamageAbsorbClampMode", getters[3],
                { "MissingHealth", "MissingHealthWithoutIncomingHeals", "MaximumHealth" } },
        }) do
            for _, key in ipairs(group[4]) do
                local enums, enumOK = readField(Enum, group[1])
                local value, valueOK
                if enumOK then value, valueOK = readField(enums, key) end
                local entry = { enum = group[1], key = key }
                entry.setter = valueOK and call(object, group[2], true, value) or { status = "field-error" }
                -- A failed setter does not suppress the independent readback.
                entry.getter = call(object, group[3])
                row.modes[#row.modes + 1] = entry
            end
        end
        for _, name in ipairs({ "Reset", "ResetPredictedValues" }) do
            local reset = { name = name, result = call(object, name) }
            reset.getters = snapshot(object)
            row.resets[#row.resets + 1] = reset
        end
        return row
    end
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

local function observeResidualHyperlink(args)
    local fn, ok = readField(C_StringUtil, "StripHyperlinks")
    if not ok then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    for index = 1, args.n do
        if not accessible(args[index]) then return { status = "restricted-input" } end
    end
    local values = pack(pcall(fn, unpack(args, 1, args.n)))
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function captureHyperlinksResidual()
    local text = "|cffff0000|Hitem:19019|h[Item]|h|r |A:atlas:16:16|a |Ttexture:16:16|t"
    local values = pack(nil, 0, 1, "", "x", {}, function() error("residual function input invoked") end)
    local names = { "nil", "zero", "one", "empty-string", "string", "table", "function" }
    local rows = {}
    local function record(args, variant, position)
        local row = { variant = variant, position = position, argumentCount = args.n }
        row.result = observeResidualHyperlink(args)
        row.arguments = mapTuple(unpack(args, 1, args.n))
        rows[#rows + 1] = row
    end
    for position = 1, 5 do
        for index = 1, values.n do
            local args = pack(text, false, false, false, false, false)
            args[position + 1] = values[index]
            record(args, names[index], position)
        end
    end
    local corpus = {
        "|Hitem:19019", "|Hitem:1|h[outer |Hitem:2|h[inner]|h]|h",
        "|cffff0000outer|cff00ff00inner|r outer|r", "|A:atlas:16:16|Ttexture:16:16|a|t",
        "|r|a|t", "before\0|Hitem:1|h[x]|h\0after",
        "\1\9\13\31|Hitem:1|h[x]|h\127", "\128\255|Hitem:1|h[x]|h\254",
    }
    for index, input in ipairs(corpus) do record(pack(input), "literal-" .. index) end
    return rows
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

local function observeExplicitPower(unit, name, api, unmodified)
    if not accessible(unit) then return { status = "restricted-input" } end
    local powers, ok = readField(Enum, "PowerType")
    local power, found
    if ok then power, found = readField(powers, name) end
    if not found or not accessible(power) then return { status = "unavailable-enum" } end
    if type(power) ~= "number" or power ~= power or power == math.huge or power == -math.huge then
        return { status = "unavailable-enum" }
    end
    local fn, available = readField(_G, api)
    if not available then return { status = "field-error" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if not accessible(unit) or not accessible(power) then return { status = "restricted-input" } end
    local values
    if api == "UnitPowerPercent" then values = pack(pcall(fn, unit, power, unmodified, nil))
    else values = pack(pcall(fn, unit, power, unmodified)) end
    if not values[1] then return { status = "call-error" } end
    local result = mapTuple(unpack(values, 2, values.n))
    result.status = "observed"
    return result
end

local function captureExplicitPower()
    local rows = {}
    for _, unit in ipairs({ "player", "target" }) do
        for _, name in ipairs({ "Mana", "Rage", "Energy" }) do
            for _, unmodified in ipairs({ false, true }) do
                for _, api in ipairs({ "UnitPower", "UnitPowerMax", "UnitPowerPercent" }) do
                    rows[#rows + 1] = { unit = unit, powerName = name, api = api, unmodified = unmodified,
                        result = observeExplicitPower(unit, name, api, unmodified) }
                end
            end
        end
    end
    return rows
end

local function captureHyperlinkModes(record, mode)
    if mode == "resource-color-input" then
        local function inputsAccessible(args)
            for index = 1, args.n do
                if not accessible(args[index]) then return false end
            end
            return true
        end
        local function call(owner, name, args, required, receiver)
            if not inputsAccessible(required) or not inputsAccessible(args) then
                return { status = "restricted-input" }
            end
            local fn, ok = readField(owner, name)
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            if not inputsAccessible(required) or not inputsAccessible(args)
                or (receiver and not accessible(receiver)) then
                return { status = "restricted-input" }
            end
            local values = pack(pcall(fn, unpack(args, 1, args.n)))
            if not values[1] then return { status = "call-error" } end
            local result = mapTuple(unpack(values, 2, values.n))
            result.status = "observed"
            return result, values[2]
        end
        local function isObject(value)
            return accessible(value) and (type(value) == "table" or type(value) == "userdata")
        end
        local result = { status = "unavailable-input", setup = { colors = {}, points = {} },
            health = { status = "unavailable-input" }, power = { status = "unavailable-input" } }
        record.resourceColorInput = result
        local curve
        result.setup.curve, curve = call(C_CurveUtil, "CreateColorCurve", pack(), pack())
        if result.setup.curve.status ~= "observed" or not isObject(curve) then return end
        local colors = {}
        for index, rgba in ipairs({ { 1, 0, 0, 1 }, { 0, 0, 1, 1 } }) do
            result.setup.colors[index], colors[index] = call(_G, "CreateColor", pack(unpack(rgba)), pack(curve))
            if result.setup.colors[index].status ~= "observed" or not isObject(colors[index]) then return end
        end
        for index, x in ipairs({ 0, 100 }) do
            result.setup.points[index] = call(curve, "AddPoint", pack(curve, x, colors[index]),
                pack(curve, colors[1], colors[2]), curve)
            if result.setup.points[index].status ~= "observed" then return end
        end
        if not isObject(curve) or not isObject(colors[1]) or not isObject(colors[2]) then return end
        result.status = "observed"
        result.health = call(_G, "UnitHealthPercent", pack("player", false, curve), pack())
        result.power = call(_G, "UnitPowerPercent", pack("player", nil, false, curve), pack())
        return
    end
    if mode == "explicit-power" then record.explicitPower = captureExplicitPower() end
    if mode == "hyperlinks" then record.hyperlinks = captureHyperlinks() end
    if mode == "hyperlinks-residual" then record.hyperlinksResidual = captureHyperlinksResidual() end
end

local function captureNames(mode)
    if mode == "threat-lead-read" then
        local function observeThreatLead(unit, mob)
            if not accessible(unit) or not accessible(mob) then return { status = "restricted-input" } end
            local fn, ok = readField(_G, "UnitThreatLeadSituation")
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            -- Despite its documented name mobGUID, the second argument is a UnitToken.
            if not accessible(unit) or not accessible(mob) then return { status = "restricted-input" } end
            local values = pack(pcall(fn, unit, mob))
            if not values[1] then return { status = "call-error" } end
            local result = mapTuple(unpack(values, 2, values.n))
            result.status = "observed"
            return result
        end
        local result = { pairs = {} }
        for _, mob in ipairs({ "target", "focus", "party1", "nonexistent" }) do
            local row = { unit = scalar("player"), mob = scalar(mob) }
            row.result = observeThreatLead("player", mob)
            result.pairs[#result.pairs + 1] = row
        end
        return result
    end
    if mode == "guid-identity" then return captureGuidIdentity() end
    if mode == "full-names" then return captureFullNames() end
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

local function captureActions(slot, mode)
    if mode == "action-state" or mode == "action-loss-control-duration" then
        local function query(name, usesSlot, global)
            if usesSlot and not accessible(slot) then return { status = "restricted-input" } end
            local namespace, ok = _G, true
            if not global then namespace, ok = readField(_G, "C_ActionBar") end
            if not ok then return { status = "field-error" } end
            local fn
            fn, ok = readField(namespace, name)
            if not ok then return { status = "field-error" } end
            if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
            -- Namespace lookup and function guards can revoke the parsed input.
            if usesSlot and not accessible(slot) then return { status = "restricted-input" } end
            local values
            if usesSlot then values = pack(pcall(fn, slot)) else values = pack(pcall(fn)) end
            if not values[1] then return { status = "call-error" } end
            local result
            if name == "GetActionLossOfControlCooldownDuration" then
                result = { n = values.n - 1, values = {} }
                for index = 2, math.min(values.n, 17) do
                    result.values[index - 1] = inspectDuration(values[index])
                end
                if values.n > 17 then result.truncated = true end
            else
                result = mapTuple(unpack(values, 2, values.n))
            end
            result.status = "observed"
            return result
        end
        if mode == "action-loss-control-duration" then
            local result = { slot = scalar(slot), identity = query("GetActionInfo", true, true) }
            result.duration = query("GetActionLossOfControlCooldownDuration", true)
            return result
        end
        local result = { slot = scalar(slot), identity = query("GetActionInfo", true, true),
            queries = {}, bars = {} }
        for _, name in ipairs({ "GetActionAutocast", "GetActionText", "GetActionUseCount",
            "HasRangeRequirements", "IsAttackAction", "IsAutoRepeatAction", "IsConsumableAction",
            "IsEquippedAction", "IsItemAction", "IsStackableAction", "IsUsableAction", "IsActionInRange" }) do
            result.queries[name] = query(name, true)
        end
        for _, name in ipairs({ "GetExtraBarIndex", "GetMultiCastBarIndex" }) do
            result.bars[name] = query(name, false)
        end
        return result
    end
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

local function capturePublication(mode)
    local rows = {}
    if mode == "error-code-publication" then
        for _, name in ipairs({
            "LE_GAME_ERR_CHARTER_NEIGHBORHOOD_OWNERSHIP_TRANSFER_SUCCESS",
            "LE_GAME_ERR_CHARTER_NEIGHBORHOOD_RENAME_NOTIFICATION_S",
            "LE_GAME_ERR_CHARTER_SIGNATURE_REMOVED",
            "LE_GAME_ERR_ENDEAVOR_REWARD_AVAILABLE",
            "LE_GAME_ERR_HOUSING_EXTERIOR_FAILSAFE_RESET",
            "LE_GAME_ERR_HOUSING_RESULT_COSMETIC_OWNER_NOT_IN_GUILD",
            "LE_GAME_ERR_HOUSING_RESULT_MISSING_PRIVATE_NEIGHBORHOOD_INVITE",
            "LE_GAME_ERR_HOUSING_RESULT_PLOT_NOT_VACANT",
            "LE_GAME_ERR_HOUSING_RESULT_PLOT_RESERVED",
            "LE_GAME_ERR_LFG_JOINED_TRAINING_GROUNDS_QUEUE",
            "LE_GAME_ERR_PVP_TRAINING_GROUNDS_DISABLED",
            "LE_GAME_ERR_SOLO_JOIN_TRAINING_GROUND",
        }) do
            local value, ok = readField(_G, name)
            local row = ok and scalar(value) or { status = "lookup-error" }
            if row.kind == "nil" then row.status = "missing" end
            row.name = name
            rows[#rows + 1] = row
        end
        return rows
    end
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

local function callbackInputsAccessible(cb, unit)
    return accessible("UNIT_HEALTH") and accessible(unit) and accessible(cb)
end

local function callbackCall(fn, cb, unit, guardInputs)
    if guardInputs and not callbackInputsAccessible(cb, unit) then return { status = "restricted-input" } end
    if not accessible(fn) or type(fn) ~= "function" then return { status = "missing-api" } end
    if guardInputs and not callbackInputsAccessible(cb, unit) then return { status = "restricted-input" } end
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

local function registerDuplicateCallbackLane(lane, record, register)
    lane.pendingAttempts = {}
    record.registrations, record.removals, record.removalAttempts = {}, {}, { 0, 0 }
    local complete = true
    for attempt = 1, 2 do
        -- An available call can register before throwing; keep its cleanup slot.
        lane.pendingAttempts[attempt] = accessible(register) and type(register) == "function"
        if lane.pendingAttempts[attempt] then
            record.registrations[attempt] = callbackCall(register, lane.callback, lane.unit, true)
        else
            record.registrations[attempt] = { status = "missing-api" }
        end
        local outcome = callbackOutcome(record.registrations[attempt])
        if outcome ~= "accepted" then complete, record.status = false, outcome end
    end
    if complete then record.status = "registered" end
    return complete
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
    if session.duplicate then
        session[name] = lane
        return registerDuplicateCallbackLane(lane, record, register)
    end
    -- Retain identity even after a call error: the API may have registered before throwing.
    lane.pending = accessible(register) and type(register) == "function"
    record.registration = callbackCall(register, lane.callback, unit)
    local outcome = callbackOutcome(record.registration)
    record.status = outcome == "accepted" and "registered" or outcome
    session[name] = lane
    return outcome == "accepted"
end

local function stopDuplicateCallbackLane(lane, record, unregister)
    local complete = true
    for attempt = 1, 2 do
        if lane.pendingAttempts[attempt] then
            record.removalAttempts[attempt] = record.removalAttempts[attempt] + 1
            record.removals[attempt] = callbackCall(unregister, lane.callback, lane.unit, true)
            local outcome = callbackOutcome(record.removals[attempt])
            if outcome == "accepted" then
                lane.pendingAttempts[attempt] = false
            else
                complete, record.status = false, "cleanup-" .. outcome
            end
        end
    end
    if complete then record.status = "removed" end
    return complete
end

local function stopCallbackLane(session, name, unregister)
    local lane, record = session[name], session.record[name]
    if lane.pendingAttempts then return stopDuplicateCallbackLane(lane, record, unregister) end
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
    local session = { record = record, receiving = true, duplicate = mode == "callbacks-duplicate-start" }
    if session.duplicate then record.duplicate = true end
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
    if mode == "callbacks-start" or mode == "callbacks-duplicate-start" or mode == "callbacks-stop" then
        controlCallbacks(mode, string.sub(label, 1, 128)); return
    end
    if mode == "events-start" or mode == "events-stop" then
        controlEvents(mode, string.sub(label, 1, 128)); return
    end
    local slot
    if mode == "action-loss-control-duration" or mode == "action-state" or mode == "actions" or mode == "spell-metadata" or mode == "spell-duration" or mode == "spellbook-metadata" or mode == "spellbook-duration" then
        slot, label = parseActionSlot(label)
        if slot == nil then print("Usage: /apicontract " .. mode .. " <integer-slot> <label>"); return end
    end
    if mode ~= "timeline-track-info" and mode ~= "heal-calculator-modes" and mode ~= "lfg-title-match-read" and mode ~= "lfg-playstyle-format" and mode ~= "crafting-schematic-read" and mode ~= "transmog-source-validity" and mode ~= "transmog-slot-visual-info" and mode ~= "error-code-publication" and mode ~= "resource-color-input" and mode ~= "timeline-source-counts" and mode ~= "timeline-lifecycle-read" and mode ~= "timeline-current-events" and mode ~= "cloak-helm-transition" and mode ~= "threat-lead-read" and mode ~= "guid-identity" and mode ~= "item-interaction-flags" and mode ~= "house-exterior-options" and mode ~= "expansion-audio-fields" and mode ~= "perks-criteria" and mode ~= "equipped-transmog-eligibility" and mode ~= "equipped-item-info" and mode ~= "action-loss-control-duration" and mode ~= "action-state" and mode ~= "combat-audio-settings-read" and mode ~= "encounter-warning-state" and mode ~= "ping-enabled" and mode ~= "explicit-power" and mode ~= "hyperlinks-residual" and mode ~= "full-names" and mode ~= "player-state-queries" and mode ~= "outfit-tooltip" and mode ~= "tradeskill-item-quality" and mode ~= "nameplate-metrics" and mode ~= "death-recap-current" and mode ~= "quest-favor" and mode ~= "empowered-stages" and mode ~= "unit-role-predicates" and mode ~= "stable-bonus-slot" and mode ~= "cooldown-viewer-read" and mode ~= "prey-quest-widgets" and mode ~= "major-faction-renown-rewards" and mode ~= "major-faction-journey" and mode ~= "training-grounds-structures" and mode ~= "training-grounds-state" and mode ~= "housing-catalog" and mode ~= "neighborhood-structures" and mode ~= "neighborhood-state" and mode ~= "sets-catalog" and mode ~= "custom-set-names" and mode ~= "outfit-state" and mode ~= "outfit-slots" and mode ~= "outfit-catalog" and mode ~= "spell-diminish-categories" and mode ~= "weekly-progress" and mode ~= "housing-preview-modes" and mode ~= "spellbook-duration" and mode ~= "spellbook-metadata" and mode ~= "unit-target-display" and mode ~= "unit-auras-current" and mode ~= "aura-time" and mode ~= "aura-display-count" and mode ~= "spell-duration" and mode ~= "spell-metadata" and mode ~= "public-queries" and mode ~= "item-binding" and mode ~= "statusbar-fill" and mode ~= "raid-markers" and mode ~= "abbreviations" and mode ~= "heal-calculator" and mode ~= "mapvalues" and mode ~= "cast-durations" and mode ~= "color-curves" and mode ~= "curve-edit" and mode ~= "curve-state" and mode ~= "resources" and mode ~= "hyperlinks" and mode ~= "actions" and mode ~= "all" and mode ~= "curves" and mode ~= "sex" and mode ~= "names" and mode ~= "numbers" and mode ~= "casts" and mode ~= "publication" then
        print("Usage: /apicontract [timeline-track-info|heal-calculator-modes|lfg-title-match-read|lfg-playstyle-format|crafting-schematic-read|transmog-source-validity|transmog-slot-visual-info|error-code-publication|resource-color-input|timeline-source-counts|timeline-lifecycle-read|timeline-current-events|cloak-helm-transition|threat-lead-read|guid-identity|item-interaction-flags|house-exterior-options|expansion-audio-fields|perks-criteria|equipped-transmog-eligibility|equipped-item-info|action-loss-control-duration|action-state|combat-audio-settings-read|encounter-warning-state|ping-enabled|explicit-power|hyperlinks-residual|full-names|player-state-queries|outfit-tooltip|tradeskill-item-quality|nameplate-metrics|death-recap-current|quest-favor|empowered-stages|unit-role-predicates|stable-bonus-slot|cooldown-viewer-read|all|curves|curve-state|curve-edit|color-curves|sex|names|numbers|casts|cast-durations|resources|hyperlinks|mapvalues|heal-calculator|abbreviations|raid-markers|statusbar-fill|item-binding|public-queries|aura-display-count|aura-time|unit-auras-current|unit-target-display|spellbook-metadata|spellbook-duration|housing-preview-modes|weekly-progress|spell-diminish-categories|outfit-catalog|outfit-slots|outfit-state|custom-set-names|sets-catalog|neighborhood-state|neighborhood-structures|housing-catalog|training-grounds-state|training-grounds-structures|major-faction-journey|major-faction-renown-rewards|publication|events-start|events-stop|callbacks-start|callbacks-duplicate-start|callbacks-stop] [label]")
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
        if mode == "spell-diminish-categories" then record.spellDiminishCategories = captureDiminishCategories() end
        if mode == "outfit-tooltip" then record.outfitTooltip = captureOutfitTooltip() end
        if mode == "outfit-catalog" then record.outfitCatalog = captureOutfitCatalog() end
        if mode == "outfit-slots" then record.outfitSlots = captureOutfitSlots() end
        if mode == "weekly-progress" then record.weeklyProgress = captureWeeklyProgress() end
        if mode == "cooldown-viewer-read" then record.cooldownViewerRead = captureCooldownViewerRead() end
        if mode == "housing-preview-modes" then record.housingPreviewModes = captureHousingPreviewModes() end
        if mode == "empowered-stages" then record.empoweredStages = captureEmpoweredStages() end
        if mode == "threat-lead-read" then record.threatLeadRead = captureNames(mode) end
        if mode == "guid-identity" then record.guidIdentity = captureNames(mode) end
        if mode == "full-names" then record.fullNames = captureNames(mode) end
        if mode == "unit-role-predicates" then record.unitRolePredicates = captureUnitRolePredicates() end
        if mode == "unit-target-display" then record.unitTargetDisplay = captureUnitTargetDisplay() end
        if mode == "unit-auras-current" then record.unitAurasCurrent = captureCurrentUnitAuras() end
        if mode == "aura-display-count" then record.auraDisplayCount = captureAuraPage(captureAuraSlot) end
        if mode == "aura-time" then record.auraTime = captureAuraPage(captureAuraTimeSlot) end
        if mode == "spell-duration" then record.spellDuration = captureSpellDuration(slot) end
        if mode == "spell-metadata" then record.spellMetadata = captureSpellMetadata(slot) end
        if mode == "spellbook-metadata" then record.spellbookMetadata = captureSpellbookMetadata(slot) end
        if mode == "spellbook-duration" then record.spellbookDuration = captureSpellbookDuration(slot) end
        if mode == "sets-catalog" then record.setsCatalog = captureSetsCatalog() end
        if mode == "custom-set-names" then record.customSetNames = captureCustomSetNames() end
        if mode == "transmog-source-validity" then record.transmogSourceValidity = captureCustomSetNames(mode) end
        if mode == "housing-catalog" then record.housingCatalog = captureHousingCatalog() end
        if mode == "major-faction-journey" then record.majorFactionJourney = captureMajorFactionJourney() end
        if mode == "major-faction-renown-rewards" then record.majorFactionRenownRewards = captureMajorFactionRenownRewards() end
        if mode == "prey-quest-widgets" then record.preyQuestWidgets = capturePreyQuestWidgets() end
        if mode == "training-grounds-structures" then record.trainingGroundsStructures = captureTrainingGroundsStructures() end
        if mode == "nameplate-metrics" then record.nameplateMetrics = captureNameplateMetrics() end
        if mode == "death-recap-current" then record.deathRecapCurrent = captureDeathRecapCurrent() end
        if mode == "player-state-queries" then record.playerStateQueries = capturePlayerStateQueries() end
        if mode == "cloak-helm-transition" then record.cloakHelmTransition = capturePlayerStateQueries(mode) end
        if mode == "stable-bonus-slot" then record.stableBonusSlot = captureStableBonusSlot() end
        if mode == "training-grounds-state" then record.trainingGroundsState = captureTrainingGroundsState() end
        if mode == "quest-favor" then record.questFavor = captureQuestFavor() end
        if mode == "neighborhood-state" then record.neighborhoodState = captureNeighborhoodState() end
        if mode == "lfg-title-match-read" then record.lfgTitleMatchRead = captureNeighborhoodStructures(mode) end
        if mode == "lfg-playstyle-format" then record.lfgPlaystyleFormat = captureNeighborhoodStructures(mode) end
        if mode == "crafting-schematic-read" then record.craftingSchematicRead = captureNeighborhoodStructures(mode) end
        if mode == "neighborhood-structures" then record.neighborhoodStructures = captureNeighborhoodStructures() end
        if mode == "timeline-track-info" then record.timelineTrackInfo = captureNeighborhoodStructures(mode) end
        if mode == "timeline-source-counts" then record.timelineSourceCounts = captureNeighborhoodStructures(mode) end
        if mode == "timeline-lifecycle-read" then record.timelineLifecycleRead = captureNeighborhoodStructures(mode) end
        if mode == "timeline-current-events" then record.timelineCurrentEvents = captureNeighborhoodStructures(mode) end
        if mode == "perks-criteria" then record.perksCriteria = captureNeighborhoodStructures(mode) end
        if mode == "house-exterior-options" then record.houseExteriorOptions = captureNeighborhoodStructures(mode) end
        if mode == "item-interaction-flags" then record.itemInteractionFlags = captureNeighborhoodStructures(mode) end
        if mode == "outfit-state" then record.outfitState = captureOutfitState() end
        if mode == "expansion-audio-fields" then record.expansionAudioFields = capturePublicQueries(mode) end
        if mode == "public-queries" then record.publicQueries = capturePublicQueries() end
        if mode == "ping-enabled" then record.pingEnabled = capturePublicQueries(mode) end
        if mode == "encounter-warning-state" then record.encounterWarningState = capturePublicQueries(mode) end
        if mode == "combat-audio-settings-read" then record.combatAudioSettingsRead = capturePublicQueries(mode) end
        if mode == "tradeskill-item-quality" then record.tradeskillItemQuality = captureTradeskillItemQuality() end
        if mode == "item-binding" then record.itemBinding = captureItemBinding() end
        if mode == "equipped-item-info" then record.equippedItemInfo = captureItemBinding(true) end
        if mode == "transmog-slot-visual-info" then
            record.transmogSlotVisualInfo = captureItemBinding("slot-visual-info")
        end
        if mode == "equipped-transmog-eligibility" then
            record.equippedTransmogEligibility = captureItemBinding("transmog-eligibility")
        end
        if mode == "statusbar-fill" then record.statusbarFill = captureStatusbarFill() end
        if mode == "raid-markers" then record.raidMarkers = captureRaidMarkers() end
        if mode == "abbreviations" then record.abbreviations = captureAbbreviations() end
        if mode == "heal-calculator" then record.healCalculator = captureHealCalculator() end
        if mode == "heal-calculator-modes" then record.healCalculatorModes = captureHealCalculator("modes") end
        if mode == "mapvalues" then record.mapvalues = captureMapValues() end
        if mode == "cast-durations" then record.castDurations = captureCastDurations() end
        if mode == "color-curves" then record.colorCurves = captureColorCurves() end
        if mode == "curve-edit" then record.curveEdit = captureCurveEdit() end
        if mode == "curve-state" then record.curveState = captureCurveState() end
        if mode == "resources" then record.resources = captureResources() end
        if mode == "actions" then record.actions = captureActions(slot) end
        if mode == "action-state" then record.actionState = captureActions(slot, mode) end
        if mode == "action-loss-control-duration" then record.actionLossControlDuration = captureActions(slot, mode) end
        captureHyperlinkModes(record, mode)
        if mode == "all" or mode == "curves" then record.curves = captureCurves() end
        if mode == "all" or mode == "sex" then record.sex = captureSex() end
        if mode == "all" or mode == "names" then record.names = captureNames() end
        if mode == "all" or mode == "numbers" then record.numbers = captureNumbers() end
        if mode == "all" or mode == "casts" then record.casts = captureCasts() end
        if mode == "error-code-publication" then record.errorCodePublication = capturePublication(mode) end
        if mode == "all" or mode == "publication" then record.publication = capturePublication() end
        record.status = "observed"
    end
    db.captures[#db.captures + 1] = record
end
