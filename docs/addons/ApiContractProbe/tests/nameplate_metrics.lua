local root = assert(arg[1])
local passed = 0
local names = { "Friendly", "Enemy" }
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
    Enum = { NamePlateType = {} }
    for i, name in ipairs(names) do Enum.NamePlateType[name] = i * 10 + 0.5 end
    Enum.NamePlateType.Unrequested = 999
    C_NamePlateManager = { GetNamePlateHitTestInsets = fn, SetNamePlateHitTestInsets = forbidden,
        SetNamePlateSimplified = forbidden, IsNamePlateUnitBehindCamera = forbidden }
    C_NamePlate = { GetNamePlateSize = function() return 100, 20 end,
        SetNamePlateSize = forbidden, GetNamePlateForUnit = forbidden, GetNamePlates = forbidden }
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
    SlashCmdList.APICONTRACTPROBE("nameplate-metrics " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "nameplate-metrics mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].nameplateMetrics)
end
local function test(name, fn)
    fn(); assert(mutations == 0)
    passed = passed + 1; print("PASS " .. name)
end

test("fixed two names published noncanonical values two exact calls", function()
    local calls = 0
    setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 1)
        assert((...) == math.floor((calls - 1) / 2 + 1) * 10 + 0.5)
        return calls
    end)
    local r = capture()
    assert(calls == 4 and #r.modes == 2)
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
        Enum.NamePlateType.Friendly = bad
        local r = capture()
        assert(calls == 2)
        for _, q in ipairs(r.modes[1].observations) do assert(q.status == "unavailable-enum") end
    end
    setup(function() error("must not call") end)
    Enum.NamePlateType = secret
    assert(capture().modes[1].observations[1].status == "unavailable-enum")
    Enum = nil
    assert(capture().modes[2].observations[2].status == "unavailable-enum")
    setup(function() error("must not call") end)
    Enum.NamePlateType = {}
    assert(capture().modes[1].observations[1].status == "unavailable-enum")
end)

test("namespace and function missing restricted and lookup errors", function()
    for _, bad in ipairs({ false, 5, secret }) do
        setup(bad)
        assert(capture().modes[1].observations[1].status == "missing-api")
    end
    setup(nil)
    assert(capture().modes[1].observations[1].status == "missing-api")
    C_NamePlateManager = secret
    assert(capture().modes[1].observations[1].status == "field-error")
    C_NamePlateManager = setmetatable({}, { __index = function() error(secret) end })
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
        for index = 1, 2 do
            local calls, revoked = 0, false
            local blocked = index * 10 + 0.5
            local fn = function(v) assert(v ~= blocked); calls = calls + 1 end
            setup(fn)
            if stage == "lookup" then
                C_NamePlateManager = setmetatable({}, { __index = function() revoked = true; return fn end })
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
            assert(calls == 2)
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
    assert(calls == 4 and r.modes[1].observations[1].n == 0)
    assert(r.modes[1].observations[2].n == 1)
    assert(r.modes[2].observations[1].status == "call-error")
    local q = r.modes[2].observations[2]
    assert(q.n == 4 and q.values[1].kind == "nil" and q.values[2].status == "restricted")
    assert(q.values[3].value == false and q.values[4].kind == "nil")
    setup(function()
        C_NamePlateManager.GetNamePlateHitTestInsets = function() return "replacement" end
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
    local sizeCalls = 0
    C_NamePlate.GetNamePlateSize = function() sizeCalls = sizeCalls + 1; return unpack(values) end
    local r = capture(string.rep("l", 200))
    assert(r.size[1].n == 20 and r.size[1].truncated and #r.size[1].values == 16)
    assert(#r.size[1].values[1].value == 256)
    local q = r.modes[1].observations[1]
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(calls == 40 and sizeCalls == 20 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("missing access fails closed and all excludes metrics", function()
    local calls = 0
    setup(function() calls = calls + 1 end)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("nameplate-metrics absent")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(function() calls = calls + 1 end)
    SlashCmdList.APICONTRACTPROBE("all context")
    assert(calls == 0 and ApiContractProbeDB.captures[1].nameplateMetrics == nil)
end)
test("size exact zero args and native arities independently preserved", function()
    local sizes, insets = 0, 0
    setup(function(...) insets = insets + 1; assert(select("#", ...) == 1); return 1, nil, 3, 4 end)
    C_NamePlate.GetNamePlateSize = function(...)
        sizes = sizes + 1; assert(select("#", ...) == 0)
        return 12, 34
    end
    local r = capture()
    assert(sizes == 2 and insets == 4 and r.size[1].n == 2 and r.size[2].values[2].value == 34)
    assert(r.modes[1].observations[1].n == 4 and r.modes[1].observations[1].values[2].kind == "nil")
    C_NamePlate.GetNamePlateSize = function() error(secret) end
    r = capture(); assert(r.size[1].status == "call-error" and insets == 8)
    C_NamePlate = secret
    r = capture(); assert(r.size[1].status == "field-error" and insets == 12)
end)

test("all excludes size and inset calls and objects are collectible", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local v = newproxy(true); weak[1] = v; return v end)
    C_NamePlate.GetNamePlateSize = function() local v = {}; weak[2] = v; return v end
    capture(); collectgarbage("collect"); collectgarbage("collect")
    assert(weak[1] == nil and weak[2] == nil)
    C_NamePlate.GetNamePlateSize = forbidden
    C_NamePlateManager.GetNamePlateHitTestInsets = forbidden
    SlashCmdList.APICONTRACTPROBE("all excluded")
end)
test("size function guards and errors do not suppress insets", function()
    for _, phase in ipairs({ "secret", "access", "lookup" }) do
        local sizes, insets = 0, 0
        setup(function() insets = insets + 1; return 1, 2, 3, 4 end)
        local fn = function() sizes = sizes + 1 end
        C_NamePlate.GetNamePlateSize = fn
        if phase == "secret" then
            issecretvalue = function(v) return rawequal(v, fn) or rawequal(v, secret) end
        elseif phase == "access" then
            canaccessvalue = function(v) return not rawequal(v, fn) and not rawequal(v, secret) end
        else
            C_NamePlate = setmetatable({}, { __index = function() error(secret) end })
        end
        local r = capture()
        assert(sizes == 0 and insets == 4 and r.size[1].status ~= "observed")
    end
end)
print(string.format("%d nameplate-metrics fixtures passed", passed))
