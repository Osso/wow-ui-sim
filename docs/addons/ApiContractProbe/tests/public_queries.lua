local root = assert(arg[1], "addon directory required")
local passed = 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(rules, delves)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    C_GameRules, C_DelvesUI = rules, delves
    C_Housing, C_EncounterTimeline, C_InstanceEncounter = nil, nil, nil
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("public-queries sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual public-queries mode absent")
    assert(ApiContractProbeDB.captures[1].label == "sample")
    return assert(ApiContractProbeDB.captures[1].publicQueries)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("four zero-argument calls retain repeated arity and nil positions", function()
    local rules, delves = 0, 0
    setup({ IsPersonalResourceDisplayEnabled = function(...)
        assert(select("#", ...) == 0)
        rules = rules + 1
        return rules == 1, nil, rules
    end }, { GetLockedTextForCompanion = function(...)
        assert(select("#", ...) == 0, "companion argument must be omitted")
        delves = delves + 1
        if delves == 1 then return nil end
        return "locked", nil
    end, IsTraitTreeForCompanion = function() error("excluded query") end })
    local result = capture()
    assert(rules == 2 and delves == 2)
    assert(result.personalResourceDisplay[1].n == 3)
    assert(result.personalResourceDisplay[1].values[1].value == true)
    assert(result.personalResourceDisplay[2].values[1].value == false)
    assert(result.personalResourceDisplay[2].values[2].kind == "nil")
    assert(result.omittedCompanion[1].n == 1 and result.omittedCompanion[1].values[1].kind == "nil")
    assert(result.omittedCompanion[2].n == 2 and result.omittedCompanion[2].values[1].value == "locked")
end)

test("missing and throwing namespaces do not gate the other lane", function()
    local calls = 0
    setup(nil, { GetLockedTextForCompanion = function() calls = calls + 1 end })
    local result = capture()
    assert(result.personalResourceDisplay[1].status == "field-error")
    assert(calls == 2 and result.omittedCompanion[2].n == 0)
    setup({ IsPersonalResourceDisplayEnabled = function() return false end },
        setmetatable({}, { __index = function() error(secret) end }))
    result = capture()
    assert(result.personalResourceDisplay[2].values[1].value == false)
    assert(result.omittedCompanion[2].status == "field-error")
    setup({ IsPersonalResourceDisplayEnabled = function() error(secret) end }, {})
    result = capture()
    assert(result.personalResourceDisplay[2].status == "call-error")
    assert(result.omittedCompanion[2].status == "missing-api")
end)

test("restricted namespace function and results are never inspected", function()
    setup(secret, { GetLockedTextForCompanion = function() return secret end })
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    assert(result.personalResourceDisplay[1].status == "field-error")
    assert(result.omittedCompanion[2].values[1].status == "restricted")
    setup({ IsPersonalResourceDisplayEnabled = secret }, {})
    result = capture()
    assert(result.personalResourceDisplay[2].status == "missing-api")
end)

test("repeated calls recheck function access", function()
    local calls, revoked = 0, false
    local fn
    fn = function() calls = calls + 1; revoked = true; return true end
    setup({ IsPersonalResourceDisplayEnabled = fn }, {})
    canaccessvalue = function(v) return not (revoked and rawequal(v, fn)) end
    local result = capture()
    assert(calls == 1 and result.personalResourceDisplay[2].status == "missing-api")
end)

test("scalar output bounds and opaque objects", function()
    local values = {}
    for i = 1, 20 do values[i] = i end
    local hostile = setmetatable({}, { __index = function() error("inspected") end,
        __tostring = function() error("stringified") end })
    setup({ IsPersonalResourceDisplayEnabled = function() return unpack(values) end },
        { GetLockedTextForCompanion = function() return hostile, string.rep("x", 300), math.huge end })
    local result = capture()
    local rules, delves = result.personalResourceDisplay[1], result.omittedCompanion[1]
    assert(rules.n == 20 and #rules.values == 16 and rules.truncated)
    assert(delves.values[1].kind == "table" and delves.values[1].fields == nil)
    assert(#delves.values[2].value == 256 and delves.values[2].truncated)
    assert(delves.values[3].status == "nonfinite")
end)

test("missing and throwing guards fail closed", function()
    local calls = 0
    local function fn() calls = calls + 1 end
    setup({ IsPersonalResourceDisplayEnabled = fn }, { GetLockedTextForCompanion = fn })
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("public-queries")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup({ IsPersonalResourceDisplayEnabled = fn }, { GetLockedTextForCompanion = fn })
    canaccessvalue = function() error(secret) end
    local result = capture()
    assert(result.personalResourceDisplay[2].status == "field-error")
    assert(result.omittedCompanion[2].status == "field-error" and calls == 0)
end)

test("manual only and ten snapshot cap", function()
    local calls = 0
    local function fn() calls = calls + 1; return false end
    setup({ IsPersonalResourceDisplayEnabled = fn }, { GetLockedTextForCompanion = fn })
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and ApiContractProbeDB.captures[1].publicQueries == nil)
    setup({ IsPersonalResourceDisplayEnabled = fn }, { GetLockedTextForCompanion = fn })
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("public-queries") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1 and calls == 40)
end)
local publicStateQueries = {
    { "C_Housing", "IsHousingMarketShopEnabled", "housingMarketShopEnabled" },
    { "C_EncounterTimeline", "GetCurrentTime", "encounterTimelineCurrentTime" },
    { "C_InstanceEncounter", "IsEncounterLimitingResurrections", "encounterLimitingResurrections" },
    { "C_InstanceEncounter", "IsEncounterSuppressingRelease", "encounterSuppressingRelease" },
    { "C_InstanceEncounter", "ShouldShowTimelineForEncounter", "showTimelineForEncounter" },
}
local function installPublicState(factory)
    C_Housing, C_EncounterTimeline, C_InstanceEncounter = {}, {}, {}
    for index, query in ipairs(publicStateQueries) do
        _G[query[1]][query[2]] = factory(index)
    end
