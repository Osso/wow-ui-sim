-- Standalone Lua 5.1 recorder fixtures, not assertions about native damage classes.
-- Cached GarrisonInfoDocumentation: C_Garrison.GetAutoCombatDamageClassValues()
-- returns table<AutoCombatDamageClassString>: damageClassValue:number, locString:cstring.
local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local errorPayload = "fixture error payload must not be saved"

local function setup(api)
    local env = setmetatable({}, { __index = _G })
    env._G, env.SlashCmdList = env, {}
    env.C_Garrison = { GetAutoCombatDamageClassValues = api }
    env.issecretvalue = function(v) return rawequal(v, secret) end
    env.canaccessvalue = function(v) return not rawequal(v, secret) end
    env.GetBuildInfo = function() return "fixture-version", "987654", "fixture-date", 120100 end
    env.GetLocale = function() return "fixture-locale" end
    env.time, env.print = function() return 42 end, function() end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then setfenv(assert(loadfile(root .. "/" .. line)), env)() end
    end
    toc:close()
    return env
end

local function capture(env, label)
    local before = env.ApiContractProbeDB and #env.ApiContractProbeDB.captures or 0
    env.SlashCmdList.APICONTRACTPROBE("auto-combat-damage-classes " .. (label or "sample"))
    assert(env.ApiContractProbeDB and #env.ApiContractProbeDB.captures == before + 1,
        "manual auto-combat-damage-classes mode absent")
    local record = env.ApiContractProbeDB.captures[before + 1]
    return assert(record.autoCombatDamageClasses, "autoCombatDamageClasses capture absent"), record
end

local function assertNotSaved(value, forbidden, seen)
    for _, raw in ipairs(forbidden) do
        assert(not rawequal(value, raw), "raw value or error payload retained")
    end
    if type(value) ~= "table" then return end
    seen = seen or {}
    if seen[value] then return end
    seen[value] = true
    for key, item in pairs(value) do
        assertNotSaved(key, forbidden, seen)
        assertNotSaved(item, forbidden, seen)
    end
end

local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("manual capture calls documented API once with no arguments", function()
    local calls = 0
    local rows = { { damageClassValue = 37.5, locString = "synthetic-second" },
        { damageClassValue = -4, locString = "synthetic-first" } }
    local env = setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 0, "query received arguments")
        return rows
    end)
    assert(env.SLASH_APICONTRACTPROBE1 == "/apicontract")
    local result, record = capture(env, "user-context-not-a-profile-inference")
    assert(calls == 1 and record.mode == "auto-combat-damage-classes")
    assert(record.label == "user-context-not-a-profile-inference")
    local q = result.query
    assert(q.status == "observed" and q.n == 1)
    assert(q.values[1].status == "observed" and q.values[1].kind == "table")
    local entries = q.values[1].entries
    assert(#entries == 8)
    assert(entries[1].fields.damageClassValue.value == 37.5)
    assert(entries[1].fields.locString.value == "synthetic-second")
    assert(entries[2].fields.damageClassValue.value == -4)
    assert(entries[2].fields.locString.value == "synthetic-first")
    for i = 3, 8 do assert(entries[i].kind == "nil") end
    assertNotSaved(record, { secret, rows, rows[1], rows[2], errorPayload })
end)

test("build and locale are observed rather than inferred from label", function()
    local env = setup(function() return {} end)
    local buildCalls, localeCalls = 0, 0
    for _, sample in ipairs({ { "build-a", "101", "locale-a" }, { "build-b", "202", "locale-b" } }) do
        env.GetBuildInfo = function(...)
            buildCalls = buildCalls + 1; assert(select("#", ...) == 0)
            return sample[1], sample[2], nil, 120100
        end
        env.GetLocale = function(...)
            localeCalls = localeCalls + 1; assert(select("#", ...) == 0)
            return sample[3]
        end
        local result, record = capture(env, "same-user-context")
        assert(record.client.status == "observed" and record.client.n == 4)
        assert(record.client.values[1].value == sample[1] and record.client.values[2].value == sample[2])
        assert(record.client.values[3].kind == "nil" and record.client.values[4].value == 120100)
        assert(result.locale.status == "observed" and result.locale.n == 1)
        assert(result.locale.values[1].value == sample[3])
    end
    assert(buildCalls == 2 and localeCalls == 2)
end)

