local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local names = { "GetItemCraftedQualityInfo", "GetItemReagentQualityInfo" }
local fields = { "quality", "icon", "iconSmall", "iconInventory", "iconMixed", "iconAppear", "iconDissolve", "barFill", "barBackground", "barBackgroundCap", "barHighlight", "iconChat", "iconQuestObjective" }
local function setup(source, namespace)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    GetInventoryItemLink, C_TradeSkillUI = source, namespace
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("tradeskill-item-quality " .. (label or "sample"))
    local db = assert(ApiContractProbeDB, "manual mode absent")
    return assert(db.captures[#db.captures].tradeskillItemQuality, "manual mode absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("original binary links all slots both queries and thirteen fields", function()
    local sources, queries = 0, 0
    local link = string.rep("L", 400) .. "\0binary"
    local info = {}; for i, field in ipairs(fields) do info[field] = i == 1 and 3 or field end
    local function query(...)
        assert(select("#", ...) == 1 and (...) == link)
        queries = queries + 1; return info, nil, "tail"
    end
    setup(function(...)
        assert(select("#", ...) == 2)
        local unit, slot = ...; sources = sources + 1
        assert(unit == "player" and slot == sources)
        return link, nil, "producer-tail"
    end, { GetItemCraftedQualityInfo = query, GetItemReagentQualityInfo = query })
    local result = capture()
    assert(sources == 19 and queries == 38 and #result.slots == 19)
    for slot, row in ipairs(result.slots) do
        assert(row.slot == slot and row.producer.n == 3 and row.producer.values[2].kind == "nil")
        assert(#row.producer.values[1].value == 256)
        for _, name in ipairs(names) do
            local q = row.queries[name]
            assert(q.n == 3 and q.values[2].kind == "nil")
            for _, field in ipairs(fields) do assert(q.info.fields[field].value == info[field]) end
        end
    end
end)

test("missing invalid secret and error links skip only their slots", function()
    local calls = 0
    local function query() calls = calls + 1; return nil end
    setup(function(_, slot)
        if slot == 1 then return end
        if slot == 2 then return nil end
        if slot == 3 then return 123 end
        if slot == 4 then return secret end
        if slot == 5 then error(secret) end
        return "link"
    end, { GetItemCraftedQualityInfo = query, GetItemReagentQualityInfo = query })
    local r = capture(); assert(calls == 28)
    assert(r.slots[1].producer.n == 0 and r.slots[2].producer.n == 1)
    assert(r.slots[4].queries[names[1]].status == "restricted-input")
    assert(r.slots[5].producer.status == "call-error")
end)

test("query failures remain independent and tuple nils preserved", function()
    local calls = 0
    setup(function() return "link" end, setmetatable({}, { __index = function(_, key)
        if key == names[1] then error(secret) end
        return function() calls = calls + 1; return nil, false, nil end
    end }))
    local r = capture(); assert(calls == 19)
    assert(r.slots[1].queries[names[1]].status == "field-error")
    assert(r.slots[19].queries[names[2]].n == 3)
    setup(function() return "link" end, { GetItemCraftedQualityInfo = function() error(secret) end,
        GetItemReagentQualityInfo = function() return end })
    r = capture(); assert(r.slots[1].queries[names[1]].status == "call-error")
    assert(r.slots[1].queries[names[2]].n == 0)
end)

test("lookup and function guards revoke links before forwarding at every slot", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        for target = 1, 19 do
            local current, revoked, calls = 0, false, 0
            local fn = function() assert(not revoked); calls = calls + 1; return nil end
            setup(function(_, slot) current, revoked = slot, false; return "live" end,
                setmetatable({}, { __index = function()
                    if phase == "lookup" and current == target then revoked = true end
                    return fn
                end }))
            local function guard(v)
                if rawequal(v, fn) and current == target then revoked = true end
                return not (revoked and rawequal(v, "live"))
            end
            if phase == "secret" then issecretvalue = function(v) return not guard(v) end
            elseif phase == "access" then canaccessvalue = guard end
            if phase == "lookup" then canaccessvalue = function(v) return not (revoked and rawequal(v, "live")) end end
            capture(); assert(calls == 36)
        end
    end
end)

test("every object field lookup rechecks receiver", function()
    for target = 1, #fields do
        local revoked, reads = false, 0
        local info = setmetatable({}, { __index = function(_, key)
            assert(not revoked); reads = reads + 1
            if key == fields[target] then revoked = true end
            return key
        end })
        setup(function() revoked = false; return "link" end, {
            GetItemCraftedQualityInfo = function() return info end,
            GetItemReagentQualityInfo = function() return nil end,
        })
        canaccessvalue = function(v) return not (revoked and rawequal(v, info)) end
        local r = capture(); assert(reads == 19 * target)
        if target < #fields then assert(r.slots[1].queries[names[1]].info.fields[fields[target + 1]].status == "field-error") end
    end
end)

test("secret objects fields and first return only remain opaque", function()
    setup(function() return "link" end, {
        GetItemCraftedQualityInfo = function() return secret end,
        GetItemReagentQualityInfo = function() return { quality = secret, icon = "ok" }, secret end,
    })
    local r = capture()
    assert(r.slots[1].queries[names[1]].info.status == "restricted")
    assert(r.slots[1].queries[names[2]].info.fields.quality.status == "restricted")
    assert(r.slots[1].queries[names[2]].info.fields.icon.value == "ok")
end)

test("tuple string label and snapshot bounds", function()
    local sources, queries = 0, 0
    local tuple = {}; for i = 1, 20 do tuple[i] = string.rep("X", 400) end
    local function query() queries = queries + 1; return unpack(tuple) end
    setup(function() sources = sources + 1; return unpack(tuple) end,
        { GetItemCraftedQualityInfo = query, GetItemReagentQualityInfo = query })
    for i = 1, 11 do capture(string.rep("a", 200)) end
    assert(sources == 190 and queries == 380 and ApiContractProbeDB.dropped == 1)
    local record = ApiContractProbeDB.captures[1]; assert(#record.label == 128)
    local row = record.tradeskillItemQuality.slots[1]
    for _, result in ipairs({ row.producer, row.queries[names[1]] }) do
        assert(result.n == 20 and result.truncated and #result.values == 16)
        assert(#result.values[1].value == 256)
    end
end)

test("missing APIs and failed access guards fail closed", function()
    setup(nil, nil); local r = capture(); assert(r.slots[19].producer.status == "missing-api")
    setup(function() error("must not call") end, {})
    canaccessvalue = function() error(secret) end
    r = capture(); assert(r.slots[1].producer.status == "missing-api")
    setup(function() error("must not call") end, {}); issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("tradeskill-item-quality")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)

test("objects collectible and no undeclared traversal", function()
    local weak = setmetatable({}, { __mode = "v" })
    local function query()
        local info = setmetatable({}, { __index = function(_, key)
            local known = false; for _, field in ipairs(fields) do if key == field then known = true end end
            assert(known, "undeclared field"); return "value"
        end, __tostring = function() error("stringify") end })
        weak[#weak + 1] = info; return info
    end
    setup(function() return "link" end, { GetItemCraftedQualityInfo = query, GetItemReagentQualityInfo = query })
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)

test("manual mode excluded from all and crafting calls absent", function()
    local calls = 0
    setup(function() calls = calls + 1; return "link" end, setmetatable({
        GetItemCraftedQualityInfo = function() return nil end,
        GetItemReagentQualityInfo = function() return nil end,
    }, { __index = function() error("excluded API") end }))
    SlashCmdList.APICONTRACTPROBE("all"); assert(calls == 0)
    assert(ApiContractProbeDB.captures[1].tradeskillItemQuality == nil)
    ApiContractProbeDB = nil; capture(); assert(calls == 19)
end)
print(string.format("%d/%d passed", passed, passed + failed))
if failed > 0 then os.exit(1) end
