//! Bounded native handle and duration-consumer contract, not native-client parity.
#![cfg(all(feature = "native-duration-formatting", feature = "aura-containers"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const CONFIGURE: &str = r#"
    function NewActionSecondsFormatter()
        local formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetDefaultAbbreviation(Enum.SecondsFormatterAbbreviation.OneLetter)
        formatter:SetMinInterval(Enum.SecondsFormatterInterval.Seconds)
        formatter:SetDesiredUnitCount(1)
        formatter:SetMillisecondsThreshold(3)
        formatter:SetStripIntervalWhitespace(Enum.SecondsFormatterIntervalWhitespace.Strip)
        return formatter
    end
"#;

fn configured_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(CONFIGURE).unwrap();
    env
}

#[test]
fn seconds_formatter_native_handle_survives_securecopy_and_aura_validation() {
    let env = configured_env();
    env.exec(
        r#"
        local formatter = NewActionSecondsFormatter()
        assert(type(formatter) == 'userdata' and getmetatable(formatter) == false)
        assert(not pcall(rawset, formatter, 'FormatNumber', function() return 'forged' end))
        assert(not pcall(function() formatter.FormatNumber = function() return 'forged' end end))
        assert(not pcall(formatter.FormatNumber, {}, 5))
        local cloned, forged = pcall(newproxy, formatter)
        assert(not cloned or not pcall(
            C_AuraContainerUtil.ProcessCustomAuraButtonDurationTextOptions,
            {textFormatter=forged}))
        local copy = securecopy({textFormatter=formatter})
        assert(copy.textFormatter == formatter)
        local options = C_AuraContainerUtil.ProcessCustomAuraButtonDurationTextOptions(copy)
        assert(options.textFormatter == formatter)
        collectgarbage('collect'); collectgarbage('collect')
        assert(options.textFormatter:FormatNumber(5) == '5s')
        assert(options.textFormatter:Format(60) == '1m')
        assert(options.textFormatter:GetMillisecondsThreshold() == 3)
        assert(options.textFormatter:GetStripIntervalWhitespace() == 1)
        assert(Enum.SecondsFormatterAbbreviation.OneLetter == 2)
        assert(Enum.SecondsFormatterAbbreviation.Truncate == 1)
        assert(Enum.SecondsFormatterAbbreviation.Full == nil)
    "#,
    )
    .unwrap();
    #[cfg(feature = "client-wowforever")]
    env.exec("assert(C_Intl == nil)").unwrap();
}

#[test]
fn seconds_formatter_native_binding_follows_remaining_time() {
    let env = configured_env();
    env.exec(
        r#"
        local formatter = NewActionSecondsFormatter()
        local options = C_AuraContainerUtil.ProcessCustomAuraButtonDurationTextOptions(
            securecopy({textFormatter=formatter}))
        local clock = C_DurationUtil.CreateManualClock(0)
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeSpan(0, 65)
        duration:SetClock(clock)
        local label = CreateFrame('Frame'):CreateFontString()
        local binding = C_DurationUtil.CreateDurationTextBinding()
        binding:SetFontString(label)
        binding:SetDuration(duration)
        binding:SetFormatter(options.textFormatter)
        for _, case in ipairs({{0, '1m'}, {60, '5s'}, {64.875, '0.125s'}, {65, '0s'}}) do
            clock:SetTime(case[1])
            binding:UpdateFontString()
            assert(label:GetText() == case[2], label:GetText())
        end
        for _, invalid in ipairs({false, '5', {}, math.huge, -math.huge, 0/0}) do
            assert(not pcall(formatter.FormatNumber, formatter, invalid))
            assert(label:GetText() == '0s')
            assert(formatter:FormatNumber(5) == '5s')
        end
        assert(not pcall(formatter.SetStripIntervalWhitespace, formatter, 3))
        assert(formatter:GetStripIntervalWhitespace() == 1)
    "#,
    )
    .unwrap();
}

