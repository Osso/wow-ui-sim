//! Public Cooldown formatter configuration only; no countdown rendering proof.
//! Retail cache FrameAPICooldownDocumentation.lua: GetCountdownFormatter (89),
//! SetCountdownFormatter (363), NumericFormatter nilable, AllowedWhenUntainted.
//! Identity retention, nil clearing, atomic rejection, and ordinary getter output
//! after a typed-secret setter are inferred simulator policy, not native probes.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn execute(code: &str) {
    let env = WowLuaEnv::new().unwrap();
    env.exec("cooldown = CreateFrame('Cooldown')")
        .expect("real Cooldown creation");
    env.exec(code).expect("Cooldown formatter configuration");
}

#[test]
fn countdown_formatter_defaults_to_one_nil_result() {
    execute(
        r#"
        assert(cooldown:GetCountdownFormatter() == nil)
        assert(select('#', cooldown:GetCountdownFormatter()) == 1)
        local other = CreateFrame('Cooldown')
        assert(other:GetCountdownFormatter() == nil)
        "#,
    );
}

#[test]
fn countdown_formatter_retains_configured_abbreviated_object_identity() {
    execute(
        r#"
        local formatter = C_StringUtil.CreateAbbreviatedNumberFormatter()
        formatter:SetBreakpoints({{
            breakpoint = 1000, abbreviation = ' charges', abbreviationIsGlobal = false,
            significandDivisor = 100, fractionDivisor = 10,
        }})
        cooldown:SetCountdownFormatter(formatter)
        local retained = cooldown:GetCountdownFormatter()
        assert(rawequal(retained, formatter), 'must retain actual formatter, not a copy')
        assert(retained:FormatNumber(1234) == '1.2 charges')
        formatter:ClearBreakpoints()
        assert(retained:FormatNumber(1234) == '1234', 'retained configuration must stay live')
        local other = CreateFrame('Cooldown')
        assert(other:GetCountdownFormatter() == nil, 'configuration is per Cooldown')
        "#,
    );
}

#[cfg(feature = "numeric-rule-formatters")]
#[test]
fn countdown_formatter_retains_configured_numeric_rule_object_identity() {
    execute(
        r#"
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold = 0, format = '%.0f ticks'}})
        cooldown:SetCountdownFormatter(formatter)
        local retained = cooldown:GetCountdownFormatter()
        assert(rawequal(retained, formatter))
        assert(retained:FormatNumber(1234) == '1234 ticks')
        formatter:SetBreakpoints({{threshold = 0, format = '%.1f turns'}})
        assert(retained:FormatNumber(1234) == '1234.0 turns')
        "#,
    );
}

#[test]
fn countdown_formatter_retains_configured_native_seconds_object_identity() {
    execute(
        r#"
        local formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetDesiredUnitCount(2)
        formatter:SetDefaultAbbreviation(2)
        cooldown:SetCountdownFormatter(formatter)
        local retained = cooldown:GetCountdownFormatter()
        assert(rawequal(retained, formatter))
        assert(retained:FormatNumber(1234) == '20m 34s')
        formatter:SetDesiredUnitCount(1)
        assert(retained:FormatNumber(1234) == '20m')
        "#,
    );
}

#[test]
fn clearing_countdown_formatter_resets_attachment_without_mutating_objects() {
    execute(
        r#"
        local first = C_StringUtil.CreateAbbreviatedNumberFormatter()
        local second = C_StringUtil.CreateSecondsFormatter()
        second:SetDesiredUnitCount(2)
        second:SetDefaultAbbreviation(2)
        cooldown:SetCountdownAbbrevThreshold(90)
        cooldown:SetCountdownMillisecondsThreshold(5)
        cooldown:SetCountdownFormatter(first)
        assert(rawequal(cooldown:GetCountdownFormatter(), first))
        cooldown:SetCountdownFormatter(second)
        assert(rawequal(cooldown:GetCountdownFormatter(), second))
        cooldown:SetCountdownFormatter(nil)
        assert(cooldown:GetCountdownFormatter() == nil)
        assert(select('#', cooldown:GetCountdownFormatter()) == 1)
        assert(first:FormatNumber(1234) == '1.2k')
        assert(second:FormatNumber(1234) == '20m 34s')
        assert(cooldown:GetCountdownAbbrevThreshold() == 90)
        assert(cooldown:GetCountdownMillisecondsThreshold() == 5)
        cooldown:SetCountdownFormatter(first)
        assert(rawequal(cooldown:GetCountdownFormatter(), first))
        "#,
    );
}

