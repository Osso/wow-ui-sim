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
    local f = { calls = {}, pairs = {}, ids = { 2.5, -3.5, 99 }, schematics = {}, reagents = {},
        locations = { {}, {} }, guids = { "Item-" .. string.rep("A", 300) .. "\0\255", "Item-B\0\254" } }
    for r = 1, 2 do
        local sc = { reagentSlotSchematics = {} }
        f.schematics[f.ids[r]] = sc
        for s = 1, 2 do
            local slot = { reagents = {} }; sc.reagentSlotSchematics[s] = slot
            for i = 1, 2 do
                local reagent = { itemID = 100 * r + 10 * s + i, currencyID = 9000 + #f.reagents }
                slot.reagents[i] = reagent; f.reagents[#f.reagents + 1] = reagent
            end
        end
    end
    ItemLocation = { CreateFromEquipmentSlot = function(...)
        local a = pack(...); assert(a.n == 2 and rawequal(a[1], ItemLocation))
        assert(a[2] == 1 or a[2] == 2); f.calls[#f.calls + 1] = "location"
        return f.locations[a[2]]
    end }
    C_Item = { GetItemGUID = function(...)
        local a = pack(...); assert(a.n == 1); f.calls[#f.calls + 1] = "guid"
        for i = 1, 2 do if rawequal(a[1], f.locations[i]) then return f.guids[i] end end
        error("not original location")
    end }
    C_TradeSkillUI = {
        GetRecipesTracked = function(...)
            local a = pack(...); assert(a.n == 1 and a[1] == false)
            f.calls[#f.calls + 1] = "tracked"; return f.ids
        end,
        GetRecipeSchematic = function(...)
            local a = pack(...); assert(a.n == 3 and a[2] == false and a[3] == nil)
            f.calls[#f.calls + 1] = "schematic"; return f.schematics[a[1]]
        end,
        IsRecraftReagentValid = function(...)
            local a = pack(...); assert(a.n == 2)
            assert(a[1] == f.guids[1] or a[1] == f.guids[2], "GUID substituted")
            local original = false
            for _, obj in ipairs(f.reagents) do original = original or rawequal(a[2], obj) end
            assert(original, "reagent substituted")
            f.calls[#f.calls + 1] = "query"; f.pairs[#f.pairs + 1] = a
            return true, nil, false
        end,
        RecraftLimitCategoryValid = function(...) assert(select("#", ...) == 1); return true end,
    }
    local function forbidden() error("excluded operation") end
    for _, key in ipairs({ "CraftRecipe", "RecraftRecipe", "CraftEnchant", "GetRecraftRemovalWarnings",
        "GetEnchantItems", "SetRecipeTracked", "OpenTradeSkill" }) do C_TradeSkillUI[key] = forbidden end
    C_AddOns, LoadAddOn, EquipItemByName = { LoadAddOn = forbidden }, forbidden, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
    return f
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("recraft-reagent-read " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "recraft-reagent-read mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].recraftReagentRead)
end
local function row(r, n)
    local recipe = math.floor((n - 1) / 4) + 1
    local slot = math.floor(((n - 1) % 4) / 2) + 1
    return r.entries[recipe].slots[slot].reagents[(n - 1) % 2 + 1]
end
local function test(name, fn)
    local ok, err = pcall(fn); setmetatable(_G, baseMeta)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function ancestry(f, n)
    local r = math.floor((n - 1) / 4) + 1
    local s = math.floor(((n - 1) % 4) / 2) + 1
    local sc = f.schematics[f.ids[r]]; local sl = sc.reagentSlotSchematics[s]
    return { f.ids, f.ids[r], sc, sc.reagentSlotSchematics, sl, sl.reagents, f.reagents[n] }
end

test("actual TOC preserves two equipment GUIDs and all sixteen original pairs", function()
    local f = setup(); local r = capture()
    assert(#f.calls == 23 and #f.pairs == 16 and #r.equipment == 2 and #r.entries == 2)
    for n = 1, 8 do for g = 1, 2 do
        local pair = f.pairs[(n - 1) * 2 + g]
        assert(pair[1] == f.guids[g] and rawequal(pair[2], f.reagents[n]))
        local result = row(r, n).pairs[g].result
        assert(result.status == "observed" and result.n == 3 and result.values[2].kind == "nil")
    end end
    assert(#r.equipment[1].guid.values[1].value == 256, "serialization bound")
end)
test("nil optional reagent fields and original userdata are accepted without rebuilding", function()
    local f = setup(); local values = { itemID = nil, currencyID = -2.5 }
    local reagent = newproxy(true)
    getmetatable(reagent).__index = function(_, key) assert(key == "itemID" or key == "currencyID"); return values[key] end
    getmetatable(reagent).__tostring = function() error("serialized reagent") end
    f.schematics[2.5].reagentSlotSchematics[1].reagents[1] = reagent; f.reagents[1] = reagent
    f.guids[1] = ""; capture(); assert(#f.pairs == 16 and rawequal(f.pairs[1][2], reagent))
end)
test("bad GUIDs and bad reagent fields block only their branch", function()
    for _, bad in ipairs({ secret, false, 23, {}, math.huge }) do
        local f = setup(); f.guids[1] = bad
        local r = capture(); assert(#f.pairs == 8 and row(r, 1).pairs[1].result.status ~= "observed")
    end
    local f = setup(); f.guids[1] = nil; capture(); assert(#f.pairs == 8)
    for _, key in ipairs({ "itemID", "currencyID" }) do
        for _, bad in ipairs({ secret, false, "12", {}, math.huge, 0/0 }) do
            f = setup(); f.reagents[1][key] = bad
            local r = capture(); assert(#f.pairs == 14 and row(r, 1).pairs[1].result.status ~= "observed")
        end
    end
end)
test("equipment failures and tracked failures remain independent", function()
    local f = setup(); ItemLocation.CreateFromEquipmentSlot = function() error("constructor") end
    local r = capture(); assert(#f.pairs == 0 and r.producer.status == "observed" and #r.entries == 2)
    f = setup(); C_TradeSkillUI.GetRecipesTracked = function() error("tracked") end
    r = capture(); assert(#f.calls == 4 and r.equipment[2].guid.status == "observed")
    f = setup(); C_Item.GetItemGUID = function(loc)
        if rawequal(loc, f.locations[1]) then error("GUID") end
        return f.guids[2]
    end
    capture(); assert(#f.pairs == 8)
end)
test("each constructor and GUID call reauthorizes source receiver and slot", function()
    for equip = 1, 2 do for _, stage in ipairs({ "constructor", "guid" }) do
        for _, phase in ipairs({ "lookup", "secret", "access" }) do
            for _, victim in ipairs(stage == "constructor" and { "receiver", "slot" } or { "receiver", "slot", "location" }) do
                local f = setup(); local ns = stage == "constructor" and ItemLocation or C_Item
                local key = stage == "constructor" and "CreateFromEquipmentSlot" or "GetItemGUID"
                local fn, checks, revoked, bad = ns[key], 0, false, 0
                local target = victim == "receiver" and ItemLocation or victim == "slot" and equip or f.locations[equip]
                local wrapped = function(...)
                    local a = pack(...); local selected = stage == "constructor" and a[2] == equip or rawequal(a[1], f.locations[equip])
                    if selected and revoked then bad = bad + 1 end
                    return fn(...)
                end
                ns[key] = wrapped
                if phase == "lookup" then ns[key] = nil; setmetatable(ns, { __index = function(_, k)
                    assert(k == key); checks = checks + 1; if checks == equip then revoked = true end; return wrapped
                end }) end
                local function guard(v, p)
                    if phase == p and rawequal(v, wrapped) then checks = checks + 1; if checks == equip then revoked = true end end
                    return revoked and rawequal(v, target)
                end
                issecretvalue = function(v) return rawequal(v, secret) or guard(v, "secret") end
                canaccessvalue = function(v) return not rawequal(v, secret) and not guard(v, "access") end
                capture(); assert(bad == 0, stage .. ":" .. phase .. ":" .. victim)
            end
        end
    end end
end)
test("all pair positions reauthorize original GUID ancestry reagent ancestry and fields", function()
    for position = 1, 16 do
        for _, phase in ipairs({ "namespace-secret", "namespace-access", "lookup", "secret", "access" }) do
            for victim = 1, 13 do
                local f = setup(); local n = math.floor((position - 1) / 2) + 1; local g = (position - 1) % 2 + 1
                local path = ancestry(f, n)
                local targets = { ItemLocation, g, f.locations[g], f.guids[g], unpack(path) }
                targets[12], targets[13] = f.reagents[n].itemID, f.reagents[n].currencyID
                local target, revoked, checks, bad = targets[victim], false, 0, 0
                local original = C_TradeSkillUI.IsRecraftReagentValid
                local wrapped = function(guid, reagent)
                    if revoked then
                        local nn = 1; while not rawequal(f.reagents[nn], reagent) do nn = nn + 1 end
                        local gg = guid == f.guids[1] and 1 or 2
                        local p = ancestry(f, nn)
                        local own = { ItemLocation, gg, f.locations[gg], guid, unpack(p) }
                        own[#own + 1], own[#own + 2] = reagent.itemID, reagent.currencyID
                        for _, v in ipairs(own) do if rawequal(v, target) then bad = bad + 1; break end end
                    end
                    return original(guid, reagent)
                end
                local ns = C_TradeSkillUI
                ns.IsRecraftReagentValid = nil
                setmetatable(ns, { __index = function(_, key)
                    assert(key == "IsRecraftReagentValid")
                    checks = checks + 1
                    if phase == "lookup" and checks == position then revoked = true end
                    return wrapped
                end })
                local function trigger(v, p)
                    if phase == p and rawequal(v, wrapped) and checks == position then revoked = true end
                    -- Namespace checks immediately preceding each query lookup.
                    if phase == "namespace-" .. p and rawequal(v, ns) and checks == position - 1 then
                        -- Schematics also use this namespace; arm only after the target reagent is inspected.
                        if f.armed then revoked = true end
                    end
                end
                local reagent = f.reagents[n]
                local fields = { itemID = reagent.itemID, currencyID = reagent.currencyID }
                reagent.itemID, reagent.currencyID = nil, nil
                setmetatable(reagent, { __index = function(_, key) f.armed = true; return fields[key] end })
                issecretvalue = function(v) trigger(v, "secret"); return rawequal(v, secret) end
                canaccessvalue = function(v) trigger(v, "access"); return not rawequal(v, secret) and not (revoked and rawequal(v, target)) end
                capture(); assert(revoked and bad == 0, position .. ":" .. phase .. ":" .. victim)
            end
        end
    end
end)
test("late field guards cannot revoke GUID or earlier reagent fields before forwarding", function()
    for phaseIndex, phase in ipairs({ "secret", "access" }) do
        for victimIndex, victim in ipairs({ "guid", "item", "currency", "ancestor" }) do
            for position = 1, 16 do
                local f = setup(); local n = math.floor((position - 1) / 2) + 1; local g = (position - 1) % 2 + 1
                local reagent = f.reagents[n]; local target = victim == "guid" and f.guids[g]
                    or victim == "item" and reagent.itemID or victim == "currency" and reagent.currencyID or f.locations[g]
                local query, checks, armed, revoked, bad = C_TradeSkillUI.IsRecraftReagentValid, 0, false, false, 0
                local wrapped = function(guid, obj)
                    if revoked and guid == f.guids[g] and rawequal(obj, reagent) then bad = bad + 1 end
                    return query(guid, obj)
                end
                C_TradeSkillUI.IsRecraftReagentValid = wrapped
                local function guard(v, p)
                    if p == "access" and rawequal(v, wrapped) then checks = checks + 1; armed = checks == position end
                    if armed and p == phase and rawequal(v, reagent.currencyID) then revoked = true end
                    return revoked and rawequal(v, target)
                end
                issecretvalue = function(v) return rawequal(v, secret) or guard(v, "secret") end
                canaccessvalue = function(v) return not rawequal(v, secret) and not guard(v, "access") end
                capture(); assert(revoked and bad == 0, phaseIndex .. ":" .. victimIndex .. ":" .. position)
            end
        end
    end
end)
test("raw zero nil and opaque errors do not suppress later pair observations", function()
    local f = setup(); local calls = 0
    C_TradeSkillUI.IsRecraftReagentValid = function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil, false, nil end
        if calls == 3 then error(secret) end
        return true
    end
    local r = capture(); assert(calls == 16)
    assert(row(r, 1).pairs[1].result.n == 0 and row(r, 1).pairs[2].result.n == 3)
    assert(row(r, 2).pairs[1].result.status == "call-error" and row(r, 2).pairs[2].result.status == "observed")
end)
test("first returned values and bounded source positions only are inspected", function()
    local f = setup(); local original = C_TradeSkillUI.GetRecipeSchematic
    C_TradeSkillUI.GetRecipeSchematic = function(...) return original(...), secret end
    for _, sc in pairs(f.schematics) do
        sc.reagentSlotSchematics[3] = secret
        for s = 1, 2 do sc.reagentSlotSchematics[s].reagents[3] = secret end
    end
    f.ids[3] = secret; local r = capture(); assert(#f.pairs == 16 and r.entries[1].schematic.n == 2)
    f = setup(); C_TradeSkillUI.GetRecipesTracked = function() return nil, f.ids end
    capture(); assert(#f.pairs == 0)
end)
test("tuple string label and ten-snapshot bounds remain independent", function()
    local f = setup(); C_TradeSkillUI.IsRecraftReagentValid = function()
        local a = {}; for i = 1, 18 do a[i] = string.rep("x", 300) end; return unpack(a)
    end
    local r = capture(string.rep("L", 200)); local q = row(r, 1).pairs[1].result
    assert(q.n == 18 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for i = 2, 11 do SlashCmdList.APICONTRACTPROBE("recraft-reagent-read") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#f.calls == 70, "only seven source calls logged per snapshot")
end)
test("objects are collectible after capture while serialized observations remain", function()
    local f = setup(); local weak = setmetatable({}, { __mode = "v" })
    weak[1], weak[2], weak[3], weak[4] = f.ids, f.schematics[2.5], f.reagents[1], f.locations[1]
    capture(); f = nil; ItemLocation, C_Item, C_TradeSkillUI = nil, nil, nil
    collectgarbage(); collectgarbage()
    for i = 1, 4 do assert(weak[i] == nil, "raw source retained") end
    assert(ApiContractProbeDB.captures[1].recraftReagentRead)
end)
test("all excludes new mode and corrected recraft-limit behavior remains independent", function()
    local f = setup(); SlashCmdList.APICONTRACTPROBE("all"); assert(#f.calls == 0)
    local r = capture(); assert(#f.pairs == 16 and r.equipment)
    local calls = #f.calls
    SlashCmdList.APICONTRACTPROBE("recraft-limit-read")
    assert(ApiContractProbeDB.captures[3].recraftLimitRead and #f.calls == calls + 3)
end)
-- The final currency check in the reported revision is the eleventh after the
-- query function guard. Keep this boundary fixed while testing added checks.
local function finalPairGuardCase(position, phase, scenario, victimName, ancestorIndex)
    local f = setup()
    local n, g = math.floor((position - 1) / 2) + 1, (position - 1) % 2 + 1
    local reagent = f.reagents[n]
    local item, currency = reagent.itemID, reagent.currencyID
    local equipment = { ItemLocation, g, f.locations[g], f.guids[g] }
    local denied, checks, currencyChecks, armed, reached, stage, bad = {}, 0, 0, false, false, 0, 0
    local original = C_TradeSkillUI.IsRecraftReagentValid
    local query = function(guid, obj)
        if rawequal(guid, f.guids[g]) and rawequal(obj, reagent) then
            local forbidden = denied[item] or denied[currency] or denied[guid]
            for _, value in ipairs(equipment) do forbidden = forbidden or denied[value] end
            if forbidden then bad = bad + 1 end
        end
        armed = false
        return original(guid, obj)
    end
    C_TradeSkillUI.IsRecraftReagentValid = query
    local function guard(value, guardPhase)
        if guardPhase == "access" and rawequal(value, query) then
            checks = checks + 1; armed = checks == position
        end
        if not armed or guardPhase ~= phase then return end
        if rawequal(value, currency) then
            currencyChecks = currencyChecks + 1
            if currencyChecks == 11 then
                reached = true
                if scenario == "final-currency" then
                    denied[victimName == "item" and item or f.locations[g]] = true
                    stage = 1
                end
                return
            end
        end
        if not reached then return end
        local field = victimName == "item" and item or currency
        if scenario == "equipment-to-field" and stage == 0 and rawequal(value, equipment[ancestorIndex]) then
            denied[field] = true; stage = 1
        elseif scenario == "field-to-equipment" and stage == 0 and rawequal(value, field) then
            denied[equipment[ancestorIndex]] = true; stage = 1
        elseif scenario == "peer-field" and stage == 0 and rawequal(value, field) then
            denied[victimName == "item" and currency or item] = true; stage = 1
        end
    end
    issecretvalue = function(value)
        guard(value, "secret")
        return rawequal(value, secret) or (phase == "secret" and denied[value] == true)
    end
    canaccessvalue = function(value)
        guard(value, "access")
        return not rawequal(value, secret) and not denied[value]
    end
    local result = capture()
    return reached and stage == 1 and bad == 0 and row(result, n).pairs[g].result.status ~= "observed"
end

for _, phase in ipairs({ "secret", "access" }) do
    test("final currency " .. phase .. " guard cannot revoke item or equipment location before forwarding", function()
        local failures = {}
        for position = 1, 16 do
            for _, victim in ipairs({ "item", "ancestor" }) do
                if not finalPairGuardCase(position, phase, "final-currency", victim) then
                    failures[#failures + 1] = position .. "/" .. victim
                end
            end
        end
        assert(#failures == 0, "final currency forbidden forwards: " .. table.concat(failures, ","))
    end)
end

for _, scenario in ipairs({ "equipment-to-field", "field-to-equipment", "peer-field" }) do
    test("bounded final pair checks reject staged " .. scenario .. " revocation", function()
        local failures = {}
        for position = 1, 16 do
            for _, phase in ipairs({ "secret", "access" }) do
                for _, victim in ipairs({ "item", "currency" }) do
                    for ancestor = 1, scenario == "peer-field" and 1 or 4 do
                        if not finalPairGuardCase(position, phase, scenario, victim, ancestor) then
                            failures[#failures + 1] = position .. "/" .. phase .. "/" .. victim .. "/" .. ancestor
                        end
                    end
                end
            end
        end
        assert(#failures == 0, "staged pair revocation: " .. table.concat(failures, ","))
    end)
end

setmetatable(_G, baseMeta)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
