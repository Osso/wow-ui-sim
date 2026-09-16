local root = assert(arg[1])
local passed, failed, forbiddenCalls = 0, 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded operation") end
local function opaque()
    return setmetatable({}, { __index = forbidden, __len = forbidden, __pairs = forbidden, __tostring = forbidden })
end
local function setup(fn)
    ApiContractProbeDB, SlashCmdList, forbiddenCalls = nil, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_PerksActivities = { GetPerksActivitiesInfo = fn, GetPerksActivityInfo = forbidden,
        AddTrackedPerksActivity = forbidden, RemoveTrackedPerksActivity = forbidden,
        ClearPerksActivitiesPendingCompletion = forbidden, GetPerksActivityChatLink = forbidden }
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("perks-criteria " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "perks-criteria mode absent")
    local result = ApiContractProbeDB.captures[#ApiContractProbeDB.captures].perksCriteria
    assert(result, "perks criteria output absent")
    return result
end
local function activities(r, attempt)
    local value = r[attempt or 1].values[1]
    assert(value.fields, "root fields absent: " .. tostring(value.kind) .. "/" .. tostring(value.status))
    return value.fields.activities.entries
end
local function fixture()
    return { activities = { { ID = 23.5,
        criteriaList = { { criteriaID = 17.25, requiredValue = 31 } },
        requirementsList = { { completed = false, requirementText = "requirement" } } } } }
end
local function test(name, fn)
    local ok, err = pcall(function() fn(); assert(forbiddenCalls == 0, "excluded operation executed") end)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("two fresh zero-argument calls use declared activities root", function()
    local calls = 0
    setup(function(...)
        assert(select("#", ...) == 0); calls = calls + 1
        local info = fixture(); info.activities[1].ID = calls + 0.25
        return info
    end)
    local r = capture()
    assert(calls == 2 and #r == 2)
    for i = 1, 2 do
        local f = activities(r, i)[1].fields
        assert(f.ID.value == i + 0.25)
        assert(f.criteriaList.entries[1].fields.criteriaID.value == 17.25)
        assert(f.criteriaList.entries[1].fields.requiredValue.value == 31)
        assert(f.requirementsList.entries[1].fields.completed.value == false)
        assert(f.requirementsList.entries[1].fields.requirementText.value == "requirement")
    end
end)

test("arity nil holes first return only opaque errors and independent calls", function()
    local calls = 0
    setup(function() calls = calls + 1; if calls == 1 then error(secret) end; return nil, opaque(), nil end)
    local r = capture()
    assert(calls == 2 and r[1].status == "call-error" and r[2].n == 3)
    assert(r[2].values[1].kind == "nil" and r[2].values[2].fields == nil and r[2].values[3].kind == "nil")
    setup(function() end); r = capture(); assert(r[1].n == 0 and r[2].n == 0)
    setup(function() return { { ID = 99 } } end)
    r = capture(); assert(r[1].values[1].fields.activities.kind == "nil")
end)

test("namespace function and missing access guards fail closed", function()
    setup(function() return fixture() end); capture()
    for _, ns in ipairs({ secret, false, setmetatable({}, { __index = function() error(secret) end }) }) do
        setup(function() return fixture() end); C_PerksActivities = ns
        local r = capture(); assert(r[1].status ~= "observed" and r[2].status ~= "observed")
    end
    for _, phase in ipairs({ "missing", "secret", "access" }) do
        local fn = forbidden; setup(fn)
        if phase == "missing" then C_PerksActivities.GetPerksActivitiesInfo = nil
        elseif phase == "secret" then issecretvalue = function(v) return rawequal(v, fn) end
        else canaccessvalue = function(v) return not rawequal(v, fn) end end
        local r = capture(); assert(r[1].status == "missing-api" and r[2].status == "missing-api")
    end
    setup(forbidden); issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("perks-criteria guards")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)

test("independent fresh function lookup and restricted root", function()
    setup(function()
        C_PerksActivities.GetPerksActivitiesInfo = function() return fixture() end
        return secret
    end)
    local r = capture(); assert(r[1].values[1].status == "restricted")
    assert(activities(r, 2)[1].fields.ID.value == 23.5)
end)

test("only eight activities four criteria four requirements fixed fields", function()
    local info = { activities = {} }
    for i = 1, 9 do
        local a = { ID = i, criteriaList = {}, requirementsList = {} }
        for j = 1, 5 do
            a.criteriaList[j] = setmetatable({ criteriaID = j, requiredValue = i * j }, { __index = forbidden })
            a.requirementsList[j] = setmetatable({ completed = j == 2, requirementText = "r" .. j }, { __index = forbidden })
        end
        a.criteriaList[5], a.requirementsList[5] = opaque(), opaque()
        info.activities[i] = setmetatable(a, { __index = forbidden })
    end
    info.activities[9] = opaque()
    setmetatable(info, { __index = forbidden })
    setup(function() return info end)
    local rows = activities(capture())
    assert(#rows == 8 and rows[9] == nil)
    for i = 1, 8 do
        assert(#rows[i].fields.criteriaList.entries == 4 and #rows[i].fields.requirementsList.entries == 4)
        assert(rows[i].fields.criteriaList.entries[4].fields.requiredValue.value == i * 4)
    end
end)

test("all receiver levels rechecked before each lookup", function()
    for _, target in ipairs({ "root", "activities", "activity", "criteria", "criterion", "requirements", "requirement" }) do
        local info = fixture()
        local objects = { root = info, activities = info.activities, activity = info.activities[1],
            criteria = info.activities[1].criteriaList, criterion = info.activities[1].criteriaList[1],
            requirements = info.activities[1].requirementsList, requirement = info.activities[1].requirementsList[1] }
        local revoked, unsafe = false, 0
        local original = objects[target]
        local proxy = setmetatable({}, { __index = function(_, key)
            if revoked then unsafe = unsafe + 1; error("revoked lookup") end
            revoked = true; return original[key]
        end })
        if target == "root" then info = proxy
        elseif target == "activities" then info.activities = proxy
        elseif target == "activity" then info.activities[1] = proxy
        elseif target == "criteria" then info.activities[1].criteriaList = proxy
        elseif target == "criterion" then info.activities[1].criteriaList[1] = proxy
        elseif target == "requirements" then info.activities[1].requirementsList = proxy
        else info.activities[1].requirementsList[1] = proxy end
        setup(function() return info end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, proxy)) end
        capture(); assert(unsafe == 0, target)
    end
end)

test("serialization can revoke root list entry and later nested field", function()
    for _, target in ipairs({ "root", "activities", "activity", "criterion", "requirement" }) do
        local info = fixture()
        local objects = { root = info, activities = info.activities, activity = info.activities[1],
            criterion = info.activities[1].criteriaList[1], requirement = info.activities[1].requirementsList[1] }
        local denied, checks, unsafe = false, 0, 0
        local victim = objects[target]
        local data = {}; for key, value in pairs(victim) do data[key] = value; victim[key] = nil end
        setmetatable(victim, { __index = function(_, key)
            if denied then unsafe = unsafe + 1; error("revoked read") end
            return data[key]
        end })
        setup(function() return info end)
        canaccessvalue = function(v)
            if rawequal(v, victim) then
                checks = checks + 1
                if checks == 2 then denied = true; return false end
                return not denied
            end
            return not rawequal(v, secret)
        end
        capture(); assert(unsafe == 0 and checks >= 2, target)
    end
end)

test("malformed containers missing fields and peer entries remain observations", function()
    setup(function() return { activities = { secret, { ID = 2, criteriaList = false,
        requirementsList = { secret, { completed = nil, requirementText = secret }, { completed = true, requirementText = "ok" } } },
        { ID = 3, criteriaList = { setmetatable({}, { __index = function() error(secret) end }), { criteriaID = 8, requiredValue = math.huge } } } } } end)
    local rows = activities(capture())
    assert(rows[1].status == "restricted" and rows[2].fields.criteriaList.kind == "boolean")
    assert(rows[2].fields.requirementsList.entries[2].fields.completed.kind == "nil")
    assert(rows[2].fields.requirementsList.entries[2].fields.requirementText.status == "restricted")
    assert(rows[2].fields.requirementsList.entries[3].fields.requirementText.value == "ok")
    assert(rows[3].fields.criteriaList.entries[1].fields.criteriaID.status == "field-error")
    assert(rows[3].fields.criteriaList.entries[2].fields.requiredValue.status == "nonfinite")
end)

test("sixteen tuple positions strings labels and ten snapshots bounded", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        local info = fixture(); info.activities[1].requirementsList[1].requirementText = string.rep("x", 300)
        local values = { info }; for i = 2, 17 do values[i] = i end
        return unpack(values, 1, 17)
    end)
    local r = capture(string.rep("l", 150))
    assert(r[1].n == 17 and r[1].truncated and r[1].values[16].value == 16 and r[1].values[17] == nil)
    local text = activities(r)[1].fields.requirementsList.entries[1].fields.requirementText
    assert(#text.value == 256 and text.truncated and #ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do SlashCmdList.APICONTRACTPROBE("perks-criteria limit") end
    assert(calls == 20 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("objects collectible and excluded from all", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local info = fixture(); weak[#weak + 1] = info; return info end)
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
    setup(forbidden)
    SlashCmdList.APICONTRACTPROBE("all existing")
    assert(ApiContractProbeDB.captures[1].perksCriteria == nil)
end)

print(string.format("perks criteria: %d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
