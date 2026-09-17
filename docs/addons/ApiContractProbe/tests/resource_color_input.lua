local root = assert(arg[1])
local passed, failed = 0, 0
local calls, curve, colors, weak
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspected") end
local function pack(...) return { n = select("#", ...), ... } end
local function setup()
    calls, colors = {}, {}
    weak = setmetatable({}, { __mode = "v" })
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 1 end
    local function record(name, ...) calls[#calls + 1] = { name = name, args = pack(...) } end
    curve = { AddPoint = function(self, x, color)
        record("add", self, x, color)
        return nil, "added"
    end }
    C_CurveUtil = { CreateColorCurve = function(...)
        record("curve", ...); return curve
    end }
    CreateColor = function(...)
        record("color", ...)
        local color = setmetatable({}, { __index = function() error("color inspected") end })
        colors[#colors + 1] = color
        return color
    end
    UnitHealthPercent = function(...) record("health", ...); return secret, nil end
    UnitPowerPercent = function(...) record("power", ...); return 7, nil, "power" end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("resource-color-input " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "resource-color-input mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].resourceColorInput)
end
local function count(name)
    local n = 0
    for _, call in ipairs(calls) do if call.name == name then n = n + 1 end end
    return n
end
local function test(name, fn)
    setup()
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("original curve and colors with exact seven-call sequence", function()
    local r = capture()
    assert(#calls == 7 and r.status == "observed")
    assert(calls[1].name == "curve" and calls[1].args.n == 0)
    for i, rgba in ipairs({ { 1, 0, 0, 1 }, { 0, 0, 1, 1 } }) do
        assert(calls[i + 1].name == "color" and calls[i + 1].args.n == 4)
        for j = 1, 4 do assert(calls[i + 1].args[j] == rgba[j]) end
        local a = calls[i + 3].args
        assert(calls[i + 3].name == "add" and a.n == 3)
        assert(rawequal(a[1], curve) and a[2] == (i - 1) * 100 and rawequal(a[3], colors[i]))
        assert(r.setup.points[i].n == 2 and r.setup.points[i].values[1].kind == "nil")
    end
    local h, p = calls[6].args, calls[7].args
    assert(h.n == 3 and h[1] == "player" and h[2] == false and rawequal(h[3], curve))
    assert(p.n == 4 and p[1] == "player" and p[2] == nil and p[3] == false and rawequal(p[4], curve))
    assert(r.health.n == 2 and r.health.values[1].status == "restricted" and r.health.values[2].kind == "nil")
    assert(r.power.n == 3 and r.power.values[2].kind == "nil")
end)
test("missing throwing invalid and restricted setup never forwards resources", function()
    for _, stage in ipairs({ "curve", "color", "add" }) do
        for _, failure in ipairs({ "missing", "throw", "nil", "restricted" }) do
            setup()
            local fn
            if failure == "throw" then fn = function() error(secret) end
            elseif failure == "nil" then fn = function() return nil end
            elseif failure == "restricted" then fn = secret end
            if stage == "curve" then C_CurveUtil.CreateColorCurve = fn
            elseif stage == "color" then CreateColor = fn
            else curve.AddPoint = fn end
            local r = capture()
            -- An AddPoint returning nil is a successful call, not a rejected setup.
            if stage == "add" and failure == "nil" then assert(r.status == "observed")
            else assert(r.status == "unavailable-input" and count("health") == 0 and count("power") == 0) end
        end
    end
end)
test("second color or point failure gates both resources", function()
    for _, stage in ipairs({ "color", "add" }) do
        setup()
        local n = 0
        local original = stage == "color" and CreateColor or curve.AddPoint
        local fn = function(...)
            n = n + 1
            if n == 2 then error(secret) end
            return original(...)
        end
        if stage == "color" then CreateColor = fn else curve.AddPoint = fn end
        local r = capture()
        assert(r.status == "unavailable-input" and count("health") == 0 and count("power") == 0)
    end
end)
test("AddPoint lookup and function guards revoke receiver or color", function()
    for _, phase in ipairs({ "lookup", "issecretvalue", "canaccessvalue" }) do
        for _, target in ipairs({ "curve", "color" }) do
            setup()
            local original, revoked = curve.AddPoint, false
            if phase == "lookup" then
                curve.AddPoint = nil
                setmetatable(curve, { __index = function(_, key)
                    if key == "AddPoint" then revoked = true; return original end
                end })
            end
            for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
                _G[guard] = function(v)
                    if phase == guard and rawequal(v, original) then revoked = true end
                    local denied = rawequal(v, secret) or (revoked and rawequal(v, target == "curve" and curve or colors[1]))
                    if guard == "issecretvalue" then return denied end
                    return not denied
                end
            end
            local r = capture()
            assert(r.status == "unavailable-input" and count("add") == 0 and count("health") == 0)
        end
    end
end)
test("color constructor function guard revokes curve", function()
    local fn, revoked = CreateColor, false
    canaccessvalue = function(v)
        if rawequal(v, fn) then revoked = true end
        return not rawequal(v, secret) and not (revoked and rawequal(v, curve))
    end
    assert(capture().status == "unavailable-input")
    assert(count("color") == 0 and count("health") == 0)
end)
test("resource global lookup and function guards revoke curve or unit", function()
    for _, api in ipairs({ "UnitHealthPercent", "UnitPowerPercent" }) do
        for _, phase in ipairs({ "lookup", "issecretvalue", "canaccessvalue" }) do
            for _, which in ipairs({ "curve", "unit" }) do
                setup()
                local fn, revoked, old = _G[api], false, getmetatable(_G)
                if phase == "lookup" then
                    _G[api] = nil
                    setmetatable(_G, { __index = function(_, key)
                        if key == api then revoked = true; return fn end
                    end })
                end
                for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
                    _G[guard] = function(v)
                        if phase == guard and rawequal(v, fn) then revoked = true end
                        local denied = rawequal(v, secret) or (revoked and rawequal(v, which == "curve" and curve or "player"))
                        if guard == "issecretvalue" then return denied end
                        return not denied
                    end
                end
                local ok, result = pcall(capture)
                setmetatable(_G, old)
                assert(ok, result)
                assert(count(api == "UnitHealthPercent" and "health" or "power") == 0)
            end
        end
    end
end)
test("resource failures and nil arity are independent", function()
    UnitHealthPercent = function() error(secret) end
    UnitPowerPercent = function() return end
    local r = capture()
    assert(r.health.status == "call-error" and r.power.status == "observed" and r.power.n == 0)
    setup(); UnitHealthPercent = nil
    r = capture()
    assert(r.health.status == "missing-api" and count("power") == 1)
end)
test("returned objects stay opaque and input objects are not retained", function()
    local refs = weak
    local original = C_CurveUtil.CreateColorCurve
    C_CurveUtil.CreateColorCurve = function() local o = original(); refs[1] = o; return o end
    local create = CreateColor
    CreateColor = function(...) local o = create(...); refs[#colors + 1] = o; return o end
    UnitHealthPercent = function()
        local o = newproxy(true)
        getmetatable(o).__index = function() error("result method lookup") end
        getmetatable(o).__tostring = function() error("result tostring") end
        refs[4] = o; return o
    end
    capture()
    curve, colors, calls = nil, nil, nil
    C_CurveUtil, CreateColor, UnitHealthPercent, UnitPowerPercent = nil, nil, nil, nil
    collectgarbage("collect"); collectgarbage("collect")
    for i = 1, 4 do assert(refs[i] == nil) end
end)
test("tuple strings labels and ten-snapshot seven-call limit", function()
    UnitPowerPercent = function(...)
        calls[#calls + 1] = { name = "power", args = pack(...) }
        return string.rep("x", 300), nil, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, secret
    end
    local r = capture(string.rep("l", 200))
    assert(r.power.n == 17 and r.power.truncated and #r.power.values == 16)
    assert(#r.power.values[1].value == 256 and #ApiContractProbeDB.captures[1].label == 128)
    for _ = 1, 10 do capture() end
    assert(#calls == 70 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)
test("missing guards and all never run resource color setup", function()
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("resource-color-input")
    assert(ApiContractProbeDB and ApiContractProbeDB.captures[1].status == "missing-access-api")
    assert(#calls == 0)
    setup(); SlashCmdList.APICONTRACTPROBE("all")
    assert(not ApiContractProbeDB.captures[1].resourceColorInput and #calls == 0)
end)
test("partially constructed objects are released after setup failure", function()
    for _, failAt in ipairs({ 2, 5 }) do
        setup()
        local refs, steps = weak, 0
        C_CurveUtil.CreateColorCurve = function()
            steps = steps + 1
            local object = { AddPoint = function()
                steps = steps + 1
                if steps == failAt then error(secret) end
            end }
            refs[1] = object
            return object
        end
        CreateColor = function()
            steps = steps + 1
            if steps == failAt then error(secret) end
            local object = {}; refs[steps] = object; return object
        end
        local r = capture()
        assert(r.status == "unavailable-input" and count("health") == 0 and count("power") == 0)
        collectgarbage("collect"); collectgarbage("collect")
        for i = 1, 3 do assert(refs[i] == nil) end
    end
end)
test("setup output serialization revocation blocks resources", function()
    local denied = false
    curve.AddPoint = function()
        return "revoke-color"
    end
    canaccessvalue = function(v)
        if v == "revoke-color" then denied = true end
        return not rawequal(v, secret) and not (denied and rawequal(v, colors[1]))
    end
    assert(capture().status == "unavailable-input")
    assert(count("health") == 0 and count("power") == 0)
end)
test("old resource and color modes preserve their observations across new captures", function()
    Enum = { LuaCurveType = { Linear = 1 } }
    C_CurveUtil.CreateCurve = function()
        return { AddPoint = function() end, SetType = function() end }
    end
    local function old(mode)
        calls = {}
        SlashCmdList.APICONTRACTPROBE(mode .. " interleave")
        local r = ApiContractProbeDB.captures[#ApiContractProbeDB.captures]
        assert(not r.resourceColorInput)
        if mode == "resources" then
            assert(r.resources.curve.status == "constructed")
            assert(count("health") == 20 and count("power") == 35)
            assert(r.resources.units.player.power.modified.curved.n == 3)
            assert(r.resources.units.player.health.curved.values[1].status == "restricted")
        else
            assert(r.colorCurves.status == "observed" and count("curve") == 1)
            assert(count("color") == 4 and count("add") == 4)
            assert(r.colorCurves.inputs[3].x == -16 and r.colorCurves.inputs[4].x == 48)
        end
    end
    old("resources"); old("color-curves")
    calls = {}; assert(capture().status == "observed" and #calls == 7)
    old("resources"); old("color-curves")
end)
print(string.format("%d passed, %d failed resource-color-input fixtures", passed, failed))
if failed > 0 then os.exit(1) end
