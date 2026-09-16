local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(namespace)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    C_Ping = namespace
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("ping-enabled " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual ping-enabled mode absent")
    return assert(ApiContractProbeDB.captures[1].pingEnabled).IsPingSystemEnabled
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function namespaceWith(fn) return { IsPingSystemEnabled = fn } end

test("two independent zero-argument calls retain changing raw tuples", function()
    local calls = 0
    setup(namespaceWith(function(...)
        assert(select("#", ...) == 0)
        calls = calls + 1
        return calls == 1, nil, calls
    end))
    local result = capture()
    assert(calls == 2)
    for i = 1, 2 do
        assert(result[i].n == 3 and result[i].values[2].kind == "nil")
        assert(result[i].values[1].value == (i == 1) and result[i].values[3].value == i)
    end
end)

test("zero returns and explicit nil positions stay distinct", function()
    local calls = 0
    setup(namespaceWith(function()
        calls = calls + 1
        if calls == 2 then return nil, false, nil end
    end))
    local result = capture()
    assert(result[1].n == 0 and result[2].n == 3)
    assert(result[2].values[1].kind == "nil" and result[2].values[3].kind == "nil")
end)

test("missing restricted and throwing namespaces preserve both outcomes", function()
    for _, namespace in ipairs({ false, secret, setmetatable({}, { __index = function() error(secret) end }) }) do
        setup(namespace)
        local result = capture()
        assert(result[1].status == "field-error" and result[2].status == "field-error")
    end
    setup(nil)
    assert(capture()[2].status == "field-error")
end)

test("missing secret and denied functions never invoke", function()
    setup({})
    assert(capture()[1].status == "missing-api")
    setup(namespaceWith(secret))
    assert(capture()[2].status == "missing-api")
    for _, guard in ipairs({ "secret", "access" }) do
        local calls = 0
        local fn = function() calls = calls + 1 end
        setup(namespaceWith(fn))
        if guard == "secret" then issecretvalue = function(v) return rawequal(v, fn) end
        else canaccessvalue = function(v) return not rawequal(v, fn) end end
        assert(capture()[2].status == "missing-api" and calls == 0)
    end
end)

test("namespace guards prevent lookup", function()
    for _, guard in ipairs({ "secret", "access" }) do
        local lookups = 0
        local namespace = setmetatable({}, { __index = function() lookups = lookups + 1 end })
        setup(namespace)
        if guard == "secret" then issecretvalue = function(v) return rawequal(v, namespace) end
        else canaccessvalue = function(v) return not rawequal(v, namespace) end end
        local result = capture()
        assert(result[1].status == "field-error" and result[2].status == "field-error" and lookups == 0)
    end
end)

test("throwing lookup or call does not suppress next read", function()
    local lookups = 0
    setup(setmetatable({}, { __index = function()
        lookups = lookups + 1
        if lookups == 1 then error(secret) end
        return function() return false end
    end }))
    local result = capture()
    assert(result[1].status == "field-error" and result[2].values[1].value == false)
    local calls = 0
    setup(namespaceWith(function()
        calls = calls + 1
        if calls == 1 then error(secret) end
        return true
    end))
    result = capture()
    assert(calls == 2 and result[1].status == "call-error" and result[2].values[1].value == true)
end)

test("fresh reads observe namespace and function replacement", function()
    setup(namespaceWith(function()
        C_Ping = namespaceWith(function() return "namespace replacement" end)
        return "first"
    end))
    local result = capture()
    assert(result[1].values[1].value == "first" and result[2].values[1].value == "namespace replacement")
    setup(namespaceWith(function()
        C_Ping.IsPingSystemEnabled = function() return "function replacement" end
        return "first"
    end))
    assert(capture()[2].values[1].value == "function replacement")
end)

test("restricted outputs stay opaque before type inspection", function()
    setup(namespaceWith(function() return secret, {}, newproxy(true) end))
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    assert(result[1].values[1].status == "restricted")
    assert(result[1].values[2].kind == "table" and result[2].values[3].kind == "userdata")
end)

test("tuple string label and shared snapshot limits bound calls", function()
    local calls = 0
    setup(namespaceWith(function()
        calls = calls + 1
        return string.rep("x", 300), 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
    end))
    local result = capture(string.rep("l", 200))
    assert(calls == 2 and #ApiContractProbeDB.captures[1].label == 128)
    assert(result[1].n == 17 and result[1].truncated and #result[1].values == 16)
    assert(#result[2].values[1].value == 256)
    for i = 2, 11 do SlashCmdList.APICONTRACTPROBE("ping-enabled limit") end
    assert(calls == 20 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("all excludes mode and no secure ping or CVar calls occur", function()
    local calls, excluded = 0, 0
    local namespace = namespaceWith(function() calls = calls + 1; return false end)
    for _, name in ipairs({ "SendMacroPing", "TogglePingListener", "SendPing", "GetTextureKitForType" }) do
        namespace[name] = function() excluded = excluded + 1 end
    end
    setmetatable(namespace, { __index = function() excluded = excluded + 1 end })
    setup(namespace)
    C_PingSecure = setmetatable({}, { __index = function() excluded = excluded + 1 end })
    SlashCmdList.APICONTRACTPROBE("all excluded")
    assert(ApiContractProbeDB.captures[1].pingEnabled == nil and calls == 0 and excluded == 0)
    ApiContractProbeDB = nil
    local oldSet, oldCVar = SetCVar, C_CVar
    SetCVar = function() excluded = excluded + 1 end
    C_CVar = setmetatable({}, { __index = function() excluded = excluded + 1 end })
    local ok, result = pcall(capture)
    SetCVar, C_CVar, C_PingSecure = oldSet, oldCVar, nil
    assert(ok, result)
    assert(calls == 2 and excluded == 0)
end)

test("missing access APIs fail closed", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        local calls = 0
        setup(namespaceWith(function() calls = calls + 1 end))
        _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("ping-enabled inaccessible")
        assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual ping-enabled mode absent")
        assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    end
end)

test("opaque returned objects are not retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    local calls = 0
    setup(namespaceWith(function()
        calls = calls + 1
        local value = newproxy(true)
        weak[calls] = value
        return value
    end))
    capture()
    collectgarbage("collect")
    assert(calls == 2 and next(weak) == nil, "raw object retained")
end)

print(string.format("%d passed, %d failed ping-enabled fixtures", passed, failed))
if failed > 0 then os.exit(1) end