end

test("five new named outputs preserve independent zero-argument tuples", function()
    setup({}, {})
    local calls = {}
    installPublicState(function(index)
        calls[index] = 0
        return function(...)
            assert(select("#", ...) == 0)
            calls[index] = calls[index] + 1
            return index, nil, calls[index]
        end
    end)
    local result = capture()
    for index, query in ipairs(publicStateQueries) do
        local lane = assert(result[query[3]], "missing public-state output: " .. query[3])
        assert(calls[index] == 2)
        for repetition = 1, 2 do
            assert(lane[repetition].n == 3)
            assert(lane[repetition].values[1].value == index)
            assert(lane[repetition].values[2].kind == "nil")
            assert(lane[repetition].values[3].value == repetition)
        end
    end
end)

test("each new function failure leaves all peers independent", function()
    for failed, query in ipairs(publicStateQueries) do
        for _, failure in ipairs({ "missing", "secret", "lookup", "call" }) do
            local oldCalls, newCalls = 0, 0
            local function old() oldCalls = oldCalls + 1 end
            setup({ IsPersonalResourceDisplayEnabled = old }, { GetLockedTextForCompanion = old })
            installPublicState(function() return function() newCalls = newCalls + 1; return nil end end)
            local namespace = _G[query[1]]
            if failure == "missing" then namespace[query[2]] = nil
            elseif failure == "secret" then namespace[query[2]] = secret
            elseif failure == "lookup" then
                namespace[query[2]] = nil
                setmetatable(namespace, { __index = function(_, key)
                    assert(key == query[2]); error(secret)
                end })
            else namespace[query[2]] = function() error(secret) end end
            local result = capture()
            local expected = failure == "lookup" and "field-error"
                or failure == "call" and "call-error" or "missing-api"
            assert(oldCalls == 4 and newCalls == 8)
            for index, peer in ipairs(publicStateQueries) do
                for repetition = 1, 2 do
                    local observation = result[peer[3]][repetition]
                    if index == failed then assert(observation.status == expected)
                    else assert(observation.n == 1 and observation.values[1].kind == "nil") end
                end
            end
        end
    end
end)

test("new namespace absence restriction and lookup errors preserve other namespaces", function()
    for _, name in ipairs({ "C_Housing", "C_EncounterTimeline", "C_InstanceEncounter" }) do
        for _, failure in ipairs({ "missing", "secret", "lookup" }) do
            local calls = 0
            setup({}, {})
            installPublicState(function() return function() calls = calls + 1 end end)
            _G[name] = failure == "secret" and secret or failure == "lookup"
                and setmetatable({}, { __index = function() error(secret) end }) or nil
            local original = type
            type = function(value) assert(not rawequal(value, secret)); return original(value) end
            local ok, result = pcall(capture)
            type = original
            assert(ok, result)
            assert(calls == (name == "C_InstanceEncounter" and 4 or 8))
            for _, query in ipairs(publicStateQueries) do
                if query[1] == name then assert(result[query[3]][2].status == "field-error") end
            end
        end
    end
end)

test("new query results stay bounded opaque and recheck function access", function()
    for selected, query in ipairs(publicStateQueries) do
        setup({}, {})
        local calls, revoked, chosen = 0, false
        installPublicState(function(index)
            if index ~= selected then return function() return nil end end
            chosen = function()
                calls = calls + 1
                revoked = true
                local values = { secret, string.rep("z", 300), math.huge }
                for i = 4, 20 do values[i] = i end
                return unpack(values)
            end
            return chosen
        end)
        canaccessvalue = function(value)
            return not rawequal(value, secret) and not (revoked and rawequal(value, chosen))
        end
        local result = capture()[query[3]]
        assert(calls == 1 and result[2].status == "missing-api")
        assert(result[1].n == 20 and #result[1].values == 16 and result[1].truncated)
        assert(result[1].values[1].status == "restricted")
        assert(#result[1].values[2].value == 256 and result[1].values[2].truncated)
        assert(result[1].values[3].status == "nonfinite")
    end
end)

test("all seven queries are manual only with 140 total calls and no mutations", function()
    local calls = 0
    local function fn(...) assert(select("#", ...) == 0); calls = calls + 1 end
    local function install()
        setup({ IsPersonalResourceDisplayEnabled = fn }, { GetLockedTextForCompanion = fn })
        installPublicState(function() return fn end)
        for _, name in ipairs({ "C_Housing", "C_EncounterTimeline", "C_InstanceEncounter" }) do
            setmetatable(_G[name], { __index = function() error("unexpected API") end,
                __newindex = function() error("namespace mutation") end })
        end
    end
    install()
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and ApiContractProbeDB.captures[1].publicQueries == nil)
    install()
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("public-queries " .. string.rep("l", 200)) end
    assert(calls == 140 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
end)
print(string.format("%d/%d passed", passed, passed))
