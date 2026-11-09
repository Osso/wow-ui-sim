#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const SETUP: &str = r#"
    Clock = C_DurationUtil.CreateManualClock(0)
    Duration = C_DurationUtil.CreateDuration()
    Duration:SetTimeSpan(0, 8)
    Duration:SetClock(Clock)
    Label = CreateFrame('Frame'):CreateFontString()
    Binding = C_DurationUtil.CreateDurationTextBinding()
    Binding:SetDuration(Duration)
    Binding:SetFontString(Label)
    local formatter = C_StringUtil.CreateSecondsFormatter()
    formatter:SetDefaultAbbreviation(Enum.SecondsFormatterAbbreviation.OneLetter)
    formatter:SetStripIntervalWhitespace(Enum.SecondsFormatterIntervalWhitespace.Strip)
    Binding:SetFormatter(formatter)
"#;

fn setup() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(SETUP).unwrap();
    env
}

#[test]
fn automatic_binding_uses_tick_cadence_and_duration_clock() {
    let env = setup();
    env.fire_on_update(0.125).unwrap();
    env.exec(
        "assert(Label:GetText() == '8s', 'initial automatic update missing'); Clock:SetTime(2)",
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("assert(Label:GetText() == '8s', 'interval fired early')")
        .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("assert(Label:GetText() == '6s', 'interval boundary did not update')")
        .unwrap();
}

#[test]
fn automatic_binding_disable_reenable_and_configuration_invalidate_interval() {
    let env = setup();
    env.exec("Binding:SetUpdateInterval(10)").unwrap();
    env.fire_on_update(0.125).unwrap();
    env.exec("Binding:Disable(); Clock:SetTime(2)").unwrap();
    env.fire_on_update(20.0).unwrap();
    env.exec("assert(Label:GetText() == '8s'); Binding:Enable()")
        .unwrap();
    env.fire_on_update(0.125).unwrap();
    env.exec(
        r#"
        assert(Label:GetText() == '6s')
        local replacement = C_DurationUtil.CreateDuration()
        replacement:SetTimeSpan(0, 6)
        replacement:SetClock(Clock)
        Binding:SetDuration(replacement)
    "#,
    )
    .unwrap();
    env.fire_on_update(0.125).unwrap();
    env.exec("assert(Label:GetText() == '4s'); Binding:SetTextFormat('left %s')")
        .unwrap();
    env.fire_on_update(0.125).unwrap();
    env.exec("assert(Label:GetText() == 'left 4s')").unwrap();
}

#[test]
fn automatic_binding_copy_assign_and_reset_have_independent_schedules() {
    let env = setup();
    env.exec("Binding:SetUpdateInterval(2)").unwrap();
    env.fire_on_update(0.125).unwrap();
    env.exec("Clock:SetTime(1)").unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec(
        r#"
        CopyLabel = CreateFrame('Frame'):CreateFontString()
        Copy = Binding:Copy(); Copy:SetFontString(CopyLabel)
        AssignedLabel = CreateFrame('Frame'):CreateFontString()
        Assigned = C_DurationUtil.CreateDurationTextBinding()
        Assigned:Assign(Binding); Assigned:SetFontString(AssignedLabel)
    "#,
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec(
        r#"
        assert(Label:GetText() == '8s')
        assert(CopyLabel:GetText() == '7s' and AssignedLabel:GetText() == '7s')
        Copy:SetToDefaults()
        assert(Copy:GetDuration() == nil and Copy:GetFontString() == nil)
    "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec(
        r#"
        assert(CopyLabel:GetText() == '7s', 'defaults detach the previous FontString')
        assert(Label:GetText() == '8s' and AssignedLabel:GetText() == '7s')
        Clock:SetTime(2)
    "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec("assert(Label:GetText() == '6s'); assert(AssignedLabel:GetText() == '7s')")
        .unwrap();
}

#[test]
fn automatic_binding_weak_registry_does_not_keep_handle_alive() {
    let env = setup();
    env.fire_on_update(0.125).unwrap();
    env.exec(
        r#"
        Watch = setmetatable({Binding}, {__mode='v'})
        Binding = nil
        collectgarbage('collect'); collectgarbage('collect')
        assert(Watch[1] == nil, 'scheduler retained binding')
        Clock:SetTime(2)
    "#,
    )
    .unwrap();
    env.fire_on_update(2.0).unwrap();
    env.exec("assert(Label:GetText() == '8s', 'collected binding still updated')")
        .unwrap();
}

#[test]
fn automatic_binding_secret_numeric_formatter_blocks_conversion_leaks() {
    let env = setup();
    env.exec(
        r#"
        Duration:SetTimeSpan(secretwrap(0, 8))
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, format='%.0f'}})
        Binding:SetFormatter(formatter)
        assert(Binding:HasSecretValues(), 'binding lost secret duration state')
        assert(issecretvalue(Binding:GetFormattedText()), 'direct formatting lost secret output')
        Originals = {tostring, tonumber}
        ConversionLeak = false
        local function spy(original)
            local wrapped = function(value, ...)
                if type(value) == 'number' and value == 8 then ConversionLeak = true end
                return original(value, ...)
            end
            debug.setobjecttaint(wrapped, 'AutomaticConversionProbe')
            return wrapped
        end
        tostring, tonumber = spy(tostring), spy(tonumber)
    "#,
    )
    .unwrap();
    env.fire_on_update(0.125).unwrap();
    env.exec(
        r#"
        tostring, tonumber = Originals[1], Originals[2]
        local labelText = Label:GetText()
        assert(issecretvalue(labelText), 'automatic update exposed secret text')
        assert(secretunwrap(labelText) == '8', 'automatic update produced wrong secret text')
        assert(not ConversionLeak, 'automatic update called an addon conversion override')
        local read = function() return Label:GetText() end
        debug.setobjecttaint(read, 'AutomaticBindingReader')
        assert(not pcall(read), 'secret text became public')
    "#,
    )
    .unwrap();
}

