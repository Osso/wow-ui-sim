local root = assert(arg[1], "addon directory required")
local passed = 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local names = { "Standard", "StandardNoRangeFill", "Center", "Reverse" }
local function setup(factory)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    UIParent = {}
    Enum = { StatusBarFillStyle = { Standard = 11, StandardNoRangeFill = 22, Center = 33, Reverse = 44 } }
    CreateFrame = factory
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("statusbar-fill sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual statusbar-fill mode absent")
    assert(ApiContractProbeDB.captures[1].label == "sample")
    return assert(ApiContractProbeDB.captures[1].statusbarFill)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("fresh hidden widget default precedes four published transitions", function()
    local creates, hidden, state, reads = 0, false, 91, 0
    local frame = {}
    frame.Hide = function(self) assert(rawequal(self, frame)); hidden = true end
    frame.GetFillStyle = function(self, ...)
        assert(hidden and rawequal(self, frame) and select("#", ...) == 0)
        reads = reads + 1
        return state, nil, reads
    end
    frame.SetFillStyle = function(self, value, ...)
        assert(hidden and reads > 0 and rawequal(self, frame) and select("#", ...) == 0)
        state = value
        return nil, false
    end
    setup(function(...)
        creates = creates + 1
        assert(select("#", ...) == 3)
        local kind, name, parent = ...
        assert(kind == "StatusBar" and name == nil and rawequal(parent, UIParent))
        return frame, nil
    end)
    local row = capture()
    assert(creates == 1 and hidden and reads == 9)
    assert(row.constructor.n == 2 and row.constructor.values[1].value == nil)
    assert(row.default.n == 3 and row.default.values[1].value == 91)
    for i, name in ipairs(names) do
        local sample = row.styles[i]
        assert(sample.name == name and sample.input.value == i * 11)
        assert(sample.setter.n == 2 and sample.setter.values[1].kind == "nil")
        for j = 1, 2 do
            assert(sample.getters[j].values[1].value == i * 11)
            assert(sample.getters[j].values[2].kind == "nil")
            assert(sample.getters[j].values[3].value == 1 + (i - 1) * 2 + j)
        end
    end
end)

test("missing and failing setters do not suppress independent getters", function()
    for _, setter in ipairs({ false, function() error(secret) end }) do
        local reads = 0
        setup(function() return { Hide = function() end, SetFillStyle = setter,
            GetFillStyle = function() reads = reads + 1; return nil, reads end } end)
        local row = capture()
        assert(reads == 9 and row.default.n == 2)
        for _, sample in ipairs(row.styles) do
            assert(sample.setter.status == (setter == false and "missing-api" or "call-error"))
            assert(#sample.getters == 2 and sample.getters[2].n == 2)
        end
    end
end)

test("only published accessible finite scalar enum inputs are passed", function()
    local writes, reads = 0, 0
    setup(function() return { Hide = function() end,
        SetFillStyle = function(_, v) writes = writes + 1; assert(v == 73) end,
        GetFillStyle = function() reads = reads + 1 end } end)
    Enum.StatusBarFillStyle = { Standard = 73, StandardNoRangeFill = secret, Center = {}, Reverse = nil }
    local row = capture()
    assert(writes == 1 and reads == 3)
    assert(row.styles[2].input.status == "restricted" and row.styles[2].setter == nil)
    assert(row.styles[3].input.kind == "table" and row.styles[3].setter == nil)
    assert(row.styles[4].input.kind == "nil" and row.styles[4].setter == nil)
end)

test("constructor and hide failures abort without storing a frame reference", function()
    setup(function() error(secret) end)
    assert(capture().constructor.status == "call-error")
    setup(nil)
    assert(capture().constructor.status == "missing-api")
    setup(function() return secret end)
    assert(capture().constructor.values[1].status == "restricted")
    for _, hide in ipairs({ false, function() error(secret) end }) do
        local reads = 0
        setup(function() return { Hide = hide, GetFillStyle = function() reads = reads + 1 end } end)
        local row = capture()
        assert(row.status == "visibility-unconfirmed" and row.default == nil and reads == 0)
        assert(row.constructor.values[1].value == nil and row.frame == nil)
    end
end)

test("guard object method enum and results before type or lookup", function()
    local frame, calls = {}, 0
    setup(function() return frame end)
    frame.Hide = function() end
    frame.GetFillStyle = function() return secret, nil end
    frame.SetFillStyle = secret
    Enum.StatusBarFillStyle.Center = secret
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, row = pcall(capture)
    type = original
    assert(ok, row)
    assert(row.default.values[1].status == "restricted" and row.default.values[2].kind == "nil")
    assert(row.styles[1].setter.status == "missing-api")
    setup(function() return frame end)
    frame.Hide = function() calls = calls + 1 end
    canaccessvalue = function(v) return not rawequal(v, frame) end
    assert(capture().constructor.values[1].status == "restricted" and calls == 0)
end)

test("revoked object access stops lookup and missing predicates prevent creation", function()
    local frame, revoked, lookups = newproxy(true), false, 0
    getmetatable(frame).__index = function(_, name)
        lookups = lookups + 1
        assert(not revoked)
        if name == "Hide" then return function() revoked = true end end
        error("lookup after hide")
    end
    setup(function() return frame end)
    canaccessvalue = function(v) return not (rawequal(v, frame) and revoked) end
    local row = capture()
    assert(row.default.status == "field-error" and lookups == 1)
    local creates = 0
    setup(function() creates = creates + 1 end)
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("statusbar-fill")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and creates == 0)
end)

test("manual only with sixteen positions and ten snapshot cap", function()
    local creates = 0
    local values = {}
    for i = 1, 20 do values[i] = i end
    local function factory()
        creates = creates + 1
        return { Hide = function() end, SetFillStyle = function() end,
            GetFillStyle = function() return unpack(values) end }
    end
    setup(factory)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(creates == 0 and ApiContractProbeDB.captures[1].statusbarFill == nil)
    setup(factory)
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("statusbar-fill") end
    assert(creates == 10 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local value = ApiContractProbeDB.captures[1].statusbarFill.default
    assert(value.n == 20 and #value.values == 16 and value.truncated)
end)
print(string.format("%d/%d passed", passed, passed))
