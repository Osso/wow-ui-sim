local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local inputs = { 0, 999, 1000, 1234, 999999, 1000000, -1000, 1000000000 }
local function pack(...) return { n = select("#", ...), ... } end
local function hostile()
    return setmetatable({}, { __index = function() error("opaque lookup") end,
        __tostring = function() error("opaque stringify") end,
        __len = function() error("opaque length") end })
end
local function setup(producer, small, large)
    ApiContractProbeDB, SlashCmdList = nil, {}
    Enum, Constants = {}, {}
    C_StringUtil = { GetDefaultAbbreviationBreakpoints = producer }
    AbbreviateNumbers, AbbreviateLargeNumbers = small, large
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    GetLocale = function() return "fixture-locale" end
    issecretvalue = function() return false end
    canaccessvalue = function() return true end
    CreateAbbreviateConfig = function() error("config must not be constructed") end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    local index = ApiContractProbeDB and #ApiContractProbeDB.captures + 1 or 1
    SlashCmdList.APICONTRACTPROBE("abbreviation-options " .. (label or "sample"))
    assert(ApiContractProbeDB and ApiContractProbeDB.captures[index], "manual abbreviation-options mode absent")
    return assert(ApiContractProbeDB.captures[index].abbreviationOptions, "abbreviationOptions missing")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("original breakpoint identity, owned option shape, finite corpus and exact argument arity", function()
    local breakpoints, calls, options = hostile(), {}, nil
    local function abbreviation(lane, ...)
        local args = pack(...)
        calls[#calls + 1] = { lane = lane, args = args }
        if args.n == 2 then
            assert(type(args[2]) == "table" and rawequal(rawget(args[2], "breakpointData"), breakpoints))
            local count = 0
            for key in pairs(args[2]) do count = count + 1; assert(key == "breakpointData") end
            assert(count == 1 and getmetatable(args[2]) == nil)
            options = options or args[2]
            assert(rawequal(options, args[2]), "one fresh options object per capture")
        else assert(args.n == 1) end
        return lane, nil, false
    end
    local produced = 0
    setup(function(...)
        assert(select("#", ...) == 1 and (...) == nil)
        produced = produced + 1; return breakpoints, nil
    end, function(...) return abbreviation("small", ...) end,
        function(...) return abbreviation("large", ...) end)
    local result = capture()
    assert(produced == 1 and #calls == 32 and #result.samples == 8)
    assert(result.producer.n == 2 and result.producer.values[1].kind == "table")
    assert(result.producer.values[1].fields == nil and result.producer.values[2].kind == "nil")
    for index, number in ipairs(inputs) do
        local offset = (index - 1) * 4
        assert(calls[offset + 1].lane == "small" and calls[offset + 2].lane == "small")
        assert(calls[offset + 3].lane == "large" and calls[offset + 4].lane == "large")
        for step = 1, 4 do assert(calls[offset + step].args[1] == number) end
        for _, lane in ipairs({ result.samples[index].small, result.samples[index].large }) do
            for _, observation in ipairs({ lane.omitted, lane.options }) do
                assert(observation.n == 3 and observation.values[2].kind == "nil")
                assert(observation.values[3].value == false)
            end
        end
    end
end)

test("invalid, missing and throwing producers never suppress omitted controls", function()
    for _, variant in ipairs({ "missing", "error", "nil", "number", "userdata", "secret" }) do
        local breakpoints, controls, optionCalls = hostile(), 0, 0
        local function fn(...)
            if select("#", ...) == 1 then controls = controls + 1 else optionCalls = optionCalls + 1 end
        end
        local producer = function()
            if variant == "error" then error(breakpoints) end
            if variant == "nil" then return nil, breakpoints end
            if variant == "number" then return 7 end
            if variant == "userdata" then return newproxy(true) end
            return breakpoints
        end
        setup(producer, fn, fn)
        if variant == "missing" then C_StringUtil = nil end
        if variant == "secret" then issecretvalue = function(v) return rawequal(v, breakpoints) end end
        local result = capture()
        assert(controls == 16 and optionCalls == 0, variant)
        assert(result.samples[1].small.options.status ~= "observed")
    end
end)

test("producer namespace and function guards precede explicit nil invocation", function()
    for _, phase in ipairs({ "namespace", "function", "nil" }) do
        local calls, controls = 0, 0
        local producer = function() calls = calls + 1; return hostile() end
        local fn = function(...) assert(select("#", ...) == 1); controls = controls + 1 end
        setup(producer, fn, fn)
        local namespace = C_StringUtil
        if phase == "namespace" then
            C_StringUtil = setmetatable({}, { __index = function() error("namespace inspected") end })
            namespace = C_StringUtil
        end
        canaccessvalue = function(v)
            if phase == "namespace" and rawequal(v, namespace) then return false end
            if phase == "function" and rawequal(v, producer) then return false end
            if phase == "nil" and v == nil then return false end
            return true
        end
        capture()
        assert(calls == 0 and controls == 16, phase)
    end
end)

test("each option call rechecks breakpoint access after lookup and function guards", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        for target = 1, 16 do
            local breakpoints, revoked, lookups, optionCalls, controls = hostile(), false, 0, 0, 0
            local active, globalMeta
            local fn = function(...)
                if select("#", ...) == 2 then
                    assert(not revoked, "revoked breakpoint forwarded")
                    optionCalls = optionCalls + 1
                else controls = controls + 1 end
            end
            setup(function() return breakpoints end, fn, fn)
            local originalMeta = getmetatable(_G)
            rawset(_G, "AbbreviateNumbers", nil); rawset(_G, "AbbreviateLargeNumbers", nil)
            globalMeta = { __index = function(_, key)
                if key == "AbbreviateNumbers" or key == "AbbreviateLargeNumbers" then
                    lookups = lookups + 1
                    active = lookups == target * 2
                    if active and phase == "lookup" then revoked = true end
                    return fn
                end
                if originalMeta and type(originalMeta.__index) == "function" then return originalMeta.__index(_G, key) end
            end }
            setmetatable(_G, globalMeta)
            issecretvalue = function(v)
                if active and rawequal(v, fn) and phase == "secret" then revoked = true end
                return revoked and rawequal(v, breakpoints)
            end
            canaccessvalue = function(v)
                if active and rawequal(v, fn) and phase == "access" then revoked = true end
                return not (revoked and rawequal(v, breakpoints))
            end
            local ok, err = pcall(capture)
            setmetatable(_G, originalMeta)
            assert(ok, err)
            assert(controls == 16 and optionCalls == target - 1, phase .. ":" .. target)
        end
    end
end)

test("options access and owned field identity are checked before forwarding", function()
    for _, phase in ipairs({ "options", "field-secret", "field-replaced" }) do
        local original, replacement, owned, controls, optionCalls, changed = hostile(), hostile(), nil, 0, 0, false
        local function fn(...)
            local args = pack(...)
            if args.n == 1 then controls = controls + 1; return end
            assert(not changed, "invalid options forwarded")
            owned = args[2]; optionCalls = optionCalls + 1; changed = true
            if phase ~= "options" then rawset(owned, "breakpointData", replacement) end
        end
        setup(function() return original end, fn, fn)
        canaccessvalue = function(v)
            if phase == "options" and changed and rawequal(v, owned) then return false end
            if phase == "field-secret" and rawequal(v, replacement) then return false end
            return true
        end
        local result = capture()
        assert(controls == 16 and optionCalls == 1, phase)
        assert(result.samples[8].large.options.status ~= "observed")
    end
end)

test("literal input access is rechecked after global lookup and function guards", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        local revoked, calls, fn = false, 0
        fn = function(number) assert(number ~= 0 or not revoked, "revoked input forwarded"); calls = calls + 1 end
        setup(function() return hostile() end, fn, fn)
        local prior = getmetatable(_G)
        rawset(_G, "AbbreviateNumbers", nil)
        setmetatable(_G, { __index = function(_, key)
            if key == "AbbreviateNumbers" then if phase == "lookup" then revoked = true end; return fn end
            if prior and type(prior.__index) == "function" then return prior.__index(_G, key) end
        end })
        issecretvalue = function(v)
            if rawequal(v, fn) and phase == "secret" then revoked = true end
            return revoked and rawequal(v, 0)
        end
        canaccessvalue = function(v)
            if rawequal(v, fn) and phase == "access" then revoked = true end
            return not (revoked and rawequal(v, 0))
        end
        local ok, err = pcall(capture)
        setmetatable(_G, prior)
        assert(ok, err); assert(calls == 28, phase)
    end
end)

test("function failures, nil arity and output restriction remain independent", function()
    local secret, smallCalls = hostile(), 0
    setup(function() return hostile() end, function(...)
        smallCalls = smallCalls + 1
        if select("#", ...) == 1 then return end
        return nil, false, nil
    end, function() error(secret) end)
    local result = capture()
    assert(smallCalls == 16)
    assert(result.samples[1].small.omitted.n == 0)
    assert(result.samples[1].small.options.n == 3 and result.samples[1].small.options.values[3].kind == "nil")
    assert(result.samples[8].large.options.status == "call-error")
    setup(function() return hostile() end, function() return secret end, nil)
    issecretvalue = function(v) return rawequal(v, secret) end
    result = capture()
    assert(result.samples[1].small.options.values[1].status == "restricted")
    assert(result.samples[8].large.omitted.status == "missing-api")
end)

test("producer output observations cannot authorize a later revoked table", function()
    local breakpoints, revoked, controls, optionCalls = hostile(), false, 0, 0
    local trigger = hostile()
    local function fn(...)
        if select("#", ...) == 1 then controls = controls + 1 else optionCalls = optionCalls + 1 end
    end
    setup(function() return breakpoints, trigger end, fn, fn)
    canaccessvalue = function(v)
        if rawequal(v, trigger) then revoked = true end
        return not (revoked and rawequal(v, breakpoints))
    end
    capture(); assert(controls == 16 and optionCalls == 0)
end)

test("opaque objects are collectible, output tuples and strings are bounded", function()
    local weak = setmetatable({}, { __mode = "v" })
    local values = {}; for index = 1, 20 do values[index] = index end
    setup(function()
        local bp = hostile(); weak[1] = bp; return bp
    end, function(_, options)
        if options then weak[2] = options end
        local object = hostile(); weak[3] = object
        return object, string.rep("x", 300), nil
    end, function() return unpack(values) end)
    local result = capture()
    assert(result.samples[1].large.options.n == 20 and result.samples[1].large.options.truncated)
    assert(#result.samples[1].large.options.values == 16)
    assert(#result.samples[1].small.options.values[2].value == 256)
    assert(result.samples[1].small.options.values[1].fields == nil)
    collectgarbage(); collectgarbage()
    assert(weak[1] == nil and weak[2] == nil and weak[3] == nil, "raw objects retained")
end)

test("missing and throwing access APIs fail closed", function()
    local calls = 0
    local function fn() calls = calls + 1; return hostile() end
    setup(fn, fn, fn); issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("abbreviation-options")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(fn, fn, fn); canaccessvalue = function() error("guard") end
    capture(); assert(calls == 0)
end)

test("manual exclusion, shared capture cap and unchanged old abbreviation mode", function()
    local producerCalls, omitted, optionCalls, locales = 0, 0, 0, 0
    local function fn(...)
        if select("#", ...) == 1 then omitted = omitted + 1 else optionCalls = optionCalls + 1 end
        return "unchanged"
    end
    setup(function(...) assert(select("#", ...) == 1); producerCalls = producerCalls + 1; return hostile() end, fn, fn)
    GetLocale = function() locales = locales + 1; return "fixture-locale" end
    SlashCmdList.APICONTRACTPROBE("abbreviations old")
    capture(string.rep("L", 200))
    SlashCmdList.APICONTRACTPROBE("all")
    SlashCmdList.APICONTRACTPROBE("abbreviations after")
    assert(omitted == 116 and optionCalls == 16 and producerCalls == 1)
    assert(ApiContractProbeDB.captures[1].abbreviations.samples[25].large.values[1].value == "unchanged")
    assert(ApiContractProbeDB.captures[4].abbreviations.locale.values[1].value == "fixture-locale")
    assert(ApiContractProbeDB.captures[3].abbreviationOptions == nil)
    assert(#ApiContractProbeDB.captures[2].label == 128 and locales >= 2)
    setup(function() producerCalls = producerCalls + 1; return hostile() end, fn, fn)
    producerCalls, omitted, optionCalls = 0, 0, 0
    for _ = 1, 11 do SlashCmdList.APICONTRACTPROBE("abbreviation-options") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(producerCalls == 10 and omitted == 160 and optionCalls == 160)
end)

print(string.format("%d/%d passed", passed, passed + failed))
if failed > 0 then os.exit(1) end
