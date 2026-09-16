local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("restricted lookup") end
getmetatable(secret).__tostring = function() error("restricted stringify") end
local function setup(fn)
    local env = setmetatable({}, { __index = _G })
    env._G, env.SlashCmdList, env.sideEffects = env, {}, 0
    env.issecretvalue = function(v) return rawequal(v, secret) end
    env.canaccessvalue = function(v) return not rawequal(v, secret) end
    env.GetBuildInfo, env.time = function() return "fixture" end, function() return 42 end
    env.print = function() end
    local function forbidden() env.sideEffects = env.sideEffects + 1; error("excluded mutation") end
    env.C_ItemInteraction = setmetatable({ GetItemInteractionInfo = fn }, { __index = function() return forbidden end })
    env.C_AddOns, env.LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then setfenv(assert(loadfile(root .. "/" .. line)), env)() end
    end
    toc:close()
    return env
end
local function capture(env, label)
    env.SlashCmdList.APICONTRACTPROBE("item-interaction-flags " .. (label or "fixture"))
    assert(env.ApiContractProbeDB and #env.ApiContractProbeDB.captures > 0, "item-interaction-flags mode absent")
    assert(env.sideEffects == 0, "excluded call")
    return assert(env.ApiContractProbeDB.captures[#env.ApiContractProbeDB.captures].itemInteractionFlags)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("two fresh zero-argument calls inspect only current flags", function()
    local calls, reads = 0, 0
    local env = setup(function(...)
        assert(select("#", ...) == 0); calls = calls + 1
        local n = calls
        return setmetatable({}, { __index = function(_, key)
            assert(key == "flags"); reads = reads + 1; return n == 1 and 12.5 or -7
        end }), nil, "tail"
    end)
    local r = capture(env)
    assert(calls == 2 and reads == 2 and #r == 2)
    assert(r[1].n == 3 and r[1].values[2].kind == "nil" and r[1].values[3].value == "tail")
    assert(r[1].values[1].fields.flags.value == 12.5 and r[2].values[1].fields.flags.value == -7)
    local count = 0; for _ in pairs(r[1].values[1].fields) do count = count + 1 end; assert(count == 1)
end)

test("zero results nil root and wrong first return do not fabricate flags", function()
    local calls = 0
    local env = setup(function() calls = calls + 1; if calls == 1 then return end; return nil, false, nil end)
    local r = capture(env)
    assert(r[1].n == 0 and r[2].n == 3 and r[2].values[1].kind == "nil" and not r[2].values[1].fields)
    env = setup(function() return 99, setmetatable({}, { __index = function() error("second object inspected") end }) end)
    r = capture(env)
    assert(r[1].values[1].value == 99 and r[1].values[2].kind == "table" and not r[1].values[2].fields)
end)

test("namespace and function failures remain independent", function()
    for _, phase in ipairs({ "missing", "secret", "throw", "wrong" }) do
        local calls = 0
        local env = setup(function() calls = calls + 1; if calls == 1 then error(secret) end; return { flags = false } end)
        if phase == "missing" then env.C_ItemInteraction = nil
        elseif phase == "secret" then env.C_ItemInteraction = secret
        elseif phase == "wrong" then env.C_ItemInteraction = false
        else env.C_ItemInteraction = setmetatable({}, { __index = function() error(secret) end }) end
        local r = capture(env); assert(calls == 0 and r[1].status == "field-error" and r[2].status == "field-error")
    end
    local calls = 0
    local env = setup(function() calls = calls + 1; if calls == 1 then error(secret) end; return { flags = false } end)
    local r = capture(env); assert(calls == 2 and r[1].status == "call-error" and r[2].values[1].fields.flags.value == false)
    for _, value in ipairs({ false, secret, 7 }) do
        env = setup(value); r = capture(env); assert(r[1].status == "missing-api" and r[2].status == "missing-api")
    end
end)

test("namespace and function guards prevent lookup and invocation", function()
    for _, phase in ipairs({ "secret", "access" }) do
        for _, which in ipairs({ "namespace", "function" }) do
            local calls = 0
            local env = setup(function() calls = calls + 1 end)
            local blocked = which == "namespace" and env.C_ItemInteraction or env.C_ItemInteraction.GetItemInteractionInfo
            if phase == "secret" then env.issecretvalue = function(v) return rawequal(v, blocked) end
            else env.canaccessvalue = function(v) return not rawequal(v, blocked) end end
            local r = capture(env); assert(calls == 0)
            assert(r[1].status == (which == "namespace" and "field-error" or "missing-api"))
        end
    end
end)

test("fresh lookup observes namespace and function replacement", function()
    local env, calls = nil, 0
    env = setup(function()
        calls = calls + 1
        env.C_ItemInteraction = { GetItemInteractionInfo = function() calls = calls + 1; return { flags = "replacement" } end }
        return { flags = "first" }
    end)
    local r = capture(env)
    assert(calls == 2 and r[1].values[1].fields.flags.value == "first" and r[2].values[1].fields.flags.value == "replacement")
end)

test("root access rechecked after tuple and object observations", function()
    for _, revokeAt in ipairs({ 1, 2, 3 }) do
        local object = setmetatable({}, { __index = function() error("revoked receiver lookup") end })
        local env = setup(function() return object end)
        local seen = 0
        env.canaccessvalue = function(v)
            if rawequal(v, object) then seen = seen + 1; return seen < revokeAt end
            return true
        end
        local r = capture(env)
        assert(r[1].values[1].status == "restricted" or r[1].values[1].fields.flags.status == "field-error")
    end
    local object, denied = {}, false
    setmetatable(object, { __index = function() error("tuple-revoked root read") end })
    local trigger = {}
    local env = setup(function() return object, trigger end)
    env.canaccessvalue = function(v)
        if rawequal(v, trigger) then denied = true end
        return not (denied and rawequal(v, object))
    end
    assert(capture(env)[1].values[1].status == "restricted")
end)

test("flags guard precedes inspection including lookup revocation", function()
    for _, userdata in ipairs({ false, true }) do
        local value, revoked = {}, false
        local object = userdata and newproxy(true) or setmetatable({}, {})
        getmetatable(object).__index = function(_, key) assert(key == "flags"); revoked = true; return value end
        local env = setup(function() return object end)
        env.canaccessvalue = function(v) return not (revoked and rawequal(v, value)) end
        env.type = function(v) assert(not (revoked and rawequal(v, value)), "revoked flags inspected"); return type(v) end
        local r = capture(env); assert(r[1].values[1].fields.flags.status == "restricted")
    end
    local env = setup(function() return { flags = secret } end)
    env.type = function(v) assert(not rawequal(v, secret)); return type(v) end
    assert(capture(env)[2].values[1].fields.flags.status == "restricted")
end)

test("missing throwing and opaque flags remain raw observations", function()
    local reads = 0
    local env = setup(function() return setmetatable({}, { __index = function(_, key)
        assert(key == "flags"); reads = reads + 1; if reads == 1 then error(secret) end
    end }) end)
    local r = capture(env)
    assert(r[1].values[1].fields.flags.status == "field-error" and r[2].values[1].fields.flags.kind == "nil")
    local opaque = setmetatable({}, { __index = function() error("flag traversal") end })
    env = setup(function() return { flags = opaque } end)
    r = capture(env); assert(r[1].values[1].fields.flags.kind == "table" and not r[1].values[1].fields.flags.fields)
end)

test("tuple strings labels and snapshots are bounded", function()
    local calls = 0
    local env = setup(function()
        calls = calls + 1
        return { flags = string.rep("x", 300) }, nil, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, secret
    end)
    local r = capture(env, string.rep("l", 200))
    assert(r[1].n == 17 and r[1].truncated and #r[1].values == 16)
    assert(#r[1].values[1].fields.flags.value == 256 and r[1].values[1].fields.flags.truncated)
    assert(#env.ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do env.SlashCmdList.APICONTRACTPROBE("item-interaction-flags limit") end
    assert(calls == 20 and #env.ApiContractProbeDB.captures == 10 and env.ApiContractProbeDB.dropped == 1)
end)

test("all excludes mode and no interaction mutations or object retention", function()
    local calls, weak = 0, setmetatable({}, { __mode = "v" })
    local env = setup(function()
        calls = calls + 1
        local object = newproxy(true)
        getmetatable(object).__index = function(_, key) assert(key == "flags"); return 19 end
        weak[calls] = object; return object
    end)
    env.SlashCmdList.APICONTRACTPROBE("all excluded")
    assert(calls == 0 and env.ApiContractProbeDB.captures[1].itemInteractionFlags == nil)
    capture(env)
    collectgarbage("collect"); collectgarbage("collect")
    assert(calls == 2 and weak[1] == nil and weak[2] == nil and env.sideEffects == 0)
end)

test("missing access APIs fail closed", function()
    for _, key in ipairs({ "issecretvalue", "canaccessvalue" }) do
        local calls = 0
        local env = setup(function() calls = calls + 1 end); env[key] = false
        env.SlashCmdList.APICONTRACTPROBE("item-interaction-flags guarded")
        assert(env.ApiContractProbeDB and env.ApiContractProbeDB.captures[1], "item-interaction-flags mode absent")
        assert(env.ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    end
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
