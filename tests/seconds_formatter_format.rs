//! Modeled PTR duration formatting; ICU data and policy are not native WoW proof.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn seconds_formatter_format_selects_units_and_decomposes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local f = C_StringUtil.CreateSecondsFormatter()
        local cases = {{59.4, '59 seconds'}, {60, '1 minute'}, {90, '1 minute'},
            {3599, '59 minutes'}, {3600, '1 hour'}, {86400, '1 day'},
            {0, '0 seconds'}, {-90, '-1 minute'}}
        for _, c in ipairs(cases) do
            assert(f:Format(c[1]) == c[2], f:Format(c[1]))
            assert(select('#', f:Format(c[1])) == 1)
        end
        f:SetDesiredUnitCount(2)
        assert(f:Format(90) == '1 minute, 30 seconds', f:Format(90))
        assert(f:Format(3599) == '59 minutes, 59 seconds')
        assert(f:Format(3600) == '1 hour')
        f:SetDesiredUnitCount(4)
        assert(f:Format(90061) == '1 day, 1 hour, 1 minute, 1 second')
        f:SetMinInterval(1)
        f:SetMaxInterval(1)
        assert(f:Format(3599) == '59 minutes')
        f:SetMinInterval(0)
        f:SetMaxInterval(0)
        assert(f:Format(90) == '90 seconds')
        f:SetMaxInterval(3)
        f:SetMaxIntervalCurve({Evaluate = function(_, s) assert(s == 90); return 0 end})
        assert(f:Format(90) == '90 seconds')
        f:SetMaxIntervalCurve({Evaluate = function() error('curve marker') end})
        local ok, err = pcall(f.Format, f, 90)
        assert(not ok and tostring(err):find('curve marker', 1, true))
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn seconds_formatter_format_rounds_final_unit_and_handles_thresholds() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local f = C_StringUtil.CreateSecondsFormatter()
        f:SetRounding(Enum.SecondsFormatterRounding.RoundUp)
        assert(f:Format(59.4) == '1 minute', f:Format(59.4))
        assert(f:Format(3599) == '1 hour')
        f:SetDesiredUnitCount(2)
        assert(f:Format(3599.1) == '1 hour')
        f:SetCanRoundUpLastUnit(false)
        assert(f:Format(59.4) == '59 seconds')
        f:SetRounding(Enum.SecondsFormatterRounding.Truncate)
        f:SetCanRoundUpLastUnit(true)
        assert(f:Format(59.4) == '59 seconds')
        f:SetApproximationSeconds(1)
        assert(f:Format(0.25) == '< 1 second')
        assert(f:Format(1) == '1 second')
        assert(f:Format(0) == '0 seconds')
        assert(f:Format(-0.25) == '-0 seconds')
        f:SetApproximationSeconds(0)
        f:SetMillisecondsThreshold(1)
        assert(f:Format(0.125) == '0.125 seconds', f:Format(0.125))
        assert(f:Format(0.9999) == '0.999 seconds')
        assert(f:Format(1.5) == '1 second')
        assert(f:Format(-0.125) == '-0.125 seconds')
        f:SetRounding(0)
        assert(f:Format(0.9999) == '1 second')
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn seconds_formatter_format_uses_locale_width_and_survives_gc() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local f = C_StringUtil.CreateSecondsFormatter()
        local other = C_StringUtil.CreateSecondsFormatter()
        f:SetDesiredUnitCount(2)
        f:SetDefaultAbbreviation(1)
        assert(f:Format(90) == '1m 30s', f:Format(90))
        assert(f:Format(90, 0) == '1 minute, 30 seconds')
        assert(f:Format(90, 2) == '1 min, 30 sec', f:Format(90, 2))
        assert(f:Format(90, 3) == '1 minute, 30 seconds')
        assert(other:Format(90) == '1 minute')
        collectgarbage('collect'); collectgarbage('collect')
        assert(f:Format(90) == '1m 30s')
        local old = GetLocale
        GetLocale = function() return 'deDE' end
        assert(other:Format(1) == '1 Sekunde', other:Format(1))
        assert(other:Format(2) == '2 Sekunden', other:Format(2))
        GetLocale = function() return 'en-US-u-nu-arab' end
        assert(other:Format(2):find('٢', 1, true))
        GetLocale = old
        assert(other:Format(2) == '2 seconds')
        -- No private renderer is published as a helper on public tables.
        assert(rawget(C_StringUtil, 'FormatDurationUnits') == nil)
        assert(rawget(_G, '__wow_render_duration_units') == nil)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn seconds_formatter_format_rejects_invalid_state_without_mutating_it() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local f = C_StringUtil.CreateSecondsFormatter()
        for _, value in ipairs({false, '1', {}, math.huge, -math.huge, 0/0}) do
            assert(not pcall(f.Format, f, value))
        end
        assert(not pcall(f.Format, f))
        assert(not pcall(f.Format, {}, 1))
        for _, value in ipairs({-1, 4, 0.5, '1', false}) do
            assert(not pcall(f.Format, f, 1, value))
        end
        f:SetMinInterval(3); f:SetMaxInterval(0)
        assert(not pcall(f.Format, f, 1))
        f:SetMinInterval(0); f:SetMaxInterval(3)
        f:SetRounding(9); assert(not pcall(f.Format, f, 1))
        f:SetRounding(1)
        f:SetCanRoundUpLastUnit('yes'); assert(not pcall(f.Format, f, 1))
        f:SetCanRoundUpLastUnit(true)
        f:SetDesiredUnitCount(0); assert(not pcall(f.Format, f, 1))
        f:SetDesiredUnitCount(1)
        f:SetMaxIntervalCurve({Evaluate = function() return 0.5 end})
        assert(not pcall(f.Format, f, 1))
        f:SetMaxInterval(3)
        assert(f:Format(1) == '1 second')
        assert(f:GetApproximationSeconds() == 0 and f:GetMillisecondsThreshold() == 0)
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn seconds_formatter_format_preserves_retail_placeholder_without_native_api() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local f = C_StringUtil.CreateSecondsFormatter()
        for _, seconds in ipairs({59.4,60,90,3599,3600,86400,0,-90}) do
            assert(f:Format(seconds) == tostring(seconds))
        end
        f:SetMinInterval(3); f:SetMaxInterval(0); f:SetRounding(99)
        assert(f:Format(2.5) == '2.5')
        assert(f:Format() == '0')
        assert(C_Intl == nil)
        assert(rawget(C_StringUtil, 'FormatDurationUnits') == nil)
    "#).unwrap();
}
