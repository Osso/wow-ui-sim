//! Common documented NumericFormatter consumption; simulator policy, not native parity.
#![cfg(all(feature = "retail-12-0-5", feature = "numeric-rule-formatters"))]
use wow_ui_sim::lua_api::WowLuaEnv;

const SETUP: &str = r#"
    clock = C_DurationUtil.CreateManualClock(300)
    d = C_DurationUtil.CreateDuration()
    d:SetClock(clock)
    d:SetTimeFromStart(0, 2468, 2)
    abbreviated = C_StringUtil.CreateAbbreviatedNumberFormatter()
    numeric = C_StringUtil.CreateNumericRuleFormatter()
    numeric:SetBreakpoints({{threshold=0, format='%.0f ticks'}})
    seconds = C_StringUtil.CreateSecondsFormatter()
    seconds:SetDesiredUnitCount(2)
    seconds:SetDefaultAbbreviation(2)
    formatters = {abbreviated, numeric, seconds}
    methods = {'FormatTotalDuration', 'FormatElapsedDuration', 'FormatRemainingDuration'}
"#;

fn execute(code: &str) {
    let env = WowLuaEnv::new().unwrap();
    env.exec(SETUP).expect("real formatter setup");
    env.exec(if cfg!(feature = "native-duration-formatting") {
        "nativeSeconds = true"
    } else {
        "nativeSeconds = false"
    })
    .unwrap();
    env.exec(code).expect("common duration formatter behavior");
}

#[test]
fn all_three_formatters_render_real_elapsed_remaining_and_base_modifiers() {
    execute(
        r#"
        local real = {{'1.2k', '300', '934'}, {'1234 ticks', '300 ticks', '934 ticks'},
            {'20m 34s', '5m', '15m 34s'}}
        local base = {{'2.4k', '600', '1.8k'}, {'2468 ticks', '600 ticks', '1868 ticks'},
            {'41m 8s', '10m', '31m 8s'}}
        if not nativeSeconds then
            real[3] = {'1234', '300', '934'}
            base[3] = {'2468', '600', '1868'}
        end
        for index, formatter in ipairs(formatters) do
            for column, name in ipairs(methods) do
                local output = d[name](d, formatter)
                assert(output == real[index][column], name .. ': ' .. tostring(output))
                assert(not issecretvalue(output))
                assert(select('#', d[name](d, formatter)) == 1)
                assert(d[name](d, formatter, 0) == real[index][column])
                assert(d[name](d, formatter, 1) == base[index][column])
            end
        end
        clock:SetTime(1500)
        assert(d:FormatRemainingDuration(seconds) == (nativeSeconds and '0s' or '0'))
        assert(d:FormatElapsedDuration(numeric) == '1234 ticks')
        local original = string.format
        string.format = function() error('arbitrary replacement printf') end
        assert(d:FormatTotalDuration(numeric) == '1234 ticks')
        string.format = original
        local metatable = debug.getmetatable(numeric)
        local originalMethod = metatable.FormatNumber
        metatable.FormatNumber = function() error('arbitrary replacement formatter') end
        assert(d:FormatTotalDuration(numeric) == '1234 ticks')
        metatable.FormatNumber = originalMethod
    "#,
    );
}

