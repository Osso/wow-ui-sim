local root = assert(arg[1], "addon directory required")
local passed, failed, calls, forbiddenCalls = 0, 0, {}, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspected") end
getmetatable(secret).__tostring = function() error("secret serialized") end
local names = { "GetAvailableCategories", "GetAvailableActivityGroups", "GetAvailableActivities",
    "DoesEntryTitleMatchPrebuiltTitle" }
local categories = { 11.5, 22.5 }
local groups = { [11.5] = { 31.5, 32.5 }, [22.5] = { 41.5, 42.5 } }
local activities = { [31.5] = { 101.5, 102.5 }, [32.5] = { 103.5, 104.5 },
    [41.5] = { 201.5, 202.5 }, [42.5] = { 203.5, 204.5 } }
local function pack(...) return { n = select("#", ...), ... } end
local function count(name)
    local n = 0
    for _, call in ipairs(calls) do if call.name == name then n = n + 1 end end
    return n
end
local function setup(overrides)
    ApiContractProbeDB, SlashCmdList, calls, forbiddenCalls = nil, {}, {}, 0
    issecretvalue = function(value) return rawequal(value, secret) end
    canaccessvalue = function(value) return not rawequal(value, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { LFGListFilter = { PvE = 51.5 } }
    local functions = {
        GetAvailableCategories = function(...)
            assert(select("#", ...) == 1 and (...) == 51.5)
            return { categories[1], categories[2], 999 }
        end,
        GetAvailableActivityGroups = function(...)
            local category, filter = ...
            assert(select("#", ...) == 2 and filter == 51.5)
            local list = assert(groups[category]); return { list[1], list[2], 999 }
        end,
        GetAvailableActivities = function(...)
            local category, group, filter = ...
            assert(select("#", ...) == 3 and filter == 51.5)
            assert(group == groups[category][1] or group == groups[category][2], "category/group relationship lost")
            local list = assert(activities[group]); return { list[1], list[2], 999 }
        end,
        DoesEntryTitleMatchPrebuiltTitle = function(...)
            local activity, group, playstyle, general = ...
            assert(select("#", ...) == 4 and playstyle == nil and general == nil, "explicit nil arity lost")
            assert(activity == activities[group][1] or activity == activities[group][2], "group/activity relationship lost")
            return true, nil, "observation", nil
        end,
    }
    C_LFGList = {}
    for _, name in ipairs(names) do
        local fn = overrides and overrides[name] or functions[name]
        C_LFGList[name] = function(...)
            calls[#calls + 1] = { name = name, args = pack(...) }
            return fn(...)
        end
    end
    local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded API invoked") end
    for _, name in ipairs({ "Search", "GetSearchResults", "GetSearchResultInfo", "GetActiveEntryInfo",
        "CreateListing", "UpdateListing", "RemoveListing", "ApplyToGroup", "SetEntryTitle", "GetActivityInfoTable",
        "GetPlaystyleString", "RequestAvailableActivities" }) do C_LFGList[name] = forbidden end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("lfg-title-match-read " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "lfg-title-match-read mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].lfgTitleMatchRead, "title-match capture absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("original selected-flow pairs and exact explicit-nil predicate arity", function()
    setup(); local result = capture()
    assert(result.experiment == "explicit-nil-playstyles")
    assert(#calls == 15 and count(names[1]) == 1 and count(names[2]) == 2)
    assert(count(names[3]) == 4 and count(names[4]) == 8 and #result.categories == 2)
    for _, category in ipairs(result.categories) do
        assert(#category.groups == 2)
        for _, group in ipairs(category.groups) do
            assert(#group.activities == 2)
            for _, activity in ipairs(group.activities) do
                assert(activity.match.n == 4 and activity.match.values[2].kind == "nil")
                assert(activity.match.values[4].kind == "nil")
            end
        end
    end
    assert(forbiddenCalls == 0)
end)

test("missing restricted or invalid publication never substitutes numeric filters", function()
    for _, value in ipairs({ false, "4", math.huge, -math.huge, 0 / 0, secret }) do
        setup(); Enum.LFGListFilter.PvE = value; capture(); assert(#calls == 0)
    end
    setup(); Enum.LFGListFilter.PvE = nil; capture(); assert(#calls == 0)
    setup(); Enum.LFGListFilter = secret; capture(); assert(#calls == 0)
    setup(); Enum = secret; capture(); assert(#calls == 0)
    setup(); Enum.LFGListFilter = setmetatable({}, { __index = function() error(secret) end })
    capture(); assert(#calls == 0)
end)

test("only first lists and accessible finite original IDs feed later stages", function()
    for _, stage in ipairs({ names[1], names[2], names[3] }) do
        setup({ [stage] = function() return nil, { 99 } end })
        capture(); assert(count(names[4]) == 0)
        setup({ [stage] = function() return { secret, math.huge, 77 } end })
        capture(); assert(count(names[4]) == 0)
        setup({ [stage] = function() return { "1", false } end })
        capture(); assert(count(names[4]) == 0)
    end
end)

test("peer groups and activities continue after opaque errors and nil holes", function()
    setup({ GetAvailableActivities = function(_, group)
        if group == 31.5 then error(secret) end
        return { activities[group][1], nil }, nil
    end, DoesEntryTitleMatchPrebuiltTitle = function(activity)
        if activity == 103.5 then error(secret) end
        if activity == 201.5 then return end
        return nil, false, nil
    end })
    local r = capture()
    assert(count(names[3]) == 4 and count(names[4]) == 3)
    assert(r.categories[1].groups[1].producer.status == "call-error")
    assert(r.categories[1].groups[2].activities[1].match.status == "call-error")
    assert(r.categories[2].groups[1].activities[1].match.n == 0)
    local last = r.categories[2].groups[2].activities[1].match
    assert(last.n == 3 and last.values[1].kind == "nil" and last.values[2].value == false)
end)

test("each query rechecks ancestor and forwarded inputs after API guards", function()
    local positions = { { names[1], 1 }, { names[2], 2 }, { names[3], 4 }, { names[4], 8 } }
    local queryOrder = { [names[1]] = { 1 }, [names[2]] = { 2, 9 },
        [names[3]] = { 3, 6, 10, 13 }, [names[4]] = { 4, 5, 7, 8, 11, 12, 14, 15 } }
    local groupIDs = { 31.5, 32.5, 41.5, 42.5 }
    local activityIDs = { 101.5, 102.5, 103.5, 104.5, 201.5, 202.5, 203.5, 204.5 }
    for _, stage in ipairs(positions) do for pos = 1, stage[2] do
        local inputs = { { value = 51.5 } }
        if stage[1] ~= names[1] then
            local categoryIndex = stage[1] == names[2] and pos or stage[1] == names[3] and math.ceil(pos / 2) or math.ceil(pos / 4)
            inputs[#inputs + 1] = { value = categories[categoryIndex] }
        end
        if stage[1] == names[3] or stage[1] == names[4] then
            inputs[#inputs + 1] = { value = groupIDs[stage[1] == names[3] and pos or math.ceil(pos / 2)] }
        end
        if stage[1] == names[4] then inputs[#inputs + 1] = { value = activityIDs[pos] }; inputs[#inputs + 1] = {} end
        for _, input in ipairs(inputs) do for _, phase in ipairs({ "namespace-secret", "namespace-access", "lookup", "secret", "access" }) do
            setup()
            local original, namespace = C_LFGList[stage[1]], C_LFGList
            local revoked, seen, forbidden = false, 0, 0
            local fn = function(...)
                local forwarded = pack(...)
                if stage[1] == names[4] then
                    local group = forwarded[2]
                    forwarded[5], forwarded[6], forwarded.n = group < 40 and 11.5 or 22.5, 51.5, 6
                end
                for index = 1, forwarded.n do
                    if revoked and rawequal(forwarded[index], input.value) then forbidden = forbidden + 1 end
                end
                return original(...)
            end
            namespace[stage[1]] = nil
            setmetatable(namespace, { __index = function(_, key)
                if key ~= stage[1] then return nil end
                if phase == "lookup" then seen = seen + 1; if seen == pos then revoked = true end end
                return fn
            end })
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, fn) then seen = seen + 1; if seen == pos then revoked = true end end
                if phase == "namespace-secret" and rawequal(v, namespace) then
                    seen = seen + 1; if seen == queryOrder[stage[1]][pos] then revoked = true end
                end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "access" and rawequal(v, fn) then seen = seen + 1; if seen == pos then revoked = true end end
                if phase == "namespace-access" and rawequal(v, namespace) then
                    seen = seen + 1; if seen == queryOrder[stage[1]][pos] then revoked = true end
                end
                return not rawequal(v, secret) and not (revoked and rawequal(v, input.value))
            end
            capture()
            assert(revoked and forbidden == 0, stage[1] .. ":" .. pos .. ":" .. phase)
        end end
    end end
end)

test("every source list receiver is guarded before every bounded index", function()
    for _, stage in ipairs({ names[1], names[2], names[3] }) do
        local revoked, reads = false, 0
        local list = setmetatable({}, { __index = function(_, index)
            assert(not revoked, "revoked list indexed"); assert(index == 1); reads = reads + 1; revoked = true
            return stage == names[1] and 11.5 or stage == names[2] and 31.5 or 101.5
        end, __len = function() error("source list measured") end })
        setup({ [stage] = function() return list end })
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
        capture(); assert(reads == 1)
    end
end)

test("output observations can revoke IDs filters or nil before downstream use", function()
    local marker, revoked = {}, false
    setup({ GetAvailableActivityGroups = function(id) return groups[id], marker end })
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, 51.5))
    end
    capture(); assert(count(names[3]) == 0 and count(names[4]) == 0)
    revoked = false
    setup({ DoesEntryTitleMatchPrebuiltTitle = function() return marker end })
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and v == nil)
    end
    capture(); assert(count(names[4]) == 1)
end)

test("missing throwing namespaces APIs and access guards fail closed", function()
    setup(); C_LFGList.GetAvailableCategories = nil
    assert(capture().producer.status == "missing-api" and #calls == 0)
    setup(); C_LFGList = secret; assert(capture().producer.status == "field-error" and #calls == 0)
    setup(); C_LFGList = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().producer.status == "field-error" and #calls == 0)
    setup(); C_LFGList.DoesEntryTitleMatchPrebuiltTitle = secret; capture(); assert(#calls == 7)
    setup(); canaccessvalue = function() error(secret) end; capture(); assert(#calls == 0)
    setup(); issecretvalue = nil; SlashCmdList.APICONTRACTPROBE("lfg-title-match-read missing")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and #calls == 0)
end)

test("opaque result objects are not inspected invoked or retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup({ DoesEntryTitleMatchPrebuiltTitle = function()
        local v = newproxy(true)
        getmetatable(v).__index = function() error("result inspected") end
        getmetatable(v).__tostring = function() error("result stringified") end
        weak[#weak + 1] = v; return v, secret
    end })
    local r = capture(); assert(count(names[4]) == 8)
    assert(r.categories[1].groups[1].activities[1].match.values[2].status == "restricted")
    calls = {}; collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)

test("two-by-two-by-two traversal and return string label snapshot caps", function()
    local function bounded(a, b)
        return setmetatable({}, { __index = function(_, i) assert(i == 1 or i == 2); return i == 1 and a or b end,
            __len = function() error("source list measured") end })
    end
    setup({ GetAvailableCategories = function() return bounded(11.5, 22.5) end,
        GetAvailableActivityGroups = function(id) return bounded(groups[id][1], groups[id][2]) end,
        GetAvailableActivities = function(_, id) return bounded(activities[id][1], activities[id][2]) end,
        DoesEntryTitleMatchPrebuiltTitle = function()
            local t = {}; for i = 1, 18 do t[i] = string.rep("x", 300) end; return unpack(t)
        end })
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(#calls == 150 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    local r = ApiContractProbeDB.captures[1].lfgTitleMatchRead.categories[1].groups[1].activities[1].match
    assert(r.n == 18 and r.truncated and #r.values == 16 and #r.values[1].value == 256 and r.values[1].truncated)
end)

test("all and unrelated manual modes never invoke title-match or title mutators", function()
    setup(); SlashCmdList.APICONTRACTPROBE("all untouched"); assert(#calls == 0 and forbiddenCalls == 0)
    SlashCmdList.APICONTRACTPROBE("public-queries untouched"); assert(#calls == 0)
    capture(); assert(#calls == 15 and forbiddenCalls == 0)
end)
print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