#[test]
fn invalid_countdown_formatter_inputs_leave_existing_attachment_unchanged() {
    execute(
        r#"
        local formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetDesiredUnitCount(2)
        formatter:SetDefaultAbbreviation(2)
        cooldown:SetCountdownFormatter(formatter)
        local calls = 0
        local impostor = setmetatable({FormatNumber = function()
            calls = calls + 1
            return 'not a formatter'
        end}, {__index = function()
            calls = calls + 1
            error('untrusted formatter lookup')
        end})
        local invalid = {
            false, true, 1234, 'formatter', {}, impostor,
            newproxy(), newproxy(formatter), CreateFrame('Frame'),
            C_DurationUtil.CreateDuration(), secretwrap(1234), secretwrap({}),
        }
        for _, input in ipairs(invalid) do
            assert(not pcall(cooldown.SetCountdownFormatter, cooldown, input))
            assert(rawequal(cooldown:GetCountdownFormatter(), formatter))
            assert(formatter:FormatNumber(1234) == '20m 34s')
        end
        assert(calls == 0, 'invalid input must not execute addon callbacks')
        "#,
    );
}

#[test]
fn untainted_countdown_formatter_setter_accepts_typed_secret_object_and_nil() {
    execute(
        r#"
        local formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetDesiredUnitCount(2)
        formatter:SetDefaultAbbreviation(2)
        cooldown:SetCountdownFormatter(formatter)
        local wrapped = secretwrap(formatter)
        assert(issecure() and issecretvalue(wrapped))
        cooldown:SetCountdownFormatter(wrapped)
        local retained = cooldown:GetCountdownFormatter()
        assert(not issecretvalue(retained))
        assert(rawequal(retained, formatter))
        assert(retained:FormatNumber(1234) == '20m 34s')
        assert(issecretvalue(wrapped), 'setter must not mutate secret argument')
        cooldown:SetCountdownFormatter(secretwrap(nil))
        assert(cooldown:GetCountdownFormatter() == nil)
        assert(formatter:FormatNumber(1234) == '20m 34s')
        "#,
    );
}

#[test]
fn tainted_countdown_formatter_setter_rejects_secrets_atomically_but_accepts_plain_values() {
    execute(
        r#"
        local initial = C_StringUtil.CreateAbbreviatedNumberFormatter()
        local replacement = C_StringUtil.CreateSecondsFormatter()
        replacement:SetDesiredUnitCount(2)
        replacement:SetDefaultAbbreviation(2)
        cooldown:SetCountdownFormatter(initial)
        local wrapped = secretwrap(replacement)
        local wrappedNil = secretwrap(nil)
        local function tainted()
            assert(not issecure())
            assert(not pcall(cooldown.SetCountdownFormatter, cooldown, wrapped))
            assert(rawequal(cooldown:GetCountdownFormatter(), initial))
            assert(not pcall(cooldown.SetCountdownFormatter, cooldown, wrappedNil))
            assert(rawequal(cooldown:GetCountdownFormatter(), initial))
            cooldown:SetCountdownFormatter(replacement)
            assert(rawequal(cooldown:GetCountdownFormatter(), replacement))
            assert(replacement:FormatNumber(1234) == '20m 34s')
            cooldown:SetCountdownFormatter(nil)
            assert(cooldown:GetCountdownFormatter() == nil)
            assert(debug.getstacktaint() == 'CooldownFormatterProbe')
        end
        debug.setobjecttaint(tainted, 'CooldownFormatterProbe')
        tainted()
        assert(cooldown:GetCountdownFormatter() == nil)
        assert(issecretvalue(wrapped) and issecretvalue(wrappedNil))
        "#,
    );
}
