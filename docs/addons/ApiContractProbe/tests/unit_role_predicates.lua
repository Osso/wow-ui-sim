local root = assert(arg[1])
local passed = 0
local tokens = { "player", "target", "focus", "pet", "party1", "nonexistent", "invalid-unit-token", "" }
local names = { "UnitIsLieutenant", "UnitIsMinion", "UnitIsNPCAsPlayer" }
local secret = newproxy(true)
getmetatable(secret).__tostring = function() error("secret stringify") end
getmetatable(secret).__index = function() error("secret lookup") end
local forbiddenCalls = 0
local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded API") end
local function setup(factory)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return not not rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    for i, name in ipairs(names) do _G[name] = factory(i, name) end
    UnitNameFromGUID, UnitThreatLeadSituation = forbidden, forbidden
    forbiddenCalls = 0
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("unit-role-predicates " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "unit-role-predicates mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].unitRolePredicates)
end
local function test(name, fn)
    fn(); assert(forbiddenCalls == 0, "excluded API invoked")
    passed = passed + 1; print("PASS " .. name)
end

test("eight exact tokens and three independent one-argument calls", function()
    local calls = 0
    setup(function(i)
        return function(...)
            calls = calls + 1
            assert(select("#", ...) == 1)
            assert((...) == tokens[math.floor((calls - 1) / 3) + 1])
            assert(i == (calls - 1) % 3 + 1)
            return calls
        end
    end)
    local r = capture()
    assert(calls == 24 and #r.units == 8)
    for i, token in ipairs(tokens) do
        assert(r.units[i].unit.value == token)
        for j, name in ipairs(names) do
            local q = r.units[i].queries[name]
            assert(q.status == "observed" and q.n == 1 and q.values[1].value == (i - 1) * 3 + j)
        end
    end
end)

test("missing nonfunction and secret functions do not suppress peers", function()
    for _, bad in ipairs({ false, 12, secret, "missing" }) do
        for blocked, name in ipairs(names) do
            local calls = 0
            setup(function(i)
                if i == blocked then return bad == "missing" and nil or bad end
                return function() calls = calls + 1 end
            end)
            if bad == "missing" then _G[name] = nil end
            local r = capture()
            assert(calls == 16)
            for _, row in ipairs(r.units) do assert(row.queries[name].status == "missing-api") end
        end
    end
end)

test("zero returns nil holes opaque errors and restricted outputs", function()
    local calls = 0
    setup(function()
        return function()
            calls = calls + 1
            if calls == 1 then return end
            if calls == 2 then return nil end
            if calls == 3 then error(secret) end
            return nil, secret, nil, false
        end
    end)
    local r = capture()
    assert(calls == 24)
    assert(r.units[1].queries[names[1]].n == 0)
    assert(r.units[1].queries[names[2]].n == 1)
    assert(r.units[1].queries[names[2]].values[1].kind == "nil")
    assert(r.units[1].queries[names[3]].status == "call-error")
    local q = r.units[2].queries[names[1]]
    assert(q.n == 4 and q.values[1].kind == "nil" and q.values[2].status == "restricted")
    assert(q.values[3].kind == "nil" and q.values[4].value == false)
end)

test("every inaccessible token skips only its three calls", function()
    for _, blocked in ipairs(tokens) do
        local calls = 0
        setup(function() return function(unit) assert(unit ~= blocked); calls = calls + 1 end end)
        canaccessvalue = function(v) return not rawequal(v, secret) and v ~= blocked end
        local r = capture()
        assert(calls == 21)
        for i, token in ipairs(tokens) do
            if token == blocked then
                assert(r.units[i].unit.status == "restricted")
                for _, name in ipairs(names) do assert(r.units[i].queries[name].status == "restricted-input") end
            end
        end
    end
end)

test("both function guards revoke every token before each API call", function()
    for _, guard in ipairs({ "secret", "access" }) do
        for ti, blocked in ipairs(tokens) do
            for ni, name in ipairs(names) do
                local calls, revoked = 0, false
                setup(function() return function(unit) assert(not (revoked and unit == blocked)); calls = calls + 1 end end)
                local fn = _G[name]
                issecretvalue = function(v)
                    if guard == "secret" and rawequal(v, fn) and calls == (ti - 1) * 3 + ni - 1 then revoked = true end
                    return rawequal(v, secret)
                end
                canaccessvalue = function(v)
                    if guard == "access" and rawequal(v, fn) and calls == (ti - 1) * 3 + ni - 1 then revoked = true end
                    return not rawequal(v, secret) and not (revoked and v == blocked)
                end
                local r = capture()
                assert(revoked and calls == 24 - (4 - ni))
                assert(r.units[ti].queries[name].status == "restricted-input")
            end
        end
    end
end)

test("peer failure and function replacement remain independent", function()
    local calls = 0
    setup(function(i)
        return function()
            calls = calls + 1
            if i == 1 then
                UnitIsMinion = function() return "replacement" end
                error(secret)
            end
            return "original"
        end
    end)
    local r = capture()
    assert(calls == 16)
    for _, row in ipairs(r.units) do
        assert(row.queries[names[1]].status == "call-error")
        assert(row.queries[names[2]].values[1].value == "replacement")
        assert(row.queries[names[3]].values[1].value == "original")
    end
end)

test("output guards revoke token before subsequent queries", function()
    local marker, revoked, calls = {}, false, 0
    setup(function() return function(unit) assert(not (revoked and unit == "player")); calls = calls + 1; return marker end end)
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and v == "player")
    end
    local r = capture()
    assert(calls == 22 and r.units[1].queries[names[2]].status == "restricted-input")
    assert(r.units[1].queries[names[3]].status == "restricted-input")
end)

test("tuple string label snapshot and total call bounds", function()
    local calls, values = 0, {}
    for i = 1, 20 do values[i] = string.rep("x", 300) end
    setup(function() return function() calls = calls + 1; return unpack(values) end end)
    local r = capture(string.rep("l", 200))
    local q = r.units[1].queries[names[1]]
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(calls == 240 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("missing and throwing access APIs fail closed", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        local calls = 0
        setup(function() return function() calls = calls + 1 end end)
        _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("unit-role-predicates missing")
        assert(calls == 0 and ApiContractProbeDB.captures[1].status == "missing-access-api")
        setup(function() return function() calls = calls + 1 end end)
        _G[guard] = function() error("guard failure") end
        capture()
        assert(calls == 0)
    end
end)

test("all excludes role predicates", function()
    local calls = 0
    setup(function() return function() calls = calls + 1 end end)
    SlashCmdList.APICONTRACTPROBE("all context")
    assert(calls == 0 and ApiContractProbeDB.captures[1].unitRolePredicates == nil)
end)
print(string.format("%d unit-role-predicates fixtures passed", passed))
