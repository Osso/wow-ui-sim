//! Real engine tick -> library countdown text consumer, not API-only coverage.
//! Clock/modRate, attachment lifetime, gates, and output policy are simulator guesses.

use super::{cooldown_countdown_text, cooldown_remaining_seconds};
use crate::lua_api::WowLuaEnv;
use std::time::{Duration, Instant};

fn setup(code: &str) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec("cooldown = CreateFrame('Cooldown', 'FormatterRenderCooldown'); cooldown:GetCountdownFontString()")
        .expect("real Cooldown with countdown child");
    env.exec(code)
        .expect("configure real formatter and cooldown");
    env
}

fn tick_at(env: &WowLuaEnv, seconds: f64) -> Option<String> {
    env.state().borrow_mut().start_time = Instant::now() - Duration::from_secs_f64(seconds);
    env.fire_on_update(0.125).expect("real engine tick");
    let sim = env.state().borrow();
    let id = sim
        .widgets
        .get_id_by_name("FormatterRenderCooldown")
        .unwrap();
    let cooldown = sim.widgets.get(id).unwrap();
    let remaining = cooldown_remaining_seconds(cooldown, sim.start_time.elapsed().as_secs_f64())?;
    cooldown_countdown_text(cooldown, remaining)
}

#[cfg(feature = "numeric-rule-formatters")]
#[test]
fn configured_numeric_rule_renderer_ticks_mod_rate_and_live_rules() {
    let env = setup(
        r#"
        formatter = C_StringUtil.CreateNumericRuleFormatter()
        formatter:SetBreakpoints({{threshold = 0, format = '%.0f ticks'}})
        cooldown:SetCountdownFormatter(formatter)
        cooldown:SetCooldown(0, 10, 2)
    "#,
    );
    assert_eq!(tick_at(&env, 1.1).as_deref(), Some("8 ticks"));
    assert_eq!(tick_at(&env, 2.1).as_deref(), Some("6 ticks"));
    env.exec(
        r#"
        cooldown:SetScript('OnUpdate', function(self)
            formatter:SetBreakpoints({{threshold = 0, format = '%.0f turns'}})
            self:SetScript('OnUpdate', nil)
        end)
    "#,
    )
    .unwrap();
    assert_eq!(tick_at(&env, 2.1).as_deref(), Some("6 turns"));
}

#[test]
fn configured_abbreviated_renderer_uses_live_breakpoints() {
    let env = setup(
        r#"
        formatter = C_StringUtil.CreateAbbreviatedNumberFormatter()
        formatter:SetBreakpoints({{
            breakpoint = 1000, abbreviation = ' charges', abbreviationIsGlobal = false,
            significandDivisor = 100, fractionDivisor = 10,
        }})
        cooldown:SetCountdownFormatter(formatter)
        cooldown:SetCooldown(0, 1234.5)
    "#,
    );
    assert_eq!(tick_at(&env, 0.25).as_deref(), Some("1.2 charges"));
    env.exec(
        r#"
        formatter:SetBreakpoints({{
            breakpoint = 1000, abbreviation = ' stacks', abbreviationIsGlobal = false,
            significandDivisor = 100, fractionDivisor = 10,
        }})
    "#,
    )
    .unwrap();
    assert_eq!(tick_at(&env, 0.25).as_deref(), Some("1.2 stacks"));
}

#[test]
fn configured_native_seconds_renderer_ticks_and_live_unit_configuration() {
    let env = setup(
        r#"
        formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetDesiredUnitCount(2)
        formatter:SetDefaultAbbreviation(2)
        cooldown:SetCountdownFormatter(formatter)
        cooldown:SetCooldown(0, 1234.5)
    "#,
    );
    assert_eq!(tick_at(&env, 0.25).as_deref(), Some("20m 34s"));
    assert_eq!(tick_at(&env, 60.25).as_deref(), Some("19m 34s"));
    env.exec("formatter:SetDesiredUnitCount(1)").unwrap();
    assert_eq!(tick_at(&env, 60.25).as_deref(), Some("19m"));
}

