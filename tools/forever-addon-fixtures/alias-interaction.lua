-- Cached Alias8260596: real Add/Remove buttons, slash expansion, and chat output.
local function check(label, condition)
    assert(condition, label)
    print('ALIAS_CHECK', label, 'PASS')
end
local function descendant(frame, predicate)
    for _, child in ipairs({frame:GetChildren()}) do
        if predicate(child) then return child end
        local found = descendant(child, predicate)
        if found then return found end
    end
end
SlashCmdList.ALIASMANAGER('')
local ui = assert(AliasFrame)
check('manager opens', ui:IsShown())
local aliasInput = assert(descendant(ui, function(f) return f:GetObjectType() == 'EditBox' and f:GetWidth() == 120 end))
local commandInput = assert(descendant(ui, function(f) return f:GetObjectType() == 'EditBox' and f:GetWidth() == 180 end))
local add = assert(descendant(ui, function(f) return f:GetObjectType() == 'Button' and f:GetText() == 'Add' end))
aliasInput:SetText('ghi')
commandInput:SetText('say hello alias')
add:Click()
check('add button normalizes and stores alias', AliasDB['/ghi'] == '/say hello alias')
check('input fields clear', aliasInput:GetText() == '' and commandInput:GetText() == '')
check('slash published', _G.SLASH_CUSTOM_ALIAS_GHI1 == '/ghi' and type(SlashCmdList.CUSTOM_ALIAS_GHI) == 'function')
SlashCmdList.CUSTOM_ALIAS_GHI('raid')
local delivered = false
for i = 1, ChatFrame1:GetNumMessages() do
    local text = ChatFrame1:GetMessageInfo(i)
    if text and text:find('hello alias raid', 1, true) then delivered = true end
end
check('expanded message reaches chat output', delivered)
local remove = assert(descendant(ui, function(f) return f:GetObjectType() == 'Button' and f:GetText() == 'Remove' end))
remove:Click()
check('remove button clears alias', AliasDB['/ghi'] == nil)
print('ALIAS_REMOVE_VALUES', tostring(_G.SLASH_CUSTOM_ALIAS_GHI1), tostring(SlashCmdList.CUSTOM_ALIAS_GHI), tostring(hash_SlashCmdList['/GHI']), tostring(rawget(SlashCmdList, 'CUSTOM_ALIAS_GHI')), tostring(rawget(hash_SlashCmdList, '/GHI')))
check('slash removed', _G.SLASH_CUSTOM_ALIAS_GHI1 == nil and SlashCmdList.CUSTOM_ALIAS_GHI == nil and hash_SlashCmdList['/GHI'] == nil)
SlashCmdList.ALIASMANAGER('')
check('manager closes', not ui:IsShown())
print('ALIAS_INTERACTION_PASS')
