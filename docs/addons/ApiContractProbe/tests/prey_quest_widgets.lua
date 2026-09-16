local root = assert(arg[1])
local passed = 0
local names = { "Tooltip", "BehindIcon", "AdventureMapDetails" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
local function setup(producer, query)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { MapIconUIWidgetSetType = { Tooltip = 10.5, BehindIcon = 20.5, AdventureMapDetails = 30.5 } }
    C_QuestLog = { GetActivePreyQuest = producer }
    C_TaskQuest = { GetQuestUIWidgetSetByType = query }
    C_UIWidgetManager = setmetatable({}, { __index = function() error("widget followup") end })
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("prey-quest-widgets " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].preyQuestWidgets)
end
local function test(name, fn)
    fn(); passed = passed + 1; print("PASS " .. name)
end

test("original quest and published values exact arguments", function()
    local p, q = 0, 0
    setup(function(...) p = p + 1; assert(select("#", ...) == 0); return 77.25 end,
        function(...) q = q + 1; assert(select("#", ...) == 2); local id, value = ...; assert(id == 77.25 and value == q * 10 + .5); return q end)
    local r = capture(); assert(p == 1 and q == 3 and r.producer.values[1].value == 77.25)
    for i, name in ipairs(names) do assert(r.queries[name].values[1].value == i) end
end)
test("invalid original IDs never forwarded", function()
    for _, id in ipairs({ false, "77", math.huge, 0/0, secret }) do
        setup(function() return id end, function() error("forwarded") end)
        local r = capture(); for _, name in ipairs(names) do assert(r.queries[name].status ~= "observed") end
    end
end)
test("missing and invalid enum has no fallback", function()
    for _, bad in ipairs({ false, "10", math.huge, 0/0, secret }) do
        local calls = 0
        setup(function() return 77 end, function() calls = calls + 1 end)
        Enum.MapIconUIWidgetSetType.Tooltip = bad
        capture(); assert(calls == 2)
    end
    setup(function() return 77 end, function() error("forwarded") end)
    Enum = secret; capture()
end)
test("zero nil and opaque errors preserve peer observations", function()
    local n = 0
    setup(function() return 77, nil, "tail" end, function()
        n = n + 1; if n == 1 then return end; if n == 2 then return nil, "x", nil end; error(secret)
    end)
    local r = capture(); assert(r.producer.n == 3)
    assert(r.queries.Tooltip.n == 0 and r.queries.BehindIcon.n == 3)
    assert(r.queries.AdventureMapDetails.status == "call-error")
end)
test("missing producer and empty producer", function()
    for _, producer in ipairs({ false, function() return end, function() error(secret) end }) do
        setup(producer, function() error("forwarded") end)
        local r = capture(); for _, name in ipairs(names) do assert(r.queries[name].status ~= "observed") end
    end
end)
test("lookup and both function guards revoke either input", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        for _, which in ipairs({ "id", "enum" }) do
            for position = 1, 3 do
                local revoked, calls, lookups = false, 0, 0
                local fn = function() calls = calls + 1 end
                setup(function() return 77 end, fn)
                local target = which == "id" and 77 or position * 10 + .5
                C_TaskQuest = setmetatable({}, { __index = function()
                    lookups = lookups + 1
                    if phase == "lookup" and lookups == position then revoked = true end
                    return fn
                end })
                issecretvalue = function(v)
                    if phase == "secret" and rawequal(v, fn) and lookups == position then revoked = true end
                    return false
                end
                canaccessvalue = function(v)
                    if phase == "access" and rawequal(v, fn) and lookups == position then revoked = true end
                    return not (revoked and rawequal(v, target))
                end
                capture(); assert(calls == (which == "id" and position - 1 or 2))
            end
        end
    end
end)
test("namespace guard revokes quest before use", function()
    local revoked, calls = false, 0
    setup(function() return 77 end, function() calls = calls + 1 end)
    local ns = C_TaskQuest
    canaccessvalue = function(v) if rawequal(v, ns) then revoked = true end; return not (revoked and rawequal(v, 77)) end
    capture(); assert(calls == 0)
end)
test("bounded tuples strings labels snapshots and manual only", function()
    local calls = 0
    setup(function() calls = calls + 1; return 77 end, function()
        calls = calls + 1; return unpack({ string.rep("x", 300), 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17 })
    end)
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(calls == 40 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local c = ApiContractProbeDB.captures[1]
    assert(#c.label == 128 and c.preyQuestWidgets.queries.Tooltip.n == 17)
    assert(#c.preyQuestWidgets.queries.Tooltip.values == 16 and #c.preyQuestWidgets.queries.Tooltip.values[1].value == 256)
    setup(function() error("all producer") end, function() error("all query") end)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(ApiContractProbeDB.captures[1].preyQuestWidgets == nil)
end)
print("Passed " .. passed .. " tests")
