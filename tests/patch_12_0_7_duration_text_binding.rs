#![cfg(feature = "retail-12-0-7")]

use wow_ui_sim::lua_api::WowLuaEnv;

const SETUP: &str = r#"
    Clock = C_DurationUtil.CreateManualClock(10)
    Duration = C_DurationUtil.CreateDuration()
    Duration:SetClock(Clock)
    Duration:SetTimeFromStart(10, 16, 2)
    Label = CreateFrame('Frame'):CreateFontString()
    Formatter = C_StringUtil.CreateSecondsFormatter()
    Formatter:SetDefaultAbbreviation(Enum.SecondsFormatterAbbreviation.OneLetter)
    Formatter:SetStripIntervalWhitespace(Enum.SecondsFormatterIntervalWhitespace.Strip)
    Binding = C_DurationUtil.CreateDurationTextBinding()
    Binding:SetDuration(Duration)
    Binding:SetFontString(Label)
    Binding:SetFormatter(Formatter)
"#;

fn binding_environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("duration binding environment");
    env.exec(SETUP).expect("concrete binding fixture");
    env
}

#[test]
fn p1207_binding_defaults_and_exact_query_arity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local a = C_DurationUtil.CreateDurationTextBinding()
        local b = C_DurationUtil.CreateDurationTextBinding()
        assert(type(a) == 'userdata' and a ~= b)
        -- INFERRED: cleared configuration has no implicit duration/formatter/text.
        for _, name in ipairs({'GetDuration','GetFontString','GetExpiredText','GetZeroDurationText'}) do
            assert(select('#', a[name](a)) == 1 and a[name](a) == nil, name)
        end
        for _, item in ipairs({{'CanFormatText',false},{'CanUpdateFontString',false},
                              {'IsEnabled',true},{'GetTimeModifier',0},{'GetUpdateInterval',1}}) do
            assert(select('#', a[item[1]](a)) == 1 and a[item[1]](a) == item[2], item[1])
        end
        assert(not pcall(a.GetFormattedText, a))
        assert(select('#', a:SetZeroDurationText('unset')) == 0)
        assert(a:CanFormatText() and a:GetFormattedText() == 'unset')
        assert(not a:CanUpdateFontString() and not b:CanFormatText())
        assert(select('#', a:Disable()) == 0 and not a:IsEnabled())
        assert(select('#', a:Enable()) == 0 and a:IsEnabled())
    "#).unwrap();
}

