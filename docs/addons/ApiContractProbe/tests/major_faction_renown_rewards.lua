local root = assert(arg[1], "addon directory required")
local passed, failed, calls = 0, 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspection") end
local fields = { "renownRewardID", "uiOrder", "isAccountUnlock", "itemID", "spellID", "mountID", "transmogID", "transmogSetID", "titleMaskID", "transmogIllusionSourceID", "icon", "name", "description", "toastDescription", "rewardType" }
local function setup(ids, levels, rewards)
    ApiContractProbeDB, SlashCmdList, calls = nil, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_MajorFactions = {
        GetMajorFactionIDs = function(...) calls = calls + 1; assert(select("#", ...) == 1 and (...) == nil); return ids() end,
        GetRenownLevels = function(...) calls = calls + 1; assert(select("#", ...) == 1); return levels(...) end,
        GetRenownRewardsForLevel = function(...) calls = calls + 1; assert(select("#", ...) == 2); return rewards(...) end,
    }
    setmetatable(C_MajorFactions, { __index = function(_, key) error("excluded API " .. key) end })
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("major-faction-renown-rewards " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "renown rewards mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].majorFactionRenownRewards)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("original duplicate fractional faction and level pair plus all declared fields", function()
    local reward = {}; for i, key in ipairs(fields) do reward[key] = i end
    setup(function() return { 12.5, 12.5 } end, function(id)
        assert(id == 12.5); return { { factionID = 999, level = -2.25, locked = false, isMilestone = true, isCapstone = false } }
    end, function(id, level) assert(id == 12.5 and level == -2.25); return { reward } end)
    local r = capture(); assert(calls == 5)
    local level = r.entries[1].levels.entries[1]
    assert(level.fields.factionID.value == 999 and level.fields.locked.value == false)
    for i, key in ipairs(fields) do assert(level.rewards.entries[1].fields[key].value == i) end
    assert(level.rewards.entries[1].fields.isCollected == nil)
end)
test("exact arity nil holes and opaque errors stay independent", function()
    setup(function() return { 1, 2 }, nil, 9 end, function(id)
        if id == 1 then error(secret) end
        return { { level = 3 }, { level = 4 } }, nil, 7
    end, function(_, level) if level == 3 then error(secret) end; return nil, { {} }, nil end)
    local r = capture(); assert(r.producer.n == 3 and r.producer.values[2].kind == "nil")
    assert(r.entries[1].levels.status == "call-error")
    assert(r.entries[2].levels.n == 3)
    assert(r.entries[2].levels.entries[1].rewards.status == "call-error")
    local q = r.entries[2].levels.entries[2].rewards
    assert(q.n == 3 and q.values[3].kind == "nil" and q.entries == nil)
end)
test("invalid inputs and first return only never invent levels", function()
    setup(function() return { secret, false, "2", math.huge, 0/0, 6 } end,
        function(id) assert(id == 6); return { { level = secret }, { level = "3" }, { level = math.huge }, { level = 4.5 } } end,
        function(id, level) assert(id == 6 and level == 4.5); return false, { {} } end)
    local r = capture(); assert(calls == 3)
    assert(r.entries[1].levels.status == "restricted-input")
    assert(r.entries[6].levels.entries[4].rewards.entries == nil)
end)
test("both pair inputs revoked during lookup and function guards", function()
    for _, target in ipairs({ "GetRenownLevels", "GetRenownRewardsForLevel" }) do
        for _, phase in ipairs({ "lookup", "secret", "access" }) do
            for _, value in ipairs({ 12.5, 3.25 }) do
                local revoked = false
                setup(function() return { 12.5 } end, function() return { { level = 3.25 } } end, function() error("revoked pair forwarded") end)
                local original = C_MajorFactions[target]
                if phase == "lookup" then C_MajorFactions[target] = nil; setmetatable(C_MajorFactions, { __index = function(_, key)
                    if key == target then revoked = true; return original end
                end }) end
                issecretvalue = function(v) if phase == "secret" and rawequal(v, original) then revoked = true end; return rawequal(v, secret) end
                canaccessvalue = function(v)
                    if phase == "access" and rawequal(v, original) then revoked = true end
                    return not rawequal(v, secret) and not (revoked and v == value)
                end
                local q = capture().entries[1].levels
                if target == "GetRenownLevels" and value == 12.5 then assert(q.status == "restricted-input")
                else assert(q.entries[1].rewards.status == "restricted-input") end
            end
        end
    end
end)
test("entry field serialization can revoke original level before downstream use", function()
    local revoked = false
    setup(function() return { 1 } end, function() return { { level = 2, isCapstone = "revoke" } } end, function() error("revoked level") end)
    canaccessvalue = function(v) if v == "revoke" then revoked = true end; return not (revoked and v == 2) end
    assert(capture().entries[1].levels.entries[1].rewards.status == "restricted-input" and calls == 2)
end)
test("every level and reward field receiver rechecked", function()
    for _, keys in ipairs({ { "factionID", "level", "locked", "isMilestone", "isCapstone" }, fields }) do
        for _, stop in ipairs(keys) do
            local revoked = false
            local obj = setmetatable({}, { __index = function(_, key) assert(not revoked); if key == stop then revoked = true end; return 3 end })
            setup(function() return { 1 } end, function() return { keys == fields and { level = 2 } or obj } end,
                function() return { obj } end)
            canaccessvalue = function(v) return not (rawequal(v, obj) and revoked) end
            local entry = capture().entries[1].levels.entries[1]
            if keys == fields then entry = entry.rewards.entries[1] end
            local after = false
            for _, key in ipairs(keys) do if after then assert(entry.fields[key].status == "field-error") end; if key == stop then after = true end end
        end
    end
end)
test("every list index receiver rechecked at all three levels", function()
    for depth = 1, 3 do
        local bound = depth == 1 and 8 or 4
        for stop = 1, bound do
            local revoked = false
            local list = setmetatable({}, { __index = function(_, index)
                assert(not revoked); if index == stop then revoked = true end
                return depth == 1 and 1 or (depth == 2 and { level = 2 } or { rewardType = 3 })
            end })
            setup(function() return depth == 1 and list or { 1 } end,
                function() return depth == 2 and list or { { level = 2 } } end,
                function() return list end)
            canaccessvalue = function(v) return not (rawequal(v, list) and revoked) end
            local r = capture(); local entries = r.entries
            if depth > 1 then entries = entries[1].levels.entries end
            if depth > 2 then entries = entries[1].rewards.entries end
            if stop < bound then assert(entries[stop + 1].status == "field-error") end
        end
    end
end)
test("missing namespace functions and secret guards fail closed", function()
    setup(function() return { 1 } end, function() return { { level = 2 } } end, function() return {} end)
    C_MajorFactions.GetRenownLevels = secret
    assert(capture().entries[1].levels.status == "missing-api")
    C_MajorFactions = secret; assert(capture().producer.status == "field-error")
    canaccessvalue = nil; SlashCmdList.APICONTRACTPROBE("major-faction-renown-rewards missing-guard")
    assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].status == "missing-access-api")
