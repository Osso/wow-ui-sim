local root = assert(arg[1])
local passed, failed = 0, 0
local names = { "Encounter", "Script", "EditMode" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local forbiddenCalls = 0
local function forbidden()
    forbiddenCalls = forbiddenCalls + 1
    error("excluded timeline operation")
end
local function setup(fn)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(value) return rawequal(value, secret) end
    canaccessvalue = function(value) return not rawequal(value, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { EncounterTimelineEventSource = { Encounter = 10.5, Script = -20.5, EditMode = 0, Extra = 999 } }
    C_EncounterTimeline = {
        GetEventCountBySource = fn, GetEventList = forbidden,
        AddScriptEvent = forbidden, CancelScriptEvent = forbidden, FinishScriptEvent = forbidden,
        PauseScriptEvent = forbidden, ResumeScriptEvent = forbidden,
        AddEditModeEvents = forbidden, CancelEditModeEvents = forbidden, SetEventIconTextures = forbidden,
    }
    C_AddOns = { LoadAddOn = forbidden }
    LoadAddOn = forbidden
    forbiddenCalls = 0
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("timeline-source-counts " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "timeline-source-counts mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].timelineSourceCounts,
        "timeline-source-counts capture absent")
end
local function observation(result, position)
    return result.sources[math.floor((position - 1) / 2) + 1].observations[(position - 1) % 2 + 1]
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok and forbiddenCalls ~= 0 then ok, err = false, "excluded operation invoked" end
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("three fixed published values, two independent exact one-argument calls", function()
    local calls, inputs = 0, { 10.5, 10.5, -20.5, -20.5, 0, 0 }
    setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 1 and (...) == inputs[calls])
        return calls * 3
    end)
    local result = capture()
    assert(calls == 6 and #result.sources == 3)
    for index, row in ipairs(result.sources) do assert(row.name == names[index]) end
    for index = 1, 6 do
        local value = observation(result, index)
        assert(value.status == "observed" and value.n == 1)
        assert(value.input.value == inputs[index] and value.values[1].value == index * 3)
    end
end)

test("published values are re-read without deduplication or numeric fallback", function()
    local calls = 0
    setup(function(value)
        calls = calls + 1
        assert(value == (calls == 1 and 10.5 or 77.25))
        for _, name in ipairs(names) do Enum.EncounterTimelineEventSource[name] = 77.25 end
        return value
    end)
    local result = capture()
    assert(calls == 6 and observation(result, 2).input.value == 77.25)
end)

test("missing, nonnumeric, nonfinite and secret enum members skip only their calls", function()
    local invalid = { false, "0", {}, function() end, math.huge, -math.huge, 0/0, secret }
    for _, name in ipairs(names) do
        for index = 0, #invalid do
            local calls = 0
            setup(function() calls = calls + 1; return 5 end)
            Enum.EncounterTimelineEventSource[name] = index == 0 and nil or invalid[index]
            local result = capture()
            assert(calls == 4)
            for _, row in ipairs(result.sources) do
                for _, value in ipairs(row.observations) do
                    assert(value.status == (row.name == name and "unavailable-enum" or "observed"))
                end
            end
        end
    end
end)

test("publication containers are guarded before lookup", function()
    for _, level in ipairs({ "root", "members" }) do
        for _, guard in ipairs({ "secret", "access" }) do
            local reads, calls = 0, 0
            setup(function() calls = calls + 1 end)
            local container = setmetatable({}, { __index = function() reads = reads + 1; error("denied lookup") end })
            if level == "root" then Enum = container else Enum.EncounterTimelineEventSource = container end
            issecretvalue = function(value)
                return rawequal(value, secret) or (guard == "secret" and rawequal(value, container))
            end
            canaccessvalue = function(value)
                return not rawequal(value, secret) and not (guard == "access" and rawequal(value, container))
            end
            assert(observation(capture(), 1).status == "unavailable-enum")
            assert(reads == 0 and calls == 0)
        end
    end
    setup(function() error("must not call") end)
    Enum = setmetatable({}, { __index = function() error(secret) end })
    assert(observation(capture(), 6).status == "unavailable-enum")
end)

test("namespace and function denials preserve explicit independent outcomes", function()
    for _, value in ipairs({ false, 12, secret }) do
        setup(value)
        assert(observation(capture(), 1).status == "missing-api")
    end
    setup(nil)
    assert(observation(capture(), 6).status == "missing-api")
    for _, namespace in ipairs({ secret, setmetatable({}, { __index = function() error(secret) end }) }) do
        setup(function() error("must not call") end)
        C_EncounterTimeline = namespace
        assert(observation(capture(), 1).status == "field-error")
    end
end)

test("every call rechecks its original value after API lookup and function guards", function()
    for _, phase in ipairs({ "namespace", "lookup", "function-secret", "function-access" }) do
        for position = 1, 6 do
            local lookups, calls, revoked = 0, 0, false
            local blocked = ({ 10.5, -20.5, 0 })[math.floor((position - 1) / 2) + 1]
            local fn = function(value)
                assert(not (revoked and value == blocked), "revoked value forwarded")
                calls = calls + 1
                return calls
            end
            setup(fn)
            local namespace = setmetatable({}, { __index = function(_, key)
                assert(key == "GetEventCountBySource")
                if phase ~= "namespace" then lookups = lookups + 1 end
                if phase == "lookup" and lookups == position then revoked = true end
                return fn
            end })
            C_EncounterTimeline = namespace
            issecretvalue = function(value)
                if phase == "function-secret" and rawequal(value, fn) and lookups == position then revoked = true end
                return rawequal(value, secret)
            end
            canaccessvalue = function(value)
                if phase == "namespace" and rawequal(value, namespace) then
                    lookups = lookups + 1
                    if lookups == position then revoked = true end
                end
                if phase == "function-access" and rawequal(value, fn) and lookups == position then revoked = true end
                return not rawequal(value, secret) and not (revoked and rawequal(value, blocked))
            end
            local result = capture()
            assert(observation(result, position).status == "restricted-input")
            assert(calls == 6 - (position % 2 == 1 and 2 or 1))
        end
    end
end)

test("value observation and prior query output can revoke future forwarding", function()
    local checks, calls, revoked = 0, 0, false
    setup(function() calls = calls + 1; return 9 end)
    canaccessvalue = function(value)
        if rawequal(value, 10.5) then
            checks = checks + 1
            if checks == 1 then revoked = true; return true end
            return not revoked
        end
        return not rawequal(value, secret)
    end
    assert(observation(capture(), 1).status == "restricted-input" and calls == 4)
    calls, revoked = 0, false
    local output = {}
    setup(function() calls = calls + 1; return output end)
    canaccessvalue = function(value)
        if rawequal(value, output) then revoked = true end
        return not rawequal(value, secret) and not (revoked and rawequal(value, 10.5))
    end
    assert(observation(capture(), 2).status == "unavailable-enum" and calls == 5)
end)

test("zero returns, nil holes, opaque errors and fresh replacement are independent", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil end
        if calls == 3 then error(secret) end
        return nil, secret, false, nil
    end)
    local result = capture()
    assert(calls == 6 and observation(result, 1).n == 0 and observation(result, 2).n == 1)
    assert(observation(result, 3).status == "call-error")
    local value = observation(result, 4)
    assert(value.n == 4 and value.values[1].kind == "nil" and value.values[2].status == "restricted")
    assert(value.values[3].value == false and value.values[4].kind == "nil")
    setup(function()
        C_EncounterTimeline.GetEventCountBySource = function() return "replacement" end
        error(secret)
    end)
    result = capture()
    assert(observation(result, 1).status == "call-error")
    assert(observation(result, 2).values[1].value == "replacement")
end)

