use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn seconds_formatter_evaluation_uses_current_configuration() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = C_StringUtil.CreateSecondsFormatter()
        local second = C_StringUtil.CreateSecondsFormatter()
        local interval = Enum.SecondsFormatterInterval
        assert(interval.Seconds == 0 and interval.Minutes == 1)
        assert(interval.Hours == 2 and interval.Days == 3)
        for _, seconds in ipairs({-1, 0, 0.25, 59.999, 60, 3600, 86400}) do
            assert(first:CanApproximate(seconds) == false)
            assert(first:EvaluateMinInterval(seconds) == interval.Seconds)
            assert(first:EvaluateMaxInterval(seconds) == interval.Days)
            assert(first:EvaluateDesiredUnitCount(seconds) == 1)
        end
        first:SetApproximationSeconds(2.5)
        for _, case in ipairs({{-1, false}, {0, false}, {0.25, true},
                               {2.499, true}, {2.5, false}, {2.501, false}}) do
            assert(first:CanApproximate(case[1]) == case[2])
        end
        first:SetMinInterval(interval.Minutes)
        first:SetMaxInterval(interval.Hours)
        first:SetDesiredUnitCount(2)
        first:SetMillisecondsThreshold(0.5)
        for _, seconds in ipairs({0, 0.25, 60, 86400}) do
            assert(first:EvaluateMinInterval(seconds) == interval.Minutes)
            assert(first:EvaluateMaxInterval(seconds) == interval.Hours)
            assert(first:EvaluateDesiredUnitCount(seconds) == 2)
            assert(second:EvaluateMinInterval(seconds) == interval.Seconds)
            assert(second:EvaluateMaxInterval(seconds) == interval.Days)
            assert(second:EvaluateDesiredUnitCount(seconds) == 1)
        end
        first:SetApproximationSeconds(0)
        assert(not first:CanApproximate(0.1))
        first:SetApproximationSeconds(-1)
        assert(not first:CanApproximate(0.1))
        first:SetDesiredUnitCount(3)
        assert(first:EvaluateDesiredUnitCount(0.25) == 3)
        for _, method in ipairs({first.CanApproximate, first.EvaluateMinInterval,
                                  first.EvaluateMaxInterval, first.EvaluateDesiredUnitCount}) do
            assert(select('#', method(first, 0.25)) == 1)
        end
        assert(first:Format(2.5) == '2.5') -- Existing placeholder stays unchanged.
        "#,
    )
    .unwrap();
}

#[test]
fn seconds_formatter_evaluation_resolves_live_curve_methods_without_fallback() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local formatter = C_StringUtil.CreateSecondsFormatter()
        local other = C_StringUtil.CreateSecondsFormatter()
        local interval = Enum.SecondsFormatterInterval
        local curve = C_CurveUtil.CreateCurve()
        curve:AddPoint(0, interval.Seconds)
        curve:AddPoint(60, interval.Minutes)
        formatter:SetMaxInterval(interval.Days)
        formatter:SetMaxIntervalCurve(curve)
        assert(formatter:EvaluateMaxInterval(0) == interval.Seconds)
        assert(formatter:EvaluateMaxInterval(60) == interval.Minutes)
        assert(formatter:EvaluateMaxInterval(120) == interval.Minutes)
        assert(other:EvaluateMaxInterval(0) == interval.Days)
        -- Fractional curve results cannot silently become the static maximum.
        local ok, err = pcall(formatter.EvaluateMaxInterval, formatter, 30)
        assert(not ok and tostring(err):find('interval', 1, true))
        curve:ClearPoints()
        curve:AddPoint(0, interval.Hours)
        collectgarbage('collect')
        assert(formatter:EvaluateMaxInterval(0.25) == interval.Hours)
        formatter:SetMaxInterval(interval.Minutes)
        assert(formatter:EvaluateMaxInterval(0.25) == interval.Minutes)
        formatter:SetMaxIntervalCurve(curve)
        assert(formatter:EvaluateMaxInterval(0.25) == interval.Hours)
        local observed
        local proxy = setmetatable({}, {__index = {
            Evaluate = function(self, seconds) observed = seconds; return interval.Seconds end,
        }})
        formatter:SetMaxIntervalCurve(proxy)
        assert(formatter:EvaluateMaxInterval(12.5) == interval.Seconds and observed == 12.5)
        for _, invalid in ipairs({false, {}, {Evaluate = false},
                {Evaluate = function() return nil end},
                {Evaluate = function() return 4 end},
                {Evaluate = function() return 0/0 end}}) do
            formatter:SetMaxIntervalCurve(invalid)
            assert(not pcall(formatter.EvaluateMaxInterval, formatter, 12.5))
        end
        formatter:SetMaxIntervalCurve({Evaluate = function() error('curve failure marker') end})
        ok, err = pcall(formatter.EvaluateMaxInterval, formatter, 12.5)
        assert(not ok and tostring(err):find('curve failure marker', 1, true))
        formatter:SetMaxIntervalCurve(nil)
        assert(formatter:EvaluateMaxInterval(12.5) == interval.Minutes)
        "#,
    )
    .unwrap();
}

