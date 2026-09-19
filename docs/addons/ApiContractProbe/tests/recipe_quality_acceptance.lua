local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local baseMeta = getmetatable(_G)
local fields = { "quality", "icon", "iconSmall", "iconInventory", "iconMixed", "iconAppear",
    "iconDissolve", "barFill", "barBackground", "barBackgroundCap", "barHighlight", "iconChat", "iconQuestObjective" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted serialization") end
local calls
local function pack(...) return { n = select("#", ...), ... } end
local function eq(actual, expected, message)
    if actual ~= expected then error(message or ("mismatch: " .. type(actual) .. " / " .. type(expected))) end
end
local function forbidden() error("excluded API invoked") end
local function setup(producer, schematic, query)
    setmetatable(_G, baseMeta)
    ApiContractProbeDB, SlashCmdList, calls = nil, {}, { tracked = 0, schematic = 0, query = 0 }
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture", "123" end, function() return 42 end
    C_TradeSkillUI = {
        GetRecipesTracked = function(...)
            calls.tracked = calls.tracked + 1
            eq(select("#", ...), 1); eq((...), false)
            return producer()
        end,
        GetRecipeSchematic = function(...)
            calls.schematic = calls.schematic + 1
            eq(select("#", ...), 3); eq(select(2, ...), false); eq(select(3, ...), nil)
            return schematic(...)
        end,
        GetRecipeItemQualityInfo = function(...)
            calls.query = calls.query + 1
            eq(select("#", ...), 2)
            return query(...)
        end,
        CraftRecipe = forbidden, RecraftRecipe = forbidden, SetRecipeTracked = forbidden,
        GetRecipeQualityReagentLink = forbidden, OpenTradeSkill = forbidden,
    }
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("recipe-quality-acceptance " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "recipe-quality-acceptance mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].recipeQualityAcceptance, "capture absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    setmetatable(_G, baseMeta)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function info()
    local object = {}
    for index, name in ipairs(fields) do object[name] = index == 1 and -3.75 or name end
    return object
end
local function ordinarySetup()
    setup(function() return { 11.5, 22.5, 33.5, 44.5, 55.5 } end,
        function(id) return { recipeID = 999, productQuality = id + 0.125 } end,
        function() return info() end)
end

test("original same-recipe fractional and negative qualities with thirteen fields", function()
    local ids, qualities = { 11.5, 22.5, 11.5, -4.5, 55 }, { -2.75, 0, 3.125, 99.875 }
    local n, seen = 0, {}
    setup(function() return ids end, function(id)
        n = n + 1; eq(id, ids[n]); return { recipeID = 777, productQuality = qualities[n] }
    end, function(id, quality)
        seen[#seen + 1] = { id, quality }; return info(), nil, false, nil
    end)
    local r = capture(); eq(calls.tracked, 1); eq(calls.schematic, 4); eq(calls.query, 4)
    eq(#r.entries, 4)
    for i = 1, 4 do
        eq(seen[i][1], ids[i]); eq(seen[i][2], qualities[i])
        eq(r.entries[i].id.value, ids[i]); eq(r.entries[i].productQuality.value, qualities[i])
        local q = r.entries[i].query
        eq(q.n, 4); eq(q.values[2].kind, "nil"); eq(q.values[4].kind, "nil")
        for index, name in ipairs(fields) do eq(q.info.fields[name].value, index == 1 and -3.75 or name) end
    end
end)

test("nullable invalid nonfinite and restricted quality suppress only dependent query", function()
    for _, case in ipairs({ {}, { false }, { "2" }, { math.huge }, { -math.huge }, { 0/0 }, { secret } }) do
        local n = 0
        setup(function() return { 11.5, 22.5 } end, function(id)
            return { productQuality = id == 11.5 and case[1] or 8.25 }
        end, function(id, value)
            eq(id, 22.5); eq(value, 8.25); n = n + 1; return nil
        end)
        -- Preserve false/nil rather than using a truthiness-based producer fallback.
        C_TradeSkillUI.GetRecipeSchematic = function(id, recraft, level)
            calls.schematic = calls.schematic + 1; eq(recraft, false); eq(level, nil)
            if id == 11.5 then return { productQuality = case[1] } end
            return { productQuality = 8.25 }
        end
        local r = capture(); eq(n, 1); eq(calls.query, 1)
        assert(r.entries[1].query.status ~= "observed")
    end
end)

test("producer and schematic first result only preserve zero nil holes and opaque errors", function()
    setup(function() return { 1, 2, 3, 4 }, nil, false, nil end, function(id)
        if id == 1 then return nil, { productQuality = 8 } end
        if id == 2 then error(secret) end
        if id == 3 then return end
        return { productQuality = 2.125 }, nil, false, nil
    end, function() return nil, info(), nil end)
    local r = capture(); eq(r.producer.n, 4); eq(calls.query, 1)
    eq(r.entries[1].schematic.n, 2); eq(r.entries[2].schematic.status, "call-error")
    eq(r.entries[3].schematic.n, 0); eq(r.entries[4].schematic.n, 4)
    eq(r.entries[4].query.n, 3); eq(r.entries[4].query.info.kind, "nil")
    setup(function() return nil, { 1 } end, function() error("not called") end, forbidden)
    eq(next(capture().entries), nil); eq(calls.schematic, 0)
end)

test("missing throwing and restricted APIs and guards remain explicit", function()
    for _, name in ipairs({ "GetRecipesTracked", "GetRecipeSchematic", "GetRecipeItemQualityInfo" }) do
        ordinarySetup(); C_TradeSkillUI[name] = nil
        local r = capture()
        if name == "GetRecipesTracked" then eq(r.producer.status, "missing-api")
        elseif name == "GetRecipeSchematic" then eq(r.entries[1].schematic.status, "missing-api")
        else eq(r.entries[1].query.status, "missing-api") end
    end
    ordinarySetup(); C_TradeSkillUI = secret; eq(capture().producer.status, "field-error")
    ordinarySetup(); C_TradeSkillUI.GetRecipeItemQualityInfo = nil
    setmetatable(C_TradeSkillUI, { __index = function() error(secret) end })
    eq(capture().entries[1].query.status, "field-error")
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        ordinarySetup(); _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("recipe-quality-acceptance")
        eq(ApiContractProbeDB.captures[1].status, "missing-access-api")
        eq(calls.tracked + calls.schematic + calls.query, 0)
        ordinarySetup(); _G[guard] = function() error(secret) end
        capture(); eq(calls.tracked + calls.schematic + calls.query, 0)
    end
end)

test("all four quality queries reauthorize original ID quality and both ancestors", function()
    for index = 1, 4 do
        for _, victimName in ipairs({ "id", "quality", "ids", "schematic" }) do
            for _, phase in ipairs({ "namespace-secret", "namespace-access", "lookup", "secret", "access" }) do
                local ids, roots = { 11.5, 22.5, 33.5, 44.5 }, {}
                local current, revoked = 0, false
                for i, id in ipairs(ids) do roots[i] = { productQuality = id + 0.125 } end
                local victims = { id = ids[index], quality = roots[index].productQuality, ids = ids, schematic = roots[index] }
                local victim = victims[victimName]
                setup(function() return ids end, function(id)
                    current = current + 1; return roots[current]
                end, function(id, quality)
                    assert(not revoked or current ~= index, "revoked source forwarded")
                    eq(id, ids[current]); eq(quality, roots[current].productQuality); return info()
                end)
                local ns, fn = C_TradeSkillUI, C_TradeSkillUI.GetRecipeItemQualityInfo
                rawset(ns, "GetRecipeItemQualityInfo", nil)
                setmetatable(ns, { __index = function(_, key)
                    assert(key == "GetRecipeItemQualityInfo")
                    if phase == "lookup" and current == index then revoked = true end
                    return fn
                end })
                local function check(v, guardPhase)
                    if current == index then
                        if phase == guardPhase and rawequal(v, fn) then revoked = true end
                        if phase == "namespace-" .. guardPhase and rawequal(v, ns) then revoked = true end
                    end
                    return not rawequal(v, secret) and not (revoked and rawequal(v, victim))
                end
                issecretvalue = function(v) return not check(v, "secret") end
                canaccessvalue = function(v) return check(v, "access") end
                local r = capture(); assert(r.entries[index].query.status ~= "observed")
                assert(calls.query <= 3, victimName .. " " .. phase)
            end
        end
    end
end)

test("schematic calls reauthorize list ID and explicit literal arguments", function()
    for index = 1, 4 do for _, victimName in ipairs({ "id", "ids", "false", "nil" }) do
        for _, phase in ipairs({ "lookup", "secret", "access" }) do
            local ids = { 11.5, 22.5, 33.5, 44.5 }
            local revoked, lookups, callsAfterRevocation = false, 0, 0
            local victim = victimName == "ids" and ids or ids[index]
            if victimName == "false" then victim = false elseif victimName == "nil" then victim = nil end
            setup(function() return ids end, function()
                if revoked then callsAfterRevocation = callsAfterRevocation + 1 end
                return { productQuality = 3.25 }
            end, function() return info() end)
            local ns, fn = C_TradeSkillUI, C_TradeSkillUI.GetRecipeSchematic
            rawset(ns, "GetRecipeSchematic", nil)
            setmetatable(ns, { __index = function(_, key)
                assert(key == "GetRecipeSchematic"); lookups = lookups + 1
                if phase == "lookup" and lookups == index then revoked = true end
                return fn
            end })
            local function check(v, p)
                if phase == p and lookups == index and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, victim))
            end
            issecretvalue = function(v) return not check(v, "secret") end
            canaccessvalue = function(v) return check(v, "access") end
            local r = capture(); assert(r.entries[index].schematic.status ~= "observed")
            if victimName ~= "id" then eq(callsAfterRevocation, 0) end
        end
    end end
end)

test("list and schematic receivers rechecked after prior reads and serialization", function()
    local revoked, reads = false, 0
    local ids = setmetatable({}, { __index = function(_, i)
        assert(not revoked); reads = reads + 1; revoked = true; return 11.5
    end })
    setup(function() return ids end, function() error("revoked ancestor forwarded") end, forbidden)
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, ids)) end
    capture(); eq(reads, 1); eq(calls.schematic, 0)
    for _, victimName in ipairs({ "id", "schematic" }) do
        revoked = false
        local object = setmetatable({}, { __index = function(_, key)
            eq(key, "productQuality"); revoked = true; return 3.125
        end })
        setup(function() return { 11.5 } end, function() return object end, forbidden)
        local victim = victimName == "id" and 11.5 or object
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, victim)) end
        capture(); eq(calls.query, 0)
    end
    revoked = false
    local object, armed = { productQuality = 9.125 }, false
    setup(function() return { 11.5 } end, function() armed = true; return object end, forbidden)
    canaccessvalue = function(v)
        if armed and rawequal(v, object) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, object))
    end
    capture(); eq(calls.query, 0)
