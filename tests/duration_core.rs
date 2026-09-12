use wow_ui_sim::lua_api::WowLuaEnv;

// Fraction, zero-span, and validation expectations are simulator policies, not native proof.
#[test]
fn duration_percent_tracks_clock_boundaries_and_rewind() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local clock = C_DurationUtil.CreateManualClock(5)
        local d = C_DurationUtil.CreateDuration()
        d:SetClock(clock)
        d:SetTimeFromStart(10, 20, 2)
        local real = Enum.DurationTimeModifier.RealTime
        local base = Enum.DurationTimeModifier.BaseTime
        for _, sample in ipairs({
            {5, 0, 1}, {10, 0, 1}, {12.5, 0.25, 0.75},
            {15, 0.5, 0.5}, {20, 1, 0}, {25, 1, 0}, {12.5, 0.25, 0.75},
        }) do
            clock:SetTime(sample[1])
            local function fractions(modifier)
                local elapsed = d:GetElapsedPercent(modifier)
                local remaining = d:GetRemainingPercent(modifier)
                assert(type(elapsed) == 'number' and elapsed == sample[2], 'elapsed fraction')
                assert(type(remaining) == 'number' and remaining == sample[3], 'remaining fraction')
            end
            fractions(nil)
            fractions(real)
            fractions(base)
            assert(d:GetElapsedPercent() == sample[2])
            assert(d:GetRemainingPercent() == sample[3])
            assert(d:GetStartTime() == 10 and d:GetEndTime() == 20)
            assert(d:GetModRate() == 2 and d:GetClock() == clock)
            assert(d:GetClockTime() == sample[1])
            assert(d:GetTotalDuration() == 10 and d:GetTotalDuration(base) == 20)
            assert(d:GetElapsedDuration() == sample[2] * 10)
            assert(d:GetRemainingDuration() == sample[3] * 10)
            assert(d:GetElapsedDuration(base) == sample[2] * 20)
            assert(d:GetRemainingDuration(base) == sample[3] * 20)
            assert(not d:IsZero())
            assert(d:HasStarted() == (sample[1] >= 10))
            assert(d:HasExpired() == (sample[1] >= 20))
            assert(d:IsActive() == (sample[1] >= 10 and sample[1] < 20))
        end
    "#,
    )
    .expect("duration fractions follow the clock without changing configured timing");
}

#[test]
fn duration_percent_zero_spans_remain_zero() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local d = C_DurationUtil.CreateDuration()
        local clock = C_DurationUtil.CreateManualClock(15)
        local function zero()
            assert(d:GetElapsedPercent() == 0 and d:GetRemainingPercent() == 0)
            assert(d:GetElapsedPercent(nil) == 0 and d:GetRemainingPercent(nil) == 0)
            for _, modifier in ipairs({Enum.DurationTimeModifier.RealTime, Enum.DurationTimeModifier.BaseTime}) do
                assert(d:GetElapsedPercent(modifier) == 0)
                assert(d:GetRemainingPercent(modifier) == 0)
            end
            assert(d:IsZero() and d:GetTotalDuration() == 0)
            assert(d:GetElapsedDuration() == 0 and d:GetRemainingDuration() == 0)
        end
        zero()
        d:SetClock(clock)
        d:SetTimeFromStart(10, 20, 2)
        d:Reset()
        zero()
        assert(d:GetClock() == clock and d:GetModRate() == 1)
        assert(d:GetStartTime() == 0 and d:GetEndTime() == 0)
        d:SetTimeFromStart(10, 0, 2)
        zero()
        assert(d:GetClock() == clock and d:GetModRate() == 2)
        assert(d:GetStartTime() == 10 and d:GetEndTime() == 10)
    "#).expect("new, reset, and configured zero spans have zero fractions");
}

#[test]
fn duration_percent_rejects_unknown_modifier_without_mutation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local clock = C_DurationUtil.CreateManualClock(15)
        local d = C_DurationUtil.CreateDuration()
        d:SetClock(clock)
        d:SetTimeFromStart(10, 20, 2)
        for _, method in ipairs({'GetElapsedPercent', 'GetRemainingPercent'}) do
            assert(not pcall(d[method], d, 2), method .. ' must reject unknown modifier')
            assert(d:GetClock() == clock and d:GetClockTime() == 15)
            assert(d:GetStartTime() == 10 and d:GetEndTime() == 20)
            assert(d:GetModRate() == 2 and d:GetTotalDuration() == 10)
            assert(d:GetElapsedDuration() == 5 and d:GetRemainingDuration() == 5)
        end
    "#,
    )
    .expect("invalid percent modifiers reject without changing duration state");
}

#[test]
fn duration_percent_rejects_invalid_bound_clock_without_mutation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local clock = C_DurationUtil.CreateManualClock(15)
        local d = C_DurationUtil.CreateDuration()
        d:SetClock(clock)
        d:SetTimeFromStart(10, 20, 2)
        local function reject(time)
            clock.time = time
            for _, method in ipairs({'GetElapsedPercent', 'GetRemainingPercent'}) do
                assert(not pcall(d[method], d), method .. ' must reject invalid clock')
                assert(d:GetClock() == clock)
                assert(d:GetStartTime() == 10 and d:GetEndTime() == 20)
                assert(d:GetModRate() == 2 and d:GetTotalDuration() == 10)
                if type(time) == 'number' and time ~= time then
                    assert(clock.time ~= clock.time)
                else
                    assert(clock.time == time)
                end
            end
            clock:SetTime(15)
            assert(d:GetElapsedDuration() == 5 and d:GetRemainingDuration() == 5)
        end
        reject(nil)
        for _, time in ipairs({'15', false, {}, math.huge, -math.huge, 0/0}) do
            reject(time)
        end
    "#,
    )
    .expect("malformed and nonfinite clocks reject without changing timing or binding");
}

