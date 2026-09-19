local root = assert(arg[1])
local passed, failed, excluded = 0, 0, 0
local names = { "Queued", "Short", "Medium", "Long", "Indeterminate" }
local inputs = { 10.5, -20.5, 0, 77.25, 101 }
local fields = { "id", "type", "minimumDuration", "maximumDuration", "minimumEventIntroDuration",
    "minimumEventGapDuration", "maximumEventCount", "sortDirection" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function forbidden()
    excluded = excluded + 1
    error("excluded operation")
end
local function setup(fn)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(value) return rawequal(value, secret) end
    canaccessvalue = function(value) return not rawequal(value, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { EncounterTimelineTrack = {} }
    for index, name in ipairs(names) do Enum.EncounterTimelineTrack[name] = inputs[index] end
    Enum.EncounterTimelineTrack.Extra = 900
    C_EncounterTimeline = { GetTrackInfo = fn, GetEventList = forbidden,
        AddScriptEvent = forbidden, CancelScriptEvent = forbidden, FinishScriptEvent = forbidden,
        AddEditModeEvents = forbidden, CancelEditModeEvents = forbidden, SetEventIconTextures = forbidden }
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    excluded = 0
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("timeline-track-info " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "timeline-track-info mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].timelineTrackInfo,
        "timeline-track-info capture absent")
end
local function query(result, index) return result.tracks[index].observation end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok and excluded ~= 0 then ok, err = false, "excluded operation invoked" end
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("five published names forward original noncanonical values and eight fields", function()
    local calls = 0
    setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 1 and (...) == inputs[calls])
        local object = {}
        for index, key in ipairs(fields) do object[key] = index * 13 + calls end
        object.extra = secret
        return object
    end)
    local result = capture()
    assert(calls == 5 and #result.tracks == 5)
    for index, row in ipairs(result.tracks) do
        assert(row.name == names[index])
        local q = row.observation
        assert(q.status == "observed" and q.n == 1 and q.input.value == inputs[index])
        for offset, key in ipairs(fields) do assert(q.info.fields[key].value == offset * 13 + index) end
        assert(q.info.fields.extra == nil)
    end
end)

test("fresh publication reads preserve duplicates without numeric fallback", function()
    local calls = 0
    setup(function(value)
        calls = calls + 1
        assert(value == (calls == 1 and inputs[1] or 99.5))
        for _, name in ipairs(names) do Enum.EncounterTimelineTrack[name] = 99.5 end
        return { id = value }
    end)
    assert(query(capture(), 5).info.fields.id.value == 99.5 and calls == 5)
end)

test("invalid enum members skip only their own query", function()
    local invalid = { false, "0", {}, function() end, secret, math.huge, -math.huge, 0/0 }
    for position, name in ipairs(names) do
        for index = 0, #invalid do
            local calls = 0
            setup(function() calls = calls + 1; return {} end)
            if index == 0 then Enum.EncounterTimelineTrack[name] = nil
            else Enum.EncounterTimelineTrack[name] = invalid[index] end
            assert(query(capture(), position).status == "unavailable-enum" and calls == 4)
        end
    end
end)

test("publication containers are guarded before lookup", function()
    for _, level in ipairs({ "root", "members" }) do
        for _, phase in ipairs({ "secret", "access" }) do
            local reads, calls = 0, 0
            setup(function() calls = calls + 1 end)
            local container = setmetatable({}, { __index = function() reads = reads + 1; error(secret) end })
            if level == "root" then Enum = container else Enum.EncounterTimelineTrack = container end
            issecretvalue = function(v) return rawequal(v, secret) or (phase == "secret" and rawequal(v, container)) end
            canaccessvalue = function(v) return not rawequal(v, secret) and not (phase == "access" and rawequal(v, container)) end
            assert(query(capture(), 1).status == "unavailable-enum" and reads == 0 and calls == 0)
        end
    end
end)

test("every enum is rechecked after namespace lookup and both function guards", function()
    for position = 1, 5 do
        for _, phase in ipairs({ "namespace", "lookup", "function-secret", "function-access" }) do
            local lookups, calls, revoked = 0, 0, false
            local blocked = inputs[position]
            local fn = function(value)
                assert(not (revoked and value == blocked), "revoked enum forwarded")
                calls = calls + 1
                return {}
            end
            setup(fn)
            local namespace = setmetatable({}, { __index = function(_, key)
                assert(key == "GetTrackInfo")
                if phase ~= "namespace" then lookups = lookups + 1 end
                if phase == "lookup" and lookups == position then revoked = true end
                return fn
            end })
            C_EncounterTimeline = namespace
            issecretvalue = function(value)
                if phase == "function-secret" and rawequal(value, fn) and lookups == position then revoked = true end
                return rawequal(value, secret)
            end
            canaccessvalue = function(value)
                if phase == "namespace" and rawequal(value, namespace) then
                    lookups = lookups + 1
                    if lookups == position then revoked = true end
                end
                if phase == "function-access" and rawequal(value, fn) and lookups == position then revoked = true end
                return not rawequal(value, secret) and not (revoked and rawequal(value, blocked))
            end
            assert(query(capture(), position).status == "restricted-input" and calls == 4)
        end
    end
end)

test("every field read rechecks the receiver after earlier field serialization", function()
    for position = 1, 5 do
        for offset, key in ipairs(fields) do
            for _, userdata in ipairs({ false, true }) do
                local calls, reads, revoked = 0, 0, false
                local marker = {}
                local object = userdata and newproxy(true) or {}
                local mt = userdata and getmetatable(object) or {}
                if not userdata then setmetatable(object, mt) end
                mt.__index = function(_, requested)
                    assert(not revoked, "revoked receiver indexed")
                    reads = reads + 1
                    assert(requested == fields[reads], "unexpected field")
                    return requested == key and marker or 12
                end
                setup(function() calls = calls + 1; return calls == position and object or {} end)
                canaccessvalue = function(value)
                    if rawequal(value, marker) then revoked = true end
                    return not rawequal(value, secret) and not (revoked and rawequal(value, object))
                end
                local info = query(capture(), position).info
                assert(reads == offset and info.fields[key].kind == "table")
                for later = offset + 1, #fields do assert(info.fields[fields[later]].status == "field-error") end
                assert(calls == 5)
            end
        end
    end
end)

test("tuple serialization can revoke first object and field values remain opaque", function()
    local object, checks, reads = {}, 0, 0
    setmetatable(object, { __index = function() reads = reads + 1; error("revoked lookup") end })
    setup(function() return object end)
    canaccessvalue = function(value)
        if rawequal(value, object) then checks = checks + 1; return checks == 1 end
        return not rawequal(value, secret)
    end
    assert(query(capture(), 1).info.status == "restricted" and reads == 0)
    setup(function() return { id = secret, type = false, maximumDuration = 0/0 } end)
    local info = query(capture(), 1).info
    assert(info.fields.id.status == "restricted" and info.fields.type.value == false)
    assert(info.fields.maximumDuration.status == "nonfinite" and info.fields.minimumDuration.kind == "nil")
end)

test("namespace and function failures do not suppress subsequent calls", function()
    for _, fn in ipairs({ false, 9, secret }) do
        setup(fn)
        assert(query(capture(), 5).status == "missing-api")
    end
    setup(nil)
    assert(query(capture(), 1).status == "missing-api")
    setup(function() error("must not call") end)
    C_EncounterTimeline = secret
    assert(query(capture(), 1).status == "field-error")
    setup(function()
        C_EncounterTimeline.GetTrackInfo = function() return { id = "replacement" } end
        error(secret)
    end)
    local result = capture()
    assert(query(result, 1).status == "call-error" and query(result, 2).info.fields.id.value == "replacement")
end)

test("exact arity nil holes and first-object-only inspection", function()
    local calls = 0
    local extra = setmetatable({}, { __index = forbidden })
    setup(function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil end
        if calls == 3 then return nil, extra end
        if calls == 4 then return false, extra, nil end
        return { id = 42 }, extra, nil, secret
    end)
    local result = capture()
    assert(query(result, 1).n == 0 and query(result, 2).n == 1)
    assert(query(result, 3).info.kind == "nil" and query(result, 3).n == 2)
    assert(query(result, 4).info.value == false and query(result, 4).values[3].kind == "nil")
    assert(query(result, 5).info.fields.id.value == 42 and query(result, 5).values[4].status == "restricted")
end)

test("five calls per snapshot and tuple string label capture limits", function()
    local calls, values = 0, {}
    for index = 1, 20 do values[index] = string.rep("x", 300) end
    values[1] = { id = string.rep("y", 300) }
    setup(function() calls = calls + 1; return unpack(values) end)
    local q = query(capture(string.rep("l", 200)), 1)
    assert(q.n == 20 and q.truncated and #q.values == 16 and q.values[17] == nil)
    assert(#q.values[2].value == 256 and #q.info.fields.id.value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(calls == 50 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("returned objects and opaque field values are collectible", function()
    local weak, count = setmetatable({}, { __mode = "v" }), 0
    setup(function()
        local object, nested = {}, newproxy(true)
        getmetatable(nested).__index, getmetatable(nested).__tostring = forbidden, forbidden
        object.id = nested
        count = count + 1; weak[count] = object
        count = count + 1; weak[count] = nested
        return object, function() forbidden() end
    end)
    assert(query(capture(), 1).info.fields.id.kind == "userdata")
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
end)

test("missing or throwing access guards fail closed and all excludes the mode", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        local calls = 0
        setup(function() calls = calls + 1 end)
        _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("timeline-track-info missing")
        assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
        setup(function() calls = calls + 1 end)
        _G[guard] = function() error(secret) end
        assert(query(capture(), 1).status == "unavailable-enum" and calls == 0)
    end
    local calls = 0
    setup(function() calls = calls + 1 end)
    SlashCmdList.APICONTRACTPROBE("all context")
    assert(calls == 0 and ApiContractProbeDB.captures[1].timelineTrackInfo == nil)
end)

print(string.format("%d passed, %d failed timeline-track-info fixtures", passed, failed))
if failed > 0 then os.exit(1) end
