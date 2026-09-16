local root = assert(arg[1])
local passed, calls, mutations = 0, {}, 0
local fields = { "setID", "name", "collected", "favorite", "validForCharacter" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function forbidden() mutations = mutations + 1; error("forbidden operation") end
local function setup(list, filters)
    ApiContractProbeDB, SlashCmdList, calls, mutations = nil, {}, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_TransmogSets = {
        GetAvailableSets = list, IsUsingDefaultSetsFilters = filters,
        SetDefaultSetsFilters = forbidden, SetSetsFilter = forbidden,
        GetSetInfo = forbidden, SetFavorite = forbidden,
    }
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("sets-catalog " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "sets-catalog mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].setsCatalog)
end
local function test(name, fn)
    fn(); assert(mutations == 0); passed = passed + 1; print("PASS " .. name)
end

test("one list and two independent no-argument filter calls", function()
    setup(function(...)
        assert(select("#", ...) == 0); calls[#calls + 1] = "list"
        return { { setID = 3.5, name = "Set", collected = false, favorite = true, validForCharacter = false } }
    end, function(...)
        assert(select("#", ...) == 0); calls[#calls + 1] = "filter"
        return #calls == 2
    end)
    local r = capture()
    assert(table.concat(calls, ",") == "list,filter,filter")
    local f = r.available.values[1].entries[1].fields
    assert(f.setID.value == 3.5 and f.name.value == "Set" and f.collected.value == false)
    assert(f.favorite.value == true and f.validForCharacter.value == false)
    assert(r.filters[1].values[1].value == true and r.filters[2].values[1].value == false)
end)

test("missing restricted functions namespaces and throwing lookups", function()
    for _, bad in ipairs({ false, 1, secret }) do
        setup(bad, bad)
        local r = capture()
        assert(r.available.status == "missing-api" and r.filters[2].status == "missing-api")
    end
    setup(nil, nil)
    assert(capture().available.status == "missing-api")
    for _, ns in ipairs({ secret, setmetatable({}, { __index = function() error(secret) end }) }) do
        C_TransmogSets = ns
        local r = capture()
        assert(r.available.status == "field-error" and r.filters[1].status == "field-error")
    end
end)

test("both function guards prevent restricted calls", function()
    for _, stage in ipairs({ "secret", "access" }) do
        local fn = function() error("must not call") end
        setup(fn, fn)
        if stage == "secret" then issecretvalue = function(v) return rawequal(v, fn) end
        else canaccessvalue = function(v) return not rawequal(v, fn) end end
        local r = capture()
        assert(r.available.status == "missing-api" and r.filters[2].status == "missing-api")
    end
end)

test("zero returns nil holes opaque errors and fresh peer lookup", function()
    setup(function() error(secret) end, function()
        C_TransmogSets.IsUsingDefaultSetsFilters = function() return nil, secret, false, nil end
        return
    end)
    local r = capture()
    assert(r.available.status == "call-error" and r.filters[1].n == 0)
    assert(r.filters[2].n == 4 and r.filters[2].values[1].kind == "nil")
    assert(r.filters[2].values[2].status == "restricted" and r.filters[2].values[3].value == false)
    assert(r.filters[2].values[4].kind == "nil")
    setup(function() return nil, { { setID = 99 } }, nil end, function() return nil end)
    r = capture()
    assert(r.available.n == 3 and r.available.values[2].entries == nil and r.filters[1].n == 1)
end)

test("first table only eight entries five fields opaque weak references", function()
    local reads, fieldReads = 0, 0
    local weak = setmetatable({}, { __mode = "v" })
    setup(function()
        local entry = newproxy(true)
        getmetatable(entry).__index = function(_, key)
            local found = false
            for _, field in ipairs(fields) do if key == field then found = true end end
            assert(found, "unexpected field: " .. key); fieldReads = fieldReads + 1
            return key == "name" and string.rep("x", 300) or secret
        end
        getmetatable(entry).__tostring = forbidden
        getmetatable(entry).__newindex = forbidden
        local list = setmetatable({}, { __index = function(_, index)
            assert(index >= 1 and index <= 8); reads = reads + 1; return entry
        end, __len = forbidden, __pairs = forbidden, __newindex = forbidden })
        local extra = setmetatable({}, { __index = forbidden })
        weak[1], weak[2], weak[3] = list, entry, extra
        return list, extra
    end, function() return true end)
    local r = capture()
    assert(reads == 8 and fieldReads == 40 and r.available.values[2].entries == nil)
    assert(#r.available.values[1].entries[8].fields.name.value == 256)
    assert(r.available.values[1].entries[8].fields.setID.status == "restricted")
    collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)

test("list receiver rechecked at every index", function()
    for stop = 1, 8 do
        local reads, revoked = 0, false
        local list = setmetatable({}, { __index = function(_, index)
            assert(not revoked); reads = reads + 1
            if index == stop then revoked = true end
            return { name = "entry" }
        end })
        setup(function() return list end, function() return true end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
        local r = capture()
        assert(reads == stop and r.filters[2].values[1].value == true)
        if stop < 8 then assert(r.available.values[1].entries[stop + 1].status == "field-error") end
    end
end)

test("entry receiver rechecked at every field and peers continue", function()
    for stop = 1, 5 do
        local reads, revoked = 0, false
        local entry = setmetatable({}, { __index = function(_, key)
            assert(not revoked and key == fields[reads + 1]); reads = reads + 1
            if reads == stop then revoked = true end
            return key
        end })
        setup(function() return { entry, { setID = 7 } } end, function() return false end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, entry)) end
        local r = capture()
        assert(reads == stop and r.available.values[1].entries[2].fields.setID.value == 7)
        if stop < 5 then assert(r.available.values[1].entries[1].fields[fields[stop + 1]].status == "field-error") end
    end
end)

test("tuple serialization revocation and field guards prevent inspection", function()
    local revoked, reads = false, 0
    local list = setmetatable({}, { __index = function() reads = reads + 1; error("revoked list") end })
    local trigger = {}
    setup(function() return list, trigger end, function() return true end)
    canaccessvalue = function(v)
        if rawequal(v, trigger) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, list))
    end
    local r = capture()
    assert(reads == 0 and r.available.values[1].entries[1].status == "field-error")
    setup(function() return { secret, { name = secret, setID = math.huge }, false } end, function() return true end)
    r = capture()
    assert(r.available.values[1].entries[1].status == "restricted")
    assert(r.available.values[1].entries[2].fields.name.status == "restricted")
    assert(r.available.values[1].entries[2].fields.setID.status == "nonfinite")
    assert(r.available.values[1].entries[3].fields == nil)
end)

test("field and entry errors remain independent", function()
    local entry = setmetatable({}, { __index = function(_, key)
        if key == "name" then error(secret) end
        return key
    end })
    setup(function() return setmetatable({}, { __index = function(_, index)
        if index == 1 then error(secret) end
        return entry
    end }) end, function() error(secret) end)
    local r = capture()
    assert(r.available.values[1].entries[1].status == "field-error")
    local f = r.available.values[1].entries[2].fields
    assert(f.name.status == "field-error" and f.collected.value == "collected")
    assert(r.filters[1].status == "call-error" and r.filters[2].status == "call-error")
end)

test("tuple strings label snapshot and three-call bounds", function()
    local count, values = 0, {}
    for i = 1, 20 do values[i] = string.rep("v", 300) end
    local fn = function() count = count + 1; return unpack(values) end
    setup(fn, fn)
    local r = capture(string.rep("l", 200))
    assert(r.available.n == 20 and r.available.truncated and #r.available.values == 16)
    assert(#r.filters[1].values[1].value == 256 and #ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(count == 30 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("manual only and missing access API fail closed", function()
    setup(forbidden, forbidden)
    SlashCmdList.APICONTRACTPROBE("all sample")
    assert(ApiContractProbeDB.captures[1].setsCatalog == nil)
    setup(forbidden, forbidden)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("sets-catalog missing")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)
print("PASS " .. passed .. " sets-catalog fixtures")
