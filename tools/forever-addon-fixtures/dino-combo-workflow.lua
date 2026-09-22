-- Real Dino event handlers and GUI unit-watch visibility; no direct addon updates.
assert(ShadowUF and ShadowUF.Units, "DinoUnitFrames did not initialize")
local output = io.stdout
local function mark(value)
    output:write("DINO_COMBO_WORKFLOW\t" .. tostring(value) .. "\n")
    output:flush()
end
A_Admin.SetPlayerPower(0, 5, 4)
A_Admin.SetTarget("Combo Test Target", 60, 1, true)
A_Admin.SetTargetHealth(1000, 1000)
A_Admin.FireEvent("PLAYER_TARGET_CHANGED")
local frame = assert(ShadowUF.Units.unitFrames.target, "target frame missing")
local points = assert(frame.comboPoints, "target combo frame missing")
local blocks = assert(points.blocks, "combo blocks missing")
local samples = {0, 2, 5, 1, 0}
local index = 1
local pending = false
local started = GetTime()
local observer = CreateFrame("Frame", nil, UIParent)
local function observe()
    assert(GetTime() - started < 8, "combo workflow did not complete")
    -- Target unit-watch visibility and OnShow initialization run on GUI updates.
    if not frame:IsVisible() or points.visibleBlocks ~= 5 then return end
    if not pending then
        A_Admin.SetPlayerPower(samples[index], 5, 4)
        A_Admin.FireEvent("UNIT_POWER_UPDATE", "player", "COMBO_POINTS")
        pending = true
        return
    end
    local count = samples[index]
    assert(points:IsShown(), "combo frame hidden")
    assert(points:GetValue() == 0, "parent fill must stay empty")
    for id = 1, 5 do
        local expected = id <= count and 1 or 0
        assert(blocks[id]:IsShown(), "combo block hidden: " .. id)
        assert(blocks[id]:GetAlpha() == expected,
            string.format("count=%d block=%d alpha=%s expected=%d", count, id, tostring(blocks[id]:GetAlpha()), expected))
    end
    -- Dino retains and shows old blocks; their curve-derived alpha must be zero.
    for id = 6, #blocks do assert(blocks[id]:GetAlpha() == 0, "excess combo block opaque") end
    mark(count)
    index = index + 1
    pending = false
    if index > #samples then
        observer:SetScript("OnUpdate", nil)
        observer:Hide()
        mark("DONE")
    end
end
observer:SetScript("OnUpdate", function()
    local ok, failure = pcall(observe)
    if not ok then
        observer:SetScript("OnUpdate", nil)
        observer:Hide()
        error(failure)
    end
end)
