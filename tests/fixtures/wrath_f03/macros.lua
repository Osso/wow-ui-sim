-- INFERRED global child names from native Blizzard_MacroUI.lua/xml.
ShowMacroFrame()
assert(MacroFrame:IsShown(), "macro frame visible")
assert(PanelTemplates_GetSelectedTab(MacroFrame) == 1, "account tab selected")
local account, character = GetNumMacros()
assert(account == 2 and character == 1, "supplied macro counts")
assert(MacroButton1:IsEnabled() and MacroButton2:IsEnabled(), "populated macro buttons enabled")
assert(not MacroButton3:IsEnabled(), "empty macro slot disabled")
assert(MacroButton1Name:GetText() == "Raid Beacon", "first macro name")
assert(MacroFrameSelectedMacroName:GetText() == "Raid Beacon", "selected macro name")
assert(MacroFrameText:GetText() == "/rw Stack on star", "selected macro body")
-- Wrath Blizzard_MacroUI.lua:79,89 assigns the selected macro texture.
assert(MacroFrameSelectedMacroButtonIcon:GetTexture():lower() == "interface\\icons\\inv_misc_questionmark", "selected macro icon")
