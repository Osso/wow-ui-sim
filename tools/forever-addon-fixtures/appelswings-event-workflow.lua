-- Observe actual addon frame ticks, not a timer that can fire before its first update.
local main = assert(AppelSwingsForevermhBar, 'main-hand bar missing')
local ranged = assert(AppelSwingsForeverrangedBar, 'ranged bar missing')
local output = io.stdout
local function mark(name, value)
    output:write('SWING_WORKFLOW\t' .. name .. '\t' .. tostring(value) .. '\n')
    output:flush()
end
A_Admin.FireEvent('PLAYER_SWING', 4.0, Enum.PlayerSwingType.MainHand)
A_Admin.FireEvent('PLAYER_SWING', 6.0, Enum.PlayerSwingType.Ranged)
assert(AppelSwingsForeverSwingTypes['0'] == 4.0)
assert(AppelSwingsForeverSwingTypes['2'] == 6.0)
local started = GetTime()
local firstMain, firstRanged
local progressed, mainExpired = false, false
local observer = CreateFrame('Frame', nil, UIParent)
local function observe()
    local elapsed = GetTime() - started
    assert(elapsed < 9, 'swing workflow did not complete within its observation window')
    if not firstMain then
        if not (main.fill:IsShown() and ranged.fill:IsShown()) then return end
        firstMain, firstRanged = main.fill:GetWidth(), ranged.fill:GetWidth()
        assert(firstMain > 0 and firstMain < main:GetWidth())
        assert(firstRanged > 0 and firstRanged < ranged:GetWidth())
        mark('initial-main', firstMain)
        mark('initial-ranged', firstRanged)
    elseif not progressed then
        if main.fill:GetWidth() > firstMain and ranged.fill:GetWidth() > firstRanged then
            progressed = true
            mark('progress', true)
        end
    elseif not mainExpired then
        if elapsed > 4.2 then
            assert(not main.fill:IsShown() and ranged.fill:IsShown(), 'independent swing expiry failed')
            mainExpired = true
            mark('main-expired-ranged-active', true)
        end
    elseif elapsed > 6.2 then
        assert(not main.fill:IsShown() and not ranged.fill:IsShown(), 'completed swing fills did not clear')
        assert(main:IsShown() and ranged:IsShown(), 'idle bar tracks should remain shown')
        observer:SetScript('OnUpdate', nil)
        observer:Hide()
        mark('DONE', true)
    end
end
observer:SetScript('OnUpdate', function()
    local ok, failure = pcall(observe)
    if not ok then
        observer:SetScript('OnUpdate', nil)
        observer:Hide()
        error(failure)
    end
end)
