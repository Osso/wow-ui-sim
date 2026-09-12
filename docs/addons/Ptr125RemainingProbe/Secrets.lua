local _, Probe = ...

local function inconclusive(run, label, message)
    run.status = "inconclusive"
    Probe.note(run, label, "inconclusive", message)
end

local function capture(run, label, mode, fn, ...)
    return Probe.capture(run, "secrets:" .. label, mode, fn, Probe.pack(...), false)
end

local function is_secret(check, value)
    local ok, result = pcall(check, value)
    return ok and type(result) == "boolean" and result == true
end

local function sample_key(run, check)
    for _, unit in ipairs({ "player", "target" }) do
        for _, name in ipairs({ "UnitHealth", "UnitPower" }) do
            local result = capture(run, "sample:" .. name .. ":" .. unit,
                "direct", rawget(_G, name), unit)
            if result[1] and is_secret(check, result[2]) then return true, result[2] end
        end
    end
    return false
end

local function sample_clock(run, check)
    local namespace = rawget(_G, "C_UnitAuras")
    local getter = type(namespace) == "table" and rawget(namespace, "GetAuraDataByIndex")
    for _, unit in ipairs({ "player", "target" }) do
        for _, filter in ipairs({ "HELPFUL", "HARMFUL" }) do
            for index = 1, 3 do
                local label = "sample:aura:" .. unit .. ":" .. filter .. ":" .. index
                local aura = capture(run, label, "direct", getter, unit, index, filter)
                if aura[1] then
                    local clock = capture(run, label .. ":expirationTime", "direct", function()
                        return aura[2].expirationTime
                    end)
                    if clock[1] and is_secret(check, clock[2]) then return true, clock[2] end
                end
            end
        end
    end
    return false
end

local function modes()
    if type(rawget(_G, "securecallfunction")) == "function" then
        return { "direct", "securecallfunction" }
    end
    return { "direct" }
end

local function verify_secret(run, label, check, fn, args)
    for _, mode in ipairs(modes()) do
        local result = Probe.capture(run, "secrets:" .. label .. ":" .. mode,
            mode, fn, args, false)
        if result[1] and is_secret(check, result[2]) then return true end
    end
    inconclusive(run, "secrets:" .. label, "Could not verify an observed secret in the fresh fixture")
    return false
end

local function accessible_number(check, access, value)
    if type(check) ~= "function" or type(access) ~= "function" then return false end
    local checked, secret = pcall(check, value)
    if not checked or type(secret) ~= "boolean" or secret ~= false then return false end
    local inspected, accessible = pcall(access, value)
    if not inspected or type(accessible) ~= "boolean" or accessible ~= true then return false end
    return type(value) == "number"
end

local function table_controls(run, functions)
    for _, fixture in ipairs({ { "empty", {} }, { "populated", { control = true } } }) do
        for _, name in ipairs({ "count", "getcountinfo", "isempty" }) do
            for _, mode in ipairs(modes()) do
                capture(run, "table:control:" .. fixture[1] .. ":" .. name .. ":" .. mode,
                    mode, functions[name], fixture[2])
            end
        end
    end
end

local function table_fixture(run, check, key, seedMode, functions)
    local object = {}
    local label = "table:" .. seedMode
    local seeded = capture(run, label .. ":seed", seedMode, functions.rawset, object, key, true)
    if not seeded[1] then
        inconclusive(run, "secrets:" .. label, "Native secret-key insertion did not succeed")
        return
    end
    if not verify_secret(run, label .. ":verify-key", check, functions.next, Probe.pack(object)) then
        return
    end
    for _, name in ipairs({ "count", "getcountinfo", "isempty" }) do
        for _, mode in ipairs({ "direct", "securecallfunction" }) do
            capture(run, label .. ":" .. name .. ":" .. mode, mode, functions[name], object)
        end
    end
end

local function capture_map_methods(run, object, label)
    local result = capture(run, label .. ":methods", "direct", function()
        return object.SignalAt, object.CancelAllSignals, object.GetSignalTime,
            object.GetSignalCount, object.HasSignal, object.GetNextSignal
    end)
    if not result[1] then return nil end
    return {
        SignalAt = result[2], CancelAllSignals = result[3], GetSignalTime = result[4],
        GetSignalCount = result[5], HasSignal = result[6], GetNextSignal = result[7],
    }
end

local function clear_and_verify(run, label, mode, object, methods, check)
    capture(run, label, mode, methods.CancelAllSignals, object)
    local count = capture(run, label .. ":count", mode, methods.GetSignalCount, object)
    if not count[1] then return false end
    local ok, empty = pcall(function()
        if not accessible_number(check, rawget(_G, "canaccessvalue"), count[2]) then
            return false
        end
        return count[2] == 0
    end)
    return ok and empty == true
end

