local root = assert(arg[1])
local passed = 0
local names = { "Raid", "Activities", "World", "RankedPvP", "Concession" }
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
    Enum = { WeeklyRewardChestThresholdType = {} }
    for i, name in ipairs(names) do Enum.WeeklyRewardChestThresholdType[name] = i * 10 + 0.5 end
    Enum.WeeklyRewardChestThresholdType.Unrequested = 999
    C_WeeklyRewards = { GetSortedProgressForActivity = fn, SetPreviewState = forbidden }
    C_HouseEditor = { ActivateWeeklyRewardChestThresholdType = forbidden }
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
    SlashCmdList.APICONTRACTPROBE("weekly-progress " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "weekly-progress mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].weeklyProgress)
end
local function test(name, fn)
    fn(); assert(mutations == 0)
    passed = passed + 1; print("PASS " .. name)
end

test("fixed five names published noncanonical values two exact calls", function()
    local calls = 0
    setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 2)
        local _, combine = ...
        assert(combine == (calls % 2 == 0))
        assert((...) == math.floor((calls - 1) / 2 + 1) * 10 + 0.5)
        return calls
    end)
    local r = capture()
    assert(calls == 10 and #r.modes == 5)
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
        Enum.WeeklyRewardChestThresholdType.Raid = bad
        local r = capture()
        assert(calls == 8)
        for _, q in ipairs(r.modes[1].observations) do assert(q.status == "unavailable-enum") end
    end
    setup(function() error("must not call") end)
    Enum.WeeklyRewardChestThresholdType = secret
    assert(capture().modes[1].observations[1].status == "unavailable-enum")
    Enum = nil
    assert(capture().modes[5].observations[2].status == "unavailable-enum")
    setup(function() error("must not call") end)
    Enum.WeeklyRewardChestThresholdType = {}
    assert(capture().modes[1].observations[1].status == "unavailable-enum")
end)