#[test]
fn duration_core_manual_progress_rate_and_rewind() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local clock = C_DurationUtil.CreateManualClock(8)
        local d = C_DurationUtil.CreateDuration()
        d:SetClock(clock)
        d:SetTimeFromStart(10, 20, 2)
        local base = Enum.DurationTimeModifier.BaseTime
        assert(d:GetStartTime() == 10 and d:GetEndTime() == 20)
        assert(d:GetModRate() == 2)
        assert(d:GetTotalDuration() == 10 and d:GetTotalDuration(base) == 20)
        assert(d:GetElapsedDuration() == 0 and d:GetRemainingDuration() == 10)
        assert(not d:IsZero() and not d:HasStarted())
        clock:SetTime(15)
        assert(d:GetClock() == clock and d:GetClockTime() == 15)
        assert(d:GetElapsedDuration() == 5 and d:GetElapsedDuration(base) == 10)
        assert(d:GetRemainingDuration() == 5 and d:GetRemainingDuration(base) == 10)
        assert(d:HasStarted() and d:IsActive() and not d:HasExpired())
        clock:AdvanceTime(10)
        assert(d:GetElapsedDuration() == 10 and d:GetRemainingDuration() == 0)
        assert(d:HasExpired() and not d:IsActive() and not d:IsZero())
        clock:RewindTime(10)
        assert(d:GetRemainingDuration() == 5 and d:IsActive())
    "#,
    )
    .unwrap();
}

#[test]
fn duration_core_end_span_reset_and_validation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local d = C_DurationUtil.CreateDuration()
        local c = C_DurationUtil.CreateManualClock(12)
        d:SetClock(c)
        d:SetTimeFromEnd(20, 20, 2)
        assert(d:GetStartTime() == 10 and d:GetEndTime() == 20)
        assert(d:GetRemainingDuration() == 8)
        d:SetTimeSpan(10, 30)
        assert(d:GetModRate() == 1 and d:GetTotalDuration() == 20)
        assert(d:GetElapsedDuration() == 2)
        for _, mutate in ipairs({
            function() d:SetTimeFromStart(1, -1) end,
            function() d:SetTimeFromEnd(1, 4, 0) end,
            function() d:SetTimeSpan(5, 4) end,
            function() d:SetTimeFromStart(0/0, 1) end,
            function() d:SetTimeFromStart(0, math.huge) end,
        }) do
            assert(not pcall(mutate))
            assert(d:GetStartTime() == 10 and d:GetEndTime() == 30)
        end
        d:Reset()
        assert(d:IsZero() and d:GetTotalDuration() == 0 and d:GetModRate() == 1)
        assert(d:GetStartTime() == 0 and d:GetEndTime() == 0)
        assert(d:GetClock() == c and d:GetClockTime() == 12)
        assert(not d:HasStarted() and not d:HasExpired() and not d:IsActive())
        d:SetTimeFromStart(10, 0)
        assert(d:IsZero() and d:GetRemainingDuration() == 0 and not d:IsActive())
        d:SetToDefaults()
        assert(d:GetClock() == nil and d:IsZero())
    "#,
    )
    .unwrap();
}

#[test]
fn duration_core_set_to_defaults_clears_configured_timing_and_clock() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local clock = C_DurationUtil.CreateManualClock(15)
        local d = C_DurationUtil.CreateDuration()
        d:SetClock(clock)
        d:SetTimeFromStart(10, 20, 2)
        assert(d:GetStartTime() == 10 and d:GetEndTime() == 20)
        assert(d:GetTotalDuration() == 10 and d:GetModRate() == 2)
        assert(d:GetClock() == clock)

        d:SetToDefaults()

        assert(d:GetStartTime() == 0 and d:GetEndTime() == 0)
        assert(d:GetTotalDuration() == 0 and d:GetModRate() == 1)
        assert(d:GetClock() == nil and d:IsZero())
        assert(not d:HasStarted() and not d:HasExpired() and not d:IsActive())
        assert(d:GetElapsedPercent() == 0 and d:GetRemainingPercent() == 0)
    "#,
    )
    .expect("SetToDefaults clears configured timing, rate, and clock binding");
}

#[test]
fn duration_core_default_clock_and_instances() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().start_time =
        std::time::Instant::now() - std::time::Duration::from_secs(40);
    env.exec(
        r#"
        local before = C_DurationUtil.GetCurrentTime()
        local d = C_DurationUtil.CreateDuration()
        assert(type(d) == 'table' and getmetatable(d) == false)
        d:SetTimeFromStart(30, 30)
        local now = d:GetClockTime()
        assert(before >= 40 and now >= before)
        assert(d:GetElapsedDuration() >= 10 and d:GetRemainingDuration() <= 20)
        local other = C_DurationUtil.CreateDuration()
        assert(other:IsZero() and other:GetTotalDuration() == 0)
        local clock = C_DurationUtil.CreateManualClock(35)
        d:SetClock(clock)
        assert(d:GetElapsedDuration() == 5)
        d:SetClock(nil)
        assert(d:GetElapsedDuration() >= 10)
    "#,
    )
    .unwrap();
}
