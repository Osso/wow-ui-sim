local root = assert(arg[1])
local passed, forbiddenCalls = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local fields = { "slot", "type", "collectionType", "slotName", "isSecondary" }
local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded call") end
local function setup(locations, groups, query)
    ApiContractProbeDB, SlashCmdList = nil, {}
    forbiddenCalls = 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_TransmogOutfitInfo = { GetAllSlotLocationInfo = locations, GetSlotGroupInfo = groups,
        GetEquippedSlotOptionFromTransmogSlot = query, GetUnassignedAtlasForSlot = query,
        SetActiveOutfit = forbidden, CommitPendingTransmog = forbidden }
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("outfit-slots " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "outfit-slots mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].outfitSlots)
end
local function test(name, fn)
    fn(); assert(forbiddenCalls == 0); passed = passed + 1; print("PASS " .. name)
end
local function list(slot) return { { slot = slot, type = 8, collectionType = 9, slotName = "head", isSecondary = false } } end

test("two independent noarg producers original slots exact one-argument queries", function()
    local calls, producers = {}, 0
    setup(function(...) assert(select("#", ...) == 0); producers = producers + 1; return list(3.5), list(-8.25) end,
        function(...) assert(select("#", ...) == 0); producers = producers + 1
            return { { position = 42, appearanceSlotInfo = list(999), illusionSlotInfo = list(888) } } end,
        function(...) assert(select("#", ...) == 1); calls[#calls + 1] = (...); return "atlas", nil end)
    local r = capture()
    assert(producers == 2 and #calls == 4 and calls[1] == 3.5 and calls[2] == 3.5 and calls[3] == -8.25 and calls[4] == -8.25)
    local row = r.locations.values[1].entries[1]
    for _, k in ipairs(fields) do assert(row.fields[k].status == "observed") end
    assert(row.fields.slot.value == 3.5 and row.fields.isSecondary.value == false)
    assert(row.queries.GetUnassignedAtlasForSlot.n == 2)
    local group = r.groups.values[1].entries[1]
    assert(group.fields.position.value == 42)
    assert(group.fields.appearanceSlotInfo.entries[1].fields.slot.value == 999)
    assert(group.fields.illusionSlotInfo.entries[1].queries == nil)
end)

test("nil holes and extra returns do not change schemas", function()
    local calls = 0
    setup(function() return nil, list(2), list(333), nil end,
        function() return nil, { { position = 99 } }, nil end,
        function() calls = calls + 1 end)
    local r = capture()
    assert(r.locations.n == 4 and r.locations.values[1].kind == "nil" and calls == 2)
    assert(r.locations.values[2].entries[1].fields.slot.value == 2)
    assert(r.locations.values[3].entries == nil and r.groups.values[2].entries == nil)
    assert(r.groups.n == 3)
    setup(function() end, function() end, forbidden)
    r = capture(); assert(r.locations.n == 0 and r.groups.n == 0)
end)

test("producer errors and inaccessible APIs preserve independent groups", function()
    setup(function() error(secret) end, function() return { { position = 5 } } end, forbidden)
    local r = capture(); assert(r.locations.status == "call-error" and r.groups.values[1].entries[1].fields.position.value == 5)
    setup(nil, secret, forbidden)
    r = capture(); assert(r.locations.status == "missing-api" and r.groups.status == "missing-api")
    C_TransmogOutfitInfo = secret
    r = capture(); assert(r.locations.status == "field-error" and r.groups.status == "field-error")
end)

test("invalid secret slots and query errors do not suppress peers", function()
    for _, bad in ipairs({ false, "2", math.huge, -math.huge, 0/0, secret }) do
        local calls = 0
        setup(function() return { { slot = bad }, { slot = 4 } } end, function() end,
            function(id) calls = calls + 1; assert(id == 4); return nil, false, nil end)
        local r = capture(); assert(calls == 2)
        assert(r.locations.values[1].entries[2].queries.GetUnassignedAtlasForSlot.n == 3)
    end
    setup(function() return list(2) end, function() end, function() return false end)
    C_TransmogOutfitInfo.GetEquippedSlotOptionFromTransmogSlot = function() error(secret) end
    local row = capture().locations.values[1].entries[1]
    assert(row.queries.GetEquippedSlotOptionFromTransmogSlot.status == "call-error")
    assert(row.queries.GetUnassignedAtlasForSlot.values[1].value == false)
end)

test("every original slot is rechecked after namespace lookup and function guards", function()
    for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
        for selected = 1, 32 do
            local revoked, attempts, calls = false, 0, 0
            local rows = {}; for i = 1, 8 do rows[i] = { slot = i + 0.5 } end
            local query = function(id) assert(not revoked); calls = calls + 1 end
            setup(function() return rows, rows end, function() end, query)
            local ns = C_TransmogOutfitInfo
            ns.GetEquippedSlotOptionFromTransmogSlot, ns.GetUnassignedAtlasForSlot = nil, nil
            setmetatable(ns, { __index = function(_, key)
                assert(key == "GetEquippedSlotOptionFromTransmogSlot" or key == "GetUnassignedAtlasForSlot")
                attempts = attempts + 1
                if phase == "lookup" and attempts == selected then revoked = true end
                return query
            end })
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, query) and attempts == selected then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "namespace" and rawequal(v, ns) and attempts + 1 == selected then revoked = true end
                if phase == "access" and rawequal(v, query) and attempts == selected then revoked = true end
                if type(v) == "number" and v % 1 == 0.5 then return not revoked end
                return true
            end
            capture(); assert(calls == selected - 1)
        end
    end
end)

test("every slot field rechecks its receiver after earlier fields revoke it", function()
    for selected = 1, 5 do
        local revoked, reads = false, 0
        local entry = setmetatable({}, { __index = function(_, key)
            assert(not revoked); reads = reads + 1; assert(key == fields[reads])
            if reads == selected then revoked = true end
            return key == "slot" and 2 or "field"
        end })
        setup(function() return { entry } end, function() end, function() return true end)
        canaccessvalue = function(v) return not (rawequal(v, entry) and revoked) end
        capture(); assert(reads == selected)
    end
end)

test("all list and nested receiver levels are rechecked before indexing", function()
    for _, level in ipairs({ "locations", "groups", "group", "nested" }) do
        local revoked, reads = false, 0
        local guarded = setmetatable({}, { __index = function(_, key)
            assert(not revoked); reads = reads + 1; revoked = true
            if level == "group" then assert(key == "position"); return 3 end
            if level == "groups" then return { position = 4 } end
            return { slot = 4 }
        end })
        setup(function() return level == "locations" and guarded or {} end,
            function()
                if level == "groups" then return guarded end
                if level == "group" then return { guarded } end
                return { { position = 2, appearanceSlotInfo = guarded, illusionSlotInfo = guarded } }
            end, function() return true end)
        canaccessvalue = function(v) return not (rawequal(v, guarded) and revoked) end
        capture(); assert(reads == 1)
    end
end)

test("serialization revocation blocks original slot forwarding", function()
    local revoked = false
    setup(function() return { { slot = 3.5, slotName = "revoke" } } end, function() end, forbidden)
    canaccessvalue = function(v)
        if v == "revoke" then revoked = true end
        if v == 3.5 then return not revoked end
        return true
    end
    local row = capture().locations.values[1].entries[1]
    assert(row.queries.GetUnassignedAtlasForSlot.status == "restricted-input")
end)

test("bounds cap producers queries nested entries tuple strings labels and captures", function()
    local producers, calls, nestedReads = 0, 0, 0
    local rows = {}; for i = 1, 9 do rows[i] = { slot = i, slotName = string.rep("x", 300) } end
    local nested = setmetatable({}, { __index = function(_, i) assert(i <= 8); nestedReads = nestedReads + 1; return rows[i] end })
    local groups = {}; for i = 1, 9 do groups[i] = { position = i, appearanceSlotInfo = nested, illusionSlotInfo = nested } end
    local many = {}; for i = 1, 20 do many[i] = "value" end
    setup(function() producers = producers + 1; return rows, rows, unpack(many) end,
        function() producers = producers + 1; return groups, unpack(many) end,
        function() calls = calls + 1; return unpack(many) end)
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(producers == 20 and calls == 320 and nestedReads == 1280)
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local record = ApiContractProbeDB.captures[1]; local r = record.outfitSlots
    assert(#record.label == 128 and r.locations.n == 22 and #r.locations.values == 16 and r.locations.truncated)
    assert(#r.locations.values[1].entries == 8 and #r.groups.values[1].entries == 8)
    local row = r.locations.values[1].entries[1]
    assert(#row.fields.slotName.value == 256 and #row.queries.GetUnassignedAtlasForSlot.values == 16)
end)

test("opaque userdata and producer objects are not retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function()
        local entry = newproxy(true); getmetatable(entry).__index = function(_, k) return k == "slot" and 7 or nil end
        local rows = { entry }; weak[1], weak[2] = rows, entry; return rows
    end, function() return secret end, function() return newproxy() end)
    local r = capture(); assert(r.groups.values[1].status == "restricted")
    collectgarbage("collect"); collectgarbage("collect"); assert(weak[1] == nil and weak[2] == nil)
end)

test("manual mode excluded from all and missing guards fail closed", function()
    setup(forbidden, forbidden, forbidden)
    SlashCmdList.APICONTRACTPROBE("all fixture")
    assert(ApiContractProbeDB.captures[1].outfitSlots == nil)
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("outfit-slots missing")
    assert(ApiContractProbeDB.captures[2].status == "missing-access-api")
end)
print("PASS " .. passed .. "/" .. passed)
