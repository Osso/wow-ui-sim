local root = assert(arg[1], "addon directory required")
local passed = 0
local names = { "FindBaseSpellByID", "FindFlyoutSlotBySpellID", "FindSpellOverrideByID" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(producer, namespace)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    GetActionInfo, C_SpellBook, C_Spell, C_UnitAuras = producer, namespace, nil, nil
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function namespace(fn)
    local result = {}
    for _, name in ipairs(names) do result[name] = fn end
    return result
end
local function capture(input)
    SlashCmdList.APICONTRACTPROBE(input or "spellbook-metadata 17 sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "manual spellbook-metadata mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].spellbookMetadata)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("actual slot producer supplies original fractional ID to three independent calls", function()
    local calls = 0
    setup(function(...)
        assert(select("#", ...) == 1 and (...) == 17)
        return "spell", 12345.5, nil, "extra"
    end, namespace(function(...)
        assert(select("#", ...) == 1 and (...) == 12345.5)
        calls = calls + 1
        return nil, false, 91
    end))
    local result = capture()
    assert(calls == 3 and result.slot == 17 and result.identity.n == 4)
    assert(result.identity.values[3].kind == "nil")
    for _, name in ipairs(names) do
        local row = result.queries[name]
        assert(row.n == 3 and row.values[1].kind == "nil" and row.values[2].value == false)
        assert(row.values[3].value == 91)
    end
    assert(ApiContractProbeDB.captures[1].label == "sample")
end)

test("invalid and restricted producer identities never reach queries", function()
    for _, pair in ipairs({ {}, { "item", 1 }, { "Spell", 1 }, { "spell", "1" },
        { "spell", math.huge }, { "spell", -math.huge }, { "spell", 0/0 },
        { secret, 1 }, { "spell", secret } }) do
        setup(function() return unpack(pair, 1, 2) end, namespace(function() error("query reached") end))
        local result = capture()
        for _, name in ipairs(names) do assert(result.queries[name].status ~= "observed") end
    end
    setup(nil, {})
    assert(capture().identity.status == "missing-api")
    setup(function() error(secret) end, {})
    assert(capture().identity.status == "call-error")
end)

test("lookup missing and call errors preserve independent peers and zero returns", function()
    for _, failure in ipairs({ "lookup", "missing", "call" }) do
        local calls = 0
        setup(function() return "spell", 42 end, setmetatable({}, { __index = function(_, name)
            if name == names[1] then
                if failure == "lookup" then error(secret) end
                if failure == "missing" then return nil end
                return function() error(secret) end
            end
            return function() calls = calls + 1 end
        end }))
        local rows = capture().queries
        assert(rows[names[1]].status ~= "observed" and calls == 2)
        assert(rows[names[2]].n == 0 and rows[names[3]].n == 0)
    end
end)

test("namespace function and result restrictions stay opaque before inspection", function()
    for _, ns in ipairs({ secret, namespace(secret), namespace(function() return secret, nil end) }) do
        setup(function() return "spell", 42 end, ns)
        local original = type
        type = function(v) assert(not rawequal(v, secret), "secret type inspected"); return original(v) end
        local ok, result = pcall(capture)
        type = original
        assert(ok, result)
        for _, name in ipairs(names) do
            local row = result.queries[name]
            assert(row.status ~= "observed" or (row.n == 2 and row.values[1].status == "restricted"))
        end
    end
end)

test("each lookup and both function guards can revoke original kind or ID", function()
    for _, stage in ipairs({ "lookup", "secret-guard", "access-guard" }) do
        for position = 1, 3 do
            for _, revokedValue in ipairs({ "spell", 42 }) do
                local revoked, calls = false, 0
                local functions = {}
                for index = 1, 3 do functions[index] = function() calls = calls + 1 end end
                setup(function() return "spell", 42 end, setmetatable({}, { __index = function(_, name)
                    for index, expected in ipairs(names) do
                        if name == expected then
                            if stage == "lookup" and index == position then revoked = true end
                            return functions[index]
                        end
                    end
                    error("unexpected API")
                end }))
                issecretvalue = function(v)
                    if stage == "secret-guard" and rawequal(v, functions[position]) then revoked = true end
                    return rawequal(v, secret)
                end
                canaccessvalue = function(v)
                    if stage == "access-guard" and rawequal(v, functions[position]) then revoked = true end
                    return not (revoked and rawequal(v, revokedValue))
                end
                local rows = capture().queries
                assert(calls == position - 1)
                assert(rows[names[position]].status == "restricted-input")
            end
        end
    end
end)

test("earlier query can revoke inputs before later query lookup", function()
    local revoked, calls = false, 0
    setup(function() return "spell", 42 end, namespace(function()
        calls = calls + 1; revoked = true; return nil
    end))
    canaccessvalue = function(v) return not (revoked and rawequal(v, 42)) end
    local rows = capture().queries
    assert(calls == 1 and rows[names[1]].n == 1)
    assert(rows[names[2]].status == "restricted-input" and rows[names[3]].status == "restricted-input")
end)

test("bounded tuples strings labels and ten snapshots limit queries to thirty", function()
    local values, produced, queried = {}, 0, 0
    for index = 1, 20 do values[index] = string.rep("X", 400) end
    setup(function() produced = produced + 1; return "spell", 42, unpack(values) end,
        namespace(function() queried = queried + 1; return unpack(values) end))
    for index = 1, 11 do SlashCmdList.APICONTRACTPROBE("spellbook-metadata 17 " .. string.rep("L", 200)) end
    assert(produced == 10 and queried == 30 and #ApiContractProbeDB.captures == 10)
    assert(ApiContractProbeDB.dropped == 1)
    local record = ApiContractProbeDB.captures[1]
    assert(#record.label == 128 and record.spellbookMetadata.identity.n == 22)
    local row = record.spellbookMetadata.queries[names[1]]
    assert(row.n == 20 and #row.values == 16 and row.truncated)
    assert(#row.values[1].value == 256 and row.values[1].truncated)
end)

test("slot validation guards fail closed and all and prior metadata exclude new calls", function()
    for _, slot in ipairs({ "", "word", "1.5", "9007199254740992" }) do
        setup(function() error("invalid slot reached producer") end, {})
        SlashCmdList.APICONTRACTPROBE("spellbook-metadata " .. slot)
        assert(ApiContractProbeDB == nil)
    end
    setup(function(slot) assert(slot == -2); return "spell", 0 end, namespace(function(id)
        assert(id == 0); return nil
    end))
    assert(capture("spellbook-metadata -2 negative").queries[names[1]].n == 1)
    setup(function() error("producer reached") end, {})
    canaccessvalue = function() error(secret) end
    assert(capture().identity.status == "missing-api")
    setup(function() error("producer reached") end, {})
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("spellbook-metadata 17")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
    setup(function() return "spell", 42 end, namespace(function() error("excluded query reached") end))
    SlashCmdList.APICONTRACTPROBE("all")
    assert(ApiContractProbeDB.captures[1].spellbookMetadata == nil)
    SlashCmdList.APICONTRACTPROBE("spell-metadata 17 previous")
    assert(ApiContractProbeDB.captures[2].spellMetadata.queries.IsSpellImportant.status == "field-error")
    assert(ApiContractProbeDB.captures[2].spellbookMetadata == nil)
end)
print("PASS " .. passed .. " spellbook metadata tests")
