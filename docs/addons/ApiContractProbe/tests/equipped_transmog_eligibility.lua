local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local function setup(factory, query)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function() return false end
    canaccessvalue = function() return true end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    ItemLocation, C_Item = factory, { CanItemTransmogAppearance = query }
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("equipped-transmog-eligibility " .. (label or "sample"))
    local db = assert(ApiContractProbeDB, "manual mode absent")
    return assert(db.captures[#db.captures].equippedTransmogEligibility, "manual mode absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function opaque()
    return setmetatable({}, { __index = function() error("location traversed") end,
        __tostring = function() error("location stringified") end })
end

test("nineteen method receivers and original opaque identities", function()
    local created, queried, locations = 0, 0, {}
    local factory = {}
    factory.CreateFromEquipmentSlot = function(...)
        assert(select("#", ...) == 2)
        local self, slot = ...; created = created + 1
        assert(rawequal(self, factory) and slot == created)
        local location = slot % 2 == 0 and newproxy(true) or opaque()
        if type(location) == "userdata" then
            getmetatable(location).__index = function() error("userdata traversed") end
        end
        locations[slot] = location
        return location, nil, "tail"
    end
    setup(factory, function(...)
        assert(select("#", ...) == 1); queried = queried + 1
        assert(rawequal((...), locations[queried])); return false, 73
    end)
    local r = capture()
    assert(created == 19 and queried == 19 and #r.slots == 19)
    for slot, row in ipairs(r.slots) do
        assert(row.slot == slot and row.producer.n == 3)
        assert(row.producer.values[2].kind == "nil")
        assert(row.eligibility.n == 2 and row.eligibility.values[1].value == false)
        assert(row.eligibility.values[2].value == 73)
    end
end)

test("constructor errors and invalid first values do not suppress peers", function()
    local queries = 0
    setup({ CreateFromEquipmentSlot = function(_, slot)
        if slot == 1 then error(opaque()) end
        if slot == 2 then return end
        if slot == 3 then return nil, opaque() end
        if slot == 4 then return "not a location" end
        return opaque()
    end }, function() queries = queries + 1; return true, nil end)
    local r = capture()
    assert(queries == 15 and r.slots[1].producer.status == "call-error")
    assert(r.slots[2].producer.n == 0 and r.slots[3].producer.n == 2)
    for i = 1, 4 do assert(r.slots[i].eligibility.status == "unavailable-input") end
    assert(r.slots[19].eligibility.values[2].kind == "nil")
end)

test("missing and throwing APIs are explicit", function()
    for _, factory in ipairs({ {}, setmetatable({}, { __index = function() error("lookup") end }) }) do
        setup(factory, function() error("must not query") end)
        local r = capture()
        assert(r.slots[1].producer.status ~= "observed")
    end
    setup({ CreateFromEquipmentSlot = function() return opaque() end }, nil)
    assert(capture().slots[1].eligibility.status == "missing-api")
    setup({ CreateFromEquipmentSlot = function() return opaque() end }, function() end)
    C_Item = setmetatable({}, { __index = function() error("lookup") end })
    assert(capture().slots[1].eligibility.status == "field-error")
end)

test("constructor receiver revoked by method guards is never invoked", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        local denied, calls = false, 0
        local factory = { CreateFromEquipmentSlot = function() calls = calls + 1; return opaque() end }
        setup(factory, function() error("must not query") end)
        local fn = factory.CreateFromEquipmentSlot
        issecretvalue = function(v)
            if guard == "issecretvalue" and rawequal(v, fn) then denied = true end
            return false
        end
        canaccessvalue = function(v)
            if guard == "canaccessvalue" and rawequal(v, fn) then denied = true end
            return not (denied and rawequal(v, factory))
        end
        local r = capture()
        assert(calls == 0 and r.slots[1].producer.status ~= "observed")
    end
end)

test("location rechecked after namespace lookup and both function guards", function()
    for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
        for revoked = 1, 19 do
            local current, denied, calls = nil, nil, 0
            local fn = function(location)
                assert(not rawequal(location, denied)); calls = calls + 1; return true, 0
            end
            setup({ CreateFromEquipmentSlot = function(_, slot)
                current = opaque(); if slot == revoked then denied = current end; return current
            end }, fn)
            local armed = false
            local ns = setmetatable({}, { __index = function(_, key)
                assert(key == "CanItemTransmogAppearance")
                if phase == "lookup" and rawequal(current, denied) then armed = true end
                return fn
            end })
            C_Item = ns
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, fn) and rawequal(current, denied) then armed = true end
                return false
            end
            canaccessvalue = function(v)
                if ((phase == "namespace" and rawequal(v, ns)) or
                    (phase == "access" and rawequal(v, fn))) and rawequal(current, denied) then armed = true end
                return not (armed and rawequal(v, denied))
            end
            local r = capture()
            assert(calls == 18 and r.slots[revoked].eligibility.status == "restricted-input")
        end
    end
end)

test("producer serialization revocation and secret locations stay opaque", function()
    local location, denied, calls = nil, false, 0
    setup({ CreateFromEquipmentSlot = function() location = opaque(); denied = false; return location, "revoke" end },
        function() calls = calls + 1 end)
    canaccessvalue = function(v)
        if v == "revoke" then denied = true end
        return not (denied and rawequal(v, location))
    end
    assert(capture().slots[1].eligibility.status == "restricted-input" and calls == 0)
    setup({ CreateFromEquipmentSlot = function() return location end }, function() calls = calls + 1 end)
    issecretvalue = function(v) return rawequal(v, location) end
    assert(capture().slots[1].producer.values[1].status == "restricted" and calls == 0)
end)

test("zero nil error and sixteen-position bounds", function()
    local slot
    setup({ CreateFromEquipmentSlot = function(_, s) slot = s; return opaque() end }, function()
        if slot == 1 then return end
        if slot == 2 then return nil, false, nil end
        if slot == 3 then error(opaque()) end
        local values = {}; for i = 1, 18 do values[i] = string.rep("x", 300) end
        return unpack(values, 1, 18)
    end)
    local r = capture()
    assert(r.slots[1].eligibility.n == 0)
    assert(r.slots[2].eligibility.n == 3 and r.slots[2].eligibility.values[3].kind == "nil")
    assert(r.slots[3].eligibility.status == "call-error")
    local q = r.slots[4].eligibility
    assert(q.n == 18 and q.truncated and #q.values == 16 and #q.values[16].value == 256)
end)

test("ten snapshots label bounds and no retained locations", function()
    local weak = setmetatable({}, { __mode = "v" })
    local creates, queries = 0, 0
    setup({ CreateFromEquipmentSlot = function()
        creates = creates + 1; local location = opaque(); weak[creates] = location; return location
    end }, function() queries = queries + 1; return true, 0 end)
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(creates == 190 and queries == 190 and #ApiContractProbeDB.captures == 10)
    assert(ApiContractProbeDB.dropped == 1 and #ApiContractProbeDB.captures[1].label == 128)
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
end)

test("manual mode excluded from all and missing guards fail closed", function()
    local calls = 0
    setup({ CreateFromEquipmentSlot = function() calls = calls + 1; return opaque() end },
        function() calls = calls + 1 end)
    EquipItemByName = function() error("equip mutation") end
    C_Transmog = setmetatable({}, { __index = function() error("transmog operation") end })
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and ApiContractProbeDB.captures[1].equippedTransmogEligibility == nil)
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("equipped-transmog-eligibility")
    assert(calls == 0 and ApiContractProbeDB.captures[2].status == "missing-access-api")
end)

print(string.format("equipped-transmog-eligibility: %d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
