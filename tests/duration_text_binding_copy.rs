#![cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "forbidden-aspects")]
#[test]
fn duration_binding_font_string_accepts_native_views_and_checks_taint() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local label = CreateFrame('Frame'):CreateFontString()
        local forbidden = GetForbiddenObjectTable(label)
        assert(label ~= forbidden, 'FontString public and forbidden views are distinct')
        local binding = C_DurationUtil.CreateDurationTextBinding()
        binding:SetFontString(label)
        binding:SetFontString(forbidden)
        local function addonSet(value) binding:SetFontString(value) end
        debug.setobjecttaint(addonSet, 'FontStringProbe')
        assert(pcall(addonSet, label))
        local ok, err = pcall(addonSet, forbidden)
        assert(not ok, 'tainted SetFontString rejects the forbidden view')
        assert(err:find('Attempt to access forbidden object from code tainted by an AddOn - Usage: self:SetFontString(fontString)', 1, true), err)
        "#,
    )
    .unwrap();
}

/// Native decisions shared by Forever 70707 and Retail 70688 (R17).
#[test]
fn duration_binding_native_empty_state_and_typed_setters_are_atomic() {
    let env = binding_environment();
    env.exec(
        r#"
        local binding = C_DurationUtil.CreateDurationTextBinding()
        assert(binding:GetDuration() == nil, 'fresh binding has no duration')
        assert(not binding:CanFormatText() and binding:GetFormattedText() == '')
        binding:SetZeroDurationText('unset')
        assert(not binding:CanFormatText() and binding:GetFormattedText() == 'unset')
        local duration = CreateBindingDuration(6)
        binding:SetDuration(duration)
        assert(not binding:CanFormatText(), 'duration alone is not sufficient')
        binding:SetTextFormat('remaining %s', {})
        assert(binding:CanFormatText(), 'duration and nonempty format are sufficient')
        local formatter = CreateBindingFormatter('%.0f')
        binding:SetFormatter(formatter)
        assert(binding:GetFormattedText() == '6', 'SetFormatter resets the prior format')
        assert(not pcall(binding.SetDuration, binding, nil))
        assert(binding:GetDuration() == duration and binding:GetFormattedText() == '6')
        local fake = newproxy(true)
        getmetatable(fake).__index = {FormatNumber=function() return 'forged' end}
        for _, value in ipairs({{}, {FormatNumber=function() return 'forged' end}, fake}) do
            assert(not pcall(binding.SetFormatter, binding, value), 'formatter identity is authenticated')
            assert(binding:GetFormattedText() == '6', 'rejected formatter preserves configuration')
        end
        assert(not pcall(binding.SetFormatter, binding, nil))
        binding:SetFormatter(formatter:Copy())
        assert(binding:GetFormattedText() == '6')
        binding:Assign(binding)
        assert(binding:GetDuration() == duration and binding:GetFormattedText() == '6')
        binding:SetToDefaults()
        assert(binding:GetDuration() == nil and not binding:CanFormatText())
        assert(binding:GetFormattedText() == '' and not binding:CanUpdateFontString())
        "#,
    )
    .expect("R17 native empty/reset state, predicate and atomic typed setter behavior");
}

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
        assert(secretunwrap(binding:GetFormattedText()) == 'remaining 18')
        binding:UpdateFontString()
        assert(secretunwrap(label:GetText()) == 'remaining 18')
        local function untrusted()
            assert(binding:HasSecretValues())
            assert(not pcall(binding.GetFormattedText, binding))
            assert(not pcall(binding.UpdateFontString, binding))
            assert(not pcall(label.GetText, label))
        end
        debug.setobjecttaint(untrusted, 'BindingProbe')
        untrusted()
        assert(secretunwrap(label:GetText()) == 'remaining 18')
        local shorter = C_DurationUtil.CreateDuration()
        shorter:SetTimeFromStart(secretwrap(0, 12, 1))
        shorter:SetClock(C_DurationUtil.CreateManualClock(0))
        binding:SetDuration(shorter)
        assert(binding:HasSecretValues())
        assert(secretunwrap(binding:GetFormattedText()) == 'remaining 12')
        binding:SetToDefaults()
        assert(not binding:HasSecretValues())
    "#,
    )
    .expect("secret duration and numeric input retain secrecy across formatting and text output");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn duration_binding_secret_formatter_copy_preserves_output_aspect() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeFromStart(secretwrap(10, 20, 1))
        duration:SetClock(C_DurationUtil.CreateManualClock(12))
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, format='%.0f'}})
        local binding = C_DurationUtil.CreateDurationTextBinding()
        binding:SetDuration(duration)
        binding:SetFormatter(formatter:Copy())
        local text = binding:GetFormattedText()
        assert(issecretvalue(text) and secretunwrap(text) == '18')
    "#,
    )
    .expect("an authentic formatter copy formats secret duration text without exposing it");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn duration_binding_secret_format_errors_do_not_replace_font_string_text() {
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
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, format='%.0f'}})
        binding:SetFormatter(formatter)
        binding:SetTextFormat('%d')
        assert(not pcall(binding.GetFormattedText, binding))
        assert(not pcall(binding.UpdateFontString, binding))
        assert(label:GetText() == 'unchanged')
    "#,
    )
    .expect("format errors propagate before changing the target text");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn duration_binding_secret_formatting_does_not_use_addon_conversion_overrides() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local binding = C_DurationUtil.CreateDurationTextBinding()
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeSpan(secretwrap(0, 18))
        binding:SetDuration(duration)
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, format='%.0f'}})
        binding:SetFormatter(formatter)
        binding:SetTextFormat('remaining %s')
        local originals = {type, tostring, tonumber, string.format}
        local leaked = false
        local function spy(original)
            local function replacement(...)
                for index = 1, select('#', ...) do
                    local value = select(index, ...)
                    if value == 18 or value == '18' then leaked = true end
                end
                return original(...)
            end
            debug.setobjecttaint(replacement, 'ConversionProbe')
            return replacement
        end
        type, tostring, tonumber, string.format = spy(type), spy(tostring), spy(tonumber), spy(string.format)
        local ok, value = pcall(binding.GetFormattedText, binding)
        type, tostring, tonumber, string.format = unpack(originals)
        assert(ok, tostring(value))
        assert(secretunwrap(value) == 'remaining 18')
        assert(not leaked, 'secret conversion passed timing into an addon override')
    "#).expect("bootstrap conversions keep decoded timing away from later addon overrides");
}

