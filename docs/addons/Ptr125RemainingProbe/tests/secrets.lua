local root = assert(arg[1], "addon directory required")
local realRawset = rawset
local scenarios = 0

local function scenario(options)
    options = options or {}
    local Probe, secrets, calls = {}, {}, { cleanup = 0, predicates = 0, seeds = 0 }
    local function fakeSecret()
        local marker = setmetatable({}, { __tostring = function() error("secret stringified") end })
        secrets[marker] = true
        return marker
    end
    local health, clock = fakeSecret(), fakeSecret()
    _G.issecretvalue = options.missingClassifier and nil or function(value) return secrets[value] == true end
    if options.missingClassifier then _G.issecretvalue = nil end
    _G.canaccessvalue = function(value) return secrets[value] ~= true end
    _G.issecure = function() return false end
    _G.securecallfunction = options.noWrapper and nil or function(fn, ...) return fn(...) end
    if options.noWrapper then _G.securecallfunction = nil end
    _G.GetBuildInfo = function() return "12.1.5", "69594", "fixture", 120105 end
    _G.GetLocale = function() return "enUS" end
    _G.time = function() return 1 end
    _G.print = function() end
    _G.Ptr125RemainingProbeDB = nil
    _G.UnitHealth = function() return options.noSecrets and 100 or health end
    _G.UnitPower = function() return options.noSecrets and 10 or health end
    _G.C_UnitAuras = { GetAuraDataByIndex = function()
        return { expirationTime = options.noSecrets and 123 or clock }
    end }
    _G.rawset = function(t, key, value)
        if secrets[key] then
            calls.seeds = calls.seeds + 1
            if options.blockSeed then error(health) end
            if options.dropSeed then return t end
        end
        return realRawset(t, key, value)
    end
    local function predicate(t)
        calls.predicates = calls.predicates + 1
        assert(secrets[next(t)], "predicate ran without verified secret key")
        return health
    end
    table.count, table.getcountinfo, table.isempty = predicate, predicate, predicate
    _G.C_Timer = { NewTimedSignalMap = function(callback)
        assert(type(callback) == "function")
        local value, seeded = nil, false
        local methods = {}
        methods.SignalAt = function(_, key, when)
            assert(key == 1 and secrets[when], "non-native clock fixture")
            calls.seeds = calls.seeds + 1
            if options.blockSeed then error(clock) end
            seeded = true
            value = options.dropSeed and 0 or when
        end
        methods.CancelAllSignals = function()
            calls.cleanup = calls.cleanup + 1
            value, seeded = nil, false
        end
        methods.GetSignalTime = function(_, key)
            assert(key == 1)
            if options.blockLookup and seeded then error(clock) end
            return value
        end
        methods.GetSignalCount = function() return seeded and health or 0 end
        methods.HasSignal = function() return seeded and health or false end
        methods.GetNextSignal = function() return seeded and 1 or nil, value end
        return setmetatable({}, { __index = function(_, key)
            assert(not seeded, "method fetched after secret mutation")
            return methods[key]
        end })
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
    for _, observation in ipairs(run.observations) do labels[observation.label] = true end
    local function serializable(value, seen)
        assert(not secrets[value], "raw secret persisted")
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
    if options.noSecrets or options.missingClassifier or options.blockSeed or options.dropSeed or options.blockLookup then
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
    if options.noSecrets or options.missingClassifier then assert(calls.seeds == 0) end
    if options.blockSeed or options.dropSeed then assert(calls.predicates == 0) end
    if not options.noSecrets and not options.missingClassifier then assert(calls.cleanup > 0) end
    _G.rawset = realRawset
    scenarios = scenarios + 1
end

scenario()
scenario({ noSecrets = true })
scenario({ missingClassifier = true })
scenario({ blockSeed = true })
scenario({ dropSeed = true })
scenario({ blockLookup = true })
scenario({ noWrapper = true })
io.write("Secrets controlled fixtures passed: ", scenarios, " scenarios; no native proof\n")
