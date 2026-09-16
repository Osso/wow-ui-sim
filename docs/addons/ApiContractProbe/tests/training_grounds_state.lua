local root = assert(arg[1], "addon directory required")
local passed = 0
local names = { "AreTrainingGroundsEnabled", "CanPlayerUseTrainingGroundsUI",
    "HasRandomTrainingGroundWinToday" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(namespace)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    C_PvP = namespace
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("training-grounds-state " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual training-grounds-state mode absent")
    return assert(ApiContractProbeDB.captures[1].trainingGroundsState)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end
local function namespaceWith(fn)
    local namespace = {}
    for _, name in ipairs(names) do namespace[name] = fn end
    return namespace
end

test("three independent query pairs preserve zero arguments and changing raw results", function()
    local namespace, calls = {}, {}
    for _, name in ipairs(names) do
        local key = name
        namespace[key] = function(...)
            assert(select("#", ...) == 0)
            calls[key] = (calls[key] or 0) + 1
            return key, nil, calls[key]
        end
    end
    setup(namespace)
    local result = capture()
    for _, name in ipairs(names) do
        assert(calls[name] == 2)
        for i = 1, 2 do
            assert(result[name][i].n == 3)
            assert(result[name][i].values[1].value == name)
            assert(result[name][i].values[2].kind == "nil")
            assert(result[name][i].values[3].value == i)
        end
    end
end)

test("raw returns keep zero and explicit nil positions distinct", function()
    local calls = 0
    setup(namespaceWith(function() return false end))
    C_PvP.CanPlayerUseTrainingGroundsUI = function()
        calls = calls + 1
        if calls == 1 then return 12345, 7 end
    end
    local result = capture().CanPlayerUseTrainingGroundsUI
    assert(result[1].n == 2 and result[1].values[1].value == 12345 and result[1].values[2].value == 7)
    assert(result[2].n == 0)
    setup(namespaceWith(function() return nil, nil end))
    result = capture().CanPlayerUseTrainingGroundsUI
    assert(result[1].n == 2 and result[2].values[2].kind == "nil")
end)

test("missing and throwing namespaces preserve every observation", function()
    for _, namespace in ipairs({ false, secret, setmetatable({}, { __index = function() error(secret) end }) }) do
        setup(namespace)
        local result = capture()
        for _, name in ipairs(names) do
            assert(result[name][1].status == "field-error" and result[name][2].status == "field-error")
        end
    end
    setup(nil)
    assert(capture().HasRandomTrainingGroundWinToday[1].status == "field-error")
end)

test("each missing secret or throwing function leaves peers independent", function()
    for _, blocked in ipairs(names) do
        for _, kind in ipairs({ "missing", "secret", "throw" }) do
            local calls = 0
            local namespace = namespaceWith(function() calls = calls + 1; return true end)
            namespace[blocked] = kind == "secret" and secret or kind == "throw" and function() error(secret) end or nil
            setup(namespace)
            local result = capture()
            assert(calls == 4)
            for i = 1, 2 do
                assert(result[blocked][i].status == (kind == "throw" and "call-error" or "missing-api"))
            end
        end
    end
end)

test("namespace guard prevents lookup and function guards prevent invocation", function()
    local namespace = setmetatable({}, { __index = function() error("must not lookup") end })
    setup(namespace)
    canaccessvalue = function(v) return not rawequal(v, namespace) end
    assert(capture().AreTrainingGroundsEnabled[1].status == "field-error")
    for _, guard in ipairs({ "secret", "access" }) do
        local fn = function() error("must not call") end
        setup(namespaceWith(fn))
        if guard == "secret" then issecretvalue = function(v) return rawequal(v, fn) end
        else canaccessvalue = function(v) return not rawequal(v, fn) end end
        local result = capture()
        for _, name in ipairs(names) do assert(result[name][2].status == "missing-api") end
    end
end)

test("fresh lookup sees replacement namespace and replacement function", function()
    local replacementCalls = 0
    local replacement = namespaceWith(function() replacementCalls = replacementCalls + 1; return "replacement" end)
    local namespace = namespaceWith(function() return "old" end)
    namespace.AreTrainingGroundsEnabled = function() C_PvP = replacement; return "first" end
    setup(namespace)
    local result = capture()
    assert(replacementCalls == 5)
    assert(result.AreTrainingGroundsEnabled[1].values[1].value == "first")
    assert(result.AreTrainingGroundsEnabled[2].values[1].value == "replacement")
    setup(namespaceWith(function() return "peer" end))
    C_PvP.CanPlayerUseTrainingGroundsUI = function()
        C_PvP.CanPlayerUseTrainingGroundsUI = function() return "new" end
        return "old"
    end
    result = capture()
    assert(result.CanPlayerUseTrainingGroundsUI[1].values[1].value == "old")
    assert(result.CanPlayerUseTrainingGroundsUI[2].values[1].value == "new")
end)

test("restricted and opaque results remain uninspected", function()
    setup(namespaceWith(function() return secret, {}, newproxy(true) end))
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    for _, name in ipairs(names) do
        assert(result[name][1].values[1].status == "restricted")
        assert(result[name][1].values[2].kind == "table")
        assert(result[name][1].values[3].kind == "userdata")
    end
end)

test("tuple string label and snapshot caps bound six calls", function()
    local calls = 0
    setup(namespaceWith(function()
        calls = calls + 1
        return string.rep("x", 300), 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
    end))
    local result = capture(string.rep("l", 200))
    assert(calls == 6 and #ApiContractProbeDB.captures[1].label == 128)
    for _, name in ipairs(names) do
        assert(result[name][1].n == 17 and result[name][1].truncated)
        assert(#result[name][1].values == 16 and #result[name][1].values[1].value == 256)
    end
    for i = 2, 11 do SlashCmdList.APICONTRACTPROBE("training-grounds-state limit") end
    assert(calls == 60 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("all excludes training grounds state and mutations are never queried", function()
    local excludedCalls = 0
    local namespace = namespaceWith(function() excludedCalls = excludedCalls + 1; return false end)
    setmetatable(namespace, { __index = function(_, name) error("excluded lookup: " .. name) end })
    setup(namespace)
    SlashCmdList.APICONTRACTPROBE("all excluded")
    assert(ApiContractProbeDB.captures[1].trainingGroundsState == nil and excludedCalls == 0)
    local allowed = namespaceWith(function() return false end)
    for _, name in ipairs({ "JoinRandomTrainingGround", "JoinTrainingGround", "JoinBattlefield",
        "GetTrainingGrounds", "GetRandomTrainingGroundRewards", "GetBattlegroundInfo",
        "RequestRatedInfo", "RequestPVPRewards", "HasMatchStarted" }) do
        allowed[name] = function() excludedCalls = excludedCalls + 1; return {} end
    end
    setup(setmetatable(allowed, {
        __index = function() excludedCalls = excludedCalls + 1; return nil end,
    }))
    assert(capture().HasRandomTrainingGroundWinToday[2].values[1].value == false)
    assert(excludedCalls == 0)
end)

test("missing access APIs fail closed without query calls", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        setup(namespaceWith(function() error("unguarded call") end))
        _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("training-grounds-state inaccessible")
        assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
    end
end)

print(string.format("%d training-grounds-state fixtures passed", passed))