local function cleanup_map(run, label, object, methods, check)
    local done, attempts = false, 0
    local function clean()
        if done then return true end
        if attempts >= 2 then return false end
        attempts = attempts + 1
        local mode = type(rawget(_G, "securecallfunction")) == "function"
            and "securecallfunction" or "direct"
        done = clear_and_verify(run, label .. ":cleanup", mode, object, methods, check)
        if not done then
            local message = attempts == 1
                and "Cleanup count unverified; deferred cleanup will retry once"
                or "Cleanup count remains unverified after the bounded retry"
            inconclusive(run, "secrets:" .. label .. ":cleanup", message)
        end
        return done
    end
    Probe.defer(run, "secrets:" .. label, function() clean() end)
    return clean
end

local function map_queries(run, label, object, methods)
    for _, name in ipairs({ "GetSignalCount", "HasSignal", "GetSignalTime", "GetNextSignal" }) do
        for _, mode in ipairs({ "direct", "securecallfunction" }) do
            local args = (name == "HasSignal" or name == "GetSignalTime")
                and Probe.pack(object, 1) or Probe.pack(object)
            Probe.capture(run, "secrets:" .. label .. ":" .. name .. ":" .. mode,
                mode, methods[name], args, false)
        end
    end
end

local function map_controls(run, check, label, object, methods)
    map_queries(run, label .. ":control:empty", object, methods)
    local now = capture(run, label .. ":control:clock", "direct", rawget(_G, "GetTime"))
    local access = rawget(_G, "canaccessvalue")
    if not now[1] or not accessible_number(check, access, now[2]) then
        inconclusive(run, "secrets:" .. label, "Ordinary control clock could not be verified")
        return false
    end
    local time = now[2]
    if time ~= time or time == math.huge or time == -math.huge then
        inconclusive(run, "secrets:" .. label, "Ordinary control clock is not finite")
        return false
    end
    local seeded = capture(run, label .. ":control:seed", "direct", methods.SignalAt, object, 1, time + 60)
    map_queries(run, label .. ":control:populated", object, methods)
    local cleared = clear_and_verify(run, label .. ":control:clear", "direct", object, methods, check)
    if not seeded[1] or not cleared then
        inconclusive(run, "secrets:" .. label, "Ordinary map control could not be cleared and verified")
        return false
    end
    return true
end

local function map_fixture(run, check, clock, seedMode, factory)
    local label = "map:" .. seedMode
    local created = capture(run, label .. ":create", "direct", factory, function() end)
    if not created[1] then
        inconclusive(run, "secrets:" .. label, "Native timed-map factory did not succeed")
        return
    end
    local object = created[2]
    local methods = capture_map_methods(run, object, label)
    if not methods or type(methods.CancelAllSignals) ~= "function" then
        inconclusive(run, "secrets:" .. label, "Cannot capture native methods and cleanup before mutation")
        return
    end
    local clean = cleanup_map(run, label, object, methods, check)
    if not map_controls(run, check, label, object, methods) then
        clean()
        return
    end
    local seeded = capture(run, label .. ":seed", seedMode, methods.SignalAt, object, 1, clock)
    if not seeded[1] then
        inconclusive(run, "secrets:" .. label, "Native secret-clock insertion did not succeed")
    elseif verify_secret(run, label .. ":verify-time", check,
        methods.GetSignalTime, Probe.pack(object, 1)) then
        map_queries(run, label, object, methods)
    end
    if clean() then
        map_queries(run, label .. ":recovery", object, methods)
    end
end

function Probe.actions.secrets(run)
    local check = rawget(_G, "issecretvalue")
    if type(check) ~= "function" then
        inconclusive(run, "secrets:classifier", "issecretvalue unavailable; no fixtures constructed")
        return
    end
    Probe.note(run, "secrets:scope", "observation",
        "Direct and securecallfunction are invocation paths only; wrapper success does not establish privilege")
    local functions = { rawset = rawset, next = next, count = rawget(table, "count"),
        getcountinfo = rawget(table, "getcountinfo"), isempty = rawget(table, "isempty") }
    table_controls(run, functions)
    local timers = rawget(_G, "C_Timer")
    local factory = type(timers) == "table" and rawget(timers, "NewTimedSignalMap")
    local hasKey, key = sample_key(run, check)
    if hasKey then
        for _, mode in ipairs(modes()) do table_fixture(run, check, key, mode, functions) end
    else
        inconclusive(run, "secrets:key", "No secret observed in bounded native health/power samples")
    end
    local hasClock, clock = sample_clock(run, check)
    if hasClock then
        for _, mode in ipairs(modes()) do map_fixture(run, check, clock, mode, factory) end
    else
        inconclusive(run, "secrets:clock", "No secret aura expirationTime observed; no substitute clock used")
    end
end
