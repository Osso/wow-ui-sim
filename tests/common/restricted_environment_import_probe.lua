local _, helpfulSpell = C_Item.GetItemSpell(5512)
local _, harmfulSpell = C_Item.GetItemSpell(6948)
A_Admin.SetSpellTargeting(helpfulSpell, "helpful")
A_Admin.SetSpellTargeting(harmfulSpell, "harmful")
A_Admin.SetActionSlot(8001, 123456)
local header = CreateFrame("Frame", nil, UIParent, "SecureHandlerBaseTemplate")
SecureHandlerExecute(header, [[
    self:SetAttribute("helpful", IsHelpfulItem(5512))
    self:SetAttribute("harmful", IsHarmfulItem(6948))
    self:SetAttribute("not-helpful", IsHelpfulItem(6948))
    self:SetAttribute("not-harmful", IsHarmfulItem(5512))
    self:SetAttribute("has-action", HasAction(8001))
    self:SetAttribute("no-action", HasAction(8002))
]])
return header:GetAttribute("helpful"), header:GetAttribute("harmful"),
    header:GetAttribute("not-helpful"), header:GetAttribute("not-harmful"),
    header:GetAttribute("has-action"), header:GetAttribute("no-action")
