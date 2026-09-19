local root = assert(arg[1], "addon directory required")
local passed, failed, calls = 0, 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspected") end
getmetatable(secret).__tostring = function() error("secret serialized") end
local function setup(producer, query)
    ApiContractProbeDB, SlashCmdList, calls = nil, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_TradeSkillUI = {
        GetRecipesTracked = function(...)
            calls = calls + 1
            assert(select("#", ...) == 1 and (...) == false)
            return producer()
        end,
        GetEnchantItems = function(...)
            calls = calls + 1
            assert(select("#", ...) == 2 and select(2, ...) == nil)
            return query(...)
        end,
    }
    local function forbidden() error("excluded API invoked") end
    for _, name in ipairs({ "CraftRecipe", "CraftEnchant", "RecraftRecipe", "GetRecraftRemovalWarnings",
        "IsRecraftReagentValid", "RecraftLimitCategoryValid", "SetRecipeTracked", "OpenTradeSkill" }) do
        C_TradeSkillUI[name] = forbidden
    end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("crafting-enchant-items " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "crafting-enchant-items mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].craftingEnchantItems)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function items(r, i) return r.entries[i or 1].items end

test("four original finite IDs and exactly two query arguments", function()
    local ids, seen = { 2.5, 2.5, -7.5, 0, 99 }, {}
    local guid = "original\0GUID\255"
    setup(function() return ids end, function(id) seen[#seen + 1] = id; return { guid } end)
    local r = capture(); assert(calls == 5 and #r.entries == 4)
    for i = 1, 4 do assert(seen[i] == ids[i]); assert(items(r, i).values[1].entries[1].value == guid) end
end)
test("raw tuples distinguish zero returns nil holes errors and wrong first result", function()
    setup(function() return { 1, 2, 3, 4 }, nil, 8 end, function(id)
        if id == 1 then return nil, { "not-first" }, nil end
        if id == 2 then error(secret) end
        if id == 3 then return end
        return { "guid" }, nil, 6, nil
    end)
    local r = capture(); assert(calls == 5 and r.producer.n == 3)
    assert(items(r).n == 3 and items(r).values[1].kind == "nil")
    assert(items(r).values[2].entries == nil)
    assert(items(r, 2).status == "call-error" and items(r, 3).n == 0)
    assert(items(r, 4).n == 4 and items(r, 4).values[4].kind == "nil")
end)
test("invalid inaccessible and absent producer values never fabricate IDs", function()
    for _, value in ipairs({ secret, false, 8, "bad" }) do
        setup(function() return value, { 1 } end, function() error("unexpected query") end)
        assert(next(capture().entries) == nil and calls == 1)
    end
    setup(function() return { secret, math.huge, "1", -1.5 } end, function(id) assert(id == -1.5); return {} end)
    local r = capture(); assert(calls == 2)
    assert(items(r).status == "restricted-input" and items(r, 2).status == "unavailable-input")
    assert(items(r, 3).status == "unavailable-input")
    setup(function() return { 0/0, -math.huge, false } end, function() error("invalid ID") end)
    capture(); assert(calls == 1)
end)
test("each query blocks recipe ID revocation at namespace lookup and function guards", function()
    for index = 1, 4 do for _, phase in ipairs({ "namespace-secret", "namespace-access", "lookup", "secret", "access" }) do
        local revoked, n = false, 0
        setup(function() return { 1.5, 2.5, 3.5, 4.5 } end, function(id)
            assert(not (revoked and id == index + 0.5), "revoked ID forwarded"); return {}
        end)
        local fn, namespace = C_TradeSkillUI.GetEnchantItems, C_TradeSkillUI
        if phase == "lookup" then
            namespace.GetEnchantItems = nil
            setmetatable(namespace, { __index = function(_, key)
                assert(key == "GetEnchantItems"); n = n + 1; if n == index then revoked = true end; return fn
            end })
        end
        local function trigger(v, guard)
            local match = (phase == guard and rawequal(v, fn))
                or (phase == "namespace-" .. guard and rawequal(v, namespace))
            if match then
                n = n + 1
                local position = phase:match("namespace") and index + 1 or index
                if n == position then revoked = true end
            end
        end
        issecretvalue = function(v) trigger(v, "secret"); return rawequal(v, secret) end
        canaccessvalue = function(v)
            trigger(v, "access")
            return not rawequal(v, secret) and not (revoked and rawequal(v, index + 0.5))
        end
        local r = capture(); assert(calls == 4 and items(r, index).status == "restricted-input", phase)
    end end
end)
test("nil reagent argument and false producer argument are guarded after function access", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        local revoked = false
        setup(function() return { 1 } end, function() error("revoked nil forwarded") end)
        local fn, namespace = C_TradeSkillUI.GetEnchantItems, C_TradeSkillUI
        if phase == "lookup" then namespace.GetEnchantItems = nil; setmetatable(namespace, { __index = function() revoked = true; return fn end }) end
        issecretvalue = function(v) if phase == "secret" and rawequal(v, fn) then revoked = true end; return rawequal(v, secret) end
        canaccessvalue = function(v)
            if phase == "access" and rawequal(v, fn) then revoked = true end
            return not rawequal(v, secret) and not (revoked and v == nil)
        end
        assert(items(capture()).status == "restricted-input" and calls == 1)
    end
    setup(function() error("false forwarded") end, function() error("query") end)
    local fn, revoked = C_TradeSkillUI.GetRecipesTracked, false
    canaccessvalue = function(v) if rawequal(v, fn) then revoked = true end; return not (revoked and v == false) end
    assert(capture().producer.status == "restricted-input" and calls == 0)
end)
test("argument checks cannot silently revoke their peer input", function()
    for _, victim in ipairs({ "id", "nil" }) do
        local armed, revoked = false, false
        setup(function() return { 3.5 } end, function() error("revoked argument forwarded") end)
        local fn = C_TradeSkillUI.GetEnchantItems
        canaccessvalue = function(v)
            if rawequal(v, fn) then armed = true end
            if armed and ((victim == "id" and v == nil) or (victim == "nil" and rawequal(v, 3.5))) then revoked = true end
            return not rawequal(v, secret) and not (revoked and ((victim == "id" and rawequal(v, 3.5)) or (victim == "nil" and v == nil)))
        end
        assert(items(capture()).status == "restricted-input" and calls == 1)
    end
end)
test("missing APIs throwing lookups and missing access APIs stay explicit", function()
    setup(function() return { 1 } end, function() return {} end)
    C_TradeSkillUI.GetEnchantItems = nil
    assert(items(capture()).status == "missing-api")
    setmetatable(C_TradeSkillUI, { __index = function() error(secret) end })
    assert(items(capture()).status == "field-error")
    C_TradeSkillUI = secret; assert(capture().producer.status == "field-error")
    setup(function() error("producer") end, function() return {} end)
    assert(capture().producer.status == "call-error" and calls == 1)
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        setup(function() error("unguarded producer") end, function() error("query") end)
        _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("crafting-enchant-items label")
        assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    end
end)
test("each producer and result list index rechecks receiver access", function()
    for _, level in ipairs({ "producer", "result" }) do
        for stop = 1, (level == "producer" and 4 or 8) do
            local revoked, reads = false, 0
            local list = setmetatable({}, { __index = function(_, key)
                assert(not revoked, "revoked list indexed"); reads = reads + 1
                if key == stop then revoked = true end
                return level == "producer" and key or "guid"
            end, __len = function() error("length inspection") end })
            setup(function() return level == "producer" and list or { 1 } end, function() return level == "result" and list or {} end)
            canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
            capture(); assert(reads == stop)
        end
    end
end)
test("result values are scalar opaque and can revoke later IDs or list access", function()
    local opaque = setmetatable({}, { __index = function() error("nested traversal") end, __tostring = function() error("stringification") end })
    setup(function() return { 1 } end, function() return { secret, opaque, false, 0, nil, "guid" } end)
    local entries = items(capture()).values[1].entries
    assert(entries[1].status == "restricted" and entries[2].kind == "table" and entries[2].fields == nil)
    assert(entries[3].value == false and entries[4].value == 0 and entries[5].kind == "nil")
    local marker, revoked = {}, false
    setup(function() return { 1, 2 } end, function() return { marker } end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not (revoked and rawequal(v, 2)) end
    local r = capture(); assert(calls == 2 and items(r, 2).status == "restricted-input")
end)
test("eight result positions sixteen tuple positions and snapshot bounds", function()
    local guid = string.rep("g", 300)
    local list = setmetatable({}, { __index = function(_, key) assert(key >= 1 and key <= 8); return guid end })
    setup(function() return { 1, 2, 3, 4, 5 } end, function()
        return list, nil, 3,4,5,6,7,8,9,10,11,12,13,14,15,16,secret
    end)
    local r = capture(string.rep("l", 200)); local q = items(r)
    assert(calls == 5 and q.n == 17 and q.truncated and #q.values == 16)
    assert(#q.values[1].entries == 8 and #q.values[1].entries[8].value == 256)
    assert(q.values[2].kind == "nil" and #ApiContractProbeDB.captures[1].label == 128)
    for _ = 1, 10 do capture() end
    assert(calls == 50 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    setup(function() error("manual producer") end, function() error("manual query") end)
    SlashCmdList.APICONTRACTPROBE("all"); assert(calls == 0)
end)
test("old schematic interleaving keeps original arguments and observations", function()
    local schematicCalls, enchantCalls = 0, 0
    setup(function() return { 9.5 } end, function(id) enchantCalls = enchantCalls + 1; assert(id == 9.5); return { "guid" } end)
    C_TradeSkillUI.GetRecipeSchematic = function(...)
        schematicCalls = schematicCalls + 1
        assert(select("#", ...) == 3 and (...) == 9.5 and select(2, ...) == false and select(3, ...) == nil)
        return { recipeID = 9.5, productQuality = 2, reagentSlotSchematics = {} }
    end
    SlashCmdList.APICONTRACTPROBE("crafting-schematic-read before")
    capture()
    SlashCmdList.APICONTRACTPROBE("all")
    SlashCmdList.APICONTRACTPROBE("crafting-schematic-read after")
    assert(schematicCalls == 2 and enchantCalls == 1)
    for _, index in ipairs({ 1, 4 }) do
        assert(ApiContractProbeDB.captures[index].craftingSchematicRead.entries[1].schematic.values[1].fields.recipeID.value == 9.5)
    end
end)
test("original producer and result objects are not retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local ids = { 1 }; weak[1] = ids; return ids end, function()
        local object = newproxy(true); getmetatable(object).__index = function() error("opaque traversal") end
        local list = { object }; weak[2], weak[3] = list, object; return list
    end)
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