#[test]
fn automatic_binding_reports_format_error_without_aborting_other_updates() {
    let env = setup();
    env.exec(
        r#"
        BadLabel = CreateFrame('Frame'):CreateFontString()
        BadLabel:SetText('unchanged')
        Bad = Binding:Copy()
        Bad:SetFontString(BadLabel)
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeSpan(0, 8)
        Bad:SetDuration(duration)
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, format='%.0f'}})
        Bad:SetFormatter(formatter)
        formatter:ClearBreakpoints()
    "#,
    )
    .unwrap();
    let previous_frames = env.state().borrow().app_frame_metrics.session_frame_count;
    let previous_errors = env.state().borrow().lua_errors.len();
    env.fire_on_update(0.125)
        .expect("one formatting failure must not abort the engine tick");
    assert_eq!(
        env.state().borrow().app_frame_metrics.session_frame_count,
        previous_frames + 1
    );
    env.exec(
        r#"
        assert(Label:GetText() == '8s', 'healthy binding starved')
        assert(BadLabel:GetText() == 'unchanged', 'failure mutated text')
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.len() > previous_errors);
}

#[test]
fn automatic_binding_updates_native_custom_aura_button_after_assignment() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(r#"
                    Clock = C_DurationUtil.CreateManualClock(0)
                    local container = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
                    local provider = __secureenv.AuraContainerUtil.CreateCustomFrameProvider(
                        GetForbiddenObjectTable(container), {batchSize=1,
                        templateNames={'CustomAuraButtonTemplate'}, initializeFrame=function(button)
                            NativeLabel = button:CreateFontString(nil, 'OVERLAY')
                            local formatter = C_StringUtil.CreateSecondsFormatter()
                            formatter:SetDefaultAbbreviation(Enum.SecondsFormatterAbbreviation.OneLetter)
                            formatter:SetStripIntervalWhitespace(Enum.SecondsFormatterIntervalWhitespace.Strip)
                            button:SetDurationText(NativeLabel, {textFormatter=formatter})
                        end})
                    NativeButton = provider:AcquireFrame()
                    local private = GetForbiddenObjectTable(NativeButton)
                    private:SetAuraInstance('player', {auraInstanceID=9981, spellId=19750,
                        name='Automatic binding', duration=8, expirationTime=8, timeMod=1,
                        applications=1, isHelpful=true, sourceUnit='player'})
                    private:GetAuraDuration():SetClock(Clock)
                "#).unwrap();
                env.fire_on_update(0.125).unwrap();
                env.exec("assert(NativeLabel:GetText() == '8s', 'native automatic initial update missing'); Clock:SetTime(2)").unwrap();
                env.fire_on_update(1.0).unwrap();
                env.exec(
                    "assert(NativeLabel:GetText() == '6s', 'native automatic progression missing')",
                )
                .unwrap();
            },
        );
    });
}
