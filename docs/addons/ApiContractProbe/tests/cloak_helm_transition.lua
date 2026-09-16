local root = assert(arg[1], "addon directory required")
local passed, total = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspection") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function setup(cloak, helm)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture" end
    time = function() return 42 end
    local f = { state = { cloak = cloak, helm = helm }, calls = {}, reads = {}, writes = {} }
    for _, lane in ipairs({ "cloak", "helm" }) do
        local key = lane
        local suffix = lane == "cloak" and "Cloak" or "Helm"
        _G["Showing" .. suffix] = function(...)
            assert(select("#", ...) == 0)
            f.calls[#f.calls + 1] = { key, "read" }
            f.reads[key] = (f.reads[key] or 0) + 1
            if f.get then return f.get(key, f.reads[key]) end
            return f.state[key]
        end
        _G["Show" .. suffix] = function(...)
            assert(select("#", ...) == 1)
            local value = ...
            assert(type(value) == "boolean")
            f.calls[#f.calls + 1] = { key, "set", value }
            f.writes[key] = (f.writes[key] or 0) + 1
            f.state[key] = value
            if f.set then return f.set(key, value, f.writes[key]) end
        end
    end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
    return f
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("cloak-helm-transition " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "transition mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].cloakHelmTransition,
        "transition observation absent")
end
local function test(name, fn)
    total = total + 1
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("original false and true baselines restored in exact independent seven-call lanes", function()
    for _, initial in ipairs({ { false, true }, { true, false } }) do
        local f = setup(initial[1], initial[2])
        local r = capture()
        assert(#f.calls == 14)
        for i, lane in ipairs({ "cloak", "helm" }) do
            local offset = (i - 1) * 7
            local expected = { { "read" }, { "set", false }, { "read" }, { "set", true },
                { "read" }, { "set", initial[i] }, { "read" } }
            for j, call in ipairs(expected) do
                local actual = f.calls[offset + j]
                assert(actual[1] == lane and actual[2] == call[1] and actual[3] == call[2])
            end
            assert(f.state[lane] == initial[i])
            assert(r[lane].restoration.status == "confirmed-by-observation")
            assert(r[lane].restoration.read.values[1].value == initial[i])
        end
    end
end)

test("unknown missing throwing or restricted baseline performs no setters in that lane", function()
    for _, kind in ipairs({ "zero", "nil", "string", "secret", "throw", "missing" }) do
        local f = setup(false, true)
        if kind == "missing" then ShowingCloak = nil
        else
            f.get = function(lane)
                if lane == "helm" then return f.state[lane] end
                if kind == "zero" then return end
                if kind == "nil" then return nil end
                if kind == "string" then return "false" end
                if kind == "secret" then return secret end
                error(secret)
            end
        end
        local r = capture()
        assert(not f.writes.cloak and f.writes.helm == 3)
        assert(r.cloak.status == "transition-unavailable")
        assert(r.helm.restoration.status == "confirmed-by-observation")
        assert(capture().status ~= "blocked-restoration-unconfirmed")
    end
end)

test("throw after each experimental state change still attempts original restoration", function()
    for _, failing in ipairs({ 1, 2 }) do
        local f = setup(false, true)
        f.set = function(lane, _, index)
            if lane == "cloak" and index == failing then error(secret) end
        end
        local r = capture()
        assert(f.writes.cloak == 3 and f.writes.helm == 3 and f.state.cloak == false)
        assert(r.cloak.steps[failing].setter.status == "call-error")
        assert(r.cloak.restoration.status == "confirmed-by-observation")
    end
end)

test("intermediate getter failures do not suppress cleanup or the peer lane", function()
    for _, failing in ipairs({ 2, 3 }) do
        local f = setup(true, false)
        f.get = function(lane, index)
            if lane == "cloak" and index == failing then error(secret) end
            return f.state[lane]
        end
        local r = capture()
        assert(#f.calls == 14 and f.state.cloak == true and f.state.helm == false)
        assert(r.cloak.steps[failing - 1].read.status == "call-error")
        assert(r.cloak.restoration.status == "confirmed-by-observation")
    end
end)

test("restoration error missing API mismatch or inaccessible final read blocks later captures", function()
    for _, failure in ipairs({ "throw", "missing", "mismatch", "read-error", "read-secret", "read-nil" }) do
        local f = setup(false, true)
        f.set = function(lane, _, index)
            if lane == "cloak" and index == 2 and failure == "missing" then ShowCloak = nil end
            if lane == "cloak" and index == 3 then
                if failure == "throw" then error(secret) end
                if failure == "mismatch" then f.state.cloak = true end
            end
        end
        f.get = function(lane, index)
            if lane == "cloak" and index == 4 then
                if failure == "read-error" then error(secret) end
                if failure == "read-secret" then return secret end
                if failure == "read-nil" then return nil end
            end
            return f.state[lane]
        end
        local r = capture()
        assert(r.cloak.restoration.status ~= "confirmed-by-observation")
        assert(f.writes.helm == 3, "peer lane suppressed by cleanup failure")
        local before = #f.calls
        assert(capture().status == "blocked-restoration-unconfirmed")
        assert(#f.calls == before, "blocked capture invoked native calls")
        ApiContractProbeDB = nil
        assert(capture().status == "blocked-restoration-unconfirmed", "DB reset cleared session lock")
        assert(#f.calls == before)
    end
end)

test("baseline revocation after mutation stops further experimental setters and skips cleanup", function()
    local f = setup(false, true)
    local revoked = false
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and v == false) end
    f.get = function(lane, index)
        if lane == "cloak" and index == 2 then revoked = true end
        return f.state[lane]
    end
    local r = capture()
    assert(f.writes.cloak == 1, "continued mutation without usable restoration baseline")
    assert(r.cloak.steps[2].setter.status == "restricted-baseline")
    assert(r.cloak.restoration.setter.status == "restricted-baseline")
    assert(r.cloak.restoration.status == "restoration-skipped")
    local count = #f.calls
    assert(capture().status == "blocked-restoration-unconfirmed" and #f.calls == count)
end)

test("setter lookup and both function guards may revoke baseline before mutation", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        local f = setup(false, true)
        local setter, revoked = ShowCloak, false
        local old = getmetatable(_G)
        canaccessvalue = function(v)
            if phase == "access" and rawequal(v, setter) then revoked = true end
            return not rawequal(v, secret) and not (revoked and v == false)
        end
        issecretvalue = function(v)
            if phase == "secret" and rawequal(v, setter) then revoked = true end
            return rawequal(v, secret)
        end
        if phase == "lookup" then
            ShowCloak = nil
            setmetatable(_G, { __index = function(_, key)
                if key == "ShowCloak" then revoked = true; return setter end
            end })
        end
        local ok, r = pcall(capture)
        setmetatable(_G, old)
        assert(ok, r)
        assert(not f.writes.cloak, "revoked baseline forwarded to setter")
        assert(r.cloak.restoration.status == "not-needed")
    end
end)

test("setter argument and function restrictions block calls without inventing restoration", function()
    for _, phase in ipairs({ "argument", "function", "receiver" }) do
        local f = setup(true, true)
        local setter = ShowCloak
        canaccessvalue = function(v)
            if phase == "argument" and v == false then return false end
            if phase == "function" and rawequal(v, setter) then return false end
            if phase == "receiver" and rawequal(v, _G) then return false end
            return not rawequal(v, secret)
        end
        local r = capture()
        if phase == "argument" then
            assert(f.writes.cloak == 2 and f.state.cloak == true)
            assert(r.cloak.steps[1].setter.status == "restricted-argument")
        else assert(not f.writes.cloak) end
    end
end)

test("restoration function guard revocation cannot produce a false confirmed result", function()
    local f = setup(false, true)
    local setter, revoked = ShowCloak, false
    canaccessvalue = function(v)
        if rawequal(v, setter) and f.writes.cloak == 2 then revoked = true end
        return not rawequal(v, secret) and not (revoked and v == false)
    end
    local r = capture()
    assert(f.writes.cloak == 2 and f.state.cloak == true)
    assert(r.cloak.restoration.status == "restoration-skipped")
    assert(capture().status == "blocked-restoration-unconfirmed")
end)

test("final getter may revoke baseline and prevent equality confirmation", function()
    local f = setup(false, true)
    local revoked = false
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and v == false) end
    f.get = function(lane, index)
        if lane == "cloak" and index == 4 then revoked = true end
        return f.state[lane]
    end
    local r = capture()
    assert(f.writes.cloak == 3 and r.cloak.restoration.status == "restoration-unconfirmed")
    assert(capture().status == "blocked-restoration-unconfirmed")
end)

test("raw tuples nil positions strings labels and ten snapshots stay bounded", function()
    local f = setup(false, true)
    f.get = function(lane)
        return f.state[lane], nil, string.rep("g", 300), 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, secret
    end
    f.set = function()
        return nil, string.rep("s", 300), 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, secret
    end
    local r = capture(string.rep("l", 200))
    assert(#ApiContractProbeDB.captures[1].label == 128)
    assert(r.cloak.baseline.n == 17 and r.cloak.baseline.truncated)
    assert(#r.cloak.baseline.values == 16 and r.cloak.baseline.values[2].kind == "nil")
    assert(#r.cloak.baseline.values[3].value == 256)
    assert(r.cloak.steps[1].setter.n == 17 and r.cloak.steps[1].setter.truncated)
    assert(r.cloak.steps[1].setter.values[1].kind == "nil")
    assert(#r.cloak.steps[1].setter.values[2].value == 256)
    for i = 2, 11 do SlashCmdList.APICONTRACTPROBE("cloak-helm-transition cap") end
    assert(#f.calls == 140 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)

test("all and existing player-state queries never mutate even after transition lockout", function()
    local f = setup(false, true)
    f.set = function(lane, _, index) if lane == "cloak" and index == 3 then error(secret) end end
    capture()
    local writes = f.writes.cloak + f.writes.helm
    SlashCmdList.APICONTRACTPROBE("all passive")
    SlashCmdList.APICONTRACTPROBE("player-state-queries passive")
    assert(f.writes.cloak + f.writes.helm == writes)
    assert(ApiContractProbeDB.captures[2].cloakHelmTransition == nil)
    assert(ApiContractProbeDB.captures[3].playerStateQueries.ShowingCloak[1].status == "observed")
    setup(false, true)
    SlashCmdList.APICONTRACTPROBE("all fresh")
    assert(ApiContractProbeDB.captures[1].cloakHelmTransition == nil)
end)

test("missing guard APIs fail closed before baseline or mutation", function()
    for _, name in ipairs({ "issecretvalue", "canaccessvalue" }) do
        local f = setup(false, true)
        _G[name] = nil
        SlashCmdList.APICONTRACTPROBE("cloak-helm-transition unavailable")
        assert(ApiContractProbeDB and ApiContractProbeDB.captures[1].status == "missing-access-api")
        assert(#f.calls == 0)
    end
end)

test("opaque results and errors are not retained by records or session lock", function()
    local f = setup(false, true)
    local weak = setmetatable({}, { __mode = "v" })
    f.set = function(lane, _, index)
        local object = newproxy(true)
        weak[#weak + 1] = object
        if lane == "cloak" and index == 3 then error(object) end
        return object
    end
    capture()
    collectgarbage("collect")
    assert(next(weak) == nil)
    assert(capture().status == "blocked-restoration-unconfirmed")
end)

print(string.format("%d/%d cloak-helm-transition fixtures passed", passed, total))
assert(passed == total, "cloak-helm-transition fixture failures")
