#![cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-wowforever")]
#[test]
fn duration_binding_secret_duration_preserves_font_string_boundary() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeFromStart(secretwrap(10, 20, 1))
        duration:SetClock(C_DurationUtil.CreateManualClock(12))
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, format='%.0f'}})
        local label = CreateFrame('Frame'):CreateFontString()
        local binding = C_DurationUtil.CreateDurationTextBinding()
        binding:SetDuration(duration)
        binding:SetFormatter(formatter)
        binding:SetFontString(label)
        binding:SetTextFormat('remaining %s')
        assert(binding:HasSecretValues() and binding:Copy():HasSecretValues())
        assert(binding:GetFormattedText() == 'remaining 18')
        binding:UpdateFontString()
        assert(label:GetText() == 'remaining 18')
        local function untrusted()
            assert(binding:HasSecretValues())
            assert(not pcall(binding.GetFormattedText, binding))
            assert(not pcall(binding.UpdateFontString, binding))
            assert(not pcall(label.GetText, label))
        end
        debug.setobjecttaint(untrusted, 'BindingProbe')
        untrusted()
        assert(label:GetText() == 'remaining 18')
        binding:SetDuration(secretwrap(12))
        assert(binding:HasSecretValues())
        assert(binding:GetFormattedText() == 'remaining 12')
        binding:SetToDefaults()
        assert(not binding:HasSecretValues())
    "#,
    )
    .expect("secret duration and numeric input retain secrecy across formatting and text output");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn duration_binding_secret_formatter_callback_receives_only_wrapped_number() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeFromStart(secretwrap(10, 20, 1))
        duration:SetClock(C_DurationUtil.CreateManualClock(12))
        local observed
        local function format(_, value)
            assert(not issecure())
            assert(issecretvalue(value), 'formatter callback received plain timing')
            assert(not pcall(secretunwrap, value))
            observed = value
            return 'opaque callback'
        end
        debug.setobjecttaint(format, 'FormatterProbe')
        local formatter = newproxy(true)
        getmetatable(formatter).__index = {FormatNumber=format}
        local binding = C_DurationUtil.CreateDurationTextBinding()
        binding:SetDuration(duration)
        binding:SetFormatter(formatter)
        assert(binding:GetFormattedText() == 'opaque callback')
        assert(issecretvalue(observed))
        assert(secretunwrap(observed) == 18)
    "#,
    )
    .expect("addon userdata formatter cannot observe plain secret timing");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn duration_binding_secret_callback_failures_are_not_replaced_with_text() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeSpan(secretwrap(10, 30))
        duration:SetClock(C_DurationUtil.CreateManualClock(12))
        local label = CreateFrame('Frame'):CreateFontString()
        label:SetText('unchanged')
        local binding = C_DurationUtil.CreateDurationTextBinding()
        binding:SetDuration(duration)
        binding:SetFontString(label)
        local function format(value)
            assert(not issecure())
            assert(value == duration)
            return value:GetRemainingDuration()
        end
        debug.setobjecttaint(format, 'FormatterProbe')
        binding:SetFormatter(format)
        assert(not pcall(binding.GetFormattedText, binding))
        assert(not pcall(binding.UpdateFontString, binding))
        assert(label:GetText() == 'unchanged')
        local function failure() error('secret formatter failure') end
        debug.setobjecttaint(failure, 'FormatterProbe')
        binding:SetFormatter({Format=failure})
        local ok, err = pcall(binding.GetFormattedText, binding)
        assert(not ok and tostring(err):find('secret formatter failure', 1, true))
    "#,
    )
    .expect("secret formatting errors propagate before changing the target text");
}

