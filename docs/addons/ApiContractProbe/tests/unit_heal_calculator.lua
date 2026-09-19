local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local baseMeta, originalType = getmetatable(_G), type
local names = { "GetHealAbsorbMode", "GetHealAbsorbClampMode", "GetDamageAbsorbClampMode",
    "GetHealAbsorbs", "GetDamageAbsorbs" }
local function pack(...) return { n = select("#", ...), ... } end
local function opaque()
    return setmetatable({}, { __index = function() error("opaque lookup") end,
        __tostring = function() error("opaque stringify") end })
end
local function setup()
    setmetatable(_G, baseMeta)
    type = originalType
    ApiContractProbeDB, SlashCmdList, Enum = nil, {}, {}
    issecretvalue, canaccessvalue = function() return false end, function() return true end
    GetBuildInfo, time = function() return "fixture", "123" end, function() return 42 end
    local ctx = { objects = {}, methods = {}, state = {}, trace = {}, constructors = 0, populations = 0 }
    CreateUnitHealPredictionCalculator = function(...)
        assert(select("#", ...) == 0, "constructor has no arguments")
        ctx.constructors = ctx.constructors + 1
        local index = ctx.constructors
        local object, methods = newproxy(true), {}
        ctx.objects[index], ctx.methods[index], ctx.state[index] = object, methods, 0
        getmetatable(object).__index = methods
        getmetatable(object).__eq = function() error("calculator compared") end
        getmetatable(object).__tostring = function() error("calculator stringified") end
        ctx.trace[#ctx.trace + 1] = { kind = "construct", index = index }
        for position, name in ipairs(names) do
            methods[name] = function(self, ...)
                assert(rawequal(self, ctx.objects[index]), "original getter receiver")
                assert(select("#", ...) == 0, "getter has no arguments")
                ctx.trace[#ctx.trace + 1] = { kind = name, index = index }
                return index * 100 + ctx.state[index] + position, nil, false
            end
        end
        return object, nil
    end
    UnitGetDetailedHealPrediction = function(...)
        local args = pack(...)
        assert(args.n == 3 and args[2] == nil, "explicit nil healer and three arguments")
        local index = args[1] == "player" and 1 or 2
        assert(args[1] == "player" or args[1] == "target", "fixed unit")
        assert(rawequal(args[3], ctx.objects[index]), "original populated calculator")
        ctx.populations = ctx.populations + 1
        ctx.trace[#ctx.trace + 1] = { kind = "populate", index = index, args = args }
        ctx.state[index] = 50
    end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
    return ctx
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("unit-heal-calculator " .. (label or "sample"))
    local captures = ApiContractProbeDB and ApiContractProbeDB.captures
    assert(captures and #captures > 0, "unit-heal-calculator manual mode absent")
    return assert(captures[#captures].unitHealCalculator, "unit-heal-calculator output absent")
end
local function test(name, fn)
    local ok, err = pcall(fn)
    setmetatable(_G, baseMeta)
    type = originalType
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function installObjectHook(ctx, hook)
    local factory = CreateUnitHealPredictionCalculator
    CreateUnitHealPredictionCalculator = function(...)
        local object, tail = factory(...)
        local index = ctx.constructors
        hook(object, ctx.methods[index], index)
        return object, tail
    end
end

test("two fresh originals and exact before populate after order", function()
    local ctx = setup()
    local row = capture()
    assert(#row.units == 2 and ctx.constructors == 2 and ctx.populations == 2)
    assert(not rawequal(ctx.objects[1], ctx.objects[2]) and #ctx.trace == 24)
    local offset = 0
    for index, unit in ipairs({ "player", "target" }) do
        local lane = row.units[index]
        assert(lane.unit.value == unit and lane.constructor.n == 2)
        assert(lane.constructor.values[1].kind == "userdata" and lane.constructor.values[2].kind == "nil")
        assert(ctx.trace[offset + 1].kind == "construct")
        for position, name in ipairs(names) do
            assert(ctx.trace[offset + 1 + position].kind == name)
            assert(ctx.trace[offset + 7 + position].kind == name)
            local before, after = lane.before[name], lane.after[name]
            assert(before.n == 3 and after.n == 3)
            assert(before.values[1].value == index * 100 + position)
            assert(after.values[1].value == index * 100 + 50 + position)
            assert(before.values[2].kind == "nil" and after.values[3].value == false)
        end
        assert(ctx.trace[offset + 7].kind == "populate")
        assert(lane.populate.status == "observed" and lane.populate.n == 0)
        offset = offset + 12
    end
end)

test("population throws after state change but after getters and peer still run", function()
    local ctx = setup()
    local populate = UnitGetDetailedHealPrediction
    UnitGetDetailedHealPrediction = function(...)
        populate(...)
        error(opaque())
    end
    local row = capture()
    assert(ctx.populations == 2 and #ctx.trace == 24)
    for index, lane in ipairs(row.units) do
        assert(lane.populate.status == "call-error" and lane.populate.error == nil)
        for position, name in ipairs(names) do
            assert(lane.after[name].values[1].value == index * 100 + 50 + position)
        end
    end
end)

test("missing or restricted population never suppresses ten getter reads per lane", function()
    for _, scenario in ipairs({ "missing", "secret", "access", "lookup" }) do
        local ctx = setup()
        local fn = UnitGetDetailedHealPrediction
        if scenario == "missing" then UnitGetDetailedHealPrediction = nil
        elseif scenario == "secret" then issecretvalue = function(v) return rawequal(v, fn) end
        elseif scenario == "access" then canaccessvalue = function(v) return not rawequal(v, fn) end
        else
            UnitGetDetailedHealPrediction = nil
            setmetatable(_G, { __index = function(_, key)
                if key == "UnitGetDetailedHealPrediction" then error(opaque()) end
            end })
        end
        local row = capture()
        assert(ctx.populations == 0 and #ctx.trace == 22)
        for _, lane in ipairs(row.units) do
            assert(lane.populate.status ~= "observed")
            for _, name in ipairs(names) do assert(lane.after[name].status == "observed") end
        end
    end
end)

test("constructor errors invalid first results and inaccessible objects stay isolated", function()
    for _, scenario in ipairs({ "error", "zero", "nil", "boolean", "number", "string", "restricted" }) do
        local ctx = setup()
        local factory, attempts = CreateUnitHealPredictionCalculator, 0
        local secret = opaque()
        CreateUnitHealPredictionCalculator = function(...)
            attempts = attempts + 1
            if attempts == 1 then
                if scenario == "error" then error(secret)
                elseif scenario == "zero" then return
                elseif scenario == "nil" then return nil, secret
                elseif scenario == "boolean" then return false
                elseif scenario == "number" then return 7
                elseif scenario == "string" then return "calculator"
                else return secret end
            end
            -- Keep the surviving target calculator at index 2 in the modeled backend.
            ctx.constructors = 1
            return factory(...)
        end
        canaccessvalue = function(v) return not rawequal(v, secret) end
        local row = capture()
        assert(attempts == 2 and row.units[1].before == nil)
        assert(row.units[2].populate.status == "observed" and ctx.populations == 1)
        assert(row.units[2].after.GetDamageAbsorbs.status == "observed")
    end
end)

test("constructor globals and function guards fail closed without type inspection", function()
    for _, phase in ipairs({ "missing", "root-secret", "root-access", "function-secret", "function-access", "lookup" }) do
        local ctx = setup()
        local fn = CreateUnitHealPredictionCalculator
        if phase == "missing" then CreateUnitHealPredictionCalculator = nil
        elseif phase == "root-secret" then issecretvalue = function(v) return rawequal(v, _G) end
        elseif phase == "root-access" then canaccessvalue = function(v) return not rawequal(v, _G) end
        elseif phase == "function-secret" then issecretvalue = function(v) return rawequal(v, fn) end
        elseif phase == "function-access" then canaccessvalue = function(v) return not rawequal(v, fn) end
        else
            CreateUnitHealPredictionCalculator = nil
            setmetatable(_G, { __index = function(_, key)
                if key == "CreateUnitHealPredictionCalculator" then error(opaque()) end
            end })
        end
        local row = capture()
        assert(ctx.constructors == 0 and ctx.populations == 0)
        for _, lane in ipairs(row.units) do assert(lane.constructor.status ~= "observed") end
    end
end)

test("getter lookup failures and zero nil secret tuples do not suppress peers", function()
    local ctx = setup()
    local secret, opaqueResult = opaque(), opaque()
    installObjectHook(ctx, function(object, methods)
        methods[names[1]] = function() return end
        methods[names[2]] = function() return nil, false, nil end
        methods[names[3]] = function() return secret, opaqueResult, nil end
        methods[names[4]] = function() error(secret) end
        getmetatable(object).__index = function(_, key)
            if key == names[5] then error(secret) end
            return methods[key]
        end
    end)
    issecretvalue = function(v) return rawequal(v, secret) end
    local row = capture()
    assert(ctx.populations == 2)
    for _, lane in ipairs(row.units) do
        for _, stage in ipairs({ lane.before, lane.after }) do
            assert(stage[names[1]].n == 0 and stage[names[2]].n == 3)
            assert(stage[names[2]].values[1].kind == "nil" and stage[names[2]].values[3].kind == "nil")
            assert(stage[names[3]].values[1].status == "restricted")
            assert(stage[names[3]].values[2].kind == "table" and stage[names[3]].values[2].fields == nil)
            assert(stage[names[4]].status == "call-error" and stage[names[5]].status == "field-error")
        end
    end
end)

test("receiver revocation at every getter lookup and function guard blocks forwarding", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        for target = 1, 20 do
            local ctx = setup()
            local lookups, current, victim, revoked, forbidden = 0, nil, nil, false, 0
            installObjectHook(ctx, function(object, methods)
                for _, name in ipairs(names) do
                    local old = methods[name]
                    methods[name] = function(self, ...)
                        if revoked and rawequal(self, victim) then forbidden = forbidden + 1 end
                        return old(self, ...)
                    end
                end
                getmetatable(object).__index = function(_, key)
                    lookups = lookups + 1
                    current = methods[key]
                    if lookups == target then
                        victim = object
                        if phase == "lookup" then revoked = true end
                    end
                    return current
                end
            end)
            local function blocked(v)
                if phase ~= "lookup" and lookups == target and rawequal(v, current) then revoked = true end
                return revoked and rawequal(v, victim)
            end
            if phase == "secret" then issecretvalue = blocked
            else canaccessvalue = function(v) return not blocked(v) end end
            local row = capture()
            assert(forbidden == 0, phase .. " receiver forwarded")
            local lane = row.units[target <= 10 and 1 or 2]
            local stage = ((target - 1) % 10) < 5 and lane.before or lane.after
            assert(stage[names[(target - 1) % 5 + 1]].status ~= "observed")
        end
    end
end)

test("population lookup and both function guards reauthorize unit nil and calculator", function()
    for _, phase in ipairs({ "global-secret", "global-access", "lookup", "secret", "access" }) do
        for _, kind in ipairs({ "unit", "nil", "object" }) do
            for target = 1, 2 do
                local ctx = setup()
                local fn, lookups, revoked, forbidden, active = UnitGetDetailedHealPrediction, 0, false, 0, false
                local unit = target == 1 and "player" or "target"
                local function victim(v)
                    if kind == "unit" then return v == unit end
                    if kind == "nil" then return v == nil end
                    return rawequal(v, ctx.objects[target]) and ctx.objects[target] ~= nil
                end
                local wrapped = function(...)
                    if revoked then
                        local args = pack(...)
                        for i = 1, args.n do if victim(args[i]) then forbidden = forbidden + 1 end end
                    end
                    return fn(...)
                end
                UnitGetDetailedHealPrediction = nil
                setmetatable(_G, { __index = function(_, key)
                    if key == "UnitGetDetailedHealPrediction" then
                        lookups = lookups + 1
                        active = lookups == target
                        if active and phase == "lookup" then revoked = true end
                        return wrapped
                    end
                end })
                local function blocked(v, guard)
                    if guard == phase and active and rawequal(v, wrapped) then revoked = true end
                    -- The population lookup follows the fifth before getter.
                    if phase == "global-" .. guard and rawequal(v, _G) then
                        local last = ctx.trace[#ctx.trace]
                        if last and last.kind == names[5] and last.index == target and ctx.state[target] == 0 then
                            revoked = true
                        end
                    end
                    return revoked and victim(v)
                end
                issecretvalue = function(v) return blocked(v, "secret") end
                canaccessvalue = function(v) return not blocked(v, "access") end
                local row = capture()
                assert(forbidden == 0, phase .. " " .. kind .. " forwarded")
                assert(row.units[target].populate.status ~= "observed", phase .. " " .. kind)
                if kind ~= "object" then
                    for _, name in ipairs(names) do assert(row.units[target].after[name].status == "observed") end
                end
            end
        end
    end
end)

test("getter serialization revocation blocks later receiver lookups and population", function()
    local ctx = setup()
    local marker, revoked, illegalLookups = opaque(), false, 0
    installObjectHook(ctx, function(object, methods, index)
        if index == 1 then
            methods[names[1]] = function() return marker end
            getmetatable(object).__index = function(_, name)
                if revoked then illegalLookups = illegalLookups + 1 end
                return methods[name]
            end
        end
    end)
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not (revoked and rawequal(v, ctx.objects[1]))
    end
    local row = capture()
    assert(illegalLookups == 0 and row.units[1].populate.status ~= "observed")
    assert(row.units[2].populate.status == "observed" and ctx.populations == 1)
end)

test("population raw results are opaque and can revoke after-getter receiver", function()
    local ctx = setup()
    local fn, secret, marker, revoked = UnitGetDetailedHealPrediction, opaque(), opaque(), false
    UnitGetDetailedHealPrediction = function(...)
        fn(...)
        return secret, nil, marker
    end
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v)
        if rawequal(v, marker) then revoked = true end
        return not (revoked and rawequal(v, ctx.objects[1]))
    end
    local row = capture()
    assert(row.units[1].populate.n == 3 and row.units[1].populate.values[1].status == "restricted")
    assert(row.units[1].populate.values[2].kind == "nil")
    for _, name in ipairs(names) do assert(row.units[1].after[name].status ~= "observed") end
    assert(row.units[2].after[names[5]].status == "observed")
end)

test("missing access APIs make no constructor population or getter calls", function()
    for _, phase in ipairs({ "missing-secret", "missing-access", "throw-secret", "throw-access" }) do
        local ctx = setup()
        if phase == "missing-secret" then issecretvalue = nil
        elseif phase == "missing-access" then canaccessvalue = nil
        elseif phase == "throw-secret" then issecretvalue = function() error(opaque()) end
        else canaccessvalue = function() error(opaque()) end end
        SlashCmdList.APICONTRACTPROBE("unit-heal-calculator")
        assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual mode absent")
        assert(ctx.constructors == 0 and ctx.populations == 0 and #ctx.trace == 0)
    end
end)

test("bounded tuples strings and produced objects are not retained", function()
    setup()
    local weak = setmetatable({}, { __mode = "v" })
    local index = 0
    local function result()
        local value = opaque()
        weak[5] = value
        local values = { string.rep("x", 300), value }
        for i = 3, 20 do values[i] = i end
        return unpack(values, 1, 20)
    end
    CreateUnitHealPredictionCalculator = function()
        index = index + 1
        local object = setmetatable({}, { __index = function() return result end })
        weak[index] = object
        return object, nil
    end
    UnitGetDetailedHealPrediction = result
    local row = capture(string.rep("z", 180))
    for _, lane in ipairs(row.units) do
        local sample = lane.after.GetHealAbsorbs
        assert(sample.n == 20 and #sample.values == 16 and sample.truncated)
        assert(#sample.values[1].value == 256 and sample.values[1].truncated)
        assert(sample.values[2].kind == "table" and sample.values[2].fields == nil)
        assert(lane.populate.n == 20 and #lane.populate.values == 16)
    end
    collectgarbage("collect"); collectgarbage("collect")
    assert(weak[1] == nil and weak[2] == nil and weak[5] == nil, "raw object retained")
    assert(#ApiContractProbeDB.captures[1].label == 128)
end)

test("manual-only ten snapshots permit at most 240 experiment calls", function()
    local ctx = setup()
    local factory = CreateUnitHealPredictionCalculator
    -- Allow each snapshot its own two fresh receiver identities.
    CreateUnitHealPredictionCalculator = function(...)
        if ctx.constructors % 2 == 0 then ctx.state = {} end
        return factory(...)
    end
    UnitGetDetailedHealPrediction = function(...)
        local a = pack(...)
        assert(a.n == 3 and a[2] == nil)
        ctx.populations = ctx.populations + 1
        ctx.trace[#ctx.trace + 1] = { kind = "populate" }
    end
    SlashCmdList.APICONTRACTPROBE("all")
    assert(ctx.constructors == 0 and ctx.populations == 0)
    ApiContractProbeDB = nil
    for _ = 1, 11 do capture() end
    assert(ctx.constructors == 20 and ctx.populations == 20 and #ctx.trace == 240)
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("old calculator modes retain behavior when interleaved with new mode", function()
    local ctx = setup()
    Enum = {
        UnitHealAbsorbMode = { ReducedByIncomingHeals = 71, Total = 72 },
        UnitHealAbsorbClampMode = { CurrentHealth = 81, MaximumHealth = 82 },
        UnitDamageAbsorbClampMode = { MissingHealth = 91, MissingHealthWithoutIncomingHeals = 92, MaximumHealth = 93 },
    }
    installObjectHook(ctx, function(_, methods)
        for _, name in ipairs({ "SetHealAbsorbMode", "SetHealAbsorbClampMode", "SetDamageAbsorbClampMode",
            "Reset", "ResetPredictedValues" }) do
            methods[name] = function(self, ...)
                ctx.trace[#ctx.trace + 1] = { kind = name, args = pack(...) }
            end
        end
    end)
    UnitGetDetailedHealPrediction = function(...)
        local a = pack(...)
        assert(a.n == 3 and a[2] == nil and (a[1] == "player" or a[1] == "target"))
        assert(rawequal(a[3], ctx.objects[ctx.constructors]))
        ctx.populations = ctx.populations + 1
        ctx.trace[#ctx.trace + 1] = { kind = "populate" }
    end
    local function check(mode, constructors, calls, populations)
        local c, n, p = ctx.constructors, #ctx.trace, ctx.populations
        SlashCmdList.APICONTRACTPROBE(mode)
        assert(ctx.constructors - c == constructors and #ctx.trace - n == calls and ctx.populations - p == populations)
    end
    check("heal-calculator", 2, 22, 0)
    check("unit-heal-calculator", 2, 24, 2)
    check("heal-calculator-modes", 1, 26, 0)
    check("all", 0, 0, 0)
    check("heal-calculator", 2, 22, 0)
    check("unit-heal-calculator", 2, 24, 2)
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
