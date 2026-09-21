-- EllesmereUI 9.2.2 AuraKit BuildRuleDurationFormatter(false) breakpoints.
assert(type(C_StringUtil.CreateNumericRuleFormatter) == 'function', 'native numeric-rule formatter constructor missing')
local rounding = assert(Enum.NumericRuleFormatRounding)
local formatter = C_StringUtil.CreateNumericRuleFormatter()
formatter:SetBreakpoints({
    { threshold = 0, format = '%d', step = 1, rounding = rounding.Up },
    { threshold = 60, format = '%dm', step = 1, rounding = rounding.Up, components = {{ div = 60 }} },
    { threshold = 61, format = '%dm', step = 1, rounding = rounding.Down, components = {{ div = 60 }} },
    { threshold = 3600, format = '%dh', step = 1, rounding = rounding.Down, components = {{ div = 3600 }} },
    { threshold = 86400, format = '%dd', step = 1, rounding = rounding.Down, components = {{ div = 86400 }} },
})
for _, case in ipairs({
    {10, '10'}, {10.1, '11'}, {59, '59'}, {59.9, '60'},
    {60, '1m'}, {60.9, '1m'}, {61, '1m'}, {119.9, '1m'},
    {120, '2m'}, {3599.9, '59m'}, {3600, '1h'}, {86400, '1d'},
}) do
    local actual = formatter:FormatNumber(case[1])
    assert(actual == case[2], tostring(case[1]) .. ': expected ' .. case[2] .. ', got ' .. actual)
end
-- Current model selects the rule before rounding: 59.9 stays in the seconds
-- bucket. This asserts that existing contract, not unmeasured native parity.
print('ELLESMERE_NUMERIC_RULE_FORMATTER_GREEN')
