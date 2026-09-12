local root = assert(arg[1], "addon directory required")
local realRawset = rawset
local scenarios = 0

local function scenario(options)
    options = options or {}
    local Probe, secrets, objects = {}, {}, {}
    local calls = { cleanup = 0, predicates = 0, controls = 0, seeds = 0,
        secretCleanup = 0, maps = 0 }
    local function fakeSecret()
        local marker = setmetatable({}, { __tostring = function() error("secret stringified") end })
        secrets[marker] = true
        return marker
    end
    local health, clock = fakeSecret(), fakeSecret()
    local clearedSecretFixture = false
    _G.issecretvalue = function(value)
        if clearedSecretFixture and value == 0 then
            if options.secretZero then return true end
            if options.unknownClassifier then return nil end
            if options.failedClassifier then error(health) end
        end
        return secrets[value] == true
    end
    if options.missingClassifier then _G.issecretvalue = nil end
    _G.canaccessvalue = function(value)
        if options.inaccessibleZero and clearedSecretFixture and value == 0 then return false end
        return secrets[value] ~= true
    end
    if options.missingAccess then _G.canaccessvalue = nil end
    _G.issecure = function() return false end
    _G.securecallfunction = function(fn, ...)
        if not options.swallowErrors then return fn(...) end
        local result = { pcall(fn, ...) }
        if result[1] then return unpack(result, 2) end
    end
    if options.noWrapper then _G.securecallfunction = nil end
    _G.GetBuildInfo = function() return "12.1.5", "69594", "fixture", 120105 end
    _G.GetLocale = function() return "enUS" end
    _G.GetTime = function() return 123 end
    _G.time = function() return 1 end
    _G.print = function() end
    _G.Ptr125RemainingProbeDB = nil
    _G.UnitHealth = function() return options.noSecrets and 100 or health end
    _G.UnitPower = function() return options.noSecrets and 10 or health end
    _G.C_UnitAuras = { GetAuraDataByIndex = function(unit, index, filter)
        if options.missingAuras then
            if unit == "player" and filter == "HELPFUL" and index == 2 then
                return { expirationTime = 123 }
            end
            if unit ~= "target" or filter ~= "HELPFUL" or index ~= 2 then return nil end
        end
        return { expirationTime = options.noSecrets and 123 or clock }
    end }
    _G.rawset = function(t, key, value)
        if secrets[key] then
            calls.seeds = calls.seeds + 1
            assert(calls.controls > 0, "secret seeded before ordinary table controls")
            if options.blockSeed then error(health) end
            if options.dropSeed then return t end
        end
        return realRawset(t, key, value)
    end
    local function predicate(t, name)
        if secrets[next(t)] then
            calls.predicates = calls.predicates + 1
            return health
        end
        calls.controls = calls.controls + 1
        local count = 0
        for _ in pairs(t) do count = count + 1 end
        if name == "isempty" then return count == 0 end
        if name == "getcountinfo" then return count, false end
        return count
    end
    table.count = function(t) return predicate(t, "count") end
    table.getcountinfo = function(t) return predicate(t, "getcountinfo") end
    table.isempty = function(t) return predicate(t, "isempty") end
    _G.C_Timer = { NewTimedSignalMap = function(callback)
        assert(type(callback) == "function")
        calls.maps = calls.maps + 1
        local value, seeded, secretSeen = nil, false, false
        local controls, secretAttempts = 0, 0
        local methods = {}
        methods.SignalAt = function(_, key, when)
            assert(key == 1)
            if secrets[when] then
                assert(controls >= 2, "secret seeded before ordinary map controls")
                calls.seeds = calls.seeds + 1
                secretSeen = true
                if options.blockSeed then error(clock) end
                seeded = true
                value = options.dropSeed and 0 or when
            else
                assert(type(when) == "number" and when > GetTime(), "control clock must be future and ordinary")
                seeded, value = true, when
            end
        end
        methods.CancelAllSignals = function()
            calls.cleanup = calls.cleanup + 1
            if secretSeen then
                secretAttempts = secretAttempts + 1
                calls.secretCleanup = calls.secretCleanup + 1
                assert(secretAttempts <= 2, "cleanup retried more than once")
                if options.failedCleanup or (options.failCleanupOnce and secretAttempts == 1) then
                    error(clock)
                end
            end
            value, seeded = nil, false
            if secretSeen then clearedSecretFixture = true end
        end
        methods.GetSignalTime = function(_, key)
            assert(key == 1)
            if options.blockLookup and secretSeen and seeded then error(clock) end
            return value
        end
        methods.GetSignalCount = function()
            if not secretSeen then controls = controls + 1 end
            if secretSeen and not seeded then
                if options.secretCount then return health end
                if options.failedCount then error(clock) end
                if options.nonzeroCount then return 1 end
                if options.stringCount then return "0" end
                if options.nilCount then return nil end
            end
            return seeded and (secrets[value] and health or 1) or 0
        end
        methods.HasSignal = function() return seeded and (secrets[value] and health or true) or false end
        methods.GetNextSignal = function() return seeded and 1 or nil, value end
        local object = setmetatable({}, { __index = function(_, key)
            assert(not secretSeen, "method fetched after secret mutation")
            return methods[key]
        end })
        objects[object] = true
        return object
    end }
    assert(loadfile(root .. "/Core.lua"))("Ptr125RemainingProbe", Probe)
    assert(loadfile(root .. "/Secrets.lua"))("Ptr125RemainingProbe", Probe)
    local capture = Probe.capture
    Probe.capture = function(run, label, mode, fn, args, includeValues)
        if label:sub(1, 8) == "secrets:" then
            assert(includeValues == false, "secret flow requested raw values")
        end
        return capture(run, label, mode, fn, args, includeValues)
    end
    local run = Probe.run("secrets")
    local labels = {}
    for _, observation in ipairs(run.observations) do
        labels[observation.label] = labels[observation.label] or observation
    end
    local function serializable(value, seen)
        assert(not secrets[value], "raw secret persisted")
        assert(not objects[value], "runtime map persisted")
        local kind = type(value)
        assert(kind ~= "function" and kind ~= "userdata" and kind ~= "thread")
        if kind ~= "table" then return end
        assert(getmetatable(value) == nil, "runtime object persisted")
        assert(not seen[value], "cycle persisted")
        seen[value] = true
        for key, child in pairs(value) do
            assert(type(key) == "string" or type(key) == "number")
            serializable(child, seen)
        end
        seen[value] = nil
    end
    serializable(Ptr125RemainingProbeDB, {})
    local uncertainCleanup = options.failedCleanup or options.failCleanupOnce or options.secretCount
        or options.failedCount or options.nonzeroCount or options.stringCount or options.nilCount
        or options.inaccessibleZero or options.missingAccess or options.secretZero
        or options.unknownClassifier or options.failedClassifier
    if options.noSecrets or options.missingClassifier or options.blockSeed or options.dropSeed
        or options.blockLookup or uncertainCleanup then
        assert(run.status == "inconclusive", "unverified fixture was treated as evidence")
    else
        assert(run.status == "recorded", "supported fixture failed")
        assert(calls.predicates > 0 and calls.cleanup > 0)
        assert(labels["secrets:table:direct:count:direct"])
        assert(labels["secrets:map:direct:recovery:GetSignalCount:direct"])
        if not options.noWrapper then
            assert(labels["secrets:table:direct:count:securecallfunction"])
            assert(labels["secrets:map:direct:GetSignalCount:securecallfunction"])
        end
    end
    if not options.missingClassifier then
        for _, state in ipairs({ "empty", "populated" }) do
            for _, name in ipairs({ "count", "getcountinfo", "isempty" }) do
                local row = labels["secrets:table:control:" .. state .. ":" .. name .. ":direct"]
                assert(row and row.ok, "missing successful table control")
                assert(row.results[1].secret == false, "table control was not ordinary")
                assert(row.results[1].value == nil, "control persisted raw data")
            end
        end
    end
    if options.missingAuras then
        local later = labels["secrets:sample:aura:target:HELPFUL:2:expirationTime"]
        assert(later and later.ok and later.results[1].secret == true,
            "missing auras prevented discovery of a later secret timestamp")
        for _, observation in ipairs(run.observations) do
            if observation.label:match("^secrets:sample:aura:.*:expirationTime$") then
                assert(observation.ok, "missing aura generated an expirationTime error")
            end
        end
    end
    if options.noSecrets or options.missingClassifier then assert(calls.seeds == 0) end
    if options.blockSeed or options.dropSeed then assert(calls.predicates == 0) end
    if calls.maps > 0 then
        assert(calls.cleanup > 0)
        assert(labels["secrets:map:direct:control:empty:GetSignalCount:direct"], "missing empty map control")
        if not options.missingAccess then
            assert(labels["secrets:map:direct:control:populated:GetSignalCount:direct"], "missing populated map control")
        end
    end
    if options.failedCleanup or options.failCleanupOnce then
        assert(calls.secretCleanup == calls.maps * 2, "unverified cleanup did not retry exactly once")
        assert(not labels["secrets:map:direct:recovery:GetSignalCount:direct"], "unverified cleanup reported recovery")
        local row = labels["secrets:map:direct:cleanup:count"]
        local cancel = labels["secrets:map:direct:cleanup"]
        assert(row and cancel and row.mode == cancel.mode, "cleanup count used a different invocation path")
        assert(row.mode == (options.noWrapper and "direct" or "securecallfunction"))
        assert(row.results[1].secret == true, "failed cleanup count not observed as secret")
        if options.swallowErrors then
            assert(cancel.ok == true, "fixture did not swallow the failed cleanup")
        end
        local counts = {}
        for _, observation in ipairs(run.observations) do
            if observation.label == "secrets:map:direct:cleanup:count" then
                counts[#counts + 1] = observation
            end
        end
        assert(#counts == 2, "deferred cleanup count was not checked exactly once")
        if options.failCleanupOnce then
            local retried = counts[2].results[1]
            assert(retried.kind == "number" and retried.secret == false and retried.accessible == true)
        else
            assert(counts[2].results[1].secret == true, "failed retry claimed a public count")
        end
    elseif uncertainCleanup and calls.seeds > 0 then
        assert(not labels["secrets:map:direct:recovery:GetSignalCount:direct"], "unverified count reported recovery")
    end
    _G.rawset = realRawset
    scenarios = scenarios + 1
end

scenario({ missingAuras = true })
scenario({ swallowErrors = true, failedCleanup = true })
scenario({ swallowErrors = true, failCleanupOnce = true })
scenario({ noWrapper = true, failedCleanup = true })
scenario()
scenario({ noSecrets = true })
scenario({ missingClassifier = true })
scenario({ blockSeed = true })
scenario({ dropSeed = true })
scenario({ blockLookup = true })
scenario({ noWrapper = true })
scenario({ secretCount = true })
scenario({ swallowErrors = true, failedCount = true })
scenario({ nonzeroCount = true })
scenario({ stringCount = true })
scenario({ nilCount = true })
scenario({ inaccessibleZero = true })
scenario({ missingAccess = true })
scenario({ secretZero = true })
scenario({ unknownClassifier = true })
scenario({ failedClassifier = true })
io.write("Secrets controlled fixtures passed: ", scenarios, " scenarios; no native proof\n")