#[test]
fn duration_binding_userdata_copy_retains_resources_through_collection() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDurationTextBinding()
        assert(type(source) == 'userdata')
        assert(not pcall(rawget, source, 'duration'))
        assert(not pcall(rawset, source, 'duration', 99))

        local duration = C_DurationUtil.CreateDuration()
        local clock = C_DurationUtil.CreateManualClock(12)
        local label = CreateFrame('Frame'):CreateFontString()
        local formatter = {Format=function(_, value) return 'retained:' .. value end}
        local curve = C_CurveUtil.CreateColorCurve()
        source:SetDuration(duration)
        source:SetClock(clock)
        source:SetFontString(label)
        source:SetFormatter(formatter)
        source:SetTextColorCurve(curve, Enum.DurationTextBindingProperty.RemainingDuration)
        local resources = setmetatable({duration, clock, label, formatter, curve}, {__mode='v'})
        local retained = source:Copy()
        local identity = {[retained]=true}
        source, duration, clock, label, formatter, curve = nil, nil, nil, nil, nil, nil
        collectgarbage('collect')

        assert(identity[retained], 'retained binding keeps its table-key identity')
        for index = 1, 5 do assert(resources[index] ~= nil, 'copied resource was collected') end
        assert(retained:GetDuration() == resources[1])
        assert(retained:GetClock() == resources[2])
        assert(retained:GetFontString() == resources[3])
        assert(retained:GetTextColorCurve() == resources[5])
        retained:GetClock():SetTime(29)
        assert(resources[2]:GetTime() == 29)
        retained:SetDuration(17)
        retained:UpdateFontString()
        assert(resources[3]:GetText() == 'retained:17')
        resources[4].Format = function(_, value) return 'shared:' .. value end
        retained:UpdateFontString()
        assert(resources[3]:GetText() == 'shared:17')
        "#,
    )
    .expect("userdata binding retains identity and copied resource handles through collection");
}

#[test]
fn duration_binding_assign_preserves_receiver_and_configuration_handles() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDurationTextBinding()
        local target = C_DurationUtil.CreateDurationTextBinding()
        local retainedTarget = target
        local duration = C_DurationUtil.CreateDuration()
        local clock = C_DurationUtil.CreateManualClock(12)
        local label = CreateFrame('Frame'):CreateFontString()
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, step=1, rounding=Enum.NumericRuleFormatRounding.Up, format='%.0f'}})
        local curve = C_CurveUtil.CreateColorCurve()
        local property = Enum.DurationTextBindingProperty.RemainingPercent
        source:SetDuration(duration)
        source:SetClock(clock)
        source:SetFontString(label)
        source:SetFormatter(formatter)
        source:SetTextColorCurve(curve, property)
        source:SetTimeModifier(1)
        source:SetUpdateInterval(0.25)
        source:SetExpiredText('finished')
        source:SetZeroDurationText('empty')
        source:Disable()

        assert(select('#', target:Assign(source)) == 0)
        assert(target == retainedTarget and target ~= source)
        assert(target:GetDuration() == duration and target:GetClock() == clock)
        assert(target:GetFontString() == label)
        local copiedCurve, copiedProperty = target:GetTextColorCurve()
        assert(copiedCurve == curve and copiedProperty == property)
        assert(target:GetTimeModifier() == 1 and target:GetUpdateInterval() == 0.25)
        assert(target:GetExpiredText() == 'finished' and target:GetZeroDurationText() == 'empty')
        assert(not target:IsEnabled())
        target:Enable()
        assert(target:IsEnabled() and not source:IsEnabled())
        target:SetDuration(8.2)
        target:UpdateFontString()
        assert(label:GetText() == '9')
        formatter:SetBreakpoints({{threshold=0, format='%.1f'}})
        assert(target:GetFormattedText() == '8.2', 'formatter handle remains shared')
        assert(source:GetDuration() == duration, 'receiver reconfiguration must not alter source')
        "#,
    )
    .expect("Assign copies configuration without replacing the receiver or cloning handles");
}

#[test]
fn duration_binding_survives_secure_option_copy_without_losing_handle_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDurationTextBinding()
        local label = CreateFrame('Frame'):CreateFontString()
        source:SetFontString(label)
        source:SetDuration(7)
        local options = {binding=source, textFormat={formatString='%s', components={}}}
        local copied = securecopy(options)
        assert(copied ~= options and copied.textFormat ~= options.textFormat)
        assert(copied.binding == source, 'securecopy preserves the binding handle')
        local processed = C_AuraContainerUtil.ProcessCustomAuraButtonDurationTextOptions(copied)
        local receiver = C_DurationUtil.CreateDurationTextBinding()
        receiver:Assign(processed.binding)
        assert(receiver:GetFontString() == label)
        receiver:UpdateFontString()
        assert(label:GetText() == '7')
        assert(not pcall(receiver.Assign, receiver, {__wowDurationTextBinding=true}))
        "#,
    )
    .expect("secure option copying preserves native binding identity and rejects forged tables");
}

