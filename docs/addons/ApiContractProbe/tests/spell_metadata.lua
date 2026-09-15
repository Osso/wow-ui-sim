local root = assert(arg[1], "addon directory required")
local passed = 0
local names = { "GetSpellDisplayCount", "GetSpellMaxCumulativeAuraApplications", "IsConsumableSpell",
    "IsExternalDefensive", "IsPriorityAura", "IsSpellCrowdControl", "IsSpellImportant" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(producer, namespace)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    GetActionInfo, C_Spell = producer, namespace
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(input)
    SlashCmdList.APICONTRACTPROBE(input or "spell-metadata 17 sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual spell-metadata mode absent")
    return assert(ApiContractProbeDB.captures[1].spellMetadata)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end
local function namespace(fn)
    local result = {}
    for _, name in ipairs(names) do result[name] = fn end
    return result
end

test("selected slot is sole producer argument and original ID sole query argument", function()
    local queries = 0
    setup(function(...)
        assert(select("#", ...) == 1 and (...) == 17)
        return "spell", 12345.5, nil, "extra"
    end, namespace(function(...)
        assert(select("#", ...) == 1 and (...) == 12345.5)
        queries = queries + 1
        return false, nil, "raw"
    end))
    local result = capture()
    assert(result.slot == 17 and result.identity.n == 4 and result.identity.values[3].kind == "nil")
    assert(queries == 7 and ApiContractProbeDB.captures[1].label == "sample")
    for _, name in ipairs(names) do
        local row = result.queries[name]
        assert(row.n == 3 and row.values[1].value == false and row.values[2].kind == "nil")
    end
end)

test("invalid identities never reach spell queries", function()
    local inputs = { { "item", 12 }, { "Spell", 12 }, { "spell", "12" }, { "spell", math.huge },
        { "spell", -math.huge }, { "spell", 0/0 }, { secret, 12 }, { "spell", secret }, {} }
    for _, input in ipairs(inputs) do
        local calls = 0
        setup(function() return unpack(input, 1, 2) end, namespace(function() calls = calls + 1 end))
        local result = capture()
        assert(calls == 0)
        for _, name in ipairs(names) do assert(result.queries[name].status ~= "observed") end
    end
    setup(function() error(secret) end, namespace(function() error("unexpected query") end))
    assert(capture().identity.status == "call-error")
    setup(nil, {})
    assert(capture().identity.status == "missing-api")
end)

test("namespace and function failures preserve independent peers", function()
    local lookups, calls = 0, 0
    setup(function() return "spell", 12345 end, setmetatable({}, { __index = function(_, name)
        lookups = lookups + 1
        assert(name == names[lookups], "unexpected API including visibility query")
        if lookups == 1 then error(secret) end
        if lookups == 2 then return nil end
        return function()
            calls = calls + 1
            if calls == 1 then error(secret) end
            return true
        end
    end }))
    local rows = capture().queries
    assert(lookups == 7 and calls == 5)
    assert(rows[names[1]].status == "field-error" and rows[names[2]].status == "missing-api")
    assert(rows[names[3]].status == "call-error" and rows[names[7]].values[1].value == true)
end)

test("restricted identities namespaces and results are never inspected", function()
    setup(function() return secret, secret end, secret)
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    assert(result.identity.values[1].status == "restricted")
    setup(function() return "spell", 12345 end, secret)
    assert(capture().queries[names[1]].status == "field-error")
    setup(function() return "spell", 12345 end, namespace(function() return secret, nil end))
    result = capture()
    assert(result.queries[names[7]].n == 2 and result.queries[names[7]].values[1].status == "restricted")
end)

test("ID access rechecked after each lookup and function guard", function()
    local revoked, calls = false, 0
    local fn = function() calls = calls + 1 end
    setup(function() return "spell", 12345 end, namespace(fn))
    canaccessvalue = function(v)
        if rawequal(v, fn) then revoked = true end
        return not (revoked and rawequal(v, 12345))
    end
    local rows = capture().queries
    assert(calls == 0 and rows[names[1]].status == "restricted-input")
    setup(function() return "spell", 12345 end, namespace(function()
        calls = calls + 1
        canaccessvalue = function(v) return not rawequal(v, 12345) end
        return true
    end))
    rows = capture().queries
    assert(calls == 1 and rows[names[2]].status == "restricted-input")
end)

test("raw tuple arity and bytes stay bounded", function()
    local tuple = { "spell", 12345 }
    for i = 3, 20 do tuple[i] = string.rep("X", 400) end
    setup(function() return unpack(tuple) end, namespace(function() return unpack(tuple) end))
    local result = capture()
    for _, row in ipairs({ result.identity, result.queries[names[1]] }) do
        assert(row.n == 20 and #row.values == 16 and row.truncated)
        assert(#row.values[3].value == 256 and row.values[3].truncated)
    end
end)

test("parser rejects invalid slots before invoking APIs", function()
    for _, input in ipairs({ "", "word", "1.5", "9007199254740992" }) do
        setup(function() error("invalid slot reached producer") end, {})
        SlashCmdList.APICONTRACTPROBE("spell-metadata " .. input)
        assert(ApiContractProbeDB == nil)
    end
    setup(function(slot) assert(slot == -2); return "spell", 0 end, namespace(function(id)
        assert(id == 0); return nil
    end))
    assert(capture("spell-metadata -2 negative").queries[names[1]].n == 1)
end)

test("missing predicates fail closed and mode remains manual with ten snapshot cap", function()
    local produced = 0
    local function producer() produced = produced + 1; return "spell", 12345 end
    setup(producer, {})
    canaccessvalue = function() error(secret) end
    capture()
    assert(produced == 0)
    setup(producer, {})
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("spell-metadata 1")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and produced == 0)
    setup(producer, {})
    SlashCmdList.APICONTRACTPROBE("all")
    assert(produced == 0 and ApiContractProbeDB.captures[1].spellMetadata == nil)
    setup(producer, namespace(function() return false end))
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("spell-metadata 1") end
    assert(produced == 10 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)
print(string.format("%d/%d passed", passed, passed))
