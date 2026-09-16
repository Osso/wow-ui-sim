local root = assert(arg[1])
local passed = 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local keys = { "omittedFilterNegative", "helpful", "harmful" }
local fields = { "auraInstanceID", "spellId", "applications" }
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
    SlashCmdList.APICONTRACTPROBE("unit-auras-current sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "unit-auras-current mode absent")
    return assert(ApiContractProbeDB.captures[1].unitAurasCurrent)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("exact three calls and required-filter negative case remain independent", function()
    local calls = 0
    setup({ GetUnitAuras = function(...)
        calls = calls + 1
        local unit, filter, count = ...
        assert(unit == "player")
        if calls == 1 then
            assert(select("#", ...) == 1, "negative case must omit filter")
            error("filter required")
        end
        assert(select("#", ...) == 3 and count == 8)
        assert(filter == (calls == 2 and "HELPFUL" or "HARMFUL"))
        return { { auraInstanceID = 17.5, spellId = 31, applications = calls } }
    end })
    local r = capture()
    assert(calls == 3 and r.omittedFilterNegative.status == "call-error")
    for i = 2, 3 do
        local q = r[keys[i]]
        assert(q.n == 1 and q.values[1].kind == "table")
        assert(q.values[1].entries[1].fields.auraInstanceID.value == 17.5)
        assert(q.values[1].entries[1].fields.applications.value == i)
        assert(q.values[1].entries[8].kind == "nil")
    end
end)

test("return arity nil positions and only first returned table inspected", function()
    local calls = 0
    local opaque = setmetatable({}, { __index = function() error("extra return indexed") end })
    setup({ GetUnitAuras = function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil, opaque, nil end
        return { { auraInstanceID = false, spellId = "raw", applications = math.huge } }, nil, opaque
    end })
    local r = capture()
    assert(r.omittedFilterNegative.n == 0)
    assert(r.helpful.n == 3 and r.helpful.values[1].kind == "nil")
    assert(r.helpful.values[2].kind == "table" and r.helpful.values[2].entries == nil)
    assert(r.helpful.values[3].kind == "nil")
    local f = r.harmful.values[1].entries[1].fields
    assert(f.auraInstanceID.value == false and f.spellId.value == "raw" and f.applications.status == "nonfinite")
    assert(r.harmful.values[3].entries == nil)
end)

test("bounded numeric indexing whitelisted fields no traversal mutation or retention", function()
    local tableReads, fieldReads = 0, 0
    local weak = setmetatable({}, { __mode = "v" })
    local function forbidden() error("opaque traversal or mutation") end
    setup({ GetUnitAuras = function()
        local entry = newproxy(true)
        local mt = getmetatable(entry)
        mt.__index = function(_, key)
            fieldReads = fieldReads + 1
            assert(key == "auraInstanceID" or key == "spellId" or key == "applications")
            return key == "applications" and secret or key
        end
        mt.__newindex, mt.__tostring, mt.__len, mt.__pairs = forbidden, forbidden, forbidden, forbidden
        local t = setmetatable({}, { __index = function(_, index)
            tableReads = tableReads + 1
            assert(type(index) == "number" and index >= 1 and index <= 8)
            return entry
        end, __newindex = forbidden, __len = forbidden, __pairs = forbidden, __tostring = forbidden })
        weak[1], weak[2] = t, entry
        return t
    end })
    local r = capture()
    assert(tableReads == 24 and fieldReads == 72)
    assert(r.helpful.values[1].entries[8].fields.applications.status == "restricted")
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil, "native values retained in capture")
end)

test("table revocation stops subsequent lookups without suppressing peer calls", function()
    local t, revoked, reads, calls = nil, false, 0, 0
    t = setmetatable({}, { __index = function(_, index)
        reads = reads + 1
        assert(index == 1, "revoked table read")
        revoked = true
        return { auraInstanceID = 1 }
    end })
    setup({ GetUnitAuras = function() calls = calls + 1; return calls == 1 and t or {} end })
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, t)) end
    local r = capture()
    assert(calls == 3 and reads == 1)
    assert(r.omittedFilterNegative.values[1].entries[2].status == "field-error")
    assert(r.helpful.values[1].entries[1].kind == "nil")
end)

test("entry and field revocation checked at every field position", function()
    for target = 1, 3 do
        local entry, revoked, reads = nil, false, 0
        entry = setmetatable({}, { __index = function(_, key)
            reads = reads + 1
            assert(key == fields[reads] and reads <= target, "revoked entry read")
            if reads == target then revoked = true end
            return secret
        end })
        local calls = 0
        setup({ GetUnitAuras = function()
            calls = calls + 1
            return calls == 1 and { entry, { applications = 9 } } or {}
        end })
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, entry)) end
        local entries = capture().omittedFilterNegative.values[1].entries
        assert(reads == target and entries[1].fields[fields[target]].status == "restricted")
        for i = target + 1, 3 do assert(entries[1].fields[fields[i]].status == "field-error") end
        assert(entries[2].fields.applications.value == 9 and calls == 3)
    end
