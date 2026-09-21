#![cfg(feature = "numeric-rule-formatters")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn numeric_rule_formatter_formats_ellesmere_aura_duration_breakpoints() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(include_str!("fixtures/ellesmere_duration_formatter.lua"))
        .expect("Ellesmere AuraKit preferred numeric duration formatter");
}

#[test]
fn numeric_rule_formatter_formats_resource_bar_countdown_and_rounding_modes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local rounding = Enum.NumericRuleFormatRounding
        assert(rounding.Nearest == 0 and rounding.Up == 1 and rounding.Down == 2)
        local metadata = EnumMeta.NumericRuleFormatRounding
        assert(metadata.MinValue == 0 and metadata.MaxValue == 2 and metadata.NumValues == 3)
        assert(__secureenv.Enum.NumericRuleFormatRounding.Up == 1)
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        assert(type(formatter) == 'userdata')
        formatter:SetBreakpoints({{threshold = 0, step = 1, rounding = rounding.Up, format = '%.0f'}})
        assert(formatter:FormatNumber(0) == '0')
        assert(formatter:FormatNumber(1) == '1')
        assert(formatter:FormatNumber(1.01) == '2')
        assert(formatter:FormatNumber(8.99) == '9')
        local modes = {
            { mode = rounding.Nearest, value = 1.12, expected = '1.00' },
            { mode = rounding.Nearest, value = 1.14, expected = '1.25' },
            { mode = rounding.Up, value = 1.12, expected = '1.25' },
            { mode = rounding.Down, value = 1.24, expected = '1.00' },
        }
        for _, case in ipairs(modes) do
            formatter:SetBreakpoints({{threshold = 0, step = 0.25, rounding = case.mode, format = '%.2f'}})
            assert(formatter:FormatNumber(case.value) == case.expected)
        end
        formatter:SetBreakpoints({{threshold = 0, step = 1, format = '%.0f'}})
        assert(formatter:FormatNumber(1.2) == '1', 'omitted rounding defaults to Nearest')
        assert(formatter:FormatNumber(1.8) == '2')
        "#,
    )
    .expect("native numeric countdown formatting");
}

#[test]
fn numeric_rule_formatter_selects_threshold_before_rounding_and_clamps_afterward() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        local up = Enum.NumericRuleFormatRounding.Up
        formatter:SetBreakpoints({
            {threshold = 100, format = 'large %.0f'},
            {threshold = 0, step = 10, rounding = up, format = 'small %.0f'},
            {threshold = 10, format = 'medium %.1f'},
        })
        assert(formatter:FormatNumber(9.1) == 'small 10')
        assert(formatter:FormatNumber(10) == 'medium 10.0')
        assert(formatter:FormatNumber(99) == 'medium 99.0')
        assert(formatter:FormatNumber(100) == 'large 100')
        formatter:SetBreakpoints({{threshold = 0, step = 10, rounding = up, min = 12, max = 18, format = '%.0f'}})
        assert(formatter:FormatNumber(1) == '12', 'minimum clamp follows step rounding')
        assert(formatter:FormatNumber(13) == '18', 'maximum clamp follows step rounding')
        formatter:SetBreakpoints({{threshold = 0, format = 'ready %%'}})
        assert(formatter:FormatNumber(77) == 'ready %', 'constant format needs no numeric conversion')
        "#,
    )
    .expect("threshold selection and transformation ordering");
}

#[test]
fn numeric_rule_formatter_components_apply_division_modulo_then_rounding() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        local down = Enum.NumericRuleFormatRounding.Down
        formatter:SetBreakpoints({{
            threshold = 0, step = 1, rounding = down, format = '%02d:%02d',
            components = {
                {div = 60, step = 1, rounding = down},
                {mod = 60, step = 1, rounding = down},
            },
        }})
        assert(formatter:FormatNumber(125.9) == '02:05')
        assert(formatter:FormatNumber(59.9) == '00:59')
        formatter:SetBreakpoints({{
            threshold = 0, format = '%.0f',
            components = {{div = 10, mod = 6, step = 1, rounding = down}},
        }})
        assert(formatter:FormatNumber(125) == '0', '(125 / 10) modulo 6 is rounded last')
        "#,
    )
    .expect("documented component ordering");
}