#[test]
fn seconds_formatter_evaluation_rejects_invalid_inputs_without_mutation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetApproximationSeconds(5)
        formatter:SetMinInterval(1)
        formatter:SetMaxInterval(2)
        formatter:SetDesiredUnitCount(2)
        for _, method in ipairs({formatter.CanApproximate, formatter.EvaluateMinInterval,
                                  formatter.EvaluateMaxInterval, formatter.EvaluateDesiredUnitCount}) do
            for _, invalid in ipairs({false, {}, '1', math.huge, -math.huge, 0/0}) do
                assert(not pcall(method, formatter, invalid))
            end
            assert(not pcall(method, formatter))
            assert(not pcall(method, {}, 1))
        end
        assert(formatter:GetApproximationSeconds() == 5)
        assert(formatter:EvaluateMinInterval(1) == 1)
        assert(formatter:EvaluateMaxInterval(1) == 2)
        assert(formatter:EvaluateDesiredUnitCount(1) == 2)
        for _, value in ipairs({-1, 0.5, 4, false, '1'}) do
            formatter:SetMinInterval(value)
            assert(not pcall(formatter.EvaluateMinInterval, formatter, 1))
            formatter:SetMaxInterval(value)
            assert(not pcall(formatter.EvaluateMaxInterval, formatter, 1))
        end
        for _, count in ipairs({0, -1, 1.5, false, '2', math.huge}) do
            formatter:SetDesiredUnitCount(count)
            assert(not pcall(formatter.EvaluateDesiredUnitCount, formatter, 1))
        end
        formatter:SetMinInterval(0)
        formatter:SetMaxInterval(3)
        formatter:SetDesiredUnitCount(1)
        assert(formatter:EvaluateMinInterval(1) == 0)
        assert(formatter:EvaluateMaxInterval(1) == 3)
        assert(formatter:EvaluateDesiredUnitCount(1) == 1)
        "#,
    )
    .unwrap();
}

#[test]
fn seconds_formatter_configuration_is_independent_and_has_exact_arity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = C_StringUtil.CreateSecondsFormatter()
        local second = C_StringUtil.CreateSecondsFormatter()
        assert(first ~= second)
        for _, formatter in ipairs({first, second}) do
            assert(formatter:GetApproximationSeconds() == 0)
            assert(formatter:GetMillisecondsThreshold() == 0)
            assert(select('#', formatter:GetApproximationSeconds()) == 1)
            assert(select('#', formatter:GetMillisecondsThreshold()) == 1)
        end
        for _, value in ipairs({2.75, 0, -1.5, 42.125}) do
            assert(select('#', first:SetApproximationSeconds(value)) == 0)
            assert(first:GetApproximationSeconds() == value)
            assert(first:GetMillisecondsThreshold() == 0)
            assert(second:GetApproximationSeconds() == 0)
            assert(select('#', second:SetMillisecondsThreshold(value)) == 0)
            assert(second:GetMillisecondsThreshold() == value)
            assert(second:GetApproximationSeconds() == 0)
        end
        first:SetMillisecondsThreshold(-0.25)
        second:SetApproximationSeconds(9.5)
        assert(first:GetApproximationSeconds() == 42.125)
        assert(first:GetMillisecondsThreshold() == -0.25)
        assert(second:GetApproximationSeconds() == 9.5)
        assert(second:GetMillisecondsThreshold() == 42.125)
        "#,
    )
    .unwrap();
}

#[test]
fn seconds_formatter_configuration_rejects_invalid_values_atomically() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local formatter = C_StringUtil.CreateSecondsFormatter()
        formatter:SetApproximationSeconds(12.5)
        formatter:SetMillisecondsThreshold(-3.25)
        local setters = {formatter.SetApproximationSeconds, formatter.SetMillisecondsThreshold}
        for _, setter in ipairs(setters) do
            for _, invalid in ipairs({false, {}, "3", math.huge, -math.huge, 0/0}) do
                local ok, err = pcall(setter, formatter, invalid)
                assert(not ok and tostring(err):find("finite number", 1, true))
                assert(formatter:GetApproximationSeconds() == 12.5)
                assert(formatter:GetMillisecondsThreshold() == -3.25)
            end
            assert(not pcall(setter, formatter))
            assert(not pcall(setter, formatter, nil))
            assert(formatter:GetApproximationSeconds() == 12.5)
            assert(formatter:GetMillisecondsThreshold() == -3.25)
        end
        "#,
    )
    .unwrap();
}

#[test]
fn seconds_formatter_configuration_survives_gc_and_preserves_existing_methods() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        RetainedSecondsFormatter = C_StringUtil.CreateSecondsFormatter()
        RetainedSecondsFormatter:SetApproximationSeconds(0.125)
        RetainedSecondsFormatter:SetMillisecondsThreshold(-8.5)
        "#,
    )
    .unwrap();
    env.exec("collectgarbage('collect'); collectgarbage('collect')")
        .unwrap();
    env.exec(
        r#"
        local formatter = RetainedSecondsFormatter
        assert(formatter:GetApproximationSeconds() == 0.125)
        assert(formatter:GetMillisecondsThreshold() == -8.5)
        local curve = C_CurveUtil.CreateCurve()
        curve:AddPoint(0, 10)
        formatter:SetDefaultAbbreviation(2)
        formatter:SetRounding(1)
        formatter:SetCanRoundUpLastUnit(true)
        formatter:SetMinInterval(3)
        formatter:SetMaxInterval(4)
        formatter:SetMaxIntervalCurve(curve)
        formatter:SetDesiredUnitCount(2)
        assert(formatter.defaultAbbreviation == 2 and formatter.rounding == 1)
        assert(formatter.canRoundUpLastUnit == true)
        assert(formatter.minInterval == 3 and formatter.maxInterval == 4)
        assert(formatter.maxIntervalCurve == curve and formatter.desiredUnitCount == 2)
        -- Preserve the existing placeholder, not a native formatting claim.
        assert(formatter:Format(2.5) == "2.5")
        assert(formatter:GetApproximationSeconds() == 0.125)
        assert(formatter:GetMillisecondsThreshold() == -8.5)
        assert(C_StringUtil.CreateSecondsFormatter():GetApproximationSeconds() == 0)
        "#,
    )
    .unwrap();
}
