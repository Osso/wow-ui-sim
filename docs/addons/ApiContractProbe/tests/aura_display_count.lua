local root = assert(arg[1])
local passed = 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(namespace)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_UnitAuras = namespace
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("aura-display-count sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "aura-display-count mode absent")
    return assert(ApiContractProbeDB.captures[1].auraDisplayCount)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end
local function namespace()
    return { GetAuraSlots = function() return nil, 17 end,
        GetAuraDataBySlot = function() return { auraInstanceID = 91 } end,
        GetAuraApplicationDisplayCount = function() return "2" end }
end

test("vararg first page and exact five argument variants", function()
    local ns, data, queries, pages = namespace(), 0, 0, 0
    ns.GetAuraSlots = function(...)
        pages = pages + 1
        assert(select("#", ...) == 3)
        local unit, filter, count = ...
        assert(unit == "player" and filter == "HELPFUL" and count == 8)
        return nil, 17, 24
    end
    ns.GetAuraDataBySlot = function(unit, slot)
        data = data + 1; assert(unit == "player" and slot == (data == 1 and 17 or 24))
        return setmetatable({ auraInstanceID = slot + 100 }, { __index = function() error("extra field") end }), nil
    end
    ns.GetAuraApplicationDisplayCount = function(...)
        queries = queries + 1
        local unit, id, minimum, maximum = ...
        local variant = (queries - 1) % 5 + 1
        assert(unit == "player" and id == (queries <= 5 and 117 or 124))
        assert(select("#", ...) == ({ 2, 3, 3, 4, 4 })[variant])
        assert(minimum == ({ [2] = 1, [3] = 2, [4] = 2, [5] = 1 })[variant])
        assert(maximum == ({ [4] = 5, [5] = 1 })[variant])
        return "count", nil
    end
    setup(ns)
    local r = capture()
    assert(pages == 1 and data == 2 and queries == 10)
    assert(r.producer.n == 3 and r.producer.values[1].kind == "nil")
    assert(r.slots[1].producer.n == 2 and r.slots[1].producer.values[1].kind == "table")
    assert(r.slots[2].queries[5].n == 2 and r.slots[2].queries[5].values[2].kind == "nil")
end)

test("missing APIs empty page and opaque producer errors", function()
    for _, ns in ipairs({ {}, { GetAuraSlots = function() end }, { GetAuraSlots = function() error(secret) end } }) do
        setup(ns); local r = capture(); assert(#r.slots == 0)
    end
    local ns = namespace(); ns.GetAuraDataBySlot = nil
    setup(ns); assert(capture().slots[1].producer.status == "missing-api")
    ns = namespace(); ns.GetAuraApplicationDisplayCount = nil
    setup(ns); assert(capture().slots[1].queries[5].status == "missing-api")
end)

test("invalid restricted slots and IDs preserve independent positions", function()
    local ns, calls = namespace(), 0
    ns.GetAuraSlots = function() return 123, secret, nil, "slot", math.huge, 3, 4, 5, 6 end
    ns.GetAuraDataBySlot = function(_, slot)
        calls = calls + 1
        if slot == 3 then return secret end
        if slot == 4 then return { auraInstanceID = secret } end
        if slot == 5 then return { auraInstanceID = "id" } end
        return { auraInstanceID = 91 }
    end
    setup(ns); local r = capture()
    assert(calls == 4 and #r.slots == 8)
    assert(r.slots[1].producer.status == "restricted-input")
    assert(r.slots[2].producer.status == "unavailable-input")
    assert(r.slots[5].queries[1].status == "restricted-input")
    assert(r.slots[6].queries[1].status == "restricted-input")
    assert(r.slots[7].queries[1].status == "unavailable-input")
    assert(r.slots[8].queries[5].values[1].value == "2")
end)

test("slot and ID access rechecked after lookup and function guards", function()
    for _, stage in ipairs({ "lookup", "function" }) do
        for _, target in ipairs({ "GetAuraDataBySlot", "GetAuraApplicationDisplayCount" }) do
            local ns, revoked, calls = namespace(), false, 0
            local value = target == "GetAuraDataBySlot" and 17 or 91
            local fn = function() calls = calls + 1; error("revoked input forwarded") end
            ns[target] = nil
            setmetatable(ns, { __index = function(_, key)
                if key == target then if stage == "lookup" then revoked = true end; return fn end
            end })
            setup(ns)
            canaccessvalue = function(v)
                if stage == "function" and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, value))
            end
            local r = capture()
            assert(calls == 0)
            local result = target == "GetAuraDataBySlot" and r.slots[1].producer or r.slots[1].queries[1]
            assert(result.status == "restricted-input")
        end
    end
end)

test("userdata ID only and query failures remain independent", function()
    local ns, calls = namespace(), 0
    local aura = newproxy(true)
    getmetatable(aura).__index = function(_, key) assert(key == "auraInstanceID"); return 91 end
    ns.GetAuraDataBySlot = function() return aura end
    ns.GetAuraApplicationDisplayCount = function()
        calls = calls + 1
        if calls == 1 then error(secret) end
        if calls == 2 then return end
        if calls == 3 then return nil, secret, nil end
        return "ok"
    end
    setup(ns); local r = capture().slots[1]
    assert(calls == 5 and r.queries[1].status == "call-error" and r.queries[2].n == 0)
    assert(r.queries[3].n == 3 and r.queries[3].values[2].status == "restricted")
    assert(r.queries[5].values[1].value == "ok")
end)

test("bounded tuples slots snapshots strings and manual exclusion", function()
    local ns, pages, data, queries = namespace(), 0, 0, 0
    local tuple = {}; for i = 1, 20 do tuple[i] = i end
    ns.GetAuraSlots = function() pages = pages + 1; return unpack(tuple) end
    ns.GetAuraDataBySlot = function() data = data + 1; return { auraInstanceID = 91 } end
    ns.GetAuraApplicationDisplayCount = function()
        queries = queries + 1; local values = { unpack(tuple) }; values[1] = string.rep("x", 400); return unpack(values)
    end
    setup(ns); local r = capture()
    assert(pages == 1 and data == 8 and queries == 40 and r.producer.n == 20 and r.producer.truncated)
    local q = r.slots[1].queries[1]
    assert(q.n == 20 and #q.values == 16 and q.truncated and #q.values[1].value == 256)
    setup(ns); pages, data, queries = 0, 0, 0
    SlashCmdList.APICONTRACTPROBE("all")
    assert(pages == 0 and ApiContractProbeDB.captures[1].auraDisplayCount == nil)
    setup(ns)
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("aura-display-count " .. string.rep("L", 200)) end
    assert(pages == 10 and data == 80 and queries == 400)
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
end)
test("lookup errors missing fixtures and secret functions fail independently", function()
    local ns, lookups = namespace(), 0
    ns.GetAuraSlots = function() return nil, 1, 2, 3, 4 end
    ns.GetAuraDataBySlot = function(_, slot)
        if slot == 1 then error(secret) end
        if slot == 2 then return nil end
        if slot == 3 then return setmetatable({}, { __index = function() error(secret) end }) end
        return { auraInstanceID = 91 }
    end
    ns.GetAuraApplicationDisplayCount = nil
    setmetatable(ns, { __index = function(_, key)
        assert(key == "GetAuraApplicationDisplayCount")
        lookups = lookups + 1
        if lookups == 1 then error(secret) end
        if lookups == 2 then return secret end
        return function() return "live" end
    end })
    setup(ns); local r = capture()
    assert(r.slots[1].producer.status == "call-error")
    assert(r.slots[2].queries[1].status == "unavailable-input")
    assert(r.slots[3].queries[1].status == "field-error")
    assert(r.slots[4].queries[1].status == "field-error")
    assert(r.slots[4].queries[2].status == "missing-api")
    assert(r.slots[4].queries[5].values[1].value == "live")
    setup(secret); assert(capture().producer.status == "field-error")
    setup(namespace()); issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("aura-display-count")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)

test("no type inspection of inaccessible aura values and failed guards", function()
    local ns = namespace()
    ns.GetAuraDataBySlot = function() return secret end
    setup(ns)
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    assert(result.slots[1].queries[5].status == "restricted-input")
    setup(namespace()); canaccessvalue = function() error(secret) end
    assert(capture().producer.status == "field-error")
end)
print(string.format("%d/%d passed", passed, passed))
