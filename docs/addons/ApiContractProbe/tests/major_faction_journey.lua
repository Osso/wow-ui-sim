local root = assert(arg[1], "addon directory required")
local passed, failed, calls = 0, 0, 0
local names = { "ShouldDisplayMajorFactionAsJourney", "ShouldUseJourneyRewardTrack" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspection") end
local function setup(producer, predicate)
    ApiContractProbeDB, SlashCmdList, calls = nil, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_MajorFactions = { GetMajorFactionIDs = function(...)
        calls = calls + 1; assert(select("#", ...) == 1 and (...) == nil)
        return producer()
    end }
    for _, name in ipairs(names) do
        C_MajorFactions[name] = function(...)
            calls = calls + 1; assert(select("#", ...) == 1)
            return predicate(...)
        end
    end
    local function forbidden() error("excluded call") end
    for _, name in ipairs({ "GetMajorFactionData", "GetMajorFactionRenownInfo", "GetRenownLevels", "UnlockMajorFaction" }) do
        C_MajorFactions[name] = forbidden
    end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("major-faction-journey " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "major-faction-journey mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].majorFactionJourney)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("explicit nil producer and original fractional duplicate IDs", function()
    local seen = {}
    setup(function() return { 12.5, 12.5, -3.25 } end, function(id) seen[#seen + 1] = id; return false end)
    local r = capture(); assert(calls == 7 and #seen == 6)
    assert(seen[1] == 12.5 and seen[3] == 12.5 and seen[5] == -3.25)
    assert(r.entries[1].queries[names[1]].values[1].value == false)
end)
test("raw zero arity nil holes and opaque errors", function()
    setup(function() return { 2 }, nil, 7 end, function() return nil, 4, nil end)
    local r = capture(); assert(r.producer.n == 3 and r.producer.values[2].kind == "nil")
    assert(r.entries[1].queries[names[1]].n == 3 and r.entries[1].queries[names[1]].values[3].kind == "nil")
    C_MajorFactions[names[1]] = function() error(secret) end
    C_MajorFactions[names[2]] = function() end
    r = capture(); assert(r.entries[1].queries[names[1]].status == "call-error" and r.entries[1].queries[names[2]].n == 0)
end)
test("missing producer and invalid first return never invent IDs", function()
    for _, value in ipairs({ false, secret, 9 }) do
        setup(function() return value, { 42 } end, function() error("unexpected predicate") end)
        assert(next(capture().entries) == nil and calls == 1)
    end
    setup(function() error(secret) end, function() end)
    assert(capture().producer.status == "call-error")
    C_MajorFactions.GetMajorFactionIDs = nil
    assert(capture().producer.status == "missing-api")
end)
test("invalid and restricted IDs do not suppress peers", function()
    setup(function() return { secret, false, "42", math.huge, 0/0, 3.5 } end, function(id) assert(id == 3.5); return true end)
    local r = capture(); assert(calls == 3)
    assert(r.entries[1].queries[names[1]].status == "restricted-input")
    assert(r.entries[6].queries[names[2]].values[1].value == true)
end)
test("namespace and function guards preserve independent outcomes", function()
    setup(function() return { 2 } end, function() return true end)
    C_MajorFactions[names[1]] = secret
    local r = capture(); assert(r.entries[1].queries[names[1]].status == "missing-api" and calls == 2)
    C_MajorFactions = secret; assert(capture().producer.status == "field-error")
    C_MajorFactions = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().producer.status == "field-error")
end)
test("ID revoked by lookup and both function guards at each predicate", function()
    for _, target in ipairs(names) do
        for _, phase in ipairs({ "lookup", "secret", "access" }) do
            local revoked = false
            setup(function() return { 7.5 } end, function() assert(not revoked); return true end)
            local original = C_MajorFactions[target]
            if phase == "lookup" then
                C_MajorFactions[target] = nil
                setmetatable(C_MajorFactions, { __index = function(_, key)
                    if key == target then revoked = true; return original end
                end })
            end
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, original) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "access" and rawequal(v, original) then revoked = true end
                return not rawequal(v, secret) and not (revoked and v == 7.5)
            end
            assert(capture().entries[1].queries[target].status == "restricted-input")
        end
    end
end)
test("list receiver guarded before every index", function()
    for stop = 1, 8 do
        local revoked = false
        local list = setmetatable({}, { __index = function(_, index)
            assert(not revoked); if index == stop then revoked = true end; return 2
        end })
        setup(function() return list end, function() return true end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (rawequal(v, list) and revoked) end
        local r = capture()
        if stop < 8 then assert(r.entries[stop + 1].status == "field-error") end
    end
end)
test("earlier output serialization can revoke next input", function()
    local revoked = false
    setup(function() return { 8 } end, function() return "revoke" end)
    canaccessvalue = function(v)
        if v == "revoke" then revoked = true end
        return not rawequal(v, secret) and not (revoked and v == 8)
    end
    local r = capture(); assert(r.entries[1].queries[names[2]].status == "restricted-input" and calls == 2)
end)
test("eight IDs seventeen calls tuple string label and snapshot limits", function()
    local ids = {}; for i = 1, 20 do ids[i] = i end
    setup(function() return ids end, function()
        local values = {}; for i = 1, 20 do values[i] = string.rep("x", 300) end
        return unpack(values)
    end)
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(calls == 170 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local row = ApiContractProbeDB.captures[1]
    assert(#row.label == 128 and #row.majorFactionJourney.entries == 8)
    local q = row.majorFactionJourney.entries[1].queries[names[1]]
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
end)
test("opaque objects are collectible and all excludes the mode", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local list = { 3 }; weak[1] = list; return list end, function()
        local obj = setmetatable({}, { __index = function() error("opaque lookup") end }); weak[2] = obj; return obj
    end)
    capture(); collectgarbage(); collectgarbage(); assert(weak[1] == nil and weak[2] == nil)
    local before = calls; SlashCmdList.APICONTRACTPROBE("all other"); assert(calls == before)
end)
test("data original IDs fields highlights and first object only", function()
    local seen = {}
    setup(function() return { 12.5, -3.25 } end, function() return true end)
    C_MajorFactions.GetMajorFactionData = function(...)
        assert(select("#", ...) == 1); local id = ...; seen[#seen + 1] = id
        return { description = "faction", playerCompanionID = 9,
            highlights = { { title = "title", description = "detail", level = 3 } } }, nil, "tail"
    end
    local r = capture(); local q = assert(r.entries[1].queries.GetMajorFactionData, "data query absent")
    assert(#seen == 2 and seen[1] == 12.5 and seen[2] == -3.25)
    assert(q.n == 3 and q.values[2].kind == "nil" and q.values[3].value == "tail")
    assert(q.data.fields.description.value == "faction" and q.data.fields.playerCompanionID.value == 9)
    assert(q.data.fields.highlights.kind == "table" and q.data.fields.highlights.fields == nil)
    assert(q.data.highlights.entries[1].fields.title.value == "title")
    assert(q.data.highlights.entries[1].fields.description.value == "detail")
    assert(q.data.highlights.entries[1].fields.level.value == 3)
end)
test("data input revoked after lookup and both function guards", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        setup(function() return { 7.5 } end, function() return true end)
        local revoked = false
        local fn = function() error("revoked ID forwarded") end
        C_MajorFactions.GetMajorFactionData = fn
        if phase == "lookup" then
            C_MajorFactions.GetMajorFactionData = nil
            setmetatable(C_MajorFactions, { __index = function(_, key)
                if key == "GetMajorFactionData" then revoked = true; return fn end
            end })
        end
        issecretvalue = function(v)
            if phase == "secret" and rawequal(v, fn) then revoked = true end
            return rawequal(v, secret)
        end
        canaccessvalue = function(v)
            if phase == "access" and rawequal(v, fn) then revoked = true end
            return not rawequal(v, secret) and not (revoked and v == 7.5)
        end
        local q = capture().entries[1].queries
        assert(q.GetMajorFactionData.status == "restricted-input")
        assert(q[names[1]].status == "observed" and q[names[2]].status == "observed")
    end
end)
test("data fields list indices and highlight fields recheck receivers", function()
    for _, keys in ipairs({ { "description", "playerCompanionID", "highlights" },
        { 1, 2, 3, 4 }, { "title", "description", "level" } }) do
        for stop = 1, #keys do
            setup(function() return { 2 } end, function() return true end)
            local revoked, reads = false, 0
            local receiver = setmetatable({}, { __index = function(_, key)
                assert(not revoked, "revoked receiver indexed")
                reads = reads + 1; assert(key == keys[reads])
                if reads == stop then revoked = true end
                return nil
            end })
            local data = receiver
            if type(keys[1]) == "number" then data = { highlights = receiver }
            elseif keys[1] == "title" then data = { highlights = { receiver } } end
            C_MajorFactions.GetMajorFactionData = function() return data end
            canaccessvalue = function(v) return not rawequal(v, secret) and not (rawequal(v, receiver) and revoked) end
            local q = capture().entries[1].queries.GetMajorFactionData
            assert(q and q.data and reads == stop)
        end
    end
end)
test("data nil errors and inaccessible highlights stay independent", function()
    setup(function() return { 1, 2, 3, 4 } end, function() return false end)
    C_MajorFactions.GetMajorFactionData = function(id)
        if id == 1 then error(secret) end
        if id == 2 then return nil, { description = "wrong return" } end
        if id == 3 then return { description = secret, highlights = secret } end
    end
    local r = capture()
    assert(r.entries[1].queries.GetMajorFactionData.status == "call-error")
    assert(r.entries[2].queries.GetMajorFactionData.data.kind == "nil")
    assert(r.entries[3].queries.GetMajorFactionData.data.fields.description.status == "restricted")
    assert(r.entries[3].queries.GetMajorFactionData.data.highlights.status == "restricted")
    assert(r.entries[4].queries.GetMajorFactionData.n == 0)
    assert(r.entries[4].queries[names[2]].values[1].value == false)
end)
test("data twenty five calls four highlights bounded tuples and collection", function()
    local ids = {}; for i = 1, 20 do ids[i] = i end
    local weak = setmetatable({}, { __mode = "v" })
    local dataCalls = 0
    setup(function() return ids end, function() return true end)
    C_MajorFactions.GetMajorFactionData = function()
        dataCalls = dataCalls + 1
        local highlights = setmetatable({}, { __index = function(_, index)
            assert(index <= 4); local entry = { title = string.rep("x", 300), description = "d", level = index }
            weak[#weak + 1] = entry; return entry
        end })
        local data = setmetatable({ description = "d", playerCompanionID = 1, highlights = highlights }, {
            __index = function() error("unlisted data field") end })
        weak[#weak + 1] = data; weak[#weak + 1] = highlights
        local values = { data }; for i = 2, 20 do values[i] = string.rep("z", 300) end
        return unpack(values)
    end
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(calls + dataCalls == 250 and dataCalls == 80)
    local q = ApiContractProbeDB.captures[1].majorFactionJourney.entries[1].queries.GetMajorFactionData
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[16].value == 256)
    assert(#q.data.highlights.entries == 4 and #q.data.highlights.entries[4].fields.title.value == 256)
    collectgarbage(); collectgarbage(); assert(next(weak) == nil)
    local before = dataCalls; SlashCmdList.APICONTRACTPROBE("all other"); assert(dataCalls == before)
end)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
