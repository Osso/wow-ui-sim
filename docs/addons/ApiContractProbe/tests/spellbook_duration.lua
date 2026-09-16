local root = assert(arg[1], "addon directory required")
local passed = 0
local names = { "GetSpellBookItemChargeDuration", "GetSpellBookItemCooldownDuration",
    "GetSpellBookItemLossOfControlCooldownDuration" }
local methods = { "GetTotalDuration", "GetElapsedDuration", "GetRemainingDuration", "GetElapsedPercent",
    "GetRemainingPercent", "GetStartTime", "GetEndTime", "GetClockTime", "GetModRate", "HasExpired" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(producer, lookup, query)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture", "123" end, function() return 42 end
    GetActionInfo, C_SpellBook, C_Spell = producer, { FindSpellBookSlotForSpell = lookup }, nil
    UnitCastingDuration, UnitChannelDuration, UnitEmpoweredChannelDuration = nil, nil, nil
    for _, name in ipairs(names) do C_SpellBook[name] = query end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(input)
    SlashCmdList.APICONTRACTPROBE(input or "spellbook-duration 17 sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "manual spellbook-duration mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].spellbookDuration)
end
local function identity() return "spell", 12345.5 end
local function pair() return 7.25, -3.5 end
local function duration(fn)
    local object = {}
    for _, name in ipairs(methods) do
        object[name] = function(self, ...)
            assert(self == object and select("#", ...) == 0)
            return fn(name)
        end
    end
    return object
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("actual action identity supplies exact lookup branch and original pair", function()
    local reads, calls = 0, 0
    local object = duration(function() reads = reads + 1; return 12.5, nil, false end)
    setup(function(...)
        assert(select("#", ...) == 1 and (...) == 17)
        return "spell", 12345.5, nil, "extra"
    end, function(...)
        local args = { ... }
        assert(select("#", ...) == 5 and args[1] == 12345.5)
        assert(args[2] == false and args[3] == true and args[4] == true and args[5] == true)
        return 7.25, -3.5, nil, "extra"
    end)
    for index, name in ipairs(names) do C_SpellBook[name] = function(...)
        local args = { ... }
        assert(select("#", ...) == (index == 2 and 3 or 2))
        assert(args[1] == 7.25 and args[2] == -3.5)
        if index == 2 then assert(args[3] == false) end
        calls = calls + 1; return object, nil
    end end
    local result = capture()
    assert(result.slot == 17 and result.identity.n == 4 and result.producer.n == 4)
    assert(result.producer.values[3].kind == "nil" and reads == 30 and calls == 3)
    for _, name in ipairs(names) do
        local row = result.queries[name]
        assert(row.n == 2 and row.values[2].kind == "nil")
        assert(row.values[1].methods.GetTotalDuration.n == 3)
    end
end)

test("invalid or restricted identity blocks slot lookup", function()
    for _, values in ipairs({ {}, { "item", 1 }, { "spell", "1" }, { "spell", math.huge },
        { "spell", 0/0 }, { secret, 1 }, { "spell", secret } }) do
        local calls = 0
        setup(function() return unpack(values, 1, 2) end,
            function() calls = calls + 1 end, function() calls = calls + 1 end)
        local result = capture()
        assert(calls == 0 and result.producer.status ~= "observed")
    end
end)

test("lookup arity and nil positions preserved and invalid pairs never forwarded", function()
    for _, values in ipairs({ { n = 0 }, { n = 1, 7 }, { n = 3, [2] = 9, [3] = 10 },
        { n = 2, 7, "1" }, { n = 2, math.huge, 1 }, { n = 2, 7, 0/0 },
        { n = 2, secret, 1 }, { n = 2, 7, secret } }) do
        local calls = 0
        setup(identity, function() return unpack(values, 1, values.n) end,
            function() calls = calls + 1 end)
        local result = capture()
        assert(result.producer.n == values.n and calls == 0)
        for _, name in ipairs(names) do assert(result.queries[name].status ~= "observed") end
    end
    setup(identity, function() error(secret) end)
    assert(capture().producer.status == "call-error")
    setup(identity, nil)
    assert(capture().producer.status == "missing-api")
    setup(nil, pair)
    assert(capture().identity.status == "missing-api")
end)

test("lookup and function guards revoke original kind or ID", function()
    for _, stage in ipairs({ "lookup", "secret", "access" }) do
        for _, input in ipairs({ "spell", 12345.5 }) do
            local revoked, calls = false, 0
            local fn = function() calls = calls + 1; return pair() end
            setup(identity)
            C_SpellBook = setmetatable({}, { __index = function(_, name)
                if name == "FindSpellBookSlotForSpell" then
                    if stage == "lookup" then revoked = true end
                    return fn
                end
            end })
            issecretvalue = function(v)
                if stage == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if stage == "access" and rawequal(v, fn) then revoked = true end
                return not (revoked and rawequal(v, input))
            end
            assert(capture().producer.status == "restricted-input" and calls == 0)
        end
    end
end)