end)

test("bounded peer guards reject staged cross-input revocation", function()
    for _, triggerName in ipairs({ "id", "quality", "ids", "schematic" }) do
        for _, victimName in ipairs({ "id", "quality", "ids", "schematic" }) do
            if triggerName ~= victimName then
                for _, phase in ipairs({ "secret", "access" }) do
                    local ids, object = { 11.5 }, { productQuality = 3.125 }
                    local values = { id = 11.5, quality = 3.125, ids = ids, schematic = object }
                    local armed, fired = false, false
                    setup(function() return ids end, function() return object end, forbidden)
                    local fn = C_TradeSkillUI.GetRecipeItemQualityInfo
                    local function check(v, p)
                        if rawequal(v, fn) then armed = true end
                        if armed and phase == p and rawequal(v, values[triggerName]) then fired = true end
                        return not rawequal(v, secret) and not (fired and rawequal(v, values[victimName]))
                    end
                    issecretvalue = function(v) return not check(v, "secret") end
                    canaccessvalue = function(v) return check(v, "access") end
                    local r = capture(); assert(fired); eq(calls.query, 0)
                    eq(r.entries[1].query.status, "restricted-input")
                end
            end
        end
    end
end)

test("every quality field read reauthorizes table and userdata receivers", function()
    for _, userdata in ipairs({ false, true }) do for target = 1, #fields do
        local revoked, reads = false, 0
        local function index(_, key)
            assert(not revoked, "revoked quality info inspected")
            reads = reads + 1; eq(key, fields[reads])
            if reads == target then revoked = true end
            return key
        end
        local object
        if userdata then object = newproxy(true); getmetatable(object).__index = index
        else object = setmetatable({}, { __index = index }) end
        setup(function() return { 11.5 } end, function() return { productQuality = 3.125 } end,
            function() return object end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
        local q = capture().entries[1].query
        eq(reads, target)
        if target < #fields then eq(q.info.fields[fields[target + 1]].status, "field-error") end
    end end
end)

test("only original first schematic quality is read and secret result fields remain opaque", function()
    local reads = 0
    local object = newproxy(true)
    getmetatable(object).__index = function(_, key)
        eq(key, "productQuality"); reads = reads + 1; return 7.75
    end
    local result = info(); result.icon = secret
    setup(function() return { 11.5 } end, function() return object, { productQuality = 9 } end,
        function(id, quality) eq(id, 11.5); eq(quality, 7.75); return result, secret end)
    local q = capture().entries[1].query
    eq(reads, 1); eq(q.info.fields.icon.status, "restricted"); eq(q.values[2].status, "restricted")
    setup(function() return { 11.5 } end, function() return secret, { productQuality = 9 } end, forbidden)
    capture(); eq(calls.query, 0)
end)

test("independent query outcomes preserve source pairs and opaque outputs", function()
    local n = 0
    setup(function() return { 11.5, 22.5, 33.5, 44.5 } end,
        function(id) return { productQuality = id / 10 } end,
        function(id, quality)
            eq(quality, id / 10); n = n + 1
            if n == 1 then error(secret) end
            if n == 2 then return end
            if n == 3 then return nil, info(), nil end
            return secret
        end)
    local r = capture(); eq(calls.query, 4)
    eq(r.entries[1].query.status, "call-error"); eq(r.entries[2].query.n, 0)
    eq(r.entries[3].query.info.kind, "nil"); eq(r.entries[4].query.info.status, "restricted")
end)

test("four entries sixteen returns and byte label snapshot bounds stay local", function()
    local ids = setmetatable({}, { __index = function(_, i) assert(i >= 1 and i <= 4); return i + 0.5 end,
        __len = function() error("list length inspected") end })
    local function many(first) return first, nil, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, secret end
    setup(function() return many(ids) end, function(id) return many({ productQuality = id + 0.25 }) end,
        function() local object = info(); object.icon = string.rep("x", 300); return many(object) end)
    local r = capture(string.rep("l", 200)); eq(#ApiContractProbeDB.captures[1].label, 128)
    eq(r.producer.n, 17); assert(r.producer.truncated); eq(#r.entries, 4)
    for _, row in ipairs(r.entries) do
        eq(row.schematic.n, 17); assert(row.schematic.truncated)
        eq(row.query.n, 17); assert(row.query.truncated); eq(row.query.values[17], nil)
        eq(#row.query.info.fields.icon.value, 256)
    end
    for _ = 1, 10 do capture() end
    eq(#ApiContractProbeDB.captures, 10); eq(ApiContractProbeDB.dropped, 1)
    eq(calls.tracked, 10); eq(calls.schematic, 40); eq(calls.query, 40)
end)

test("all excludes the mode and raw objects are not retained", function()
    ordinarySetup(); SlashCmdList.APICONTRACTPROBE("all")
    eq(calls.tracked + calls.schematic + calls.query, 0)
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local ids = { 11.5 }; weak[1] = ids; return ids end,
        function() local object = { productQuality = 3.125 }; weak[2] = object; return object end,
        function() local object = info(); weak[3] = object; return object end)
    capture(); collectgarbage("collect"); collectgarbage("collect")
    eq(next(weak), nil, "raw objects retained")
end)

local function checkFrozenRevocation(position, triggerName, occurrence, victimName, phase)
    local ids, qualities, roots = { 101.5, 202.5, 303.5, 404.5 }, { -2.125, 3.375, 0, 6.875 }, {}
    for i = 1, 4 do roots[i] = { productQuality = qualities[i] } end
    local values = { id = ids[position], quality = qualities[position], list = ids, schematic = roots[position] }
    local current, hits, forbiddenCalls, successfulPeers = 0, 0, 0, 0
    local armed, revoked = false, false
    setup(function() return ids end, function()
        current = current + 1; return roots[current]
    end, function(id, quality)
        eq(id, ids[current]); eq(quality, qualities[current])
        if current == position and revoked then forbiddenCalls = forbiddenCalls + 1
        else successfulPeers = successfulPeers + 1 end
        return info()
    end)
    local query = C_TradeSkillUI.GetRecipeItemQualityInfo
    local function check(value, guardPhase)
        if current == position and guardPhase == "access" and rawequal(value, query) then armed = true end
        if armed and current == position and guardPhase == phase and rawequal(value, values[triggerName]) then
            hits = hits + 1
            if hits == occurrence then revoked = true end
        end
        return not rawequal(value, secret) and not (revoked and rawequal(value, values[victimName]))
    end
    issecretvalue = function(value) return not check(value, "secret") end
    canaccessvalue = function(value) return check(value, "access") end
    local record = capture()
    local expectedPeers = victimName == "list" and position - 1 or 3
    return revoked and forbiddenCalls == 0 and successfulPeers == expectedPeers
        and record.entries[position].query.status == "restricted-input", forbiddenCalls
end

for _, phase in ipairs({ "secret", "access" }) do
    test("frozen ninth ID " .. phase .. " guard cannot revoke quality or ancestors before forwarding", function()
        local failures, cases, forwarded = 0, 0, 0
        for position = 1, 4 do
            for _, victim in ipairs({ "quality", "list", "schematic" }) do
                local ok, bad = checkFrozenRevocation(position, "id", 9, victim, phase)
                cases, forwarded = cases + 1, forwarded + bad
                if not ok then failures = failures + 1 end
            end
        end
        eq(cases, 12)
        print("DETAIL frozen ninth ID " .. phase .. ": " .. cases .. " cases, " .. forwarded .. " forbidden forwards")
        eq(failures, 0, "frozen boundary failures")
    end)
end

-- Occurrences are frozen to the original authorization passes, never a moving final check.
local originalOccurrences = { id = { 7, 8, 9 }, quality = { 3, 4, 5 }, list = { 2, 3, 4 }, schematic = { 2, 3, 4 } }
for stage = 1, 3 do
    test("bounded final authorization rejects original staged cross-input pass " .. stage, function()
        local failures, cases, forwarded = 0, 0, 0
        for position = 1, 4 do
            for _, trigger in ipairs({ "id", "quality", "list", "schematic" }) do
                for _, victim in ipairs({ "id", "quality", "list", "schematic" }) do
                    if trigger ~= victim then
                        for _, phase in ipairs({ "secret", "access" }) do
                            local ok, bad = checkFrozenRevocation(position, trigger, originalOccurrences[trigger][stage], victim, phase)
                            cases, forwarded = cases + 1, forwarded + bad
                            if not ok then failures = failures + 1 end
                        end
                    end
                end
            end
        end
        eq(cases, 96)
        print("DETAIL staged pass " .. stage .. ": " .. cases .. " cases, " .. forwarded .. " forbidden forwards")
        eq(failures, 0, "staged boundary failures")
    end)
end

print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
