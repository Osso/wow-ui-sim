local root = assert(arg[1])
local passed = 0
local names = { "Essential", "Utility", "TrackedBuff", "TrackedBar", "GroupBuff", "SpecAgnosticEssential", "SpecAgnosticTracked", "EquipSlotEssential", "EquipSlotTracked" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspected") end
local calls
local function setup()
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { CooldownViewerCategory = {} }
    for i, name in ipairs(names) do Enum.CooldownViewerCategory[name] = i + 0.5 end
    calls = { sets = 0, info = 0, alerts = 0 }
    C_CooldownViewer = {
        GetCooldownViewerCategorySet = function(...) assert(select("#", ...) == 2); local category, flag = ...; assert(flag == false); calls.sets = calls.sets + 1; return { category + 100, category + 100 } end,
        GetCooldownViewerCooldownInfo = function(...) assert(select("#", ...) == 1); local id = ...; calls.info = calls.info + 1; return setmetatable({ cooldownID = id, category = 99 }, { __index = function() error("unrequested field") end }) end,
        GetValidAlertTypes = function(...) assert(select("#", ...) == 1); calls.alerts = calls.alerts + 1; return { 3, 8 } end,
    }
    setmetatable(C_CooldownViewer, { __index = function() error("excluded API") end })
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("cooldown-viewer-read " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "cooldown-viewer-read mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].cooldownViewerRead)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("published categories original duplicate IDs and fixed fields", function()
    setup(); local r = capture()
    assert(calls.sets == 9 and calls.info == 18 and calls.alerts == 18)
    for i, row in ipairs(r.categories) do
        assert(row.name == names[i])
        for index = 1, 2 do
            local item = row.items[index]
            assert(item.info.object.fields.cooldownID.value == i + 100.5)
            assert(item.info.object.fields.category.value == 99)
            assert(item.alerts.entries[2].value == 8)
        end
    end
end)
test("invalid publications never fall back", function()
    for _, v in ipairs({ false, "1", math.huge, 0/0, secret }) do
        setup(); Enum.CooldownViewerCategory.Essential = v; capture(); assert(calls.sets == 8)
    end
    setup(); Enum = secret; capture(); assert(calls.sets == 0)
end)
test("category and ID revocation after lookup and function guards", function()
    for _, api in ipairs({ "GetCooldownViewerCategorySet", "GetCooldownViewerCooldownInfo", "GetValidAlertTypes" }) do
        for _, phase in ipairs({ "lookup", "secret", "access" }) do
            setup(); local revoked = false
            local value = api == "GetCooldownViewerCategorySet" and 1.5 or 101.5
            local fn = C_CooldownViewer[api]
            C_CooldownViewer[api] = nil
            setmetatable(C_CooldownViewer, { __index = function(_, key)
                assert(key == api); if phase == "lookup" then revoked = true end; return fn
            end })
            issecretvalue = function(v) if phase == "secret" and rawequal(v, fn) then revoked = true end; return rawequal(v, secret) end
            canaccessvalue = function(v) if phase == "access" and rawequal(v, fn) then revoked = true end; return not rawequal(v, secret) and not (revoked and rawequal(v, value)) end
            local r = capture()
            if api == "GetCooldownViewerCategorySet" then assert(r.categories[1].producer.status == "restricted-input")
            else assert(r.categories[1].items[1][api == "GetValidAlertTypes" and "alerts" or "info"].status == "restricted-input") end
        end
    end
end)
test("nil holes invalid IDs and peer errors remain independent", function()
    setup()
    C_CooldownViewer.GetCooldownViewerCategorySet = function() return { [1] = 7.5, [3] = 8.5, [4] = secret, [5] = "bad" } end
    C_CooldownViewer.GetCooldownViewerCooldownInfo = function() error(secret) end
    local r = capture(); assert(calls.alerts == 18)
    assert(r.categories[1].items[1].info.status == "call-error")
    assert(r.categories[1].items[2].alerts.status == "unavailable-input")
    assert(r.categories[1].items[4].info.status == "restricted-input")
end)
test("raw arity zero nil errors and scalar truncation", function()
    setup(); C_CooldownViewer.GetCooldownViewerCooldownInfo = function() return nil, string.rep("x", 300), nil end
    C_CooldownViewer.GetValidAlertTypes = function() end
    local r = capture(); local item = r.categories[1].items[1]
    assert(item.info.n == 3 and item.info.values[1].kind == "nil" and #item.info.values[2].value == 256)
    assert(item.alerts.n == 0)
end)
test("list and field receiver revocation prevents subsequent lookup", function()
    setup(); local blocked, reads = false, 0
    local object = setmetatable({}, { __index = function(_, key) reads = reads + 1; assert(key == "cooldownID"); blocked = true; return 7 end })
    C_CooldownViewer.GetCooldownViewerCooldownInfo = function() return object end
    canaccessvalue = function(v) return not rawequal(v, secret) and not (blocked and rawequal(v, object)) end
    local r = capture(); assert(reads == 1); assert(r.categories[1].items[1].info.object.fields.category.status == "field-error")
end)
test("bounded tuples lists calls snapshots labels and manual exclusion", function()
    setup()
    C_CooldownViewer.GetCooldownViewerCategorySet = function() calls.sets = calls.sets + 1; return {1,2,3,4,5,6,7,8,9} end
    C_CooldownViewer.GetValidAlertTypes = function() calls.alerts = calls.alerts + 1; return {1,2,3,4,5,6,7,8,9} end
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(calls.sets == 90 and calls.info == 720 and calls.alerts == 720)
    assert(#ApiContractProbeDB.captures == 10 and #ApiContractProbeDB.captures[1].label == 128)
    assert(#ApiContractProbeDB.captures[1].cooldownViewerRead.categories[1].items == 8)
    assert(#ApiContractProbeDB.captures[1].cooldownViewerRead.categories[1].items[1].alerts.entries == 8)
    setup(); SlashCmdList.APICONTRACTPROBE("all"); assert(calls.sets == 0)
end)
test("opaque objects not retained", function()
    setup(); local refs = setmetatable({}, { __mode = "v" })
    C_CooldownViewer.GetCooldownViewerCooldownInfo = function() local o = { cooldownID = newproxy(), category = newproxy() }; refs[#refs + 1] = o; return o end
    capture(); collectgarbage(); collectgarbage(); assert(next(refs) == nil)
end)
print("Passed " .. passed .. " cooldown viewer read tests")
