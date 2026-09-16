local root = assert(arg[1])
local passed = 0
local apis = { "UnitPower", "UnitPowerMax", "UnitPowerPercent" }
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret inspection") end
local calls
local function setup()
    calls = {}
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 1 end
    Enum = { PowerType = { Mana = 10.5, Rage = 20.5, Energy = 30.5 } }
    for _, name in ipairs(apis) do
        _G[name] = function(...)
            calls[#calls + 1] = { name = name, n = select("#", ...), ... }
            return 7, nil, "result"
        end
    end
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("explicit-power " .. (label or "sample"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "explicit-power mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].explicitPower)
end
local function test(name, fn)
    setup(); fn(); passed = passed + 1; print("PASS " .. name)
end

test("36 exact calls preserve published values and explicit nil curve", function()
    local rows = capture()
    assert(#calls == 36 and #rows == 36)
    for i, call in ipairs(calls) do
        local offset = i - 1
        assert(call[1] == (offset < 18 and "player" or "target"))
        assert(call[2] == (math.floor(offset % 18 / 6) + 1) * 10 + 0.5)
        assert(call[3] == (math.floor(offset % 6 / 3) == 1))
        assert(call.name == apis[offset % 3 + 1])
        assert(call.n == (call.name == "UnitPowerPercent" and 4 or 3) and call[4] == nil)
        assert(rows[i].result.n == 3 and rows[i].result.values[2].kind == "nil")
    end
end)
test("invalid publications have no numeric fallback", function()
    for _, bad in ipairs({ false, "10", math.huge, 0/0, secret }) do
        setup(); Enum.PowerType.Mana = bad; capture(); assert(#calls == 24)
    end
    setup(); Enum = nil; capture(); assert(#calls == 0)
end)
test("function guards revoke original unit or type before call", function()
    for _, input in ipairs({ "player", 10.5 }) do
        for _, guard in ipairs({ "issecretvalue", "canaccessvalue" }) do
            setup()
            local fn, revoked = UnitPower, false
            _G[guard] = function(v)
                if rawequal(v, fn) then revoked = true end
                local denied = rawequal(v, secret) or (revoked and rawequal(v, input))
                if guard == "issecretvalue" then return denied end
                return not denied
            end
            capture()
            for _, c in ipairs(calls) do assert(c[1] ~= input and c[2] ~= input) end
        end
    end
end)
test("global lookup revocation blocks forwarding", function()
    local fn, revoked = UnitPower, false
    UnitPower = nil
    local old = getmetatable(_G)
    setmetatable(_G, { __index = function(_, key)
        if key == "UnitPower" then revoked = true; return fn end
    end })
    canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and v == 10.5) end
    capture()
    setmetatable(_G, old)
    for _, c in ipairs(calls) do assert(c[2] ~= 10.5) end
end)
test("errors missing functions and restricted results remain independent", function()
    UnitPower = function() error(secret) end
    UnitPowerMax = nil
    UnitPowerPercent = function() return secret, nil end
    local rows = capture()
    assert(rows[1].result.status == "call-error")
    assert(rows[2].result.status == "missing-api")
    assert(rows[3].result.n == 2 and rows[3].result.values[1].status == "restricted")
end)
test("tuple string label and snapshot bounds", function()
    UnitPowerPercent = function() return unpack({string.rep("x",300),2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17}) end
    local rows = capture(string.rep("l",200))
    assert(rows[3].result.n == 17 and rows[3].result.truncated)
    assert(#rows[3].result.values == 16 and #rows[3].result.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    for _ = 1, 10 do capture() end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)
test("opaque result objects are collectible", function()
    local weak = setmetatable({}, { __mode = "v" })
    UnitPowerPercent = function() local o = {}; weak[1] = o; return o end
    capture(); collectgarbage("collect"); assert(weak[1] == nil)
end)
test("all excludes explicit power calls", function()
    for _, name in ipairs(apis) do _G[name] = function() error("explicit power called from all") end end
    SlashCmdList.APICONTRACTPROBE("all")
    assert(not ApiContractProbeDB.captures[1].explicitPower and #calls == 0)
end)
print("PASS " .. passed .. " explicit-power fixtures")
