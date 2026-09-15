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
print(string.format("%d/%d passed", passed, passed))
