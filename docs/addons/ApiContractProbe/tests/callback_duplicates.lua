local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local secret = newproxy(true)
getmetatable(secret).__tostring = function() error("opaque error inspected") end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function setup()
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    GetTime = function() return 42 end
    time = GetTime
    local state = { registrations = { global = {}, unit = {} }, removals = { global = {}, unit = {} } }
    for _, lane in ipairs({ "global", "unit" }) do
        local function register(...)
            local event, cb, unit = ...
            assert(event == "UNIT_HEALTH" and type(cb) == "function")
            assert(select("#", ...) == (lane == "unit" and 3 or 2))
            assert(unit == (lane == "unit" and "player" or nil))
            local calls = state.registrations[lane]
            calls[#calls + 1] = cb
            if state.onRegister then return state.onRegister(lane, #calls, cb) end
            return true, nil, #calls
        end
        local function unregister(...)
            local event, cb, unit = ...
            assert(event == "UNIT_HEALTH" and type(cb) == "function")
            assert(select("#", ...) == (lane == "unit" and 3 or 2))
            assert(unit == (lane == "unit" and "player" or nil))
            local calls = state.removals[lane]
            calls[#calls + 1] = cb
            if state.onRemove then return state.onRemove(lane, #calls, cb) end
            return true
        end
        if lane == "global" then RegisterEventCallback, UnregisterEventCallback = register, unregister
        else RegisterUnitEventCallback, UnregisterUnitEventCallback = register, unregister end
    end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
    return state
end
local function start(label)
    SlashCmdList.APICONTRACTPROBE("callbacks-duplicate-start " .. (label or "sample"))
    assert(ApiContractProbeDB and ApiContractProbeDB.callbackSessions, "duplicate mode absent")
    return assert(ApiContractProbeDB.callbackSessions[#ApiContractProbeDB.callbackSessions])
end
local function stop() SlashCmdList.APICONTRACTPROBE("callbacks-stop") end
local function count(calls) return #calls.global + #calls.unit end

test("duplicate lanes register and remove the exact same closure twice", function()
    local s = setup()
    local record = start()
    assert(count(s.registrations) == 4 and record.status == "recording")
    assert(s.registrations.global[1] ~= s.registrations.unit[1])
    for _, lane in ipairs({ "global", "unit" }) do
        assert(s.registrations[lane][1] == s.registrations[lane][2])
        for i = 1, 2 do
            assert(record[lane].registrations[i].n == 3)
            assert(record[lane].registrations[i].values[2].kind == "nil")
            assert(record[lane].registrations[i].values[3].value == i)
        end
    end
    stop()
    assert(record.status == "stopped" and count(s.removals) == 4)
    for _, lane in ipairs({ "global", "unit" }) do
        for i = 1, 2 do
            assert(s.removals[lane][i] == s.registrations[lane][1])
            assert(record[lane].removals[i].values[1].value == true)
        end
    end
    stop()
    assert(count(s.removals) == 4 and ApiContractProbeDB.callbackStatus == "not-running")
end)

test("throw after registration and second refusal retain both cleanup slots", function()
    local s = setup()
    s.onRegister = function(lane, i)
        if lane == "global" and i == 1 then error(secret) end
        if i == 2 then return false end
    end
    local r = start()
    assert(count(s.registrations) == 4 and r.status == "registration-incomplete")
    assert(r.global.registrations[1].status == "call-error")
    assert(r.global.registrations[2].values[1].value == false)
    assert(r.unit.registrations[1].n == 0)
    stop()
    assert(count(s.removals) == 4 and r.status == "stopped")
end)

test("second registration errors keep both scheduled removals", function()
    local s = setup()
    s.onRegister = function(_, i) if i == 2 then error(secret) end end
    local r = start()
    assert(r.global.registrations[1].n == 0 and r.unit.registrations[1].n == 0)
    assert(r.global.registrations[2].status == "call-error")
    assert(r.unit.registrations[2].status == "call-error")
    stop()
    assert(count(s.removals) == 4 and r.status == "stopped")
end)

test("registration tuples preserve zero nil and bounded raw output", function()
    local s = setup()
    s.onRegister = function(lane, i)
        if lane == "unit" then return nil, true, nil end
        if i == 1 then return end
        return true, string.rep("x", 300), 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, nil
    end
    local r = start()
    assert(r.global.registrations[1].n == 0)
    assert(r.global.registrations[2].n == 17 and r.global.registrations[2].truncated)
    assert(#r.global.registrations[2].values == 16)
    assert(#r.global.registrations[2].values[2].value == 256)
    assert(r.unit.registrations[2].n == 3 and r.unit.registrations[2].values[3].kind == "nil")
    assert(r.status == "registration-incomplete")
    stop()
    assert(count(s.removals) == 4)
end)

test("partial cleanup retries only pending exact identities on explicit stop", function()
    local s = setup()
    local r = start()
    s.onRemove = function(lane, i)
        if lane == "global" and i == 1 then return false end
        if lane == "unit" and i == 2 then error(secret) end
        return true
    end
    stop()
    assert(count(s.removals) == 4 and r.status == "cleanup-incomplete")
    assert(r.global.removals[1].values[1].value == false)
    assert(r.unit.removals[2].status == "call-error")
    SlashCmdList.APICONTRACTPROBE("callbacks-start refused")
    start("also refused")
    assert(#ApiContractProbeDB.callbackSessions == 1 and count(s.registrations) == 4)
    assert(ApiContractProbeDB.callbackStatus == "already-running")
    stop()
    assert(count(s.removals) == 6 and r.status == "stopped")
    assert(s.removals.global[3] == s.registrations.global[1])
    assert(s.removals.unit[3] == s.registrations.unit[1])
    assert(r.global.removalAttempts[1] == 2 and r.global.removalAttempts[2] == 1)
    assert(r.unit.removalAttempts[1] == 1 and r.unit.removalAttempts[2] == 2)
end)

test("deduplicating fixture false second removal is never guessed away", function()
    local s = setup()
    local r = start()
    s.onRemove = function(_, i) return i == 1 end
    stop()
    assert(r.status == "cleanup-incomplete" and count(s.removals) == 4)
    stop()
    assert(r.status == "cleanup-incomplete" and count(s.removals) == 6)
    start("blocked")
    assert(#ApiContractProbeDB.callbackSessions == 1 and count(s.registrations) == 4)
end)

test("uncertain cleanup results preserve pending slots", function()
    local s = setup()
    local r = start()
    s.onRemove = function(lane, i)
        if lane == "global" and i == 1 then return secret end
        if lane == "unit" and i == 2 then
            return true, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
        end
    end
    stop()
    assert(r.status == "cleanup-incomplete")
    assert(r.global.removals[1].values[1].status == "restricted")
    assert(r.unit.removals[2].n == 17 and r.unit.removals[2].truncated)
    stop()
    assert(r.status == "stopped" and count(s.removals) == 6)
end)

test("missing registration API has no pending removal while peer still duplicates", function()
    local s = setup()
    RegisterEventCallback = nil
    local r = start()
    assert(#s.registrations.global == 0 and #s.registrations.unit == 2)
    assert(r.global.registrations[1].status == "missing-api")
    assert(r.global.registrations[2].status == "missing-api")
    stop()
    assert(#s.removals.global == 0 and #s.removals.unit == 2 and r.status == "stopped")
end)

test("denied registration availability cannot invoke an untracked attempt", function()
    local s = setup()
    local fn, checked = RegisterEventCallback, false
    canaccessvalue = function(v)
        if rawequal(v, fn) and not checked then checked = true; return false end
        return not rawequal(v, secret)
    end
    local r = start()
    assert(#s.registrations.global == 1, "denied availability was retried inside one attempt")
    assert(r.global.registrations[1].status == "missing-api")
    stop()
    assert(#s.removals.global == 1 and #s.removals.unit == 2 and r.status == "stopped")
end)

test("missing cleanup API retains slots and peer cleanup proceeds", function()
    local s = setup()
    local r = start()
    local restore = UnregisterEventCallback
    UnregisterEventCallback = nil
    stop()
    assert(#s.removals.global == 0 and #s.removals.unit == 2)
    assert(r.status == "cleanup-incomplete" and r.global.removals[2].status == "missing-api")
    UnregisterEventCallback = restore
    stop()
    assert(count(s.removals) == 4 and r.status == "stopped")
end)

test("event and unit revocation by function guards blocks forwarding", function()
    for _, guard in ipairs({ "secret", "access" }) do
        for _, token in ipairs({ "UNIT_HEALTH", "player" }) do
            for _, phase in ipairs({ "register", "remove" }) do
                local s = setup()
                local r
                if phase == "remove" then r = start() end
                local fn = phase == "register" and RegisterUnitEventCallback or UnregisterUnitEventCallback
                local revoked = false
                local originalSecret, originalAccess = issecretvalue, canaccessvalue
                issecretvalue = function(v)
                    if guard == "secret" and rawequal(v, fn) then revoked = true end
                    return originalSecret(v)
                end
                canaccessvalue = function(v)
                    if guard == "access" and rawequal(v, fn) then revoked = true end
                    return originalAccess(v) and not (revoked and rawequal(v, token))
                end
                if phase == "register" then r = start() else stop() end
                local calls = phase == "register" and s.registrations or s.removals
                assert(#calls.unit == 0, "revoked token forwarded")
                local result = phase == "register" and r.unit.registrations or r.unit.removals
                assert(result[1].status == "restricted-input")
            end
        end
    end
end)

test("callback revocation at registration and removal never forwards it", function()
    for _, phase in ipairs({ "register", "remove" }) do
        local s = setup()
        local owned
        local r
        if phase == "remove" then r = start(); owned = s.registrations.global[1] end
        local fn = phase == "register" and RegisterEventCallback or UnregisterEventCallback
        local revoked = false
        local known = { [RegisterEventCallback] = true, [RegisterUnitEventCallback] = true,
            [UnregisterEventCallback] = true, [UnregisterUnitEventCallback] = true,
            [GetBuildInfo] = true, [GetTime] = true }
        canaccessvalue = function(v)
            if type(v) == "function" and not known[v] and not owned then owned = v end
            if rawequal(v, fn) then revoked = true end
            return not rawequal(v, secret) and not (revoked and owned ~= nil and rawequal(v, owned))
        end
        if phase == "register" then r = start() else stop() end
        local calls = phase == "register" and s.registrations or s.removals
        assert(#calls.global == 0, "revoked callback forwarded")
        assert(#calls.unit == 2, "peer callback suppressed")
    end
end)

test("fixture deliveries are bounded and stop disables recording before removal", function()
    local s = setup()
    local r = start(string.rep("l", 200))
    assert(#r.label == 128 and #r.events == 0, "recorder dispatched callback")
    local cb = s.registrations.global[1]
    cb(nil, secret, string.rep("x", 300), 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, nil)
    assert(r.events[1].payload.n == 17 and r.events[1].payload.truncated)
    assert(r.events[1].payload.values[1].kind == "nil")
    assert(r.events[1].payload.values[2].status == "restricted")
    assert(#r.events[1].payload.values[3].value == 256)
    for i = 1, 130 do s.registrations.unit[1](i) end
    assert(#r.events == 128 and r.dropped == 3)
    s.onRemove = function(_, _, callback) callback("during cleanup"); return true end
    stop(); cb("late")
    assert(#r.events == 128 and r.dropped == 3)
end)

test("one active session and ten completed sessions bound registration", function()
    local s = setup()
    for i = 1, 10 do
        local r = start(tostring(i))
        assert(#ApiContractProbeDB.callbackSessions == i)
        start("refused")
        assert(count(s.registrations) == i * 4)
        stop()
        assert(r.status == "stopped")
    end
    SlashCmdList.APICONTRACTPROBE("callbacks-duplicate-start overflow")
    assert(ApiContractProbeDB.callbackStatus == "session-limit")
    assert(count(s.registrations) == 40 and count(s.removals) == 40)
end)

test("normal mode preserves singular records cleanup retries and interleaving", function()
    local s = setup()
    SlashCmdList.APICONTRACTPROBE("callbacks-start normal")
    local db = ApiContractProbeDB
    local normal = db.callbackSessions[1]
    assert(count(s.registrations) == 2 and normal.global.registration.n == 3)
    assert(normal.global.registrations == nil and normal.unit.registrations == nil)
    start("refused")
    assert(#db.callbackSessions == 1 and count(s.registrations) == 2)
    s.onRemove = function(lane, i) if lane == "global" and i == 1 then return false end end
    stop()
    assert(normal.status == "cleanup-incomplete" and normal.global.removal.values[1].value == false)
    stop()
    assert(normal.status == "stopped" and count(s.removals) == 3)
    assert(normal.global.removals == nil)
    s.onRemove = nil
    local duplicate = start("duplicate")
    assert(count(s.registrations) == 6)
    stop()
    assert(duplicate.status == "stopped" and count(s.removals) == 7)
    SlashCmdList.APICONTRACTPROBE("callbacks-start normal-again")
    assert(count(s.registrations) == 8 and #db.callbackSessions == 3)
    stop()
    assert(count(s.removals) == 9)
end)

test("missing access APIs cannot start duplicate registrations", function()
    local s = setup()
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("callbacks-duplicate-start blocked")
    assert(ApiContractProbeDB and ApiContractProbeDB.callbackStatus == "missing-access-api")
    assert(count(s.registrations) == 0)
end)

print(string.format("%d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