#[test]
fn duration_binding_copy_updates_text_with_independent_configuration() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDurationTextBinding()
        local owner = CreateFrame('Frame')
        local firstLabel = owner:CreateFontString()
        local secondLabel = owner:CreateFontString()
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, step=1, rounding=Enum.NumericRuleFormatRounding.Up, format='%.0f'}})
        local duration = C_DurationUtil.CreateDuration()
        local curve = C_CurveUtil.CreateColorCurve()
        source:SetDuration(duration)
        source:SetFontString(firstLabel)
        source:SetFormatter(formatter)
        source:SetTextColorCurve(curve, Enum.DurationTextBindingProperty.RemainingDuration)
        source:SetExpiredText('expired')
        source:SetZeroDurationText('zero')
        source:SetUpdateInterval(0.125)
        source:Disable()
        local copy = source:Copy()
        assert(copy ~= source and copy:GetDuration() == duration)
        assert(copy:GetFontString() == firstLabel and copy:GetTextColorCurve() == curve)
        assert(not copy:IsEnabled() and copy:GetUpdateInterval() == 0.125)
        assert(copy:GetExpiredText() == 'expired' and copy:GetZeroDurationText() == 'zero')

        copy:Enable()
        copy:SetFontString(secondLabel)
        copy:SetDuration(3.2)
        copy:UpdateFontString()
        assert(secondLabel:GetText() == '4')
        assert(source:GetFontString() == firstLabel and not source:IsEnabled())
        source:SetDuration(8.2)
        source:UpdateFontString()
        assert(firstLabel:GetText() == '9' and secondLabel:GetText() == '4')
        source:SetFormatter({Format=function(_, value) return 'source:' .. value end})
        source:SetExpiredText('changed')
        source:ClearTextColorCurve()
        assert(copy:GetFormattedText() == '4' and copy:GetExpiredText() == 'expired')
        assert(copy:GetTextColorCurve() == curve)
        formatter:SetBreakpoints({{threshold=0, format='%.1f'}})
        copy:UpdateFontString()
        assert(secondLabel:GetText() == '3.2', 'copied binding retains the formatter handle')
        "#,
    )
    .expect("Copy owns configuration while retaining referenced display objects");
}

#[test]
fn duration_binding_copies_mutable_format_components_without_cloning_formatter_handles() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDurationTextBinding()
        local assigned = C_DurationUtil.CreateDurationTextBinding()
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, format='%.0f'}})
        local property = Enum.DurationTextBindingProperty.RemainingDuration
        local components = {{property=property, formatter=formatter}}
        source:SetDuration(7)
        source:SetFormatter(formatter)
        source:SetTextFormat('remaining %s', components)
        assigned:Assign(source)
        local copied = source:Copy()
        -- The existing table-backed binding exposes its configuration. Mutate
        -- the supplied records to check state independence, not method layout.
        components[1].property = Enum.DurationTextBindingProperty.ElapsedDuration
        components[2] = {property=Enum.DurationTextBindingProperty.TotalDuration, formatter=formatter}
        assert(assigned.textFormatComponents[1].property == property)
        assert(copied.textFormatComponents[1].property == property)
        assert(#assigned.textFormatComponents == 1 and #copied.textFormatComponents == 1)
        assert(assigned.textFormatComponents[1].formatter == formatter)
        assert(copied.textFormatComponents[1].formatter == formatter)
        assigned.textFormatComponents[1].property = Enum.DurationTextBindingProperty.TotalDuration
        assert(copied.textFormatComponents[1].property == property)
        source:SetTextFormat('changed %s', {})
        assert(assigned:GetFormattedText() == 'remaining 7')
        assert(copied:GetFormattedText() == 'remaining 7')
        assert(source:GetFormattedText() == 'changed 7')
        "#,
    )
    .expect("component records are copied while object handles remain shared");
}