#[test]
fn numeric_rule_formatter_copies_its_configuration_and_replaces_or_clears_rules() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        assert(#formatter:GetBreakpoints() == 0)
        local input = {{threshold = 0, format = '%.1f', components = {{div = 10}}}}
        formatter:SetBreakpoints(input)
        input[1].format = 'changed'
        input[1].components[1].div = 1
        assert(formatter:FormatNumber(25) == '2.5', 'configuration owns its component data')
        local returned = formatter:GetBreakpoints()
        returned[1].components[1].div = 1
        returned[1].format = 'changed'
        assert(formatter:FormatNumber(25) == '2.5', 'readback is a copy')
        local copy = formatter:Copy()
        assert(copy ~= formatter and type(copy) == 'userdata')
        formatter:AddBreakpoint({threshold = 100, format = 'hundreds %.0f'})
        assert(formatter:FormatNumber(100) == 'hundreds 100')
        assert(copy:FormatNumber(100) == '10.0')
        formatter:ClearBreakpoints()
        assert(#formatter:GetBreakpoints() == 0)
        assert(#copy:GetBreakpoints() == 1 and copy:FormatNumber(25) == '2.5')
        copy:SetBreakpoints({{threshold = 0, format = '%.0f'}})
        assert(#copy:GetBreakpoints() == 1 and copy:FormatNumber(25) == '25')
        "#,
    )
    .expect("independent state, copy, replace, add and clear");
}

#[test]
fn numeric_rule_formatter_updates_duration_binding_text() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold = 0, step = 1, rounding = Enum.NumericRuleFormatRounding.Up, format = '%.0f'}})
        local parent = CreateFrame('Frame')
        local label = parent:CreateFontString()
        local binding = C_DurationUtil.CreateDurationTextBinding()
        binding:SetDuration(1.2)
        binding:SetFontString(label)
        binding:SetFormatter(formatter)
        assert(binding:GetFormattedText() == '2')
        binding:UpdateFontString()
        assert(label:GetText() == '2')
        binding:SetDuration(8.2)
        binding:UpdateFontString()
        assert(label:GetText() == '9')
        local copy = formatter:Copy()
        copy:SetBreakpoints({{threshold = 0, step = 1, rounding = Enum.NumericRuleFormatRounding.Down, format = '%.0f'}})
        binding:SetFormatter(copy)
        assert(binding:GetFormattedText() == '8')
        binding:SetFormatter({Format = function(_, value) return 'custom:' .. value end})
        assert(binding:GetFormattedText() == 'custom:8.2', 'custom table formatter contract is unchanged')
        "#,
    )
    .expect("native formatter feeds the duration binding consumer");
}

#[test]
fn numeric_rule_formatter_rejects_invalid_rules_without_replacing_valid_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold = 0, format = '%.0f'}})
        local invalid = {
            {threshold = 0, step = 0, format = '%.0f'},
            {threshold = 0, step = 1, rounding = 3, format = '%.0f'},
            {threshold = 0, format = '%.0f %.0f'},
            {threshold = 0, format = '%s'},
            {threshold = 0, format = '%.0f', components = {{div = 0}}},
            {threshold = 0, format = '%.0f', components = {{mod = 0}}},
            {threshold = 0, min = 10, max = 1, format = '%.0f'},
        }
        for _, rule in ipairs(invalid) do
            assert(not pcall(formatter.SetBreakpoints, formatter, {rule}))
            assert(formatter:FormatNumber(25) == '25', 'invalid replacement keeps existing configuration')
        end
        -- Explicit model limitations, not claims about undocumented native edge policies.
        assert(not pcall(formatter.AddBreakpoint, formatter, {threshold = 0, format = '%.0f'}))
        formatter:ClearBreakpoints()
        assert(not pcall(formatter.FormatNumber, formatter, 25))
        formatter:SetBreakpoints({{threshold = 10, format = '%.0f'}})
        assert(not pcall(formatter.FormatNumber, formatter, 9))
        formatter:SetBreakpoints({{threshold = 0, step = 1, format = '%.0f'}})
        assert(not pcall(formatter.FormatNumber, formatter, 1.5))
        "#,
    )
    .expect("invalid formatting configuration is explicit and transactional");
}