#[test]
fn duration_binding_rejects_full_lockdown_font_string_from_xml_scope() {
    let env = WowLuaEnv::new().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let toc = directory.path().join("LockdownProbe.toc");
    std::fs::write(&toc, "## Title: LockdownProbe\nfixture.xml\n").unwrap();
    std::fs::write(
        directory.path().join("fixture.xml"),
        r#"<Ui>
            <ScopedModifier fullLockdown="true">
                <Frame name="DurationLockdownParent" parent="UIParent"/>
            </ScopedModifier>
            <Frame name="DurationLockdownChild" parent="DurationLockdownParent">
                <Layers><Layer level="OVERLAY">
                    <FontString name="$parentText" parentKey="Text" inherits="GameFontNormal"/>
                </Layer></Layers>
            </Frame>
        </Ui>"#,
    )
    .unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        local label = DurationLockdownChild.Text
        assert(DurationLockdownParent:IsForbidden() and DurationLockdownChild:IsForbidden())
        assert(label:IsForbidden(), 'fully locked parent state reaches a FontString created later')
        local binding = C_DurationUtil.CreateDurationTextBinding()
        local function rejected(value)
            local ok, err = pcall(binding.SetFontString, binding, value)
            assert(not ok and err == 'Attempt to access fully locked-down object.', tostring(err))
        end
        rejected(label)
        rejected(GetForbiddenObjectTable(label))
        local addon = function(value) rejected(value) end
        debug.setobjecttaint(addon, 'FullLockdownProbe')
        addon(label)
        "#,
    )
    .expect("R18 full-lockdown state is inherited at creation and rejects all FontString views");
}

const BINDING_FIXTURES: &str = r#"
    function CreateBindingDuration(value)
        local duration = C_DurationUtil.CreateDuration()
        duration:SetClock(C_DurationUtil.CreateManualClock(0))
        duration:SetTimeFromStart(0, value)
        return duration
    end
    function CreateBindingFormatter(format)
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, format=format}})
        return formatter
    end
"#;

fn binding_environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(BINDING_FIXTURES).unwrap();
    env
}

#[test]
fn duration_binding_userdata_copy_retains_resources_through_collection() {
    let env = binding_environment();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDurationTextBinding()
        assert(type(source) == 'userdata')
        assert(not pcall(rawget, source, 'duration'))
        assert(not pcall(rawset, source, 'duration', 99))

        local duration = C_DurationUtil.CreateDuration()
        local clock = C_DurationUtil.CreateManualClock(12)
        local label = CreateFrame('Frame'):CreateFontString()
        local formatter = CreateBindingFormatter('retained:%.0f')
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
        retained:SetDuration(CreateBindingDuration(17))
        retained:UpdateFontString()
        assert(resources[3]:GetText() == 'retained:17')
        resources[4]:SetBreakpoints({{threshold=0, format='shared:%.0f'}})
        retained:UpdateFontString()
        assert(resources[3]:GetText() == 'shared:17')
        "#,
    )
    .expect("userdata binding retains identity and copied resource handles through collection");
}

