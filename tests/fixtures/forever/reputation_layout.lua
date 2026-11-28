-- INFERRED geometry from Forever 70707 Blizzard_UIPanels_Game/Camelot:
-- CharacterFrameConstants.lua:4-7, CharacterFrame.lua:3-29,205-258,
-- CharacterFrame.xml:407-412, ReputationFrame.xml:197-249,259-300,
-- ReputationFrame.lua:77-90.
ToggleCharacter("ReputationFrame")
assert(CharacterFrame:IsShown() and ReputationFrame:IsShown())
assert(not PaperDollFrame:IsShown(), "paper doll hidden")
assert(CharacterFrame.activeSubframe == "ReputationFrame")
assert(CharacterFrame.selectedTab == CHARACTER_FRAME_TAB.Reputation, "selected mode tab")
assert(CharacterFrame:GetTab(CHARACTER_FRAME_TAB.Reputation):GetChecked(), "reputation tab checked")
assert(not CharacterFrame:GetTab(CHARACTER_FRAME_TAB.Character):GetChecked(), "character tab unchecked")

local function near(actual, expected, name)
    assert(actual and math.abs(actual - expected) < 0.75,
        name .. ": expected " .. tostring(expected) .. ", got " .. tostring(actual))
end

local left, bottom, width, height = CharacterFrame:GetRect()
local expectedWidth = CharacterFrame:IsRightPaneCollapsed() and CHARACTER_FRAME_COLLAPSED_WIDTH or CHARACTER_FRAME_WIDTH
near(width, expectedWidth, "character width")
near(height, CHARACTER_FRAME_HEIGHT, "character height")
local top = bottom + height

local repLeft, repBottom, repWidth, repHeight = ReputationFrame:GetRect()
near(repLeft, left, "reputation left")
near(repBottom, bottom, "reputation bottom")
near(repWidth, width, "reputation width")
near(repHeight, height, "reputation height")

local host = CharacterFrameLeftPaneHost
near(host:GetWidth(), 398, "left pane width")
near(host:GetLeft(), left, "left pane left")
near(host:GetTop(), top - 20, "left pane top")
near(host:GetBottom(), bottom, "left pane bottom")

assert(not ReputationFrame.filterDropdown:IsShown(), "Camelot hides the filter dropdown")

local box = ReputationFrame.ScrollBox
assert(box:IsShown(), "scroll box shown")
near(box:GetLeft(), host:GetLeft() + 10, "scroll box left")
near(box:GetTop(), host:GetTop() - 40, "scroll box top")
near(box:GetRight(), host:GetRight() - 25, "scroll box right")
near(box:GetBottom(), host:GetBottom() + 15, "scroll box bottom")

local bar = ReputationFrame.ScrollBar
near(bar:GetLeft(), box:GetRight() + 5, "scroll bar left")
near(bar:GetTop(), box:GetTop() - 2, "scroll bar top")
near(bar:GetBottom(), box:GetBottom() + 4, "scroll bar bottom")

local detail = ReputationFrame.ReputationDetailFrame
local atWar, inactive, watch = detail.AtWarCheckbox, detail.MakeInactiveCheckbox, detail.WatchFactionCheckbox
for name, box26 in pairs({ atWar = atWar, inactive = inactive, watch = watch }) do
    near(box26:GetWidth(), 26, name .. " width")
    near(box26:GetHeight(), 26, name .. " height")
end
near(inactive:GetLeft(), atWar:GetLeft(), "inactive left")
near(inactive:GetTop(), atWar:GetBottom() - 2, "inactive top")
near(watch:GetLeft(), inactive:GetLeft(), "watch left")
near(watch:GetTop(), inactive:GetBottom() - 2, "watch top")
