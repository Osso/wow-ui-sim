local root = assert(arg[1], "addon directory required")
local passed, failed, calls = 0, 0, 0
local keys = { "name", "icon", "gameType", "shortDescription", "longDescription", "mapDescription",
    "maxPlayers", "battlegroundID", "lfgDungeonID", "mapID", "isHoliday", "isRandom", "canEnter", "isTrainingGround" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
local function setup(grounds, rewards)
    ApiContractProbeDB, SlashCmdList, calls = nil, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    local function wrap(fn) return function(...) calls = calls + 1; assert(select("#", ...) == 0); return fn() end end
    C_PvP = { GetTrainingGrounds = wrap(grounds), GetRandomTrainingGroundRewards = wrap(rewards) }
    local function forbidden() error("excluded call") end
    C_PvP.JoinTrainingGround, C_PvP.JoinRandomTrainingGround = forbidden, forbidden
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("training-grounds-structures " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "training-grounds-structures mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].trainingGroundsStructures)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("fourteen fields and five reward return positions", function()
    local entry = {}; for i, key in ipairs(keys) do entry[key] = i end
    local opaque = setmetatable({}, { __index = function() error("nested reward read") end })
    setup(function() return { entry } end, function() return 50, 60, opaque, nil, opaque end)
    local r = capture(); assert(calls == 2)
    for i, key in ipairs(keys) do assert(r.grounds.values[1].entries[1].fields[key].value == i) end
    assert(r.rewards.n == 5 and r.rewards.values[1].value == 50 and r.rewards.values[2].value == 60)
    assert(r.rewards.values[3].kind == "table" and r.rewards.values[3].fields == nil)
    assert(r.rewards.values[4].kind == "nil" and r.rewards.values[5].kind == "table")
end)
test("zero arity and nil holes remain distinct", function()
    setup(function() return nil, nil, 3 end, function() end)
    local r = capture(); assert(r.grounds.n == 3 and r.grounds.values[1].kind == "nil" and r.rewards.n == 0)
    setup(function() end, function() return nil, 7, nil, nil, nil end)
    r = capture(); assert(r.grounds.n == 0 and r.rewards.n == 5 and r.rewards.values[5].kind == "nil")
end)
test("errors remain opaque and peers continue", function()
    setup(function() error(secret) end, function() return 9 end)
    local r = capture(); assert(r.grounds.status == "call-error" and r.rewards.values[1].value == 9)
    setup(function() return {} end, function() error(secret) end)
    assert(capture().rewards.status == "call-error")
end)
test("namespace and function guards", function()
    for _, bad in ipairs({ false, secret }) do
        setup(function() end, function() end); C_PvP.GetTrainingGrounds = bad
        local r = capture(); assert(r.grounds.status == "missing-api" and r.rewards.status == "observed")
        C_PvP = bad; r = capture(); assert(r.grounds.status == "field-error" and r.rewards.status == "field-error")
    end
    setup(function() end, function() end)
    C_PvP = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().grounds.status == "field-error")
end)
test("secret list entries and fields are not inspected", function()
    setup(function() return { secret, { name = secret } } end, function() return secret end)
    local r = capture(); assert(r.grounds.values[1].entries[1].status == "restricted")
    assert(r.grounds.values[1].entries[2].fields.name.status == "restricted" and r.rewards.values[1].status == "restricted")
end)
test("list receiver checked before every index", function()
    for stop = 1, 8 do
        local revoked, list = false
        list = setmetatable({}, { __index = function(_, i) assert(not revoked); if i == stop then revoked = true end; return {} end })
        setup(function() return list end, function() return 4 end)
        canaccessvalue = function(v) return not (rawequal(v, list) and revoked) end
        local r = capture(); assert(r.rewards.values[1].value == 4)
        if stop < 8 then assert(r.grounds.values[1].entries[stop + 1].status == "field-error") end
    end
end)
test("entry receiver checked before every field", function()
    for stop = 1, 14 do
        local revoked, entry, reads = false, nil, 0
        entry = setmetatable({}, { __index = function() assert(not revoked); reads = reads + 1; if reads == stop then revoked = true end; return 1 end })
        setup(function() return { entry } end, function() end)
        canaccessvalue = function(v) return not (rawequal(v, entry) and revoked) end
        local r = capture(); assert(reads == stop)
        if stop < 14 then assert(r.grounds.values[1].entries[1].fields[keys[stop + 1]].status == "field-error") end
    end
end)
test("tuple serialization can revoke list before indexing", function()
    local list, revoked, marker = {}, false, {}
    setup(function() return list, marker end, function() end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not (rawequal(v, list) and revoked) end
    assert(capture().grounds.values[1].entries[1].status == "field-error")
end)
test("first table only eight entries and sixteen tuple positions", function()
    local list = setmetatable({}, { __index = function(_, i) assert(i <= 8); return { name = string.rep("x", 300) } end })
    local extra = setmetatable({}, { __index = function() error("extra result traversed") end })
    setup(function() return list, extra end, function() return unpack({1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17}) end)
    local r = capture(); assert(#r.grounds.values[1].entries == 8 and #r.grounds.values[1].entries[1].fields.name.value == 256)
    assert(r.grounds.values[2].entries == nil and r.rewards.n == 17 and r.rewards.truncated and r.rewards.values[17] == nil)
end)
test("snapshot label bounds and manual exclusion", function()
    setup(function() return {} end, function() end)
    for i = 1, 11 do capture(string.rep("x", 150)) end
    assert(calls == 20 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    setup(function() error("manual only") end, function() error("manual only") end)
    SlashCmdList.APICONTRACTPROBE("all"); assert(calls == 0)
end)
test("raw objects are not retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local entry = {}; weak[1] = entry; return {entry} end,
        function() local obj = newproxy(true); weak[2] = obj; return 1, 2, obj end)
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(weak[1] == nil and weak[2] == nil)
end)
print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
