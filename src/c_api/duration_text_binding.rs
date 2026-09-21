//! Userdata duration text binding handles exposed by C_DurationUtil.
//!
//! Configuration copying is modeled here. Existing formatting, clock, and
//! update behavior is retained; this does not establish native timing or
//! secret-value parity.

use crate::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};
use rilua::LuaApiMut;

const DURATION_TEXT_BINDING_LUA: &str = r#"
do
    local isPatch121 = ...
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
    local bindings = setmetatable({}, { __mode = "k" })
    local configurationFields = {
        "duration", "fontString", "enabled", "updateInterval", "timeModifier",
        "expiredText", "zeroDurationText", "formatter", "textFormat", "clock",
        "textColorCurve", "textColorProperty",
    }
    local function require_binding(value)
        if type(value) ~= "userdata" or not bindings[value] then
            error("DurationTextBinding expected", 3)
        end
    end
    local function copy_component(component)
        if type(component) ~= "table" then return component end
        local copy = {}
        for key, value in pairs(component) do copy[key] = value end
        return copy
    end
    local function copy_components(components)
        if type(components) ~= "table" then return components end
        local copy = {}
        for key, component in pairs(components) do
            copy[key] = copy_component(component)
        end
        return copy
    end
    local function create_duration_text_binding(duration, fontString)
        local configuration = {
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
        -- Native handles survive securecopy(options); configuration tables do not.
        local binding = newproxy(true)
        local metatable = getmetatable(binding)
        metatable.__index = configuration
        metatable.__newindex = configuration
        bindings[binding] = true
        function binding:Assign(other)
            require_binding(self)
            require_binding(other)
            local components = copy_components(other.textFormatComponents)
            for _, field in ipairs(configurationFields) do self[field] = other[field] end
            self.textFormatComponents = components
        end
        function binding:Copy()
            require_binding(self)
            local copy = create_duration_text_binding()
            copy:Assign(self)
            return copy
        end
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
            function binding:ClearTextColorCurve()
                self.textColorCurve = nil
                self.textColorProperty = nil
            end
            function binding:GetFormattedTextColor() return 1, 1, 1, 1 end
            function binding:GetTextColorCurve() return self.textColorCurve, self.textColorProperty end
            function binding:SetTextColorCurve(curve, property)
                self.textColorCurve = curve
                self.textColorProperty = property
            end
        end
        return binding
    end
    set_default(durationUtil, "CreateDurationTextBinding", create_duration_text_binding)
end
"#;

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    let modern_methods = match ACTIVE {
        ClientProfile::WowForever => true,
        ClientProfile::Retail | ClientProfile::Ptr if ACTIVE_INTERFACE_VERSION >= 120007 => {
            ACTIVE_INTERFACE_VERSION >= 120100
        }
        _ => return Ok(()),
    };
    let bootstrap = lua.load_bytes(
        DURATION_TEXT_BINDING_LUA.as_bytes(),
        "@duration-text-binding-bootstrap",
    )?;
    lua.call_function(&bootstrap, &[rilua::Val::Bool(modern_methods)])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn duration_binding_availability_preserves_client_versions() {
        let env = WowLuaEnv::new().unwrap();
        let (interface, binding, modern): (i32, bool, bool) = env
            .eval(
                r#"
                local factory = C_DurationUtil and C_DurationUtil.CreateDurationTextBinding
                local binding = type(factory) == 'function' and factory() or nil
                if binding then
                    local label = CreateFrame('Frame'):CreateFontString()
                    binding:SetDuration(12)
                    binding:SetFontString(label)
                    binding:SetFormatter({Format=function(_, value) return 'value:' .. value end})
                    binding:UpdateFontString()
                    assert(label:GetText() == 'value:12')
                    assert(binding:Copy():GetFontString() == label)
                end
                return select(4, GetBuildInfo()), binding ~= nil,
                    binding ~= nil and type(binding.SetTextColorCurve) == 'function'
                "#,
            )
            .unwrap();
        assert_eq!(binding, interface == 16001 || interface >= 120007);
        assert_eq!(modern, interface == 16001 || interface >= 120100);
    }
}