#[test]
fn configured_renderer_clear_restores_existing_default_thresholds() {
    let env = setup(
        r#"
        formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetDefaultAbbreviation(2)
        cooldown:SetCountdownFormatter(formatter)
        cooldown:SetCountdownAbbrevThreshold(5)
        cooldown:SetCountdownMillisecondsThreshold(3)
        cooldown:SetCooldown(0, 10)
    "#,
    );
    assert_eq!(tick_at(&env, 1.25).as_deref(), Some("8s"));
    env.exec("cooldown:SetCountdownFormatter(nil)").unwrap();
    assert_eq!(tick_at(&env, 1.25).as_deref(), Some("9s"));
    assert_eq!(tick_at(&env, 7.6).as_deref(), Some("2.4"));
    env.exec("cooldown:SetCountdownFormatter(formatter)")
        .unwrap();
    assert_eq!(tick_at(&env, 1.25).as_deref(), Some("8s"));
}

#[test]
fn configured_renderer_preserves_hide_minimum_and_expiry_gates() {
    let env = setup(
        r#"
        formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetDefaultAbbreviation(2)
        cooldown:SetCountdownFormatter(formatter)
        cooldown:SetCooldown(0, 10)
    "#,
    );
    assert_eq!(tick_at(&env, 1.25).as_deref(), Some("8s"));
    env.exec("cooldown:SetHideCountdownNumbers(true)").unwrap();
    assert_eq!(tick_at(&env, 1.25), None);
    env.exec("cooldown:SetHideCountdownNumbers(false); cooldown:SetMinimumCountdownDuration(11)")
        .unwrap();
    assert_eq!(tick_at(&env, 1.25), None);
    env.exec("cooldown:SetMinimumCountdownDuration(0)").unwrap();
    assert_eq!(tick_at(&env, 1.25).as_deref(), Some("8s"));
    assert_eq!(tick_at(&env, 11.0), None);
    env.exec("cooldown:Clear()").unwrap();
    assert_eq!(tick_at(&env, 1.25), None);
}

#[test]
fn configured_renderer_roots_handle_after_caller_release_and_collection() {
    let env = setup(
        r#"
        local formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetDesiredUnitCount(2)
        formatter:SetDefaultAbbreviation(2)
        cooldown:SetCountdownFormatter(formatter)
        cooldown:SetCooldown(0, 1234.5)
        formatter = nil
        collectgarbage('collect')
        assert(cooldown:GetCountdownFormatter():FormatNumber(1234) == '20m 34s')
    "#,
    );
    assert_eq!(tick_at(&env, 0.25).as_deref(), Some("20m 34s"));
    env.exec("cooldown:GetCountdownFormatter():SetDesiredUnitCount(1); collectgarbage('collect')")
        .unwrap();
    assert_eq!(tick_at(&env, 0.25).as_deref(), Some("20m"));
}

#[test]
fn configured_renderer_ignores_public_frame_formatter_callbacks_for_secret_timing() {
    let env = setup(
        r#"
        formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetDefaultAbbreviation(2)
        cooldown:SetCountdownFormatter(formatter)
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeFromStart(secretwrap(0), secretwrap(10))
        cooldown:SetCooldownFromDurationObject(duration)
        calls = 0
        local impostor = {FormatNumber = function(_, value)
            calls = calls + 1
            error('addon callback received countdown input')
        end}
        cooldown.formatter = impostor
        cooldown.__countdown_formatter = impostor
        cooldown.GetCountdownFormatter = function() return impostor end
        assert(issecure())
    "#,
    );
    assert_eq!(tick_at(&env, 1.25).as_deref(), Some("8s"));
    env.exec("assert(calls == 0 and issecure())").unwrap();
}