test("every query rechecks both pair values after lookup and function guards", function()
    for _, stage in ipairs({ "lookup", "secret", "access" }) do
        for position = 1, 3 do
            for _, input in ipairs({ 7.25, -3.5 }) do
                local revoked, calls, functions = false, 0, {}
                for index = 1, 3 do functions[index] = function() calls = calls + 1 end end
                setup(identity, pair)
                setmetatable(C_SpellBook, { __index = function(_, name)
                    for index, expected in ipairs(names) do
                        if name == expected then
                            if stage == "lookup" and index == position then revoked = true end
                            return functions[index]
                        end
                    end
                end })
                issecretvalue = function(v)
                    if stage == "secret" and rawequal(v, functions[position]) then revoked = true end
                    return rawequal(v, secret)
                end
                canaccessvalue = function(v)
                    if stage == "access" and rawequal(v, functions[position]) then revoked = true end
                    return not (revoked and rawequal(v, input))
                end
                local result = capture()
                assert(calls == position - 1 and result.queries[names[position]].status == "restricted-input")
            end
        end
    end
end)

test("query failures and nil holes do not suppress independent peers", function()
    for _, stage in ipairs({ "lookup", "missing", "call", "restricted" }) do
        local calls = 0
        setup(identity, pair)
        setmetatable(C_SpellBook, { __index = function(_, name)
            if name == names[1] then
                if stage == "lookup" then error(secret) end
                if stage == "missing" then return nil end
                if stage == "restricted" then return secret end
                return function() error(secret) end
            end
            return function() calls = calls + 1; return nil, false, nil end
        end })
        local result = capture()
        assert(calls == 2 and result.queries[names[1]].status ~= "observed")
        assert(result.queries[names[2]].n == 3 and result.queries[names[2]].values[3].kind == "nil")
    end
    local revoked, calls = false, 0
    setup(identity, pair, function() calls = calls + 1; revoked = true end)
    canaccessvalue = function(v) return not (revoked and rawequal(v, -3.5)) end
    local result = capture()
    assert(calls == 1 and result.queries[names[1]].n == 0)
    assert(result.queries[names[2]].status == "restricted-input")
end)

test("duration receiver guards prevent calls after lookup or function revocation", function()
    for _, stage in ipairs({ "lookup", "secret", "access" }) do
        local revoked, calls = false, 0
        local fn = function() calls = calls + 1 end
        local object = setmetatable({}, { __index = function()
            if stage == "lookup" then revoked = true end
            return fn
        end })
        setup(identity, pair, function() return object, secret end)
        issecretvalue = function(v)
            if stage == "secret" and rawequal(v, fn) then revoked = true end
            return rawequal(v, secret)
        end
        canaccessvalue = function(v)
            if stage == "access" and rawequal(v, fn) then revoked = true end
            return not (revoked and rawequal(v, object))
        end
        local result = capture()
        assert(calls == 0 and result.queries[names[1]].values[2].status == "restricted")
    end
end)

test("current objects collectible and cast retention remains independent", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(identity, pair, function()
        local object = duration(function() return 1 end)
        weak[#weak + 1] = object
        return object
    end)
    local castReads = 0
    local cast = duration(function() castReads = castReads + 1; return 2 end)
    UnitCastingDuration = function() return cast end
    SlashCmdList.APICONTRACTPROBE("cast-durations before")
    local before = castReads
    capture()
    assert(castReads == before)
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
    SlashCmdList.APICONTRACTPROBE("cast-durations after")
    assert(castReads > before)
end)

test("method tuple string label snapshot and 480 method bounds", function()
    local calls, reads, produced = 0, 0, 0
    local long = {}; for index = 1, 20 do long[index] = string.rep("x", 400) end
    local object = duration(function() reads = reads + 1; return unpack(long) end)
    local objects = {}; for index = 1, 20 do objects[index] = object end
    setup(identity, function() produced = produced + 1; return 7.25, -3.5, unpack(long) end,
        function() calls = calls + 1; return unpack(objects) end)
    for index = 1, 11 do capture("spellbook-duration 17 " .. string.rep("L", 200)) end
    assert(produced == 10 and calls == 30 and reads == 4800)
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local record = ApiContractProbeDB.captures[1]
    assert(#record.label == 128 and record.spellbookDuration.producer.truncated)
    local row = record.spellbookDuration.queries[names[1]]
    assert(row.n == 20 and #row.values == 16 and row.truncated)
    local method = row.values[1].methods.GetTotalDuration
    assert(method.n == 20 and #method.values == 16 and method.truncated)
    assert(#method.values[1].value == 256 and method.values[1].truncated)
end)

test("manual only slot validation and absent access guards", function()
    for _, slot in ipairs({ "", "word", "1.5", "9007199254740992" }) do
        setup(function() error("invalid slot reached") end)
        SlashCmdList.APICONTRACTPROBE("spellbook-duration " .. slot)
        assert(ApiContractProbeDB == nil)
    end
    setup(identity, pair, function() error("all reached new queries") end)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(ApiContractProbeDB.captures[1].spellbookDuration == nil)
    setup(identity, pair)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("spellbook-duration 17")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)
print("PASS " .. passed .. " spellbook duration tests")