#[test]
fn duration_binding_assign_validates_atomically_clears_absent_values_and_handles_self_assignment() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local binding = C_DurationUtil.CreateDurationTextBinding()
        local label = CreateFrame('Frame'):CreateFontString()
        binding:SetDuration(6)
        binding:SetFontString(label)
        binding:SetTextFormat('kept %s', {})
        binding:SetExpiredText('expired')
        binding:SetZeroDurationText('zero')
        assert(select('#', binding:Assign(binding)) == 0)
        assert(binding:GetDuration() == 6 and binding:GetFontString() == label)
        assert(binding:GetFormattedText() == 'kept 6')
        local invalid = {17, 'invalid', {}, {GetDuration=function() return 99 end}}
        assert(not pcall(binding.Assign, binding, nil))
        for _, value in ipairs(invalid) do
            assert(not pcall(binding.Assign, binding, value))
            assert(binding:GetDuration() == 6 and binding:GetFormattedText() == 'kept 6')
        end
        assert(not pcall(binding.Assign, {}, binding), 'receiver must also be a binding')
        assert(not pcall(binding.Copy, {}), 'Copy validates its receiver')
        binding:UpdateFontString()
        assert(label:GetText() == 'kept 6')

        local empty = C_DurationUtil.CreateDurationTextBinding()
        binding:Assign(empty)
        assert(binding:GetDuration() == empty:GetDuration())
        assert(binding:GetFontString() == nil and not binding:CanUpdateFontString())
        assert(binding:GetExpiredText() == nil and binding:GetZeroDurationText() == nil)
        assert(binding:GetTextColorCurve() == nil)
        assert(binding:GetFormattedText() == empty:GetFormattedText())
        "#,
    )
    .expect("invalid assignments leave state intact and empty configuration replaces old values");
}

#[test]
fn duration_binding_assignment_supports_actual_custom_aura_button_duration_text() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(
                    r#"
                    local formatter = C_StringUtil.CreateNumericRuleFormatter()
                    formatter:SetBreakpoints({{threshold=0, step=1, rounding=Enum.NumericRuleFormatRounding.Up, format='%.0f'}})
                    local originalLabel = CreateFrame('Frame'):CreateFontString()
                    local source = C_DurationUtil.CreateDurationTextBinding()
                    source:SetDuration(2.2)
                    source:SetFontString(originalLabel)
                    source:SetFormatter(formatter)
                    source:SetUpdateInterval(0.25)
                    local curve = C_CurveUtil.CreateColorCurve()
                    curve:SetType(Enum.LuaCurveType.Step)
                    curve:AddPoint(0, CreateColor(1, 1, 1, 1))
                    curve:AddPoint(60, CreateColor(1, 0.82, 0, 1))
                    local container = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
                    local provider = __secureenv.AuraContainerUtil.CreateCustomFrameProvider(
                        GetForbiddenObjectTable(container), {batchSize=1,
                        templateNames={'CustomAuraButtonTemplate'}, initializeFrame=function(button)
                            local label = button:CreateFontString(nil, 'OVERLAY')
                            button:SetDurationText(label, {binding=source, textColor={
                                curve=curve, property=Enum.DurationTextBindingProperty.RemainingDuration}})
                            assert(button:GetDurationText() == label)
                            local ownedBinding = GetForbiddenObjectTable(button):GetDurationTextBinding()
                            assert(ownedBinding ~= source and ownedBinding:GetFontString() == label)
                            assert(ownedBinding:GetUpdateInterval() == 0.25)
                            assert(ownedBinding:GetTextColorCurve() == curve)
                            local r,g,b,a = curve:Evaluate(30):GetRGBA()
                            assert(r == 1 and g == 1 and b == 1 and a == 1)
                            r,g,b,a = curve:Evaluate(60):GetRGBA()
                            assert(r == 1 and math.abs(g - 0.82) < 0.00001 and b == 0 and a == 1)
                            ownedBinding:SetDuration(8.2)
                            ownedBinding:UpdateFontString()
                            assert(label:GetText() == '9')
                            source:UpdateFontString()
                            assert(originalLabel:GetText() == '3' and source:GetFontString() == originalLabel)
                            durationBindingInitializerRan = true
                        end})
                    assert(provider:AcquireFrame() ~= nil)
                    assert(durationBindingInitializerRan)
                    "#,
                )
                .expect("native binding assignment feeds the real aura initializer");
            },
        );
    });
}