end)
test("41 calls eight factions four levels four rewards and scalar bounds", function()
    local ids, levels, rewards, extra = {}, {}, {}, {}
    for i = 1, 20 do ids[i] = i; levels[i] = { level = i }; rewards[i] = { name = string.rep("x", 300) }; extra[i] = string.rep("y", 300) end
    setup(function() return ids, unpack(extra) end, function() return levels end, function() return rewards, unpack(extra) end)
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(calls == 410 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local row = ApiContractProbeDB.captures[1]; assert(#row.label == 128)
    local r = row.majorFactionRenownRewards; assert(#r.entries == 8 and #r.producer.values == 16 and r.producer.truncated)
    local q = r.entries[1].levels; assert(#q.entries == 4)
    q = q.entries[1].rewards; assert(#q.entries == 4 and #q.values == 16 and q.truncated and #q.entries[1].fields.name.value == 256)
end)
test("objects collected and all excludes recorder", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local t = { 1 }; weak[1] = t; return t end,
        function() local t = { { level = 2 } }; weak[2] = t; return t end,
        function() local t = { { rewardType = secret } }; weak[3] = t; return t end)
    capture(); collectgarbage(); collectgarbage(); assert(weak[1] == nil and weak[2] == nil and weak[3] == nil)
    local before = calls; SlashCmdList.APICONTRACTPROBE("all other"); assert(calls == before)
end)
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
