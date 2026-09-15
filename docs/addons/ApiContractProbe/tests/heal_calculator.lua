local root = assert(arg[1], "addon directory required")
local passed = 0
local names = { "GetHealAbsorbMode", "GetHealAbsorbClampMode", "GetDamageAbsorbClampMode", "GetHealAbsorbs", "GetDamageAbsorbs" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(factory)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    CreateUnitHealPredictionCalculator = factory
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("heal-calculator sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual heal-calculator mode absent")
    assert(ApiContractProbeDB.captures[1].label == "sample")
    return assert(ApiContractProbeDB.captures[1].healCalculator)
end
local function object(fn)
    local value = newproxy(true)
    local methods = {}
    for _, name in ipairs(names) do methods[name] = fn end
    getmetatable(value).__index = methods
    getmetatable(value).__eq = function() error("object compared") end
    getmetatable(value).__tostring = function() error("object stringified") end
    return value
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("two fresh constructors and repeated raw getters preserve arity without comparison", function()
    local constructions, calls = 0, 0
    setup(function(...)
        assert(select("#", ...) == 0)
        constructions = constructions + 1
        local id, invocation = constructions, 0
        local owned
        owned = object(function(self, ...)
            assert(rawequal(self, owned) and select("#", ...) == 0)
            calls, invocation = calls + 1, invocation + 1
            return id * 100 + invocation, false, nil
        end)
        return owned, nil
    end)
    local record = capture()
    assert(constructions == 2 and calls == 20 and #record.objects == 2)
    for i, row in ipairs(record.objects) do
        assert(row.constructor.n == 2 and row.constructor.values[1].kind == "userdata")
        assert(row.constructor.values[1].value == nil and row.constructor.values[2].kind == "nil")
        for j, name in ipairs(names) do
            local samples = row.methods[name]
            assert(#samples == 2)
            for k, sample in ipairs(samples) do
                assert(sample.n == 3 and sample.values[1].value == i * 100 + (j - 1) * 2 + k)
                assert(sample.values[2].value == false and sample.values[3].kind == "nil")
            end
        end
    end
end)

test("restricted constructor objects are guarded before type and lookup", function()
    setup(function() return secret end)
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type inspection"); return original(v) end
    local ok, record = pcall(capture)
    type = original
    assert(ok, record)
    for _, row in ipairs(record.objects) do
        assert(row.constructor.values[1].status == "restricted" and row.methods == nil)
    end
end)

test("missing restricted throwing and invalid constructors stay opaque", function()
    setup(nil)
    assert(capture().objects[1].constructor.status == "missing-api")
    local calls = 0
    local factory = function() calls = calls + 1 end
    setup(factory)
    canaccessvalue = function(v) return not rawequal(v, factory) end
    assert(capture().objects[1].constructor.status == "restricted" and calls == 0)
    setup(function() error(secret) end)
    for _, row in ipairs(capture().objects) do
        assert(row.constructor.status == "call-error" and row.methods == nil)
    end
    for _, value in ipairs({ false, 9, "not-object" }) do
        setup(function() return value end)
        assert(capture().objects[1].methods == nil)
    end
    setup(function() end)
    assert(capture().objects[1].constructor.n == 0)
end)

test("method lookup functions results and errors are guarded and opaque", function()
    local unsafeCalls = 0
    local restrictedMethod = function() unsafeCalls = unsafeCalls + 1 end
    local hostile = setmetatable({}, { __index = function() error("result indexed") end,
        __tostring = function() error("result stringified") end })
    setup(function()
        return setmetatable({}, { __index = function(_, key)
            if key == names[1] then error(secret) end
            if key == names[2] then return restrictedMethod end
            if key == names[3] then return nil end
            if key == names[4] then return function() error(secret) end end
            return function() return secret, hostile, nil end
        end })
    end)
    canaccessvalue = function(v) return not rawequal(v, secret) and not rawequal(v, restrictedMethod) end
    local original = type
    type = function(v)
        assert(not rawequal(v, secret) and not rawequal(v, restrictedMethod), "restricted type inspection")
        return original(v)
    end
    local ok, record = pcall(capture)
    type = original
    assert(ok, record)
    assert(unsafeCalls == 0)
    for _, row in ipairs(record.objects) do
        assert(row.methods[names[1]][1].status == "field-error")
        assert(row.methods[names[2]][1].status == "missing-api")
        assert(row.methods[names[3]][1].status == "missing-api")
        assert(row.methods[names[4]][1].status == "call-error")
        local result = row.methods[names[5]][2]
        assert(result.n == 3 and result.values[1].status == "restricted")
        assert(result.values[2].kind == "table" and result.values[2].fields == nil)
        assert(result.values[3].kind == "nil")
    end
end)

test("sixteen positions retain larger exact arity", function()
    local values = {}
    for i = 1, 20 do values[i] = i end
    setup(function() return object(function() return unpack(values) end), unpack(values) end)
    local row = capture().objects[1]
    assert(row.constructor.n == 21 and #row.constructor.values == 16 and row.constructor.truncated)
    local result = row.methods[names[1]][1]
    assert(result.n == 20 and #result.values == 16 and result.truncated and result.values[16].value == 16)
end)

test("missing or failing guards prevent construction", function()
    local calls = 0
    setup(function() calls = calls + 1 end)
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("heal-calculator")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(function() calls = calls + 1 end)
    canaccessvalue = function() error(secret) end
    assert(capture().objects[1].constructor.status == "restricted" and calls == 0)
end)

test("manual only and ten snapshot bound cap constructor and getter work", function()
    local constructors, getters = 0, 0
    local function factory()
        constructors = constructors + 1
        return object(function() getters = getters + 1; return 7 end)
    end
    setup(factory)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(constructors == 0 and ApiContractProbeDB.captures[1].healCalculator == nil)
    setup(factory)
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("heal-calculator") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(constructors == 20 and getters == 200)
end)
print(string.format("%d/%d passed", passed, passed))
