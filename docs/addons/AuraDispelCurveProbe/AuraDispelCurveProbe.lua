-- Raw observations only: curve domain is an experiment, not a dispel-ID claim.
local function accessible(value)
    if type(issecretvalue) ~= "function" or type(canaccessvalue) ~= "function" then
        return false
    end
    local okSecret, secret = pcall(issecretvalue, value)
    if not okSecret or secret then return false end
    local okAccess, access = pcall(canaccessvalue, value)
    return okAccess and access == true
end

local function scalar(value)
    if not accessible(value) then return nil, false end
    local kind = type(value)
    if kind == "string" then return string.sub(value, 1, 256), true end
    if kind == "number" then
        if value ~= value or value == math.huge or value == -math.huge then return nil, false end
        return value, true
    end
    return nil, kind == "nil"
end

local function field(object, key)
    if not accessible(object) then return nil, false end
    local ok, value = pcall(function() return object[key] end)
    if not ok then return nil, false end
    return scalar(value)
end

local function definitions()
    return {
        { type = "Linear", points = {
            { x = 0, r = 0, g = 1, b = 0.25, a = 1 },
            { x = 32, r = 1, g = 0, b = 0.25, a = 1 },
        } },
        { type = "Linear", points = {
            { x = -16, r = 0, g = 0.25, b = 1, a = 1 },
            { x = 48, r = 1, g = 0.75, b = 0, a = 1 },
        } },
    }
end

local function makeCurve(definition)
    local curve = C_CurveUtil.CreateColorCurve()
    curve:SetType(Enum.LuaCurveType.Linear)
    for _, p in ipairs(definition.points) do
        curve:AddPoint(p.x, CreateColor(p.r, p.g, p.b, p.a))
    end
    return curve
end

local function color(id, curve)
    local ok, value = pcall(C_UnitAuras.GetAuraDispelTypeColor, "player", id, curve)
    if not ok then return { status = "api-error" } end
    if not accessible(value) then return { status = "restricted" } end
    local result = { status = "ok" }
    for _, key in ipairs({ "r", "g", "b", "a" }) do
        local component, safe = field(value, key)
        if not safe or type(component) ~= "number" then
            return { status = "restricted-or-invalid-component" }
        end
        result[key] = component
    end
    return result
end

local function observeAura(aura, curves, filter, index, record)
    local name, safeName = field(aura, "name")
    local dispel, safeDispel = field(aura, "dispelName")
    local id, safeID = field(aura, "auraInstanceID")
    if not safeName or not safeDispel or not safeID or type(id) ~= "number"
        or type(name) ~= "string" or (dispel ~= nil and type(dispel) ~= "string") then
        record.failures[#record.failures + 1] = { filter = filter, index = index, status = "restricted-or-invalid-aura" }
        return
    end
    local entry = { name = name, dispelName = dispel, auraInstanceID = id,
        filter = filter, index = index, colors = {} }
    for n, curve in ipairs(curves) do entry.colors[n] = color(id, curve) end
    record.auras[#record.auras + 1] = entry
end

local function scan(record, curves, filter)
    for index = 1, 40 do
        local ok, aura = pcall(C_UnitAuras.GetAuraDataByIndex, "player", index, filter)
        if not ok then
            record.failures[#record.failures + 1] = { filter = filter, index = index, status = "enumeration-error" }
            return
        end
        if not accessible(aura) then
            record.failures[#record.failures + 1] = { filter = filter, index = index, status = "restricted-aura" }
            return
        end
        if aura == nil then return end
        observeAura(aura, curves, filter, index, record)
        if index == 40 then record.scanLimitReached = true end
    end
end

local function clientInfo()
    local ok, version, build, date, interface = pcall(GetBuildInfo)
    if not ok then return { status = "build-error" } end
    return { version = scalar(version), build = scalar(build),
        buildDate = scalar(date), interface = scalar(interface) }
end

local function capture()
    local record = { client = clientInfo(), curves = definitions(), auras = {}, failures = {} }
    local okTime, now = pcall(time)
    if okTime then record.time = scalar(now) end
    if type(C_UnitAuras) ~= "table" or type(C_UnitAuras.GetAuraDataByIndex) ~= "function"
        or type(C_UnitAuras.GetAuraDispelTypeColor) ~= "function"
        or type(C_CurveUtil) ~= "table" or type(C_CurveUtil.CreateColorCurve) ~= "function"
        or type(CreateColor) ~= "function" or type(Enum) ~= "table"
        or type(Enum.LuaCurveType) ~= "table" or not accessible(Enum.LuaCurveType.Linear)
        or type(Enum.LuaCurveType.Linear) ~= "number"
        or type(issecretvalue) ~= "function" or type(canaccessvalue) ~= "function" then
        record.status = "missing-api"
        return record
    end
    local curves = {}
    for n, definition in ipairs(record.curves) do
        local ok, curve = pcall(makeCurve, definition)
        if not ok then record.status = "curve-construction-error"; return record end
        curves[n] = curve
    end
    scan(record, curves, "HELPFUL")
    scan(record, curves, "HARMFUL")
    record.status = "observed"
    return record
end

SLASH_AURADISPELCURVEPROBE1 = "/auradispelcurve"
SlashCmdList.AURADISPELCURVEPROBE = function()
    if AuraDispelCurveProbeDB == nil then
        AuraDispelCurveProbeDB = { schema = 1, captures = {}, dropped = 0 }
    end
    local db = AuraDispelCurveProbeDB
    if #db.captures >= 10 then db.dropped = db.dropped + 1; return end
    db.captures[#db.captures + 1] = capture()
end
