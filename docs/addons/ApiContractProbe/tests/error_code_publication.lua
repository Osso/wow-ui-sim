local root = assert(arg[1], "addon directory required")
local names = {
    "LE_GAME_ERR_CHARTER_NEIGHBORHOOD_OWNERSHIP_TRANSFER_SUCCESS",
    "LE_GAME_ERR_CHARTER_NEIGHBORHOOD_RENAME_NOTIFICATION_S",
    "LE_GAME_ERR_CHARTER_SIGNATURE_REMOVED",
    "LE_GAME_ERR_ENDEAVOR_REWARD_AVAILABLE",
    "LE_GAME_ERR_HOUSING_EXTERIOR_FAILSAFE_RESET",
    "LE_GAME_ERR_HOUSING_RESULT_COSMETIC_OWNER_NOT_IN_GUILD",
    "LE_GAME_ERR_HOUSING_RESULT_MISSING_PRIVATE_NEIGHBORHOOD_INVITE",
    "LE_GAME_ERR_HOUSING_RESULT_PLOT_NOT_VACANT",
    "LE_GAME_ERR_HOUSING_RESULT_PLOT_RESERVED",
    "LE_GAME_ERR_LFG_JOINED_TRAINING_GROUNDS_QUEUE",
    "LE_GAME_ERR_PVP_TRAINING_GROUNDS_DISABLED",
    "LE_GAME_ERR_SOLO_JOIN_TRAINING_GROUND",
}
local globals, initialMeta = _G, getmetatable(_G)
local expected = {}
for index, name in ipairs(names) do assert(not expected[name]); expected[name] = index end
local passed, total = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret field inspection") end
getmetatable(secret).__tostring = function() error("secret stringify") end

