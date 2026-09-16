local root = assert(arg[1])
local passed, forbiddenCalls = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local infoKeys = { "isLoaded", "neighborhoodGUID", "initiativeID", "currentCycleID", "progressRequired",
    "currentProgress", "playerTotalContribution", "duration", "tasks", "milestones", "title", "description" }
local activityKeys = { "isLoaded", "neighborhoodGUID", "nextUpdateTime", "taskActivity" }
local logKeys = { "taskID", "playerName", "taskName", "completionTime", "amount" }
local taskKeys = { "ID", "taskName", "description", "progressContributionAmount", "tracked", "supersedes",
    "timesCompleted", "completed", "inProgress", "taskType", "sortOrder", "rewardQuestID", "requirementsList", "criteriaList" }
local names = { "GetNeighborhoodInitiativeInfo", "GetInitiativeActivityLogInfo", "GetTrackedInitiativeTasks",
    "GetInitiativeTaskInfo", "GetInitiativeTaskChatLink" }
local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded call") end
local function opaque()
    return setmetatable({}, { __index = forbidden, __len = forbidden, __pairs = forbidden, __tostring = forbidden })
end
local function setup(overrides)
    ApiContractProbeDB, SlashCmdList = nil, {}
    forbiddenCalls = 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_NeighborhoodInitiative = {
        GetNeighborhoodInitiativeInfo = function() return {} end,
        GetInitiativeActivityLogInfo = function() return {} end,
        GetTrackedInitiativeTasks = function() return { trackedIDs = {} } end,
        GetInitiativeTaskInfo = forbidden, GetInitiativeTaskChatLink = forbidden,
        RequestNeighborhoodInitiativeInfo = forbidden, RequestInitiativeActivityLog = forbidden,
        SetActiveNeighborhood = forbidden, SetViewingNeighborhood = forbidden,
        AddTrackedInitiativeTask = forbidden, RemoveTrackedInitiativeTask = forbidden,
    }
    for key, value in pairs(overrides or {}) do C_NeighborhoodInitiative[key] = value end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("neighborhood-structures " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "neighborhood-structures mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].neighborhoodStructures)
end
local function tracked(r) return r.GetTrackedInitiativeTasks.values[1].fields.trackedIDs.entries end
local function test(name, fn)
    fn(); assert(forbiddenCalls == 0); passed = passed + 1; print("PASS " .. name)
end

test("three noarg producers original IDs and declared fields", function()
    local counts = {}
    local function count(name, n, ...) assert(select("#", ...) == n); counts[name] = (counts[name] or 0) + 1 end
    local info = { isLoaded = false, neighborhoodGUID = "guid", initiativeID = 3, currentCycleID = 4,
        progressRequired = 5, currentProgress = 6, playerTotalContribution = 7, duration = 8,
        tasks = {}, milestones = {}, title = "title", description = "description" }
    local task = { ID = 99, taskName = "task", description = "desc", progressContributionAmount = 3,
        tracked = false, supersedes = 4, timesCompleted = 5, completed = false, inProgress = true,
        taskType = 6, sortOrder = 7, rewardQuestID = 8, requirementsList = opaque(), criteriaList = opaque() }
    setup({
        GetNeighborhoodInitiativeInfo = function(...) count(names[1], 0, ...); return info end,
        GetInitiativeActivityLogInfo = function(...) count(names[2], 0, ...); return {
            isLoaded = true, neighborhoodGUID = "log", nextUpdateTime = 12,
            taskActivity = { { taskID = 1000, playerName = "player", taskName = "log task", completionTime = 22, amount = 23 } } } end,
        GetTrackedInitiativeTasks = function(...) count(names[3], 0, ...); return { trackedIDs = { 1.25, -2.5 } } end,
        GetInitiativeTaskInfo = function(...) count(names[4], 1, ...); assert((...) == (counts[names[4]] == 1 and 1.25 or -2.5)); return task end,
        GetInitiativeTaskChatLink = function(...) count(names[5], 1, ...); assert((...) == (counts[names[5]] == 1 and 1.25 or -2.5)); return "link", nil end,
    })
    local r = capture()
    for _, key in ipairs(infoKeys) do assert(r[names[1]].values[1].fields[key].status == "observed") end
    assert(r[names[1]].values[1].fields.tasks.fields == nil)
    local log = r[names[2]].values[1].fields.taskActivity.entries[1].fields
    assert(log.taskID.value == 1000 and log.playerName.value == "player" and log.amount.value == 23)
    local rows = tracked(r)
    for _, key in ipairs(taskKeys) do assert(rows[1].info.values[1].fields[key].status == "observed") end
    assert(rows[1].info.values[1].fields.ID.value == 99 and rows[1].chatLink.n == 2)
    assert(rows[1].info.values[1].fields.criteriaList.fields == nil)
    assert(counts[names[1]] == 1 and counts[names[2]] == 1 and counts[names[3]] == 1)
    assert(counts[names[4]] == 2 and counts[names[5]] == 2)
end)

test("zero arity nil holes first return only and opaque errors", function()
    setup({ GetNeighborhoodInitiativeInfo = function() end,
        GetInitiativeActivityLogInfo = function() return nil, opaque(), nil end,
        GetTrackedInitiativeTasks = function() error(secret) end })
    local r = capture()
    assert(r[names[1]].n == 0 and r[names[2]].n == 3 and r[names[2]].values[1].kind == "nil")
    assert(r[names[2]].values[2].fields == nil and r[names[3]].status == "call-error")
    setup({ GetTrackedInitiativeTasks = function() return { trackedIDs = { 1, 2 } } end,
        GetInitiativeTaskInfo = function(id) if id == 1 then error(secret) end end,
        GetInitiativeTaskChatLink = function(id) if id == 1 then return nil, false, nil end return "ok" end })
    local rows = tracked(capture())
    assert(rows[1].info.status == "call-error" and rows[1].chatLink.n == 3)
    assert(rows[2].info.n == 0 and rows[2].chatLink.values[1].value == "ok")
end)

test("missing restricted and throwing namespaces functions guards", function()
    for _, value in ipairs({ secret, false, setmetatable({}, { __index = function() error(secret) end }) }) do
        setup(); C_NeighborhoodInitiative = value
        local r = capture(); for i = 1, 3 do assert(r[names[i]].status ~= "observed") end
    end
    for selected = 1, 3 do
        for _, phase in ipairs({ "missing", "secret", "access", "error" }) do
            setup(); local fn = C_NeighborhoodInitiative[names[selected]]
            if phase == "missing" then C_NeighborhoodInitiative[names[selected]] = nil
            elseif phase == "secret" then issecretvalue = function(v) return rawequal(v, fn) or rawequal(v, secret) end
            elseif phase == "access" then canaccessvalue = function(v) return not rawequal(v, fn) and not rawequal(v, secret) end
            else C_NeighborhoodInitiative[names[selected]] = function() error(secret) end end
            local r = capture(); assert(r[names[selected]].status ~= "observed")
            for i = 1, 3 do if i ~= selected then assert(r[names[i]].status == "observed") end end
        end
    end
    setup(); issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("neighborhood-structures guards")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)

test("invalid IDs are never forwarded and peers continue", function()
    for _, bad in ipairs({ false, "5", math.huge, -math.huge, 0/0, secret, opaque() }) do
        local calls = 0
        local query = function(id) assert(id == 2.25); calls = calls + 1 end
        setup({ GetTrackedInitiativeTasks = function() return { trackedIDs = { bad, 2.25 } } end,
            GetInitiativeTaskInfo = query, GetInitiativeTaskChatLink = query })
        local rows = tracked(capture()); assert(calls == 2 and rows[1].info.status ~= "observed")
    end
end)

test("every task ID rechecked after namespace lookup and both function guards", function()
    for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
        for selected = 1, 4 do
            for _, api in ipairs({ names[4], names[5] }) do
                local revoked, calls = false, 0
                local id = selected + 0.25
                local query = function(value) assert(not (revoked and value == id)); calls = calls + 1 end
                setup({ GetTrackedInitiativeTasks = function() return { trackedIDs = { 1.25, 2.25, 3.25, 4.25 } } end,
                    GetInitiativeTaskInfo = query, GetInitiativeTaskChatLink = query })
                local namespace = C_NeighborhoodInitiative
                local target = function(value) return query(value) end
                namespace[api] = nil
                setmetatable(namespace, { __index = function(_, key)
                    assert(key == api); if phase == "lookup" then revoked = true end; return target
                end })
                issecretvalue = function(v)
                    if phase == "secret" and rawequal(v, target) then revoked = true end
                    return rawequal(v, secret)
                end
                canaccessvalue = function(v)
                    if phase == "namespace" and rawequal(v, namespace) then revoked = true end
                    if phase == "access" and rawequal(v, target) then revoked = true end
                    return not rawequal(v, secret) and not (revoked and rawequal(v, id))
                end
                local rows = tracked(capture())
                assert(rows[selected][api == names[4] and "info" or "chatLink"].status == "restricted-input")
                assert(calls >= 6 and calls <= 7)
            end
        end
    end
end)

local function installReceiver(which, object)
    local overrides = {
        GetNeighborhoodInitiativeInfo = function() return which == "info" and object or {} end,
        GetInitiativeActivityLogInfo = function()
            if which == "activity" then return object end
            return { taskActivity = which == "logList" and object or { which == "logEntry" and object or {} } }
        end,
        GetTrackedInitiativeTasks = function()
            if which == "tracked" then return object end
            return { trackedIDs = which == "ids" and object or { 1.25 } }
        end,
        GetInitiativeTaskInfo = function() return which == "task" and object or {} end,
        GetInitiativeTaskChatLink = function() return "link" end,
    }
    setup(overrides)
end

test("every object field and list index rechecks its receiver", function()
    local cases = { { "info", infoKeys }, { "activity", activityKeys }, { "logEntry", logKeys },
        { "task", taskKeys }, { "tracked", { "trackedIDs" } }, { "logList", { 1,2,3,4,5,6,7,8 } }, { "ids", { 1,2,3,4 } } }
    for _, case in ipairs(cases) do
        for stop = 1, #case[2] do
            local revoked, reads = false, 0
            local object = setmetatable({}, { __index = function(_, key)
                assert(not revoked); reads = reads + 1; assert(key == case[2][reads])
                if reads == stop then revoked = true end
                if case[1] == "logList" then return {} end
                return 1.25
            end })
            installReceiver(case[1], object)
            canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
            capture(); assert(reads == stop)
        end
    end
end)

test("restricted objects fields and nested opaque exclusions", function()
    setup({ GetNeighborhoodInitiativeInfo = function() return { tasks = secret, milestones = {}, title = secret } end,
        GetInitiativeActivityLogInfo = function() return { taskActivity = { secret, { amount = secret } } } end,
        GetTrackedInitiativeTasks = function() return { trackedIDs = { secret, 1 } } end,
        GetInitiativeTaskInfo = function() return { requirementsList = opaque(), criteriaList = secret, taskName = secret } end,
        GetInitiativeTaskChatLink = function() return secret end })
    local r = capture()
    assert(r[names[1]].values[1].fields.tasks.status == "restricted")
    assert(r[names[2]].values[1].fields.taskActivity.entries[1].status == "restricted")
    assert(r[names[2]].values[1].fields.taskActivity.entries[2].fields.amount.status == "restricted")
    local rows = tracked(r); assert(rows[1].info.status == "restricted-input")
    assert(rows[2].info.values[1].fields.criteriaList.status == "restricted")
    assert(rows[2].chatLink.values[1].status == "restricted")
end)

test("serialization revocation prevents later traversal and downstream calls", function()
    local revoked, calls = false, 0
    local ids = { 1.25 }
    local marker = {}
    setup({ GetTrackedInitiativeTasks = function() return { trackedIDs = ids }, marker end,
        GetInitiativeTaskInfo = function() calls = calls + 1; return {}, marker end,
        GetInitiativeTaskChatLink = function() calls = calls + 1 end })
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, ids))
    end
    capture(); assert(calls == 0)
    revoked = false
    setup({ GetTrackedInitiativeTasks = function() return { trackedIDs = { 1.25 } } end,
        GetInitiativeTaskInfo = function() return {}, marker end,
        GetInitiativeTaskChatLink = forbidden })
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, 1.25))
    end
    assert(tracked(capture())[1].chatLink.status == "restricted-input")
