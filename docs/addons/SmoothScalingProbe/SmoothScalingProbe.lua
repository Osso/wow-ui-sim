local function readSample(fs)
    local path, size, flags = fs:GetFont()
    return {
        smooth = fs:GetSmoothScaling(), height = fs:GetStringHeight(),
        autoHeight = fs:GetHeight(), lines = fs:GetNumLines(),
        scale = fs:GetScale(), effectiveScale = fs:GetEffectiveScale(),
        textScale = fs:GetTextScale(), font = path, fontSize = size, fontFlags = flags,
        animationMode = fs:GetScaleAnimationMode(),
    }
end

local panel
local function capture()
    if panel then panel:Hide() end
    panel = CreateFrame("Frame", nil, UIParent)
    panel:SetSize(900, 650)
    panel:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 30, -30)
    local background = panel:CreateTexture(nil, "BACKGROUND")
    background:SetAllPoints()
    background:SetColorTexture(0, 0, 0, .9)
    local db = {
        build = {GetBuildInfo()}, physicalSize = {GetPhysicalScreenSize()},
        uiScale = UIParent:GetEffectiveScale(), samples = {}, errors = {},
    }
    SmoothScalingProbeDB = db
    local ok, result = pcall(function()
        return {
            omitted = readSample(SmoothScalingProbeXML.Default),
            smooth = readSample(SmoothScalingProbeXML.Smooth),
            snapped = readSample(SmoothScalingProbeXML.Snapped),
        }
    end)
    if ok then db.xml = result else db.errors.xml = tostring(result) end
    local cases = {
        {name = "frame 1.1 single", scale = 1.1, text = "H"},
        {name = "frame 1.1 newline", scale = 1.1, text = "H\nH"},
        {name = "frame 1.1 wrapped", scale = 1.1, width = 70, text = "Hello Hello Hello Hello"},
        {name = "text 1.1 newline", textScale = 1.1, text = "H\nH"},
        {name = "font 13.2 newline", fontSize = 13.2, text = "H\nH"},
    }
    for index, case in ipairs(cases) do
        local label = panel:CreateFontString(nil, "OVERLAY", "GameFontNormal")
        label:SetPoint("TOPLEFT", panel, "TOPLEFT", 10, -20 - (index-1)*110)
        label:SetText(case.name .. " | false / true / false")
        db.samples[index] = {name = case.name, modes = {}}
        for column = 1, 3 do
            local success, sample = pcall(function()
                local fs = panel:CreateFontString(nil, "OVERLAY")
                fs:SetFont("Fonts\\FRIZQT__.TTF", case.fontSize or 12)
                fs:SetPoint("TOPLEFT", panel, "TOPLEFT", 240 + (column-1)*180, -20-(index-1)*110)
                fs:SetScale(case.scale or 1)
                fs:SetTextScale(case.textScale or 1)
                if case.width then fs:SetWidth(case.width) end
                local default = fs:GetSmoothScaling()
                fs:SetText(case.text)
                local flips = {}
                for _, mode in ipairs({false, true, false}) do
                    fs:SetSmoothScaling(mode)
                    flips[#flips+1] = readSample(fs)
                end
                fs:SetSmoothScaling(column == 2)
                local before = fs:GetSmoothScaling()
                fs:SetScaleAnimationMode(Enum.FontStringScaleAnimationMode.Vertex)
                local after = fs:GetSmoothScaling()
                fs:SetScaleAnimationMode(Enum.FontStringScaleAnimationMode.FontSize)
                return {default = default, flips = flips, visible = readSample(fs),
                    smoothBeforeAnimationMode = before, smoothAfterAnimationMode = after}
            end)
            if success then db.samples[index].modes[column] = sample
            else db.errors[case.name .. ":" .. column] = tostring(sample) end
        end
    end
    print("SmoothScalingProbe recorded; capture screenshot, then reload/logout to save.")
end

SLASH_SMOOTHSCALINGPROBE1 = "/smoothscalingprobe"
SlashCmdList.SMOOTHSCALINGPROBE = capture
