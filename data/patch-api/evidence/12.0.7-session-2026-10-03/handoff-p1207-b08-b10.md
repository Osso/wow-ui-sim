# Retail 12.0.7 B08–B10 audit handoff

Status: authored, not integrated; 2026-10-04.

Active goal: audit pending B08/B09/B10 and duration-binding-grouped B29/B30; deliver row evidence, exact anchored edits, staged public-API tests and spec. Writes restricted to this cache. No repository/git mutation, cargo, builds, tests, simulator, agents or model CLIs. Verification limited to static source reading and unique edit-anchor checks; runtime proof reserved for integrator.

Existing provider must remain sole provider; preserve Copy/Assign/SetClock and host-owned duration clocks. No alternate provider or fallback.

## Progress
- [x] Constraints recorded before detailed investigation.
- [x] Pending row selection and quoted evidence.
- [x] Existing-provider anchored edits.
- [x] Staged tests and spec.
- [x] Static checks and final limitations.

## Selected row evidence (checkpoint)

Master anchor: `78afc8825086676ef37ed0753b354eb5714e9070`. 21 pending rows selected; none skipped in this slice. B29/B30 explicitly group with binding storage/formatting. Later cache is not historical build 68182 proof.

### `prose-undated-022`
Source L22: `- Added DurationTextBinding script object type.`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationUtilDocumentation.lua:21`:
```lua
			Name = "CreateDurationTextBinding",
			Type = "Function",
			Documentation = { "Creates a duration text binding, which automatically updates a font string with formatted text derived from a duration object." },

			Returns =
			{
				{ Name = "binding", Type = "DurationTextBinding", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:118`. Must default to unconfigured nil duration/font string; retain sole userdata provider, methods and scheduler. Scalar settings become host-owned, not public Lua table values. Factory arguments are undocumented extensions: keep compatibility, no historical credit.

### `global api-C_DurationUtil-CreateDurationTextBinding-031`
Source L31: `C_DurationUtil.CreateDurationTextBinding`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationUtilDocumentation.lua:21`:
```lua
			Name = "CreateDurationTextBinding",
			Type = "Function",
			Documentation = { "Creates a duration text binding, which automatically updates a font string with formatted text derived from a duration object." },

			Returns =
			{
				{ Name = "binding", Type = "DurationTextBinding", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:118`. Must default to unconfigured nil duration/font string; retain sole userdata provider, methods and scheduler. Scalar settings become host-owned, not public Lua table values. Factory arguments are undocumented extensions: keep compatibility, no historical credit.

### `scriptobjects-DurationObject-HasExpired-089`
Source L89: `DurationObject:HasExpired`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:335`:
```lua
			Name = "HasExpired",
			Type = "Function",
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Returns true once the duration has reached its end time." },

			Arguments =
			{
				{ Name = "modifier", Type = "DurationTimeModifier", Nilable = false, Default = "RealTime" },
			},

			Returns =
			{
				{ Name = "hasExpired", Type = "bool", Nilable = false },
			},
		},
		{
```
Today: `src/lua_api/globals/lua_duration_object/core.rs:340–347` returns `cfg!(feature = "retail-12-0-5")` for zero span. Source only lists addition; cache says reached end time, does not specify zero-span exception. Existing 12.0.5 fully-elapsed policy is inherited. Correct bounded fix: startup test expects true on 12.0.7; do not reverse inherited producer. Native zero-span semantics remain INFERRED.

### `scriptobjects-DurationTextBinding-CanFormatText-093`
Source L93: `DurationTextBinding:CanFormatText`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:22`:
```lua
			Name = "CanFormatText",
			Type = "Function",
			Documentation = { "Returns true if this binding has enough configuration to produce formatted text." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "canFormatText", Type = "bool", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:155`. Must replace unconditional/simplified formatting with live Rust duration sampling, configured zero/expired text and configuration eligibility; preserve sole provider.

### `scriptobjects-DurationTextBinding-CanUpdateFontString-094`
Source L94: `DurationTextBinding:CanUpdateFontString`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:36`:
```lua
			Name = "CanUpdateFontString",
			Type = "Function",
			Documentation = { "Returns true if this binding has enough configuration to update its font string with formatted text." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "canUpdateText", Type = "bool", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:156`. Must replace unconditional/simplified formatting with live Rust duration sampling, configured zero/expired text and configuration eligibility; preserve sole provider.

### `scriptobjects-DurationTextBinding-Disable-095`
Source L95: `DurationTextBinding:Disable`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:74`:
```lua
			Name = "Disable",
			Type = "Function",
			Documentation = { "Disables automatic updates for this duration text binding." },

			Arguments =
			{
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:157`. Must prove exact arity, identity/state defaults, live scalar getters and scheduling behavior; getter eligibility must include formatting configuration.

### `scriptobjects-DurationTextBinding-Enable-096`
Source L96: `DurationTextBinding:Enable`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:83`:
```lua
			Name = "Enable",
			Type = "Function",
			Documentation = { "Enables automatic updates for this duration text binding." },

			Arguments =
			{
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:158`. Must prove exact arity, identity/state defaults, live scalar getters and scheduling behavior; getter eligibility must include formatting configuration.

### `scriptobjects-DurationTextBinding-GetDuration-097`
Source L97: `DurationTextBinding:GetDuration`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:92`:
```lua
			Name = "GetDuration",
			Type = "Function",
			Documentation = { "Returns the duration object used by this duration text binding." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "duration", Type = "LuaDurationObject", Nilable = true },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:160`. Must prove exact arity, identity/state defaults, live scalar getters and scheduling behavior; getter eligibility must include formatting configuration.

### `scriptobjects-DurationTextBinding-GetExpiredText-098`
Source L98: `DurationTextBinding:GetExpiredText`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:106`:
```lua
			Name = "GetExpiredText",
			Type = "Function",
			Documentation = { "Returns the text shown when the duration has fully expired." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "text", Type = "string", Nilable = true },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:161`. Must prove exact arity, identity/state defaults, live scalar getters and scheduling behavior; getter eligibility must include formatting configuration.

### `scriptobjects-DurationTextBinding-GetFontString-099`
Source L99: `DurationTextBinding:GetFontString`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:120`:
```lua
			Name = "GetFontString",
			Type = "Function",
			Documentation = { "Returns the font string updated by this duration text binding." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "fontString", Type = "SimpleFontString", Nilable = true },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:162`. Must prove exact arity, identity/state defaults, live scalar getters and scheduling behavior; getter eligibility must include formatting configuration.

### `scriptobjects-DurationTextBinding-GetFormattedText-100`
Source L100: `DurationTextBinding:GetFormattedText`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:134`:
```lua
			Name = "GetFormattedText",
			Type = "Function",
			Documentation = { "Returns the text that would currently be assigned to the configured font string." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "text", Type = "string", Nilable = false, ConditionalSecret = true },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:163`. Must replace unconditional/simplified formatting with live Rust duration sampling, configured zero/expired text and configuration eligibility; preserve sole provider.

### `scriptobjects-DurationTextBinding-GetTimeModifier-101`
Source L101: `DurationTextBinding:GetTimeModifier`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:179`:
```lua
			Name = "GetTimeModifier",
			Type = "Function",
			Documentation = { "Returns the time modifier used when sampling duration values for this binding." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "modifier", Type = "DurationTimeModifier", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:182`. Must prove exact arity, identity/state defaults, live scalar getters and scheduling behavior; getter eligibility must include formatting configuration.

### `scriptobjects-DurationTextBinding-GetUpdateInterval-102`
Source L102: `DurationTextBinding:GetUpdateInterval`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:193`:
```lua
			Name = "GetUpdateInterval",
			Type = "Function",
			Documentation = { "Returns the minimum number of seconds between automatic text updates. A value of zero updates every game tick." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "updateInterval", Type = "number", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:183`. Must prove exact arity, identity/state defaults, live scalar getters and scheduling behavior; getter eligibility must include formatting configuration.

### `scriptobjects-DurationTextBinding-GetZeroDurationText-103`
Source L103: `DurationTextBinding:GetZeroDurationText`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:207`:
```lua
			Name = "GetZeroDurationText",
			Type = "Function",
			Documentation = { "Returns the text shown when the duration is not configured, or represents a zero-duration time span." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "text", Type = "string", Nilable = true },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:184`. Must prove exact arity, identity/state defaults, live scalar getters and scheduling behavior; getter eligibility must include formatting configuration.

### `scriptobjects-DurationTextBinding-IsEnabled-104`
Source L104: `DurationTextBinding:IsEnabled`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:236`:
```lua
			Name = "IsEnabled",
			Type = "Function",
			Documentation = { "Returns true if this duration text binding updates its font string automatically." },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "enabled", Type = "bool", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:189`. Must prove exact arity, identity/state defaults, live scalar getters and scheduling behavior; getter eligibility must include formatting configuration.

### `scriptobjects-DurationTextBinding-SetDuration-105`
Source L105: `DurationTextBinding:SetDuration`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:250`:
```lua
			Name = "SetDuration",
			Type = "Function",
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Configures the duration object used by this duration text binding." },

			Arguments =
			{
				{ Name = "duration", Type = "LuaDurationObject", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:191`. Must authenticate ALL supplied arguments/extras before receiver/type validation; strict declared input types, atomic rejection and host-owned scalar mutation where applicable.

### `scriptobjects-DurationTextBinding-SetExpiredText-106`
Source L106: `DurationTextBinding:SetExpiredText`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:272`:
```lua
			Name = "SetExpiredText",
			Type = "Function",
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Configures the text shown when the duration has fully expired." },

			Arguments =
			{
				{ Name = "text", Type = "string", Nilable = true },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:193`. Must authenticate ALL supplied arguments/extras before receiver/type validation; strict declared input types, atomic rejection and host-owned scalar mutation where applicable.

### `scriptobjects-DurationTextBinding-SetFontString-107`
Source L107: `DurationTextBinding:SetFontString`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:283`:
```lua
			Name = "SetFontString",
			Type = "Function",
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Configures the font string updated by this duration text binding." },

			Arguments =
			{
				{ Name = "fontString", Type = "SimpleFontString", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:194`. Must authenticate ALL supplied arguments/extras before receiver/type validation; strict declared input types, atomic rejection and host-owned scalar mutation where applicable.

### `scriptobjects-DurationTextBinding-SetTimeModifier-108`
Source L108: `DurationTextBinding:SetTimeModifier`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:329`:
```lua
			Name = "SetTimeModifier",
			Type = "Function",
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Configures the time modifier used when sampling duration values for this binding." },

			Arguments =
			{
				{ Name = "modifier", Type = "DurationTimeModifier", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:200`. Must authenticate ALL supplied arguments/extras before receiver/type validation; strict declared input types, atomic rejection and host-owned scalar mutation where applicable.

### `scriptobjects-DurationTextBinding-SetUpdateInterval-109`
Source L109: `DurationTextBinding:SetUpdateInterval`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:349`:
```lua
			Name = "SetUpdateInterval",
			Type = "Function",
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Configures the minimum number of seconds between automatic text updates. A value of zero updates every game tick." },

			Arguments =
			{
				{ Name = "updateInterval", Type = "number", Nilable = false },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:213`. Must authenticate ALL supplied arguments/extras before receiver/type validation; strict declared input types, atomic rejection and host-owned scalar mutation where applicable.

### `scriptobjects-DurationTextBinding-SetZeroDurationText-110`
Source L110: `DurationTextBinding:SetZeroDurationText`
Cached declaration `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:360`:
```lua
			Name = "SetZeroDurationText",
			Type = "Function",
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Configures the text shown when the duration is not configured, or represents a zero-duration time span." },

			Arguments =
			{
				{ Name = "text", Type = "string", Nilable = true },
			},
		},
		{
```
Today: `src/c_api/duration_text_binding.rs:214`. Must authenticate ALL supplied arguments/extras before receiver/type validation; strict declared input types, atomic rejection and host-owned scalar mutation where applicable.

## Implementation checkpoint
Existing Lua userdata provider and weak scheduler retained. Staged `src/c_api/duration_text_binding/state.rs` introduces only host-owned scalar settings (enabled, interval, modifier) and live Rust sampling of the existing duration object, not a second binding provider. Lua references remain VM-rooted in existing configuration closures; no Rust-held untraced `Val` roots. Binding formatting/default/input changes are gated by `retail-12-0-7`. Exact defaults and eligibility/failure rules remain INFERRED pending historical/native probes.

## Anchored edits against master

Apply all state/test-support first for RED; withhold producer edits. New helper file is state/test-support infrastructure; its `sample_remaining` is unused until producers apply. Each OLD matches once in captured master bytes. Full staged snapshots are conveniences, not RED-phase replacements.

### E01 [state] `src/c_api/duration_text_binding.rs`
OLD:
```text
const SCHEDULER_KEY: &str = "__duration_text_binding_scheduler";
```
NEW:
```text
mod state;

const SCHEDULER_KEY: &str = "__duration_text_binding_scheduler";
```

### E02 [state] `src/c_api/duration_text_binding.rs`
OLD:
```text
    local isPatch121, hasSecretInput, readSecretInput, wrapSecretOutput, reportUpdateError = ...
```
NEW:
```text
    local isPatch121, hasSecretInput, readSecretInput, wrapSecretOutput, reportUpdateError = ...
    local isPatch1207, createSettings, readSetting, writeSetting, authenticateArguments, validateDuration, sampleRemaining = select(6, ...)
    local scalarFields = {enabled = true, updateInterval = true, timeModifier = true}
    local unpackArguments = unpack
```

### E03 [producer] `src/c_api/duration_text_binding.rs`
OLD:
```text
    local function set_default(namespace, key, fn)
        if rawget(namespace, key) == nil then
            namespace[key] = fn
        end
    end


```
NEW:
```text

```

### E04 [producer] `src/c_api/duration_text_binding.rs`
OLD:
```text
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

```
NEW:
```text
    local function create_duration_clock(initialTime)
        return durationUtil.CreateManualClock(initialTime)
    end
    local function create_duration_value(initialTime)
        local duration = durationUtil.CreateDuration()
        duration.value = initialTime or 0
        return duration
    end

```

### E05 [producer] `src/c_api/duration_text_binding.rs`
OLD:
```text
            duration = duration ~= nil and duration or create_duration_value(0),
            fontString = fontString,
            enabled = true,
            updateInterval = 1,
            timeModifier = 0,
```
NEW:
```text
            -- INFERRED: nil default configuration follows later reset declaration.
            duration = duration,
            fontString = fontString,
```

### E06 [state] `src/c_api/duration_text_binding.rs`
OLD:
```text
        metatable.__index = configuration
        metatable.__newindex = function(_, key, value)
            configuration[key] = value
            if scheduledFields[key] then schedule.dirty = true end
        end
```
NEW:
```text
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
```

### E07 [producer] `src/c_api/duration_text_binding.rs`
OLD:
```text
        function binding:CanFormatText() return true end
        function binding:CanUpdateFontString() return self.fontString ~= nil and type(self.fontString.SetText) == "function" end
```
NEW:
```text
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
            if duration:IsZero() then return has_text(binding.zeroDurationText) end
            if duration:HasExpired() and has_text(binding.expiredText) then return true end
            return type(binding.formatter) == "userdata" and type(binding.formatter.FormatNumber) == "function"
        end
        function binding:CanFormatText()
            if isPatch1207 then return can_format(self) end
            return true
        end
        function binding:CanUpdateFontString()
            return self:CanFormatText() and self.fontString ~= nil and type(self.fontString.SetText) == "function"
        end
```

### E08 [producer] `src/c_api/duration_text_binding.rs`
OLD:
```text
        function binding:GetFormattedText()
            local duration = self.duration
```
NEW:
```text
        function binding:GetFormattedText()
            if isPatch1207 then
                if not can_format(self) then error("DurationTextBinding is not configured for formatting", 2) end
                local duration = self.duration
                if duration == nil then return self.zeroDurationText end
                local secret = hasSecretInput(duration)
                if secret then duration = readSecretInput(duration) end
                -- INFERRED: zero-duration text takes precedence over expiration text.
                if duration:IsZero() then return protect_text(self.zeroDurationText, secret) end
                if duration:HasExpired() and has_text(self.expiredText) then return protect_text(self.expiredText, secret) end
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
```

### E09 [producer] `src/c_api/duration_text_binding.rs`
OLD:
```text
        function binding:SetDuration(value) self.duration = value ~= nil and value or create_duration_value(0) end
```
NEW:
```text
        function binding:SetDuration(value)
            if isPatch1207 then validateDuration(value) end
            self.duration = value ~= nil and value or create_duration_value(0)
        end
```

### E10 [producer] `src/c_api/duration_text_binding.rs`
OLD:
```text
        function binding:SetToDefaults()
            self.duration = create_duration_value(0)
```
NEW:
```text
        function binding:SetToDefaults()
            -- INFERRED: cleared default fields are not authenticated for build 68182.
            self.duration = nil
            if not isPatch1207 then self.duration = create_duration_value(0) end
            if isPatch1207 then self.fontString = nil end
```

### E11 [producer] `src/c_api/duration_text_binding.rs`
OLD:
```text
                if secret then text = wrapSecretOutput(text) end
                self.fontString:SetText(text)
```
NEW:
```text
                if secret and not hasSecretInput(text) then text = wrapSecretOutput(text) end
                self.fontString:SetText(text)
```

### E12 [producer] `src/c_api/duration_text_binding.rs`
OLD:
```text
        return binding
    end
    set_default(durationUtil, "CreateDurationTextBinding", create_duration_text_binding)
```
NEW:
```text
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
```

### E13 [state] `src/c_api/duration_text_binding.rs`
OLD:
```text
            report_update_error,
        ),
    ];
```
NEW:
```text
            report_update_error,
        ),
        Val::Bool(cfg!(feature = "retail-12-0-7")),
        secret_callback(state, "DurationBinding.CreateSettings", self::state::create),
        secret_callback(state, "DurationBinding.ReadSetting", self::state::read),
        secret_callback(state, "DurationBinding.WriteSetting", self::state::write),
        secret_callback(state, "DurationBinding.Authenticate", self::state::authenticate),
        secret_callback(state, "DurationBinding.ValidateDuration", self::state::validate_duration),
        secret_callback(state, "DurationBinding.SampleRemaining", self::state::sample_remaining),
    ];
```

### E14 [test-support] `src/lua_api/globals/lua_duration_object.rs`
OLD:
```text
fn require_duration(state: &mut LuaState, index: i32) -> LuaResult<Val> {
```
NEW:
```text
pub(crate) fn require_duration(state: &mut LuaState, index: i32) -> LuaResult<Val> {
```

### E15 [producer] `src/lua_api/globals/lua_duration_object/core.rs`
OLD:
```text
    if matches!(kind, Query::Started | Query::Active) {
        authenticate_activity_modifier(state)?;
    }
```
NEW:
```text
    if matches!(kind, Query::Started | Query::Active)
        || (cfg!(feature = "retail-12-0-7") && matches!(kind, Query::Expired))
    {
        authenticate_activity_modifier(state)?;
    }
```

### E16 [test-support] `src/loader/tests/wow_api_globals/startup_globals.rs`
OLD:
```text
if durationObject:HasExpired() ~= false then return "DurationObject.HasExpired" end
```
NEW:
```text
if durationObject:HasExpired() ~= true then return "DurationObject.HasExpired" end
```

### E17 [test-support] `src/loader/tests/wow_api_globals/startup_globals.rs`
OLD:
```text
            if binding:GetDuration() == nil then return "DurationTextBinding.GetDuration default" end
            if binding:CanFormatText() ~= true then return "DurationTextBinding.CanFormatText" end
```
NEW:
```text
            if binding:GetDuration() ~= nil then return "DurationTextBinding.GetDuration default" end
            if binding:CanFormatText() ~= false then return "DurationTextBinding.CanFormatText" end
```

### E18 [test-support] `src/loader/tests/wow_api_globals/startup_globals.rs`
OLD:
```text
            if binding:GetFormattedText() ~= "0" then return "DurationTextBinding.GetFormattedText default" end
```
NEW:
```text
            if pcall(binding.GetFormattedText, binding) then return "DurationTextBinding.GetFormattedText unconfigured" end
```

### E19 [test-support] `src/loader/tests/wow_api_globals/startup_globals.rs`
OLD:
```text
            binding:SetDuration(10)
            if binding:GetDuration() ~= 10 then return "DurationTextBinding.SetDuration" end
            if binding:GetFormattedText() ~= "10" then return "DurationTextBinding.GetFormattedText" end
```
NEW:
```text
            durationObject:SetTimeFromStart(0, 10)
            binding:SetDuration(durationObject)
            local formatter = C_StringUtil.CreateSecondsFormatter()
            formatter:SetDefaultAbbreviation(Enum.SecondsFormatterAbbreviation.OneLetter)
            formatter:SetStripIntervalWhitespace(Enum.SecondsFormatterIntervalWhitespace.Strip)
            binding:SetFormatter(formatter)
            if binding:GetDuration() ~= durationObject then return "DurationTextBinding.SetDuration" end
            if binding:GetFormattedText() ~= "10s" then return "DurationTextBinding.GetFormattedText" end
```

### E20 [test-support] `src/loader/tests/wow_api_globals/startup_globals.rs`
OLD:
```text
            binding:SetTimeModifier(1.5)
            if binding:GetTimeModifier() ~= 1.5 then return "DurationTextBinding.SetTimeModifier" end
```
NEW:
```text
            binding:SetTimeModifier(1)
            if binding:GetTimeModifier() ~= 1 then return "DurationTextBinding.SetTimeModifier" end
```

### E21 [test-support] `src/loader/tests/wow_api_globals/startup_globals.rs`
OLD:
```text
            binding:SetDuration(7)
            binding:UpdateFontString()
            if fontString:GetText() ~= "7" then return "DurationTextBinding.UpdateFontString" end
```
NEW:
```text
            durationObject:SetTimeFromStart(0, 7)
            binding:UpdateFontString()
            if fontString:GetText() ~= "7s" then return "DurationTextBinding.UpdateFontString" end
```

RED partition clarification: E01/E02/E06/E13 + E14 visibility are scaffolding/state/test-support. Apply every state/test-support edit and new helper/test/spec files, withholding all producer-tagged edits. Scalar state exists but old default/formatter/setter behavior remains, so new tests fail on behavior rather than missing callback plumbing. Do not copy full producer snapshots for RED.

## Cached consumer (later cache, not 68182 proof)

`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:167–190`:
```lua
	local binding = self:GetDurationTextBinding();

	if options.binding then
		binding:Assign(options.binding);
	else
		binding:SetToDefaults();
		binding:SetFormatter(addonTable.DefaultAuraDurationFormatter);
	end

	binding:SetFontString(fontString);
	binding:SetDuration(self:GetAuraDuration());

	if options.textFormat then
		binding:SetTextFormat(options.textFormat.formatString, options.textFormat.components);
	elseif options.textFormatter then
		binding:SetFormatter(options.textFormatter);
	end

	if options.textColor then
		binding:SetTextColorCurve(options.textColor.curve, options.textColor.property);
	end

	-- Intentionally not preserving options for now; the options here are used
	-- as a convenience to build a binding. We could export them if this changes.
```

`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:312–318`:
```lua
	-- Retain the duration text binding across reconfiguration; replacing it
	-- would require explicitly disabling the previous active binding.
	self.durationTextBinding = C_DurationUtil.CreateDurationTextBinding();
end

function CustomAuraButtonPrivateMixin:OnUpdate(_elapsedTime)
	self:ApplyPandemicRegions();
```

### E22 [test-support] `src/c_api/duration_text_binding.rs`
OLD:
```text
                    binding:SetDuration(12)
                    binding:SetFontString(label)
                    binding:SetFormatter({Format=function(_, value) return 'value:' .. value end})
                    binding:UpdateFontString()
                    assert(label:GetText() == 'value:12')
```
NEW:
```text
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
```

## HasExpired adjudication

`data/patch-api/sources/12.0.5-api-changes.txt:26`: "Duration objects that measure a zero-span are now considered fully elapsed." `Cargo.toml:120`: `retail-12-0-7 = ["retail-12-0-5"]`. Combined with the 12.0.7 source addition and current "reached its end time" declaration, the bounded existing contract supports correcting the test to true (E16), not changing core zero-span behavior. Mapping fully-elapsed to expired remains INFERRED; native zero-span behavior cannot be definitively determined from these extracts. E15 separately fixes the cached AllowedWhenUntainted modifier/extras boundary.

## Row-to-edit and proof map

| Rows | Edits | New behavioral tests (all UNRUN) |
|---|---|---|
| 022 / 031 | E01–E06 / E10 / E12–E14 | defaults, roundtrips, isolation |
| 093 / 094 / 100 | E07 / E08 / E11 | defaults, live clock/rate, secret timing |
| 095 / 096 / 104 | E02 / E06 / E12 / E13 | automatic updates, defaults |
| 097 / 098 / 099 / 103 | E05–E10 / E12 | defaults/arity, roundtrips, opaque text |
| 101 / 102 | E01 / E02 / E06 / E13 | roundtrips, isolation, live modifier/cadence |
| 105 / 106 / 107 / 108 / 109 / 110 | E09 / E12–E14 | invalid-input atomicity, all-input authentication |
| 089 | E15 / E16 | HasExpired inherited zero-span/modifier/extras |

## Existing tests requiring changes

1. `startup_globals::test_patch_12_0_7_duration_objects_and_text_binding`: E16–E21 correct zero-span true, nil/default eligibility, configured duration+real NumericFormatter, valid modifier enum and actual formatted FontString text.
2. `duration_text_binding::tests::duration_binding_availability_preserves_client_versions`: E22 replaces numeric pseudo-duration/table formatter fixture with a real duration/manual clock/SecondsFormatter; availability assertions remain intact.
3. Existing `tests/patch_12_0_7_duration_clocks.rs`, `tests/duration_text_binding_copy.rs`, `tests/duration_text_binding_tick.rs`: unchanged; clock tests are 12.0.7 proof and copy/tick tests are Forever-gated. Integrator must run relevant retained coverage, not assume it passes.

## Exclusions and blocked claims

- No historical build 68182 declaration/native execution: exact default text, rounding, eligibility/coercion, output-secret tagging and zero-span native semantics stay INFERRED, not accepted whole-row native parity.
- B31 formatting options/components and later color methods: separate rows; existing methods remain installed. Current percent-format extension is retained; cached `{}` component semantics are not claimed.
- FontString/NumericFormatter validation currently uses public protocol/type information, not unforgeable native handle identity. Factory optional constructor arguments are retained simulator extensions, not declared 12.0.7 inputs. Binding-level SetClock identity/configuration remains existing behavior; formatting samples the duration object's clock.
- Earlier-profile behavioral proof, exact secret branch tagging, weak-registry collection and exhaustive scheduler-error routing are not established by these new cases. Shared host scalar storage should not be promoted as native evidence.
- Cargo, builds, tests, simulator, native probes, agents/model CLIs, integration and ledger promotion were expressly excluded. No passing RED/GREEN or compilation claim.

## Integration risk

Author-only proposal. Highest risk is uncompiled host callback API usage and inferred native formatting/default policies, followed by regressions to older profile extensions. Do not merge as verified. Keep producer-tagged edits withheld for RED, integrate them for GREEN only after the new test-support scaffolding is in place, and qualify any eventual row promotion to the actually exercised bounded behavior.

## Final static proof ledger — 2026-10-04

| Action | Exact scope | Result | Limits |
|---|---|---|---|
| Read-only `git rev-parse master` and `git show master:<path>` | Master `78afc8825086676ef37ed0753b354eb5714e9070`; four anchored existing files | Working bytes equal master; every OLD matches exactly once | No execution/compilation proof |
| Static anchor replay after final edit | E01–E22 and four staged existing-file snapshots | 22 unique, nonoverlapping anchors; full snapshots exactly equal replay | No tests run |
| Static coverage inventory | Current 12.0.7 page ledger, selected source IDs | 21 still audit-pending; 20 binding rows plus HasExpired089 | No row status/capability updates |
| `rustfmt --edition 2024` on two staged new Rust artifacts | New helper and new nine-test integration module | Exit 0 after final relevant changes | Formatter parsing only, not type checking or test execution |
| Manual Rust readability review | New host helper, changed Rust registration/core/visibility and test fixtures | Short host functions, bounded tests, no warning suppressions; version-policy conditions named | Inline Lua retains existing provider structure; no independent review |

Final inventory: **21 audited pending rows, 22 anchored edits (4 state / 10 producer / 8 test-support), 9 new behavioral tests, 7 staged files**. No repository/git state written; no cargo/build/test/simulator/agent/model CLI execution. Staging is an authoring deliverable only.

- `docs/specs/duration-text-binding-12-0-7-audit.md` — SHA-256 `5a16e371b3df86a8e39dc2bbb890b43e739ea8b754a4c7aed40f8c264d46ff55`
- `src/c_api/duration_text_binding/state.rs` — SHA-256 `e01f801364bc4171fe2e039cc7b94d1f45166521a9dc563fb2ac6d0cc1b1fce9`
- `src/c_api/duration_text_binding.rs` — SHA-256 `add040291024c5c36ee524bdff8dbf01d14617cef4ca7452133987e594493f2a`
- `src/loader/tests/wow_api_globals/startup_globals.rs` — SHA-256 `15d5151667a4947088ad7581ecea2a69c4b6a0d72dc86935013d01f2b48b7e2b`
- `src/lua_api/globals/lua_duration_object/core.rs` — SHA-256 `34895f8ccdc2ec6a4ed934035a401af16e759550a22f7571a82e344f20fd2d91`
- `src/lua_api/globals/lua_duration_object.rs` — SHA-256 `91b7c19142b56b26c04613607b1e487a257e7ffe52a2cc8fc48d649d8a0ca068`
- `tests/patch_12_0_7_duration_text_binding.rs` — SHA-256 `f819d537395e022f2c8aa915b9efb82afc3737fc4ce303bb892c6a17b679049f`
