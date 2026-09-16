local root = assert(arg[1])
local passed, forbiddenCalls = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local keys = { "outfitID", "name", "icon", "isEventOutfit", "isDisabled", "playerFacingOutfitIndex", "situationCategories" }
local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded call") end
local function setup(list, query)
    ApiContractProbeDB, SlashCmdList = nil, {}
    forbiddenCalls = 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_TransmogOutfitInfo = { GetOutfitsInfo = list, GetOutfitInfo = query,
        SetActiveOutfit = forbidden, GetOutfitInfoByName = forbidden, CreateOutfit = forbidden }
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("outfit-catalog " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "outfit-catalog mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].outfitCatalog)
end
local function entries(r) return r.values[1].entries end
local function test(name, fn)
    fn(); assert(forbiddenCalls == 0); passed = passed + 1; print("PASS " .. name)
end

test("original IDs fixed fields nested categories and nonrecursive queries", function()
    local calls = 0
    setup(function(...) assert(select("#", ...) == 0); return {
        { outfitID = 12.5, name = string.rep("n", 300), icon = 99, isEventOutfit = false,
          isDisabled = true, playerFacingOutfitIndex = 4, situationCategories = { "raid", "dungeon" } },
        { outfitID = -3.25 },
    } end, function(...)
        calls = calls + 1; assert(select("#", ...) == 1)
        assert((...) == (calls == 1 and 12.5 or -3.25))
        return { outfitID = 999, name = "detail", situationCategories = { "world" } }, nil
    end)
    local r = capture(); local rows = entries(r)
    assert(r.n == 1 and calls == 2 and #rows == 8)
    assert(rows[1].fields.outfitID.value == 12.5 and #rows[1].fields.name.value == 256)
    assert(rows[1].fields.icon.value == 99 and rows[1].fields.isEventOutfit.value == false)
    assert(rows[1].fields.isDisabled.value and rows[1].fields.playerFacingOutfitIndex.value == 4)
    assert(rows[1].fields.situationCategories.entries[2].value == "dungeon")
    assert(rows[1].query.n == 2 and rows[1].query.values[2].kind == "nil")
    assert(rows[1].query.values[1].fields.outfitID.value == 999)
    assert(rows[1].query.values[1].query == nil)
end)

test("missing restricted throwing APIs and invalid IDs stay independent", function()
    setup(nil, nil); assert(capture().status == "missing-api")
    C_TransmogOutfitInfo = secret; assert(capture().status == "field-error")
    C_TransmogOutfitInfo = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().status == "field-error")
    for _, bad in ipairs({ false, "12", math.huge, -math.huge, 0/0, secret }) do
        local calls = 0
        setup(function() return { { outfitID = bad }, { outfitID = 3 } } end,
            function(id) calls = calls + 1; assert(id == 3); return {} end)
        local rows = entries(capture()); assert(calls == 1 and rows[2].query.status == "observed")
        assert(rows[1].query.status ~= "observed")
    end
    setup(function() return { { outfitID = 1 }, { outfitID = 2 } } end, secret)
    assert(entries(capture())[2].query.status == "missing-api")
end)

test("zero returns nil holes and opaque errors preserve arity", function()
    setup(function() end, forbidden); assert(capture().n == 0)
    setup(function() return nil, { { outfitID = 1 } }, nil end, forbidden)
    local r = capture(); assert(r.n == 3 and r.values[1].kind == "nil" and r.values[2].entries == nil)
    setup(function() error(secret) end, forbidden); assert(capture().status == "call-error")
    setup(function() return { { outfitID = 1 }, { outfitID = 2 }, { outfitID = 3 } } end,
        function(id) if id == 1 then error(secret) elseif id == 2 then return nil, false, nil end end)
    local rows = entries(capture())
    assert(rows[1].query.status == "call-error" and rows[2].query.n == 3 and rows[3].query.n == 0)
end)

test("query lookup and function guards revoke each original ID", function()
    for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
        for selected = 1, 8 do
            local revoked, calls = false, 0
            local list = {}; for i = 1, 8 do list[i] = { outfitID = i + 0.5 } end
            local query = function(id) assert(not (revoked and id == selected + 0.5)); calls = calls + 1 end
            setup(function() return list end, query)
            local namespace = C_TransmogOutfitInfo
            namespace.GetOutfitInfo = nil
            setmetatable(namespace, { __index = function(_, key)
                assert(key == "GetOutfitInfo")
                if phase == "lookup" then revoked = true end
                return query
            end })
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, query) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "namespace" and rawequal(v, namespace) then revoked = true end
                if phase == "access" and rawequal(v, query) then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, selected + 0.5))
            end
            capture(); assert(calls == 7)
        end
    end
