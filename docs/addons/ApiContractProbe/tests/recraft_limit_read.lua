local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local baseMeta = getmetatable(_G)
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspected") end
getmetatable(secret).__tostring = function() error("secret serialized") end
local function pack(...) return { n = select("#", ...), ... } end
local function setup()
    setmetatable(_G, baseMeta)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 1 end
    local f = { calls = {}, queries = {}, ids = { 2.5, -3.5, 99 }, schematics = {}, objects = {} }
    for r = 1, 2 do
        local schematic = { reagentSlotSchematics = {} }
        f.schematics[f.ids[r]] = schematic
        for s = 1, 2 do
            local slot = { reagents = {} }
            schematic.reagentSlotSchematics[s] = slot
            for i = 1, 2 do
                local reagent = { itemID = 100 * r + 10 * s + i, currencyID = nil }
                slot.reagents[i] = reagent
                f.objects[#f.objects + 1] = reagent
            end
        end
    end
    C_TradeSkillUI = {
        GetRecipesTracked = function(...)
            local a = pack(...); assert(a.n == 1 and a[1] == false)
            f.calls[#f.calls + 1] = "tracked"
            return f.ids
        end,
        GetRecipeSchematic = function(...)
            local a = pack(...); assert(a.n == 3 and a[2] == false and a[3] == nil)
            f.calls[#f.calls + 1] = "schematic"
            return f.schematics[a[1]]
        end,
        RecraftLimitCategoryValid = function(...)
            local a = pack(...); assert(a.n == 1)
            f.calls[#f.calls + 1] = "query"
            f.queries[#f.queries + 1] = a[1]
            return true, nil, false
        end,
    }
    local function forbidden() error("excluded operation") end
    for _, key in ipairs({ "CraftRecipe", "RecraftRecipe", "CraftEnchant", "GetRecraftRemovalWarnings",
        "IsRecraftReagentValid", "GetEnchantItems", "SetRecipeTracked", "OpenTradeSkill" }) do
        C_TradeSkillUI[key] = forbidden
    end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
    return f
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("recraft-limit-read " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "recraft-limit-read mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].recraftLimitRead)
end
local function row(result, r, s, i)
    return result.entries[r or 1].slots[s or 1].reagents[i or 1]
end
local function test(name, fn)
    local ok, err = pcall(fn)
    setmetatable(_G, baseMeta)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("actual TOC forwards eight original reagents with exact recipe arguments", function()
    local f = setup()
    local r = capture()
    assert(#f.calls == 11 and #f.queries == 8)
    for n = 1, 8 do assert(rawequal(f.queries[n], f.objects[n])) end
    assert(r.producer.n == 1 and #r.entries == 2)
    assert(row(r).fields.itemID.value == 111 and row(r).fields.currencyID.kind == "nil")
    assert(row(r).result.n == 3 and row(r).result.values[2].kind == "nil")
end)
test("optional fields accept nil and original userdata without rebuilding", function()
    local f = setup()
    local data, lookups = { itemID = nil, currencyID = -2.5 }, 0
    local reagent = newproxy(true)
    getmetatable(reagent).__index = function(_, key)
        assert(key == "itemID" or key == "currencyID"); lookups = lookups + 1; return data[key]
    end
    getmetatable(reagent).__tostring = function() error("reagent stringified") end
    f.schematics[2.5].reagentSlotSchematics[1].reagents[1] = reagent
    f.objects[2].itemID = nil
    local r = capture()
    assert(rawequal(f.queries[1], reagent) and #f.queries == 8 and lookups > 0)
    assert(row(r).fields.itemID.kind == "nil" and row(r).fields.currencyID.value == -2.5)
    assert(row(r, 1, 1, 2).result.status == "observed")
end)
test("invalid or inaccessible declared fields block only their original reagent", function()
    for _, field in ipairs({ "itemID", "currencyID" }) do
        for _, bad in ipairs({ secret, "12", {}, false, math.huge, -math.huge, 0/0 }) do
            local f = setup(); f.objects[1][field] = bad
            local r = capture()
            assert(#f.queries == 7 and row(r).result.status ~= "observed")
            for _, obj in ipairs(f.queries) do assert(not rawequal(obj, f.objects[1])) end
        end
    end
end)
test("all eight query positions reauthorize ancestry and fields after lookup and function guards", function()
    for n = 1, 8 do
        for _, phase in ipairs({ "namespace-secret", "namespace-access", "lookup", "secret", "access" }) do
            for _, victim in ipairs({ "ids", "id", "schematic", "slots", "slot", "reagents", "reagent", "itemID", "currencyID" }) do
                local f = setup()
                local r = math.floor((n - 1) / 4) + 1
                local s = math.floor(((n - 1) % 4) / 2) + 1
                local schematic = f.schematics[f.ids[r]]
                local slot = schematic.reagentSlotSchematics[s]
                local reagent = f.objects[n]
                reagent.currencyID = 9000 + n
                local targets = { ids = f.ids, id = f.ids[r], schematic = schematic,
                    slots = schematic.reagentSlotSchematics, slot = slot, reagents = slot.reagents,
                    reagent = reagent, itemID = reagent.itemID, currencyID = reagent.currencyID }
                local target, revoked, queryChecks, namespaceChecks = targets[victim], false, 0, 0
                local ns, fn = C_TradeSkillUI, C_TradeSkillUI.RecraftLimitCategoryValid
                local forbiddenForwards, affected = 0, {}
                for rr = 1, 2 do
                    local sc = f.schematics[f.ids[rr]]
                    for ss = 1, 2 do for ii = 1, 2 do
                        local sl = sc.reagentSlotSchematics[ss]
                        local obj = sl.reagents[ii]
                        local ancestry = { f.ids, f.ids[rr], sc, sc.reagentSlotSchematics, sl, sl.reagents, obj }
                        for _, ancestor in ipairs(ancestry) do
                            if rawequal(ancestor, target) then affected[obj] = true end
                        end
                        if rawequal(obj.itemID, target) or rawequal(obj.currencyID, target) then affected[obj] = true end
                    end end
                end
                local wrapped = function(obj, ...)
                    if revoked and affected[obj] then forbiddenForwards = forbiddenForwards + 1 end
                    return fn(obj, ...)
                end
                ns.RecraftLimitCategoryValid = wrapped
                if phase == "lookup" then
                    ns.RecraftLimitCategoryValid = nil
                    setmetatable(ns, { __index = function(_, key)
                        assert(key == "RecraftLimitCategoryValid")
                        queryChecks = queryChecks + 1
                        revoked = revoked or queryChecks == n
                        return wrapped
                    end })
                end
                local function trigger(v, guard)
                    if phase == guard and rawequal(v, wrapped) then
                        queryChecks = queryChecks + 1
                        if queryChecks == n then revoked = true end
                    elseif phase == "namespace-" .. guard and rawequal(v, ns) then
                        namespaceChecks = namespaceChecks + 1
                        -- One producer, one schematic per recipe, and preceding queries.
                        if namespaceChecks == n + r + 1 then revoked = true end
                    end
                end
                issecretvalue = function(v) trigger(v, "secret"); return rawequal(v, secret) end
                canaccessvalue = function(v)
                    trigger(v, "access")
                    return not rawequal(v, secret) and not (revoked and rawequal(v, target))
                end
                -- After revocation, later independent paths may still be usable.
                local result = capture()
                assert(forbiddenForwards == 0, "revoked ancestor/value forwarded: " .. victim .. ":" .. phase)
                assert(row(result, r, s, ((n - 1) % 2) + 1).result.status ~= "observed", victim .. ":" .. phase)
            end
        end
    end
end)
test("lookup guards block revoked recipe inputs and literal arguments", function()
    for _, victim in ipairs({ "ids", "id", "false", "nil" }) do
        local f = setup(); local fn = C_TradeSkillUI.GetRecipeSchematic
        local target = victim == "ids" and f.ids or victim == "id" and f.ids[1] or false
        local revoked = false
        C_TradeSkillUI.GetRecipeSchematic = function(...) assert(not revoked, "revoked recipe input"); return fn(...) end
        local wrapped = C_TradeSkillUI.GetRecipeSchematic
        canaccessvalue = function(v)
            if rawequal(v, wrapped) then revoked = true end
            if revoked and ((victim == "nil" and v == nil) or (victim ~= "nil" and rawequal(v, target))) then return false end
            return not rawequal(v, secret)
        end
        local r = capture(); assert(r.entries[1].schematic.status ~= "observed" and #f.queries == 0)
    end
end)
test("every ancestor lookup is checked and later field reads can revoke earlier values", function()
    for _, victim in ipairs({ "ids", "schematic", "slots", "slot", "reagents", "reagent", "itemID" }) do
        local f = setup(); local schematic = f.schematics[2.5]; local slot = schematic.reagentSlotSchematics[1]
        local original = f.objects[1]
        local targets = { ids = f.ids, schematic = schematic, slots = schematic.reagentSlotSchematics,
            slot = slot, reagents = slot.reagents, reagent = original, itemID = original.itemID }
        local revoked = false
        original.currencyID = nil
        setmetatable(original, { __index = function(_, key)
            assert(key == "currencyID"); revoked = true; return nil
        end })
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, targets[victim])) end
        local r = capture()
        assert(row(r).result.status ~= "observed")
        for _, obj in ipairs(f.queries) do assert(not rawequal(obj, original)) end
    end
end)
test("raw arity nil positions and opaque errors do not suppress independent paths", function()
    local f = setup(); local query = C_TradeSkillUI.RecraftLimitCategoryValid
    C_TradeSkillUI.RecraftLimitCategoryValid = function(obj)
        query(obj)
        if rawequal(obj, f.objects[1]) then error(secret) end
        if rawequal(obj, f.objects[2]) then return end
        if rawequal(obj, f.objects[3]) then return nil, false, nil end
        return true
    end
    local r = capture(); assert(#f.queries == 8)
    assert(row(r).result.status == "call-error" and row(r, 1, 1, 2).result.n == 0)
    assert(row(r, 1, 2, 1).result.n == 3 and row(r, 1, 2, 1).result.values[3].kind == "nil")
end)
test("missing APIs invalid first results and throwing fields remain explicit", function()
    for _, bad in ipairs({ false, secret, 3, "bad" }) do
        local f = setup(); C_TradeSkillUI.GetRecipesTracked = function() return bad, f.ids end
        assert(next(capture().entries) == nil and #f.queries == 0)
    end
    local f = setup(); C_TradeSkillUI.RecraftLimitCategoryValid = nil
    assert(row(capture()).result.status == "missing-api")
    f = setup(); setmetatable(f.objects[1], { __index = function() error(secret) end })
    assert(row(capture()).result.status == "field-error" and #f.queries == 7)
    f = setup(); C_TradeSkillUI.GetRecipeSchematic = function() return nil, f.schematics[2.5] end
    assert(next(capture().entries[1].slots) == nil and #f.queries == 0)
    f = setup(); canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("recraft-limit-read")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and #f.calls == 0)
end)
test("nested positions query counts labels and result tuples stay bounded", function()
    local f = setup(); local query = C_TradeSkillUI.RecraftLimitCategoryValid
    C_TradeSkillUI.RecraftLimitCategoryValid = function(obj)
        query(obj); return unpack({ string.rep("x", 300), 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, secret })
    end
    for _, schematic in pairs(f.schematics) do
        setmetatable(schematic.reagentSlotSchematics, { __index = function(_, i) assert(i <= 2) end })
        for _, slot in ipairs(schematic.reagentSlotSchematics) do
            setmetatable(slot.reagents, { __index = function(_, i) assert(i <= 2) end })
        end
    end
    local r = capture(string.rep("L", 200))
    assert(#ApiContractProbeDB.captures[1].label == 128)
    assert(row(r).result.n == 17 and row(r).result.truncated and #row(r).result.values == 16)
    assert(#row(r).result.values[1].value == 256 and row(r).result.values[1].truncated)
    for _ = 2, 11 do capture() end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#f.calls == 110 and #f.queries == 80)
end)
test("raw producer objects are not retained in saved observations", function()
    local f = setup(); local weak = setmetatable({}, { __mode = "v" })
    C_TradeSkillUI.GetRecipesTracked = function() local ids = { 1 }; weak[1] = ids; return ids end
    C_TradeSkillUI.GetRecipeSchematic = function()
        local reagent = { itemID = 1 }
        local reagents = { reagent }; local slot = { reagents = reagents }; local slots = { slot }
        local schematic = { reagentSlotSchematics = slots }
        weak[2], weak[3], weak[4], weak[5], weak[6] = schematic, slots, slot, reagents, reagent
        return schematic
    end
    C_TradeSkillUI.RecraftLimitCategoryValid = function() return true end
    capture()
    collectgarbage(); collectgarbage()
    for i = 1, 6 do assert(weak[i] == nil, "raw object retained") end
end)
test("all excludes the new mode and neighboring crafting modes retain routing", function()
    local f = setup()
    C_TradeSkillUI.RecraftLimitCategoryValid = function() error("recraft query from all") end
    SlashCmdList.APICONTRACTPROBE("all")
    assert(#f.calls == 0)
    SlashCmdList.APICONTRACTPROBE("crafting-schematic-read")
    assert(ApiContractProbeDB.captures[2].craftingSchematicRead and #f.calls > 0)
end)
setmetatable(_G, baseMeta)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
