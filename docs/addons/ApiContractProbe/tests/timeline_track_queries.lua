local root = assert(arg[1], "addon directory required")
local passed, failed, calls, excluded = 0, 0, 0, 0
local fields = { "id", "type", "minimumDuration", "maximumDuration", "minimumEventIntroDuration",
    "minimumEventGapDuration", "maximumEventCount", "sortDirection" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret formatting") end
local function forbidden() excluded = excluded + 1; error("excluded operation") end
local function setup(list, track, tracks, visible)
    ApiContractProbeDB, SlashCmdList, calls, excluded = nil, {}, 0, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_EncounterTimeline = {
        GetEventList = function(...) calls = calls + 1; assert(select("#", ...) == 0); return list() end,
        GetEventTrack = function(...) calls = calls + 1; assert(select("#", ...) == 1); return track(...) end,
        GetTrackList = function(...) calls = calls + 1; assert(select("#", ...) == 0); return tracks() end,
        HasVisibleEvents = function(...) calls = calls + 1; assert(select("#", ...) == 0); return visible() end,
    }
    for _, name in ipairs({ "AddEditModeEvents", "CancelEditModeEvents", "AddScriptEvent",
        "CancelScriptEvent", "FinishScriptEvent", "PauseScriptEvent", "ResumeScriptEvent", "SetEventIconTextures" }) do
        C_EncounterTimeline[name] = forbidden
    end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function defaults()
    setup(function() return { 10.5, 20.5 } end, function(id) return id, nil end,
        function() return {} end, function() return false end)
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("timeline-track-queries " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "timeline-track-queries mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].timelineTrackQueries,
        "timeline-track-queries capture absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok and excluded ~= 0 then ok, err = false, "excluded operation invoked" end
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function info(seed)
    local object = {}
    for index, key in ipairs(fields) do object[key] = seed + index / 10 end
    return object
end

test("original fractional duplicate IDs, nil sort indices, and eight track fields", function()
    local seen = {}
    setup(function() return { 4.5, 4.5, -2.25 } end,
        function(id) seen[#seen + 1] = id; return -7.5, nil end,
        function() return { info(91), info(-3) } end, function() return false end)
    local r = capture()
    assert(calls == 6 and #seen == 3 and seen[1] == 4.5 and seen[2] == 4.5 and seen[3] == -2.25)
    assert(r.entries[1].id.value == 4.5 and r.entries[1].track.n == 2)
    assert(r.entries[1].track.values[1].value == -7.5 and r.entries[1].track.values[2].kind == "nil")
    for index, key in ipairs(fields) do assert(r.trackList.entries[1].fields[key].value == 91 + index / 10) end
    assert(r.visible.values[1].value == false and r.trackList.entries[3].kind == "nil")
end)

test("producer failures and wrong first returns do not suppress independent reads", function()
    for _, value in ipairs({ false, 7, "list", secret, newproxy(true) }) do
        setup(function() return value, { 99 } end, forbidden, function() return { info(1) } end,
            function() return true end)
        local r = capture()
        assert(#r.entries == 0 and r.trackList.entries[1].fields.id.value == 1.1 and r.visible.values[1].value)
    end
    setup(function() error(secret) end, forbidden, function() error(secret) end, function() return false end)
    local r = capture()
    assert(r.producer.status == "call-error" and r.trackList.status == "call-error" and r.visible.n == 1)
    defaults(); C_EncounterTimeline.GetEventList = nil
    r = capture(); assert(r.producer.status == "missing-api" and r.visible.status == "observed")
end)

test("invalid and restricted IDs skip independently, without coercion", function()
    local bad = { false, "10", {}, function() end, secret, math.huge, -math.huge, 0/0 }
    for _, value in ipairs(bad) do
        local seen = {}
        setup(function() return { value, 7.25 } end, function(id) seen[#seen + 1] = id; return id end,
            function() return {} end, function() return true end)
        local r = capture()
        assert(#seen == 1 and seen[1] == 7.25 and r.entries[1].track.status ~= "observed")
    end
end)

test("every event ID is rechecked after namespace lookup and function guards", function()
    for position = 1, 8 do
        for _, phase in ipairs({ "namespace-secret", "namespace-access", "lookup", "function-secret", "function-access" }) do
            local ids, revoked, forwarded = {}, false, 0
            for i = 1, 8 do ids[i] = 100 + i / 10 end
            local blocked = ids[position]
            setup(function() return ids end, function(id)
                assert(not (revoked and id == blocked), "revoked ID forwarded")
                forwarded = forwarded + 1; return id, nil
            end, function() return {} end, function() return true end)
            local functions = C_EncounterTimeline
            local lookups, namespace = 0, {}
            setmetatable(namespace, { __index = function(_, key)
                if key == "GetEventTrack" then
                    lookups = lookups + 1
                    if lookups == position and phase == "lookup" then revoked = true end
                end
                return functions[key]
            end })
            C_EncounterTimeline = namespace
            local function denied(v, current)
                if rawequal(v, namespace) and current == phase and lookups == position - 1 then
                    -- The producer namespace guard precedes the event queries.
                    if ApiContractProbeDB and calls >= 1 then revoked = true end
                end
                if rawequal(v, functions.GetEventTrack) and lookups == position and current == phase then revoked = true end
                return rawequal(v, secret) or (revoked and rawequal(v, blocked))
            end
            issecretvalue = function(v) return denied(v, rawequal(v, namespace) and "namespace-secret" or "function-secret") end
            canaccessvalue = function(v) return not denied(v, rawequal(v, namespace) and "namespace-access" or "function-access") end
            local r = capture()
            assert(forwarded == 7 and r.entries[position].track.status == "restricted-input", phase)
            assert(r.visible.status == "observed")
        end
    end
end)

test("both returned lists recheck receiver access before every index", function()
    for _, lane in ipairs({ "events", "tracks" }) do
        for position = 1, (lane == "events" and 8 or 5) do
            local reads, denied = 0, false
            local list = setmetatable({}, { __index = function(_, i)
                assert(not denied, "revoked list indexed")
                reads = reads + 1
                if i == position then denied = true end
                return lane == "events" and (10 + i) or info(i)
            end })
            setup(function() return lane == "events" and list or {} end, function(id) return id end,
                function() return lane == "tracks" and list or {} end, function() return true end)
            canaccessvalue = function(v) return not rawequal(v, secret) and not (denied and rawequal(v, list)) end
            local r = capture()
            assert(reads == position and r.visible.status == "observed")
        end
    end
end)

test("track entries guard every field on table and userdata receivers", function()
    for _, userdata in ipairs({ false, true }) do
        for position, key in ipairs(fields) do
            local revoked, reads = false, 0
            local object = userdata and newproxy(true) or {}
            local mt = userdata and getmetatable(object) or {}
            mt.__index = function(_, name)
                assert(not revoked, "revoked track receiver inspected")
                reads = reads + 1
                if name == key then revoked = true end
                return name
            end
            if not userdata then setmetatable(object, mt) end
            setup(function() return {} end, forbidden, function() return { object } end, function() return true end)
            canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
            local r = capture()
            assert(reads == position and r.trackList.entries[1].fields[key].value == key)
            if position < #fields then assert(r.trackList.entries[1].fields[fields[position + 1]].status == "field-error") end
        end
    end
end)

test("serialization revocation blocks later object and input access", function()
    local id, revoked, forwarded = 17.25, false, 0
    setup(function() return { id } end, function() forwarded = forwarded + 1 end,
        function() return {} end, function() return true end)
    local guards = 0
    canaccessvalue = function(v)
        if rawequal(v, id) then guards = guards + 1; if guards == 1 then revoked = true; return true end end
        return not rawequal(v, secret) and not (revoked and rawequal(v, id))
    end
    local r = capture(); assert(forwarded == 0 and r.entries[1].track.status == "restricted-input")
    local object, denied, reads = {}, false, 0
    setmetatable(object, { __index = function() assert(not denied); reads = reads + 1; return 999.25 end })
    setup(function() return {} end, forbidden, function() return { object } end, function() return true end)
    canaccessvalue = function(v)
        if rawequal(v, 999.25) then denied = true end
        return not rawequal(v, secret) and not (denied and rawequal(v, object))
    end
    r = capture(); assert(reads == 1 and r.trackList.entries[1].fields.type.status == "field-error")
end)

test("raw zero arity, nil holes, errors and fresh function replacement", function()
    setup(function() return { 1, 2, 3 }, nil, 4 end, function(id)
        if id == 1 then return end
        if id == 2 then return 8, nil, 4, nil end
        error(secret)
    end, function() return nil, 9, nil end, function() return nil, false, nil end)
    local r = capture()
    assert(r.producer.n == 3 and r.entries[1].track.n == 0)
    assert(r.entries[2].track.n == 4 and r.entries[2].track.values[4].kind == "nil")
    assert(r.entries[3].track.status == "call-error" and r.trackList.n == 3 and not r.trackList.entries)
    assert(r.visible.n == 3 and r.visible.values[2].value == false)
    setup(function() return { 1, 2 } end, function()
        C_EncounterTimeline.GetEventTrack = function(id) return "replacement", id end
        return "first"
    end, function() return {} end, function() return true end)
    r = capture(); assert(r.entries[2].track.values[1].value == "replacement")
end)

test("namespace and function guards reject secret values without dereference", function()
    defaults(); C_EncounterTimeline = secret
    local r = capture(); assert(r.producer.status == "field-error" and r.visible.status == "field-error")
    for _, name in ipairs({ "GetEventList", "GetEventTrack", "GetTrackList", "HasVisibleEvents" }) do
        defaults(); C_EncounterTimeline[name] = secret
        r = capture()
        if name == "GetEventList" then assert(r.producer.status == "missing-api")
        elseif name == "GetEventTrack" then assert(r.entries[1].track.status == "missing-api")
        elseif name == "GetTrackList" then assert(r.trackList.status == "missing-api")
        else assert(r.visible.status == "missing-api") end
    end
    defaults(); canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("timeline-track-queries guards")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
end)

test("only first lists, first eight IDs, first five entries and fixed fields are inspected", function()
    local extra = setmetatable({}, { __index = forbidden })
    local ids, tracks = {}, {}
    for i = 1, 9 do ids[i] = i; tracks[i] = info(i) end
    ids[9], tracks[6] = secret, secret
    for i = 1, 5 do setmetatable(tracks[i], { __index = forbidden }) end
    setup(function() return ids, extra end, function(id) return id end,
        function() return tracks, extra end, function() return true end)
    local r = capture()
    assert(calls == 11 and #r.entries == 8 and #r.trackList.entries == 5)
    assert(r.producer.values[2].kind == "table" and r.trackList.values[2].kind == "table")
    assert(not r.trackList.entries[1].fields.extra)
end)

test("tuple strings labels and snapshot call bounds are enforced", function()
    local values = { n = 18 }
    for i = 1, 18 do values[i] = string.rep("x", 300) end
    setup(function() return { 1, 2, 3, 4, 5, 6, 7, 8 } end,
        function() return unpack(values, 1, values.n) end,
        function() return { { id = string.rep("y", 300) } } end,
        function() return unpack(values, 1, values.n) end)
    for i = 1, 10 do capture(string.rep("l", 200)) end
    local r = ApiContractProbeDB.captures[1].timelineTrackQueries
    assert(calls == 110 and r.entries[1].track.n == 18 and r.entries[1].track.truncated)
    assert(#r.entries[1].track.values == 16 and #r.visible.values[1].value == 256)
    assert(#r.trackList.entries[1].fields.id.value == 256 and #ApiContractProbeDB.captures[1].label == 128)
    SlashCmdList.APICONTRACTPROBE("timeline-track-queries limit")
    assert(calls == 110 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("opaque values are collectible and old track-info mode remains independent", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() return {} end, forbidden, function()
        local object = newproxy(true)
        getmetatable(object).__index = forbidden
        weak[1] = object
        return false, object
    end, function() return true end)
    assert(capture().trackList.values[2].kind == "userdata")
    collectgarbage("collect"); collectgarbage("collect"); assert(weak[1] == nil)
    defaults()
    Enum = { EncounterTimelineTrack = { Queued = 1, Short = 2, Medium = 3, Long = 4, Indeterminate = 5 } }
    local old = 0
    C_EncounterTimeline.GetTrackInfo = function(v) old = old + 1; return info(v) end
    SlashCmdList.APICONTRACTPROBE("timeline-track-info before")
    assert(old == 5 and calls == 0)
    capture()
    SlashCmdList.APICONTRACTPROBE("all excluded")
    assert(old == 5 and calls == 5)
    SlashCmdList.APICONTRACTPROBE("timeline-track-info after")
    assert(old == 10 and calls == 5)
end)

print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
