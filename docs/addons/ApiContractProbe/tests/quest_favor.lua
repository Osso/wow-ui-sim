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
    local r = capture(); assert(calls == 5 and #seen == 4)
    for i, args in ipairs(seen) do assert(args[1] == 2.5 and args.n == (i % 2 == 1 and 1 or 2)); if args.n == 2 then assert(args[2] == true) end end
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
        assert(next(capture().entries) == nil and calls == 1)
    end
    setup(function() error(secret) end, function() end); assert(capture().producer.status == "call-error")
    C_NeighborhoodInitiative = secret; assert(capture().producer.status == "field-error")
end)
test("invalid IDs skip independently", function()
    setup(function() return { tasks = { { rewardQuestID = secret }, { rewardQuestID = math.huge }, { rewardQuestID = "4" }, { rewardQuestID = -4.5 } } } end,
        function(id) assert(id == -4.5); return 2 end)
    local r = capture(); assert(calls == 3 and r.entries[1].omitted.status == "restricted-input")
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
        assert(capture().entries[1].omitted.status == "restricted-input" and calls == 1)
    end
end)
test("receiver guards before each field and index", function()
    local revoked, reads = false, 0
    local tasks = setmetatable({}, { __index = function(_, index) reads = reads + 1; revoked = true; return { rewardQuestID = index } end })
    setup(function() return { tasks = tasks } end, function() return 1 end)
    canaccessvalue = function(v) return not rawequal(v, secret) and not (rawequal(v, tasks) and revoked) end
    capture(); assert(reads == 1 and calls == 3)
    setup(function() return { tasks = { secret } } end, function() error("unexpected") end)
    assert(capture().entries[1].status == "field-error" and calls == 1)
end)
test("output access revokes ID before independent second call", function()
    local revoked, marker = false, {}
    setup(function() return info(8.5) end, function() return marker end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not rawequal(v, secret) and not (revoked and v == 8.5) end
    local r = capture(); assert(calls == 2 and r.entries[1].clamped.status == "restricted-input")
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
    local r = capture(string.rep("l", 200)); assert(calls == 9 and #r.entries == 4)
    assert(r.entries[1].omitted.n == 17 and r.entries[1].omitted.truncated and #r.entries[1].omitted.values == 16)
    assert(#r.entries[1].omitted.values[1].value == 256 and #ApiContractProbeDB.captures[1].label == 128)
    for _ = 1, 10 do capture() end; assert(calls == 90 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    setup(function() error("manual producer invoked") end, function() error("manual query invoked") end)
    SlashCmdList.APICONTRACTPROBE("all"); assert(calls == 0)
end)
test("producer objects not retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local object = info(4); weak[1] = object; return object end, function() return 1 end)
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(weak[1] == nil)
end)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
