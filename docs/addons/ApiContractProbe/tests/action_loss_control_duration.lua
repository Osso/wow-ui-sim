local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local methodNames = { "GetTotalDuration", "GetElapsedDuration", "GetRemainingDuration", "GetElapsedPercent",
    "GetRemainingPercent", "GetStartTime", "GetEndTime", "GetClockTime", "GetModRate", "HasExpired" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function setup(control, producer)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture" end
    time = function() return 42 end
    GetActionInfo = control
    C_ActionBar = { GetActionLossOfControlCooldownDuration = producer }
    UnitCastingDuration, UnitChannelDuration, UnitEmpoweredChannelDuration = nil, nil, nil
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(input)
    SlashCmdList.APICONTRACTPROBE(input or "action-loss-control-duration 17 sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "action loss-control duration mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].actionLossControlDuration,
        "action loss-control duration output absent")
end
local function duration(callback)
    local object = {}
    for _, name in ipairs(methodNames) do
        object[name] = function(self, ...)
            assert(rawequal(self, object) and select("#", ...) == 0)
            return callback(name)
        end
    end
    return setmetatable(object, { __index = function(_, name) error("unexpected method " .. name) end })
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("original signed slots and macro controls reach only the requested duration query", function()
    for _, slot in ipairs({ -17, 0, 9007199254740991, -9007199254740991 }) do
        local controls, calls, reads = 0, 0, 0
        setup(function(...)
            assert(select("#", ...) == 1 and (...) == slot)
            controls = controls + 1; return "macro", 999, nil
        end, function(...)
            assert(select("#", ...) == 1 and (...) == slot)
            calls = calls + 1
            return duration(function() reads = reads + 1; return 12.5, nil, false end), nil
        end)
        setmetatable(C_ActionBar, { __index = function(_, name) error("excluded API " .. name) end })
        local r = capture("action-loss-control-duration " .. string.format("%.0f", slot) .. " label words")
        assert(r.slot.value == slot and r.identity.n == 3 and r.identity.values[1].value == "macro")
        assert(controls == 1 and calls == 1 and reads == 10 and r.duration.n == 2)
        assert(r.duration.values[2].kind == "nil")
        for _, name in ipairs(methodNames) do
            local m = r.duration.values[1].methods[name]
            assert(m.n == 3 and m.values[2].kind == "nil" and m.values[3].value == false)
        end
        assert(ApiContractProbeDB.captures[1].label == "label words")
    end
end)

test("control failure and nonspell identities never gate the independent query", function()
    for _, scenario in ipairs({ "missing", "throw", "secret", "zero", "nil", "item" }) do
        local calls, control = 0
        if scenario == "throw" then control = function() error(secret) end
        elseif scenario == "secret" then control = secret
        elseif scenario == "zero" then control = function() end
        elseif scenario == "nil" then control = function() return nil end
        elseif scenario == "item" then control = function() return "item", 22 end end
        setup(control, function() calls = calls + 1; return nil, false end)
        local r = capture()
        assert(calls == 1 and r.duration.n == 2 and r.duration.values[1].kind == "nil")
    end
end)

test("missing restricted and failing queries preserve their control and raw arity", function()
    for _, scenario in ipairs({ "missing", "secret", "lookup", "throw", "zero", "nil" }) do
        local controls = 0
        setup(function() controls = controls + 1; return "item" end, nil)
        if scenario == "secret" then C_ActionBar.GetActionLossOfControlCooldownDuration = secret
        elseif scenario == "lookup" then setmetatable(C_ActionBar, { __index = function() error(secret) end })
        elseif scenario == "throw" then C_ActionBar.GetActionLossOfControlCooldownDuration = function() error(secret) end
        elseif scenario == "zero" then C_ActionBar.GetActionLossOfControlCooldownDuration = function() end
        elseif scenario == "nil" then C_ActionBar.GetActionLossOfControlCooldownDuration = function() return nil end end
        local r = capture()
        assert(controls == 1 and r.identity.values[1].value == "item")
        if scenario == "zero" then assert(r.duration.n == 0)
        elseif scenario == "nil" then assert(r.duration.n == 1 and r.duration.values[1].kind == "nil")
        else assert(r.duration.status ~= "observed" and r.duration.values == nil) end
    end
end)

test("slot is rechecked after namespace lookup and both function guards", function()
    for _, phase in ipairs({ "namespace", "lookup", "secret-guard", "access-guard", "control" }) do
        local revoked, calls = false, 0
        local fn = function() calls = calls + 1 end
        setup(function() if phase == "control" then revoked = true end; return "macro" end, fn)
        local api = C_ActionBar
        if phase == "lookup" then
            api.GetActionLossOfControlCooldownDuration = nil
            setmetatable(api, { __index = function() revoked = true; return fn end })
        end
        issecretvalue = function(v)
            if phase == "secret-guard" and rawequal(v, fn) then revoked = true end
            return rawequal(v, secret)
        end
        canaccessvalue = function(v)
            if (phase == "namespace" and rawequal(v, api)) or
                (phase == "access-guard" and rawequal(v, fn)) then revoked = true end
            return not rawequal(v, secret) and not (rawequal(v, 17) and revoked)
        end
        assert(capture().duration.status == "restricted-input" and calls == 0)
    end
end)