end)

test("field errors do not suppress sibling fields or task links", function()
    setup({ GetNeighborhoodInitiativeInfo = function() return setmetatable({}, { __index = function(_, key)
        if key == "title" then error(secret) end; return false
    end }) end, GetTrackedInitiativeTasks = function() return { trackedIDs = { 1 } } end,
        GetInitiativeTaskInfo = function() return setmetatable({}, { __index = function() error(secret) end }) end,
        GetInitiativeTaskChatLink = function() return "peer" end })
    local r = capture(); assert(r[names[1]].values[1].fields.title.status == "field-error")
    assert(r[names[1]].values[1].fields.description.value == false)
    assert(tracked(r)[1].info.values[1].fields.ID.status == "field-error")
    assert(tracked(r)[1].chatLink.values[1].value == "peer")
end)

test("bounds cap calls fields strings tuples labels and snapshots", function()
    local calls = 0
    local function many() return unpack({ 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18 }) end
    setup({ GetNeighborhoodInitiativeInfo = function() calls = calls + 1; return { title = string.rep("x", 300) }, many() end,
        GetInitiativeActivityLogInfo = function() calls = calls + 1; return { taskActivity = setmetatable({}, {
            __index = function(_, i) assert(i <= 8); return { amount = i } end }) } end,
        GetTrackedInitiativeTasks = function() calls = calls + 1; return { trackedIDs = setmetatable({}, {
            __index = function(_, i) assert(i <= 4); return i + 0.25 end }) } end,
        GetInitiativeTaskInfo = function() calls = calls + 1; return {}, many() end,
        GetInitiativeTaskChatLink = function() calls = calls + 1; return string.rep("l", 300), many() end })
    local r = capture(string.rep("L", 200))
    assert(r[names[1]].n == 19 and r[names[1]].truncated and #r[names[1]].values == 16)
    assert(#r[names[1]].values[1].fields.title.value == 256)
    assert(tracked(r)[1].info.n == 19 and tracked(r)[1].info.truncated)
    assert(#tracked(r)[1].chatLink.values[1].value == 256 and calls == 11)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for i = 2, 11 do capture() end
    assert(calls == 110 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("objects and userdata collectible without retention and all excludes mode", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup({ GetNeighborhoodInitiativeInfo = function()
        local info = newproxy(true); getmetatable(info).__index = function(_, key)
            if key == "tasks" or key == "milestones" then return {} end
            return opaque()
        end
        weak[1] = info; return info
    end, GetTrackedInitiativeTasks = function()
        local ids = { 1 }; local object = { trackedIDs = ids }; weak[2], weak[3] = ids, object; return object
    end, GetInitiativeTaskInfo = function() local object = { requirementsList = opaque() }; weak[4] = object; return object end,
        GetInitiativeTaskChatLink = function() return "link" end })
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
    setup(); for _, name in ipairs(names) do C_NeighborhoodInitiative[name] = forbidden end
    SlashCmdList.APICONTRACTPROBE("all excluded")
    assert(ApiContractProbeDB.captures[1].neighborhoodStructures == nil)
end)

local milestoneKeys = { "milestoneOrderIndex", "requiredContributionAmount", "rewards" }
local rewardKeys = { "title", "description", "decorID", "decorQuantity", "favor", "money", "rewardQuestID" }
local function initiative(r) return r[names[1]].values[1] end
local function nestedFixture(kind, object)
    local task = { ID = 91, requirementsList = opaque(), criteriaList = opaque() }
    local reward = { title = "reward", money = 7 }
    local rewards = { kind == "reward" and object or reward }
    local milestone = { milestoneOrderIndex = 1, requiredContributionAmount = 2,
        rewards = kind == "rewards" and object or rewards }
    return { tasks = kind == "tasks" and object or { kind == "task" and object or task },
        milestones = kind == "milestones" and object or { kind == "milestone" and object or milestone } }
end

test("nested four-entry bounds preserve scalar fields and eleven-call behavior", function()
    local calls, reads = 0, { tasks = 0, milestones = 0, rewards = 0 }
    local function list(kind, make)
        return setmetatable({}, { __index = function(_, i)
            assert(i >= 1 and i <= 4); reads[kind] = reads[kind] + 1; return make(i)
        end, __len = forbidden, __pairs = forbidden })
    end
    local info = { title = "original", description = "kept",
        tasks = list("tasks", function(i) return { ID = 100 + i, taskName = "task",
            requirementsList = opaque(), criteriaList = opaque() } end),
        milestones = list("milestones", function(i) return { milestoneOrderIndex = i,
            requiredContributionAmount = 1000 + i, rewards = list("rewards", function(j)
                return { title = string.rep("r", 300), description = "desc", decorID = j,
                    decorQuantity = 2, favor = 3, money = 4, rewardQuestID = 5 }
            end) } end) }
    setup({ GetNeighborhoodInitiativeInfo = function() calls = calls + 1; return info, opaque(), nil end,
        GetInitiativeActivityLogInfo = function() calls = calls + 1; return { taskActivity = {
            {}, {}, {}, {}, {}, {}, {}, { amount = 88 } } } end,
        GetTrackedInitiativeTasks = function() calls = calls + 1; return { trackedIDs = { 1, 2, 3, 4 } } end,
        GetInitiativeTaskInfo = function(id) assert(id <= 4); calls = calls + 1; return { ID = id } end,
        GetInitiativeTaskChatLink = function(id) assert(id <= 4); calls = calls + 1; return "link" end })
    local r = capture(); local got = initiative(r)
    assert(got.taskEntries, "missing initiative nested task observations")
    assert(got.milestoneEntries, "missing initiative nested milestone observations")
    assert(got.fields.tasks.kind == "table" and got.fields.tasks.entries == nil)
    assert(got.fields.milestones.kind == "table" and got.fields.title.value == "original")
    assert(got.fields.description.value == "kept" and r[names[1]].n == 3)
    assert(r[names[1]].values[2].fields == nil)
    assert(got.taskEntries.entries[4].fields.ID.value == 104)
    assert(got.taskEntries.entries[4].fields.criteriaList.fields == nil)
    local m = got.milestoneEntries.entries[4]
    assert(m.fields.milestoneOrderIndex.value == 4 and m.fields.requiredContributionAmount.value == 1004)
    assert(m.fields.rewards.kind == "table" and m.fields.rewards.entries == nil)
    local reward = m.rewardEntries.entries[4].fields
    assert(#reward.title.value == 256 and reward.description.value == "desc" and reward.decorID.value == 4)
    assert(reward.decorQuantity.value == 2 and reward.favor.value == 3 and reward.money.value == 4 and reward.rewardQuestID.value == 5)
    assert(r[names[2]].values[1].fields.taskActivity.entries[8].fields.amount.value == 88)
    assert(tracked(r)[4].info.values[1].fields.ID.value == 4 and calls == 11)
    for i = 2, 11 do capture() end
    assert(calls == 110 and reads.tasks == 40 and reads.milestones == 40 and reads.rewards == 160)
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("nested receivers rechecked at every index and declared field", function()
    local cases = { { "tasks", { 1,2,3,4 } }, { "task", taskKeys },
        { "milestones", { 1,2,3,4 } }, { "milestone", milestoneKeys },
        { "rewards", { 1,2,3,4 } }, { "reward", rewardKeys } }
    for _, case in ipairs(cases) do
        for stop = 1, #case[2] do
            local revoked, reads = false, 0
            local object = setmetatable({}, { __index = function(_, key)
                assert(not revoked); reads = reads + 1; assert(key == case[2][reads])
                if reads == stop then revoked = true end
                return type(key) == "number" and {} or 12
            end })
            setup({ GetNeighborhoodInitiativeInfo = function() return nestedFixture(case[1], object) end })
            canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
            capture(); assert(reads == stop, case[1] .. " receiver boundary")
        end
    end
end)

test("nested serialization revokes later receivers and values before inspection", function()
    for _, kind in ipairs({ "tasks", "task", "milestones", "milestone", "rewards", "reward" }) do
        local revoked, reads = false, 0
        local object = setmetatable({}, { __index = function() assert(not revoked); reads = reads + 1; return {} end })
        local marker = {}
        local info = nestedFixture(kind, object)
        info.description = marker
        setup({ GetNeighborhoodInitiativeInfo = function() return info end })
        canaccessvalue = function(v)
            if rawequal(v, marker) then revoked = true end
            return not rawequal(v, secret) and not (revoked and rawequal(v, object))
        end
        capture(); assert(reads == 0, kind .. " revoked by earlier field")
    end
    for _, kind in ipairs({ "task", "milestone", "reward" }) do
        local revoked = false
        local keys = kind == "task" and taskKeys or (kind == "milestone" and milestoneKeys or rewardKeys)
        local value = opaque()
        local object = setmetatable({}, { __index = function(_, key)
            if key == keys[1] then revoked = true; return value end
        end })
        setup({ GetNeighborhoodInitiativeInfo = function() return nestedFixture(kind, object) end })
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, value)) end
        local got = initiative(capture())
        local entry = kind == "task" and got.taskEntries.entries[1] or got.milestoneEntries.entries[1]
        if kind == "reward" then entry = entry.rewardEntries.entries[1] end
        assert(entry.fields[keys[1]].status == "restricted")
    end
end)

test("nested nils field errors and restricted entries do not suppress peers", function()
    local bad = setmetatable({}, { __index = function() error(secret) end })
    setup({ GetNeighborhoodInitiativeInfo = function() return {
        tasks = { secret, nil, bad, { ID = 44, requirementsList = opaque(), criteriaList = opaque() } },
        milestones = { { rewards = { secret, nil, bad, { title = "peer" } } }, nil, bad,
            { milestoneOrderIndex = 4, rewards = false } } } end })
    local got = initiative(capture())
    assert(got.taskEntries.entries[1].status == "restricted" and got.taskEntries.entries[2].kind == "nil")
    assert(got.taskEntries.entries[3].fields.ID.status == "field-error")
    assert(got.taskEntries.entries[4].fields.ID.value == 44)
    local rewards = got.milestoneEntries.entries[1].rewardEntries.entries
    assert(rewards[1].status == "restricted" and rewards[2].kind == "nil")
    assert(rewards[3].fields.title.status == "field-error" and rewards[4].fields.title.value == "peer")
    assert(got.milestoneEntries.entries[2].kind == "nil")
    assert(got.milestoneEntries.entries[4].rewardEntries.value == false)
end)

test("nested tables and userdata are collectible and excluded fields stay opaque", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup({ GetNeighborhoodInitiativeInfo = function()
        local reward = newproxy(true)
        getmetatable(reward).__index = function(_, key)
            for _, name in ipairs(rewardKeys) do if key == name then return 1 end end
            return forbidden()
        end
        local task = { ID = 1, requirementsList = opaque(), criteriaList = opaque() }
        local rewards = { reward }; local milestone = { rewards = rewards }
        local tasks, milestones = { task }, { milestone }
        local info = { tasks = tasks, milestones = milestones }
        for i, value in ipairs({ info, tasks, task, milestones, milestone, rewards, reward }) do weak[i] = value end
        return info
    end })
    local got = initiative(capture())
    assert(got.milestoneEntries.entries[1].rewardEntries.entries[1].fields.money.value == 1)
    collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)

print(string.format("%d neighborhood-structures fixtures passed", passed))
