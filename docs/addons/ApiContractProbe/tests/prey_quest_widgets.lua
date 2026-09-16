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
local fields = { "shownState", "progressState", "tooltip", "tooltipLoc", "widgetSizeSetting", "textureKit", "frameTextureKit", "hasTimer", "orderIndex", "widgetTag", "inAnimType", "outAnimType", "widgetScale", "layoutDirection", "modelSceneLayer", "scriptedAnimationEffectID" }
local function chain(list, visual)
    setup(function() return 77 end, function() return 91.25 end)
    Enum.UIWidgetVisualizationType = { PreyHuntProgress = 51.5 }
    C_UIWidgetManager = { GetAllWidgetsBySetID = list, GetPreyHuntProgressWidgetVisualizationInfo = visual }
end
test("chain originals discriminator fields and wrong types", function()
    local lists, calls = 0, 0
    chain(function(id) lists = lists + 1; assert(id == 91.25); return { { widgetID = 13.25, widgetType = 51.5 }, { widgetID = 14, widgetType = 31 } } end,
        function(id) calls = calls + 1; assert(id == 13.25); local r = {}; for i, key in ipairs(fields) do r[key] = i end; return r, nil end)
    local r = capture(); assert(r.details and lists == 3 and calls == 3)
    for _, name in ipairs(names) do
        local v = r.details[name].entries[1].visualization
        assert(v.n == 2)
        for i, key in ipairs(fields) do assert(v.fields[key].value == i) end
    end
end)
test("chain lookup and function guards revoke dispatch inputs", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        for _, target in ipairs({ 13.25, 51.5 }) do
            local revoked, calls = false, 0
            local fn = function() calls = calls + 1 end
            chain(function() return { { widgetID = 13.25, widgetType = 51.5 } } end, fn)
            C_UIWidgetManager.GetPreyHuntProgressWidgetVisualizationInfo = nil
            setmetatable(C_UIWidgetManager, { __index = function() if phase == "lookup" then revoked = true end; return fn end })
            issecretvalue = function(v) if phase == "secret" and rawequal(v, fn) then revoked = true end; return false end
            canaccessvalue = function(v) if phase == "access" and rawequal(v, fn) then revoked = true end; return not (revoked and rawequal(v, target)) end
            capture(); assert(calls == 0)
        end
    end
end)
test("chain set ID revoked during list lookup", function()
    local revoked, calls = false, 0
    chain(function() calls = calls + 1 end, function() error("visual") end)
    local fn = C_UIWidgetManager.GetAllWidgetsBySetID
    C_UIWidgetManager.GetAllWidgetsBySetID = nil
    setmetatable(C_UIWidgetManager, { __index = function() revoked = true; return fn end })
    canaccessvalue = function(v) return not (revoked and rawequal(v, 91.25)) end
    capture(); assert(calls == 0)
end)
test("chain visualization receiver checked before every field", function()
    for position = 1, #fields do
        local revoked, reads = false, 0
        local obj = setmetatable({}, { __index = function(_, key) assert(not revoked); reads = reads + 1; if key == fields[position] then revoked = true end; return 4 end })
        chain(function() return { { widgetID = 13, widgetType = 51.5 } } end, function() return obj end)
        canaccessvalue = function(v) return not (revoked and rawequal(v, obj)) end
        capture(); assert(reads == position)
    end
end)
test("chain nil errors independent and first return only", function()
    local calls = 0
    chain(function() return { { widgetID = 1, widgetType = 51.5 }, { widgetID = 2, widgetType = 51.5 }, { widgetID = 3, widgetType = 51.5 } } end,
        function(id) calls = calls + 1; if id == 1 then error(secret) elseif id == 2 then return nil, {} else return end end)
    local r = capture(); assert(calls == 9)
    local e = r.details.Tooltip.entries
    assert(e[1].visualization.status == "call-error" and e[2].visualization.n == 2 and e[3].visualization.n == 0)
end)
test("chain bounded nineteen calls and collectible objects", function()
    local calls, weak = 0, setmetatable({}, { __mode = "v" })
    chain(function() calls = calls + 1; local list = {}; for i = 1, 9 do list[i] = { widgetID = i, widgetType = 51.5 } end; weak[#weak + 1] = list; return list end,
        function() calls = calls + 1; local obj = { tooltip = string.rep("x", 300), widgetTag = secret }; weak[#weak + 1] = obj; return obj end)
    for i = 1, 11 do capture() end
    assert(calls == 150 and #ApiContractProbeDB.captures == 10)
    assert(#ApiContractProbeDB.captures[1].preyQuestWidgets.details.Tooltip.entries == 4)
    assert(#ApiContractProbeDB.captures[1].preyQuestWidgets.details.Tooltip.entries[1].visualization.fields.tooltip.value == 256)
    collectgarbage(); collectgarbage(); assert(next(weak) == nil)
end)
test("chain discriminator revoked by intervening kind guard is not compared", function()
    local armed, revoked, calls = false, false, 0
    chain(function() return { { widgetID = 9, widgetType = 82.25 } } end,
        function() calls = calls + 1 end)
    Enum.UIWidgetVisualizationType = setmetatable({}, { __index = function(_, key)
        assert(key == "PreyHuntProgress")
        armed = true
        return 81.25
    end })
    canaccessvalue = function(value)
        if armed and rawequal(value, 82.25) then revoked = true end
        return not (revoked and rawequal(value, 81.25))
    end
    local row = capture().details.Tooltip.entries[1].visualization
    assert(row.status == "restricted-input", "inaccessible discriminator compared: " .. row.status)
    assert(calls == 0)
end)
test("chain discriminator guard revoking widget ID prevents forwarding", function()
    local armed, revoked, calls = false, false, 0
    local fn = function() calls = calls + 1 end
    chain(function() return { { widgetID = 9, widgetType = 51.5 } } end, fn)
    canaccessvalue = function(value)
        if rawequal(value, fn) then armed = true end
        if armed and rawequal(value, 51.5) then revoked = true end
        return not (revoked and rawequal(value, 9))
    end
    local row = capture().details.Tooltip.entries[1].visualization
    assert(calls == 0, "revoked widget ID forwarded")
    assert(row.status == "restricted-input")
end)
print("Passed " .. passed .. " tests")
