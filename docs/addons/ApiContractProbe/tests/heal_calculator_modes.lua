local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local getters = { "GetHealAbsorbMode", "GetHealAbsorbClampMode", "GetDamageAbsorbClampMode" }
local groups = {
    { "UnitHealAbsorbMode", "SetHealAbsorbMode", getters[1], { "ReducedByIncomingHeals", "Total" } },
    { "UnitHealAbsorbClampMode", "SetHealAbsorbClampMode", getters[2], { "CurrentHealth", "MaximumHealth" } },
    { "UnitDamageAbsorbClampMode", "SetDamageAbsorbClampMode", getters[3],
        { "MissingHealth", "MissingHealthWithoutIncomingHeals", "MaximumHealth" } },
}
local function pack(...) return { n = select("#", ...), ... } end
local function setup()
    ApiContractProbeDB, SlashCmdList, Enum = nil, {}, {}
    issecretvalue = function() return false end
    canaccessvalue = function() return true end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    local trace, methods, values, count = {}, {}, {}, 0
    for _, group in ipairs(groups) do
        Enum[group[1]] = {}
        for _, key in ipairs(group[4]) do
            count = count + 1
            Enum[group[1]][key] = count + 100.25
            values[#values + 1] = count + 100.25
        end
    end
    local object = newproxy(true)
    getmetatable(object).__index = methods
    getmetatable(object).__eq = function() error("receiver compared") end
    getmetatable(object).__tostring = function() error("receiver stringified") end
    local function add(name)
        methods[name] = function(self, ...)
            assert(rawequal(self, object), "original receiver required")
            trace[#trace + 1] = { name = name, args = pack(...) }
            return #trace, nil, false
        end
    end
    for _, group in ipairs(groups) do add(group[2]); add(group[3]) end
    add("Reset"); add("ResetPredictedValues")
    local constructors = 0
    CreateUnitHealPredictionCalculator = function(...)
        assert(select("#", ...) == 0)
        constructors = constructors + 1
        return object, nil
    end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
    return object, methods, trace, values, function() return constructors end
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("heal-calculator-modes " .. (label or "sample"))
    local records = ApiContractProbeDB and ApiContractProbeDB.captures
    assert(records and #records > 0, "manual heal-calculator-modes mode absent")
    return assert(records[#records].healCalculatorModes, "mode output absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("one original calculator and exact 25-method sequence", function()
    local _, _, trace, values, constructors = setup()
    local row = capture()
    assert(constructors() == 1 and #trace == 25)
    assert(row.constructor.n == 2 and row.constructor.values[2].kind == "nil")
    local position = 0
    local function expect(name, value)
        position = position + 1
        assert(trace[position].name == name, "method sequence")
        assert(trace[position].args.n == (value and 1 or 0), "method arity")
        if value then assert(trace[position].args[1] == value, "original published enum") end
    end
    for _, name in ipairs(getters) do expect(name) end
    local index = 0
    for _, group in ipairs(groups) do
        for _, key in ipairs(group[4]) do
            index = index + 1
            expect(group[2], values[index]); expect(group[3])
            assert(row.modes[index].enum == group[1] and row.modes[index].key == key)
            assert(row.modes[index].setter.n == 3 and row.modes[index].getter.n == 3)
            assert(row.modes[index].getter.values[2].kind == "nil")
        end
    end
    for _, name in ipairs({ "Reset", "ResetPredictedValues" }) do
        expect(name)
        for _, getter in ipairs(getters) do expect(getter) end
    end
    assert(position == 25 and #row.modes == 7 and #row.resets == 2)
end)

test("setter and reset errors never suppress readbacks or peers", function()
    local _, methods, trace = setup()
    for _, group in ipairs(groups) do methods[group[2]] = function() error({}) end end
    methods.Reset = function() error({}) end
    methods.ResetPredictedValues = nil
    local row = capture()
    assert(#trace == 16)
    for _, mode in ipairs(row.modes) do
        assert(mode.setter.status == "call-error" and mode.getter.status == "observed")
    end
    assert(row.resets[1].result.status == "call-error")
    assert(row.resets[2].result.status == "missing-api")
    for _, reset in ipairs(row.resets) do
        for _, name in ipairs(getters) do assert(reset.getters[name].status == "observed") end
    end
end)

test("missing invalid and inaccessible enum keys do not use numeric fallbacks", function()
    local _, _, trace = setup()
    local secret = newproxy(true)
    Enum.UnitHealAbsorbMode.ReducedByIncomingHeals = secret
    Enum.UnitHealAbsorbMode.Total = nil
    Enum.UnitHealAbsorbClampMode.CurrentHealth = "1"
    Enum.UnitHealAbsorbClampMode.MaximumHealth = math.huge
    Enum.UnitDamageAbsorbClampMode.MissingHealth = 0 / 0
    Enum.UnitDamageAbsorbClampMode.MissingHealthWithoutIncomingHeals = false
    Enum.UnitDamageAbsorbClampMode.MaximumHealth = -math.huge
    issecretvalue = function(v) return rawequal(v, secret) end
    local row = capture()
    assert(#trace == 18)
    for _, mode in ipairs(row.modes) do
        assert(mode.setter.status ~= "observed" and mode.getter.status == "observed")
    end
end)

test("enum containers are checked before member lookup", function()
    for _, level in ipairs({ "root", "group" }) do
        setup()
        local forbidden = setmetatable({}, { __index = function() error("forbidden lookup") end })
        if level == "root" then Enum = forbidden else Enum.UnitHealAbsorbMode = forbidden end
        canaccessvalue = function(v) return not rawequal(v, forbidden) end
        local row = capture()
        assert(row.modes[1].setter.status ~= "observed")
        assert(row.resets[2].result.status == "observed")
    end
end)

test("receiver revocation during every lookup or function guard blocks invocation", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        for target = 1, 25 do
            local object, methods, trace = setup()
            local lookups, revoked, current = 0, false, nil
            getmetatable(object).__index = function(_, key)
                lookups = lookups + 1
                current = methods[key]
                if phase == "lookup" and lookups == target then revoked = true end
                return current
            end
            issecretvalue = function(v)
                if phase == "secret" and lookups == target and rawequal(v, current) then revoked = true end
                return revoked and rawequal(v, object)
            end
            canaccessvalue = function(v)
                if phase == "access" and lookups == target and rawequal(v, current) then revoked = true end
                return not (revoked and rawequal(v, object))
            end
            capture()
            assert(#trace == target - 1, phase .. " target " .. target)
        end
    end
end)

test("each setter rejects enum revoked by lookup and function guards", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        for target = 1, 7 do
            local object, methods, trace, values = setup()
            local lookups, revoked, current = 0, false, nil
            local targetLookup = 3 + (target - 1) * 2 + 1
            getmetatable(object).__index = function(_, key)
                lookups = lookups + 1; current = methods[key]
                if phase == "lookup" and lookups == targetLookup then revoked = true end
                return current
            end
            issecretvalue = function(v)
                if phase == "secret" and lookups == targetLookup and rawequal(v, current) then revoked = true end
                return revoked and rawequal(v, values[target])
            end
            canaccessvalue = function(v)
                if phase == "access" and lookups == targetLookup and rawequal(v, current) then revoked = true end
                return not (revoked and rawequal(v, values[target]))
            end
            local row = capture()
            assert(row.modes[target].setter.status == "restricted-input")
            assert(row.modes[target].getter.status == "observed" and #trace == 24)
        end
    end
end)

test("revoked calculator is not type-inspected after constructor tuple observation", function()
    local object = setup()
    local revoked = false
    local extra = {}
    CreateUnitHealPredictionCalculator = function() return object, extra end
    canaccessvalue = function(v)
        if rawequal(v, extra) then revoked = true end
        return not (revoked and rawequal(v, object))
    end
    local original = type
    type = function(v)
        assert(not (revoked and rawequal(v, object)), "revoked type inspection")
        return original(v)
    end
    local ok, row = pcall(capture)
    type = original
    assert(ok, row)
    assert(row.status == "restricted-input" and row.modes == nil)
end)

test("constructor failure invalid first values and missing access prevent methods", function()
    for _, value in ipairs({ false, 3, "not-object" }) do
        local _, _, trace = setup()
        CreateUnitHealPredictionCalculator = function() return value, {} end
        local row = capture()
        assert(row.status == "unavailable-input" and #trace == 0)
    end
    setup(); CreateUnitHealPredictionCalculator = function() error({}) end
    assert(capture().constructor.status == "call-error")
    setup(); CreateUnitHealPredictionCalculator = nil
    assert(capture().constructor.status == "missing-api")
    local _, _, _, _, constructors = setup()
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("heal-calculator-modes")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and constructors() == 0)
end)

test("method failures and restricted outputs remain opaque with exact arity", function()
    local object, methods = setup()
    local secret = newproxy(true)
    getmetatable(secret).__index = function() error("secret inspected") end
    issecretvalue = function(v) return rawequal(v, secret) end
    methods[getters[1]] = function() return secret, nil, false end
    methods[getters[2]] = function() end
    methods[getters[3]] = function() error(secret) end
    local row = capture()
    assert(row.baseline[getters[1]].n == 3)
    assert(row.baseline[getters[1]].values[1].status == "restricted")
    assert(row.baseline[getters[1]].values[2].kind == "nil")
    assert(row.baseline[getters[2]].n == 0 and row.baseline[getters[3]].status == "call-error")
    getmetatable(object).__index = function() error(secret) end
    row = capture()
    assert(row.baseline[getters[1]].status == "field-error")
end)

test("sixteen positions and 256-byte strings do not retain produced objects", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup()
    CreateUnitHealPredictionCalculator = function()
        local result = setmetatable({}, { __index = function() error("opaque output traversed") end })
        local object = setmetatable({}, { __index = function()
            return function()
                local values = { string.rep("x", 300), result }
                for i = 3, 20 do values[i] = i end
                return unpack(values, 1, 20)
            end
        end })
        weak[1], weak[2] = object, result
        return object
    end
    local row = capture()
    local sample = row.baseline[getters[1]]
    assert(sample.n == 20 and #sample.values == 16 and sample.truncated)
    assert(#sample.values[1].value == 256 and sample.values[1].truncated)
    assert(sample.values[2].kind == "table" and sample.values[2].fields == nil)
    collectgarbage("collect"); collectgarbage("collect")
    assert(weak[1] == nil and weak[2] == nil)
end)

test("manual-only ten snapshots cap calls at 260 and labels at 128 bytes", function()
    local _, _, trace, _, constructors = setup()
    SlashCmdList.APICONTRACTPROBE("all")
    assert(constructors() == 0)
    ApiContractProbeDB = nil
    for _ = 1, 11 do capture(string.rep("l", 180)) end
    assert(constructors() == 10 and #trace == 250)
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