test("namespace and function missing restricted and lookup errors", function()
    for _, bad in ipairs({ false, 5, secret }) do
        setup(bad)
        assert(capture().modes[1].observations[1].status == "missing-api")
    end
    setup(nil)
    assert(capture().modes[1].observations[1].status == "missing-api")
    C_WeeklyRewards = secret
    assert(capture().modes[1].observations[1].status == "field-error")
    C_WeeklyRewards = setmetatable({}, { __index = function() error(secret) end })
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
        for index = 1, 5 do
            local calls, revoked = 0, false
            local blocked = index * 10 + 0.5
            local fn = function(v) assert(v ~= blocked); calls = calls + 1 end
            setup(fn)
            if stage == "lookup" then
                C_WeeklyRewards = setmetatable({}, { __index = function() revoked = true; return fn end })
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
            assert(calls == 8)
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
    assert(calls == 10 and r.modes[1].observations[1].n == 0)
    assert(r.modes[1].observations[2].n == 1)
    assert(r.modes[2].observations[1].status == "call-error")
    local q = r.modes[2].observations[2]
    assert(q.n == 4 and q.values[1].kind == "nil" and q.values[2].status == "restricted")
    assert(q.values[3].value == false and q.values[4].kind == "nil")
    setup(function()
        C_WeeklyRewards.GetSortedProgressForActivity = function() return "replacement" end
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
    assert(calls == 100 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

local fields = { "activityTierID", "difficulty", "numPoints" }

test("first table only bounded indices fields and no mutation or retention", function()
    local reads, fieldReads, calls = 0, 0, 0
    local weak = setmetatable({}, { __mode = "v" })
    local function forbiddenRead() error("unexpected traversal or mutation") end
    local extra = setmetatable({}, { __index = forbiddenRead })
    setup(function()
        calls = calls + 1
        local entry = newproxy(true)
        local mt = getmetatable(entry)
        mt.__index = function(_, key)
            fieldReads = fieldReads + 1
            assert(key == "activityTierID" or key == "difficulty" or key == "numPoints")
            return key == "numPoints" and secret or key
        end
        mt.__newindex, mt.__len, mt.__tostring, mt.__pairs = forbiddenRead, forbiddenRead, forbiddenRead, forbiddenRead
        local values = setmetatable({}, { __index = function(_, index)
            reads = reads + 1
            assert(index >= 1 and index <= 8)
            return entry
        end, __newindex = forbiddenRead, __len = forbiddenRead, __pairs = forbiddenRead })
        weak[1], weak[2] = values, entry
        return values, nil, extra
    end)
    local r = capture()
    assert(calls == 10 and reads == 80 and fieldReads == 240)
    local q = r.modes[1].observations[1]
    assert(q.n == 3 and q.values[2].kind == "nil" and q.values[3].entries == nil)
    assert(q.values[1].entries[8].fields.numPoints.status == "restricted")
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil)
end)

test("table and entry revocation guards every lookup", function()
    for stop = 1, 8 do
        local revoked, reads, calls = false, 0, 0
        local values = setmetatable({}, { __index = function(_, index)
            reads = reads + 1
            assert(index <= stop)
            if index == stop then revoked = true end
            return { numPoints = index }
        end })
        setup(function() calls = calls + 1; return calls == 1 and values or {} end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, values)) end
        local r = capture()
        assert(reads == stop and calls == 10)
        if stop < 8 then assert(r.modes[1].observations[1].values[1].entries[stop + 1].status == "field-error") end
    end
    for stop = 1, 3 do
        local revoked, reads = false, 0
        local entry = setmetatable({}, { __index = function(_, key)
            reads = reads + 1
            assert(reads <= stop and key == fields[reads])
            if reads == stop then revoked = true end
            return secret
        end })
        local calls = 0
        setup(function() calls = calls + 1; return calls == 1 and { entry, { numPoints = 9 } } or {} end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, entry)) end
        local entries = capture().modes[1].observations[1].values[1].entries
        assert(reads == stop and entries[1].fields[fields[stop]].status == "restricted")
        for i = stop + 1, 3 do assert(entries[1].fields[fields[i]].status == "field-error") end
        assert(entries[2].fields.numPoints.value == 9)
    end
end)

test("restricted fields errors raw values and serialization revocation", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        if calls == 1 then return secret end
        return { secret, setmetatable({}, { __index = function(_, key)
            if key == "activityTierID" then error(secret) end
            return key == "difficulty" and secret or math.huge
        end }), { activityTierID = false, difficulty = "raw", numPoints = nil } }
    end)
    local r = capture()
    assert(r.modes[1].observations[1].values[1].status == "restricted")
    local entries = r.modes[1].observations[2].values[1].entries
    assert(entries[1].status == "restricted")
    assert(entries[2].fields.activityTierID.status == "field-error")
    assert(entries[2].fields.difficulty.status == "restricted" and entries[2].fields.numPoints.status == "nonfinite")
    assert(entries[3].fields.activityTierID.value == false and entries[3].fields.difficulty.value == "raw")
    assert(entries[3].fields.numPoints.kind == "nil")
    local revoked = false
    local values = setmetatable({}, { __index = function() error("revoked table indexed") end })
    local extra = newproxy(true)
    setup(function() return values, extra end)
    canaccessvalue = function(v)
        if rawequal(v, extra) then revoked = true end
        return not (revoked and rawequal(v, values))
    end
    assert(capture().modes[1].observations[1].values[1].entries[1].status == "field-error")
end)

test("missing access fails closed and all excludes weekly progress", function()
    local calls = 0
    setup(function() calls = calls + 1 end)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("weekly-progress absent")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(function() calls = calls + 1 end)
    SlashCmdList.APICONTRACTPROBE("all context")
    assert(calls == 0 and ApiContractProbeDB.captures[1].weeklyProgress == nil)
end)
print(string.format("%d weekly-progress fixtures passed", passed))
