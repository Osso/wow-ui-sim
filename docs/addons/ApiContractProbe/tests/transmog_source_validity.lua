local root = assert(arg[1])
local passed, failed, excluded = 0, 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function forbidden() excluded = excluded + 1; error("excluded operation") end
local function setup(producer, query)
    ApiContractProbeDB, SlashCmdList, excluded = nil, {}, 0
    issecretvalue = function(value) return rawequal(value, secret) end
    canaccessvalue = function(value) return not rawequal(value, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_TransmogCollection = {
        GetNumTransmogSources = producer, IsValidTransmogSource = query,
        GetAppearanceSourceInfo = forbidden, SetSourceTypeFilter = forbidden,
        GetAllAppearanceSources = forbidden, SetAllSourceTypeFilters = forbidden,
        GetNumMaxCustomSets = function() return 2 end,
        GetCustomSets = function() return { 42 } end,
        GetCustomSetInfo = function(id) assert(id == 42); return "original name" end,
        IsValidCustomSetName = function(name) assert(name == "original name"); return true end,
    }
    C_Transmog, TransmogUtil = { GetSlotVisualInfo = forbidden }, { CreateTransmogLocation = forbidden }
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("transmog-source-validity " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "transmog-source-validity mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].transmogSourceValidity,
        "transmog-source-validity result absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok and excluded ~= 0 then ok, err = false, "excluded API invoked" end
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("vendor count produces exactly the bounded one-based indices", function()
    for _, count in ipairs({ 0, 1, 8, 20, 1e100 }) do
        local producers, calls = 0, 0
        setup(function(...)
            producers = producers + 1; assert(select("#", ...) == 0)
            return count, nil, "count producer"
        end, function(...)
            calls = calls + 1
            assert(select("#", ...) == 1 and (...) == calls)
            return calls % 2 == 0
        end)
        local r = capture()
        assert(producers == 1 and calls == math.min(count, 8) and #r.sources == calls)
        assert(r.status == "observed" and r.producer.n == 3)
        assert(r.producer.values[1].value == count and r.producer.values[2].kind == "nil")
        for i, observation in ipairs(r.sources) do
            assert(observation.input.value == i and observation.n == 1)
            assert(observation.values[1].value == (i % 2 == 0))
        end
    end
end)

test("invalid fractional negative infinite and inaccessible counts do not derive indices", function()
    local invalid = { false, true, "8", {}, function() end, -1, -0.5, 0.5, 8.25,
        math.huge, -math.huge, 0/0, secret }
    for i = 0, #invalid do
        local value = invalid[i]
        setup(function() return value end, forbidden)
        local r = capture()
        assert(#r.sources == 0)
        assert(r.status == (rawequal(value, secret) and "restricted-input" or "unavailable-input"))
    end
end)

test("producer errors missing APIs and invalid first return preserve evidence", function()
    setup(function() error(secret) end, forbidden)
    local r = capture(); assert(r.producer.status == "call-error" and #r.sources == 0)
    setup(nil, forbidden)
    r = capture(); assert(r.producer.status == "missing-api" and #r.sources == 0)
    for _, namespace in ipairs({ secret, setmetatable({}, { __index = function() error(secret) end }) }) do
        setup(nil, forbidden); C_TransmogCollection = namespace
        r = capture(); assert(r.producer.status == "field-error" and #r.sources == 0)
    end
    setup(function() return nil, 8, nil end, forbidden)
    r = capture(); assert(r.producer.n == 3 and r.producer.values[1].kind == "nil" and #r.sources == 0)
    setup(function() end, forbidden)
    r = capture(); assert(r.producer.n == 0 and #r.sources == 0)
end)

test("query errors and missing functions never suppress independent later indices", function()
    local calls = 0
    setup(function() return 8 end, function(index)
        calls = calls + 1
        if index == 1 then return end
        if index == 2 then return nil end
        if index == 3 then error(secret) end
        return nil, secret, false, nil
    end)
    local r = capture()
    assert(calls == 8 and r.sources[1].n == 0 and r.sources[2].n == 1)
    assert(r.sources[3].status == "call-error" and r.sources[4].n == 4)
    assert(r.sources[4].values[1].kind == "nil" and r.sources[4].values[2].status == "restricted")
    assert(r.sources[4].values[3].value == false and r.sources[4].values[4].kind == "nil")
    local lookups = 0
    setup(function() return 8 end, nil)
    setmetatable(C_TransmogCollection, { __index = function(_, key)
        assert(key == "IsValidTransmogSource"); lookups = lookups + 1
        if lookups == 1 then error(secret) end
        if lookups == 2 then return secret end
        if lookups == 3 then return false end
        return function(index) return index end
    end })
    r = capture()
    assert(lookups == 8 and r.sources[1].status == "field-error")
    assert(r.sources[2].status == "missing-api" and r.sources[3].status == "missing-api")
    assert(r.sources[8].values[1].value == 8)
end)

test("original count is rechecked after every query lookup and function guard", function()
    for _, phase in ipairs({ "namespace-secret", "namespace-access", "lookup", "function-secret", "function-access" }) do
        for position = 1, 8 do
            local lookups, calls, revoked, armed = 0, 0, false, false
            local fn = function(index)
                assert(not revoked, "revoked count allowed a derived index")
                calls = calls + 1; assert(index == calls); return index
            end
            setup(function() armed = true; return 20 end, nil)
            local namespace = C_TransmogCollection
            setmetatable(namespace, { __index = function(_, key)
                assert(key == "IsValidTransmogSource")
                if phase ~= "namespace-secret" and phase ~= "namespace-access" then lookups = lookups + 1 end
                if phase == "lookup" and lookups == position then revoked = true end
                return fn
            end })
            issecretvalue = function(value)
                if armed and phase == "namespace-secret" and rawequal(value, namespace) then
                    lookups = lookups + 1; if lookups == position then revoked = true end
                end
                if phase == "function-secret" and rawequal(value, fn) and lookups == position then revoked = true end
                return rawequal(value, secret)
            end
            canaccessvalue = function(value)
                if armed and phase == "namespace-access" and rawequal(value, namespace) then
                    lookups = lookups + 1; if lookups == position then revoked = true end
                end
                if phase == "function-access" and rawequal(value, fn) and lookups == position then revoked = true end
                return not rawequal(value, secret) and not (revoked and rawequal(value, 20))
            end
            local r = capture()
            assert(calls == position - 1 and #r.sources == 8)
            for i = position, 8 do assert(r.sources[i].status == "restricted-input") end
        end
    end
end)

test("fixed indices are guarded after function checks and cannot revoke count unnoticed", function()
    for position = 1, 8 do
        for _, revokeCount in ipairs({ false, true }) do
            local calls, lookups, armed, revoked = {}, 0, false, false
            local fn = function(index)
                assert(not (revoked and (revokeCount or index == position)), "restricted input forwarded")
                calls[#calls + 1] = index; return true
            end
            setup(function() return 20 end, nil)
            setmetatable(C_TransmogCollection, { __index = function(_, key)
                assert(key == "IsValidTransmogSource"); lookups = lookups + 1; return fn
            end })
            canaccessvalue = function(value)
                if rawequal(value, fn) and lookups == position then armed = true end
                if armed and rawequal(value, position) then revoked = true end
                if revoked and rawequal(value, revokeCount and 20 or position) then return false end
                return not rawequal(value, secret)
            end
            local r = capture()
            assert(r.sources[position].status == "restricted-input")
            assert(#calls == (revokeCount and position - 1 or 7))
        end
    end
end)

test("producer serialization and preceding query observations can revoke original count", function()
    local revoked, output = false, {}
    setup(function() return 20, output end, forbidden)
    canaccessvalue = function(value)
        if rawequal(value, output) then revoked = true end
        return not rawequal(value, secret) and not (revoked and rawequal(value, 20))
    end
    local r = capture(); assert(r.status == "restricted-input" and #r.sources == 0)
    revoked = false
    local calls = 0
    setup(function() return 20 end, function(index) calls = calls + 1; assert(index == 1); return output end)
    canaccessvalue = function(value)
        if rawequal(value, output) then revoked = true end
        return not rawequal(value, secret) and not (revoked and rawequal(value, 20))
    end
    r = capture(); assert(calls == 1 and r.sources[2].status == "restricted-input")
end)

test("fresh replacement is observed without re-reading or replacing the producer count", function()
    local producerCalls, firstCalls, laterCalls = 0, 0, 0
    setup(function() producerCalls = producerCalls + 1; return 8 end, function(index)
        firstCalls = firstCalls + 1; assert(index == 1)
        C_TransmogCollection.GetNumTransmogSources = forbidden
        C_TransmogCollection.IsValidTransmogSource = function(nextIndex)
            laterCalls = laterCalls + 1; return nextIndex
        end
        error(secret)
    end)
    local r = capture()
    assert(producerCalls == 1 and firstCalls == 1 and laterCalls == 7)
    assert(r.sources[1].status == "call-error" and r.sources[8].values[1].value == 8)
end)

test("producer and query tuple bounds strings labels snapshots and total calls", function()
    local producerCalls, queryCalls, values = 0, 0, {}
    for i = 1, 20 do values[i] = string.rep("x", 300) end
    setup(function() producerCalls = producerCalls + 1; return 20, unpack(values) end,
        function() queryCalls = queryCalls + 1; return unpack(values) end)
    local r = capture(string.rep("l", 200))
    assert(r.producer.n == 21 and r.producer.truncated and #r.producer.values == 16)
    assert(r.sources[1].n == 20 and r.sources[1].truncated and #r.sources[1].values == 16)
    assert(#r.sources[1].values[1].value == 256 and r.sources[1].values[1].truncated)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(producerCalls == 10 and queryCalls == 80 and #ApiContractProbeDB.captures == 10)
    assert(ApiContractProbeDB.dropped == 1)
end)

test("opaque results and invalid count objects are never inspected or retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() return 8 end, function(index)
        local object = newproxy(true)
        getmetatable(object).__index, getmetatable(object).__tostring = forbidden, forbidden
        weak[index] = object
        return object, { nested = object }, function() forbidden() end
    end)
    assert(capture().sources[1].values[1].kind == "userdata")
    collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
    setup(function()
        local object = newproxy(true)
        getmetatable(object).__index, getmetatable(object).__tostring = forbidden, forbidden
        weak[1] = object; return object
    end, forbidden)
    assert(capture().status == "unavailable-input")
    collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)

test("guards fail closed and old custom-set/all modes do not enumerate sources", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        setup(function() return 1 end, function() return true end)
        assert(capture().sources[1].status == "observed")
        _G[guard] = nil
        SlashCmdList.APICONTRACTPROBE("transmog-source-validity missing")
        assert(ApiContractProbeDB.captures[2].status == "missing-access-api")
        setup(function() return 1 end, forbidden)
        _G[guard] = function() error(secret) end
        assert(capture().producer.status == "field-error")
    end
    local producers, queries = 0, 0
    setup(function() producers = producers + 1; return 1 end,
        function(index) queries = queries + 1; assert(index == 1); return true end)
    SlashCmdList.APICONTRACTPROBE("custom-set-names before")
    assert(ApiContractProbeDB.captures[1].customSetNames.sets.values[1].entries[1].validation.values[1].value)
    capture()
    SlashCmdList.APICONTRACTPROBE("all context")
    SlashCmdList.APICONTRACTPROBE("custom-set-names after")
    assert(producers == 1 and queries == 1 and ApiContractProbeDB.captures[3].transmogSourceValidity == nil)
    assert(ApiContractProbeDB.captures[4].customSetNames.maximum[1].values[1].value == 2)
end)

print(string.format("%d passed, %d failed transmog-source-validity fixtures", passed, failed))
if failed > 0 then os.exit(1) end
