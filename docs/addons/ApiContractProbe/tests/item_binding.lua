local root = assert(arg[1], "addon directory required")
local passed = 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(producer, namespace)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return not not rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    GetInventoryItemLink, C_Item = producer, namespace
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("item-binding sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual item-binding mode absent")
    assert(ApiContractProbeDB.captures[1].label == "sample")
    return assert(ApiContractProbeDB.captures[1].itemBinding)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("real producer links only with exact arguments and arity", function()
    local sources, queries = {}, {}
    setup(function(...)
        assert(select("#", ...) == 2)
        local unit, slot = ...
        assert(unit == "player" and slot == #sources + 1)
        local link = "|Hitem:" .. slot .. "|h[fixture]|h"
        sources[#sources + 1] = link
        return link, nil, "extra"
    end, { IsItemBindToAccount = function(...)
        assert(select("#", ...) == 1)
        local link = ...
        queries[#queries + 1] = link
        assert(link == sources[#queries])
        return #queries % 2 == 0, nil
    end })
    local result = capture()
    assert(#sources == 19 and #queries == 19 and #result.slots == 19)
    for i, row in ipairs(result.slots) do
        assert(row.slot == i and row.producer.n == 3 and row.producer.values[2].kind == "nil")
        assert(row.binding.n == 2 and row.binding.values[2].kind == "nil")
        assert(row.binding.values[1].value == (i % 2 == 0))
    end
end)

test("absent invalid restricted and errored producers never classify false", function()
    local calls = 0
    setup(function(_, slot)
        if slot == 1 then return nil end
        if slot == 2 then return 123 end
        if slot == 3 then return secret end
        if slot == 4 then error(secret) end
        if slot == 5 then return end
        return "actual-link"
    end, { IsItemBindToAccount = function() calls = calls + 1; return false end })
    local result = capture()
    assert(result.slots[1].producer.n == 1 and result.slots[1].binding.status == "unavailable-input")
    assert(result.slots[2].binding.status == "unavailable-input")
    assert(result.slots[3].binding.status == "restricted-input")
    assert(result.slots[4].producer.status == "call-error" and result.slots[4].binding.status == "unavailable-input")
    assert(result.slots[5].producer.n == 0 and result.slots[5].binding.status == "unavailable-input")
    assert(calls == 14 and result.slots[19].binding.values[1].value == false)
end)

test("namespace failures do not stop independent slots", function()
    local produced, queried, lookups = 0, 0, 0
    setup(function() produced = produced + 1; return "link" end,
        setmetatable({}, { __index = function()
            lookups = lookups + 1
            if lookups == 1 then error(secret) end
            if lookups == 2 then return nil end
            return function() queried = queried + 1; if queried == 1 then error(secret) end; return true end
        end }))
    local result = capture()
    assert(produced == 19 and queried == 17)
    assert(result.slots[1].binding.status == "field-error")
    assert(result.slots[2].binding.status == "missing-api")
    assert(result.slots[3].binding.status == "call-error")
    assert(result.slots[19].binding.values[1].value == true)
end)

test("access precedes type and namespace inspection", function()
    setup(function(_, slot) if slot == 1 then return secret end; return "link" end, secret)
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    assert(result.slots[1].binding.status == "restricted-input")
    assert(result.slots[19].binding.status == "field-error")
    setup(function() return "link" end, { IsItemBindToAccount = secret })
    result = capture()
    assert(result.slots[19].binding.status == "missing-api")
end)

test("link access rechecked after function guard immediately before passing", function()
    local revoked, called = false, 0
    local fn = function() called = called + 1; error("revoked link passed") end
    setup(function() revoked = false; return "live-link" end, { IsItemBindToAccount = fn })
    canaccessvalue = function(v)
        if rawequal(v, fn) then revoked = true end
        return not (revoked and rawequal(v, "live-link"))
    end
    local result = capture()
    assert(called == 0 and result.slots[19].binding.status == "restricted-input")
end)

test("original link forwarded while both result tuples and bytes stay bounded", function()
    local link = string.rep("L", 400)
    local tuple = { link }
    for i = 2, 20 do tuple[i] = i end
    setup(function() return unpack(tuple) end, { IsItemBindToAccount = function(value)
        assert(value == link and #value == 400)
        return unpack(tuple)
    end })
    local row = capture().slots[1]
    for _, result in ipairs({ row.producer, row.binding }) do
        assert(result.n == 20 and #result.values == 16 and result.truncated)
        assert(#result.values[1].value == 256 and result.values[1].truncated)
    end
    setup(function() return "link" end, { IsItemBindToAccount = function() return secret, nil end })
    row = capture().slots[19]
    assert(row.binding.n == 2 and row.binding.values[1].status == "restricted")
end)

test("missing functions and failed predicates fail closed", function()
    setup(nil, nil)
    local result = capture()
    assert(result.slots[19].producer.status == "missing-api")
    assert(result.slots[19].binding.status == "unavailable-input")
    local calls = 0
    setup(function() calls = calls + 1; return "link" end, {})
    canaccessvalue = function() error(secret) end
    result = capture()
    assert(calls == 0 and result.slots[19].producer.status == "missing-api")
    setup(function() calls = calls + 1 end, {})
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("item-binding")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
end)

test("manual only and ten snapshot bound", function()
    local produced, queried = 0, 0
    local function source() produced = produced + 1; return "link" end
    local namespace = { IsItemBindToAccount = function() queried = queried + 1; return true end }
    setup(source, namespace)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(produced == 0 and queried == 0 and ApiContractProbeDB.captures[1].itemBinding == nil)
    setup(source, namespace)
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("item-binding") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(produced == 190 and queried == 190)
end)
print(string.format("%d/%d passed", passed, passed))
