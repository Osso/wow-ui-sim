local root = assert(arg[1])
local passed, failed = 0, 0
local tokens = { "player", "target", "focus", "pet", "party1", "nonexistent", "invalid-unit-token", "" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspected") end
getmetatable(secret).__tostring = function() error("secret stringified") end
local function setup(fn)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    UnitFullName = fn
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("full-names " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "full-names mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].fullNames)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("eight exact one-argument calls preserve two independent returns", function()
    local calls = 0
    setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 1 and (...) == tokens[calls])
        return "Name" .. calls, "Realm" .. calls
    end)
    local r = capture()
    assert(calls == 8 and #r.units == 8)
    for i, row in ipairs(r.units) do
        assert(row.unit.value == tokens[i] and row.result.n == 2)
        assert(row.result.values[1].value == "Name" .. i and row.result.values[2].value == "Realm" .. i)
    end
end)

test("zero nil holes errors and restricted results stay distinct", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil, "Realm" end
        if calls == 3 then error(secret) end
        if calls == 4 then return secret, nil end
        return false, 7
    end)
    local r = capture()
    assert(calls == 8 and r.units[1].result.n == 0)
    assert(r.units[2].result.n == 2 and r.units[2].result.values[1].kind == "nil")
    assert(r.units[2].result.values[2].value == "Realm")
    assert(r.units[3].result.status == "call-error")
    assert(r.units[4].result.values[1].status == "restricted" and r.units[4].result.values[2].kind == "nil")
    assert(r.units[5].result.values[1].value == false and r.units[5].result.values[2].value == 7)
end)

test("missing and inaccessible functions are never invoked", function()
    for _, value in ipairs({ false, 12, secret }) do
        setup(value)
        for _, row in ipairs(capture().units) do assert(row.result.status == "missing-api") end
    end
    setup(nil)
    for _, row in ipairs(capture().units) do assert(row.result.status == "missing-api") end
end)

test("every restricted input skips only itself", function()
    for _, blocked in ipairs(tokens) do
        local calls = 0
        setup(function() calls = calls + 1 end)
        canaccessvalue = function(v) return not rawequal(v, secret) and v ~= blocked end
        local r = capture()
        assert(calls == 7)
        for i, token in ipairs(tokens) do
            if token == blocked then assert(r.units[i].result.status == "restricted-input") end
        end
    end
end)

test("both function guard phases can revoke each token", function()
    for _, blocked in ipairs(tokens) do
        for _, phase in ipairs({ "secret", "access" }) do
            local revoked, calls = false, 0
            local fn = function() calls = calls + 1 end
            setup(fn)
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "access" and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and v == blocked)
            end
            local r = capture()
            assert(calls == 7)
            for i, token in ipairs(tokens) do
                if token == blocked then assert(r.units[i].result.status == "restricted-input") end
            end
        end
    end
end)

test("global lookup failures remain opaque and peers recover", function()
    setup(nil)
    local old = getmetatable(_G)
    local lookups = 0
    setmetatable(_G, { __index = function(_, key)
        if key == "UnitFullName" then
            lookups = lookups + 1
            if lookups == 1 then error(secret) end
            return function() return "peer", nil end
        end
    end })
    local ok, result = pcall(capture)
    setmetatable(_G, old)
    assert(ok, result)
    assert(lookups == 8 and result.units[1].result.status == "field-error")
    assert(result.units[2].result.values[1].value == "peer")
end)

test("output guards can revoke subsequent positional results", function()
    local a, b = {}, {}
    local revoked = false
    setup(function() return a, b end)
    canaccessvalue = function(v)
        if rawequal(v, a) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, b))
    end
    local r = capture()
    assert(r.units[1].result.values[1].kind == "table")
    assert(r.units[1].result.values[2].status == "restricted")
end)

test("tuple string label and snapshot caps are independent", function()
    local calls, values = 0, {}
    for i = 1, 18 do values[i] = string.rep("x", 300) end
    setup(function() calls = calls + 1; return unpack(values) end)
    local r = capture(string.rep("L", 200))
    assert(r.units[1].result.n == 18 and r.units[1].result.truncated)
    assert(#r.units[1].result.values == 16 and #r.units[1].result.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 12 do capture() end
    assert(calls == 80 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 2)
end)

test("returned objects are opaque and collectible", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function()
        local value = newproxy(true)
        getmetatable(value).__index = function() error("object inspected") end
        getmetatable(value).__tostring = function() error("object stringified") end
        weak[1] = value
        return value, {}
    end)
    local r = capture()
    collectgarbage("collect"); collectgarbage("collect")
    assert(weak[1] == nil and r.units[1].result.values[1].kind == "userdata")
end)

test("all and names never invoke full-name query", function()
    local calls = 0
    setup(function() calls = calls + 1; return "name", "realm" end)
    capture()
    SlashCmdList.APICONTRACTPROBE("all fixture")
    SlashCmdList.APICONTRACTPROBE("names fixture")
    assert(calls == 8)
    assert(ApiContractProbeDB.captures[2].fullNames == nil and ApiContractProbeDB.captures[3].fullNames == nil)
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
