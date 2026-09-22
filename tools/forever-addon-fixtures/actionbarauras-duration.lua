local playerContainer, targetContainer
for _, child in ipairs({ActionButton1:GetChildren()}) do
    if child:GetName() == 'ActionButton1ABAContainer' then
        if child:GetUnit() == 'player' then playerContainer = child end
        if child:GetUnit() == 'target' then targetContainer = child end
    end
end
assert(playerContainer and targetContainer and playerContainer ~= targetContainer, 'separate addon containers missing')
local slot = assert(playerContainer:GetAuraSlotFrame('ABA'))
local text = assert(slot:GetDurationText())
local output = io.stdout
local function mark(name, value)
    output:write('ABA_ACCEPTANCE\t' .. name .. '\t' .. tostring(value) .. '\n')
    output:flush()
end
mark('containers', true)
A_Admin.AddBuff(19750, 'Flash of Light', 135944, 8, 1)
C_Timer.After(0.4, function()
    local first = text:GetText()
    mark('initial-text', first)
    assert(slot:IsShown() and first and first:find('s', 1, true), 'timed aura text did not appear')
    C_Timer.After(1.2, function()
        local second = text:GetText()
        mark('advanced-text', second)
        assert(slot:IsShown() and second and second ~= first, 'duration text did not advance')
        A_Admin.RemoveBuff(19750)
        C_Timer.After(0.4, function()
            mark('removed-visible', slot:IsShown())
            assert(not slot:IsShown(), 'aura button remained shown after removal')
            assert(playerContainer:GetParent() == ActionButton1 and targetContainer:GetParent() == ActionButton1)
            mark('DONE', true)
        end)
    end)
end)
