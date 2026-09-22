local output = io.stdout
local function mark(phase, value)
    output:write('BIGWIGS_TESTBAR\t' .. phase .. '\t' .. tostring(value) .. '\n')
    output:flush()
end

assert(SlashCmdList.BigWigs, 'BigWigs slash missing')()
assert(BigWigsOptions and BigWigsOptions:IsOpen(), 'BigWigs options did not open')
for _, name in ipairs({'BigWigs_Core', 'BigWigs_Plugins', 'BigWigs_Options'}) do
    local _, loaded = C_AddOns.IsAddOnLoaded(name)
    assert(loaded, name .. ' did not load')
end
mark('options-open', true)
local expectedLabel = BigWigsAPI:GetLocale('BigWigs: Common').count:format(
    BigWigsAPI:GetLocale('BigWigs').test, 1)

local function each_frame(visit)
    local frame = EnumerateFrames()
    while frame do
        if visit(frame) then return frame end
        frame = EnumerateFrames(frame)
    end
end

local function tree_button(value)
    return each_frame(function(frame)
        local widget = frame.obj
        return widget and widget.type == 'TreeGroup' and widget.buttons
            and frame.uniquevalue == value and frame:IsVisible()
    end)
end

local function test_button()
    return each_frame(function(frame)
        local widget = frame.obj
        if not (widget and widget.type == 'Button' and widget.frame == frame
            and frame:IsVisible() and frame:IsEnabled()) then return false end
        local user = widget:GetUserDataTable()
        local path = user.path
        return user.appName == 'BigWigs' and path and path[#path] == 'testButton'
    end)
end

local function is_running_bar(frame)
    return frame.running and frame.candyBarBar and frame.candyBarLabel
        and frame.updater and type(frame.exp) == 'number'
end

local function running_bars()
    local bars = {}
    each_frame(function(frame)
        if is_running_bar(frame) then bars[frame] = true end
    end)
    return bars
end

local observer = CreateFrame('Frame', nil, UIParent)
local started = GetTime()
local phase, expanded, previousBars = 'tree', false, nil
local bar, initialRemaining, initialValue, expiry
local lastScan = 0

local function observe()
    local now = GetTime()
    assert(now - started < 45, 'test bar did not expire within 45 seconds')
    if phase == 'progress' or phase == 'expiry' then
        if phase == 'progress' then
            if bar.remaining < initialRemaining - 0.1
                and math.abs(bar.candyBarBar:GetValue() - initialValue) > 0.001 then
                mark('progress', bar.remaining)
                phase = 'expiry'
            end
        elseif now > expiry then
            assert(not bar.running and not bar:IsShown(), 'test bar did not stop and hide')
            observer:SetScript('OnUpdate', nil)
            observer:Hide()
            mark('expired', true)
            mark('DONE', true)
        end
        return
    end

    assert(now - started < 10, 'test bar control did not appear within 10 seconds')
    if now - lastScan < 0.1 then return end
    lastScan = now

    if phase == 'tree' then
        local bars = tree_button('general\001Bars')
        if bars then
            bars:Click()
            mark('bars-selected', true)
            phase = 'control'
        elseif not expanded then
            local general = tree_button('general')
            if general and general.toggle and general.toggle:IsVisible() then
                general.toggle:Click()
                expanded = true
                mark('general-expanded', true)
            end
        end
    elseif phase == 'control' then
        local button = test_button()
        if button then
            previousBars = running_bars()
            button:Click()
            mark('test-button-clicked', true)
            phase = 'bar'
        end
    elseif phase == 'bar' then
        bar = each_frame(function(frame)
            return is_running_bar(frame) and not previousBars[frame]
                and frame:GetLabel() == expectedLabel
        end)
        if bar then
            local duration = bar.exp - bar.start
            assert(duration >= 11 - 0.001 and duration <= 30 + 0.001,
                'test bar duration outside 11..30 seconds')
            assert(bar:IsShown(), 'test bar did not show')
            assert(bar.candyBarLabel:GetText() == expectedLabel, 'test bar label mismatch')
            initialRemaining = bar.remaining
            initialValue = bar.candyBarBar:GetValue()
            assert(type(initialRemaining) == 'number' and initialRemaining > 0,
                'test bar did not start counting down')
            assert(type(initialValue) == 'number', 'test bar status value missing')
            expiry = bar.exp + 1
            mark('created-duration', duration)
            phase = 'progress'
        end
    end
end

observer:SetScript('OnUpdate', function()
    local ok, failure = pcall(observe)
    if not ok then
        observer:SetScript('OnUpdate', nil)
        observer:Hide()
        mark('FAILED', failure)
        error(failure)
    end
end)
