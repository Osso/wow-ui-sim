local root = assert(arg[1], "addon directory required")
local passed, failed, calls, excluded = 0, 0, 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspection") end
getmetatable(secret).__tostring = function() error("secret formatting") end
local fields = { "id", "source", "spellName", "spellID", "iconFileID", "duration",
    "maxQueueDuration", "icons", "severity", "isApproximate" }
local function setup(producer, info, color)
    ApiContractProbeDB, SlashCmdList, calls, excluded = nil, {}, 0, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_EncounterTimeline = {
        GetEventList = function(...) calls = calls + 1; assert(select("#", ...) == 0); return producer() end,
        GetEventInfo = function(...) calls = calls + 1; assert(select("#", ...) == 1); return info(...) end,
        GetEventColor = function(...) calls = calls + 1; assert(select("#", ...) == 1); return color(...) end,
    }
    local function forbidden() excluded = excluded + 1; error("excluded operation") end
    for _, name in ipairs({ "AddScriptEvent", "CancelScriptEvent", "FinishScriptEvent", "SetEventIconTextures" }) do
        C_EncounterTimeline[name] = forbidden
    end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("timeline-current-events " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "timeline-current-events mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].timelineCurrentEvents)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function event(id)
    return { id = id, source = 4, spellName = "fixture", spellID = 123, iconFileID = 456,
        duration = 8.5, maxQueueDuration = 2, icons = 3, severity = 7, isApproximate = false }
end

test("original fractional duplicate IDs and all ten fields; color remains opaque", function()
    local seen, colorReads = {}, 0
    local color = setmetatable({}, { __index = function() colorReads = colorReads + 1; error("color traversal") end })
    setup(function() return { 4.5, 4.5, -2.25 } end,
        function(id) seen[#seen + 1] = id; return event(id) end,
        function(id) seen[#seen + 1] = id; return color end)
    local r = capture()
    assert(calls == 7 and #seen == 6 and seen[1] == 4.5 and seen[3] == 4.5 and seen[5] == -2.25)
    for _, key in ipairs(fields) do
        assert(r.entries[1].info.values[1].fields[key].value == event(4.5)[key], key)
    end
    assert(r.entries[1].color.values[1].kind == "table" and not r.entries[1].color.values[1].fields)
    assert(colorReads == 0 and excluded == 0)
end)
test("raw producer and query nil holes zero returns and errors remain independent", function()
    setup(function() return { 2, 3 }, nil, 6 end,
        function(id) if id == 2 then return nil, 9, nil end; error(secret) end,
        function(id) if id == 2 then return end; return nil, false, nil end)
    local r = capture()
    assert(r.producer.n == 3 and r.producer.values[2].kind == "nil")
    assert(r.entries[1].info.n == 3 and r.entries[1].info.values[3].kind == "nil")
    assert(r.entries[1].color.n == 0 and r.entries[2].info.status == "call-error")
    assert(r.entries[2].color.n == 3 and r.entries[2].color.values[2].value == false and calls == 5)
end)
test("first producer result only, secret malformed and unavailable lists", function()
    for _, value in ipairs({ secret, false, 9, "list", newproxy(true) }) do
        setup(function() return value, { 2 } end, function() error("unexpected") end, function() error("unexpected") end)
        assert(next(capture().entries) == nil and calls == 1)
    end
    setup(function() return nil, { 2 } end, function() end, function() end)
    assert(next(capture().entries) == nil)
    setup(function() return end, function() end, function() end)
    assert(capture().producer.n == 0)
    setup(function() error(secret) end, function() end, function() end)
    assert(capture().producer.status == "call-error")
end)
test("holes invalid and secret IDs skipped without suppressing later entries", function()
    local ids = { [2] = secret, [3] = math.huge, [4] = -math.huge, [5] = 0/0, [6] = "12", [7] = {}, [8] = -3.5 }
    setup(function() return ids end, function(id) assert(id == -3.5); return event(id) end,
        function(id) assert(id == -3.5); return false end)
    local r = capture()
    assert(calls == 3 and r.entries[1].info.status == "unavailable-input")
    assert(r.entries[2].color.status == "restricted-input" and r.entries[8].color.values[1].value == false)
end)
test("missing namespace functions lookup errors and guard failures", function()
    setup(function() return { 1 } end, function() return event(1) end, function() return true end)
    C_EncounterTimeline = secret; assert(capture().producer.status == "field-error")
    C_EncounterTimeline = {}; assert(capture().producer.status == "missing-api")
    C_EncounterTimeline = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().producer.status == "field-error")
    setup(function() return { 1 } end, function() return event(1) end, function() return true end)
    C_EncounterTimeline.GetEventInfo = secret
    local r = capture(); assert(r.entries[1].info.status == "missing-api" and r.entries[1].color.values[1].value == true)
    setup(function() return { 1 } end, function() end, function() end)
    canaccessvalue = function() error(secret) end
    assert(capture().producer.status == "field-error" and calls == 0)
    setup(function() return { 1 } end, function() end, function() end)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("timeline-current-events missing")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
end)
test("original ID rechecked after namespace lookup and both function guards", function()
    for _, name in ipairs({ "GetEventInfo", "GetEventColor" }) do
        for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
            local revoked, armed = false, false
            setup(function() armed = true; return { 5.75 } end,
                function() assert(not revoked); return event(5.75) end,
                function() assert(not revoked); return true end)
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
            local r = capture(); local key = name == "GetEventInfo" and "info" or "color"
            assert(r.entries[1][key].status == "restricted-input", name .. ":" .. phase)
            assert(calls == (name == "GetEventColor" and phase ~= "namespace" and 2 or 1))
        end
    end
end)
test("list access rechecked at every index and after producer tuple observation", function()
    for denied = 1, 8 do
        local revoked, reads = false, 0
        local list = setmetatable({}, { __index = function(_, i) assert(not revoked); reads = reads + 1; if i == denied then revoked = true end; return i end })
        setup(function() return list end, function(id) return event(id) end, function() return true end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
        capture(); assert(reads == denied and calls == 1 + denied * 2)
    end
    local revoked, list, marker = false, { 1 }, {}
    setup(function() return list, marker end, function() error("unexpected") end, function() error("unexpected") end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not (revoked and rawequal(v, list)) end
    capture(); assert(calls == 1)
end)
test("info receiver rechecked before every fixed field, including userdata", function()
    for _, asUserdata in ipairs({ false, true }) do
        for denied = 1, #fields do
            local revoked, reads = false, 0
            local object = asUserdata and newproxy(true) or {}
            local mt = asUserdata and getmetatable(object) or {}
            mt.__index = function(_, key)
                assert(not revoked and key == fields[reads + 1]); reads = reads + 1
                if reads == denied then revoked = true end
                return key
            end
            if not asUserdata then setmetatable(object, mt) end
            setup(function() return { 2 } end, function() return object end, function() return true end)
            canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
            local r = capture(); assert(reads == denied and calls == 3)
            if denied < #fields then assert(r.entries[1].info.values[1].fields[fields[denied + 1]].status == "field-error") end
        end
    end
end)
test("secret fields stay opaque and info errors do not suppress color", function()
    local object = event(1); for _, key in ipairs(fields) do object[key] = secret end
    setup(function() return { 1 } end, function() return object end, function() return secret end)
    local r = capture()
    for _, key in ipairs(fields) do assert(r.entries[1].info.values[1].fields[key].status == "restricted") end
    assert(r.entries[1].color.values[1].status == "restricted")
    setup(function() return { 1 } end, function() return secret end, function() return true end)
    r = capture(); assert(r.entries[1].info.values[1].status == "restricted" and calls == 3)
    setup(function() return { 1 } end, function() return setmetatable({}, { __index = function() error(secret) end }) end, function() return true end)
    r = capture(); assert(r.entries[1].info.values[1].fields.id.status == "field-error" and r.entries[1].color.values[1].value == true)
end)
test("info tuple and field serialization revocation blocks later inspections and calls", function()
    local revoked, object, marker = false, event(4.5), {}
    setup(function() return { 4.5 } end, function() return object, marker end, function() return true end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not (revoked and rawequal(v, object)) end
    assert(capture().entries[1].info.values[1].status == "restricted")
    revoked = false; object = event(4.5); object.spellName = marker
    setup(function() return { 4.5 } end, function() return object end, function() error("revoked ID forwarded") end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not (revoked and rawequal(v, 4.5)) end
    local r = capture(); assert(calls == 2 and r.entries[1].color.status == "restricted-input")
end)
test("bounded tuples strings labels snapshots and first eight IDs only", function()
    local list = {}; for i = 1, 12 do list[i] = i end
    local tuple = { string.rep("x", 300) }; for i = 2, 17 do tuple[i] = i end
    setup(function() return list end, function(id) local object = event(id); object.spellName = string.rep("s", 300); return object, unpack(tuple, 2, 17) end,
        function() return unpack(tuple) end)
    local r = capture(string.rep("l", 200))
    assert(calls == 17 and #r.entries == 8 and #ApiContractProbeDB.captures[1].label == 128)
    assert(r.entries[1].info.n == 17 and r.entries[1].info.truncated and #r.entries[1].info.values == 16)
    assert(#r.entries[1].info.values[1].fields.spellName.value == 256)
    assert(r.entries[1].color.n == 17 and r.entries[1].color.truncated and #r.entries[1].color.values[1].value == 256)
    for _ = 1, 10 do capture() end
    assert(calls == 170 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1 and excluded == 0)
end)
test("manual-only and no retained event or color objects", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local list = { 1 }; weak[1] = list; return list end,
        function() local object = event(1); weak[2] = object; return object end,
        function() local color = newproxy(true); getmetatable(color).__index = function() error("color methods") end; weak[3] = color; return color end)
    capture(); collectgarbage("collect"); collectgarbage("collect")
    assert(weak[1] == nil and weak[2] == nil and weak[3] == nil and excluded == 0)
    setup(function() error("manual producer invoked") end, function() end, function() end)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and excluded == 0 and not ApiContractProbeDB.captures[1].timelineCurrentEvents)
end)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
