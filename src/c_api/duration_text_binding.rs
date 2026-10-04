//! Userdata duration text binding handles exposed by C_DurationUtil.
//!
//! Configuration copying and automatic engine-tick updates are modeled here.
//! Tick cadence and first-update policy are inferred, not native-verified.

use crate::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};
use crate::lua_api::globals::lua_duration_object::duration_has_secret_values;
use crate::lua_api::methods::{call_function, registry_get, registry_set, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

mod state;

const SCHEDULER_KEY: &str = "__duration_text_binding_scheduler";

const DURATION_TEXT_BINDING_LUA: &str = r#"
do
    local isPatch121, hasSecretInput, readSecretInput, wrapSecretOutput, reportUpdateError = ...
    local isPatch1207, createSettings, readSetting, writeSetting, authenticateArguments, validateDuration, sampleRemaining, durationIsZero, durationHasExpired = select(6, ...)
    local scalarFields = {enabled = true, updateInterval = true, timeModifier = true}
    local unpackArguments = unpack
    -- Capture host bootstrap functions, not later addon replacements.
    local secretType, secretToString, secretToNumber, secretStringFormat = type, tostring, tonumber, string.format
    local function ensure_namespace(name)
        _G[name] = _G[name] or __wow_namespace()
        return _G[name]
    end

    local durationUtil = ensure_namespace("C_DurationUtil")
    local function create_duration_clock(initialTime)
        return durationUtil.CreateManualClock(initialTime)
    end
    local function create_duration_value(initialTime)
        local duration = durationUtil.CreateDuration()
        duration.value = initialTime or 0
        return duration
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
    local function secret_duration_text(duration)
        duration = readSecretInput(duration)
        local value = duration
        if secretType(duration) ~= "number" then value = duration:GetRemainingDuration() end
        return secretToString(value)
    end
    local function format_secret_duration(binding, duration)
        local text = secret_duration_text(duration)
        local formatter = binding.formatter
        local value
        if secretType(formatter) == "userdata" and secretType(formatter.FormatNumber) == "function" then
            value = formatter:FormatNumber(wrapSecretOutput(secretToNumber(text)))
        elseif secretType(formatter) == "function" then
            value = formatter(duration)
        elseif secretType(formatter) == "table" and secretType(formatter.Format) == "function" then
            value = formatter:Format(duration)
        else
            value = text
        end
        if value == nil then error("Secret duration formatter returned nil", 3) end
        text = secretToString(readSecretInput(value))
        if secretType(binding.textFormat) == "string" and binding.textFormat ~= "" then
            text = secretStringFormat(binding.textFormat, text)
        end
        return text
    end
    local bindings = setmetatable({}, { __mode = "k" })
    local nextBinding, protectedCall = next, pcall
    local configurationFields = {
        "duration", "fontString", "enabled", "updateInterval", "timeModifier",
        "expiredText", "zeroDurationText", "formatter", "textFormat", "clock",
        "textColorCurve", "textColorProperty",
    }
    local scheduledFields = {textFormatComponents = true}
    for _, field in ipairs(configurationFields) do scheduledFields[field] = true end
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
            -- INFERRED: nil default configuration follows later reset declaration.
            duration = duration,
            fontString = fontString,
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
        local schedule = {dirty = true, elapsed = 0}
        local settings = createSettings()
        if not isPatch1207 and duration == nil then configuration.duration = create_duration_value(0) end
        metatable.__index = function(_, key)
            if scalarFields[key] then return readSetting(settings, key) end
            return configuration[key]
        end
        metatable.__newindex = function(_, key, value)
            if scalarFields[key] then
                writeSetting(settings, key, value)
            else
                configuration[key] = value
            end
            if scheduledFields[key] then schedule.dirty = true end
        end
        bindings[binding] = schedule
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
        local function has_text(value)
            return hasSecretInput(value) or value ~= nil
        end
        local function protect_text(text, secret)
            -- INFERRED: source timing secrecy also protects zero/expired text selection.
            if secret and not hasSecretInput(text) then return wrapSecretOutput(text) end
            return text
        end
        local function can_format(binding)
            -- INFERRED: fallback text suffices for unconfigured/zero/expired state;
            -- a nonzero active duration requires a configured NumericFormatter.
            local duration = binding.duration
            if duration == nil then return has_text(binding.zeroDurationText) end
            if hasSecretInput(duration) then duration = readSecretInput(duration) end
            if durationIsZero(duration) then return has_text(binding.zeroDurationText) end
            if durationHasExpired(duration) and has_text(binding.expiredText) then return true end
            return type(binding.formatter) == "userdata" and type(binding.formatter.FormatNumber) == "function"
        end
        function binding:CanFormatText()
            if isPatch1207 then return can_format(self) end
            return true
        end
        function binding:CanUpdateFontString()
            return self:CanFormatText() and self.fontString ~= nil and type(self.fontString.SetText) == "function"
        end
        function binding:Disable() self:SetEnabled(false) end
        function binding:Enable() self:SetEnabled(true) end
        function binding:GetClock() return self.clock end
        function binding:GetDuration() return self.duration end
        function binding:GetExpiredText() return self.expiredText end
        function binding:GetFontString() return self.fontString end
        function binding:GetFormattedText()
            if isPatch1207 then
                if not can_format(self) then error("DurationTextBinding is not configured for formatting", 2) end
                local duration = self.duration
                if duration == nil then return self.zeroDurationText end
                local secret = hasSecretInput(duration)
                if secret then duration = readSecretInput(duration) end
                -- INFERRED: zero-duration text takes precedence over expiration text.
                if durationIsZero(duration) then return protect_text(self.zeroDurationText, secret) end
                if durationHasExpired(duration) and has_text(self.expiredText) then return protect_text(self.expiredText, secret) end
                local value = sampleRemaining(settings, duration)
                -- INFERRED: sampled secret timing is handed to the formatter as a VM secret.
                if secret then value = wrapSecretOutput(value) end
                local text = self.formatter:FormatNumber(value)
                local decodedText = text
                if hasSecretInput(text) then decodedText = readSecretInput(text) end
                if type(decodedText) ~= "string" then error("DurationTextBinding formatter must return string", 2) end
                if self.textFormat ~= nil then text = secretStringFormat(self.textFormat, text) end
                return protect_text(text, secret)
            end
            local duration = self.duration
            if hasSecretInput(duration) then return format_secret_duration(self, duration) end
            local text = duration_value_to_text(duration)
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
        function binding:HasSecretValues() return hasSecretInput(self.duration) end
        function binding:HasStarted() return true end
        function binding:IsActive() return self.enabled end
        function binding:IsEnabled() return self.enabled end
        function binding:SetClock(clock) self.clock = clock end
        function binding:SetDuration(value)
            if isPatch1207 then validateDuration(value) end
            self.duration = value ~= nil and value or create_duration_value(0)
        end
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
            -- INFERRED: cleared default fields are not authenticated for build 68182.
            self.duration = nil
            if not isPatch1207 then self.duration = create_duration_value(0) end
            if isPatch1207 then self.fontString = nil end
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
                local secret = hasSecretInput(self.duration)
                local text = self:GetFormattedText()
                if secret and not hasSecretInput(text) then text = wrapSecretOutput(text) end
                self.fontString:SetText(text)
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
        if isPatch1207 then
            -- INFERRED: strict noncoercing validation; scalar/reference wrappers normalize,
            -- while text wrappers remain opaque. Native coercion/identity policy is unproven.
            local setters = {"SetDuration", "SetFontString", "SetExpiredText", "SetZeroDurationText",
                "SetTimeModifier", "SetUpdateInterval", "SetEnabled", "SetFormatter", "SetClock", "Assign"}
            for _, name in ipairs(setters) do
                local original = binding[name]
                binding[name] = function(self, ...)
                    local count = 1 + select('#', ...)
                    local decoded = {authenticateArguments(self, ...)}
                    require_binding(decoded[1])
                    local value = decoded[2]
                    if name == "SetExpiredText" or name == "SetZeroDurationText" then
                        if value ~= nil and type(value) ~= "string" then error("text must be string or nil", 2) end
                        -- Keep VM secret wrappers rooted; do not publish decoded secret text.
                        if value ~= nil then decoded[2] = select(1, ...) end
                    elseif name == "SetTimeModifier" then
                        if value ~= 0 and value ~= 1 then error("DurationTimeModifier expected", 2) end
                    elseif name == "SetUpdateInterval" then
                        if type(value) ~= "number" or value ~= value or value < 0 or value == math.huge then
                            error("finite nonnegative update interval expected", 2)
                        end
                    elseif name == "SetEnabled" then
                        if type(value) ~= "boolean" then error("enabled must be boolean", 2) end
                    elseif name == "SetFontString" then
                        if value == nil or type(value.GetObjectType) ~= "function" or value:GetObjectType() ~= "FontString" then
                            error("FontString expected", 2)
                        end
                    elseif name == "SetFormatter" then
                        if type(value) ~= "userdata" or type(value.FormatNumber) ~= "function" then error("NumericFormatter expected", 2) end
                    end
                    return original(unpackArguments(decoded, 1, count))
                end
            end
        end
        return binding
    end
    durationUtil.CreateDurationTextBinding = create_duration_text_binding

    -- This closure is retained only in the host registry, not a Lua global.
    -- Cadence uses engine elapsed time; duration objects retain their own clocks.
    local function update_binding(binding, schedule, elapsed)
        if binding:IsEnabled() and binding:CanUpdateFontString() then
            schedule.elapsed = schedule.elapsed + elapsed
            if schedule.dirty or schedule.elapsed >= binding:GetUpdateInterval() then
                schedule.dirty = false
                schedule.elapsed = 0
                binding:UpdateFontString()
            end
        end
    end
    return function(elapsed)
        for binding, schedule in nextBinding, bindings do
            local ok, errorValue = protectedCall(update_binding, binding, schedule, elapsed)
            if not ok then reportUpdateError(errorValue) end
        end
    end
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
    let state = lua.state_mut();
    let arguments = [
        Val::Bool(modern_methods),
        secret_callback(state, "DurationBinding.HasSecretInput", has_secret_input),
        secret_callback(state, "DurationBinding.ReadSecretInput", read_secret_input),
        secret_callback(
            state,
            "DurationBinding.WrapSecretOutput",
            wrap_secret_output,
        ),
        secret_callback(
            state,
            "DurationBinding.ReportUpdateError",
            report_update_error,
        ),
        Val::Bool(cfg!(feature = "retail-12-0-7")),
        secret_callback(state, "DurationBinding.CreateSettings", self::state::create),
        secret_callback(state, "DurationBinding.ReadSetting", self::state::read),
        secret_callback(state, "DurationBinding.WriteSetting", self::state::write),
        secret_callback(
            state,
            "DurationBinding.Authenticate",
            self::state::authenticate,
        ),
        secret_callback(
            state,
            "DurationBinding.ValidateDuration",
            self::state::validate_duration,
        ),
        secret_callback(
            state,
            "DurationBinding.SampleRemaining",
            self::state::sample_remaining,
        ),
        secret_callback(state, "DurationBinding.IsZero", self::state::is_zero),
        secret_callback(
            state,
            "DurationBinding.HasExpired",
            self::state::has_expired,
        ),
    ];
    let callback = lua.call_function(&bootstrap, &arguments)?;
    let callback = callback.into_iter().next().ok_or_else(|| {
        rilua::runtime_error("duration binding bootstrap did not return its scheduler")
    })?;
    registry_set(lua.state_mut(), SCHEDULER_KEY, callback);
    Ok(())
}

