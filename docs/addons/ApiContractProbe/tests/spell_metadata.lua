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
    GetActionInfo, C_Spell, Enum, C_UnitAuras = producer, namespace, nil, nil
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
local visibilityNames = { "RaidInCombat", "RaidOutOfCombat", "EnemyTarget" }
local function visibilitySetup(fn)
    local baseCalls = 0
    local api = namespace(function() baseCalls = baseCalls + 1; return true end)
    api.GetVisibilityInfo = fn
    setup(function() return "spell", 12345 end, api)
    Enum = { SpellAuraVisibilityType = { RaidInCombat = 41, RaidOutOfCombat = 73, EnemyTarget = 99,
        Unrequested = 400 } }
    return function() return baseCalls end
end

test("visibility uses only three fixed published values and preserves empty and nil tuples", function()
    local calls = 0
    local baseCalls = visibilitySetup(function(...)
        calls = calls + 1
        local id, value = ...
        assert(select("#", ...) == 2 and id == 12345)
        assert(value == ({ 41, 73, 99 })[calls])
        if calls == 1 then return end
        if calls == 2 then return false, nil, true end
        error(secret)
    end)
    local result = capture()
    assert(calls == 3 and baseCalls() == 7)
    assert(result.visibility.RaidInCombat.n == 0)
    local row = result.visibility.RaidOutOfCombat
    assert(row.n == 3 and row.values[1].value == false and row.values[2].kind == "nil")
    assert(row.values[3].value == true and row.input.value == 73)
    assert(result.visibility.EnemyTarget.status == "call-error")
end)

test("missing and invalid published visibility values have no fallback", function()
    for _, invalid in ipairs({ false, "41", math.huge, -math.huge, 0/0, secret, {} }) do
        local calls = 0
        local baseCalls = visibilitySetup(function() calls = calls + 1 end)
        Enum.SpellAuraVisibilityType.RaidInCombat = invalid
        Enum.SpellAuraVisibilityType.RaidOutOfCombat = nil
        local rows = capture().visibility
        assert(rows.RaidInCombat.status == "unavailable-enum")
        assert(rows.RaidOutOfCombat.status == "unavailable-enum")
        assert(calls == 1 and baseCalls() == 7)
    end
    for _, variant in ipairs({ "missing", "restricted", "throwing" }) do
        local baseCalls = visibilitySetup(function() error("unavailable enum reached query") end)
        if variant == "missing" then Enum = nil end
        if variant == "restricted" then Enum.SpellAuraVisibilityType = secret end
        if variant == "throwing" then Enum = setmetatable({}, { __index = function() error(secret) end }) end
        local result = capture()
        for _, name in ipairs(visibilityNames) do assert(result.visibility[name].status == "unavailable-enum") end
        assert(baseCalls() == 7)
    end
end)

test("visibility peer calls survive missing APIs and opaque failures", function()
    local calls, lookups = 0, 0
    visibilitySetup(nil)
    setmetatable(C_Spell, { __index = function(_, name)
        assert(name == "GetVisibilityInfo")
        lookups = lookups + 1
        if lookups == 1 then return nil end
        if lookups == 2 then error(secret) end
        return function() calls = calls + 1; return secret, nil end
    end })
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    assert(lookups == 3 and calls == 1)
    assert(result.visibility.RaidInCombat.status == "missing-api")
    assert(result.visibility.RaidOutOfCombat.status == "field-error")
    assert(result.visibility.EnemyTarget.n == 2 and result.visibility.EnemyTarget.values[1].status == "restricted")
end)

test("visibility rechecks spell and enum access after function guards and previous calls", function()
    for _, revokedValue in ipairs({ 12345, 41 }) do
        local revoked, calls = false, 0
        local fn = function(_, value)
            assert(value ~= 41, "revoked input passed")
            calls = calls + 1
        end
        visibilitySetup(fn)
        canaccessvalue = function(v)
            if rawequal(v, fn) then revoked = true end
            return not (revoked and rawequal(v, revokedValue))
        end
        local result = capture()
        assert(result.visibility.RaidInCombat.status == "restricted-input")
        assert(calls == (revokedValue == 12345 and 0 or 2))
    end
    local calls = 0
    visibilitySetup(function()
        calls = calls + 1
        canaccessvalue = function(v) return not rawequal(v, 12345) end
        return true
    end)
    local rows = capture().visibility
    assert(calls == 1 and rows.RaidOutOfCombat.status == "restricted-input")
    assert(rows.EnemyTarget.status == "restricted-input")
end)