end)

test("lookup errors and inaccessible entries fields and return tables remain opaque", function()
    local entry = setmetatable({}, { __index = function(_, key)
        if key == "auraInstanceID" then error(secret) end
        return key == "spellId" and secret or 4
    end })
    local calls = 0
    setup({ GetUnitAuras = function()
        calls = calls + 1
        if calls == 1 then return secret end
        if calls == 2 then return { secret, entry, false } end
        return setmetatable({}, { __index = function(_, index)
            if index == 1 then error(secret) end
            return { applications = 6 }
        end })
    end })
    local r = capture()
    assert(r.omittedFilterNegative.values[1].status == "restricted")
    local entries = r.helpful.values[1].entries
    assert(entries[1].status == "restricted" and entries[1].fields == nil)
    assert(entries[2].fields.auraInstanceID.status == "field-error")
    assert(entries[2].fields.spellId.status == "restricted" and entries[2].fields.applications.value == 4)
    assert(entries[3].value == false and entries[3].fields == nil)
    assert(r.harmful.values[1].entries[1].status == "field-error")
    assert(r.harmful.values[1].entries[2].fields.applications.value == 6)
end)

test("field values revoked by lookup and table revoked during tuple serialization", function()
    for _, target in ipairs(fields) do
        local revoked = false
        local token = newproxy(true)
        getmetatable(token).__tostring = function() error("revoked field stringified") end
        local entry = setmetatable({}, { __index = function(_, key)
            if key == target then revoked = true; return token end
            return 8
        end })
        setup({ GetUnitAuras = function() return { entry } end })
        canaccessvalue = function(v) return not (revoked and rawequal(v, token)) end
        local f = capture().helpful.values[1].entries[1].fields
        assert(f[target].status == "restricted")
        for _, key in ipairs(fields) do if key ~= target then assert(f[key].value == 8) end end
    end
    local t = setmetatable({}, { __index = function() error("revoked table indexed") end })
    local extra, revoked = newproxy(true), false
    setup({ GetUnitAuras = function() return t, extra end })
    canaccessvalue = function(v)
        if rawequal(v, extra) then revoked = true end
        return not (revoked and rawequal(v, t))
    end
    local r = capture()
    assert(r.omittedFilterNegative.values[1].entries[1].status == "field-error")
    assert(r.helpful.values[1].status == "restricted")
    setup({ GetUnitAuras = function() return extra end })
    getmetatable(extra).__index = function() error("non-table first result indexed") end
    assert(capture().helpful.values[1].entries == nil)
end)

test("guards fail closed and unavailable APIs do not suppress peer lookups", function()
    for _, ns in ipairs({ {}, secret, { GetUnitAuras = secret },
        setmetatable({}, { __index = function() error(secret) end }) }) do
        setup(ns)
        local r = capture()
        for _, key in ipairs(keys) do assert(r[key].status ~= "observed") end
    end
    local lookups = 0
    setup(setmetatable({}, { __index = function(_, key)
        assert(key == "GetUnitAuras")
        lookups = lookups + 1
        if lookups == 1 then error(secret) end
        if lookups == 2 then return secret end
        return function() return {} end
    end }))
    local r = capture()
    assert(lookups == 3 and r.omittedFilterNegative.status == "field-error")
    assert(r.helpful.status == "missing-api" and r.harmful.status == "observed")
    setup({ GetUnitAuras = function() error("must not call") end })
    canaccessvalue = function() error(secret) end
    r = capture(); assert(r.helpful.status == "field-error")
    setup({}); issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("unit-auras-current")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)

test("tuple string label snapshot bounds and manual all exclusion", function()
    local calls = 0
    local function producer()
        calls = calls + 1
        local values = { { { applications = string.rep("x", 400) } } }
        for i = 2, 20 do values[i] = string.rep("v", 400) end
        return unpack(values)
    end
    setup({ GetUnitAuras = producer })
    local r = capture()
    for _, key in ipairs(keys) do
        local q = r[key]
        assert(q.n == 20 and q.truncated and #q.values == 16 and q.values[17] == nil)
        assert(#q.values[2].value == 256 and q.values[2].truncated)
        assert(#q.values[1].entries == 8)
        assert(#q.values[1].entries[1].fields.applications.value == 256)
    end
    setup({ GetUnitAuras = producer }); calls = 0
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and ApiContractProbeDB.captures[1].unitAurasCurrent == nil)
    setup({ GetUnitAuras = producer })
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("unit-auras-current " .. string.rep("L", 200)) end
    assert(calls == 30 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
end)

print(passed .. "/" .. passed .. " passed")
