local root = assert(arg[1])
local passed, forbiddenCalls = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded call") end
local function setup(producer, query)
    ApiContractProbeDB, SlashCmdList = nil, {}
    forbiddenCalls = 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_TransmogOutfitInfo = { GetOutfitsInfo = producer, SetActiveOutfit = forbidden,
        GetOutfitInfo = forbidden, CreateOutfit = forbidden }
    C_TooltipInfo = { GetOutfit = query, GetUnitAuraByAuraInstanceID = forbidden }
    GameTooltip = setmetatable({}, { __index = forbidden })
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("outfit-tooltip " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "outfit-tooltip mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].outfitTooltip)
end
local function rows(r) return r.values[1].entries end
local function test(name, fn)
    local ok, err = pcall(fn)
    if not ok then io.stderr:write("FAIL " .. name .. ": " .. tostring(err) .. "\n"); os.exit(1) end
    assert(forbiddenCalls == 0); passed = passed + 1; print("PASS " .. name)
end

test("original fractional IDs and opaque tooltip tuple", function()
    local n = 0
    local opaque = setmetatable({}, { __index = forbidden, __tostring = forbidden })
    setup(function(...) assert(select("#", ...) == 0); return {
        { outfitID = 12.5 }, { outfitID = -3.25 }, { outfitID = 12.5 }, { outfitID = 0 }, { outfitID = 99 },
    } end, function(...)
        n = n + 1; assert(select("#", ...) == 1)
        assert((...) == ({12.5, -3.25, 12.5, 0})[n]); return opaque, nil, false
    end)
    local r = capture(); assert(n == 4 and r.n == 1 and #rows(r) == 4)
    assert(rows(r)[1].query.n == 3 and rows(r)[1].query.values[1].kind == "table")
    assert(rows(r)[1].query.values[2].kind == "nil" and rows(r)[1].query.values[3].value == false)
end)

test("missing and invalid inputs preserve independent peers", function()
    for _, bad in ipairs({false, "12", math.huge, -math.huge, 0/0, secret}) do
        local calls = 0
        setup(function() return {{outfitID=bad},{outfitID=3}} end,
            function(id) assert(id == 3); calls = calls + 1 end)
        local r = rows(capture()); assert(calls == 1 and r[2].query.n == 0)
    end
    setup(nil, forbidden); assert(capture().status == "missing-api")
    C_TransmogOutfitInfo = secret; assert(capture().status == "field-error")
    setup(function() return {{outfitID=3}} end, nil)
    assert(rows(capture())[1].query.status == "missing-api")
end)

test("zero nil holes errors and only first list", function()
    setup(function() end, forbidden); assert(capture().n == 0)
    setup(function() return nil, {{outfitID=3}}, nil end, forbidden)
    local r = capture(); assert(r.n == 3 and r.values[2].entries == nil)
    setup(function() error(secret) end, forbidden); assert(capture().status == "call-error")
    setup(function() return {{outfitID=1},{outfitID=2},{outfitID=3}} end,
        function(id) if id == 1 then error(secret) elseif id == 2 then return nil, false, nil end end)
    r = rows(capture()); assert(r[1].query.status == "call-error" and r[2].query.n == 3 and r[3].query.n == 0)
end)

test("original ID revoked at every namespace lookup and function guard", function()
    for _, phase in ipairs({"namespace", "lookup", "secret", "access"}) do
        for selected = 1, 4 do
            local revoked, calls = false, 0
            local query = function(id) assert(not (revoked and id == selected + 0.5)); calls = calls + 1 end
            setup(function() return {{outfitID=1.5},{outfitID=2.5},{outfitID=3.5},{outfitID=4.5}} end, query)
            local ns, lookups = C_TooltipInfo, 0
            ns.GetOutfit = nil
            setmetatable(ns, {__index=function(_, key)
                assert(key == "GetOutfit"); lookups = lookups + 1
                if phase == "lookup" and lookups == selected then revoked = true end
                return query
            end})
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, query) and lookups == selected then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "namespace" and rawequal(v, ns) and lookups + 1 == selected then revoked = true end
                if phase == "access" and rawequal(v, query) and lookups == selected then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, selected + 0.5))
            end
            local r = rows(capture()); assert(calls == 3 and r[selected].query.status == "restricted-input")
        end
    end
end)

test("list and entry receiver access rechecked before lookup", function()
    for selected = 1, 4 do
        local revoked, calls = false, 0
        local list = setmetatable({}, {__index=function(_, i)
            assert(not revoked); return {outfitID=i}
        end})
        setup(function() return list end, function(id) calls = calls + 1; if id == selected then revoked = true end end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
        capture(); assert(calls == selected)
    end
    local revoked = false
    local entry = setmetatable({}, {__index=function() assert(not revoked); return 3 end})
    setup(function() return {entry} end, forbidden)
    canaccessvalue = function(v)
        if rawequal(v, entry) then revoked = true; return false end
        return not rawequal(v, secret)
    end
    capture()
end)

test("field serialization revokes original ID before query", function()
    local revoked, calls = false, 0
    setup(function() return {setmetatable({}, {__index=function(_, key)
        assert(key == "outfitID"); revoked = true; return 1.5
    end}), {outfitID=2.5}} end, function(id) assert(id == 2.5); calls = calls + 1 end)
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, 1.5)) end
    local r = rows(capture()); assert(calls == 1 and r[1].query.status == "restricted-input")
end)

test("tuple string label snapshot and call bounds", function()
    local producers, queries = 0, 0
    setup(function() producers = producers + 1; return {{outfitID=1},{outfitID=2},{outfitID=3},{outfitID=4},{outfitID=5}} end,
        function() queries = queries + 1; return unpack({string.rep("x",300),2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17}) end)
    for _ = 1, 12 do capture(string.rep("l",200)) end
    assert(producers == 10 and queries == 40 and ApiContractProbeDB.dropped == 2)
    local r = ApiContractProbeDB.captures[1]; assert(#r.label == 128)
    local q = rows(r.outfitTooltip)[1].query
    assert(q.n == 17 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
end)

test("opaque tooltip objects collectible and no field traversal", function()
    local weak = setmetatable({}, {__mode="v"})
    setup(function() return {{outfitID=1}} end, function()
        local object = newproxy(true); getmetatable(object).__index = forbidden
        getmetatable(object).__tostring = forbidden; weak[1] = object; return object
    end)
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(weak[1] == nil)
end)

test("manual excluded from all and access APIs required", function()
    setup(forbidden, forbidden)
    SlashCmdList.APICONTRACTPROBE("all fixture"); assert(ApiContractProbeDB.captures[1].outfitTooltip == nil)
    setup(forbidden, forbidden); canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("outfit-tooltip fixture")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)
print(string.format("%d passed", passed))