local function setup()
    setmetatable(globals, initialMeta)
    for _, name in ipairs(names) do rawset(globals, name, nil) end
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(value) return rawequal(value, secret) end
    canaccessvalue = function(value) return not rawequal(value, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    C_CVar = nil
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end

local function publish(lookup)
    setmetatable(globals, { __index = function(_, key)
        if expected[key] then return lookup(key, expected[key]) end
        if initialMeta and type(initialMeta.__index) == "function" then
            return initialMeta.__index(globals, key)
        end
    end })
end

local function capture(label)
    SlashCmdList.APICONTRACTPROBE("error-code-publication " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual error-code-publication mode absent")
    local rows = assert(ApiContractProbeDB.captures[1].errorCodePublication, "error code observations absent")
    assert(#rows == 12)
    for index, name in ipairs(names) do assert(rows[index].name == name) end
    return rows
end

local function test(name, fn)
    total = total + 1
    local ok, err = pcall(fn)
    setmetatable(globals, initialMeta)
    if ok then passed = passed + 1; print("PASS " .. name)
    else print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("exact twelve distinct globals preserve noncanonical values", function()
    setup()
    local lookups, seen = 0, {}
    publish(function(name, index)
        lookups = lookups + 1
        assert(not seen[name], "duplicate lookup")
        seen[name] = true
        return -index - 0.125
    end)
    local rows = capture()
    assert(lookups == 12)
    for index, row in ipairs(rows) do
        assert(row.status == "observed" and row.kind == "number" and row.value == -index - 0.125)
    end
end)

test("absent globals remain missing without numeric defaults", function()
    setup()
    local lookups = 0
    publish(function() lookups = lookups + 1 end)
    local rows = capture()
    assert(lookups == 12)
    for _, row in ipairs(rows) do assert(row.status == "missing" and row.kind == "nil" and row.value == nil) end
end)

test("zero false and empty strings remain observed values", function()
    setup()
    publish(function(_, index)
        if index % 3 == 1 then return 0 end
        if index % 3 == 2 then return false end
        return ""
    end)
    local rows = capture()
    for index, row in ipairs(rows) do
        assert(row.status == "observed")
        if index % 3 == 1 then assert(row.value == 0)
        elseif index % 3 == 2 then assert(row.value == false)
        else assert(row.value == "") end
    end
end)

test("nonfinite values are marked without serialization", function()
    setup()
    publish(function(_, index)
        if index % 3 == 1 then return 0 / 0 end
        if index % 3 == 2 then return math.huge end
        return -math.huge
    end)
    local rows = capture()
    for _, row in ipairs(rows) do
        assert(row.status == "nonfinite" and row.kind == "number" and row.value == nil)
    end
end)

test("secret and inaccessible values stay opaque at every position", function()
    for blocked = 1, 12 do
        for _, phase in ipairs({ "secret", "access" }) do
            setup()
            local value = newproxy(true)
            getmetatable(value).__index = function() error("opaque field read") end
            getmetatable(value).__tostring = function() error("opaque stringify") end
            if phase == "secret" then issecretvalue = function(v) return rawequal(v, value) end
            else canaccessvalue = function(v) return not rawequal(v, value) end end
            publish(function(_, index) if index == blocked then return value end; return index + 0.5 end)
            local rows = capture()
            for index, row in ipairs(rows) do
                if index == blocked then assert(row.status == "restricted" and row.kind == nil and row.value == nil)
                else assert(row.status == "observed" and row.value == index + 0.5) end
            end
        end
    end
end)

test("throwing global lookups remain independent opaque errors", function()
    for blocked = 1, 12 do
        setup()
        local lookups = 0
        publish(function(_, index)
            lookups = lookups + 1
            if index == blocked then error(secret) end
            return index
        end)
        local rows = capture()
        assert(lookups == 12)
        for index, row in ipairs(rows) do
            if index == blocked then assert(row.status == "lookup-error" and row.value == nil)
            else assert(row.value == index) end
        end
    end
end)

test("global receiver guards prevent all field access", function()
    for _, phase in ipairs({ "secret", "access" }) do
        setup()
        local lookups = 0
        publish(function() lookups = lookups + 1; error("unguarded global lookup") end)
        if phase == "secret" then issecretvalue = function(v) return rawequal(v, globals) end
        else canaccessvalue = function(v) return not rawequal(v, globals) end end
        local rows = capture()
        assert(lookups == 0)
        for _, row in ipairs(rows) do assert(row.status == "lookup-error") end
    end
end)

test("value observation revokes the global receiver before later lookups", function()
    for revokeAt = 1, 12 do
        for _, phase in ipairs({ "secret", "access" }) do
            setup()
            local lookups, revoked = 0, false
            local marker = {}
            publish(function(_, index)
                assert(not revoked, "lookup after receiver access revoked")
                lookups = lookups + 1
                if index == revokeAt then return marker end
                return index
            end)
            if phase == "secret" then
                issecretvalue = function(value)
                    if rawequal(value, marker) then revoked = true end
                    return revoked and rawequal(value, globals)
                end
            else
                canaccessvalue = function(value)
                    if rawequal(value, marker) then revoked = true end
                    return not (revoked and rawequal(value, globals))
                end
            end
            local rows = capture()
            assert(lookups == revokeAt)
            for index, row in ipairs(rows) do
                if index > revokeAt then assert(row.status == "lookup-error")
                else assert(row.status == "observed") end
            end
        end
    end
end)

test("looked-up functions and objects are never invoked or retained", function()
    setup()
    local calls, lookups = 0, 0
    local weak = setmetatable({}, { __mode = "v" })
    publish(function(_, index)
        lookups = lookups + 1
        local value
        if index % 3 == 1 then value = function() calls = calls + 1; error("error generation forbidden") end
        elseif index % 3 == 2 then
            value = setmetatable({}, { __index = function() error("table traversal") end,
                __tostring = function() error("table stringify") end })
        else
            value = newproxy(true)
            getmetatable(value).__index = function() error("userdata traversal") end
            getmetatable(value).__tostring = function() error("userdata stringify") end
        end
        weak[index] = value
        return value
    end)
    local rows = capture()
    assert(calls == 0 and lookups == 12)
    for index, row in ipairs(rows) do
        assert(row.status == "observed" and row.value == nil)
        assert(row.kind == ({ "function", "table", "userdata" })[(index - 1) % 3 + 1])
    end
    collectgarbage("collect")
    assert(next(weak) == nil)
end)

test("binary strings labels and ten snapshots are bounded", function()
    setup()
    local text = string.rep("x\0\255", 100)
    local lookups = 0
    publish(function() lookups = lookups + 1; return text end)
    local rows = capture(string.rep("l", 200))
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _, row in ipairs(rows) do
        assert(row.kind == "string" and row.value == string.sub(text, 1, 256) and row.truncated)
    end
    for index = 2, 11 do SlashCmdList.APICONTRACTPROBE("error-code-publication limit") end
    assert(lookups == 120 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("missing or throwing access APIs fail closed", function()
    for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
        for _, behavior in ipairs({ "missing", "throw" }) do
            setup()
            local lookups = 0
            publish(function() lookups = lookups + 1; error("unguarded lookup") end)
            if behavior == "missing" then rawset(globals, guard, nil)
            else rawset(globals, guard, function() error(secret) end) end
            SlashCmdList.APICONTRACTPROBE("error-code-publication guarded")
            assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual mode absent")
            local record = ApiContractProbeDB.captures[1]
            if behavior == "missing" then assert(record.status == "missing-access-api")
            else
                assert(record.errorCodePublication and #record.errorCodePublication == 12)
                for _, row in ipairs(record.errorCodePublication) do assert(row.status == "lookup-error") end
            end
            assert(lookups == 0)
        end
    end
end)

test("publication and all retain their target routes without error-code lookups", function()
    setup()
    local lookups, cvarReads = 0, 0
    publish(function(_, index) lookups = lookups + 1; return index + 0.75 end)
    C_CVar = {
        GetCVar = function(name) cvarReads = cvarReads + 1; return "current:" .. name end,
        GetCVarDefault = function(name) cvarReads = cvarReads + 1; return "default:" .. name end,
    }
    local function checkPublication(record)
        assert(record.errorCodePublication == nil)
        assert(#record.publication == #ApiContractProbeTargets.publication)
        for index, target in ipairs(ApiContractProbeTargets.publication) do
            local row = record.publication[index]
            assert(row.id == target.id and row.plan == target.plan and row.owner == target.owner)
            if target.owner == "cvar" then
                assert(row.current.values[1].value == "current:" .. target.path)
                assert(row.default.values[1].value == "default:" .. target.path)
            end
        end
    end
    SlashCmdList.APICONTRACTPROBE("publication before")
    checkPublication(ApiContractProbeDB.captures[1])
    local firstReads = cvarReads
    assert(firstReads > 0 and lookups == 0)
    SlashCmdList.APICONTRACTPROBE("error-code-publication between")
    assert(ApiContractProbeDB.captures[2].errorCodePublication, "manual mode absent")
    assert(lookups == 12 and cvarReads == firstReads)
    SlashCmdList.APICONTRACTPROBE("all excluded")
    checkPublication(ApiContractProbeDB.captures[3])
    assert(lookups == 12 and cvarReads == firstReads * 2)
    SlashCmdList.APICONTRACTPROBE("publication after")
    checkPublication(ApiContractProbeDB.captures[4])
    assert(lookups == 12 and cvarReads == firstReads * 3)
end)

print(string.format("%d/%d error-code-publication fixtures passed", passed, total))
assert(passed == total, "error-code-publication fixture failures")
