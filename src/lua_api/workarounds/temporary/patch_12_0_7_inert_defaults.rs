//! Temporary inert defaults for remaining additive 12.0.7 API names.
//!
//! This bridge is now limited to the larger duration text-binding compatibility
//! object. Exact duration formatting and secret-value behavior remains
//! documented as paused until live client probes pin it down.

const PATCH_12_0_7_INERT_DEFAULTS_LUA: &str = r#"
if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120007 then
    local function ensure_namespace(name)
        _G[name] = _G[name] or __wow_namespace()
        return _G[name]
    end

    local function set_default(namespace, key, fn)
        if rawget(namespace, key) == nil then
            namespace[key] = fn
        end
    end

    local durationUtil = ensure_namespace("C_DurationUtil")
    local isPatch121 = type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100
    local function create_duration_clock(initialTime)
        local clock = { time = initialTime or 0 }
        function clock:GetTime() return self.time end
        function clock:SetTime(time) self.time = time or 0 end
        function clock:AdvanceTime(delta) self.time = self.time + (delta or 0) end
        function clock:RewindTime(delta) self.time = self.time - (delta or 0) end
        function clock:ResetTime() self.time = 0 end
        return clock
    end
    local function create_duration_value(initialTime)
        if type(durationUtil.CreateDuration) == "function" then
            local duration = durationUtil.CreateDuration()
            duration.value = initialTime or 0
            return duration
        end
        return { value = initialTime or 0 }
    end
    local function duration_value_to_text(duration)
        if type(duration) == "number" then
            return tostring(duration)
        end
        if type(duration) == "table" then
            if duration.value ~= nil then
                return tostring(duration.value)
            end
            if type(duration.GetRemainingDuration) == "function" then
                local ok, value = pcall(duration.GetRemainingDuration, duration)
                if ok and value ~= nil then return tostring(value) end
            end
        end
        return "0"
    end
    local function create_duration_text_binding(duration, fontString)
        local binding = {
            duration = duration ~= nil and duration or create_duration_value(0),
            fontString = fontString,
            enabled = true,
            updateInterval = 1,
            timeModifier = 0,
            expiredText = nil,
            zeroDurationText = nil,
            formatter = nil,
            textFormat = nil,
            textFormatComponents = nil,
            clock = create_duration_clock(0),
        }
        function binding:CanFormatText() return true end
        function binding:CanUpdateFontString() return self.fontString ~= nil and type(self.fontString.SetText) == "function" end
        function binding:Disable() self:SetEnabled(false) end
        function binding:Enable() self:SetEnabled(true) end
        function binding:GetClock() return self.clock end
        function binding:GetDuration() return self.duration end
        function binding:GetExpiredText() return self.expiredText end
        function binding:GetFontString() return self.fontString end
        function binding:GetFormattedText()
            local text = duration_value_to_text(self.duration)
            if type(self.formatter) == "userdata" and type(self.formatter.FormatNumber) == "function" then
                text = self.formatter:FormatNumber(tonumber(text))
            elseif type(self.formatter) == "function" then
                local ok, value = pcall(self.formatter, self.duration)
                if ok and value ~= nil then text = tostring(value) end
            elseif type(self.formatter) == "table" and type(self.formatter.Format) == "function" then
                local ok, value = pcall(self.formatter.Format, self.formatter, self.duration)
                if ok and value ~= nil then text = tostring(value) end
            end
            if type(self.textFormat) == "string" and self.textFormat ~= "" then
                local ok, value = pcall(string.format, self.textFormat, text)
                if ok then text = value end
            end
            return text
        end
        function binding:GetTimeModifier() return self.timeModifier end
        function binding:GetUpdateInterval() return self.updateInterval end
        function binding:GetZeroDurationText() return self.zeroDurationText end
        function binding:HasExpired() return type(self.duration) == "number" and self.duration <= 0 end
        function binding:HasSecretValues() return false end
        function binding:HasStarted() return true end
        function binding:IsActive() return self.enabled end
        function binding:IsEnabled() return self.enabled end
        function binding:SetClock(clock) self.clock = clock end
        function binding:SetDuration(value) self.duration = value ~= nil and value or create_duration_value(0) end
        function binding:SetEnabled(value) self.enabled = not not value end
        function binding:SetExpiredText(text) self.expiredText = text end
        function binding:SetFontString(value) self.fontString = value end
        function binding:SetFormatter(formatter) self.formatter = formatter end
        function binding:SetTextFormat(format, components)
            self.textFormat = format
            self.textFormatComponents = components
        end
        function binding:SetTimeModifier(value) self.timeModifier = value or 0 end
        function binding:SetToDefaults()
            self.duration = create_duration_value(0)
            self.enabled = true
            self.updateInterval = 1
            self.timeModifier = 0
            self.expiredText = nil
            self.zeroDurationText = nil
            self.formatter = nil
            self.textFormat = nil
            self.textFormatComponents = nil
            self.clock = create_duration_clock(0)
        end
        function binding:SetUpdateInterval(value) self.updateInterval = value or 1 end
        function binding:SetZeroDurationText(text) self.zeroDurationText = text end
        function binding:UpdateFontString()
            if self:CanUpdateFontString() then
                self.fontString:SetText(self:GetFormattedText())
            end
        end
        if isPatch121 then
            function binding:ClearTextColorCurve() self.textColorCurve = nil end
            function binding:GetFormattedTextColor() return 1, 1, 1, 1 end
            function binding:GetTextColorCurve() return self.textColorCurve end
            function binding:SetTextColorCurve(curve) self.textColorCurve = curve end
        end
        return binding
    end
    set_default(durationUtil, "CreateDurationTextBinding", create_duration_text_binding)
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PATCH_12_0_7_INERT_DEFAULTS_LUA)?;
    Ok(())
}
