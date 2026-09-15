local root = assert(arg[1], "addon directory required")
local passed = 0
local names = { "GetSpellChargeDuration", "GetSpellLossOfControlCooldownDuration" }
local methods = { "GetTotalDuration", "GetElapsedDuration", "GetRemainingDuration", "GetElapsedPercent",
    "GetRemainingPercent", "GetStartTime", "GetEndTime", "GetClockTime", "GetModRate", "HasExpired" }
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
    UnitCastingDuration, UnitChannelDuration, UnitEmpoweredChannelDuration = nil, nil, nil
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(input)
    SlashCmdList.APICONTRACTPROBE(input or "spell-duration 17 sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "manual spell-duration mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].spellDuration)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end
local function namespace(fn) return { [names[1]] = fn, [names[2]] = fn } end
local function duration(fn)
    local object = {}
    for _, name in ipairs(methods) do
        object[name] = function(self, ...)
            assert(self == object and select("#", ...) == 0)
            return fn(name)
        end
    end
    return setmetatable(object, { __index = function(_, name) error("unexpected duration method " .. name) end })
end

test("selected slot produces original ID and ten read-only method observations", function()
    local queries, reads = 0, 0
    local object = duration(function(name)
        reads = reads + 1
        if name == "HasExpired" then return false end
        return 12.5, nil, "raw"
    end)
    setup(function(...)
        assert(select("#", ...) == 1 and (...) == 17)
        return "spell", 12345.5, nil, "extra"
    end, namespace(function(...)
        assert(select("#", ...) == 1 and (...) == 12345.5)
        queries = queries + 1
        return object, nil
    end))
    local result = capture()
    assert(result.slot == 17 and result.identity.n == 4 and result.identity.values[3].kind == "nil")
    assert(queries == 2 and reads == 20 and ApiContractProbeDB.captures[1].label == "sample")
    for _, name in ipairs(names) do
        local row = result.queries[name]
        assert(row.n == 2 and row.values[2].kind == "nil")
        local item = row.values[1]
        assert(item.retention == nil and item.observationRef == nil)
        assert(item.methods.GetTotalDuration.n == 3 and item.methods.GetTotalDuration.values[2].kind == "nil")
        assert(item.methods.HasExpired.values[1].value == false)
    end
end)

test("absent invalid and restricted producers skip queries without zero defaults", function()
    for _, input in ipairs({ { "item", 12 }, { "Spell", 12 }, { "spell", "12" }, { "spell", math.huge },
        { "spell", -math.huge }, { "spell", 0/0 }, { secret, 12 }, { "spell", secret }, {} }) do
        local calls = 0
        setup(function() return unpack(input, 1, 2) end, namespace(function() calls = calls + 1 end))
        local result = capture()
        assert(calls == 0)
        for _, name in ipairs(names) do assert(result.queries[name].status ~= "observed") end
    end
    setup(function() error(secret) end, {})
    assert(capture().identity.status == "call-error")
    setup(nil, {})
    assert(capture().identity.status == "missing-api")
end)

test("independent namespace missing lookup and call failures preserve second query", function()
    for _, failure in ipairs({ "missing", "lookup", "call" }) do
        local second = 0
        setup(function() return "spell", 42 end, setmetatable({}, { __index = function(_, name)
            if name == names[1] then
                if failure == "lookup" then error(secret) end
                if failure == "call" then return function() error(secret) end end
                return nil
            end
            assert(name == names[2])
            return function() second = second + 1; return nil, false end
        end }))
        local rows = capture().queries
        assert(rows[names[1]].status ~= "observed" and second == 1)
        assert(rows[names[2]].n == 2 and rows[names[2]].values[1].kind == "nil")
    end
    setup(function() return "spell", 42 end, namespace(function() end))
    assert(capture().queries[names[1]].n == 0)
end)

test("secret values stay opaque before type and duration method lookup", function()
    local object = duration(function(name)
        if name == "GetTotalDuration" then error(secret) end
        return secret, nil
    end)
    setup(function() return "spell", 42 end, namespace(function() return secret, object end))
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    for _, name in ipairs(names) do
        local row = result.queries[name]
        assert(row.values[1].status == "restricted")
        assert(row.values[2].methods.GetTotalDuration.status == "call-error")
        assert(row.values[2].methods.HasExpired.values[1].status == "restricted")
    end
    setup(function() return "spell", 42 end, secret)
    assert(capture().queries[names[2]].status == "field-error")
end)