test("visibility rejects invalid producers and keeps result bounds", function()
    visibilitySetup(function() error("invalid producer reached visibility") end)
    GetActionInfo = function() return "item", 12345 end
    local rows = capture().visibility
    for _, name in ipairs(visibilityNames) do assert(rows[name].status == "unavailable-input") end
    local tuple = {}
    for i = 1, 20 do tuple[i] = string.rep("Y", 400) end
    visibilitySetup(function() return unpack(tuple) end)
    rows = capture().visibility
    for _, name in ipairs(visibilityNames) do
        local row = rows[name]
        assert(row.n == 20 and #row.values == 16 and row.truncated)
        assert(#row.values[1].value == 256 and row.values[1].truncated)
    end
end)

test("visibility remains manual and capped at thirty calls", function()
    local calls = 0
    visibilitySetup(function() calls = calls + 1 end)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0)
    visibilitySetup(function() calls = calls + 1 end)
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("spell-metadata 17") end
    assert(calls == 30 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)
test("aura defensive uses original spell ID independently of failed spell queries", function()
    setup(function(slot) assert(slot == 17); return "spell", 12345.5 end, {})
    local calls = 0
    C_UnitAuras = { AuraIsBigDefensive = function(...)
        assert(select("#", ...) == 1 and (...) == 12345.5)
        calls = calls + 1
        return false, nil, true
    end }
    local result = capture()
    local row = assert(result.auraQueries, "aura queries absent").AuraIsBigDefensive
    assert(calls == 1 and row.n == 3 and row.values[1].value == false)
    assert(row.values[2].kind == "nil" and row.values[3].value == true)
    assert(result.queries[names[1]].status == "missing-api")
end)

test("aura namespace function and call failures preserve spell and visibility peers", function()
    for _, variant in ipairs({ "absent", "namespace", "lookup", "function", "error", "empty", "restricted-result" }) do
        local visibilityCalls = 0
        local baseCalls = visibilitySetup(function() visibilityCalls = visibilityCalls + 1; return true end)
        if variant == "namespace" then C_UnitAuras = secret
        elseif variant == "lookup" then C_UnitAuras = setmetatable({}, { __index = function() error(secret) end })
        elseif variant == "function" then C_UnitAuras = { AuraIsBigDefensive = secret }
        elseif variant == "error" then C_UnitAuras = { AuraIsBigDefensive = function() error(secret) end }
        elseif variant == "empty" then C_UnitAuras = { AuraIsBigDefensive = function() end }
        elseif variant == "restricted-result" then C_UnitAuras = { AuraIsBigDefensive = function() return secret, nil end } end
        local original = type
        type = function(v) assert(not rawequal(v, secret), "restricted inspection"); return original(v) end
        local ok, result = pcall(capture)
        type = original
        assert(ok, result)
        local row = assert(result.auraQueries).AuraIsBigDefensive
        assert(baseCalls() == 7 and visibilityCalls == 3)
        if variant == "empty" then assert(row.n == 0)
        elseif variant == "restricted-result" then assert(row.n == 2 and row.values[1].status == "restricted" and row.values[2].kind == "nil")
        elseif variant == "error" then assert(row.status == "call-error")
        elseif variant == "function" then assert(row.status == "missing-api")
        else assert(row.status == "field-error") end
    end
end)

test("aura rejects invalid and restricted original identities", function()
    for _, input in ipairs({ { "item", 12 }, { secret, 12 }, { "spell", secret }, { "spell", "12" }, { "spell", math.huge } }) do
        setup(function() return unpack(input) end, {})
        C_UnitAuras = setmetatable({}, { __index = function() error("invalid input lookup") end })
        local row = assert(capture().auraQueries).AuraIsBigDefensive
        assert(row.status == "restricted-input" or row.status == "unavailable-input")
    end
end)

test("aura rechecks kind and ID after namespace lookup and function guards", function()
    for _, target in ipairs({ "spell", 12345 }) do
        for _, phase in ipairs({ "lookup", "function" }) do
            local revoked, calls = false, 0
            visibilitySetup(function() return true end)
            local fn = function() calls = calls + 1 end
            C_UnitAuras = setmetatable({}, { __index = function()
                if phase == "lookup" then revoked = true end
                return fn
            end })
            canaccessvalue = function(v)
                if phase == "function" and rawequal(v, fn) then revoked = true end
                return not (revoked and rawequal(v, target))
            end
            assert(capture().auraQueries.AuraIsBigDefensive.status == "restricted-input")
            assert(calls == 0)
        end
    end
    visibilitySetup(function()
        canaccessvalue = function(v) return not rawequal(v, 12345) end
    end)
    C_UnitAuras = { AuraIsBigDefensive = function() error("revoked producer forwarded") end }
    assert(capture().auraQueries.AuraIsBigDefensive.status == "restricted-input")
end)

test("aura tuple strings labels snapshots and manual routing remain bounded", function()
    visibilitySetup(function() return true end)
    local calls, tuple = 0, {}
    for i = 1, 20 do tuple[i] = string.rep("A", 400) end
    C_UnitAuras = { AuraIsBigDefensive = function() calls = calls + 1; return unpack(tuple) end }
    local result = capture("spell-metadata 17 " .. string.rep("L", 200))
    local row = assert(result.auraQueries).AuraIsBigDefensive
    assert(row.n == 20 and #row.values == 16 and row.truncated)
    assert(#row.values[1].value == 256 and row.values[1].truncated)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for i = 1, 10 do SlashCmdList.APICONTRACTPROBE("spell-metadata 17") end
    assert(calls == 10 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    visibilitySetup(function() return true end)
    C_UnitAuras = { AuraIsBigDefensive = function() error("automatic aura query") end }
    SlashCmdList.APICONTRACTPROBE("all")
    assert(ApiContractProbeDB.captures[1].spellMetadata == nil)
end)
print(string.format("%d/%d passed", passed, passed))
