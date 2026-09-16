local root = assert(arg[1])
local passed = 0
local tokens = { "player", "target", "focus", "party1", "nonexistent", "invalid-unit-token", "" }
local keys = { "durations", "percentagesWithoutHold", "percentagesWithHold" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local excluded = 0
local function setup(factory)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    UnitEmpoweredStageDurations = factory(1)
    UnitEmpoweredStagePercentages = factory(2)
    excluded = 0
    CastSpellByName = function() excluded = excluded + 1; error("excluded cast") end
    UnitEmpoweredChannelDuration = CastSpellByName
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("empowered-stages " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "empowered-stages mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].empoweredStages)
end
local function test(name, fn)
    fn(); assert(excluded == 0)
    passed = passed + 1; print("PASS " .. name)
end

test("seven tokens and exact duration false true calls", function()
    local calls = 0
    setup(function(api)
        return function(...)
            calls = calls + 1
            local unit, hold = ...
            local position = (calls - 1) % 3 + 1
            assert(unit == tokens[math.floor((calls - 1) / 3) + 1])
            assert(api == (position == 1 and 1 or 2))
            assert(select("#", ...) == (position == 1 and 1 or 2))
            if position > 1 then assert(hold == (position == 3)) end
            return { calls, false, "raw" }
        end
    end)
    local r = capture()
    assert(calls == 21 and #r.units == 7)
    for i, token in ipairs(tokens) do
        assert(r.units[i].unit.value == token)
        for j, key in ipairs(keys) do
            local q = r.units[i].queries[key]
            assert(q.n == 1 and q.entries[1].value == (i - 1) * 3 + j)
            assert(q.entries[2].value == false and q.entries[3].value == "raw")
        end
    end
end)

test("raw arity nil holes errors and only first returned table", function()
    local calls = 0
    local later = setmetatable({}, { __index = function() error("later return inspected") end })
    setup(function() return function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil, later end
        if calls == 3 then error(secret) end
        return { [2] = secret, [3] = 150, [4] = 0 / 0 }, nil, later
    end end)
    local r = capture()
    assert(calls == 21)
    assert(r.units[1].queries.durations.n == 0)
    assert(r.units[1].queries.percentagesWithoutHold.n == 2)
    assert(r.units[1].queries.percentagesWithoutHold.entries == nil)
    assert(r.units[1].queries.percentagesWithHold.status == "call-error")
    local q = r.units[2].queries.durations
    assert(q.n == 3 and q.values[2].kind == "nil")
    assert(q.entries[1].kind == "nil" and q.entries[2].status == "restricted")
    assert(q.entries[3].value == 150 and q.entries[4].status == "nonfinite")
end)

test("missing and restricted functions leave peers independent", function()
    for api = 1, 2 do
        for _, bad in ipairs({ false, secret, 42 }) do
            local calls = 0
            setup(function(i)
                if i == api then return bad end
                return function() calls = calls + 1 end
            end)
            capture(); assert(calls == (api == 1 and 14 or 7))
        end
    end
end)

test("function guards revoke every token before forwarding", function()
    for _, guard in ipairs({ "secret", "access" }) do
        for ti, token in ipairs(tokens) do
            for position = 1, 3 do
                local calls, revoked = 0, false
                setup(function() return function(unit)
                    assert(not (revoked and unit == token)); calls = calls + 1
                end end)
                local fn = position == 1 and UnitEmpoweredStageDurations or UnitEmpoweredStagePercentages
                local function trigger(v)
                    if rawequal(v, fn) and calls == (ti - 1) * 3 + position - 1 then revoked = true end
                end
                issecretvalue = function(v)
                    if guard == "secret" then trigger(v) end
                    return rawequal(v, secret)
                end
                canaccessvalue = function(v)
                    if guard == "access" then trigger(v) end
                    return not rawequal(v, secret) and not (revoked and v == token)
                end
                local r = capture()
                assert(revoked and calls == 21 - (4 - position))
                assert(r.units[ti].queries[keys[position]].status == "restricted-input")
            end
        end
    end
end)

test("each index rechecks the returned table and stops unsafe reads", function()
    for blocked = 1, 8 do
        local list, revoked, reads = nil, false, 0
        local marker = {}
        list = setmetatable({}, { __index = function(_, i)
            assert(not revoked and i <= 8); reads = reads + 1
            if i == blocked then return marker end
            return i
        end })
        local calls = 0
        setup(function() return function()
            calls = calls + 1
            if calls == 1 then return list end
        end end)
        canaccessvalue = function(v)
            if rawequal(v, marker) then revoked = true end
            return not rawequal(v, secret) and not (revoked and rawequal(v, list))
        end
        local q = capture().units[1].queries.durations
        assert(reads == blocked and calls == 21)
        if blocked < 8 then assert(q.entries[blocked + 1].status == "field-error") end
    end
end)

test("tuple serialization revokes list before indexing", function()
    local revoked, reads = false, 0
    local list = setmetatable({}, { __index = function() reads = reads + 1; error("revoked list read") end })
    local marker = {}
    setup(function() return function() return list, marker end end)
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, list))
    end
    capture(); assert(reads == 0)
end)

test("entry lookup failures and peer API errors remain independent", function()
    local calls = 0
    setup(function(api) return function(_, hold)
        calls = calls + 1
        if api == 2 and not hold then error(secret) end
        return setmetatable({}, { __index = function(_, i)
            if i == 2 then error(secret) end
            return i
        end })
    end end)
    local r = capture(); assert(calls == 21)
    for _, row in ipairs(r.units) do
        assert(row.queries.percentagesWithoutHold.status == "call-error")
        assert(row.queries.durations.entries[2].status == "field-error")
        assert(row.queries.percentagesWithHold.entries[3].value == 3)
    end
end)

test("duration method lookup errors stay opaque and objects are collectible", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() return function()
        local object = newproxy(true)
        getmetatable(object).__index = function() error("duration method lookup") end
        getmetatable(object).__tostring = function() error("duration stringify") end
        weak[#weak + 1] = object
        return { object, secret }
    end end)
    local q = capture().units[1].queries.durations
    assert(q.entries[1].kind == "userdata" and q.entries[1].methods.GetTotalDuration.status == "field-error")
    assert(q.entries[2].status == "restricted")
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
end)

test("tuple entry string label and snapshot bounds", function()
    local calls, reads = 0, 0
    setup(function() return function()
        calls = calls + 1
        local list = setmetatable({}, { __index = function(_, i)
            assert(i <= 8); reads = reads + 1; return string.rep("x", 300)
        end })
        local values = { list }
        for i = 2, 20 do values[i] = string.rep("y", 300) end
        return unpack(values)
    end end)
    local q = capture(string.rep("l", 200)).units[1].queries.durations
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.entries == 8)
    assert(#q.entries[1].value == 256 and #q.values[2].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(calls == 210 and reads == 210 * 8)
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("manual excluded from all and unavailable guards fail closed", function()
    local calls = 0
    setup(function() return function() calls = calls + 1 end end)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0)
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        setup(function() return function() calls = calls + 1 end end)
        _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("empowered-stages")
        assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
        setup(function() return function() calls = calls + 1 end end)
        _G[guard] = function() error("guard unavailable") end
        capture(); assert(calls == 0)
    end
end)
local methods = {
    "GetTotalDuration", "GetElapsedDuration", "GetRemainingDuration", "GetElapsedPercent",
    "GetRemainingPercent", "GetStartTime", "GetEndTime", "GetClockTime", "GetModRate", "HasExpired",
}

test("duration whitelist preserves receivers tuples and percentage opacity", function()
    local calls, lookups = 0, 0
    local object = newproxy(true)
    getmetatable(object).__index = function(_, name)
        lookups = lookups + 1
        local allowed = false
        for _, method in ipairs(methods) do if method == name then allowed = true end end
        assert(allowed, "non-whitelisted method")
        return function(receiver)
            assert(rawequal(receiver, object)); calls = calls + 1
            if name == "GetElapsedDuration" then error(secret) end
            if name == "HasExpired" then return end
            return 12, nil, string.rep("m", 300)
        end
    end
    setup(function() return function() return { object, nil, secret, 17 }, nil end end)
    local r = capture()
    assert(calls == 70 and lookups == 70)
    for _, row in ipairs(r.units) do
        local q = row.queries.durations
        assert(q.n == 2 and q.values[1].kind == "table" and q.values[2].kind == "nil")
        assert(q.entries[1].kind == "userdata" and q.entries[1].status == "observed")
        assert(q.entries[1].methods.GetTotalDuration.n == 3)
        assert(q.entries[1].methods.GetTotalDuration.values[2].kind == "nil")
        assert(#q.entries[1].methods.GetTotalDuration.values[3].value == 256)
        assert(q.entries[1].methods.GetElapsedDuration.status == "call-error")
        assert(q.entries[1].methods.HasExpired.n == 0)
        assert(q.entries[2].kind == "nil" and q.entries[3].status == "restricted")
        assert(q.entries[4].value == 17)
        assert(row.queries.percentagesWithoutHold.entries[1].methods == nil)
        assert(row.queries.percentagesWithHold.entries[1].methods == nil)
    end
end)

test("each method lookup and function guard can revoke its original receiver", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        for blocked, name in ipairs(methods) do
            local revoked, calls = false, 0
            local object, target = {}, nil
            setmetatable(object, { __index = function(_, key)
                assert(not revoked)
                local fn = function(receiver)
                    assert(rawequal(receiver, object) and not revoked)
                    calls = calls + 1; return calls
                end
                if key == name then
                    target = fn
                    if phase == "lookup" then revoked = true end
                end
                return fn
            end })
            setup(function(api) return function(unit)
                if api == 1 and unit == "player" then return { object } end
            end end)
            issecretvalue = function(v)
                if phase == "secret" and target and rawequal(v, target) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "access" and target and rawequal(v, target) then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, object))
            end
            local entry = capture().units[1].queries.durations.entries[1]
            assert(revoked and calls == blocked - 1)
            assert(entry.methods[name].status == "restricted-object")
        end
    end
end)

