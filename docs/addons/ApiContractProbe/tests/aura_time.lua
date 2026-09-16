local root = assert(arg[1])
local passed = 0
local names = { "DoesAuraHaveExpirationTime", "GetAuraBaseDuration", "GetRefreshExtendedDuration", "GetAuraDuration" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(ns)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_UnitAuras = ns
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("aura-time sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "aura-time mode absent")
    return assert(ApiContractProbeDB.captures[1].auraTime)
end
local function namespace()
    local ns = { GetAuraSlots = function() return nil, 17 end,
        GetAuraDataBySlot = function() return { auraInstanceID = 91 } end }
    for _, name in ipairs(names) do ns[name] = function() return 3 end end
    return ns
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("original fractional IDs exact two arguments and opaque current objects", function()
    local ns, pages, data, queries = namespace(), 0, 0, 0
    local weak = setmetatable({}, { __mode = "v" })
    ns.GetAuraSlots = function(...)
        pages = pages + 1
        assert(select("#", ...) == 3)
        local unit, filter, count = ...
        assert(unit == "player" and filter == "HELPFUL" and count == 8)
        return 999, 17.5, 24.25
    end
    ns.GetAuraDataBySlot = function(...)
        assert(select("#", ...) == 2)
        local unit, slot = ...
        data = data + 1
        assert(unit == "player" and slot == (data == 1 and 17.5 or 24.25))
        local aura = newproxy(true)
        getmetatable(aura).__index = function(_, key) assert(key == "auraInstanceID"); return slot + 100 end
        return aura, nil
    end
    for _, name in ipairs(names) do
        ns[name] = function(...)
            queries = queries + 1
            assert(select("#", ...) == 2, "spellID must be omitted, not nil")
            local unit, id = ...
            assert(unit == "player" and id == (queries <= 4 and 117.5 or 124.25))
            if name == "GetAuraDuration" then
                local object = newproxy(true)
                getmetatable(object).__index = function() error("duration inspected") end
                getmetatable(object).__tostring = function() error("duration stringified") end
                weak[#weak + 1] = object
                return object, nil
            end
            return true, nil, 5
        end
    end
    setup(ns)
    local r = capture()
    assert(pages == 1 and data == 2 and queries == 8)
    assert(r.producer.n == 3 and r.producer.values[1].value == 999)
    assert(r.slots[1].producer.n == 2 and r.slots[1].producer.values[1].kind == "userdata")
    local q = r.slots[2].queries.GetAuraDuration
    assert(q.n == 2 and q.values[1].kind == "userdata" and q.values[1].value == nil and q.values[2].kind == "nil")
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil, "duration retained")
end)

test("zero and nil arity opaque errors and missing query independence", function()
    for _, missing in ipairs(names) do
        local ns, calls = namespace(), 0
        for _, name in ipairs(names) do ns[name] = function() calls = calls + 1; return 7 end end
        ns[missing] = nil
        setup(ns); local r = capture().slots[1]
        assert(calls == 3 and r.queries[missing].status == "missing-api")
    end
    local ns = namespace()
    ns.DoesAuraHaveExpirationTime = function() error(secret) end
    ns.GetAuraBaseDuration = function() end
    ns.GetRefreshExtendedDuration = function() return nil, secret, nil end
    ns.GetAuraDuration = function() return nil end
    setup(ns); local q = capture().slots[1].queries
    assert(q.DoesAuraHaveExpirationTime.status == "call-error")
    assert(q.GetAuraBaseDuration.n == 0)
    assert(q.GetRefreshExtendedDuration.n == 3 and q.GetRefreshExtendedDuration.values[2].status == "restricted")
    assert(q.GetAuraDuration.n == 1 and q.GetAuraDuration.values[1].kind == "nil")
end)

test("missing producers empty fixtures and invalid slots do not suppress peers", function()
    for _, ns in ipairs({ {}, { GetAuraSlots = function() end }, { GetAuraSlots = function() error(secret) end } }) do
        setup(ns); assert(#capture().slots == 0)
    end
    local ns, calls = namespace(), 0
    ns.GetAuraSlots = function() return nil, secret, nil, "slot", math.huge, 1, 2, 3, 4 end
    ns.GetAuraDataBySlot = function(_, slot)
        calls = calls + 1
        if slot == 1 then return secret end
        if slot == 2 then return { auraInstanceID = secret } end
        if slot == 3 then return { auraInstanceID = 0 / 0 } end
        return { auraInstanceID = 91 }
    end
    setup(ns); local r = capture()
    assert(#r.slots == 8 and calls == 4)
    assert(r.slots[1].producer.status == "restricted-input")
    assert(r.slots[2].producer.status == "unavailable-input")
    assert(r.slots[5].queries.GetAuraDuration.status == "restricted-input")
    assert(r.slots[6].queries.GetAuraDuration.status == "restricted-input")
    assert(r.slots[7].queries.GetAuraDuration.status == "unavailable-input")
    assert(r.slots[8].queries.GetAuraDuration.values[1].value == 3)
end)

test("slot and every query ID rechecked after namespace lookup and both guards", function()
    local targets = { "GetAuraDataBySlot", unpack(names) }
    for _, target in ipairs(targets) do
        for _, stage in ipairs({ "namespace", "lookup", "secret-guard", "access-guard" }) do
            local ns, revoked, calls = namespace(), false, 0
            local value = target == "GetAuraDataBySlot" and 17 or 91
            local fn = function() calls = calls + 1; error("revoked input forwarded") end
            ns[target] = nil
            setmetatable(ns, { __index = function(_, key)
                if key == target then
                    if stage == "lookup" then revoked = true end
                    return fn
                end
            end })
            setup(ns)
            local namespaces = 0
            local targetPosition = 2
            for i, name in ipairs(names) do if name == target then targetPosition = i + 2 end end
            issecretvalue = function(v)
                if stage == "secret-guard" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if rawequal(v, ns) then
                    namespaces = namespaces + 1
                    if stage == "namespace" and namespaces == targetPosition then revoked = true end
                end
                if stage == "access-guard" and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, value))
            end
            local r = capture().slots[1]
            assert(calls == 0)
            local result = target == "GetAuraDataBySlot" and r.producer or r.queries[target]
            assert(result.status == "restricted-input", target .. ":" .. stage)
        end
    end
end)

test("earlier query can revoke original ID before later queries", function()
    local ns, revoked, calls = namespace(), false, 0
    ns.DoesAuraHaveExpirationTime = function() revoked = true; return false end
    for i = 2, 4 do ns[names[i]] = function() calls = calls + 1 end end
    setup(ns)
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, 91)) end
    local q = capture().slots[1].queries
    assert(q.DoesAuraHaveExpirationTime.values[1].value == false and calls == 0)
    for i = 2, 4 do assert(q[names[i]].status == "restricted-input") end
end)

