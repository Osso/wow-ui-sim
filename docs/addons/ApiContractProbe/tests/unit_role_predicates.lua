local root = assert(arg[1])
local passed = 0
local tokens = { "player", "target", "focus", "pet", "party1", "nonexistent", "invalid-unit-token", "" }
local names = { "UnitIsLieutenant", "UnitIsMinion", "UnitIsNPCAsPlayer" }
local secret = newproxy(true)
getmetatable(secret).__tostring = function() error("secret stringify") end
getmetatable(secret).__index = function() error("secret lookup") end
local forbiddenCalls = 0
local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded API") end
local function setup(factory, optional)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return not not rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    for i, name in ipairs(names) do _G[name] = factory(i, name) end
    local npc = UnitIsNPCAsPlayer
    if type(npc) == "function" then
        UnitIsNPCAsPlayer = function(...)
            if select("#", ...) == 0 or (...) == nil then
                if optional then return optional(...) end
                return
            end
            return npc(...)
        end
    end
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
test("omitted and explicit nil are separate calls after unchanged token matrix", function()
    local calls, omissions = {}, {}
    setup(function(i)
        return function(...)
            calls[#calls + 1] = { api = i, n = select("#", ...), unit = (...) }
            return i
        end
    end, function(...)
        omissions[#omissions + 1] = select("#", ...)
        assert(select("#", ...) <= 1 and (...) == nil)
        assert(#calls == 24)
        return nil, false, nil
    end)
    local r = capture()
    assert(#calls == 24 and #omissions == 2 and omissions[1] == 0 and omissions[2] == 1)
    for i, call in ipairs(calls) do
        assert(call.n == 1 and call.api == (i - 1) % 3 + 1)
        assert(call.unit == tokens[math.floor((i - 1) / 3) + 1])
    end
    for _, key in ipairs({ "omitted", "explicitNil" }) do
        local q = assert(r.omittedInput[key])
        assert(q.status == "observed" and q.n == 3)
        assert(q.values[1].kind == "nil" and q.values[2].value == false and q.values[3].kind == "nil")
    end
end)

test("optional errors zero returns and replacement are independent", function()
    for _, failFirst in ipairs({ true, false }) do
        local calls = 0
        setup(function() return function() end end, function(...)
            calls = calls + 1
            if (select("#", ...) == 0) == failFirst then error(secret) end
            return
        end)
        local r = capture().omittedInput
        assert(calls == 2)
        assert(r[failFirst and "omitted" or "explicitNil"].status == "call-error")
        assert(r[failFirst and "explicitNil" or "omitted"].n == 0)
    end
    setup(function() return function() end end, function(...)
        assert(select("#", ...) == 0)
        UnitIsNPCAsPlayer = function(...)
            assert(select("#", ...) == 1 and (...) == nil)
            return "replacement"
        end
        return "first"
    end)
    local r = capture().omittedInput
    assert(r.omitted.values[1].value == "first" and r.explicitNil.values[1].value == "replacement")
end)

test("optional function guards deny each call independently", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        for blocked = 1, 2 do
            local tokensSeen, checks, optionalCalls = 0, 0, 0
            setup(function() return function() tokensSeen = tokensSeen + 1 end end,
                function() optionalCalls = optionalCalls + 1 end)
            local fn = UnitIsNPCAsPlayer
            _G[guard] = function(v)
                local denied = rawequal(v, secret)
                if tokensSeen == 24 and rawequal(v, fn) then
                    checks = checks + 1
                    denied = checks == blocked
                end
                if guard == "issecretvalue" then return denied end
                return not denied
            end
            local r = capture().omittedInput
            assert(tokensSeen == 24 and optionalCalls == 1)
            assert(r[blocked == 1 and "omitted" or "explicitNil"].status == "missing-api")
            assert(r[blocked == 1 and "explicitNil" or "omitted"].status == "observed")
        end
    end
end)

test("explicit nil is rechecked after both function guards", function()
    for _, phase in ipairs({ "secret", "access" }) do
        local revoked, omittedCalls, nilCalls = false, 0, 0
        setup(function() return function() end end, function(...)
            if select("#", ...) == 0 then omittedCalls = omittedCalls + 1
            else nilCalls = nilCalls + 1; assert(not revoked, "revoked nil forwarded") end
        end)
        local fn = UnitIsNPCAsPlayer
        issecretvalue = function(v)
            if phase == "secret" and omittedCalls == 1 and rawequal(v, fn) then revoked = true end
            return rawequal(v, secret)
        end
        canaccessvalue = function(v)
            if phase == "access" and omittedCalls == 1 and rawequal(v, fn) then revoked = true end
            return not rawequal(v, secret) and not (v == nil and revoked)
        end
        local r = capture().omittedInput
        assert(revoked and omittedCalls == 1 and nilCalls == 0)
        assert(r.omitted.status == "observed" and r.explicitNil.status == "restricted-input")
    end
end)

test("missing optional function and inaccessible nil stay bounded observations", function()
    setup(function() return function() end end, function()
        UnitIsNPCAsPlayer = nil
        return secret
    end)
    local r = capture().omittedInput
    assert(r.omitted.values[1].status == "restricted" and r.explicitNil.status == "missing-api")
    local optionalCalls = 0
    setup(function() return function() end end, function(...)
        optionalCalls = optionalCalls + 1; assert(select("#", ...) == 0)
    end)
    canaccessvalue = function(v) return v ~= nil and not rawequal(v, secret) end
    r = capture().omittedInput
    assert(optionalCalls == 1 and r.omitted.status == "observed" and r.explicitNil.status == "restricted-input")
end)

test("all 26 calls and optional result bounds respect snapshot limit", function()
    local calls, values = 0, {}
    for i = 1, 20 do values[i] = string.rep("b", 300) end
    local function observe() calls = calls + 1; return unpack(values) end
    setup(function() return observe end, observe)
    local r = capture(string.rep("l", 200)).omittedInput
    for _, key in ipairs({ "omitted", "explicitNil" }) do
        assert(r[key].n == 20 and r[key].truncated and #r[key].values == 16)
        assert(#r[key].values[16].value == 256)
    end
    for _ = 2, 11 do capture() end
    assert(calls == 260 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
end)
print(string.format("%d unit-role-predicates fixtures passed", passed))
