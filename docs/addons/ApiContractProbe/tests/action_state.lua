local root = assert(arg[1], "addon directory required")
local names = { "GetActionAutocast", "GetActionText", "GetActionUseCount", "HasRangeRequirements",
    "IsAttackAction", "IsAutoRepeatAction", "IsConsumableAction", "IsEquippedAction",
    "IsItemAction", "IsStackableAction", "IsUsableAction", "IsActionInRange" }
local bars = { "GetExtraBarIndex", "GetMultiCastBarIndex" }
local passed, failed = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function setup(producer, api)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture" end
    time = function() return 42 end
    GetActionInfo, C_ActionBar = producer, api
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function namespace(fn)
    local api = {}
    for _, name in ipairs(names) do api[name] = fn end
    for _, name in ipairs(bars) do api[name] = fn end
    return api
end
local function capture(input)
    SlashCmdList.APICONTRACTPROBE(input or "action-state 17 sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "action-state mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].actionState, "action-state output absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("macro and item controls do not filter original selected slots", function()
    for _, kind in ipairs({ "macro", "item", "spell" }) do
        local count, api = 0, {}
        for _, name in ipairs(names) do api[name] = function(...)
            assert(select("#", ...) == 1 and (...) == -17)
            count = count + 1; return false, nil, "raw"
        end end
        for _, name in ipairs(bars) do api[name] = function(...)
            assert(select("#", ...) == 0); count = count + 1; return 7
        end end
        setup(function(...)
            assert(select("#", ...) == 1 and (...) == -17)
            count = count + 1; return kind, 999, nil
        end, api)
        local r = capture("action-state -17 macro label")
        assert(count == 15 and r.identity.n == 3 and r.identity.values[1].value == kind)
        assert(r.slot.value == -17)
        for _, name in ipairs(names) do
            assert(r.queries[name].n == 3 and r.queries[name].values[2].kind == "nil")
        end
        for _, name in ipairs(bars) do assert(r.bars[name].values[1].value == 7) end
        assert(ApiContractProbeDB.captures[1].label == "macro label")
    end
end)

test("missing throwing and restricted controls leave all fourteen queries independent", function()
    for _, scenario in ipairs({ "missing", "throw", "secret", "zero", "nil" }) do
        local calls = 0
        local producer
        if scenario == "throw" then producer = function() error(secret) end
        elseif scenario == "secret" then producer = secret
        elseif scenario == "zero" then producer = function() end
        elseif scenario == "nil" then producer = function() return nil end end
        setup(producer, namespace(function() calls = calls + 1 end))
        local r = capture()
        assert(calls == 14)
        if scenario == "zero" then assert(r.identity.n == 0)
        elseif scenario == "nil" then assert(r.identity.n == 1 and r.identity.values[1].kind == "nil")
        else assert(r.identity.status ~= "observed") end
    end
end)