end)

test("entry field guards prevent every revoked lookup", function()
    for stop = 1, #keys do
        local revoked, reads = false, 0
        local entry
        entry = setmetatable({}, { __index = function(_, key)
            assert(not revoked); reads = reads + 1
            assert(key == keys[reads]); if reads == stop then revoked = true end
            return key == "outfitID" and 1 or "field"
        end })
        setup(function() return { entry, { outfitID = 2 } } end, function(id) assert(id == 1 or id == 2) end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, entry)) end
        local rows = entries(capture()); assert(reads == stop and rows[2].query.status == "observed")
    end
end)

test("list and nested category index guards apply at every position", function()
    for stop = 1, 8 do
        for _, nested in ipairs({ false, true }) do
            local revoked, reads = false, 0
            local object = setmetatable({}, { __index = function(_, key)
                assert(not revoked); reads = reads + 1; assert(key == reads)
                if reads == stop then revoked = true end
                return nested and "category" or { outfitID = key }
            end })
            setup(function() return nested and { { outfitID = 1, situationCategories = object } } or object end,
                function() return {} end)
            canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
            capture(); assert(reads == stop)
        end
    end
end)

test("restricted fields entries and categories never inspected", function()
    setup(function() return { secret, { outfitID = 2, name = secret, situationCategories = secret },
        { outfitID = 3, situationCategories = { secret } } } end, function() return secret end)
    local rows = entries(capture())
    assert(rows[1].status == "restricted" and rows[2].fields.name.status == "restricted")
    assert(rows[2].fields.situationCategories.status == "restricted")
    assert(rows[3].fields.situationCategories.entries[1].status == "restricted")
    assert(rows[2].query.values[1].status == "restricted")
end)

test("detail field and nested revocation guard the same inspector", function()
    local revoked, reads = false, 0
    local detail = setmetatable({}, { __index = function(_, key)
        assert(not revoked); reads = reads + 1; revoked = true; return 5
    end })
    setup(function() return { { outfitID = 1 } } end, function() return detail end)
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, detail)) end
    local r = entries(capture())[1].query
    assert(reads == 1 and r.values[1].fields.name.status == "field-error")
end)

test("field failures and ID revocation during later fields preserve peers", function()
    local revoked = false
    setup(function() return { setmetatable({}, { __index = function(_, key)
        if key == "outfitID" then return 1 end
        if key == "name" then revoked = true; error(secret) end
        return secret
    end }), { outfitID = 2 } } end, function(id) assert(id == 2); return {} end)
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, 1)) end
    local rows = entries(capture())
    assert(rows[1].fields.name.status == "field-error" and rows[1].query.status == "restricted-input")
    assert(rows[2].query.status == "observed")
end)

test("bounds cap queries tuples nested reads and snapshots", function()
    local calls, listCalls = 0, 0
    local function many() return unpack({ 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18 }) end
    setup(function()
        listCalls = listCalls + 1
        local list = {}; for i = 1, 10 do list[i] = { outfitID = i,
            situationCategories = setmetatable({}, { __index = function(_, k) assert(k <= 8); return string.rep("x", 300) end }) } end
        return list, many()
    end, function(id) assert(id <= 8); calls = calls + 1; return many() end)
    local r = capture(string.rep("L", 200))
    assert(r.n == 19 and r.truncated and #r.values == 16 and #entries(r) == 8)
    assert(entries(r)[1].query.n == 18 and entries(r)[1].query.truncated)
    assert(#entries(r)[1].fields.situationCategories.entries[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for i = 2, 11 do capture() end
    assert(listCalls == 10 and calls == 80 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("raw objects collectible and all excludes mode", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function()
        local cats = { "world" }; local entry = { outfitID = 1, situationCategories = cats }; local list = { entry }
        weak[1], weak[2], weak[3] = list, entry, cats; return list
    end, function() local detail = { outfitID = 2 }; weak[4] = detail; return detail end)
    capture(); collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
    setup(forbidden, forbidden)
    SlashCmdList.APICONTRACTPROBE("all excluded")
    assert(ApiContractProbeDB.captures[1].outfitCatalog == nil)
end)

print(string.format("%d outfit-catalog fixtures passed", passed))