#[test]
fn p1207_binding_state_roundtrips_copy_assign_and_reset_preserve_surface() {
    let env = binding_environment();
    env.exec(r#"
        assert(Binding:GetDuration() == Duration and Binding:GetFontString() == Label)
        assert(select('#', Binding:SetTimeModifier(1)) == 0)
        assert(select('#', Binding:SetUpdateInterval(0.25)) == 0)
        assert(select('#', Binding:SetExpiredText('finished')) == 0)
        assert(select('#', Binding:SetZeroDurationText('empty')) == 0)
        Binding:SetClock(Clock)
        local copy = Binding:Copy()
        local assigned = C_DurationUtil.CreateDurationTextBinding()
        assert(select('#', assigned:Assign(Binding)) == 0)
        for _, item in ipairs({copy, assigned}) do
            assert(item ~= Binding and item:GetDuration() == Duration and item:GetFontString() == Label)
            assert(item:GetTimeModifier() == 1 and item:GetUpdateInterval() == 0.25)
            assert(item:GetExpiredText() == 'finished' and item:GetZeroDurationText() == 'empty')
            assert(item:GetClock() == Clock)
        end
        copy:SetTimeModifier(0); copy:SetUpdateInterval(2); copy:Disable()
        assert(Binding:GetTimeModifier() == 1 and Binding:GetUpdateInterval() == 0.25 and Binding:IsEnabled())
        assert(assigned:GetTimeModifier() == 1 and assigned:GetUpdateInterval() == 0.25)
        assigned:SetToDefaults()
        assert(assigned:GetDuration() == nil and assigned:GetFontString() == nil)
        assert(assigned:IsEnabled() and assigned:GetTimeModifier() == 0 and assigned:GetUpdateInterval() == 1)
        assert(assigned:GetExpiredText() == nil and assigned:GetZeroDurationText() == nil)
        assert(not assigned:CanFormatText() and Binding:GetDuration() == Duration)
        Binding:SetTextFormat('remaining %s') -- Preserve existing percent-format extension.
        assert(Binding:GetFormattedText() == 'remaining 16s')
    "#).unwrap();
}

#[test]
fn p1207_binding_formats_live_host_clock_rate_zero_and_expiration() {
    let env = binding_environment();
    env.exec(
        r#"
        Binding:SetExpiredText('done'); Binding:SetZeroDurationText('zero')
        Clock:SetTime(9)
        assert(Binding:GetFormattedText() == '8s')
        Clock:SetTime(12)
        assert(select('#', Binding:GetFormattedText()) == 1 and Binding:GetFormattedText() == '6s')
        Binding:SetTimeModifier(1)
        assert(Binding:GetFormattedText() == '12s')
        Clock:SetTime(18)
        assert(Binding:GetFormattedText() == 'done')
        Clock:RewindTime(1)
        assert(Binding:GetFormattedText() == '2s')
        Duration:SetTimeFromStart(100, 0)
        -- INFERRED: zero text wins even though inherited fully-elapsed rule reports expired.
        assert(Duration:HasExpired() and Binding:GetFormattedText() == 'zero')
        Duration:SetTimeFromStart(10, 4, 2)
        Clock:SetTime(11)
        assert(Binding:GetFormattedText() == '2s')
        Binding:UpdateFontString()
        assert(Label:GetText() == '2s')
    "#,
    )
    .unwrap();
}

#[test]
fn p1207_binding_automatic_updates_read_live_state_and_respect_disable_interval() {
    let env = binding_environment();
    env.fire_on_update(0.125).unwrap();
    env.exec("assert(Label:GetText() == '8s'); Clock:SetTime(12)")
        .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("assert(Label:GetText() == '8s')").unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("assert(Label:GetText() == '6s'); Binding:Disable(); Clock:SetTime(13)")
        .unwrap();
    env.fire_on_update(20.0).unwrap();
    env.exec("assert(Label:GetText() == '6s'); Binding:Enable()")
        .unwrap();
    env.fire_on_update(0.125).unwrap();
    env.exec("assert(Label:GetText() == '5s'); Binding:SetUpdateInterval(0); Clock:SetTime(14)")
        .unwrap();
    env.fire_on_update(0.01).unwrap();
    env.exec("assert(Label:GetText() == '4s'); Clock:SetTime(15)")
        .unwrap();
    env.fire_on_update(0.01).unwrap();
    env.exec("assert(Label:GetText() == '3s')").unwrap();
}

#[test]
fn p1207_binding_configuration_and_clock_are_environment_local() {
    let first = binding_environment();
    let second = binding_environment();
    first.exec("Clock:SetTime(12); Binding:SetTimeModifier(1); Binding:SetUpdateInterval(0); Binding:SetExpiredText('first')").unwrap();
    second
        .exec("Clock:SetTime(11); Binding:Disable(); Binding:SetExpiredText('second')")
        .unwrap();
    let first_text: String = first.eval("return Binding:GetFormattedText()").unwrap();
    let second_text: String = second.eval("return Binding:GetFormattedText()").unwrap();
    assert_eq!(first_text, "12s");
    assert_eq!(second_text, "7s");
    first.exec("assert(Binding:IsEnabled() and Binding:GetUpdateInterval() == 0 and Binding:GetExpiredText() == 'first')").unwrap();
    second.exec("assert(not Binding:IsEnabled() and Binding:GetUpdateInterval() == 1 and Binding:GetExpiredText() == 'second')").unwrap();
}

#[test]
fn p1207_binding_invalid_setters_reject_before_mutation() {
    let env = binding_environment();
    env.exec(r#"
        Binding:SetExpiredText('keep'); Binding:SetZeroDurationText('keep-zero')
        for _, name in ipairs({'SetDuration','SetFontString','SetTimeModifier','SetUpdateInterval','SetEnabled','SetFormatter'}) do
            assert(not pcall(Binding[name], Binding), name .. ' accepted missing input')
            assert(not pcall(Binding[name], Binding, nil), name .. ' accepted nil')
            assert(not pcall(Binding[name], Binding, {}), name .. ' accepted table')
        end
        for _, value in ipairs({-1, math.huge, -math.huge, 0/0, '0.25', false}) do
            assert(not pcall(Binding.SetUpdateInterval, Binding, value))
        end
        assert(not pcall(Binding.SetTimeModifier, Binding, 1.5))
        assert(not pcall(Binding.SetExpiredText, Binding, 12))
        assert(not pcall(Binding.SetZeroDurationText, Binding, false))
        assert(Binding:GetDuration() == Duration and Binding:GetFontString() == Label)
        assert(Binding:GetTimeModifier() == 0 and Binding:GetUpdateInterval() == 1 and Binding:IsEnabled())
        assert(Binding:GetExpiredText() == 'keep' and Binding:GetZeroDurationText() == 'keep-zero')
        Binding:SetExpiredText(nil); Binding:SetZeroDurationText(nil)
        assert(Binding:GetExpiredText() == nil and Binding:GetZeroDurationText() == nil)
        local formatter = newproxy(true)
        getmetatable(formatter).__index = {FormatNumber = function() error('formatter boundary') end}
        Binding:SetFormatter(formatter)
        local ok, errorValue = pcall(Binding.GetFormattedText, Binding)
        assert(not ok and tostring(errorValue):find('formatter boundary', 1, true))
        assert(Label:GetText() == nil or Label:GetText() == '')
        for _, callback in ipairs({function() return nil end, function() return 12 end}) do
            local invalid = newproxy(true)
            getmetatable(invalid).__index = {FormatNumber=callback}
            Binding:SetFormatter(invalid)
            assert(not pcall(Binding.GetFormattedText, Binding))
        end
    "#).unwrap();
}

#[test]
fn p1207_binding_allowed_when_untainted_authenticates_every_input_and_extra_first() {
    let env = binding_environment();
    env.exec(r#"
        local wrappedDuration, wrappedFont, wrappedModifier, wrappedInterval, wrappedText, extra =
            secretwrap(Duration, Label, 1, 0.25, 'wrapped', 3)
        Binding:SetDuration(wrappedDuration)
        Binding:SetFontString(wrappedFont)
        Binding:SetTimeModifier(wrappedModifier)
        Binding:SetUpdateInterval(wrappedInterval)
        Binding:SetExpiredText(wrappedText)
        assert(Binding:GetDuration() == Duration and Binding:GetFontString() == Label)
        assert(Binding:GetTimeModifier() == 1 and Binding:GetUpdateInterval() == 0.25)
        assert(issecretvalue(Binding:GetExpiredText()) and secretunwrap(Binding:GetExpiredText()) == 'wrapped')
        Binding:SetZeroDurationText(secretwrap(nil))
        assert(Binding:GetZeroDurationText() == nil)
        local cases = {{'SetDuration',wrappedDuration},{'SetFontString',wrappedFont},
            {'SetTimeModifier',wrappedModifier},{'SetUpdateInterval',wrappedInterval},
            {'SetExpiredText',wrappedText},{'SetZeroDurationText',wrappedText}}
        local function addon()
            assert(debug.getstacktaint() == 'BindingAudit')
            for _, item in ipairs(cases) do
                local method = Binding[item[1]]
                assert(not pcall(method, Binding, item[2]))
                -- Invalid receiver/early explicit input may not hide a later secret extra.
                for _, receiver in ipairs({{}, Binding}) do
                    local ok, message = pcall(method, receiver, {}, extra)
                    assert(not ok and tostring(message):find('untainted', 1, true), item[1])
                end
            end
            assert(Binding:GetDuration() == Duration and Binding:GetTimeModifier() == 1)
            Binding:SetUpdateInterval(0.5)
            assert(Binding:GetUpdateInterval() == 0.5 and debug.getstacktaint() == 'BindingAudit')
        end
        debug.setobjecttaint(addon, 'BindingAudit'); addon()
        assert(debug.getstacktaint() == nil and Binding:GetExpiredText() ~= nil)
        -- Secure callers also authenticate extras, then perform ordinary validation.
        Binding:SetUpdateInterval(0.75, extra)
        assert(Binding:GetUpdateInterval() == 0.75)
    "#).unwrap();
}

#[test]
fn p1207_binding_secret_timing_and_text_stay_opaque_to_tainted_callers() {
    let env = binding_environment();
    env.exec(
        r#"
        Duration:SetTimeFromStart(secretwrap(10, 16, 2))
        local text = Binding:GetFormattedText()
        -- INFERRED: sampled secret timing yields a VM-owned secret text result.
        assert(issecretvalue(text) and secretunwrap(text) == '8s')
        local function addon()
            assert(not pcall(Binding.GetFormattedText, Binding))
            assert(not pcall(secretunwrap, text))
            assert(debug.getstacktaint() == 'BindingSecretAudit')
        end
        debug.setobjecttaint(addon, 'BindingSecretAudit'); addon()
        assert(debug.getstacktaint() == nil)
        Binding:UpdateFontString()
        local labelText = Label:GetText()
        assert(issecretvalue(labelText) and secretunwrap(labelText) == '8s')
        Binding:SetExpiredText('expired'); Clock:SetTime(18)
        local expired = Binding:GetFormattedText()
        assert(issecretvalue(expired) and secretunwrap(expired) == 'expired')
        Binding:SetZeroDurationText('zero'); Duration:SetTimeFromStart(100, 0)
        local zeroTiming = Binding:GetFormattedText()
        assert(issecretvalue(zeroTiming) and secretunwrap(zeroTiming) == 'zero')
        Duration:SetToDefaults()
        local zero = secretwrap('hidden-zero')
        Binding:SetZeroDurationText(zero)
        local function readZero()
            local result = Binding:GetFormattedText()
            assert(issecretvalue(result) and not pcall(secretunwrap, result))
            assert(debug.getstacktaint() == 'BindingZeroAudit')
        end
        debug.setobjecttaint(readZero, 'BindingZeroAudit'); readZero()
    "#,
    )
    .unwrap();
}

#[test]
fn p1207_has_expired_inherits_zero_span_and_authenticates_modifiers_and_extras() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local clock = C_DurationUtil.CreateManualClock(-5)
        local duration = C_DurationUtil.CreateDuration()
        duration:SetClock(clock)
        -- INFERRED inherited 12.0.5 fully-elapsed semantics; source lists addition only.
        assert(duration:HasExpired())
        duration:SetTimeFromStart(10, 4)
        assert(not duration:HasExpired())
        clock:SetTime(14)
        assert(select('#', duration:HasExpired()) == 1 and duration:HasExpired())
        local modifier, extra = secretwrap(1, 5)
        assert(duration:HasExpired(modifier, extra))
        local function addon()
            local ok, message = pcall(duration.HasExpired, {}, {}, extra)
            assert(not ok and tostring(message):find('untainted', 1, true))
            assert(not pcall(duration.HasExpired, duration, modifier))
            assert(duration:HasExpired(0) and debug.getstacktaint() == 'ExpiredAudit')
        end
        debug.setobjecttaint(addon, 'ExpiredAudit'); addon()
        assert(debug.getstacktaint() == nil)
        assert(not pcall(duration.HasExpired, duration, 2))
    "#,
    )
    .unwrap();
}
