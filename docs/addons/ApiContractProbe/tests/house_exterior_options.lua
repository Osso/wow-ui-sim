local root = assert(arg[1])
local names = { "GetCurrentHouseExteriorType", "GetHouseExteriorSizeOptions", "GetHouseExteriorTypeOptions" }
local sizeFields = { "size", "name", "isLocked" }
local typeFields = { "houseExteriorTypeID", "name", "isLocked", "isInvalid", "reasonString" }
local passed, failed = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function setup(callback)
    local env = setmetatable({}, { __index = _G })
    env._G, env.SlashCmdList, env.sideEffects = env, {}, 0
    env.issecretvalue = function(v) return rawequal(v, secret) end
    env.canaccessvalue = function(v) return not rawequal(v, secret) end
    env.GetBuildInfo, env.time = function() return "fixture" end, function() return 42 end
    env.print = function() end
    local function forbidden() env.sideEffects = env.sideEffects + 1; error("excluded call") end
    env.C_HouseExterior = setmetatable({}, { __index = function() return forbidden end })
    for _, name in ipairs(names) do
        env.C_HouseExterior[name] = function(...) return callback(name, ...) end
    end
    env.C_AddOns, env.LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then setfenv(assert(loadfile(root .. "/" .. line)), env)() end
    end
    toc:close()
    return env
