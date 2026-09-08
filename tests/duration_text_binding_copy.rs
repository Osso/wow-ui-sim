#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

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
                    local container = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
                    local provider = __secureenv.AuraContainerUtil.CreateCustomFrameProvider(
                        GetForbiddenObjectTable(container), {batchSize=1,
                        templateNames={'CustomAuraButtonTemplate'}, initializeFrame=function(button)
                            local label = button:CreateFontString(nil, 'OVERLAY')
                            button:SetDurationText(label, {binding=source})
                            assert(button:GetDurationText() == label)
                            local ownedBinding = GetForbiddenObjectTable(button):GetDurationTextBinding()
                            assert(ownedBinding ~= source and ownedBinding:GetFontString() == label)
                            assert(ownedBinding:GetUpdateInterval() == 0.25)
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
