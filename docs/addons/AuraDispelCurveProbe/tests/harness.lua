local root = assert(arg[1], "addon directory required")
local secret = setmetatable({}, { __tostring = function() error("secret stringified") end })
local mode = "normal"
local calls = 0
issecretvalue = function(v) return rawequal(v, secret) end
canaccessvalue = function(v) return not rawequal(v, secret) end
GetBuildInfo = function() return "12.1.0", "69497", "fixture", 120100 end
time = function() return 12345 end
SlashCmdList = {}
Enum = { LuaCurveType = { Linear = 0 } }
CreateColor = function(r, g, b, a) return { r = r, g = g, b = b, a = a } end
C_CurveUtil = { CreateColorCurve = function()
    local curve = { points = {} }
    function curve:SetType(t) self.kind = t end
    function curve:AddPoint(x, c) self.points[#self.points + 1] = { x, c } end
    return curve
end }
C_UnitAuras = {}
function C_UnitAuras.GetAuraDataByIndex(unit, index, filter)
    assert(unit == "player")
    if mode == "error" then error("fixture API failure") end
    if mode ~= "many" and index > 1 then return nil end
    if mode == "secretAura" then return secret end
    return { name = mode == "secretName" and secret or filter .. " aura",
        dispelName = filter == "HELPFUL" and "Magic" or "Poison",
        auraInstanceID = mode == "secretID" and secret or (filter == "HELPFUL" and 101 or 202) }
end
function C_UnitAuras.GetAuraDispelTypeColor(unit, id, curve)
    calls = calls + 1
    assert(unit == "player" and (id == 101 or id == 202))
    if mode == "colorError" then error(secret) end
    if mode == "secretColor" then return secret end
    local x = id == 101 and 1 or 4
    local p, q = curve.points[1], curve.points[2]
    assert(curve.kind == 0)
    local t = (x - p[1]) / (q[1] - p[1])
    local result = {}
    for _, k in ipairs({ "r", "g", "b", "a" }) do result[k] = p[2][k] + t * (q[2][k] - p[2][k]) end
    if mode == "secretComponent" then result.g = secret end
    return result
end
local file = io.open(root .. "/AuraDispelCurveProbe.lua", "r")
if file then file:close(); assert(loadfile(root .. "/AuraDispelCurveProbe.lua"))() end
assert(type(SlashCmdList.AURADISPELCURVEPROBE) == "function", "manual capture command missing")
assert(AuraDispelCurveProbeDB == nil, "automatic capture")
local function capture(m)
    mode = m
    SlashCmdList.AURADISPELCURVEPROBE("")
    return AuraDispelCurveProbeDB.captures[#AuraDispelCurveProbeDB.captures]
end
local r = capture("normal")
assert(r.client.build == "69497" and r.client.interface == 120100 and r.time == 12345)
assert(#r.auras == 2 and #r.curves == 2)
assert(r.auras[1].name == "HELPFUL aura" and r.auras[2].dispelName == "Poison")
assert(r.auras[1].colors[1].r == 1/32 and r.auras[2].colors[1].r == 4/32)
assert(r.auras[1].colors[2].r == 17/64)
for _, m in ipairs({ "secretAura", "secretName", "secretID" }) do
    local before = calls
    r = capture(m)
    assert(#r.auras == 0 and #r.failures > 0 and calls == before, m)
end
for _, m in ipairs({ "secretColor", "secretComponent", "colorError" }) do
    r = capture(m)
    assert(r.auras[1].colors[1].status ~= "ok", m)
    assert(r.auras[1].colors[1].r == nil, m)
end
r = capture("error")
assert(#r.failures > 0 and #r.auras == 0)
r = capture("many")
assert(#r.auras == 80 and r.scanLimitReached)
C_CurveUtil.CreateColorCurve = nil
r = capture("normal")
assert(r.status == "missing-api")
for i = 1, 20 do capture("normal") end
assert(#AuraDispelCurveProbeDB.captures == 10 and AuraDispelCurveProbeDB.dropped > 0)
local function clean(v)
    assert(not rawequal(v, secret), "secret persisted")
    if type(v) == "table" then
        assert(getmetatable(v) == nil)
        for k, child in pairs(v) do clean(k); clean(child) end
    else assert(type(v) ~= "function" and type(v) ~= "userdata") end
end
clean(AuraDispelCurveProbeDB)
io.write("Aura dispel probe fixtures passed\n")