test("zero returns explicit nil and extra tuple positions stay distinct", function()
    local env = setup(function() return end)
    local q = capture(env).query
    assert(q.status == "observed" and q.n == 0 and #q.values == 0)
    env.C_Garrison.GetAutoCombatDamageClassValues = function() return nil end
    q = capture(env).query
    assert(q.n == 1 and q.values[1].status == "observed" and q.values[1].kind == "nil")
    local extra = setmetatable({}, { __index = function() error(errorPayload) end })
    env.C_Garrison.GetAutoCombatDamageClassValues = function() return {}, nil, false, extra end
    q = capture(env).query
    assert(q.n == 4 and q.values[2].kind == "nil" and q.values[3].value == false)
    assert(q.values[4].kind == "table" and q.values[4].entries == nil and q.values[4].fields == nil)
    assertNotSaved(env.ApiContractProbeDB, { extra, secret, errorPayload })
end)

test("missing malformed restricted and throwing API observations survive", function()
    for _, api in ipairs({ false, 17, "not-a-function", secret }) do
        local env = setup(api)
        assert(capture(env).query.status == "missing-api")
        assertNotSaved(env.ApiContractProbeDB, { secret, errorPayload })
    end
    local env = setup(nil)
    assert(capture(env).query.status == "missing-api")
    env = setup(function() error(errorPayload) end)
    assert(capture(env).query.status == "call-error")
    assertNotSaved(env.ApiContractProbeDB, { errorPayload, secret })
    env = setup(nil)
    env.C_Garrison = setmetatable({}, { __index = function() error(secret) end })
    assert(capture(env).query.status == "field-error")
    assertNotSaved(env.ApiContractProbeDB, { secret })
end)

test("malformed and restricted first returns are observations not coerced rows", function()
    for _, value in ipairs({ false, 23, "unexpected", secret, newproxy(true) }) do
        local env = setup(function() return value end)
        local q = capture(env).query
        assert(q.status == "observed" and q.n == 1)
        local first = q.values[1]
        if rawequal(value, secret) then assert(first.status == "restricted" and first.value == nil)
        else
            assert(first.status == "observed" and first.kind == type(value))
            if type(value) ~= "userdata" then assert(first.value == value) end
        end
        assert(first.entries == nil)
        assertNotSaved(env.ApiContractProbeDB, { secret })
    end
    local rows = setmetatable({}, { __index = function() error(errorPayload) end })
    local env = setup(function() return rows end)
    env.canaccessvalue = function(v) return not rawequal(v, rows) and not rawequal(v, secret) end
    assert(capture(env).query.values[1].status == "restricted")
    assertNotSaved(env.ApiContractProbeDB, { rows, secret, errorPayload })
end)

test("positional holes malformed rows and throwing entry lookups are retained", function()
    local rows = { [1] = false, [3] = { damageClassValue = 0, locString = "" },
        [4] = secret, [5] = 41, [6] = "malformed-row" }
    setmetatable(rows, { __index = function(_, index)
        if index == 7 then error(errorPayload) end
    end })
    local env = setup(function() return rows end)
    local entries = capture(env).query.values[1].entries
    assert(#entries == 8 and entries[1].value == false and entries[2].kind == "nil")
    assert(entries[3].fields.damageClassValue.value == 0 and entries[3].fields.locString.value == "")
    assert(entries[4].status == "restricted" and entries[5].value == 41)
    assert(entries[6].value == "malformed-row" and entries[7].status == "field-error")
    assert(entries[8].kind == "nil")
    assertNotSaved(env.ApiContractProbeDB, { rows, rows[3], secret, errorPayload })
end)

test("field nil errors restriction and unusual types are not normalized", function()
    local opaque = newproxy(true)
    local throwing = setmetatable({}, { __index = function(_, key)
        if key == "damageClassValue" then error(errorPayload) end
        if key == "locString" then return "independent-text" end
        error("undeclared row field inspected")
    end })
    local restrictedRow = setmetatable({}, { __index = function() error(errorPayload) end })
    local rawField = setmetatable({}, { __index = function() error(errorPayload) end })
    local rows = { {}, throwing, { damageClassValue = secret, locString = secret },
        { damageClassValue = "not-a-number", locString = false }, restrictedRow,
        { damageClassValue = rawField, locString = opaque } }
    local env = setup(function() return rows end)
    env.canaccessvalue = function(v) return not rawequal(v, secret) and not rawequal(v, restrictedRow) end
    local entries = capture(env).query.values[1].entries
    assert(entries[1].fields.damageClassValue.kind == "nil" and entries[1].fields.locString.kind == "nil")
    assert(entries[2].fields.damageClassValue.status == "field-error")
    assert(entries[2].fields.locString.value == "independent-text")
    assert(entries[3].fields.damageClassValue.status == "restricted" and entries[3].fields.locString.status == "restricted")
    assert(entries[4].fields.damageClassValue.value == "not-a-number" and entries[4].fields.locString.value == false)
    assert(entries[5].status == "restricted")
    assert(entries[6].fields.damageClassValue.kind == "table" and entries[6].fields.locString.kind == "userdata")
    assertNotSaved(env.ApiContractProbeDB, { rows, throwing, restrictedRow, rawField, opaque, secret, errorPayload })
end)

test("eight row bound reports truncation beyond eight", function()
    for _, count in ipairs({ 8, 9, 12 }) do
        local rows = {}
        for i = 1, count do rows[i] = { damageClassValue = 100 + i, locString = "synthetic-" .. i } end
        local env = setup(function() return rows end)
        local first = capture(env).query.values[1]
        assert(#first.entries == 8 and first.entries[9] == nil)
        for i = 1, 8 do
            assert(first.entries[i].fields.damageClassValue.value == 100 + i)
            assert(first.entries[i].fields.locString.value == "synthetic-" .. i)
        end
        assert((first.truncated == true) == (count > 8))
        assertNotSaved(env.ApiContractProbeDB, { rows, rows[count] })
    end
end)

test("scalar strings and label remain bounded", function()
    local env = setup(function()
        return { { damageClassValue = string.rep("d", 300), locString = string.rep("s", 300) } }
    end)
    env.GetLocale = function() return string.rep("c", 300) end
    local result, record = capture(env, string.rep("l", 200))
    local fields = result.query.values[1].entries[1].fields
    assert(#record.label == 128)
    for _, scalar in ipairs({ fields.damageClassValue, fields.locString, result.locale.values[1] }) do
        assert(scalar.kind == "string" and #scalar.value == 256 and scalar.truncated == true)
    end
end)

test("locale errors and restrictions remain redacted observations", function()
    local env = setup(function() return {} end)
    env.GetLocale = function() error(errorPayload) end
    assert(capture(env).locale.status == "call-error")
    env.GetLocale = function() return secret end
    assert(capture(env).locale.values[1].status == "restricted")
    env.GetLocale = function() return nil end
    assert(capture(env).locale.values[1].kind == "nil")
    assertNotSaved(env.ApiContractProbeDB, { secret, errorPayload })
end)

test("all omits manual query and its capture", function()
    local calls = 0
    local env = setup(function() calls = calls + 1; return {} end)
    env.SlashCmdList.APICONTRACTPROBE("all excluded")
    assert(env.ApiContractProbeDB and #env.ApiContractProbeDB.captures == 1)
    assert(env.ApiContractProbeDB.captures[1].autoCombatDamageClasses == nil and calls == 0)
end)

print(string.format("%d auto-combat-damage-classes fixtures passed; %d failed", passed, failed))
assert(failed == 0, "auto combat damage classes fixtures failed")