test("producer input access rechecked after lookup function guard and first query", function()
    for _, stage in ipairs({ "lookup", "guard", "query" }) do
        local revoked, calls = false, 0
        local fn = function() calls = calls + 1; if stage == "query" then revoked = true end end
        setup(function() return "spell", 42 end, setmetatable({}, { __index = function()
            if stage == "lookup" then revoked = true end
            return fn
        end }))
        canaccessvalue = function(v)
            if stage == "guard" and rawequal(v, fn) then revoked = true end
            return not (revoked and rawequal(v, 42))
        end
        local rows = capture().queries
        assert(calls == (stage == "query" and 1 or 0))
        assert(rows[names[2]].status == "restricted-input")
    end
end)

test("current-only observations neither retain objects nor touch cast retention", function()
    local oldReads, newReads, castReads = 0, 0, 0
    local old = duration(function() oldReads = oldReads + 1; return 3 end)
    local fresh = duration(function() newReads = newReads + 1; return 2 end)
    local casting = duration(function() castReads = castReads + 1; return 1 end)
    setup(function() return "spell", 42 end, namespace(function() return old end))
    UnitCastingDuration = function(unit) if unit == "player" then return casting end end
    SlashCmdList.APICONTRACTPROBE("cast-durations before")
    local first = ApiContractProbeDB.captures[1].castDurations
    assert(first.retainedCount == 1 and first.capture == 1 and castReads == 10)
    capture()
    C_Spell = namespace(function() return fresh end)
    capture()
    assert(oldReads == 20 and newReads == 20 and castReads == 10)
    SlashCmdList.APICONTRACTPROBE("cast-durations after")
    local after = ApiContractProbeDB.captures[4].castDurations
    assert(after.capture == 2 and #after.previous == 1 and castReads == 30)
    assert(oldReads == 20 and newReads == 20)
end)

test("tuple and method results preserve arity with sixteen positions and byte bounds", function()
    local tuple = {}
    for i = 1, 20 do tuple[i] = string.rep("X", 400) end
    local object = duration(function() return unpack(tuple) end)
    local objects = {}
    for i = 1, 20 do objects[i] = object end
    setup(function() return "spell", 42, unpack(tuple) end, namespace(function() return unpack(objects) end))
    local result = capture()
    assert(result.identity.n == 22 and #result.identity.values == 16 and result.identity.truncated)
    local row = result.queries[names[1]]
    assert(row.n == 20 and #row.values == 16 and row.truncated)
    local method = row.values[1].methods.GetClockTime
    assert(method.n == 20 and #method.values == 16 and method.truncated)
    assert(#method.values[1].value == 256 and method.values[1].truncated)
end)

test("slot parser rejects invalid input and retains valid negative and zero IDs", function()
    for _, input in ipairs({ "", "word", "1.5", "9007199254740992" }) do
        setup(function() error("invalid slot reached producer") end, {})
        SlashCmdList.APICONTRACTPROBE("spell-duration " .. input)
        assert(ApiContractProbeDB == nil)
    end
    setup(function(slot) assert(slot == -2); return "spell", 0 end, namespace(function(id)
        assert(id == 0); return nil
    end))
    assert(capture("spell-duration -2 negative").queries[names[1]].n == 1)
end)

test("missing predicates fail closed and all excludes mode with ten snapshot cap", function()
    local produced, queries = 0, 0
    local function producer() produced = produced + 1; return "spell", 42 end
    setup(producer, {})
    canaccessvalue = function() error(secret) end
    capture()
    assert(produced == 0)
    setup(producer, {})
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("spell-duration 1")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and produced == 0)
    setup(producer, {})
    SlashCmdList.APICONTRACTPROBE("all")
    assert(produced == 0 and ApiContractProbeDB.captures[1].spellDuration == nil)
    setup(producer, namespace(function() queries = queries + 1 end))
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("spell-duration 1 " .. string.rep("L", 200)) end
    assert(produced == 10 and queries == 20 and #ApiContractProbeDB.captures == 10)
    assert(ApiContractProbeDB.dropped == 1 and #ApiContractProbeDB.captures[1].label == 128)
end)
print(string.format("%d/%d passed", passed, passed))
