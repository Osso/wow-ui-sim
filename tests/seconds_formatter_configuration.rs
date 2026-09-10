use wow_ui_sim::lua_api::WowLuaEnv;

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
