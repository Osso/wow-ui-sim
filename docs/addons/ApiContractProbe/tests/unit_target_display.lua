local root = assert(arg[1])
local passed = 0
local tokens = { "player", "target", "focus", "party1", "nonexistent", "invalid-unit-token", "" }
local secret = newproxy(true)
getmetatable(secret).__tostring = function() error("secret stringify") end
getmetatable(secret).__index = function() error("secret lookup") end
local forbiddenCalls = 0
local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded API called") end
local function setup(fn)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    UnitShouldDisplaySpellTargetName = fn
    UnitSpellTargetClass, UnitSpellTargetName = forbidden, forbidden
    UnitCastingInfo, UnitChannelInfo = forbidden, forbidden
    forbiddenCalls = 0
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("unit-target-display " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "unit-target-display mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].unitTargetDisplay)
end
local function test(name, fn)
    fn(); assert(forbiddenCalls == 0, "excluded target/cast API invoked")
    passed = passed + 1; print("PASS " .. name)
end

test("all seven tokens twice independently with exact arguments", function()
    local calls = 0
    setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 1)
        assert((...) == tokens[math.floor((calls - 1) / 2) + 1])
        return calls
    end)
    local r = capture()
    assert(calls == 14 and #r.units == 7)
    for i, token in ipairs(tokens) do
        assert(r.units[i].unit.value == token)
        for j = 1, 2 do
            local q = r.units[i].observations[j]
            assert(q.status == "observed" and q.n == 1 and q.values[1].value == (i - 1) * 2 + j)
        end
    end
end)

test("missing nonfunction and restricted functions remain independent", function()
    for _, fn in ipairs({ false, 4, secret }) do
        setup(fn)
        local r = capture()
        for _, row in ipairs(r.units) do
            for _, q in ipairs(row.observations) do assert(q.status == "missing-api") end
        end
    end
    setup(nil); assert(capture().units[1].observations[1].status == "missing-api")
end)

test("zero nil opaque errors and output access are preserved", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil end
        if calls == 3 then error(secret) end
        return nil, secret, nil, false
    end)
    local r = capture()
    assert(calls == 14)
    assert(r.units[1].observations[1].n == 0)
    assert(r.units[1].observations[2].n == 1 and r.units[1].observations[2].values[1].kind == "nil")
    assert(r.units[2].observations[1].status == "call-error")
    local q = r.units[2].observations[2]
    assert(q.n == 4 and q.values[1].kind == "nil" and q.values[2].status == "restricted")
    assert(q.values[3].kind == "nil" and q.values[4].value == false)
end)

test("every restricted token skips only its two calls", function()
    for _, blocked in ipairs(tokens) do
        local calls = 0
        setup(function(unit) assert(unit ~= blocked); calls = calls + 1 end)
        canaccessvalue = function(v) return not rawequal(v, secret) and v ~= blocked end
        local r = capture()
        assert(calls == 12)
        for i, token in ipairs(tokens) do
            if token == blocked then
                assert(r.units[i].unit.status == "restricted")
                for _, q in ipairs(r.units[i].observations) do assert(q.status == "restricted-input") end
            end
        end
    end
end)

test("function guards revoke tokens before forwarding at every position", function()
    for _, guard in ipairs({ "secret", "access" }) do
        for _, blocked in ipairs(tokens) do
            local calls, revoked = 0, false
            local fn = function(unit) assert(unit ~= blocked); calls = calls + 1 end
            setup(fn)
            issecretvalue = function(v)
                if guard == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if guard == "access" and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and v == blocked)
            end
            local r = capture()
            assert(calls == 12)
            for i, token in ipairs(tokens) do
                if token == blocked then
                    for _, q in ipairs(r.units[i].observations) do assert(q.status == "restricted-input") end
                end
            end
        end
    end
end)

test("second observation repeats function guard and survives first failure", function()
    local calls = 0
    local fn
    fn = function()
        calls = calls + 1
        if calls == 1 then
            UnitShouldDisplaySpellTargetName = function() return "replacement" end
            error(secret)
        end
    end
    setup(fn)
    local r = capture()
    assert(calls == 1 and r.units[1].observations[1].status == "call-error")
    assert(r.units[1].observations[2].values[1].value == "replacement")
end)

test("tuple string label snapshot and call bounds", function()
    local calls, values = 0, {}
    for i = 1, 20 do values[i] = string.rep("x", 300) end
    setup(function() calls = calls + 1; return unpack(values) end)
    local r = capture(string.rep("l", 200))
    local q = r.units[1].observations[1]
    assert(q.n == 20 and q.truncated and #q.values == 16)
    assert(#q.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1 and calls == 140)
end)

test("missing access APIs fail closed and all excludes target queries", function()
    local calls = 0
    setup(function() calls = calls + 1 end)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("unit-target-display missing")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(function() calls = calls + 1 end)
    -- all includes its existing cast recorder; this fixture isolates target exclusion.
    UnitCastingInfo, UnitChannelInfo = nil, nil
    SlashCmdList.APICONTRACTPROBE("all context")
    assert(calls == 0 and ApiContractProbeDB.captures[1].unitTargetDisplay == nil)
end)
print(string.format("%d unit-target-display fixtures passed", passed))
