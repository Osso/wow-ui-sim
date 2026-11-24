local counter = CreateFrame('Frame', 'TextureSetHookProbeSecondCounter')
counter:SetSize(0, 0)

local target = assert(getmetatable(TextureHookProbeTarget).__index)
hooksecurefunc(target, 'SetTexture', function()
    local calls = counter:GetWidth() + 1
    counter:SetSize(calls, calls)
end)
