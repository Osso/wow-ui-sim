local output = io.stdout
local function mark(phase)
    output:write('DATAMINE_UI\t' .. phase .. '\n')
    output:flush()
end

local _, loaded = C_AddOns.IsAddOnLoaded('Datamine')
assert(loaded, 'Datamine not fully loaded')
assert(DatamineUnifiedFrame and not DatamineUnifiedFrame:IsShown(), 'expected initially hidden UI')
assert(type(SlashCmdList.DMINE) == 'function', 'Datamine slash registration missing')
local editBox = assert(ChatFrame1EditBox, 'chat edit box missing')
local function send()
    editBox:SetText('/dm ui')
    editBox:SendText(1)
end

local observer = CreateFrame('Frame', nil, UIParent)
local started = GetTime()
local phase = 'opening'
send()
mark('open-dispatched')
observer:SetScript('OnUpdate', function()
    local ok, failure = pcall(function()
        assert(GetTime() - started < 10, 'Datamine toggle timed out')
        if phase == 'opening' then
            if DatamineUnifiedFrame.ShowAnim:IsPlaying() then return end
            assert(DatamineUnifiedFrame:IsShown(), 'Datamine UI did not open')
            mark('opened')
            phase = 'closing'
            send()
            mark('close-dispatched')
        elseif phase == 'closing' then
            if DatamineUnifiedFrame.ShowAnim:IsPlaying() then return end
            assert(not DatamineUnifiedFrame:IsShown(), 'Datamine UI did not close')
            observer:SetScript('OnUpdate', nil)
            observer:Hide()
            mark('closed')
            mark('DONE')
        end
    end)
    if not ok then
        observer:SetScript('OnUpdate', nil)
        observer:Hide()
        mark('FAILED: ' .. tostring(failure))
        error(failure)
    end
end)
