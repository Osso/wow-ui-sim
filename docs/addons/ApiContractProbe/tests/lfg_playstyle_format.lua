local root = assert(arg[1], "addon directory required")
local passed, failed, calls, forbiddenCalls = 0, 0, {}, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspected") end
getmetatable(secret).__tostring = function() error("secret serialized") end
local names = { "GetAvailableCategories", "GetAvailableActivities", "GetActivityInfoTable", "GetPlaystyleString" }
local infos
local function pack(...) return { n = select("#", ...), ... } end
local function opaque()
    return setmetatable({}, { __index = function() error("activity info traversed") end,
        __len = function() error("activity info measured") end, __tostring = function() error("activity info stringified") end })
end
local function setup(overrides)
    ApiContractProbeDB, SlashCmdList, calls, forbiddenCalls = nil, {}, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { LFGListFilter = { PvE = 41.5 }, LFGEntryPlaystyle = { None = 52.5 },
        LFGEntryGeneralPlaystyle = { None = 63.5 } }
    infos = { [101.5] = opaque(), [102.5] = opaque(), [201.5] = opaque(), [202.5] = opaque() }
    local functions = {
        GetAvailableCategories = function(...) assert(select("#", ...) == 1 and (...) == 41.5); return { 11.5, 22.5, 99 } end,
        GetAvailableActivities = function(...)
            local id, group, filter = ...
            assert(select("#", ...) == 3 and group == 0 and filter == 41.5)
            assert(id == 11.5 or id == 22.5)
            return id == 11.5 and { 101.5, 102.5, 999 } or { 201.5, 202.5, 999 }
        end,
        GetActivityInfoTable = function(...) assert(select("#", ...) == 1); return assert(infos[(...)]) end,
        GetPlaystyleString = function(...)
            local a, b, object = ...
            assert(select("#", ...) == 3 and a == 52.5 and b == 63.5)
            local found = false; for _, info in pairs(infos) do if rawequal(info, object) then found = true end end
            assert(found, "original activity info not forwarded")
            return "formatted"
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
    for _, name in ipairs({ "Search", "GetSearchResults", "GetSearchResultInfo", "GetActiveEntryInfo", "CreateListing",
        "UpdateListing", "RemoveListing", "ApplyToGroup", "SetEntryTitle", "RequestAvailableActivities" }) do
        C_LFGList[name] = forbidden
    end
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("lfg-playstyle-format " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "lfg-playstyle-format mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].lfgPlaystyleFormat)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function count(name)
    local n = 0; for _, call in ipairs(calls) do if call.name == name then n = n + 1 end end; return n
end

test("original published values IDs and opaque info follow exact consumer call shapes", function()
    setup(); local r = capture()
    assert(#calls == 11 and #r.categories == 2)
    assert(count(names[1]) == 1 and count(names[2]) == 2 and count(names[3]) == 4 and count(names[4]) == 4)
    for _, category in ipairs(r.categories) do
        assert(#category.activities == 2)
        for _, activity in ipairs(category.activities) do
            assert(activity.info.values[1].kind == "table")
            assert(activity.formatted.values[1].value == "formatted")
        end
    end
    assert(forbiddenCalls == 0)
end)
test("missing restricted and invalid publication never falls back to zero", function()
    for _, key in ipairs({ "LFGListFilter", "LFGEntryPlaystyle", "LFGEntryGeneralPlaystyle" }) do
        for _, value in ipairs({ false, "0", math.huge, secret }) do
            setup(); Enum[key][key == "LFGListFilter" and "PvE" or "None"] = value
            capture(); assert(count(names[4]) == 0)
            if key == "LFGListFilter" then assert(#calls == 0) else assert(#calls == 7) end
        end
        setup(); Enum[key] = nil; capture(); assert(count(names[4]) == 0)
        setup(); Enum[key] = secret; capture(); assert(count(names[4]) == 0)
    end
    setup(); Enum = secret; capture(); assert(#calls == 0)
end)
test("zero arity nil holes errors and wrong first results stay independent", function()
    setup({ GetActivityInfoTable = function(id)
        if id == 101.5 then return nil, infos[id], nil end
        if id == 102.5 then error(secret) end
        if id == 201.5 then return end
        return infos[id], nil, 7, nil
    end, GetPlaystyleString = function() return nil, "answer", nil end })
    local r = capture(); assert(#calls == 8)
    assert(r.categories[1].activities[1].info.n == 3)
    assert(r.categories[1].activities[2].info.status == "call-error")
    assert(r.categories[2].activities[1].info.n == 0)
    local last = r.categories[2].activities[2]
    assert(last.info.n == 4 and last.formatted.n == 3 and last.formatted.values[3].kind == "nil")
    setup({ GetAvailableActivities = function(id) if id == 11.5 then error(secret) end; return { 201.5 } end })
    r = capture(); assert(r.categories[1].producer.status == "call-error" and count(names[4]) == 1)
end)
test("invalid IDs and inaccessible info never reach downstream calls", function()
    setup({ GetAvailableCategories = function() return { secret, math.huge } end })
    capture(); assert(#calls == 1)
    setup({ GetAvailableActivities = function() return { "101", false } end })
    capture(); assert(#calls == 3)
    for _, value in ipairs({ secret, 1, false, "info" }) do
        setup({ GetActivityInfoTable = function() return value end })
        capture(); assert(#calls == 7 and count(names[4]) == 0)
    end
end)
test("every query position rechecks inputs after namespace lookup and function guards", function()
    local stages = { { names[1], 1 }, { names[2], 2 }, { names[3], 4 }, { names[4], 4 } }
    for _, stage in ipairs(stages) do for pos = 1, stage[2] do
        local candidates = stage[1] == names[1] and { "filter" }
            or stage[1] == names[2] and { "category", "filter", "zero" }
            or stage[1] == names[3] and { "activity" } or { "playstyle", "general", "info" }
        for _, input in ipairs(candidates) do for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
            setup()
            local original, namespace = C_LFGList[stage[1]], C_LFGList
            local ids = { 101.5, 102.5, 201.5, 202.5 }
            local target = input == "filter" and 41.5 or input == "category" and ({ 11.5, 22.5 })[pos]
                or input == "zero" and 0 or input == "activity" and ids[pos]
                or input == "playstyle" and 52.5 or input == "general" and 63.5 or infos[ids[pos]]
            local revoked, seen, forbidden = false, 0, 0
            local function fn(...)
                for i = 1, select("#", ...) do
                    if revoked and rawequal(select(i, ...), target) then forbidden = forbidden + 1 end
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
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "access" and rawequal(v, fn) then seen = seen + 1; if seen == pos then revoked = true end end
                return not rawequal(v, secret) and not (revoked and rawequal(v, target))
            end
            if phase == "namespace" then
                -- Target the namespace guard immediately before this stage's position.
                local totals = stage[1] == names[1] and 1 or stage[1] == names[2] and (pos == 1 and 2 or 7)
                    or stage[1] == names[3] and ({ 3, 5, 8, 10 })[pos] or ({ 4, 6, 9, 11 })[pos]
                canaccessvalue = function(v)
                    if rawequal(v, namespace) then seen = seen + 1; if seen == totals then revoked = true end end
                    return not rawequal(v, secret) and not (revoked and rawequal(v, target))
                end
            end
            capture(); assert(revoked and forbidden == 0, stage[1] .. ":" .. pos .. ":" .. input .. ":" .. phase)
        end end
    end end
end)
test("list receiver access is checked before every bounded index", function()
    for _, which in ipairs({ "categories", "activities" }) do
        local revoked, reads = false, 0
        local list = setmetatable({}, { __index = function(_, index)
            assert(not revoked, "revoked list indexed"); reads = reads + 1; revoked = true
            return which == "categories" and 11.5 or 101.5
        end, __len = function() error("list measured") end })
        setup(which == "categories" and { GetAvailableCategories = function() return list end }
            or { GetAvailableCategories = function() return { 11.5 } end, GetAvailableActivities = function() return list end })
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
        capture(); assert(reads == 1 and count(names[4]) >= 1)
    end
end)
test("earlier output observation can revoke later input or activity info", function()
    local marker, revoked = {}, false
    setup({ GetActivityInfoTable = function(id) return infos[id], marker end })
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, infos[101.5]))
    end
    capture(); assert(count(names[4]) == 3)
    revoked = false
    setup({ GetPlaystyleString = function() return marker end })
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, 102.5))
    end
    capture(); assert(count(names[3]) == 3 and count(names[4]) == 3)
end)
test("missing throwing namespaces functions and guards fail explicitly", function()
    setup(); C_LFGList.GetAvailableCategories = nil
    assert(capture().producer.status == "missing-api" and #calls == 0)
    setup(); C_LFGList = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().producer.status == "field-error" and #calls == 0)
    setup(); C_LFGList = secret; assert(capture().producer.status == "field-error")
    setup(); C_LFGList.GetPlaystyleString = secret; capture(); assert(#calls == 7)
    setup(); canaccessvalue = function() error(secret) end; capture(); assert(#calls == 0)
    setup(); issecretvalue = nil; SlashCmdList.APICONTRACTPROBE("lfg-playstyle-format missing")
    assert(ApiContractProbeDB and ApiContractProbeDB.captures[1].status == "missing-access-api")
end)
test("userdata info and opaque returned objects are never traversed or retained", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup({ GetActivityInfoTable = function()
        local v = newproxy(true); getmetatable(v).__index = function() error("info traversed") end
        getmetatable(v).__tostring = function() error("info serialized") end
        weak[#weak + 1] = v; return v
    end, GetPlaystyleString = function(a, b, v)
        assert(type(v) == "userdata")
        local out = opaque(); weak[#weak + 1] = out; return out, secret
    end })
    local r = capture(); assert(count(names[4]) == 4 and r.categories[1].activities[1].formatted.values[2].status == "restricted")
    calls = {}; collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)
test("two by two traversal tuple strings labels and ten snapshots stay bounded", function()
    local function list(a, b)
        return setmetatable({}, { __index = function(_, i) assert(i == 1 or i == 2); return i == 1 and a or b end,
            __len = function() error("length") end })
    end
    setup({ GetAvailableCategories = function() return list(11.5, 22.5) end,
        GetAvailableActivities = function(id) return id == 11.5 and list(101.5, 102.5) or list(201.5, 202.5) end,
        GetPlaystyleString = function()
            local t = {}; for i = 1, 18 do t[i] = string.rep("x", 300) end; return unpack(t)
        end })
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(#calls == 110 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local saved = ApiContractProbeDB.captures[1]; assert(#saved.label == 128)
    local r = saved.lfgPlaystyleFormat.categories[1].activities[1].formatted
    assert(r.n == 18 and r.truncated and #r.values == 16 and #r.values[1].value == 256 and r.values[1].truncated)
end)
test("all and unrelated manual modes do not invoke the LFG chain", function()
    setup(); SlashCmdList.APICONTRACTPROBE("all untouched"); assert(#calls == 0 and forbiddenCalls == 0)
    SlashCmdList.APICONTRACTPROBE("public-queries untouched"); assert(#calls == 0)
    capture(); assert(#calls == 11 and forbiddenCalls == 0)
end)
print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
