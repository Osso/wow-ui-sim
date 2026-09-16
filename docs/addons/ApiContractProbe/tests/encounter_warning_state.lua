local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local names = { "IsFeatureAvailable", "IsFeatureEnabled" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(namespace)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    C_EncounterWarnings = namespace
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("encounter-warning-state " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual encounter-warning-state mode absent")
    return assert(ApiContractProbeDB.captures[1].encounterWarningState)
end
local function namespaceWith(fn) return { IsFeatureAvailable = fn, IsFeatureEnabled = fn } end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("both APIs receive two independent zero-argument calls", function()
    local counts, namespace = {}, {}
    for _, name in ipairs(names) do
        local key = name
        counts[key] = 0
        namespace[key] = function(...)
            assert(select("#", ...) == 0)
            counts[key] = counts[key] + 1
            return counts[key] == 1, nil, key
        end
    end
    setup(namespace)
    local result = capture()
    for _, name in ipairs(names) do
        assert(counts[name] == 2)
        for i = 1, 2 do
            local row = result[name][i]
            assert(row.n == 3 and row.values[2].kind == "nil")
            assert(row.values[1].value == (i == 1) and row.values[3].value == name)
        end
    end
end)

test("zero returns and explicit nil positions remain distinct", function()
    local calls = 0
    setup({ IsFeatureAvailable = function() end, IsFeatureEnabled = function()
        calls = calls + 1
        return nil, false, nil
    end })
    local result = capture()
    assert(calls == 2)
    for i = 1, 2 do
        assert(result.IsFeatureAvailable[i].n == 0)
        assert(result.IsFeatureEnabled[i].n == 3)
        assert(result.IsFeatureEnabled[i].values[3].kind == "nil")
    end
end)

test("missing restricted and throwing namespaces preserve four outcomes", function()
    for _, namespace in ipairs({ false, secret, setmetatable({}, { __index = function() error(secret) end }) }) do
        setup(namespace)
        local result = capture()
        for _, name in ipairs(names) do
            for i = 1, 2 do assert(result[name][i].status == "field-error") end
        end
    end
    setup(nil)
    assert(capture().IsFeatureEnabled[2].status == "field-error")
end)

test("missing secret and denied functions do not suppress peer API", function()
    for _, blocked in ipairs(names) do
        for _, guard in ipairs({ "missing", "secret-value", "secret", "access" }) do
            local calls, denied = 0, 0
            local fn = function() denied = denied + 1 end
            local namespace = namespaceWith(function() calls = calls + 1; return true end)
            namespace[blocked] = guard == "missing" and nil or guard == "secret-value" and secret or fn
            if guard == "missing" then namespace[blocked] = nil end
            setup(namespace)
            if guard == "secret" then issecretvalue = function(v) return rawequal(v, fn) end
            elseif guard == "access" then canaccessvalue = function(v) return not rawequal(v, fn) end end
            local result = capture()
            assert(calls == 2 and denied == 0)
            assert(result[blocked][1].status == "missing-api" and result[blocked][2].status == "missing-api")
        end
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
        assert(lookups == 0 and result.IsFeatureAvailable[1].status == "field-error")
        assert(result.IsFeatureEnabled[2].status == "field-error")
    end
end)

test("lookup and call errors leave independent reads available", function()
    for _, broken in ipairs(names) do
        for _, phase in ipairs({ "lookup", "call" }) do
            local counts = { IsFeatureAvailable = 0, IsFeatureEnabled = 0 }
            setup(setmetatable({}, { __index = function(_, name)
                counts[name] = counts[name] + 1
                if name == broken and counts[name] == 1 and phase == "lookup" then error(secret) end
                local fails = name == broken and counts[name] == 1 and phase == "call"
                return function() if fails then error(secret) end; return name end
            end }))
            local result = capture()
            assert(counts.IsFeatureAvailable == 2 and counts.IsFeatureEnabled == 2)
            assert(result[broken][1].status == (phase == "lookup" and "field-error" or "call-error"))
            assert(result[broken][2].values[1].value == broken)
        end
    end
end)

test("fresh reads observe namespace and function replacement", function()
    setup(namespaceWith(function()
        C_EncounterWarnings = namespaceWith(function() return "replacement" end)
        return "first"
    end))
    local result = capture()
    local first, replacement = 0, 0
    for _, name in ipairs(names) do
        for i = 1, 2 do
            if result[name][i].values[1].value == "first" then first = first + 1
            else assert(result[name][i].values[1].value == "replacement"); replacement = replacement + 1 end
        end
    end
    assert(first == 1 and replacement == 3)
    local namespace = {}
    for _, name in ipairs(names) do
        local key = name
        namespace[key] = function()
            namespace[key] = function() return "new " .. key end
            return "old " .. key
        end
    end
    setup(namespace)
    result = capture()
    for _, name in ipairs(names) do assert(result[name][2].values[1].value == "new " .. name) end
end)

test("restricted outputs remain opaque before type inspection", function()
    setup(namespaceWith(function() return secret, {}, newproxy(true) end))
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    for _, name in ipairs(names) do
        assert(result[name][1].values[1].status == "restricted")
        assert(result[name][2].values[2].kind == "table" and result[name][2].values[3].kind == "userdata")
    end
end)

test("tuple string label and shared snapshot bounds cap four calls", function()
    local calls = 0
    setup(namespaceWith(function()
        calls = calls + 1
        return string.rep("x", 300), 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
    end))
    local result = capture(string.rep("l", 200))
    assert(calls == 4 and #ApiContractProbeDB.captures[1].label == 128)
    for _, name in ipairs(names) do
        assert(result[name][1].n == 17 and result[name][1].truncated and #result[name][1].values == 16)
        assert(#result[name][2].values[1].value == 256)
    end
    for i = 2, 11 do SlashCmdList.APICONTRACTPROBE("encounter-warning-state limit") end
    assert(calls == 40 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("all excludes mode and warning side effects are never looked up", function()
    local calls, excluded = 0, 0
    local namespace = namespaceWith(function() calls = calls + 1; return false end)
    setmetatable(namespace, { __index = function() excluded = excluded + 1; error("excluded API") end })
    setup(namespace)
    SlashCmdList.APICONTRACTPROBE("all excluded")
    assert(ApiContractProbeDB.captures[1].encounterWarningState == nil and calls == 0 and excluded == 0)
    ApiContractProbeDB = nil
    capture()
    assert(calls == 4 and excluded == 0)
end)

test("missing or throwing access APIs fail closed", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        for _, missing in ipairs({ true, false }) do
            local calls = 0
            setup(namespaceWith(function() calls = calls + 1 end))
            if missing then _G[guard] = nil else _G[guard] = function() error(secret) end end
            SlashCmdList.APICONTRACTPROBE("encounter-warning-state inaccessible")
            assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual encounter-warning-state mode absent")
            assert(calls == 0)
            if missing then assert(ApiContractProbeDB.captures[1].status == "missing-access-api") end
        end
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
    assert(calls == 4 and next(weak) == nil, "raw object retained")
end)

print(string.format("%d passed, %d failed encounter-warning-state fixtures", passed, failed))
if failed > 0 then os.exit(1) end
