local root = assert(arg[1], "addon directory required")
local passed, failed, forbiddenCalls = 0, 0, 0
local specs = { "Resource1Percent", "Resource1Format", "Resource1Voice", "Resource1Volume",
    "Resource2Percent", "Resource2Format", "Resource2Voice", "Resource2Volume", "SayIfTargeted" }
local throttles = { "Sample", "PlayerHealth", "TargetHealth", "PlayerCast", "TargetCast",
    "PlayerResource1", "PlayerResource2", "PlayerHealthSamePercent", "TargetHealthSamePercent",
    "PlayerResource1SamePercent", "PlayerResource2SamePercent" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function forbidden() forbiddenCalls = forbiddenCalls + 1; error("excluded call") end
local function setup(fn)
    ApiContractProbeDB, SlashCmdList, forbiddenCalls = nil, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { CombatAudioAlertSpecSetting = {}, CombatAudioAlertThrottle = {} }
    for i, name in ipairs(specs) do Enum.CombatAudioAlertSpecSetting[name] = 100 + i + 0.25 end
    for i, name in ipairs(throttles) do Enum.CombatAudioAlertThrottle[name] = 200 + i + 0.25 end
    Enum.CombatAudioAlertSpecSetting.Extra = 999
    Enum.CombatAudioAlertThrottle.Extra = 999
    C_CombatAudioAlert = { IsEnabled = fn, GetSpecSetting = fn, GetThrottle = fn,
        SetSpecSetting = forbidden, SetThrottle = forbidden, SpeakText = forbidden,
        AddToKnownTargetingList = forbidden, RemoveFromKnownTargetingList = forbidden }
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("combat-audio-settings-read " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "combat-audio-settings-read mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].combatAudioSettingsRead)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok and forbiddenCalls == 0 then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function query(result, index)
    if index <= 2 then return result.IsEnabled[index] end
    local rows, n = result.specSettings, index - 2
    if index > 20 then rows, n = result.throttles, index - 20 end
    return rows[math.floor((n - 1) / 2) + 1].observations[(n - 1) % 2 + 1]
end

test("fixed names original noncanonical enums and exact 42 arguments", function()
    local calls = 0
    setup(function(...) calls = calls + 1; return calls end)
    C_CombatAudioAlert.IsEnabled = function(...) assert(select("#", ...) == 0); calls = calls + 1; return calls end
    C_CombatAudioAlert.GetSpecSetting = function(...)
        calls = calls + 1
        assert(select("#", ...) == 1 and (...) == 100 + math.floor((calls - 3) / 2) + 1.25)
        return calls
    end
    C_CombatAudioAlert.GetThrottle = function(...)
        calls = calls + 1
        assert(select("#", ...) == 1 and (...) == 200 + math.floor((calls - 21) / 2) + 1.25)
        return calls
    end
    local r = capture()
    assert(calls == 42 and #r.specSettings == 9 and #r.throttles == 11)
    for i = 1, 42 do assert(query(r, i).values[1].value == i) end
    for i, name in ipairs(specs) do assert(r.specSettings[i].name == name) end
    for i, name in ipairs(throttles) do assert(r.throttles[i].name == name) end
end)

test("invalid absent and secret enums never use fallback", function()
    for _, bad in ipairs({ false, "101", math.huge, -math.huge, 0/0, secret }) do
        local calls = 0
        setup(function() calls = calls + 1 end)
        Enum.CombatAudioAlertSpecSetting.Resource1Percent = bad
        Enum.CombatAudioAlertThrottle.Sample = nil
        local r = capture()
        assert(calls == 38 and query(r, 3).status == "unavailable-enum" and query(r, 21).status == "unavailable-enum")
    end
end)

test("publication containers are guarded before member lookup", function()
    for _, level in ipairs({ "root", "spec", "throttle" }) do
        local reads, calls = 0, 0
        setup(function() calls = calls + 1 end)
        local blocked = setmetatable({}, { __index = function() reads = reads + 1; error("blocked lookup") end })
        if level == "root" then Enum = blocked
        elseif level == "spec" then Enum.CombatAudioAlertSpecSetting = blocked
        else Enum.CombatAudioAlertThrottle = blocked end
        canaccessvalue = function(v) return not rawequal(v, secret) and not rawequal(v, blocked) end
        capture()
        assert(reads == 0 and calls == (level == "root" and 2 or level == "spec" and 24 or 20))
    end
end)

test("namespace function and publication lookup errors preserve peers", function()
    setup(nil)
    local r = capture()
    assert(query(r, 1).status == "missing-api" and query(r, 42).status == "missing-api")
    for _, bad in ipairs({ secret, false, 3 }) do
        setup(bad)
        assert(query(capture(), 3).status == "missing-api")
    end
    setup(function() return 1 end)
    C_CombatAudioAlert = secret
    assert(query(capture(), 42).status == "field-error")
    setup(function() return 1 end)
    C_CombatAudioAlert = setmetatable({}, { __index = function() error(secret) end })
    assert(query(capture(), 1).status == "field-error")
    setup(function() return 1 end)
    Enum.CombatAudioAlertSpecSetting = setmetatable({}, { __index = function() error(secret) end })
    r = capture()
    assert(query(r, 3).status == "unavailable-enum" and query(r, 21).status == "observed")
end)

test("all forty enum positions recheck after namespace lookup and both function guards", function()
    for _, stage in ipairs({ "namespace", "lookup", "secret", "access" }) do
        for position = 3, 42 do
            local lookups, calls, revoked = 0, 0, false
            local offset = position <= 20 and 2 or 20
            local blocked = (position <= 20 and 100 or 200) + math.floor((position - offset - 1) / 2) + 1.25
            local fn = function(v) assert(not (revoked and v == blocked)); calls = calls + 1 end
            setup(fn)
            local namespace = setmetatable({}, { __index = function()
                lookups = lookups + 1
                if stage == "lookup" and lookups == position then revoked = true end
                return fn
            end })
            C_CombatAudioAlert = namespace
            issecretvalue = function(v)
                if stage == "secret" and rawequal(v, fn) and lookups == position then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if stage == "namespace" and rawequal(v, namespace) and lookups + 1 == position then revoked = true end
                if stage == "access" and rawequal(v, fn) and lookups == position then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, blocked))
            end
            local r = capture()
            assert(query(r, position).status == "restricted-input", stage .. " position " .. position)
            assert(calls == (position % 2 == 1 and 40 or 41))
        end
    end
end)

test("serialization and earlier outputs can revoke later original inputs", function()
    local calls, hits, revoked = 0, 0, false
    setup(function(v) assert(v ~= 101.25); calls = calls + 1 end)
    canaccessvalue = function(v)
        if rawequal(v, 101.25) then hits = hits + 1; if hits == 2 then revoked = true end end
        return not rawequal(v, secret) and not (revoked and rawequal(v, 101.25))
    end
    local r = capture()
    assert(calls == 40 and query(r, 3).status ~= "observed")
    setup(function() return secret end)
    C_CombatAudioAlert.IsEnabled = function() Enum.CombatAudioAlertThrottle = {}; return false end
    r = capture()
    assert(query(r, 21).status == "unavailable-enum" and query(r, 3).values[1].status == "restricted")
end)

test("zero nil positions opaque errors and fresh function replacement", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil end
        if calls == 3 then error(secret) end
        return nil, secret, false, nil
    end)
    local r = capture()
    assert(calls == 42 and query(r, 1).n == 0 and query(r, 2).n == 1)
    assert(query(r, 3).status == "call-error")
    local q = query(r, 4)
    assert(q.n == 4 and q.values[2].status == "restricted" and q.values[3].value == false and q.values[4].kind == "nil")
    setup(function() return 1 end)
    C_CombatAudioAlert.GetSpecSetting = function()
        C_CombatAudioAlert.GetSpecSetting = function() return "replacement" end
        error(secret)
    end
    r = capture()
    assert(query(r, 3).status == "call-error" and query(r, 4).values[1].value == "replacement")
end)