test("method output can revoke the next list entry without suppressing percentages", function()
    local revoked, calls = false, 0
    local marker, second = {}, {}
    local first = { GetTotalDuration = function() return marker end }
    setmetatable(second, { __index = function() error("revoked entry inspected") end })
    setup(function() return function() calls = calls + 1; return { first, second } end end)
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, second))
    end
    local r = capture()
    assert(calls == 21 and r.units[1].queries.durations.entries[2].status == "restricted")
    assert(r.units[1].queries.percentagesWithHold.entries[1].methods == nil)
end)

test("560 method calls per snapshot and method return bounds", function()
    local methodCalls, producers = 0, 0
    local object = setmetatable({}, { __index = function()
        return function()
            methodCalls = methodCalls + 1
            local values = {}; for i = 1, 20 do values[i] = string.rep("z", 300) end
            return unpack(values)
        end
    end })
    setup(function() return function()
        producers = producers + 1
        return setmetatable({}, { __index = function(_, i) assert(i <= 8); return object end })
    end end)
    local q = capture().units[1].queries.durations.entries[1].methods.GetTotalDuration
    assert(methodCalls == 560 and producers == 21)
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
    for _ = 2, 11 do capture() end
    assert(methodCalls == 5600 and producers == 210 and ApiContractProbeDB.dropped == 1)
end)

test("current duration receivers are collectible and cast retention is isolated", function()
    local weak = setmetatable({}, { __mode = "v" })
    local oldReads = 0
    local retained = { GetTotalDuration = function() oldReads = oldReads + 1; return 1 end }
    setup(function() return function()
        local object = newproxy(true)
        getmetatable(object).__index = function()
            return function(receiver) assert(rawequal(receiver, object)); return 2 end
        end
        weak[#weak + 1] = object
        return { object }
    end end)
    UnitCastingDuration = function() return retained end
    UnitChannelDuration = function() return nil end
    UnitEmpoweredChannelDuration = function() return nil end
    SlashCmdList.APICONTRACTPROBE("cast-durations before")
    local before = oldReads
    capture(); capture()
    assert(oldReads == before)
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
    SlashCmdList.APICONTRACTPROBE("cast-durations after")
    assert(oldReads > before)
end)
print(string.format("%d empowered-stages fixtures passed", passed))
