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
        GetRecipeSchematic = function(...)
            calls = calls + 1
            assert(select("#", ...) == 3 and select(2, ...) == false and select(3, ...) == nil)
            return query(...)
        end,
    }
    local function forbidden() error("excluded API invoked") end
    for _, name in ipairs({ "CraftRecipe", "GetRecipeItemQualityInfo", "GetRecipeQualityReagentLink", "SetRecipeTracked", "OpenTradeSkill" }) do
        C_TradeSkillUI[name] = forbidden
    end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("crafting-schematic-read " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "crafting-schematic-read mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].craftingSchematicRead)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function fixture()
    local reagent = { itemID = 17, currencyID = 8 }
    return { recipeID = 4, productQuality = 2, reagentSlotSchematics = {
        { quantityRequired = 3, dataSlotIndex = 2, slotIndex = 1, reagentType = 7,
          reagents = { reagent }, variableQuantities = { { quantity = 9, reagent = reagent } } },
    } }
end
local function first(r) return r.entries[1].schematic.values[1] end
local function slot(r) return first(r).fields.reagentSlotSchematics.entries[1] end

test("original IDs and fixed nested fields preserve exact arguments", function()
    local ids = { 2.5, 2.5, -7.5, 0, 99 }
    local seen = {}
    setup(function() return ids end, function(id) seen[#seen + 1] = id; return fixture() end)
    local r = capture(); assert(calls == 5 and #r.entries == 4)
    for i = 1, 4 do assert(seen[i] == ids[i]) end
    assert(first(r).fields.recipeID.value == 4 and first(r).fields.productQuality.value == 2)
    local s = slot(r)
    assert(s.fields.quantityRequired.value == 3 and s.fields.dataSlotIndex.value == 2)
    assert(s.fields.slotIndex.value == 1 and s.fields.reagentType.value == 7)
    assert(s.fields.reagents.entries[1].fields.itemID.value == 17)
    local v = s.fields.variableQuantities.entries[1]
    assert(v.fields.quantity.value == 9 and v.fields.reagent.fields.currencyID.value == 8)
end)
test("zero returns nil holes opaque errors and first result only", function()
    setup(function() return { 1, 2, 3, 4 }, nil, 8 end, function(id)
        if id == 1 then return nil, fixture(), nil end
        if id == 2 then error(secret) end
        if id == 3 then return end
        return fixture(), nil, 6, nil
    end)
    local r = capture(); assert(calls == 5 and r.producer.n == 3)
    assert(r.entries[1].schematic.n == 3 and first(r).kind == "nil")
    assert(r.entries[2].schematic.status == "call-error" and r.entries[3].schematic.n == 0)
    assert(r.entries[4].schematic.n == 4 and r.entries[4].schematic.values[4].kind == "nil")
end)
test("unavailable producers and invalid IDs never fabricate inputs", function()
    for _, value in ipairs({ secret, false, 8 }) do
        setup(function() return value, { 1 } end, function() error("unexpected query") end)
        assert(next(capture().entries) == nil and calls == 1)
    end
    setup(function() return { secret, math.huge, "1", -1.5 } end, function(id) assert(id == -1.5); return fixture() end)
    local r = capture(); assert(calls == 2)
    assert(r.entries[1].schematic.status == "restricted-input")
    assert(r.entries[2].schematic.status == "unavailable-input" and r.entries[3].schematic.status == "unavailable-input")
end)
test("all query positions reject ID revocation during lookup and function guards", function()
    for index = 1, 4 do for _, phase in ipairs({ "lookup", "secret", "access" }) do
        local revoked = false
        setup(function() return { 1.5, 2.5, 3.5, 4.5 } end, function(id) assert(not (revoked and id == index + 0.5)); return fixture() end)
        local fn, producer = C_TradeSkillUI.GetRecipeSchematic, C_TradeSkillUI.GetRecipesTracked
        local n = 0
        if phase == "lookup" then C_TradeSkillUI = setmetatable({ GetRecipesTracked = producer }, { __index = function(_, key)
            assert(key == "GetRecipeSchematic"); n = n + 1; if n == index then revoked = true end; return fn
        end }) end
        issecretvalue = function(v)
            if phase == "secret" and rawequal(v, fn) then n = n + 1; if n == index then revoked = true end end
            return rawequal(v, secret)
        end
        canaccessvalue = function(v)
            if phase == "access" and rawequal(v, fn) then n = n + 1; if n == index then revoked = true end end
            return not rawequal(v, secret) and not (revoked and rawequal(v, index + 0.5))
        end
        local r = capture(); assert(calls == 4 and r.entries[index].schematic.status == "restricted-input")
    end end
end)
test("namespace denial throwing lookup and missing query stay explicit", function()
    setup(function() return { 1 } end, function() return fixture() end)
    C_TradeSkillUI.GetRecipeSchematic = nil
    assert(capture().entries[1].schematic.status == "missing-api")
    local producer = C_TradeSkillUI.GetRecipesTracked
    C_TradeSkillUI = setmetatable({ GetRecipesTracked = producer }, { __index = function() error(secret) end })
    assert(capture().entries[1].schematic.status == "field-error")
    C_TradeSkillUI = secret; assert(capture().producer.status == "field-error")
end)
test("receiver access is rechecked at every nested lookup", function()
    for _, level in ipairs({ "root", "slots", "slot", "reagents", "reagent", "variables", "variable", "variableReagent" }) do
        local revoked, reads = false, 0
        local data = fixture()
        local s = data.reagentSlotSchematics[1]
        local targets = { root = data, slots = data.reagentSlotSchematics, slot = s, reagents = s.reagents,
            reagent = s.reagents[1], variables = s.variableQuantities, variable = s.variableQuantities[1],
            variableReagent = { itemID = 29, currencyID = 30 } }
        s.variableQuantities[1].reagent = targets.variableReagent
        local target = targets[level]
        local values = {}; for k, v in pairs(target) do values[k] = v; target[k] = nil end
        setmetatable(target, { __index = function(_, key)
            assert(not revoked, "revoked receiver inspected"); reads = reads + 1; revoked = true; return values[key]
        end })
        setup(function() return { 1 } end, function() return data end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, target)) end
        capture(); assert(reads == 1, level)
    end
end)
test("producer table indices rechecked and prior output can revoke peer ID", function()
    local revoked, reads = false, 0
    local ids = setmetatable({}, { __index = function() assert(not revoked); reads = reads + 1; revoked = true; return 1 end })
    setup(function() return ids end, function() return fixture() end)
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, ids)) end
    capture(); assert(reads == 1 and calls == 2)
    local marker = {}
    revoked = false
    setup(function() return { 1, 2 } end, function() return { recipeID = marker } end)
    canaccessvalue = function(v) if rawequal(v, marker) then revoked = true end; return not rawequal(v, secret) and not (revoked and rawequal(v, 2)) end
    local r = capture(); assert(calls == 2 and r.entries[2].schematic.status == "restricted-input")
end)
test("secret nested fields remain opaque and userdata receivers are guarded", function()
    local data = fixture(); data.productQuality = secret
    data.reagentSlotSchematics[1].reagents[1].itemID = secret
    local proxy = newproxy(true); getmetatable(proxy).__index = data
    setup(function() return { 1 } end, function() return proxy end)
    local r = capture(); assert(first(r).kind == "userdata" and first(r).fields.productQuality.status == "restricted")
    assert(slot(r).fields.reagents.entries[1].fields.itemID.status == "restricted")
end)
test("fixed four bounds and allowed fields do not traverse unrelated structures", function()
    local function bounded(values)
        return setmetatable({}, { __index = function(_, key)
            if type(key) == "number" then assert(key >= 1 and key <= 4) end
            assert(values[key] ~= nil, "unexpected field " .. tostring(key)); return values[key]
        end, __len = function() error("length") end })
    end
    local reagent = bounded({ itemID = 8, currencyID = 9 })
    local variable = bounded({ quantity = 2, reagent = reagent })
    local function list(v) return bounded({ v, v, v, v }) end
    local s = bounded({ quantityRequired = 2, dataSlotIndex = 3, slotIndex = 4, reagentType = 5,
        reagents = list(reagent), variableQuantities = list(variable) })
    local data = bounded({ recipeID = 7, productQuality = 3, reagentSlotSchematics = list(s) })
    setup(function() return { 1, 2, 3, 4, 5 } end, function() return data end)
    local r = capture(); assert(calls == 5 and #r.entries == 4)
    assert(#slot(r).fields.reagents.entries == 4 and #slot(r).fields.variableQuantities.entries == 4)
    assert(slot(r).fields.variableQuantities.entries[4].fields.reagent.fields.currencyID.value == 9)
end)
test("tuple string label and snapshot bounds plus manual exclusion", function()
    setup(function() return { 1, 2, 3, 4, 5 } end, function()
        local data = fixture(); data.productQuality = string.rep("x", 300)
        return data, nil, 3,4,5,6,7,8,9,10,11,12,13,14,15,16,secret
    end)
    local r = capture(string.rep("l", 200))
    assert(r.entries[1].schematic.n == 17 and r.entries[1].schematic.truncated and #r.entries[1].schematic.values == 16)
    assert(#first(r).fields.productQuality.value == 256 and #ApiContractProbeDB.captures[1].label == 128)
    for _ = 1, 10 do capture() end
    assert(calls == 50 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    setup(function() error("manual producer") end, function() error("manual query") end)
    SlashCmdList.APICONTRACTPROBE("all"); assert(calls == 0)
end)
test("raw producer and nested objects are collectible", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local ids = { 1 }; weak[1] = ids; return ids end, function()
        local data = fixture(); weak[2], weak[3], weak[4] = data, data.reagentSlotSchematics[1], data.reagentSlotSchematics[1].variableQuantities[1].reagent
        return data
    end)
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
