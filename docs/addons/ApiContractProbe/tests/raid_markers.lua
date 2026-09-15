local root = assert(arg[1], "addon directory required")
local passed = 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local tokens = { "player", "target", "focus", "pet", "party1", "party2",
    "nonexistent", "invalid-unit-token", "" }
local function setup(target, active, enabled)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    CanBeRaidTarget, IsRaidMarkerActive, IsRaidMarkerSystemEnabled = target, active, enabled
    local function excluded() error("excluded API invoked") end
    GetRaidTargetIndex, SetRaidTarget, ClearRaidMarker = excluded, excluded, excluded
    PlaceRaidMarker, RemoveRaidTargets = excluded, excluded
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("raid-markers sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual raid-markers mode absent")
    assert(ApiContractProbeDB.captures[1].label == "sample")
    return assert(ApiContractProbeDB.captures[1].raidMarkers)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end

test("exact arguments and repeated raw results", function()
    local units, indices, enabledCalls = {}, {}, 0
    setup(function(...)
        assert(select("#", ...) == 1)
        local unit = ...
        units[#units + 1] = unit
        return #units % 2 == 1, nil, unit
    end, function(...)
        assert(select("#", ...) == 1)
        local index = ...
        indices[#indices + 1] = index
        return index, nil
    end, function(...)
        assert(select("#", ...) == 0)
        enabledCalls = enabledCalls + 1
        return enabledCalls % 2 == 1
    end)
    local result = capture()
    assert(#units == 18 and #indices == 16 and enabledCalls == 2)
    assert(#result.units == 9 and #result.markers == 8 and #result.enabled == 2)
    for i, unit in ipairs(tokens) do
        local row = result.units[i]
        assert(row.unit == unit and units[2*i-1] == unit and units[2*i] == unit)
        assert(row.observations[1].n == 3 and row.observations[2].n == 3)
        assert(row.observations[1].values[1].value == true)
        assert(row.observations[2].values[1].value == false)
        assert(row.observations[1].values[2].kind == "nil")
        assert(row.observations[2].values[3].value == unit)
    end
    for i, row in ipairs(result.markers) do
        assert(row.index == i and indices[2*i-1] == i and indices[2*i] == i)
        assert(row.observations[2].n == 2 and row.observations[2].values[1].value == i)
    end
    assert(result.enabled[1].values[1].value and result.enabled[2].values[1].value == false)
end)

test("missing and throwing functions do not gate independent observations", function()
    local calls = 0
    setup(nil, function() error(secret) end, function() calls = calls + 1; return nil end)
    local result = capture()
    assert(result.units[9].observations[2].status == "missing-api")
    assert(result.markers[8].observations[2].status == "call-error")
    assert(calls == 2 and result.enabled[2].n == 1 and result.enabled[2].values[1].kind == "nil")
    setup(function() end, nil, nil)
    result = capture()
    assert(result.units[9].observations[2].n == 0)
    assert(result.markers[8].observations[2].status == "missing-api")
    assert(result.enabled[2].status == "missing-api")
end)

test("access before inspection and access rechecked for repeated calls", function()
    setup(secret, function() return secret end, function() return secret end)
    local original = type
    type = function(v) assert(not rawequal(v, secret), "restricted type"); return original(v) end
    local ok, result = pcall(capture)
    type = original
    assert(ok, result)
    assert(result.units[1].observations[1].status == "missing-api")
    assert(result.markers[8].observations[2].values[1].status == "restricted")
    assert(result.enabled[2].values[1].status == "restricted")
    local calls, revoked = 0, false
    local fn
    fn = function() calls = calls + 1; revoked = true; return true end
    setup(fn, nil, nil)
    canaccessvalue = function(v) return not (revoked and rawequal(v, fn)) end
    result = capture()
    assert(calls == 1 and result.units[1].observations[2].status == "missing-api")
end)

test("scalar tuples and bytes bounded while objects stay opaque", function()
    local hostile = setmetatable({}, { __index = function() error("object inspected") end,
        __tostring = function() error("object stringified") end })
    local values = {}
    for i = 1, 20 do values[i] = i end
    setup(function() return unpack(values) end, function() return hostile, string.rep("x", 300) end,
        function() return 0/0, math.huge end)
    local result = capture()
    local unit, marker = result.units[1].observations[1], result.markers[1].observations[1]
    assert(unit.n == 20 and #unit.values == 16 and unit.truncated)
    assert(marker.values[1].kind == "table" and marker.values[1].fields == nil)
    assert(#marker.values[2].value == 256 and marker.values[2].truncated)
    assert(result.enabled[1].values[1].value == nil and result.enabled[1].values[2].value == nil)
end)

test("missing or throwing access guards fail closed", function()
    local calls = 0
    local function fn() calls = calls + 1 end
    setup(fn, fn, fn)
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("raid-markers")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(fn, fn, fn)
    canaccessvalue = function() error(secret) end
    local result = capture()
    assert(result.units[9].observations[2].status == "missing-api")
    assert(result.markers[8].observations[2].status == "missing-api")
    assert(result.enabled[2].status == "missing-api" and calls == 0)
end)

test("manual only and ten snapshot cap", function()
    local calls = 0
    local function fn() calls = calls + 1; return false end
    setup(fn, fn, fn)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and ApiContractProbeDB.captures[1].raidMarkers == nil)
    setup(fn, fn, fn)
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("raid-markers") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1 and calls == 360)
end)
print(string.format("%d/%d passed", passed, passed))
