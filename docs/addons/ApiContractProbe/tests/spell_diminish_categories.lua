local root = assert(arg[1])
local passed = 0
local rules = { "None", "PvE", "PvP" }
local categories = { "Root", "Taunt", "Stun", "AoEKnockback", "Incapacitate", "Disorient", "Silence", "Disarm" }
local fields = { "category", "name", "icon" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local excluded = 0
local function forbidden() excluded = excluded + 1; error("excluded operation") end
local function setup(list, info)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { SpellDiminishRuleset = {}, SpellDiminishCategory = {} }
    for i, name in ipairs(rules) do Enum.SpellDiminishRuleset[name] = i + 10.5 end
    for i, name in ipairs(categories) do Enum.SpellDiminishCategory[name] = i + 30.5 end
    Enum.SpellDiminishRuleset.Extra, Enum.SpellDiminishCategory.Extra = 999, 999
    C_SpellDiminish = { GetAllSpellDiminishCategories = list, GetSpellDiminishCategoryInfo = info,
        ShouldTrackSpellDiminishCategory = forbidden, IsSystemSupported = forbidden }
    CreateFrame, LoadAddOn = forbidden, forbidden
    C_AddOns = { LoadAddOn = forbidden }
    excluded = 0
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("spell-diminish-categories " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "spell-diminish-categories mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].spellDiminishCategories)
end
local function query(result, index)
    return index <= 3 and result.rulesets[index].observation or result.categories[index - 3].observation
end
local function test(name, fn)
    fn(); assert(excluded == 0)
    passed = passed + 1; print("PASS " .. name)
end

test("eleven independent calls use published noncanonical original values", function()
    local calls = 0
    local function fn(...)
        calls = calls + 1
        assert(select("#", ...) == 1)
        assert((...) == (calls <= 3 and calls + 10.5 or calls - 3 + 30.5))
        return { category = (...), name = "raw", icon = nil }
    end
    setup(fn, fn)
    local result = capture()
    assert(calls == 11 and #result.rulesets == 3 and #result.categories == 8)
    for i, name in ipairs(rules) do assert(result.rulesets[i].name == name) end
    for i, name in ipairs(categories) do
        local row = result.categories[i]
        assert(row.name == name and row.observation.input.value == i + 30.5)
        assert(row.observation.values[1].fields.category.value == i + 30.5)
        assert(row.observation.values[1].fields.icon.kind == "nil")
    end
end)

test("invalid missing and restricted enums never fall back", function()
    for _, bad in ipairs({ false, "1", math.huge, -math.huge, 0/0, secret }) do
        local calls = 0
        local function fn() calls = calls + 1 end
        setup(fn, fn)
        Enum.SpellDiminishRuleset.None, Enum.SpellDiminishCategory.Root = bad, bad
        local result = capture()
        assert(calls == 9 and query(result, 1).status == "unavailable-enum")
        assert(query(result, 4).status == "unavailable-enum")
    end
    setup(forbidden, forbidden)
    Enum.SpellDiminishRuleset, Enum.SpellDiminishCategory = secret, {}
    assert(query(capture(), 11).status == "unavailable-enum")
    Enum = nil
    assert(query(capture(), 1).status == "unavailable-enum")
end)

test("namespace function and enum access failures remain independent", function()
    setup(nil, function() return 9 end)
    local result = capture()
    assert(query(result, 1).status == "missing-api" and query(result, 4).values[1].value == 9)
    for _, bad in ipairs({ false, 4, secret }) do
        setup(bad, bad)
        assert(query(capture(), 11).status == "missing-api")
    end
    C_SpellDiminish = secret
    assert(query(capture(), 1).status == "field-error")
    C_SpellDiminish = setmetatable({}, { __index = function() error(secret) end })
    assert(query(capture(), 1).status == "field-error")
    local enumTable, revoked, reads = {}, false, 0
    setup(forbidden, forbidden)
    setmetatable(enumTable, { __index = function() reads = reads + 1; return 1 end })
    Enum = setmetatable({}, { __index = function() revoked = true; return enumTable end })
    canaccessvalue = function(v) return not (revoked and rawequal(v, enumTable)) end
    assert(query(capture(), 1).status == "unavailable-enum" and reads == 0)
end)

test("lookup and both function guards recheck every original input", function()
    for _, stage in ipairs({ "lookup", "secret", "access" }) do
        for index = 1, 11 do
            local calls, revoked = 0, false
            local blocked = index <= 3 and index + 10.5 or index - 3 + 30.5
            local function fn(value) assert(value ~= blocked); calls = calls + 1 end
            setup(fn, fn)
            if stage == "lookup" then
                C_SpellDiminish = setmetatable({}, { __index = function() revoked = true; return fn end })
            end
            issecretvalue = function(v)
                if stage == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if stage == "access" and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, blocked))
            end
            assert(query(capture(), index).status ~= "observed" and calls == 10)
        end
    end
end)

test("zero nil holes opaque errors and replacement do not suppress peers", function()
    local calls = 0
    local function fn()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil end
        if calls == 3 then error(secret) end
        return nil, secret, false, nil
    end
    setup(fn, fn)
    local result = capture()
    assert(calls == 11 and query(result, 1).n == 0 and query(result, 2).n == 1)
    assert(query(result, 3).status == "call-error")
    local q = query(result, 4)
    assert(q.n == 4 and q.values[1].kind == "nil" and q.values[2].status == "restricted")
    assert(q.values[3].value == false and q.values[4].kind == "nil")
    setup(function()
        C_SpellDiminish.GetAllSpellDiminishCategories = function() return "replacement" end
        error(secret)
    end, function() return 8 end)
    result = capture()
    assert(query(result, 1).status == "call-error" and query(result, 2).values[1].value == "replacement")
    assert(query(result, 11).values[1].value == 8)
end)

test("only first return and bounded fields are inspected without retention", function()
    local reads, fieldReads = 0, 0
    local weak = setmetatable({}, { __mode = "v" })
    local function unexpected() error("unexpected traversal") end
    local extra = setmetatable({}, { __index = unexpected })
    local function entry()
        local object = newproxy(true)
        local mt = getmetatable(object)
        mt.__index = function(_, key)
            assert(key == "category" or key == "name" or key == "icon")
            fieldReads = fieldReads + 1
            return key == "icon" and secret or key
        end
        mt.__newindex, mt.__len, mt.__tostring, mt.__pairs = unexpected, unexpected, unexpected, unexpected
        weak[#weak + 1] = object
        return object
    end
    setup(function()
        local list = setmetatable({}, { __index = function(_, index)
            assert(index >= 1 and index <= 8); reads = reads + 1; return entry()
        end, __newindex = unexpected, __pairs = unexpected, __len = unexpected })
        weak[#weak + 1] = list
        return list, nil, extra
    end, function() return entry(), extra end)
    local result = capture()
    assert(reads == 24 and fieldReads == 96)
    assert(query(result, 1).values[1].entries[8].fields.icon.status == "restricted")
    assert(query(result, 1).values[3].entries == nil and query(result, 4).values[2].fields == nil)
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
end)

test("list and info receiver guards apply at every index and field", function()
    for stop = 1, 8 do
        local revoked, reads = false, 0
        local list = setmetatable({}, { __index = function(_, index)
            reads = reads + 1; assert(index <= stop)
            if index == stop then revoked = true end
            return { category = index }
        end })
        setup(function() return list end, function() return {} end)
        canaccessvalue = function(v) return not (revoked and rawequal(v, list)) end
        capture(); assert(reads == stop)
    end
    for _, useList in ipairs({ false, true }) do
        for stop = 1, 3 do
            local revoked, reads = false, 0
            local object = setmetatable({}, { __index = function(_, key)
                reads = reads + 1; assert(reads <= stop and key == fields[reads])
                if reads == stop then revoked = true end
                return secret
            end })
            setup(function() return useList and { object } or {} end,
                function() return not useList and object or nil end)
            canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
            capture(); assert(reads == stop)
        end
    end
end)

test("tuple serialization revocation prevents subsequent list and info lookup", function()
    for _, useList in ipairs({ false, true }) do
        local revoked = false
        local object = setmetatable({}, { __index = function() error("revoked lookup") end })
        local marker = newproxy(true)
        setup(function() return useList and object or nil, marker end,
            function() return not useList and object or nil, marker end)
        canaccessvalue = function(v)
            if rawequal(v, marker) then revoked = true end
            return not (revoked and rawequal(v, object))
        end
        local result = capture()
        local first = query(result, useList and 1 or 4).values[1]
        if useList then assert(first.entries[1].status == "field-error")
        else assert(first.status == "restricted") end
    end
end)

test("restricted nil erroneous and raw fields keep independent observations", function()
    local object = setmetatable({}, { __index = function(_, key)
        if key == "category" then error(secret) end
        if key == "name" then return secret end
        return math.huge
    end })
    setup(function() return { secret, object, { category = false, name = 5 } } end,
        function() return object end)
    local result = capture()
    local entries = query(result, 1).values[1].entries
    assert(entries[1].status == "restricted")
    assert(entries[2].fields.category.status == "field-error")
    assert(entries[2].fields.name.status == "restricted" and entries[2].fields.icon.status == "nonfinite")
    assert(entries[3].fields.category.value == false and entries[3].fields.name.value == 5)
    assert(entries[3].fields.icon.kind == "nil" and entries[4].kind == "nil")
    assert(query(result, 11).values[1].fields.icon.status == "nonfinite")
end)

test("tuple string label snapshot and call bounds", function()
    local calls, values = 0, {}
    for i = 1, 20 do values[i] = string.rep("x", 300) end
    local function fn() calls = calls + 1; return unpack(values) end
    setup(fn, fn)
    local result = capture(string.rep("l", 200))
    for i = 1, 11 do
        local q = query(result, i)
        assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
    end
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(calls == 110 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("missing or failing access guards close input path and all excludes mode", function()
    setup(forbidden, forbidden)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("spell-diminish-categories absent")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
    setup(forbidden, forbidden)
    canaccessvalue = function() error(secret) end
    assert(query(capture(), 1).status == "unavailable-enum")
    setup(forbidden, forbidden)
    SlashCmdList.APICONTRACTPROBE("all context")
    assert(ApiContractProbeDB.captures[1].spellDiminishCategories == nil)
end)
print(string.format("%d spell-diminish-categories fixtures passed", passed))