test("each receiver is rechecked after method lookup and function guards", function()
    for _, phase in ipairs({ "lookup", "secret-guard", "access-guard" }) do
        for _, revokedName in ipairs(methodNames) do
            local revoked, forbiddenCalls = false, 0
            local object, methods = {}, {}
            for _, name in ipairs(methodNames) do methods[name] = function(self)
                assert(rawequal(self, object)); if revoked then forbiddenCalls = forbiddenCalls + 1 end
                return 7
            end end
            setmetatable(object, { __index = function(_, name)
                if phase == "lookup" and name == revokedName then revoked = true end
                return methods[name]
            end })
            setup(nil, function() return object end)
            issecretvalue = function(v)
                if phase == "secret-guard" and rawequal(v, methods[revokedName]) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "access-guard" and rawequal(v, methods[revokedName]) then revoked = true end
                return not rawequal(v, secret) and not (rawequal(v, object) and revoked)
            end
            local r = capture().duration.values[1]
            assert(r.methods[revokedName].status ~= "observed" and forbiddenCalls == 0)
        end
    end
end)

test("method failures nils and secret objects remain independent and opaque", function()
    local calls = 0
    local object = duration(function(name)
        calls = calls + 1
        if name == "GetTotalDuration" then error(secret) end
        if name == "GetElapsedDuration" then return end
        return nil, secret, "raw"
    end)
    setup(nil, function() return secret, nil, object, false end)
    local r = capture().duration
    assert(r.n == 4 and r.values[1].status == "restricted" and r.values[2].kind == "nil")
    assert(r.values[4].value == false and calls == 10)
    assert(r.values[3].methods.GetTotalDuration.status == "call-error")
    assert(r.values[3].methods.GetElapsedDuration.n == 0)
    assert(r.values[3].methods.GetRemainingDuration.values[2].status == "restricted")
end)

test("sixteen return positions ten snapshots and scalar limits bound all work", function()
    local queries, controls, reads = 0, 0, 0
    local objects, long = {}, string.rep("x", 300)
    for i = 1, 17 do objects[i] = duration(function()
        reads = reads + 1; local v = {}; for j = 1, 17 do v[j] = long end
        return unpack(v, 1, 17)
    end) end
    setup(function() controls = controls + 1; return long end, function()
        queries = queries + 1; return unpack(objects, 1, 17)
    end)
    for _ = 1, 11 do capture("action-loss-control-duration 17 " .. long) end
    assert(queries == 10 and controls == 10 and reads == 1600)
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local c = ApiContractProbeDB.captures[1]
    assert(#c.label == 128 and #c.actionLossControlDuration.identity.values[1].value == 256)
    local d = c.actionLossControlDuration.duration
    assert(d.n == 17 and d.truncated and #d.values == 16)
    local m = d.values[16].methods.GetTotalDuration
    assert(m.n == 17 and m.truncated and #m.values == 16 and #m.values[1].value == 256)
end)

test("objects are collectible and cast retention counters remain isolated", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(nil, function()
        local object = duration(function() return 8 end)
        weak[1] = object; return object
    end)
    local retained = duration(function() return 13 end)
    UnitCastingDuration = function(unit) if unit == "player" then return retained end end
    SlashCmdList.APICONTRACTPROBE("cast-durations first")
    local first = ApiContractProbeDB.captures[1].castDurations
    assert(first.capture == 1)
    local ref = first.units.player.casting.values[1].observationRef
    capture()
    collectgarbage("collect"); collectgarbage("collect")
    assert(weak[1] == nil)
    SlashCmdList.APICONTRACTPROBE("cast-durations second")
    local second = ApiContractProbeDB.captures[3].castDurations
    assert(second.capture == 2 and #second.previous == 1 and second.previous[1].observationRef == ref)
    assert(second.previous[1].observation.methods.GetTotalDuration.values[1].value == 13)
end)

test("all and invalid slot syntax never invoke the duration API", function()
    local calls = 0
    setup(nil, function() calls = calls + 1 end)
    for _, token in ipairs({ "", "1.5", "abc", "9007199254740992", "1e2" }) do
        SlashCmdList.APICONTRACTPROBE("action-loss-control-duration " .. token)
    end
    assert(calls == 0 and (not ApiContractProbeDB or #ApiContractProbeDB.captures == 0))
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and ApiContractProbeDB.captures[1].actionLossControlDuration == nil)
end)

test("missing access APIs fail closed before control or query execution", function()
    local calls = 0
    setup(function() calls = calls + 1 end, function() calls = calls + 1 end)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("action-loss-control-duration 17")
    assert(ApiContractProbeDB and ApiContractProbeDB.captures[1].status == "missing-access-api")
    assert(calls == 0 and ApiContractProbeDB.captures[1].actionLossControlDuration == nil)
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
