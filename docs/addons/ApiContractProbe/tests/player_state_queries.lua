local root = assert(arg[1], "addon directory required")
local names = { "GetCollapsingStarCost", "ShowingCloak", "ShowingHelm" }
local passed = 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspection") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function setup(fn)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    for _, name in ipairs(names) do rawset(_G, name, fn) end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("player-state-queries " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual player-state-queries mode absent")
    return assert(ApiContractProbeDB.captures[1].playerStateQueries)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("six zero-argument calls preserve independent changing values", function()
    setup(function() end)
    local calls = {}
    for _, name in ipairs(names) do
        local key = name
        _G[key] = function(...)
            assert(select("#", ...) == 0)
            calls[key] = (calls[key] or 0) + 1
            return key, nil, calls[key]
        end
    end
    local result = capture()
    for _, name in ipairs(names) do
        assert(calls[name] == 2)
        for i = 1, 2 do
            local r = result[name][i]
            assert(r.n == 3 and r.values[1].value == name)
            assert(r.values[2].kind == "nil" and r.values[3].value == i)
        end
    end
end)

test("missing secret and throwing functions do not suppress peers", function()
    for _, blocked in ipairs(names) do
        for _, mode in ipairs({ "missing", "secret", "throw" }) do
            local calls = 0
            setup(function() calls = calls + 1; return false end)
            _G[blocked] = mode == "secret" and secret or mode == "throw" and function() error(secret) end or nil
            local result = capture()
            assert(calls == 4)
            for i = 1, 2 do assert(result[blocked][i].status == (mode == "throw" and "call-error" or "missing-api")) end
        end
    end
end)

test("global receiver guard blocks lookup", function()
    setup(function() error("must not invoke") end)
    canaccessvalue = function(v) return not rawequal(v, _G) end
    local result = capture()
    for _, name in ipairs(names) do assert(result[name][1].status == "field-error" and result[name][2].status == "field-error") end
end)

test("throwing global lookup is contained independently", function()
    setup(function() return true end)
    rawset(_G, "ShowingCloak", nil)
    local old = getmetatable(_G)
    setmetatable(_G, { __index = function(_, key) if key == "ShowingCloak" then error(secret) end end })
    local ok, result = pcall(capture)
    setmetatable(_G, old)
    assert(ok, result)
    assert(result.ShowingCloak[1].status == "field-error")
    assert(result.ShowingHelm[2].values[1].value == true)
end)

test("both function guards block invocation", function()
    for _, phase in ipairs({ "secret", "access" }) do
        local fn = function() error("unguarded invocation") end
        setup(fn)
        if phase == "secret" then issecretvalue = function(v) return rawequal(v, fn) end
        else canaccessvalue = function(v) return not rawequal(v, fn) end end
        local result = capture()
        for _, name in ipairs(names) do assert(result[name][2].status == "missing-api") end
    end
end)

test("fresh global lookup sees function replacement", function()
    setup(function() return "peer" end)
    GetCollapsingStarCost = function()
        GetCollapsingStarCost = function() return "replacement" end
        return "initial"
    end
    local result = capture()
    assert(result.GetCollapsingStarCost[1].values[1].value == "initial")
    assert(result.GetCollapsingStarCost[2].values[1].value == "replacement")
end)

test("zero returns nil holes restricted and opaque values remain distinct", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        if calls % 2 == 1 then return end
        return nil, secret, {}, newproxy(true), nil
    end)
    local result = capture()
    for _, name in ipairs(names) do
        assert(result[name][1].n == 0)
        local r = result[name][2]
        assert(r.n == 5 and r.values[1].kind == "nil" and r.values[5].kind == "nil")
        assert(r.values[2].status == "restricted" and r.values[3].kind == "table" and r.values[4].kind == "userdata")
    end
end)

test("tuple string label and snapshot bounds cap sixty calls", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        return string.rep("x", 300), 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
    end)
    local result = capture(string.rep("l", 200))
    assert(calls == 6 and #ApiContractProbeDB.captures[1].label == 128)
    for _, name in ipairs(names) do
        local r = result[name][1]
        assert(r.n == 17 and r.truncated and #r.values == 16 and #r.values[1].value == 256)
    end
    for i = 2, 11 do SlashCmdList.APICONTRACTPROBE("player-state-queries cap") end
    assert(calls == 60 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("all excludes mode and state mutations never execute", function()
    local calls, mutations = 0, 0
    setup(function() calls = calls + 1; return false end)
    ShowCloak = function() mutations = mutations + 1 end
    ShowHelm = ShowCloak
    SlashCmdList.APICONTRACTPROBE("all excluded")
    assert(calls == 0 and ApiContractProbeDB.captures[1].playerStateQueries == nil)
    ApiContractProbeDB = nil
    capture()
    assert(calls == 6 and mutations == 0)
    ShowCloak, ShowHelm = nil, nil
end)

test("missing access APIs fail closed", function()
    for _, name in ipairs({ "issecretvalue", "canaccessvalue" }) do
        setup(function() error("unguarded call") end)
        _G[name] = nil
        SlashCmdList.APICONTRACTPROBE("player-state-queries restricted")
        assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
    end
end)

test("opaque returned objects are collectible", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function()
        local value = newproxy(true)
        weak[#weak + 1] = value
        return value
    end)
    capture()
    collectgarbage("collect")
    assert(next(weak) == nil)
end)

print(string.format("%d/11 player-state-queries fixtures passed", passed))
assert(passed == 11, "player-state-queries fixture failures")
