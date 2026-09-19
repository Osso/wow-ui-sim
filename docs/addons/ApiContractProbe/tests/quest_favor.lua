local root = assert(arg[1], "addon directory required")
local passed, failed, calls = 0, 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspection") end
local function setup(producer, query)
    ApiContractProbeDB, SlashCmdList, calls = nil, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_NeighborhoodInitiative = { GetNeighborhoodInitiativeInfo = function(...)
        calls = calls + 1; assert(select("#", ...) == 0); return producer()
    end }
    C_QuestInfoSystem = { GetQuestLogRewardFavor = function(...)
        calls = calls + 1; return query(...)
    end }
    local function forbidden() error("excluded operation") end
    C_NeighborhoodInitiative.RequestNeighborhoodInitiativeInfo = forbidden
    C_NeighborhoodInitiative.SetActiveNeighborhood = forbidden
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("quest-favor " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "quest-favor mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].questFavor)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function info(id) return { tasks = { { rewardQuestID = id } } } end

test("original fractional duplicate IDs and exact omitted versus true arity", function()
    local seen = {}
    setup(function() return { tasks = { { rewardQuestID = 2.5 }, { rewardQuestID = 2.5 } } } end,
        function(...) seen[#seen + 1] = { n = select("#", ...), ... }; return false end)
    local r = capture(); assert(calls == 7 and #seen == 6)
    for i = 1, 4 do
        local args = seen[i]
        assert(args[1] == 2.5 and args.n == (i % 2 == 1 and 1 or 2))
        if args.n == 2 then assert(args[2] == true) end
    end
    assert(seen[5].n == 0 and seen[6].n == 2 and seen[6][1] == nil and seen[6][2] == false)
    assert(r.entries[1].omitted.values[1].value == false)
end)
test("raw nil holes zero returns and opaque errors independent", function()
    setup(function() return info(3), nil, 9 end, function(_, clamp) if clamp then return end; return nil, 5, nil end)
    local r = capture(); assert(r.producer.n == 3 and r.producer.values[1].kind == "table")
    assert(r.entries[1].omitted.n == 3 and r.entries[1].omitted.values[3].kind == "nil" and r.entries[1].clamped.n == 0)
    C_QuestInfoSystem.GetQuestLogRewardFavor = function(_, clamp) if not clamp then error(secret) end; return 7 end
    r = capture(); assert(r.entries[1].omitted.status == "call-error" and r.entries[1].clamped.values[1].value == 7)
end)
test("missing malformed and restricted producers do not invent inputs", function()
    for _, v in ipairs({ secret, false, 8 }) do
        setup(function() return v, info(4) end, function() error("unexpected") end)
        assert(next(capture().entries) == nil and calls == 3)
    end
    setup(function() error(secret) end, function() end); assert(capture().producer.status == "call-error")
    C_NeighborhoodInitiative = secret; assert(capture().producer.status == "field-error")
end)
test("invalid IDs skip independently", function()
    setup(function() return { tasks = { { rewardQuestID = secret }, { rewardQuestID = math.huge }, { rewardQuestID = "4" }, { rewardQuestID = -4.5 } } } end,
        function(id) assert(id == -4.5); return 2 end)
    local r = capture(); assert(calls == 5 and r.entries[1].omitted.status == "restricted-input")
    assert(r.entries[4].clamped.values[1].value == 2)
end)
test("ID revocation during lookup and both function guards", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        local revoked = false
        setup(function() return info(7.5) end, function() assert(not revoked); return 1 end)
        local fn = C_QuestInfoSystem.GetQuestLogRewardFavor
        if phase == "lookup" then C_QuestInfoSystem = setmetatable({}, { __index = function() revoked = true; return fn end }) end
        issecretvalue = function(v) if phase == "secret" and rawequal(v, fn) then revoked = true end; return rawequal(v, secret) end
        canaccessvalue = function(v) if phase == "access" and rawequal(v, fn) then revoked = true end; return not rawequal(v, secret) and not (revoked and v == 7.5) end
        assert(capture().entries[1].omitted.status == "restricted-input" and calls == 3)
    end
end)
test("receiver guards before each field and index", function()
    local revoked, reads = false, 0
    local tasks = setmetatable({}, { __index = function(_, index) reads = reads + 1; revoked = true; return { rewardQuestID = index } end })
    setup(function() return { tasks = tasks } end, function() return 1 end)
    canaccessvalue = function(v) return not rawequal(v, secret) and not (rawequal(v, tasks) and revoked) end
    capture(); assert(reads == 1 and calls == 5)
    setup(function() return { tasks = { secret } } end, function() error("unexpected") end)
    assert(capture().entries[1].status == "field-error" and calls == 3)
end)
test("output access revokes ID before independent second call", function()
    local revoked, marker = false, {}
    setup(function() return info(8.5) end, function() return marker end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not rawequal(v, secret) and not (revoked and v == 8.5) end
    local r = capture(); assert(calls == 4 and r.entries[1].clamped.status == "restricted-input")
end)
test("missing query namespace and opaque lookup errors", function()
    setup(function() return info(2) end, function() return 1 end)
    C_QuestInfoSystem = secret; assert(capture().entries[1].omitted.status == "field-error")
    C_QuestInfoSystem = {}; assert(capture().entries[1].clamped.status == "missing-api")
    C_QuestInfoSystem = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().entries[1].clamped.status == "field-error")
end)
test("bounds and manual exclusion", function()
    local tasks = {}; for i = 1, 12 do tasks[i] = { rewardQuestID = i } end
    setup(function() return { tasks = tasks } end, function() return unpack({ string.rep("x", 300), 2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17 }) end)
    local r = capture(string.rep("l", 200)); assert(calls == 11 and #r.entries == 4)
    assert(r.entries[1].omitted.n == 17 and r.entries[1].omitted.truncated and #r.entries[1].omitted.values == 16)
    assert(#r.entries[1].omitted.values[1].value == 256 and #ApiContractProbeDB.captures[1].label == 128)
    for _, observation in pairs(r.omissions) do
        assert(observation.n == 17 and observation.truncated and #observation.values == 16)
        assert(#observation.values[1].value == 256)
    end
    for _ = 1, 10 do capture() end; assert(calls == 110 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    setup(function() error("manual producer invoked") end, function() error("manual query invoked") end)
    SlashCmdList.APICONTRACTPROBE("all"); assert(calls == 0)
end)
test("producer objects not retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local object = info(4); weak[1] = object; return object end, function() return 1 end)
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(weak[1] == nil)
end)
test("four produced IDs retain exact old matrix before independent omission calls", function()
    local seen = {}
    setup(function()
        return { tasks = { { rewardQuestID = 1.5 }, { rewardQuestID = 2.5 },
            { rewardQuestID = 1.5 }, { rewardQuestID = -4.5 }, { rewardQuestID = 99 } } }
    end, function(...) seen[#seen + 1] = { n = select("#", ...), ... }; return #seen end)
    local r = capture()
    assert(r.omissions, "omission observations absent")
    local ids = { 1.5, 2.5, 1.5, -4.5 }
    assert(calls == 11 and #seen == 10 and #r.entries == 4)
    for i, id in ipairs(ids) do
        local omitted, clamped = seen[2 * i - 1], seen[2 * i]
        assert(omitted.n == 1 and omitted[1] == id)
        assert(clamped.n == 2 and clamped[1] == id and clamped[2] == true)
    end
    assert(seen[9].n == 0)
    assert(seen[10].n == 2 and seen[10][1] == nil and seen[10][2] == false)
    assert(r.omissions.omitted.values[1].value == 9 and r.omissions.unclamped.values[1].value == 10)
end)
test("omissions run independently of unavailable producer and malformed task paths", function()
    for _, state in ipairs({ "missing", "throw", "nil", "secret", "tasks-error", "tasks-invalid" }) do
        local seen = {}
        setup(function()
            if state == "throw" then error(secret) end
            if state == "secret" then return secret end
            if state == "tasks-error" then return setmetatable({}, { __index = function() error(secret) end }) end
            if state == "tasks-invalid" then return { tasks = false } end
        end, function(...) seen[#seen + 1] = { n = select("#", ...), ... }; return 7 end)
        if state == "missing" then C_NeighborhoodInitiative = nil end
        local r = capture()
        assert(r.omissions, "omissions missing after producer failure")
        assert(#seen == 2 and seen[1].n == 0 and seen[2].n == 2)
        assert(seen[2][1] == nil and seen[2][2] == false and next(r.entries) == nil)
        assert(r.omissions.omitted.status == "observed" and r.omissions.unclamped.status == "observed")
    end
end)
test("omission return arity nils and opaque errors remain independent", function()
    setup(function() end, function(...)
        if select("#", ...) == 0 then error(secret) end
        assert(select("#", ...) == 2 and select(1, ...) == nil and select(2, ...) == false)
        return nil, 7, nil
    end)
    local r = capture(); assert(r.omissions, "omission observations absent")
    assert(r.omissions.omitted.status == "call-error")
    assert(r.omissions.unclamped.n == 3 and r.omissions.unclamped.values[3].kind == "nil")
    C_QuestInfoSystem.GetQuestLogRewardFavor = function(...)
        if select("#", ...) == 0 then return end
        error(secret)
    end
    r = capture()
    assert(r.omissions.omitted.n == 0 and r.omissions.unclamped.status == "call-error")
end)
test("omission namespace and function guards preserve unavailable outcomes", function()
    for _, state in ipairs({ "namespace", "lookup", "absent", "function" }) do
        setup(function() end, function() error("unguarded call") end)
        if state == "namespace" then C_QuestInfoSystem = secret
        elseif state == "lookup" then C_QuestInfoSystem = setmetatable({}, { __index = function() error(secret) end })
        elseif state == "absent" then C_QuestInfoSystem = {}
        else C_QuestInfoSystem.GetQuestLogRewardFavor = secret end
        local r = capture(); assert(r.omissions, "omission observations absent")
        local expected = (state == "namespace" or state == "lookup") and "field-error" or "missing-api"
        assert(r.omissions.omitted.status == expected and r.omissions.unclamped.status == expected)
        assert(calls == 1)
    end
end)
test("nil and false revocation after lookup and either function guard blocks forwarding", function()
    for _, argument in ipairs({ "nil", "false" }) do
        for _, phase in ipairs({ "lookup", "secret", "access" }) do
            local revoked, lookups, guards, queries = false, 0, 0, 0
            setup(function() end, function(...)
                queries = queries + 1
                assert(select("#", ...) == 0, "revoked optional argument forwarded")
                return 9
            end)
            local fn = C_QuestInfoSystem.GetQuestLogRewardFavor
            C_QuestInfoSystem = setmetatable({}, { __index = function()
                lookups = lookups + 1
                if phase == "lookup" and lookups == 2 then revoked = true end
                return fn
            end })
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, fn) then
                    guards = guards + 1; if guards == 2 then revoked = true end
                end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "access" and rawequal(v, fn) then
                    guards = guards + 1; if guards == 2 then revoked = true end
                end
                local blocked = (argument == "nil" and v == nil) or (argument == "false" and v == false)
                return not rawequal(v, secret) and not (revoked and blocked)
            end
            local r = capture(); assert(r.omissions, "omission observations absent")
            assert(r.omissions.omitted.status == "observed")
            assert(r.omissions.unclamped.status == "restricted-input" and queries == 1 and calls == 2)
        end
    end
end)
test("initial and preceding output restrictions block only the explicit tuple", function()
    for _, argument in ipairs({ "nil", "false" }) do
        for _, initiallyRestricted in ipairs({ false, true }) do
            local revoked, marker, queries = initiallyRestricted, {}, 0
            setup(function() end, function(...)
                queries = queries + 1; assert(select("#", ...) == 0); return marker
            end)
            canaccessvalue = function(v)
                if rawequal(v, marker) then revoked = true end
                local blocked = (argument == "nil" and v == nil) or (argument == "false" and v == false)
                return not rawequal(v, secret) and not (revoked and blocked)
            end
            local r = capture(); assert(r.omissions, "omission observations absent")
            assert(r.omissions.omitted.status == "observed")
            assert(r.omissions.unclamped.status == "restricted-input" and queries == 1)
        end
    end
end)
test("omission calls use fresh functions and never retain returned objects", function()
    local weak, seen = setmetatable({}, { __mode = "v" }), {}
    setup(function() end, function(...)
        assert(select("#", ...) == 0); seen[1] = true
        local marker = {}; weak[1] = marker
        C_QuestInfoSystem.GetQuestLogRewardFavor = function(...)
            assert(select("#", ...) == 2 and select(1, ...) == nil and select(2, ...) == false)
            seen[2] = true
            local value = newproxy(true); weak[2] = value
            getmetatable(value).__index = function() error("opaque output inspected") end
            return value, secret
        end
        return marker
    end)
    local r = capture(); assert(r.omissions, "omission observations absent")
    assert(seen[1] and seen[2] and r.omissions.unclamped.values[2].status == "restricted")
    collectgarbage("collect"); collectgarbage("collect")
    assert(weak[1] == nil and weak[2] == nil)
end)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