#[test]
fn duration_binding_assign_preserves_receiver_and_configuration_handles() {
    let env = binding_environment();
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
        target:SetDuration(CreateBindingDuration(8.2))
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
    let env = binding_environment();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDurationTextBinding()
        local label = CreateFrame('Frame'):CreateFontString()
        source:SetFontString(label)
        source:SetDuration(CreateBindingDuration(7))
        source:SetFormatter(CreateBindingFormatter('%.0f'))
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
    let env = binding_environment();
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
        copy:SetDuration(CreateBindingDuration(3.2))
        copy:UpdateFontString()
        assert(secondLabel:GetText() == '4')
        assert(source:GetFontString() == firstLabel and not source:IsEnabled())
        source:SetDuration(CreateBindingDuration(8.2))
        source:UpdateFontString()
        assert(firstLabel:GetText() == '9' and secondLabel:GetText() == '4')
        source:SetFormatter(CreateBindingFormatter('source:%.0f'))
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
    let env = binding_environment();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDurationTextBinding()
        local assigned = C_DurationUtil.CreateDurationTextBinding()
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold=0, format='%.0f'}})
        local property = Enum.DurationTextBindingProperty.RemainingDuration
        local components = {{property=property, formatter=formatter}}
        source:SetDuration(CreateBindingDuration(7))
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
    let env = binding_environment();
    env.exec(
        r#"
        local binding = C_DurationUtil.CreateDurationTextBinding()
        local label = CreateFrame('Frame'):CreateFontString()
        local duration = CreateBindingDuration(6)
        binding:SetDuration(duration)
        binding:SetFormatter(CreateBindingFormatter('%.0f'))
        binding:SetFontString(label)
        binding:SetTextFormat('kept %s', {})
        binding:SetExpiredText('expired')
        binding:SetZeroDurationText('zero')
        assert(select('#', binding:Assign(binding)) == 0)
        assert(binding:GetDuration() == duration and binding:GetFontString() == label)
        assert(binding:GetFormattedText() == 'kept 6')
        local invalid = {17, 'invalid', {}, {GetDuration=function() return 99 end}}
        assert(not pcall(binding.Assign, binding, nil))
        for _, value in ipairs(invalid) do
            assert(not pcall(binding.Assign, binding, value))
            assert(binding:GetDuration() == duration and binding:GetFormattedText() == 'kept 6')
        end
        assert(not pcall(binding.Assign, {}, binding), 'receiver must also be a binding')
        assert(not pcall(binding.Copy, {}), 'Copy validates its receiver')
        binding:UpdateFontString()
        assert(label:GetText() == 'kept 6')

        local empty = C_DurationUtil.CreateDurationTextBinding()
        binding:Assign(empty)
        assert(binding:GetDuration() == empty:GetDuration(), 'Assign copies empty duration')
        assert(binding:GetFontString() == nil and not binding:CanUpdateFontString(), 'Assign clears FontString configuration')
        assert(binding:GetExpiredText() == nil and binding:GetZeroDurationText() == nil, 'Assign clears fallback text')
        assert(binding:GetTextColorCurve() == nil, 'Assign clears color curve')
        assert(not binding:CanFormatText() and not empty:CanFormatText(), 'empty bindings cannot format text')
        -- R17: native Forever 70707 and Retail 70688 return empty text here.
        assert(binding:GetFormattedText() == '', 'assigned empty binding returns empty text')
        assert(empty:GetFormattedText() == '', 'new empty binding returns empty text')
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
                env.exec(BINDING_FIXTURES).unwrap();
                env.exec(
                    r#"
                    local formatter = C_StringUtil.CreateNumericRuleFormatter()
                    formatter:SetBreakpoints({{threshold=0, step=1, rounding=Enum.NumericRuleFormatRounding.Up, format='%.0f'}})
                    local originalLabel = CreateFrame('Frame'):CreateFontString()
                    local source = C_DurationUtil.CreateDurationTextBinding()
                    source:SetDuration(CreateBindingDuration(2.2))
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
                            local privateButton = GetForbiddenObjectTable(button)
                            local stored = privateButton.durationText.element
                            assert(button:GetDurationText() == label and stored == GetForbiddenObjectTable(label))
                            local ownedBinding = privateButton:GetDurationTextBinding()
                            assert(ownedBinding ~= source and ownedBinding:GetFontString() == stored)
                            assert(ownedBinding:GetUpdateInterval() == 0.25)
                            assert(ownedBinding:GetTextColorCurve() == curve)
                            local r,g,b,a = curve:Evaluate(30):GetRGBA()
                            assert(r == 1 and g == 1 and b == 1 and a == 1)
                            r,g,b,a = curve:Evaluate(60):GetRGBA()
                            assert(r == 1 and math.abs(g - 0.82) < 0.00001 and b == 0 and a == 1)
                            ownedBinding:SetDuration(CreateBindingDuration(8.2))
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