#[test]
fn invalid_receivers_modifiers_and_formatter_errors_leave_state_unchanged() {
    execute(
        r#"
        local calls = 0
        local fake = setmetatable({FormatNumber=function() calls=calls+1; return 'fake' end},
            {__index=function() calls=calls+1; error('untrusted lookup') end})
        for _, name in ipairs(methods) do
            for _, bad in ipairs({fake, newproxy(), newproxy(seconds), false, 42, 'formatter'}) do
                assert(not pcall(d[name], d, bad))
            end
            for _, formatter in ipairs(formatters) do
                assert(not pcall(d[name], d, formatter, 2))
                assert(not pcall(d[name], d, formatter, {}))
            end
        end
        assert(calls == 0, 'unknown formatter must never be inspected or called')
        numeric:ClearBreakpoints()
        assert(not pcall(d.FormatTotalDuration, d, numeric))
        assert(#numeric:GetBreakpoints() == 0)
        seconds:SetDesiredUnitCount(0)
        if nativeSeconds then assert(not pcall(d.FormatTotalDuration, d, seconds)) end
        assert(seconds.desiredUnitCount == 0)
        seconds:SetDesiredUnitCount(2)
        assert(d:GetTotalDuration() == 1234 and d:GetElapsedDuration() == 300)
        assert(d:FormatTotalDuration(abbreviated) == '1.2k')
        assert(d:FormatTotalDuration(seconds) == (nativeSeconds and '20m 34s' or '1234'))
    "#,
    );
}

#[test]
fn opaque_duration_modifier_and_formatter_inputs_preserve_secrecy_and_caller_guards() {
    execute(
        r#"
        local secretDuration = d:Copy()
        secretDuration:SetTimeFromStart(secretwrap(0), 2468, 2)
        local secretModifier = secretwrap(1)
        local expected = {{'1.2k','300','934'}, {'1234 ticks','300 ticks','934 ticks'},
            {'20m 34s','5m','15m 34s'}}
        if not nativeSeconds then expected[3] = {'1234', '300', '934'} end
        local wrapped = {}
        for index, formatter in ipairs(formatters) do
            wrapped[index] = secretwrap(formatter)
            for column, name in ipairs(methods) do
                local text = secretDuration[name](secretDuration, formatter)
                assert(issecretvalue(text) and secretunwrap(text) == expected[index][column])
                text = d[name](d, wrapped[index])
                assert(issecretvalue(text) and secretunwrap(text) == expected[index][column])
                text = d[name](d, formatter, secretModifier)
                assert(issecretvalue(text))
                assert(secretunwrap(text) == formatter:FormatNumber(d[name:gsub('Format','Get')](d, 1)))
            end
        end
        local function tainted()
            assert(not issecure())
            for index, formatter in ipairs(formatters) do
                for column, name in ipairs(methods) do
                    assert(not pcall(d[name], secretDuration, formatter))
                    assert(not pcall(d[name], d, wrapped[index]))
                    assert(not pcall(d[name], d, formatter, secretModifier))
                    assert(d[name](d, formatter) == expected[index][column])
                end
            end
            assert(not pcall(secretwrap, 1) and not issecure())
        end
        debug.setobjecttaint(tainted, 'DurationNumericProbe')
        tainted()
        assert(d:GetTotalDuration() == 1234)
    "#,
    );
}

#[test]
fn duration_getter_overrides_cannot_receive_decoded_secret_modifiers() {
    execute(
        r#"
        local modifier = secretwrap(1)
        local calls = 0
        rawset(d, 'GetTotalDuration', function(_, input)
            calls = calls + 1
            assert(issecretvalue(input), 'decoded modifier disclosed to Lua getter')
            error('untrusted getter invoked')
        end)
        local expected = {'2.4k', '2468 ticks', nativeSeconds and '41m 8s' or '2468'}
        for index, formatter in ipairs(formatters) do
            local result = d:FormatTotalDuration(formatter, modifier)
            assert(issecretvalue(result) and secretunwrap(result) == expected[index])
        end
        assert(calls == 0, 'duration Format must use trusted core queries')
        rawset(d, 'GetTotalDuration', nil)
        assert(d:GetTotalDuration() == 1234)
        local previous = debug.getmetatable(0)
        debug.setmetatable(0, {__tostring=function()
            error('decoded number disclosed to conversion callback')
        end})
        for index, formatter in ipairs(formatters) do
            local result = d:FormatTotalDuration(formatter, modifier)
            assert(issecretvalue(result) and secretunwrap(result) == expected[index])
        end
        debug.setmetatable(0, previous)
    "#,
    );
}

#[cfg(feature = "native-duration-formatting")]
#[test]
fn seconds_curve_receives_opaque_time_and_retains_closure_taint() {
    execute(
        r#"
        local secretDuration = d:Copy()
        secretDuration:SetTimeFromStart(secretwrap(0), 2468, 2)
        local seen = 0
        seconds:SetMaxIntervalCurve({Evaluate=function(_, input)
            seen = seen + 1
            assert(issecretvalue(input), 'curve must not receive decoded duration')
            assert(secretunwrap(input) == 1234)
            return 0
        end})
        local result = secretDuration:FormatTotalDuration(seconds)
        assert(issecretvalue(result) and secretunwrap(result) == '1234s' and seen == 1)
        local function taintedCurve(_, input)
            assert(not issecure() and issecretvalue(input))
            assert(not pcall(secretunwrap, input))
            assert(not pcall(secretwrap, 1))
            error('tainted curve marker')
        end
        debug.setobjecttaint(taintedCurve, 'DurationCurveProbe')
        seconds:SetMaxIntervalCurve({Evaluate=taintedCurve})
        local ok, message = pcall(secretDuration.FormatTotalDuration, secretDuration, seconds)
        assert(not ok and tostring(message):find('tainted curve marker', 1, true))
        assert(secretDuration:HasSecretValues())
        seconds:SetMaxInterval(3)
        assert(d:FormatTotalDuration(seconds) == '20m 34s')
    "#,
    );
}
