local root = assert(arg[1], "addon directory required")
local passed, failed, calls, excluded = 0, 0, 0, 0
local names = { "GetEventState", "GetEventTimeElapsed", "GetEventTimeRemaining", "GetEventTimer" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspection") end
getmetatable(secret).__tostring = function() error("secret formatting") end
local function setup(producer, query, highlight)
    ApiContractProbeDB, SlashCmdList, calls, excluded = nil, {}, 0, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_EncounterTimeline = {
        GetEventList = function(...) calls = calls + 1; assert(select("#", ...) == 0); return producer() end,
        GetEventHighlightTime = function(...) calls = calls + 1; assert(select("#", ...) == 0); return highlight() end,
    }
    for _, name in ipairs(names) do
        local key = name
        C_EncounterTimeline[key] = function(...)
            calls = calls + 1; assert(select("#", ...) == 1)
            return query(key, ...)
        end
    end
    local function forbidden() excluded = excluded + 1; error("excluded operation") end
    for _, name in ipairs({ "GetEventCountBySource", "GetEventInfo", "GetEventColor", "AddScriptEvent",
        "CancelScriptEvent", "FinishScriptEvent", "PauseScriptEvent", "ResumeScriptEvent", "SetEventIconTextures" }) do
        C_EncounterTimeline[name] = forbidden
    end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("timeline-lifecycle-read " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "timeline-lifecycle-read mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].timelineLifecycleRead)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("original duplicate fractional IDs and exact independent call shapes", function()
    local seen = {}
    setup(function() return { 4.5, 4.5, -2.25 } end, function(name, id)
        seen[#seen + 1] = { name, id }; return name, id
    end, function() return 3.25 end)
    local r = capture()
    assert(calls == 15 and #seen == 12 and excluded == 0)
    for i = 1, 3 do
        for j, name in ipairs(names) do
            assert(seen[(i - 1) * 4 + j][1] == name)
            assert(r.entries[i].queries[name].values[2].value == (i == 3 and -2.25 or 4.5))
        end
    end
    assert(r.highlight[1].values[1].value == 3.25 and r.highlight[2].n == 1)
end)
test("raw nil holes zero returns and errors preserve peer observations", function()
    local n = 0
    setup(function() return { 2 }, nil, 7 end, function(name)
        if name == names[1] then return nil, false, nil end
        if name == names[2] then error(secret) end
        if name == names[3] then return end
        return nil, 9
    end, function() n = n + 1; if n == 1 then error(secret) end; return nil, 4, nil end)
    local r = capture()
    assert(r.producer.n == 3 and r.producer.values[2].kind == "nil")
    assert(r.entries[1].queries[names[1]].n == 3 and r.entries[1].queries[names[1]].values[3].kind == "nil")
    assert(r.entries[1].queries[names[2]].status == "call-error")
    assert(r.entries[1].queries[names[3]].n == 0 and r.entries[1].queries[names[4]].n == 2)
    assert(r.highlight[1].status == "call-error" and r.highlight[2].n == 3 and calls == 7)
end)
test("first producer table only and highlight independent of unavailable producer", function()
    for _, value in ipairs({ secret, false, 9, "list", newproxy(true) }) do
        setup(function() return value, { 1 } end, function() error("unexpected") end, function() return 6 end)
        local r = capture(); assert(next(r.entries) == nil and calls == 3 and r.highlight[2].values[1].value == 6)
    end
    setup(function() error(secret) end, function() error("unexpected") end, function() return 8 end)
    local r = capture(); assert(r.producer.status == "call-error" and calls == 3)
    setup(function() return nil, { 1 } end, function() error("unexpected") end, function() return end)
    assert(next(capture().entries) == nil and calls == 3)
end)
test("invalid inaccessible and missing IDs do not suppress later entries", function()
    setup(function() return { [2] = secret, [3] = math.huge, [4] = -math.huge,
        [5] = 0/0, [6] = "2", [7] = {}, [8] = -3.5 } end,
        function(_, id) assert(id == -3.5); return id end, function() return 2 end)
    local r = capture()
    assert(calls == 7 and r.entries[1].queries[names[1]].status == "unavailable-input")
    assert(r.entries[2].queries[names[4]].status == "restricted-input")
    assert(r.entries[8].queries[names[1]].values[1].value == -3.5)
end)
test("missing APIs lookup failures and missing guards fail closed", function()
    setup(function() return { 1 } end, function() return 1 end, function() return 2 end)
    C_EncounterTimeline.GetEventState = secret
    local r = capture(); assert(r.entries[1].queries[names[1]].status == "missing-api" and calls == 6)
    setup(function() return {} end, function() end, function() end)
    C_EncounterTimeline = setmetatable({}, { __index = function() error(secret) end })
    r = capture(); assert(r.producer.status == "field-error" and r.highlight[2].status == "field-error" and calls == 0)
    setup(function() return {} end, function() end, function() end)
    C_EncounterTimeline = {}; r = capture(); assert(r.producer.status == "missing-api" and r.highlight[1].status == "missing-api")
    setup(function() return {} end, function() end, function() end)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("timeline-lifecycle-read missing")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
end)
test("original ID rechecked after namespace lookup and both function guards at every query", function()
    for index = 1, 8 do
        for _, name in ipairs(names) do
            for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
                local revoked, armed = false, false
                local ids = {}; ids[index] = 5.75
                setup(function() armed = true; return ids end,
                    function(_, id) assert(not revoked and id == 5.75); return 1 end, function() return 2 end)
                local namespace, fn = C_EncounterTimeline, C_EncounterTimeline[name]
                if phase == "lookup" then
                    namespace[name] = nil
                    setmetatable(namespace, { __index = function(_, key) if key == name then revoked = true; return fn end end })
                end
                issecretvalue = function(v)
                    if armed and phase == "secret" and rawequal(v, fn) then revoked = true end
                    return rawequal(v, secret)
                end
                canaccessvalue = function(v)
                    if armed and ((phase == "namespace" and rawequal(v, namespace)) or (phase == "access" and rawequal(v, fn))) then revoked = true end
                    return not rawequal(v, secret) and not (revoked and rawequal(v, 5.75))
                end
                local r = capture()
                assert(r.entries[index].queries[name].status == "restricted-input", name .. phase)
                assert(r.highlight[2].status == "observed")
            end
        end
    end
end)
test("list receiver checked at every index and after tuple serialization", function()
    for denied = 1, 8 do
        local revoked, reads = false, 0
        local list = setmetatable({}, { __index = function(_, i)
            assert(not revoked); reads = reads + 1; if i == denied then revoked = true end; return i
        end })
        setup(function() return list end, function() return 1 end, function() return 2 end)
        canaccessvalue = function(v) return not (revoked and rawequal(v, list)) end
        capture(); assert(reads == denied and calls == 3 + denied * 4)
    end
    local list, marker, revoked = { 1 }, {}, false
    setup(function() return list, marker end, function() error("unexpected") end, function() return 2 end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not (revoked and rawequal(v, list)) end
    local r = capture(); assert(calls == 3 and r.highlight[1].status == "observed")
end)
test("opaque timers secret outputs and no raw object retention", function()
    local weak = setmetatable({}, { __mode = "v" })
    local reads = 0
    setup(function() local list = { 1 }; weak[1] = list; return list end, function(name)
        if name ~= "GetEventTimer" then return secret end
        local object = newproxy(true)
        getmetatable(object).__index = function() reads = reads + 1; error("timer method") end
        weak[2] = object; return object
    end, function() return secret end)
    local r = capture()
    assert(r.entries[1].queries.GetEventTimer.values[1].kind == "userdata")
    assert(r.entries[1].queries.GetEventState.values[1].status == "restricted")
    assert(r.highlight[1].values[1].status == "restricted" and reads == 0)
    collectgarbage("collect"); collectgarbage("collect")
    assert(weak[1] == nil and weak[2] == nil)
end)
test("producer output and earlier query guards can revoke later inputs", function()
    local revoked, marker = false, {}
    setup(function() return { 4.5 } end, function(name)
        assert(not revoked); if name == names[1] then return marker end; return 2
    end, function() return 3 end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not (revoked and rawequal(v, 4.5)) end
    local r = capture()
    assert(r.entries[1].queries[names[2]].status == "restricted-input" and calls == 4)
    assert(r.highlight[2].values[1].value == 3)
end)
test("35 call bound ten snapshots sixteen tuple positions and scalar caps", function()
    local ids, values = {}, {}
    for i = 1, 20 do ids[i] = i; values[i] = string.rep("x", 300) end
    setup(function() return ids end, function() return unpack(values, 1, 20) end,
        function() return unpack(values, 1, 20) end)
    local r = capture(string.rep("l", 200))
    assert(calls == 35 and #r.entries == 8 and #ApiContractProbeDB.captures[1].label == 128)
    local q = r.entries[8].queries.GetEventTimer
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
    assert(r.highlight[2].n == 20 and #r.highlight[2].values == 16)
    for _ = 1, 10 do capture() end
    assert(calls == 350 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1 and excluded == 0)
end)
test("all excludes lifecycle queries and highlights", function()
    setup(function() error("unexpected producer") end, function() error("unexpected query") end, function() error("unexpected highlight") end)
    SlashCmdList.APICONTRACTPROBE("all control")
    assert(calls == 0 and excluded == 0 and ApiContractProbeDB.captures[1].timelineLifecycleRead == nil)
    assert(capture().producer.status == "call-error")
end)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
