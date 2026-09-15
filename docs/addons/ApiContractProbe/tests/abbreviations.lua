local root = assert(arg[1], "addon directory required")
local passed = 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(large, small)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    GetLocale = function() return "frFR" end
    AbbreviateLargeNumbers, AbbreviateNumbers = large, small
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("abbreviations sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual abbreviations mode absent")
    assert(ApiContractProbeDB.captures[1].label == "sample")
    return assert(ApiContractProbeDB.captures[1].abbreviations)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("shared numeric corpus and omitted options retain localized bytes and nil arity", function()
    local calls = 0
    local function abbreviated(...)
        assert(select("#", ...) == 1, "options must be omitted")
        calls = calls + 1
        return "1\194\160234,5", nil, false
    end
    setup(abbreviated, abbreviated)
    local result = capture()
    assert(result.locale.values[1].value == "frFR" and calls == 50 and #result.samples == 25)
    SlashCmdList.APICONTRACTPROBE("numbers")
    local numbers = ApiContractProbeDB.captures[2].numbers.samples
    for i, row in ipairs(result.samples) do
        assert(row.input == numbers[i].input)
        for _, value in ipairs({ row.large, row.small }) do
            assert(value.n == 3 and value.values[1].value == "1\194\160234,5")
            assert(value.values[2].kind == "nil" and value.values[3].value == false)
        end
    end
end)

test("missing functions and opaque errors remain independent", function()
    setup(nil, function() error(secret) end)
    local result = capture()
    assert(result.samples[1].large.status == "missing-api")
    assert(result.samples[25].small.status == "call-error")
    setup(function() end, function() return nil end)
    result = capture()
    assert(result.samples[1].large.n == 0 and result.samples[1].small.n == 1)
    assert(result.samples[1].small.values[1].kind == "nil")
end)

test("restricted functions results and locale are guarded before inspection", function()
    setup(secret, function() return secret end)
    GetLocale = function() return secret end
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    assert(result.samples[1].large.status == "missing-api")
    assert(result.samples[1].small.values[1].status == "restricted")
    assert(result.locale.values[1].status == "restricted")
end)

test("objects stay opaque and tuples and strings are bounded", function()
    local hostile = setmetatable({}, { __index = function() error("object inspected") end,
        __tostring = function() error("object stringified") end })
    local values = {}
    for i = 1, 20 do values[i] = i end
    setup(function() return unpack(values) end, function() return hostile, string.rep("x", 300) end)
    local row = capture().samples[1]
    assert(row.large.n == 20 and #row.large.values == 16 and row.large.truncated)
    assert(row.small.values[1].kind == "table" and row.small.values[1].fields == nil)
    assert(#row.small.values[2].value == 256 and row.small.values[2].truncated)
end)

test("missing and throwing access guards fail closed", function()
    local calls = 0
    local function fn() calls = calls + 1 end
    setup(fn, fn)
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("abbreviations")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(fn, fn)
    canaccessvalue = function() error(secret) end
    assert(capture().samples[1].large.status == "missing-api" and calls == 0)
end)

test("manual only and ten snapshot cap bound pure calls", function()
    local calls, locales = 0, 0
    local function fn() calls = calls + 1; return "value" end
    setup(fn, fn)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and ApiContractProbeDB.captures[1].abbreviations == nil)
    setup(fn, fn)
    GetLocale = function() locales = locales + 1; return "deDE" end
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("abbreviations") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(calls == 500 and locales == 10)
end)
print(string.format("%d/%d passed", passed, passed))