end
local function capture(env, label)
    env.SlashCmdList.APICONTRACTPROBE("house-exterior-options " .. (label or "fixture"))
    assert(env.ApiContractProbeDB and #env.ApiContractProbeDB.captures > 0, "house-exterior-options mode absent")
    assert(env.sideEffects == 0, "excluded operation executed")
    return assert(env.ApiContractProbeDB.captures[#env.ApiContractProbeDB.captures].houseExteriorOptions)
end
local function optionFixture(selected)
    return { [selected] = 17.25, options = { { size = -2.5, name = "option", isLocked = false,
        houseExteriorTypeID = 31.5, isInvalid = true, reasonString = "current reason",
        lockReasonString = secret } } }
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("three independent zero-argument calls and exact declared fields", function()
    local calls = {}
    local env = setup(function(name, ...)
        assert(select("#", ...) == 0); calls[#calls + 1] = name
        if name == names[1] then return 12.5, "current" end
        return optionFixture(name == names[2] and "selectedSize" or "selectedExteriorType")
    end)
    local r = capture(env)
    assert(#calls == 3)
    for i, name in ipairs(names) do assert(calls[i] == name) end
    assert(r[names[1]].n == 2 and r[names[1]].values[2].value == "current")
    for i = 2, 3 do
        local object = r[names[i]].values[1]
        local selected = i == 2 and "selectedSize" or "selectedExteriorType"
        assert(object.fields[selected].value == 17.25)
        assert(object.fields.options.kind == "table" and object.fields.options.entries == nil)
        local fields = object.optionEntries.entries[1].fields
        local expected = i == 2 and sizeFields or typeFields
        local count = 0; for _ in pairs(fields) do count = count + 1 end
        assert(count == #expected and fields.lockReasonString == nil)
        assert(fields.isLocked.value == false and fields.name.value == "option")
        if i == 3 then assert(fields.reasonString.value == "current reason" and fields.isInvalid.value == true) end
    end
end)

test("zero returns nil holes and errors preserve independent peers", function()
    local calls = 0
    local env = setup(function(name)
        calls = calls + 1
        if name == names[1] then error(secret) end
        if name == names[2] then return end
        return nil, false, nil
    end)
    local r = capture(env)
    assert(calls == 3 and r[names[1]].status == "call-error")
    assert(r[names[2]].n == 0 and r[names[3]].n == 3)
    assert(r[names[3]].values[1].kind == "nil" and r[names[3]].values[3].kind == "nil")
end)

test("missing restricted and throwing functions do not suppress peers", function()
    for _, bad in ipairs({ false, 7, secret }) do
        for i = 1, 3 do
            local calls = 0
            local env = setup(function() calls = calls + 1 end)
            env.C_HouseExterior[names[i]] = bad
            assert(capture(env)[names[i]].status ~= "observed" and calls == 2)
        end
    end
    local calls = 0
    local env = setup(function() calls = calls + 1 end)
    env.C_HouseExterior[names[2]] = nil
    setmetatable(env.C_HouseExterior, { __index = function() error(secret) end })
    assert(capture(env)[names[2]].status == "field-error" and calls == 2)
end)

test("namespace and function accessibility deny calls", function()
    for _, phase in ipairs({ "secret", "access" }) do
        for i = 0, 3 do
            local calls = 0
            local env = setup(function() calls = calls + 1 end)
            local blocked = i == 0 and env.C_HouseExterior or env.C_HouseExterior[names[i]]
            env.issecretvalue = function(v) return rawequal(v, secret) or (phase == "secret" and rawequal(v, blocked)) end
            env.canaccessvalue = function(v) return not rawequal(v, secret) and not (phase == "access" and rawequal(v, blocked)) end
            capture(env); assert(calls == (i == 0 and 0 or 2))
        end
    end
end)

test("every root field rechecks receiver after preceding lookups", function()
    for _, selected in ipairs({ "selectedSize", "selectedExteriorType" }) do
        local object, revoked, reads = {}, false, 0
        setmetatable(object, { __index = function(_, key)
            assert(not revoked and key == selected); reads = reads + 1; revoked = true; return 3
        end })
        local env = setup(function(name)
            if name == (selected == "selectedSize" and names[2] or names[3]) then return object end
        end)
        env.canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
        capture(env); assert(reads == 1)
    end
end)

test("each list index and option field rechecks receiver", function()
    for index = 1, 4 do
        local list, revoked, reads = {}, false, 0
        setmetatable(list, { __index = function(_, key)
            assert(not revoked and key == reads + 1); reads = reads + 1
            if key == index then revoked = true end
            return {}
        end })
        local env = setup(function(name) if name == names[2] then return { options = list } end end)
        env.canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
        capture(env); assert(reads == index)
    end
    for which, keys in ipairs({ sizeFields, typeFields }) do
        for stop = 1, #keys do
            local entry, revoked, reads = {}, false, 0
            setmetatable(entry, { __index = function(_, key)
                assert(not revoked and key == keys[reads + 1]); reads = reads + 1
                if reads == stop then revoked = true end
                return 9
            end })
            local env = setup(function(name) if name == names[which + 1] then return { options = { entry } } end end)
            env.canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, entry)) end
            capture(env); assert(reads == stop)
        end
    end
end)

test("serialization revocation blocks root and options traversal", function()
    for _, target in ipairs({ "root", "list" }) do
        local revoked, reads = false, 0
        local list = setmetatable({}, { __index = function() reads = reads + 1; error("revoked list") end })
        local object = { options = list }
        if target == "root" then setmetatable(object, { __index = function() reads = reads + 1; error("revoked root") end }) end
        local env = setup(function(name) if name == names[2] then return object, "revoke" end end)
        env.canaccessvalue = function(v)
            if rawequal(v, "revoke") then revoked = true end
            return not rawequal(v, secret) and not (revoked and rawequal(v, target == "root" and object or list))
        end
        capture(env); assert(reads == 0)
    end
end)

test("first object only no extra or historical fields and bounded entries", function()
    local reads = 0
    local extra = setmetatable({}, { __index = function() error("extra return inspected") end })
    local list = setmetatable({}, { __index = function(_, i)
        assert(i >= 1 and i <= 4); reads = reads + 1
        return setmetatable({}, { __index = function(_, key)
            assert(key ~= "lockReasonString" and key ~= "fixtureID"); return secret
        end })
    end })
    local env = setup(function(name)
        if name == names[1] then return extra end
        return { options = list }, extra
    end)
    local r = capture(env); assert(reads == 8)
    assert(r[names[1]].values[1].fields == nil)
    assert(r[names[3]].values[1].optionEntries.entries[1].fields.reasonString.status == "restricted")
end)

test("tuple string label and snapshot limits manual not all", function()
    local calls = 0
    local env = setup(function()
        calls = calls + 1
        return string.rep("x", 300), nil, nil, nil, nil, nil, nil, nil, nil, nil, nil, nil, nil, nil, nil, false, secret
    end)
    local r = capture(env, string.rep("L", 200))
    assert(r[names[1]].n == 17 and r[names[1]].truncated and #r[names[1]].values == 16)
    assert(#r[names[1]].values[1].value == 256 and #env.ApiContractProbeDB.captures[1].label == 128)
    for _ = 2, 11 do capture(env) end
    assert(calls == 30 and #env.ApiContractProbeDB.captures == 10 and env.ApiContractProbeDB.dropped == 1)
    local other = setup(function() error("all invoked house exterior") end)
    other.SlashCmdList.APICONTRACTPROBE("all")
    assert(other.ApiContractProbeDB.captures[1].houseExteriorOptions == nil)
end)

test("returned objects are not retained and userdata fields are guarded", function()
    local weak = setmetatable({}, { __mode = "v" })
    local env = setup(function(name)
        if name == names[1] then return 1, "house" end
        local entry = newproxy(true)
        getmetatable(entry).__index = function(_, key) if key == "name" then return "userdata option" end end
        local object = { options = { entry } }; weak[#weak + 1] = object
        return object
    end)
    assert(capture(env)[names[3]].values[1].optionEntries.entries[1].fields.name.value == "userdata option")
    collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