test("raw tuple string label snapshots and total call bounds", function()
    local calls, values = 0, {}
    for i = 1, 20 do values[i] = string.rep("x", 300) end
    setup(function() calls = calls + 1; return unpack(values) end)
    local r = capture(string.rep("l", 200))
    for i = 1, 42 do
        local q = query(r, i)
        assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
    end
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture() end
    assert(calls == 420 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("returned objects are opaque collectible and never retained", function()
    local refs = setmetatable({}, { __mode = "v" })
    setup(function()
        local value = newproxy(true)
        getmetatable(value).__index = forbidden
        getmetatable(value).__tostring = forbidden
        refs[#refs + 1] = value
        return value
    end)
    local r = capture()
    assert(query(r, 1).values[1].kind == "userdata" and query(r, 42).values[1].kind == "userdata")
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(refs) == nil)
end)

test("missing and throwing access APIs fail closed and all excludes mode", function()
    local calls = 0
    setup(function() calls = calls + 1 end)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("combat-audio-settings-read missing")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(function() calls = calls + 1 end)
    issecretvalue = function() error("no access") end
    capture()
    assert(calls == 0)
    setup(function() calls = calls + 1 end)
    SlashCmdList.APICONTRACTPROBE("all baseline")
    assert(calls == 0 and ApiContractProbeDB.captures[1].combatAudioSettingsRead == nil)
end)

print(string.format("%d passed, %d failed combat-audio-settings-read fixtures", passed, failed))
if failed > 0 then os.exit(1) end
