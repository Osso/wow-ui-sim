local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(producer, namespace)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    GetInventoryItemLink, C_Item = producer, namespace
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("equipped-item-info " .. (label or "sample"))
    local db = assert(ApiContractProbeDB, "manual equipped-item-info mode absent")
    return assert(db.captures[#db.captures].equippedItemInfo, "manual equipped-item-info mode absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("original binary long links and all eighteen declared return positions", function()
    local sources, queries, links = 0, 0, {}
    setup(function(...)
        assert(select("#", ...) == 2)
        local unit, slot = ...; sources = sources + 1
        assert(unit == "player" and slot == sources)
        links[slot] = string.rep("L", 400) .. "\0\255" .. slot
        return links[slot], nil, "producer-tail"
    end, { GetItemInfo = function(...)
        assert(select("#", ...) == 1)
        queries = queries + 1; assert((...) == links[queries])
        local values = {}; for i = 1, 18 do values[i] = i end
        values[16], values[17], values[18] = nil, false, string.rep("D", 300)
        return unpack(values, 1, 18)
    end })
    local r = capture()
    assert(sources == 19 and queries == 19 and #r.slots == 19)
    for slot, row in ipairs(r.slots) do
        assert(row.slot == slot and row.producer.n == 3 and row.producer.values[2].kind == "nil")
        assert(#row.producer.values[1].value == 256)
        local q = row.itemInfo
        assert(q.status == "observed" and q.n == 18 and not q.truncated)
        for i = 1, 15 do assert(q.values[i].value == i) end
        assert(q.values[16].kind == "nil" and q.values[17].value == false)
        assert(#q.values[18].value == 256 and q.values[19] == nil)
    end
end)

test("nil holes at sixteen seventeen eighteen and uninspected nineteenth sentinel", function()
    local slot, sentinelReads = 0, 0
    setup(function(_, s) slot = s; return "link" end, { GetItemInfo = function()
        local values = {}; for i = 1, 19 do values[i] = i end
        values[16], values[17], values[18], values[19] = nil, nil, nil, secret
        return unpack(values, 1, slot == 1 and 18 or 19)
    end })
    local old = issecretvalue
    issecretvalue = function(v)
        if rawequal(v, secret) then sentinelReads = sentinelReads + 1 end
        return old(v)
    end
    local r = capture()
    for i, row in ipairs(r.slots) do
        local q = row.itemInfo
        assert(q.n == (i == 1 and 18 or 19))
        assert((q.truncated == true) == (i ~= 1))
        for j = 16, 18 do assert(q.values[j].kind == "nil") end
        assert(q.values[19] == nil)
    end
    assert(sentinelReads == 0)
end)

test("producer stays sixteen while item results have a separate eighteen bound", function()
    local values = {}; for i = 1, 20 do values[i] = i end
    values[1] = "original-link"
    setup(function() return unpack(values, 1, 20) end,
        { GetItemInfo = function(link) assert(link == values[1]); return unpack(values, 1, 20) end })
    local r = capture()
    for _, row in ipairs(r.slots) do
        assert(row.producer.n == 20 and row.producer.truncated and #row.producer.values == 16)
        assert(row.itemInfo.n == 20 and row.itemInfo.truncated and #row.itemInfo.values == 18)
    end
end)

test("missing invalid secret and error producers skip independently", function()
    local calls = 0
    setup(function(_, slot)
        if slot == 1 then return end
        if slot == 2 then return nil end
        if slot == 3 then return 12 end
        if slot == 4 then return secret end
        if slot == 5 then error(secret) end
        return "link"
    end, { GetItemInfo = function() calls = calls + 1; return false end })
    local r = capture(); assert(calls == 14)
    assert(r.slots[1].producer.n == 0 and r.slots[2].producer.n == 1)
    for _, i in ipairs({ 1, 2, 3, 5 }) do assert(r.slots[i].itemInfo.status == "unavailable-input") end
    assert(r.slots[4].itemInfo.status == "restricted-input")
    assert(r.slots[5].producer.status == "call-error" and r.slots[19].itemInfo.values[1].value == false)
    setup(nil, { GetItemInfo = function() error("unexpected query") end })
    assert(capture().slots[1].producer.status == "missing-api")
end)

test("namespace function and call failures preserve peer tuples", function()
    local lookups = 0
    setup(function() return "link" end, setmetatable({}, { __index = function(_, key)
        assert(key == "GetItemInfo"); lookups = lookups + 1
        local index = lookups
        if index == 1 then error(secret) end
        if index == 2 then return nil end
        if index == 3 then return secret end
        return function()
            if index == 4 then error(secret) end
            if index == 5 then return end
            if index == 6 then return nil end
            return "ok", nil, "tail"
        end
    end }))
    local r = capture()
    assert(r.slots[1].itemInfo.status == "field-error")
    assert(r.slots[2].itemInfo.status == "missing-api" and r.slots[3].itemInfo.status == "missing-api")
    assert(r.slots[4].itemInfo.status == "call-error")
    assert(r.slots[5].itemInfo.n == 0 and r.slots[6].itemInfo.n == 1)
    assert(r.slots[19].itemInfo.n == 3 and r.slots[19].itemInfo.values[2].kind == "nil")
    setup(function() return "link" end, secret)
    assert(capture().slots[19].itemInfo.status == "field-error")
end)

test("every slot rechecks link after namespace lookup and function guards", function()
    for _, phase in ipairs({ "namespace", "lookup", "function-secret", "function-access" }) do
        for target = 1, 19 do
            local current, revoked, calls = 0, false, 0
            local link = "original-binary\0" .. string.rep("x", 300)
            local fn = function(value)
                assert(not revoked and value == link); calls = calls + 1; return true
            end
            local namespace = setmetatable({}, { __index = function()
                if phase == "lookup" and current == target then revoked = true end
                return fn
            end })
            setup(function(_, slot) current, revoked = slot, false; return link end, namespace)
            issecretvalue = function(v)
                if current == target and phase == "function-secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if current == target and ((phase == "namespace" and rawequal(v, namespace))
                    or (phase == "function-access" and rawequal(v, fn))) then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, link))
            end
            local r = capture()
            assert(calls == 18 and r.slots[target].itemInfo.status == "restricted-input", phase .. target)
        end
    end
end)

test("producer serialization can revoke original link before query", function()
    local revoked, calls = false, 0
    local link, tail = "link", {}
    setup(function() revoked = false; return link, tail end,
        { GetItemInfo = function() calls = calls + 1 end })
    canaccessvalue = function(v)
        if rawequal(v, tail) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, link))
    end
    local r = capture()
    assert(calls == 0 and r.slots[19].itemInfo.status == "restricted-input")
end)

test("restricted tail outputs stay opaque and returned objects are collectible", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() return "link" end, { GetItemInfo = function()
        local object = setmetatable({}, { __index = function() error("output lookup") end })
        weak[#weak + 1] = object
        local values = {}; for i = 1, 15 do values[i] = i end
        values[16], values[17], values[18] = secret, object, function() error("output invocation") end
        return unpack(values, 1, 18)
    end })
    local r = capture()
    for _, row in ipairs(r.slots) do
        assert(row.itemInfo.values[16].status == "restricted")
        assert(row.itemInfo.values[17].kind == "table" and row.itemInfo.values[18].kind == "function")
    end
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
end)

test("ten snapshots label output limits no mutations or output forwarding", function()
    local sources, queries = 0, 0
    setup(function() sources = sources + 1; return "link" end,
        setmetatable({ GetItemInfo = function() queries = queries + 1; return string.rep("z", 300) end },
            { __index = function(_, key) error("excluded item call " .. key) end }))
    ItemLocation = setmetatable({}, { __index = function() error("ItemLocation lookup") end })
    C_TransmogCollection = setmetatable({}, { __index = function() error("transmog lookup") end })
    for i = 1, 11 do capture(string.rep("q", 200)) end
    assert(sources == 190 and queries == 190 and #ApiContractProbeDB.captures == 10)
    assert(ApiContractProbeDB.dropped == 1 and #ApiContractProbeDB.captures[1].label == 128)
    assert(#ApiContractProbeDB.captures[1].equippedItemInfo.slots[19].itemInfo.values[1].value == 256)
    ItemLocation, C_TransmogCollection = nil, nil
end)

test("manual mode excluded from all and old item-binding still uses sixteen", function()
    local queries, bindings = 0, 0
    local values = {}; for i = 1, 18 do values[i] = i end
    setup(function() return "link" end, {
        GetItemInfo = function() queries = queries + 1; return unpack(values, 1, 18) end,
        IsItemBindToAccount = function() bindings = bindings + 1; return unpack(values, 1, 18) end,
    })
    capture()
    SlashCmdList.APICONTRACTPROBE("item-binding old")
    local old = ApiContractProbeDB.captures[2].itemBinding.slots[1].binding
    assert(old.n == 18 and old.truncated and #old.values == 16)
    SlashCmdList.APICONTRACTPROBE("all sample")
    assert(queries == 19 and bindings == 19)
    assert(ApiContractProbeDB.captures[3].equippedItemInfo == nil)
end)

test("missing access APIs fail closed", function()
    setup(function() error("unexpected producer") end, { GetItemInfo = function() error("unexpected query") end })
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("equipped-item-info sample")
    assert(ApiContractProbeDB and ApiContractProbeDB.captures[1].status == "missing-access-api")
end)

print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
