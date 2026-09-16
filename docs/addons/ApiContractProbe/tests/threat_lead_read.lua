local root = assert(arg[1])
local passed, failed = 0, 0
local mobs = { "target", "focus", "party1", "nonexistent" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspected") end
getmetatable(secret).__tostring = function() error("secret stringified") end
local function setup(fn)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    UnitThreatLeadSituation = fn
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("threat-lead-read " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "threat-lead-read mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].threatLeadRead)
end
local function test(name, fn)
    local old = getmetatable(_G)
    local ok, err = pcall(fn)
    setmetatable(_G, old)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("four original token pairs use exactly two arguments without GUID production", function()
    local calls = 0
    setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 2)
        local unit, mob = ...
        assert(unit == "player" and mob == mobs[calls])
        return calls + 0.25
    end)
    UnitGUID = function() error("GUID producer called") end
    local r = capture()
    assert(calls == 4 and #r.pairs == 4)
    for i, row in ipairs(r.pairs) do
        assert(row.unit.value == "player" and row.mob.value == mobs[i])
        assert(row.result.n == 1 and row.result.values[1].value == i + 0.25)
    end
end)

test("zero returns nil positions conditional secrets and opaque errors stay independent", function()
    setup(function(_, mob)
        if mob == "target" then return end
        if mob == "focus" then return nil, secret, "tail", nil end
        if mob == "party1" then error(secret) end
        return 17
    end)
    local r = capture()
    assert(r.pairs[1].result.n == 0)
    local result = r.pairs[2].result
    assert(result.n == 4 and result.values[1].kind == "nil" and result.values[4].kind == "nil")
    assert(result.values[2].status == "restricted" and result.values[3].value == "tail")
    assert(r.pairs[3].result.status == "call-error" and r.pairs[4].result.values[1].value == 17)
end)

test("both original tokens are checked before API lookup", function()
    for _, token in ipairs({ "player", "target", "focus", "party1", "nonexistent" }) do
        local lookups, calls = 0, 0
        setup(nil)
        canaccessvalue = function(v) return not rawequal(v, secret) and v ~= token end
        setmetatable(_G, { __index = function(_, key)
            if key == "UnitThreatLeadSituation" then
                lookups = lookups + 1
                return function() calls = calls + 1 end
            end
        end })
        local r = capture()
        local expected = token == "player" and 0 or 3
        assert(lookups == expected and calls == expected)
        for i, row in ipairs(r.pairs) do
            if token == "player" or token == mobs[i] then assert(row.result.status == "restricted-input") end
        end
        setmetatable(_G, nil)
    end
end)

test("global lookup and both function guards recheck either token at every pair", function()
    for position = 1, 4 do
        for _, token in ipairs({ "player", mobs[position] }) do
            for _, phase in ipairs({ "global", "lookup", "secret", "access" }) do
                local revoked, lookups, calls = false, 0, 0
                local fn = function(unit, mob)
                    assert(not (revoked and (unit == token or mob == token)))
                    calls = calls + 1
                end
                setup(nil)
                setmetatable(_G, { __index = function(_, key)
                    if key == "UnitThreatLeadSituation" then
                        lookups = lookups + 1
                        if phase == "lookup" and lookups == position then revoked = true end
                        return fn
                    end
                end })
                issecretvalue = function(v)
                    if phase == "secret" and rawequal(v, fn) and lookups == position then revoked = true end
                    return rawequal(v, secret)
                end
                canaccessvalue = function(v)
                    if phase == "global" and rawequal(v, _G) and lookups + 1 == position then revoked = true end
                    if phase == "access" and rawequal(v, fn) and lookups == position then revoked = true end
                    return not rawequal(v, secret) and not (revoked and v == token)
                end
                local r = capture()
                assert(r.pairs[position].result.status == "restricted-input", phase .. position .. token)
                assert(calls == (token == "player" and position - 1 or 3))
                setmetatable(_G, nil)
            end
        end
    end
end)

test("missing denied and throwing API lookups do not hide later pairs", function()
    for _, outcome in ipairs({ "missing", "secret", "denied", "throw" }) do
        local lookups, calls = 0, 0
        local denied = function() error("denied called") end
        setup(nil)
        canaccessvalue = function(v) return not rawequal(v, secret) and not rawequal(v, denied) end
        setmetatable(_G, { __index = function(_, key)
            if key == "UnitThreatLeadSituation" then
                lookups = lookups + 1
                if lookups == 1 then
                    if outcome == "throw" then error(secret) end
                    if outcome == "secret" then return secret end
                    if outcome == "denied" then return denied end
                    return nil
                end
                return function() calls = calls + 1; return "ok" end
            end
        end })
        local r = capture()
        assert(calls == 3 and lookups == 4)
        assert(r.pairs[1].result.status == (outcome == "throw" and "field-error" or "missing-api"))
        assert(r.pairs[4].result.values[1].value == "ok")
        setmetatable(_G, nil)
    end
end)

test("serialization revocation blocks later inputs and leaves later outputs opaque", function()
    local marker, revoked, calls = {}, false, 0
    setup(function() calls = calls + 1; return marker, "later" end)
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and (v == "player" or v == "later"))
    end
    local r = capture()
    assert(calls == 1 and r.pairs[1].result.values[2].status == "restricted")
    for i = 2, 4 do assert(r.pairs[i].result.status == "restricted-input") end
end)

test("missing or failing accessibility APIs fail closed", function()
    for _, name in ipairs({ "issecretvalue", "canaccessvalue" }) do
        setup(function() error("API invoked") end)
        _G[name] = nil
        SlashCmdList.APICONTRACTPROBE("threat-lead-read missing")
        assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
        setup(function() error("API invoked") end)
        _G[name] = function() error(secret) end
        local r = capture()
        for _, row in ipairs(r.pairs) do assert(row.result.status == "restricted-input") end
    end
end)

test("sixteen positions strings labels and ten snapshots are bounded", function()
    local calls, values = 0, {}
    for i = 1, 18 do values[i] = string.rep("X", 300) end
    setup(function() calls = calls + 1; return unpack(values) end)
    local r = capture(string.rep("L", 200))
    assert(r.pairs[1].result.n == 18 and r.pairs[1].result.truncated)
    assert(#r.pairs[1].result.values == 16 and #r.pairs[1].result.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 12 do capture() end
    assert(calls == 40 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 2)
end)

test("opaque objects are collectible and all excludes the manual mode", function()
    local calls, weak = 0, setmetatable({}, { __mode = "v" })
    setup(function()
        calls = calls + 1
        local value = newproxy(true)
        getmetatable(value).__index = function() error("object inspected") end
        getmetatable(value).__tostring = function() error("object stringified") end
        weak[calls] = value
        return value, {}, function() error("returned function invoked") end
    end)
    capture()
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
    UnitGUID = nil
    SlashCmdList.APICONTRACTPROBE("all fixture")
    assert(calls == 4 and ApiContractProbeDB.captures[2].threatLeadRead == nil)
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