test("every query can fail without suppressing peers or losing arity", function()
    local all = {}
    for _, name in ipairs(names) do all[#all + 1] = name end
    for _, name in ipairs(bars) do all[#all + 1] = name end
    for _, bad in ipairs(all) do
        for _, failure in ipairs({ "missing", "lookup", "call" }) do
            local calls = 0
            setup(function() return "macro" end, setmetatable({}, { __index = function(_, name)
                if name == bad then
                    if failure == "missing" then return nil end
                    if failure == "lookup" then error(secret) end
                    return function() error(secret) end
                end
                return function() calls = calls + 1; return nil, false, nil end
            end }))
            local r = capture()
            assert(calls == 13)
            assert((r.queries[bad] or r.bars[bad]).status ~= "observed")
            for _, name in ipairs(all) do if name ~= bad then
                local row = r.queries[name] or r.bars[name]
                assert(row.n == 3 and row.values[1].kind == "nil" and row.values[3].kind == "nil")
            end end
        end
    end
end)

test("namespace lookup and function guards revoke slots before every forwarding position", function()
    for _, stage in ipairs({ "namespace", "lookup", "secret", "access" }) do
        for position = 1, #names do
            local revoked, calls, barCalls, lookups = false, 0, 0, 0
            local functions, api = {}, nil
            for i = 1, #names do functions[i] = function() calls = calls + 1 end end
            api = setmetatable({}, { __index = function(_, name)
                for i, expected in ipairs(names) do if name == expected then
                    lookups = lookups + 1
                    if stage == "lookup" and i == position then revoked = true end
                    return functions[i]
                end end
                return function() barCalls = barCalls + 1 end
            end })
            setup(function() return "item" end, api)
            issecretvalue = function(v)
                if stage == "secret" and rawequal(v, functions[position]) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if stage == "namespace" and rawequal(v, api) and lookups == position - 1 then revoked = true end
                if stage == "access" and rawequal(v, functions[position]) then revoked = true end
                return not (revoked and rawequal(v, 17))
            end
            local r = capture()
            assert(calls == position - 1 and barCalls == 2, stage .. position)
            assert(r.queries[names[position]].status == "restricted-input")
        end
    end
end)

test("control function guard and control output can revoke slot while bars continue", function()
    for _, stage in ipairs({ "secret", "access", "output" }) do
        local revoked, queried, barCalls = false, 0, 0
        local marker = {}
        local producer = function() return marker end
        local api = namespace(function() queried = queried + 1 end)
        for _, name in ipairs(bars) do api[name] = function() barCalls = barCalls + 1 end end
        setup(producer, api)
        issecretvalue = function(v)
            if stage == "secret" and rawequal(v, producer) then revoked = true end
            return false
        end
        canaccessvalue = function(v)
            if stage == "access" and rawequal(v, producer) then revoked = true end
            if stage == "output" and rawequal(v, marker) then revoked = true end
            return not (revoked and rawequal(v, 17))
        end
        local r = capture()
        assert(queried == 0 and barCalls == 2)
        assert(r.queries[names[1]].status == "restricted-input")
    end
end)

test("restricted namespaces functions and results are never inspected", function()
    for _, scenario in ipairs({ "namespace", "functions", "results" }) do
        local api = scenario == "namespace" and secret or namespace(scenario == "functions" and secret
            or function() return secret, nil end)
        setup(function() return secret end, api)
        local original = type
        type = function(v) assert(not rawequal(v, secret), "secret inspected"); return original(v) end
        local ok, r = pcall(capture)
        type = original
        assert(ok, r)
        assert(r.identity.values[1].status == "restricted")
        if scenario == "results" then assert(r.queries[names[1]].values[1].status == "restricted") end
    end
end)

test("missing or throwing access guards fail closed", function()
    setup(function() error("must not call") end, namespace(function() error("must not call") end))
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("action-state 17 missing")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
    setup(function() error("must not call") end, namespace(function() error("must not call") end))
    canaccessvalue = function() error(secret) end
    local r = capture()
    assert(r.identity.status ~= "observed" and r.bars[bars[1]].status ~= "observed")
end)

test("tuples strings labels and ten snapshots enforce fifteen-call bound", function()
    local values, calls = {}, 0
    for i = 1, 20 do values[i] = string.rep("X", 400) end
    local fn = function() calls = calls + 1; return unpack(values) end
    setup(fn, namespace(fn))
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("action-state 17 " .. string.rep("L", 200)) end
    assert(calls == 150 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local record = ApiContractProbeDB.captures[1]
    assert(#record.label == 128)
    local function bounded(row)
        assert(row.n == 20 and row.truncated and #row.values == 16)
        assert(#row.values[1].value == 256 and row.values[1].truncated)
    end
    bounded(record.actionState.identity)
    for _, row in pairs(record.actionState.queries) do bounded(row) end
    for _, row in pairs(record.actionState.bars) do bounded(row) end
end)

test("invalid syntax does not call APIs and all excludes action state", function()
    local calls = 0
    setup(function() calls = calls + 1 end, namespace(function() calls = calls + 1 end))
    for _, input in ipairs({ "action-state", "action-state x", "action-state 1.5", "action-state 9007199254740992" }) do
        SlashCmdList.APICONTRACTPROBE(input)
    end
    assert(calls == 0 and ApiContractProbeDB == nil)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and ApiContractProbeDB.captures[1].actionState == nil)
end)

test("opaque results are collectible and loss-control or action mutations never run", function()
    local weak = setmetatable({}, { __mode = "v" })
    local function result()
        local value = newproxy(true)
        getmetatable(value).__index = function() error("opaque traversal") end
        weak[#weak + 1] = value
        return value
    end
    local api = namespace(result)
    setmetatable(api, { __index = function() error("excluded action API") end })
    setup(result, api)
    capture()
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
