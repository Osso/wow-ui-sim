local root = assert(arg[1])
local names = { "LE_EXPANSION_CLASSIC", "LE_EXPANSION_LEVEL_CURRENT" }
local fields = { "glueAmbianceSoundKit", "glueCreditsSoundKit", "glueMusicSoundKit" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local passed, failed = 0, 0

local function setup(fn)
    local env = setmetatable({}, { __index = _G })
    env._G, env.SlashCmdList = env, {}
    env.LE_EXPANSION_CLASSIC, env.LE_EXPANSION_LEVEL_CURRENT = 17.25, -3.5
    env.GetExpansionDisplayInfo = fn
    env.issecretvalue = function(v) return rawequal(v, secret) end
    env.canaccessvalue = function(v) return not rawequal(v, secret) end
    env.GetBuildInfo, env.time = function() return "fixture" end, function() return 42 end
    env.print = function() end
    env.sideEffects = 0
    local function forbidden() env.sideEffects = env.sideEffects + 1; error("playback or mutation") end
    env.PlaySound, env.PlayMusic, env.PlaySoundFile = forbidden, forbidden, forbidden
    env.C_Sound = { PlaySound = forbidden }
    env.C_AddOns = { LoadAddOn = forbidden }
    env.LoadAddOn, env.SetCVar = forbidden, forbidden
    -- The generated system has no Namespace; a namespaced substitute must never be used.
    env.C_Expansion = { GetExpansionDisplayInfo = forbidden }
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then setfenv(assert(loadfile(root .. "/" .. line)), env)() end
    end
    toc:close()
    return env
end

local function capture(env, label)
    env.SlashCmdList.APICONTRACTPROBE("expansion-audio-fields " .. (label or "sample"))
    assert(env.ApiContractProbeDB and #env.ApiContractProbeDB.captures > 0, "expansion-audio-fields mode absent")
    assert(env.sideEffects == 0)
    return assert(env.ApiContractProbeDB.captures[#env.ApiContractProbeDB.captures].expansionAudioFields)
end

local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("published noncanonical globals exact one-argument calls and three fields", function()
    local calls = 0
    local env = setup(function(...)
        calls = calls + 1
        assert(select("#", ...) == 1 and (...) == (calls <= 2 and 17.25 or -3.5))
        return { glueAmbianceSoundKit = calls, glueCreditsSoundKit = false, glueMusicSoundKit = 19,
            logo = secret, features = secret }
    end)
    local r = capture(env)
    assert(calls == 4 and #r.expansions == 2)
    for i, row in ipairs(r.expansions) do
        assert(row.name == names[i])
        for j, q in ipairs(row.observations) do
            assert(q.n == 1 and q.input.value == env[names[i]])
            local f = q.values[1].fields
            assert(f.glueAmbianceSoundKit.value == (i - 1) * 2 + j)
            assert(f.glueCreditsSoundKit.value == false and f.glueMusicSoundKit.value == 19)
            local n = 0; for _ in pairs(f) do n = n + 1 end; assert(n == 3)
        end
    end
end)

test("missing nonnumeric nonfinite restricted globals have no fallback", function()
    for _, bad in ipairs({ false, "17", math.huge, -math.huge, 0/0, secret }) do
        local calls = 0
        local env = setup(function() calls = calls + 1 end)
        env.LE_EXPANSION_CLASSIC = bad
        assert(capture(env).expansions[1].observations[1].status ~= "observed")
        assert(calls == 2)
    end
    local env = setup(function() error("missing input invoked") end)
    env.LE_EXPANSION_CLASSIC, env.LE_EXPANSION_LEVEL_CURRENT = nil, nil
    assert(capture(env).expansions[2].observations[2].status ~= "observed")
end)

test("global receiver denial and throwing publication lookup", function()
    local calls = 0
    local env = setup(function() calls = calls + 1 end)
    env.canaccessvalue = function(v) return not rawequal(v, env) and not rawequal(v, secret) end
    capture(env); assert(calls == 0)
    env = setup(function() calls = calls + 1 end)
    env.LE_EXPANSION_CLASSIC = nil
    setmetatable(env, { __index = function(_, k)
        if k == names[1] then error(secret) end
        return _G[k]
    end })
    capture(env); assert(calls == 2)
end)

test("missing restricted or throwing API lookup preserves independent attempts", function()
    for _, bad in ipairs({ false, 7, secret }) do
        local env = setup(bad)
        assert(capture(env).expansions[1].observations[1].status ~= "observed")
    end
    local env = setup(nil)
    setmetatable(env, { __index = function(_, k)
        if k == "GetExpansionDisplayInfo" then error(secret) end
        return _G[k]
    end })
    assert(capture(env).expansions[2].observations[2].status == "field-error")
end)

test("API lookup and both function guards revoke original expansion", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        for index = 1, 2 do
            local calls, revoked = 0, false
            local value = index == 1 and 17.25 or -3.5
            local fn = function(v) assert(v ~= value); calls = calls + 1 end
            local env = setup(fn)
            if phase == "lookup" then
                env.GetExpansionDisplayInfo = nil
                setmetatable(env, { __index = function(_, k)
                    if k == "GetExpansionDisplayInfo" then revoked = true; return fn end
                    return _G[k]
                end })
            end
            env.issecretvalue = function(v)
                if phase == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            env.canaccessvalue = function(v)
                if phase == "access" and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and rawequal(v, value))
            end
            assert(capture(env).expansions[index].observations[1].status ~= "observed")
            assert(calls == 2)
        end
    end
end)

test("field lookups recheck receiver after each preceding field", function()
    for blocked = 1, 3 do
        local object, revoked, reads = {}, false, 0
        setmetatable(object, { __index = function(_, key)
            reads = reads + 1; assert(not revoked and key == fields[reads])
            if reads == blocked then revoked = true end
            return 123
        end })
        local env = setup(function() return object end)
        env.canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
        local f = capture(env).expansions[1].observations[1].values[1].fields
        assert(reads == blocked and f[fields[blocked]].value == 123)
        for i = blocked + 1, 3 do assert(f[fields[i]].status == "field-error") end
    end
end)

test("tuple serialization revocation and restricted fields stay opaque", function()
    local object, revoked, reads = {}, false, 0
    setmetatable(object, { __index = function() reads = reads + 1; error("revoked lookup") end })
    local env = setup(function() return object, "revoke" end)
    env.canaccessvalue = function(v)
        if v == "revoke" then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, object))
    end
    capture(env); assert(reads == 0)
    env = setup(function() return { glueAmbianceSoundKit = secret, glueMusicSoundKit = 0/0 } end)
    local f = capture(env).expansions[1].observations[1].values[1].fields
    assert(f.glueAmbianceSoundKit.status == "restricted")
    assert(f.glueCreditsSoundKit.kind == "nil" and f.glueMusicSoundKit.status == "nonfinite")
end)

test("zero nil errors fresh API lookup and first object only", function()
    local calls, extraReads = 0, 0
    local extra = setmetatable({}, { __index = function() extraReads = extraReads + 1; error("extra object") end })
    local env = setup(function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then return nil end
        if calls == 3 then error(secret) end
        return nil, extra, false, nil
    end)
    local r = capture(env)
    assert(r.expansions[1].observations[1].n == 0 and r.expansions[1].observations[2].n == 1)
    assert(r.expansions[2].observations[1].status == "call-error")
    assert(r.expansions[2].observations[2].n == 4 and extraReads == 0)
    env = setup(nil)
    env.GetExpansionDisplayInfo = function()
        env.GetExpansionDisplayInfo = function() return "replacement" end
        error(secret)
    end
    r = capture(env)
    assert(r.expansions[1].observations[2].values[1].value == "replacement")
end)

test("tuple field string label snapshot bounds and no playback", function()
    local calls, values = 0, {}
    for i = 1, 20 do values[i] = string.rep("x", 300) end
    values[1] = { glueMusicSoundKit = string.rep("s", 300) }
    local env = setup(function() calls = calls + 1; return unpack(values) end)
    local q = capture(env, string.rep("l", 200)).expansions[1].observations[1]
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[2].value == 256)
    assert(#q.values[1].fields.glueMusicSoundKit.value == 256)
    assert(#env.ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture(env) end
    assert(calls == 40 and #env.ApiContractProbeDB.captures == 10 and env.ApiContractProbeDB.dropped == 1)
end)

test("missing guards fail closed and all excludes producer", function()
    local calls = 0
    local env = setup(function() calls = calls + 1 end)
    env.SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and env.sideEffects == 0)
    env.canaccessvalue = nil
    env.SlashCmdList.APICONTRACTPROBE("expansion-audio-fields missing")
    assert(env.ApiContractProbeDB.captures[2].status == "missing-access-api" and calls == 0)
end)

test("objects are collectible and only declared userdata fields are read", function()
    local weak = setmetatable({}, { __mode = "v" })
    local env = setup(function()
        local object = newproxy(true)
        getmetatable(object).__index = function(_, key)
            for _, field in ipairs(fields) do if key == field then return 9 end end
            error("undeclared field " .. key)
        end
        weak[#weak + 1] = object
        return object
    end)
    assert(capture(env).expansions[1].observations[1].values[1].fields.glueMusicSoundKit.value == 9)
    collectgarbage("collect"); collectgarbage("collect")
    assert(next(weak) == nil and env.sideEffects == 0)
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