test("tuple, string, label, snapshot and total call bounds", function()
    local calls, values = 0, {}
    for index = 1, 20 do values[index] = string.rep("x", 300) end
    setup(function() calls = calls + 1; return unpack(values) end)
    local result = capture(string.rep("l", 200))
    local value = observation(result, 1)
    assert(value.n == 20 and value.truncated and #value.values == 16)
    assert(#value.values[1].value == 256 and value.values[1].truncated)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(calls == 60 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("opaque output objects are not inspected or retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function()
        local object = newproxy(true)
        getmetatable(object).__index = forbidden
        getmetatable(object).__tostring = forbidden
        weak[#weak + 1] = object
        return object, { nested = object }, function() forbidden() end
    end)
    local result = capture()
    assert(observation(result, 1).values[1].kind == "userdata")
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
end)

test("missing or throwing access guards fail closed and all excludes source counts", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        local calls = 0
        setup(function() calls = calls + 1 end)
        _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("timeline-source-counts missing")
        assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
        setup(function() calls = calls + 1 end)
        _G[guard] = function() error(secret) end
        assert(observation(capture(), 1).status == "unavailable-enum" and calls == 0)
    end
    local calls = 0
    setup(function() calls = calls + 1 end)
    SlashCmdList.APICONTRACTPROBE("all context")
    assert(calls == 0 and ApiContractProbeDB.captures[1].timelineSourceCounts == nil)
end)

print(string.format("%d passed, %d failed timeline-source-counts fixtures", passed, failed))
if failed > 0 then os.exit(1) end
