local root = assert(arg[1])
local passed = 0
local names = { "BasicDecor", "ExpertDecor", "Customize", "Cleanup", "Layout", "ExteriorCustomization" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local mutations = 0
local function forbidden() mutations = mutations + 1; error("mutation or load") end
local function setup(fn)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { HouseEditorMode = {} }
    for i, name in ipairs(names) do Enum.HouseEditorMode[name] = i * 10 + 0.5 end
    Enum.HouseEditorMode.Unrequested = 999
    C_HousingDecor = { IsModeDisabledForPreviewState = fn, SetPreviewState = forbidden }
    C_HouseEditor = { ActivateHouseEditorMode = forbidden }
    C_AddOns = { LoadAddOn = forbidden }
    LoadAddOn = forbidden
    mutations = 0
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("housing-preview-modes " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "housing-preview-modes mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].housingPreviewModes)
end
local function test(name, fn)
    fn(); assert(mutations == 0)
    passed = passed + 1; print("PASS " .. name)
end

test("fixed six names published noncanonical values two exact calls", function()
    local calls = 0
    setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 1)
        assert((...) == math.floor((calls - 1) / 2 + 1) * 10 + 0.5)
        return calls
    end)
    local r = capture()
    assert(calls == 12 and #r.modes == 6)
    for i, row in ipairs(r.modes) do
        assert(row.name == names[i])
        for j = 1, 2 do
            assert(row.observations[j].input.value == i * 10 + 0.5)
            assert(row.observations[j].values[1].value == (i - 1) * 2 + j)
        end
    end
end)

test("missing invalid and restricted enums have no fallback", function()
    for _, bad in ipairs({ false, "10", math.huge, -math.huge, 0/0, secret }) do
        local calls = 0
        setup(function() calls = calls + 1 end)
        Enum.HouseEditorMode.BasicDecor = bad
        local r = capture()
        assert(calls == 10)
        for _, q in ipairs(r.modes[1].observations) do assert(q.status == "unavailable-enum") end
    end
    setup(function() error("must not call") end)
    Enum.HouseEditorMode = secret
    assert(capture().modes[1].observations[1].status == "unavailable-enum")
    Enum = nil
    assert(capture().modes[6].observations[2].status == "unavailable-enum")
    setup(function() error("must not call") end)
    Enum.HouseEditorMode = {}
    assert(capture().modes[1].observations[1].status == "unavailable-enum")
end)

test("namespace and function missing restricted and lookup errors", function()
    for _, bad in ipairs({ false, 5, secret }) do
        setup(bad)
        assert(capture().modes[1].observations[1].status == "missing-api")
    end
    setup(nil)
    assert(capture().modes[1].observations[1].status == "missing-api")
    C_HousingDecor = secret
    assert(capture().modes[1].observations[1].status == "field-error")
    C_HousingDecor = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().modes[1].observations[1].status == "field-error")
end)

test("enum table access revocation prevents field lookup", function()
    local reads, revoked = 0, false
    setup(function() error("must not call") end)
    local modes = setmetatable({}, { __index = function() reads = reads + 1; error("revoked lookup") end })
    Enum = setmetatable({}, { __index = function() revoked = true; return modes end })
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, modes)) end
    assert(capture().modes[1].observations[1].status == "unavailable-enum" and reads == 0)
end)

test("lookup and both function guards revoke every input before forwarding", function()
    for _, stage in ipairs({ "lookup", "secret", "access" }) do
        for index = 1, 6 do
            local calls, revoked = 0, false
            local blocked = index * 10 + 0.5
            local fn = function(v) assert(v ~= blocked); calls = calls + 1 end
            setup(fn)
            if stage == "lookup" then
                C_HousingDecor = setmetatable({}, { __index = function() revoked = true; return fn end })
            end
            issecretvalue = function(v)
                if stage == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if stage == "access" and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, blocked))
            end
            local r = capture()
            assert(calls == 10)
            assert(r.modes[index].observations[1].status ~= "observed")
            assert(r.modes[index].observations[2].status ~= "observed")
        end
    end
end)

test("zero nil opaque errors independent calls and fresh lookup", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil end
        if calls == 3 then error(secret) end
        return nil, secret, false, nil
    end)
    local r = capture()
    assert(calls == 12 and r.modes[1].observations[1].n == 0)
    assert(r.modes[1].observations[2].n == 1)
    assert(r.modes[2].observations[1].status == "call-error")
    local q = r.modes[2].observations[2]
    assert(q.n == 4 and q.values[1].kind == "nil" and q.values[2].status == "restricted")
    assert(q.values[3].value == false and q.values[4].kind == "nil")
    setup(function()
        C_HousingDecor.IsModeDisabledForPreviewState = function() return "replacement" end
        error(secret)
    end)
    r = capture()
    assert(r.modes[1].observations[1].status == "call-error")
    assert(r.modes[1].observations[2].values[1].value == "replacement")
end)

test("bounded tuple strings labels snapshots and calls", function()
    local calls, values = 0, {}
    for i = 1, 20 do values[i] = string.rep("x", 300) end
    setup(function() calls = calls + 1; return unpack(values) end)
    local r = capture(string.rep("l", 200))
    local q = r.modes[1].observations[1]
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(calls == 120 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("missing access fails closed and all excludes housing", function()
    local calls = 0
    setup(function() calls = calls + 1 end)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("housing-preview-modes absent")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(function() calls = calls + 1 end)
    SlashCmdList.APICONTRACTPROBE("all context")
    assert(calls == 0 and ApiContractProbeDB.captures[1].housingPreviewModes == nil)
end)
print(string.format("%d housing-preview-modes fixtures passed", passed))