test("lookup failures and restricted functions leave other queries observable", function()
    local ns = namespace()
    ns.DoesAuraHaveExpirationTime, ns.GetAuraBaseDuration = nil, secret
    setmetatable(ns, { __index = function() error(secret) end })
    setup(ns); local q = capture().slots[1].queries
    assert(q.DoesAuraHaveExpirationTime.status == "field-error" and q.GetAuraBaseDuration.status == "missing-api")
    assert(q.GetRefreshExtendedDuration.status == "observed" and q.GetAuraDuration.status == "observed")
    setup(secret); assert(capture().producer.status == "field-error")
    setup(namespace()); canaccessvalue = function() error(secret) end
    assert(capture().producer.status == "field-error")
    setup(namespace()); issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("aura-time")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)

test("bounded tuples first page labels snapshots and all exclusion", function()
    local ns, pages, data, queries = namespace(), 0, 0, 0
    local tuple = {}; for i = 1, 20 do tuple[i] = i end
    ns.GetAuraSlots = function() pages = pages + 1; return unpack(tuple) end
    ns.GetAuraDataBySlot = function() data = data + 1; return { auraInstanceID = 91 } end
    for _, name in ipairs(names) do ns[name] = function()
        queries = queries + 1
        local values = { unpack(tuple) }; values[1] = string.rep("x", 400); return unpack(values)
    end end
    setup(ns); local r = capture()
    assert(pages == 1 and data == 8 and queries == 32 and r.producer.n == 20 and r.producer.truncated)
    local q = r.slots[1].queries.GetAuraBaseDuration
    assert(q.n == 20 and #q.values == 16 and q.truncated and #q.values[1].value == 256)
    setup(ns); pages, data, queries = 0, 0, 0
    SlashCmdList.APICONTRACTPROBE("all")
    assert(pages == 0 and ApiContractProbeDB.captures[1].auraTime == nil)
    setup(ns)
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("aura-time " .. string.rep("L", 200)) end
    assert(pages == 10 and data == 80 and queries == 320)
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
end)

test("inaccessible aura not inspected and unavailable ID outcomes explicit", function()
    local ns = namespace()
    ns.GetAuraDataBySlot = function() return secret end
    setup(ns)
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    assert(result.slots[1].queries.GetAuraDuration.status == "restricted-input")
    for _, producer in ipairs({ function() end, function() error(secret) end,
        function() return setmetatable({}, { __index = function() error(secret) end }) end }) do
        ns = namespace(); ns.GetAuraDataBySlot = producer
        setup(ns); assert(capture().slots[1].queries.GetAuraDuration.status ~= "observed")
    end
end)
print(string.format("%d/%d passed", passed, passed))