/// Run after frame OnUpdate assignment, retaining callback taint and secret checks.
pub(crate) fn tick(lua: &mut rilua::Lua, elapsed: f64) -> crate::Result<()> {
    let callback = registry_get(lua.state_mut(), SCHEDULER_KEY);
    if callback != Val::Nil {
        call_function(lua, callback, &[Val::Num(elapsed)])?;
    }
    Ok(())
}

fn report_update_error(state: &mut LuaState) -> LuaResult<u32> {
    let message = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| {
        "duration text binding update failed with a non-string error".to_owned()
    });
    crate::lua_api::script_helpers::call_error_handler_state(state, &message);
    Ok(0)
}

fn secret_callback(state: &mut LuaState, name: &'static str, function: rilua::RustFn) -> Val {
    use rilua::vm::closure::{Closure, RustClosure};
    Val::Function(
        state
            .gc
            .alloc_closure(Closure::Rust(RustClosure::new(function, name))),
    )
}

fn has_secret_input(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let secret = rilua::table_security::is_secret_value(state, value)
        || duration_has_secret_values(state, value);
    state.push(Val::Bool(secret));
    Ok(1)
}

fn read_secret_input(state: &mut LuaState) -> LuaResult<u32> {
    // Guess: binding text from a secret duration is readable only by untainted callers.
    if !rilua::api::state_is_secure(state) {
        return Err(rilua::runtime_error(
            "secret duration text requires an untainted caller",
        ));
    }
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    state.push(value);
    Ok(1)
}

fn wrap_secret_output(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::wrap_secret(state, stack_val(state, 1))?;
    state.push(value);
    Ok(1)
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
                    local duration = C_DurationUtil.CreateDuration()
                    duration:SetClock(C_DurationUtil.CreateManualClock(0))
                    duration:SetTimeFromStart(0, 12)
                    binding:SetDuration(duration)
                    binding:SetFontString(label)
                    local formatter = C_StringUtil.CreateSecondsFormatter()
                    formatter:SetDefaultAbbreviation(Enum.SecondsFormatterAbbreviation.OneLetter)
                    formatter:SetStripIntervalWhitespace(Enum.SecondsFormatterIntervalWhitespace.Strip)
                    binding:SetFormatter(formatter)
                    binding:UpdateFontString()
                    assert(label:GetText() == '12s')
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
