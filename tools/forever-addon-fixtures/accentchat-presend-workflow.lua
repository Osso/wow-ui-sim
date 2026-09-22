local addon = LibStub("AceAddon-3.0"):GetAddon("AccentChat")
assert(addon.autoTranslateSupported, "pre-send registration missing")
addon.db.char.language = "Dwarf"
addon.db.char.strict = true
addon.db.char.flavorChance = 0
addon.db.char.channels.PARTY = true
addon:SetEnabled(nil, true)
local box = CreateFrame("EditBox", nil, UIParent)
box:SetMaxBytes(255)
local function check(label, input, channel, expected)
    box:SetAttribute("chatType", channel)
    box:SetText(input)
    EventRegistry:TriggerEvent("ChatFrame.OnEditBoxPreSendText", box)
    assert(box:GetText() == expected, label .. ": " .. tostring(box:GetText()))
    print("ACCENTCHAT_WORKFLOW", label, box:GetText())
end
check("party", "you are and the", "PARTY", "ye be an' tha")
check("party-leader", "you are and the", "PARTY_LEADER", "ye be an' tha")
check("slash", "/you are and the", "PARTY", "/you are and the")
addon.db.char.channels.YELL = false
check("disabled-channel", "you are and the", "YELL", "you are and the")
addon:SetEnabled(nil, false)
check("disabled-addon", "you are and the", "PARTY", "you are and the")
addon:SetEnabled(nil, true)
check("reenabled", "you are and the", "PARTY", "ye be an' tha")
print("ACCENTCHAT_WORKFLOW", "DONE")