#[test]
fn seconds_formatter_native_whitespace_respects_documented_locale_exceptions() {
    let env = configured_env();
    env.exec(
        r#"
        local formatter = NewActionSecondsFormatter()
        formatter:SetDefaultAbbreviation(Enum.SecondsFormatterAbbreviation.Truncate)
        formatter:SetStripIntervalWhitespace(0)
        assert(formatter:Format(60) == '1 min', formatter:Format(60))
        formatter:SetStripIntervalWhitespace(1)
        assert(formatter:Format(60) == '1min', formatter:Format(60))
        local original = GetLocale
        for _, locale in ipairs({'deDE', 'ruRU'}) do
            GetLocale = function() return locale end
            formatter:SetStripIntervalWhitespace(0)
            local preserved = formatter:Format(60)
            formatter:SetStripIntervalWhitespace(1)
            assert(formatter:Format(60) == preserved)
            formatter:SetStripIntervalWhitespace(2)
            local stripped = formatter:Format(60)
            assert(stripped ~= preserved, locale .. ':' .. stripped)
            assert(not stripped:find(' ', 1, true))
            assert(not stripped:find('\194\160', 1, true))
            assert(not stripped:find('\226\128\175', 1, true))
        end
        GetLocale = original
        formatter:SetDesiredUnitCount(2)
        formatter:SetStripIntervalWhitespace(1)
        assert(formatter:Format(90) == '1min, 30sec', formatter:Format(90))
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn seconds_formatter_native_secret_handoff_guards_text_and_callbacks() {
    let env = configured_env();
    env.exec(
        r#"
        local formatter = NewActionSecondsFormatter()
        local clock = C_DurationUtil.CreateManualClock(60)
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeSpan(secretwrap(0, 65))
        duration:SetClock(clock)
        local label = CreateFrame('Frame'):CreateFontString()
        local binding = C_DurationUtil.CreateDurationTextBinding()
        binding:SetDuration(duration)
        binding:SetFormatter(formatter)
        binding:SetFontString(label)
        binding:UpdateFontString()
        assert(label:GetText() == '5s')
        local output = formatter:FormatNumber(secretwrap(0.125))
        assert(issecretvalue(output) and secretunwrap(output) == '0.125s')
        local secret_five = secretwrap(5)
        local function untrusted()
            assert(not pcall(formatter.FormatNumber, formatter, secret_five))
            assert(not pcall(label.GetText, label))
        end
        debug.setobjecttaint(untrusted, 'SecondsProbe')
        untrusted()
        local observed
        local function curve(_, seconds)
            observed = seconds
            assert(issecretvalue(seconds), 'curve received decoded time')
            assert(not pcall(secretunwrap, seconds))
            return 0
        end
        debug.setobjecttaint(curve, 'SecondsCurveProbe')
        formatter:SetMaxIntervalCurve({Evaluate=curve})
        assert(secretunwrap(formatter:FormatNumber(secretwrap(5))) == '5s')
        assert(issecretvalue(observed))
        formatter:SetMaxIntervalCurve({Evaluate=function() return secretwrap(0) end})
        local derived = formatter:FormatNumber(5)
        assert(issecretvalue(derived) and secretunwrap(derived) == '5s')
        formatter:SetMaxIntervalCurve({Evaluate=function() error('seconds curve failure') end})
        local ok, err = pcall(binding.UpdateFontString, binding)
        assert(not ok and tostring(err):find('seconds curve failure', 1, true))
        assert(label:GetText() == '5s')
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn seconds_formatter_native_captures_conversion_functions_before_addon_overrides() {
    let env = configured_env();
    env.exec(r#"
        local formatter = NewActionSecondsFormatter()
        local input = secretwrap(5)
        local originals = {tostring, tonumber, math.floor, math.abs, math.max, math.min}
        local leaked = false
        local function spy(original)
            local function replacement(...)
                for i = 1, select('#', ...) do
                    local v = select(i, ...)
                    if type(v) == 'number' and v == 5 then leaked = true end
                end
                return original(...)
            end
            debug.setobjecttaint(replacement, 'SecondsConversionProbe')
            return replacement
        end
        tostring, tonumber, math.floor, math.abs, math.max, math.min =
            spy(tostring), spy(tonumber), spy(math.floor), spy(math.abs), spy(math.max), spy(math.min)
        local ok, output = pcall(formatter.FormatNumber, formatter, input)
        tostring, tonumber, math.floor, math.abs, math.max, math.min = unpack(originals)
        assert(ok, tostring(output))
        assert(issecretvalue(output) and secretunwrap(output) == '5s')
        assert(not leaked)
    "#).unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn seconds_formatter_native_addon_configuration_retains_native_secret_consumption() {
    let env = configured_env();
    env.exec(
        r#"
        local function configure()
            assert(not issecure())
            return NewActionSecondsFormatter()
        end
        debug.setobjecttaint(configure, 'SecondsConfigurationProbe')
        local formatter = configure()
        assert(issecure())
        local text = formatter:FormatNumber(secretwrap(5))
        assert(issecretvalue(text) and secretunwrap(text) == '5s')
        assert(formatter:GetMillisecondsThreshold() == 3)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn seconds_formatter_native_custom_aura_button_keeps_formatter_handle() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(CONFIGURE).unwrap();
                env.exec(r#"
                    local formatter = NewActionSecondsFormatter()
                    local container = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
                    local ran = false
                    local provider = __secureenv.AuraContainerUtil.CreateCustomFrameProvider(
                        GetForbiddenObjectTable(container), {batchSize=1,
                        templateNames={'CustomAuraButtonTemplate'}, initializeFrame=function(button)
                            local label = button:CreateFontString(nil, 'OVERLAY')
                            button:SetDurationText(label, {textFormatter=formatter})
                            local binding = GetForbiddenObjectTable(button):GetDurationTextBinding()
                            for _, case in ipairs({{5, '5s'}, {60, '1m'}, {0.125, '0.125s'}}) do
                                binding:SetDuration(case[1])
                                binding:UpdateFontString()
                                assert(label:GetText() == case[2], label:GetText())
                            end
                            ran = true
                        end})
                    assert(provider:AcquireFrame() ~= nil and ran)
                "#).unwrap();
            },
        );
    });
}
