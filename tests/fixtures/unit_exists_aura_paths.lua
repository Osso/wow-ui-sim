-- Only raw registrations participate: namespace compatibility lookup is not proof.
local function registered(name)
    local callback = rawget(C_UnitAuras, name)
    assert(type(callback) == 'function', 'expected registered aura endpoint: ' .. name)
    return callback
end
local function fails(callback, text)
    local ok, message = pcall(callback)
    assert(not ok and string.find(tostring(message), text, 1, true), tostring(message))
end
local function aura_matches(aura, present, name)
    if present then
        assert(aura and aura.name == name, name)
    else
        assert(aura == nil, name .. ' leaked from missing unit')
    end
end
local curve = C_CurveUtil.CreateColorCurve()
curve:AddPoint(0, CreateColor(0.1, 0.2, 0.3, 1))
curve:AddPoint(1, CreateColor(0.4, 0.5, 0.6, 1))
local access_queries = {}
local invalid_queries = {}
local secret_queries = {}
function CheckStoredTargetAuras(present)
    assert(UnitExists('target') == present)
    for _, fixture in ipairs({{'HELPFUL', 'Fixture Blessing', 3}, {'HARMFUL', 'Expose Weakness', 2}}) do
        local filter, name, id = unpack(fixture)
        local expected = present and name or nil
        local tooltip_name = filter == 'HELPFUL' and 'GetUnitBuff' or 'GetUnitDebuff'
        local tooltip_query = rawget(C_TooltipInfo, tooltip_name)
        assert(type(tooltip_query) == 'function')
        local tooltip = tooltip_query('party1', 1)
        if present then
            assert(tooltip.lines[1].leftText == name)
        else
            assert(next(tooltip.lines) == nil)
        end
        assert(UnitAura('target', 1, filter) == expected)
        local legacy = filter == 'HELPFUL' and UnitBuff or UnitDebuff
        assert(legacy('target', 1) == expected)
        for _, query in ipairs({
            {'GetAuraDataByIndex', {'target', 1, filter}},
            {filter == 'HELPFUL' and 'GetBuffDataByIndex' or 'GetDebuffDataByIndex', {'target', 1}},
            {'GetAuraDataBySlot', {'target', id}},
            {'GetAuraDataByAuraInstanceID', {'target', id}},
            {'GetAuraDataBySpellName', {'target', name}},
            {'GetUnitAuraBySpellID', {'target', id == 2 and 53338 or 990003}},
        }) do
            aura_matches(registered(query[1])(unpack(query[2])), present, name)
        end
        local slots = {registered('GetAuraSlots')('target', filter)}
        assert(select('#', registered('GetAuraSlots')('target', filter)) == (present and 2 or 1))
        assert(slots[1] == nil and slots[2] == (present and id or nil))
        for _, ns in ipairs({C_UnitAuras, AuraUtil}) do
            local auras = rawget(ns, 'GetUnitAuras')('target', filter)
            assert(#auras == (present and 1 or 0))
            aura_matches(auras[1], present, name)
            aura_matches(rawget(ns, 'GetAuraDataByAuraInstanceID')('target', id), present, name)
        end
        local ids = registered('GetUnitAuraInstanceIDs')('target', filter)
        assert(#ids == (present and 1 or 0) and ids[1] == (present and id or nil))
        local visited = {}
        AuraUtil.ForEachAura('target', filter, nil, function(aura) visited[#visited + 1] = aura.name end)
        assert(#visited == (present and 1 or 0) and visited[1] == expected)
        aura_matches(AuraUtil.FindAura(function(aura) return aura.name == name end, 'target', filter), present, name)
        aura_matches(AuraUtil.FindAuraByName(name, 'target', filter), present, name)
    end
    assert(registered('GetAuraApplicationDisplayCount')('target', 2) == (present and '2' or ''))
    assert(registered('IsAuraFilteredOutByInstanceID')('target', 2, 'HARMFUL') == not present)
    assert(registered('GetAuraBaseDuration')('target', 2) == (present and 12 or nil))
    local refresh = registered('GetRefreshExtendedDuration')('target', 2)
    if present then assert(refresh >= 12 and refresh <= 15) else assert(refresh == nil) end
    assert(registered('DoesAuraHaveExpirationTime')('target', 2) == present)
    if present then
        assert(registered('GetAuraDuration')('target', 2):GetTotalDuration() == 12)
        local r, g, b = registered('GetAuraDispelTypeColor')('target', 2, curve):GetRGB()
        assert(math.abs(r - 0.4) < 0.0001 and math.abs(g - 0.5) < 0.0001 and math.abs(b - 0.6) < 0.0001)
        assert(registered('GetAuraCasterGUID')('target', 2) == UnitGUID('target'))
    else
        fails(function() registered('GetAuraDuration')('target', 2) end, 'no such aura instance')
        fails(function() registered('GetAuraDispelTypeColor')('target', 2, curve) end, 'aura instance not found')
        assert(registered('GetAuraCasterGUID')('target', 2) == nil)
    end
    -- Player-only endpoints cannot have a missing player: retain populated controls.
    assert(GetPlayerAuraBySpellID(990003).name == 'Fixture Blessing')
    assert(registered('GetPlayerAuraBySpellID')(990003).name == 'Fixture Blessing')
    for _, query in ipairs(invalid_queries) do fails(query[1], query[2]) end
    local function tainted_secrets()
        assert(debug.getstacktaint() == 'MissingUnitAuraProbe')
        for _, query in ipairs(secret_queries) do
            local ok = pcall(query)
            assert(not ok, 'secret argument accepted by tainted caller')
        end
        assert(issecretvalue(SecretAuraID))
    end
    debug.setobjecttaint(tainted_secrets, 'MissingUnitAuraProbe')
    tainted_secrets()
end
-- Each registered authenticated endpoint validates before missing-unit lookup.
for _, name in ipairs({'GetAuraDataByIndex', 'GetBuffDataByIndex', 'GetDebuffDataByIndex',
    'GetAuraDataBySlot', 'GetAuraDispelTypeColor', 'IsAuraFilteredOutByInstanceID',
    'GetAuraApplicationDisplayCount'}) do
    local callback = registered(name)
    invalid_queries[#invalid_queries + 1] = {function() callback('target', {}, curve) end, 'argument 2'}
    secret_queries[#secret_queries + 1] = function() callback('target', SecretAuraID, curve) end
end
for _, name in ipairs({'GetAuraBaseDuration', 'GetRefreshExtendedDuration', 'GetAuraDuration', 'DoesAuraHaveExpirationTime'}) do
    local callback = registered(name)
    invalid_queries[#invalid_queries + 1] = {function() callback('target', {}) end, 'aura instance'}
    if name ~= 'GetAuraBaseDuration' and name ~= 'GetRefreshExtendedDuration' then
        secret_queries[#secret_queries + 1] = function() callback('target', SecretAuraID) end
    end
end
invalid_queries[#invalid_queries + 1] = {function() registered('GetUnitAuraBySpellID')('target', {}) end, 'argument 2'}
secret_queries[#secret_queries + 1] = function() registered('GetUnitAuraBySpellID')('target', SecretAuraID) end
invalid_queries[#invalid_queries + 1] = {function() registered('GetAuraSlots')('target', 'HARMFUL', {}) end, 'argument 3'}
invalid_queries[#invalid_queries + 1] = {function() registered('GetUnitAuraInstanceIDs')('target', 'HARMFUL', -1) end, 'maxCount must be non-negative'}
secret_queries[#secret_queries + 1] = function() registered('GetAuraSlots')('target', 'HARMFUL', SecretAuraID) end
-- These endpoints require aura access before parsing or existence lookup.
for _, name in ipairs({'GetAuraDataByIndex', 'GetBuffDataByIndex', 'GetDebuffDataByIndex',
    'GetAuraDataBySlot', 'GetAuraDataByAuraInstanceID', 'GetUnitAuras', 'GetUnitAuraInstanceIDs',
    'GetAuraSlots', 'GetAuraApplicationDisplayCount', 'IsAuraFilteredOutByInstanceID',
    'GetAuraBaseDuration', 'GetRefreshExtendedDuration', 'GetAuraDuration',
    'DoesAuraHaveExpirationTime', 'GetAuraDispelTypeColor'}) do
    access_queries[#access_queries + 1] = registered(name)
end
for _, name in ipairs({'GetUnitBuff', 'GetUnitDebuff'}) do
    local callback = rawget(C_TooltipInfo, name)
    invalid_queries[#invalid_queries + 1] = {function() callback('party1', {}) end, 'argument 2'}
    secret_queries[#secret_queries + 1] = function() callback('party1', SecretAuraID) end
    access_queries[#access_queries + 1] = callback
end
function CheckRestrictedTargetAccess()
    local function probe()
        for _, callback in ipairs(access_queries) do
            fails(function() callback('target', {}, {}) end,
                "Auras cannot be accessed when secret while tainted by 'MissingUnitAuraProbe' Lua Taint: MissingUnitAuraProbe")
        end
    end
    debug.setobjecttaint(probe, 'MissingUnitAuraProbe')
    probe()
end
