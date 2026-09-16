local root = assert(arg[1])
local passed, failed = 0, 0
local tokens = { "player", "target", "party1" }
local queries = { "UnitClassFromGUID", "UnitNameFromGUID" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspected") end
getmetatable(secret).__tostring = function() error("secret stringified") end
local function setup(producer, class, name)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    UnitGUID, UnitClassFromGUID, UnitNameFromGUID = producer, class, name
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("guid-identity " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "guid-identity mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].guidIdentity)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("exact tokens and original binary long GUIDs preserve class and name tuples", function()
    local calls, guids = {}, {}
    for i, unit in ipairs(tokens) do guids[unit] = "G\0\255" .. string.rep(tostring(i), 300) end
    setup(function(...)
        assert(select("#", ...) == 1)
        local unit = ...
        calls[#calls + 1] = "producer:" .. unit
        return guids[unit]
    end, function(...)
        assert(select("#", ...) == 1)
        local guid = ...
        local index = (#calls + 2) / 3
        assert(guid == guids[tokens[index]])
        calls[#calls + 1] = "class"
        return "Class", "CLASS", 7
    end, function(...)
        assert(select("#", ...) == 1)
        local index = (#calls + 1) / 3
        assert((...) == guids[tokens[index]])
        calls[#calls + 1] = "name"
        return "Name", "Realm"
    end)
    local r = capture()
    assert(#calls == 9 and #r.units == 3)
    for i, row in ipairs(r.units) do
        assert(row.unit.value == tokens[i] and calls[(i - 1) * 3 + 1] == "producer:" .. tokens[i])
        assert(row.producer.n == 1 and #row.producer.values[1].value == 256)
        assert(row.queries.UnitClassFromGUID.n == 3 and row.queries.UnitClassFromGUID.values[3].value == 7)
        assert(row.queries.UnitNameFromGUID.n == 2 and row.queries.UnitNameFromGUID.values[2].value == "Realm")
    end
end)

test("only accessible first string GUIDs are forwarded without parsing", function()
    for _, value in ipairs({ secret, false, 17, {}, function() end }) do
        local calls = 0
        setup(function() return value, "not-the-first-result" end,
            function() calls = calls + 1 end, function() calls = calls + 1 end)
        local r = capture()
        assert(calls == 0)
        for _, row in ipairs(r.units) do
            assert(row.queries.UnitClassFromGUID.status == (value == secret and "restricted-input" or "unavailable-input"))
        end
    end
    for _, guid in ipairs({ "", "not-a-parsed-guid" }) do
        local calls = 0
        setup(function() return guid end, function(v) assert(v == guid); calls = calls + 1 end,
            function(v) assert(v == guid); calls = calls + 1 end)
        capture(); assert(calls == 6)
    end
end)

test("producer zero nil and opaque error do not suppress peer units", function()
    for _, outcome in ipairs({ "zero", "nil", "error" }) do
        local calls = 0
        setup(function(unit)
            if unit == "player" then
                if outcome == "zero" then return end
                if outcome == "nil" then return nil, "ignored" end
                error(secret)
            end
            return unit .. "-guid"
        end, function() calls = calls + 1 end, function() calls = calls + 1 end)
        local r = capture()
        assert(calls == 4 and r.units[1].queries.UnitNameFromGUID.status == "unavailable-input")
        if outcome == "error" then assert(r.units[1].producer.status == "call-error")
        else assert(r.units[1].producer.n == (outcome == "zero" and 0 or 2)) end
    end
end)

test("query failures and secret tuple positions remain independent", function()
    local names = 0
    setup(function(unit) return unit end, function(guid)
        if guid == "player" then error(secret) end
        if guid == "target" then return secret, "CLASS", nil end
        return
    end, function() names = names + 1; return nil, secret end)
    local r = capture()
    assert(names == 3 and r.units[1].queries.UnitClassFromGUID.status == "call-error")
    assert(r.units[2].queries.UnitClassFromGUID.n == 3)
    assert(r.units[2].queries.UnitClassFromGUID.values[1].status == "restricted")
    assert(r.units[2].queries.UnitClassFromGUID.values[2].value == "CLASS")
    assert(r.units[2].queries.UnitClassFromGUID.values[3].kind == "nil")
    assert(r.units[3].queries.UnitClassFromGUID.n == 0)
    assert(r.units[1].queries.UnitNameFromGUID.n == 2 and r.units[1].queries.UnitNameFromGUID.values[1].kind == "nil")
    assert(r.units[1].queries.UnitNameFromGUID.values[2].status == "restricted")
end)

test("both producer function guards recheck every original token", function()
    for _, token in ipairs(tokens) do
        for _, phase in ipairs({ "secret", "access" }) do
            local revoked, calls = false, 0
            local fn = function(unit) calls = calls + 1; return unit .. "-guid" end
            setup(fn, function() end, function() end)
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "access" and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and v == token)
            end
            local r = capture()
            assert(calls == 2)
            for i, unit in ipairs(tokens) do
                if unit == token then assert(r.units[i].producer.status == "restricted-input") end
            end
        end
    end
end)

test("every downstream lookup and function guard rechecks original GUID", function()
    for _, token in ipairs(tokens) do
        for _, api in ipairs(queries) do
            for _, phase in ipairs({ "lookup", "secret", "access", "global" }) do
                local revoked, calls = false, 0
                local guid = token .. "-guid"
                local fn = function(v) assert(v ~= guid or not revoked); calls = calls + 1 end
                setup(function(unit) return unit .. "-guid" end, function() end, function() end)
                _G[api] = nil
                local old = getmetatable(_G)
                setmetatable(_G, { __index = function(_, key)
                    if key == api then
                        if phase == "lookup" then revoked = true end
                        return fn
                    end
                end })
                issecretvalue = function(v)
                    if phase == "secret" and rawequal(v, fn) then revoked = true end
                    return rawequal(v, secret)
                end
                canaccessvalue = function(v)
                    if (phase == "access" and rawequal(v, fn)) or (phase == "global" and rawequal(v, _G)) then revoked = true end
                    return not rawequal(v, secret) and not (revoked and v == guid)
                end
                local ok, r = pcall(capture)
                setmetatable(_G, old)
                assert(ok, r)
                assert(calls == 2)
                for i, unit in ipairs(tokens) do
                    if unit == token then assert(r.units[i].queries[api].status == "restricted-input") end
                end
            end
        end
    end
end)

test("producer lookup revocation and throwing global lookup are bounded", function()
    for _, phase in ipairs({ "revoke", "throw" }) do
        local revoked, calls, lookups = false, 0, 0
        setup(nil, function() end, function() end)
        local old = getmetatable(_G)
        setmetatable(_G, { __index = function(_, key)
            if key == "UnitGUID" then
                lookups = lookups + 1
                if lookups == 1 and phase == "throw" then error(secret) end
                revoked = true
                return function(unit) assert(unit ~= "player"); calls = calls + 1; return unit end
            end
        end })
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and v == "player") end
        local ok, r = pcall(capture)
        setmetatable(_G, old)
        assert(ok, r)
        assert(calls == 2 and lookups == 3)
        assert(r.units[1].producer.status == (phase == "throw" and "field-error" or "restricted-input"))
    end
end)

test("serialization and earlier queries can revoke downstream inputs and outputs", function()
    local marker, revoked, calls = {}, false, 0
    setup(function() return "original", marker end, function() calls = calls + 1 end,
        function() calls = calls + 1 end)
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and v == "original")
    end
    capture(); assert(calls == 0)
    revoked, calls = false, 0
    setup(function() return "original" end, function() return marker, "later" end,
        function() calls = calls + 1 end)
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and (v == "original" or v == "later"))
    end
    local r = capture()
    assert(calls == 0 and r.units[1].queries.UnitClassFromGUID.values[2].status == "restricted")
end)

test("missing APIs and access guards fail closed without hiding peers", function()
    local names = 0
    setup(function() return "guid" end, secret, function() names = names + 1 end)
    capture(); assert(names == 3)
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        setup(function() error("producer invoked") end, function() error("class invoked") end, nil)
        _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("guid-identity fixture")
        assert(ApiContractProbeDB and ApiContractProbeDB.captures[1].status == "missing-access-api")
        setup(function() error("producer invoked") end, nil, nil)
        _G[guard] = function() error(secret) end
        local r = capture()
        assert(r.units[1].producer.status == "restricted-input")
    end
end)

test("tuple string label and ten snapshot bounds cap all nine calls", function()
    local count, values = 0, {}
    for i = 1, 18 do values[i] = string.rep("X", 300) end
    local query = function() count = count + 1; return unpack(values) end
    setup(function() count = count + 1; return "guid" end, query, query)
    local r = capture(string.rep("L", 200))
    assert(r.units[1].queries.UnitClassFromGUID.n == 18 and r.units[1].queries.UnitClassFromGUID.truncated)
    assert(#r.units[1].queries.UnitClassFromGUID.values == 16)
    assert(#r.units[1].queries.UnitClassFromGUID.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 12 do capture() end
    assert(count == 90 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 2)
end)

test("opaque outputs are collectible and other name modes exclude GUID calls", function()
    local weak, calls = setmetatable({}, { __mode = "v" }), 0
    setup(function() calls = calls + 1; return "guid" end, function()
        local value = newproxy(true)
        getmetatable(value).__index = function() error("object inspected") end
        getmetatable(value).__tostring = function() error("object stringified") end
        weak[#weak + 1] = value
        return value, {}, function() error("callback invoked") end
    end, function() return "name", "realm" end)
    capture()
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
    for _, mode in ipairs({ "all", "names", "full-names" }) do SlashCmdList.APICONTRACTPROBE(mode .. " fixture") end
    assert(calls == 3)
    for i = 2, 4 do assert(ApiContractProbeDB.captures[i].guidIdentity == nil) end
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
