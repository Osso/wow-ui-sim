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

#[test]
fn duration_copy_preserves_timing_and_returns_one_independent_object() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDuration()
        source:SetTimeFromStart(10, 20, 2)
        local function check_return(...)
            assert(select('#', ...) == 1, 'Copy must return exactly one value')
            return ...
        end
        local copy = check_return(source:Copy())
        assert(type(copy) == 'table' and copy ~= source and not rawequal(copy, source))
        assert(copy:GetStartTime() == 10 and copy:GetEndTime() == 20, 'Copy timing')
        assert(copy:GetModRate() == 2 and copy:GetTotalDuration() == 10)
        assert(copy:GetTotalDuration(Enum.DurationTimeModifier.BaseTime) == 20)
        source:SetTimeFromStart(30, 12, 3)
        assert(copy:GetStartTime() == 10 and copy:GetEndTime() == 20)
        assert(copy:GetModRate() == 2 and copy:GetTotalDuration() == 10)
        copy:SetTimeFromStart(50, 24, 4)
        assert(source:GetStartTime() == 30 and source:GetEndTime() == 34)
        assert(source:GetModRate() == 3 and source:GetTotalDuration() == 4)
        assert(source:GetTotalDuration(Enum.DurationTimeModifier.BaseTime) == 12)
        assert(copy:GetStartTime() == 50 and copy:GetEndTime() == 56)
        "#,
    )
    .expect("Copy preserves configured timing without sharing mutable duration state");
}

#[test]
fn duration_copy_assign_preserves_receiver_identity_and_independent_timing() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local source = C_DurationUtil.CreateDuration()
        source:SetTimeFromStart(10, 20, 2)
        local target = C_DurationUtil.CreateDuration()
        target:SetTimeFromStart(40, 9, 3)
        -- Receiver identity/custom-field retention is simulator policy, not native evidence.
        local identity, marker = target, {}
        target.marker = marker
        source.marker = 'source marker'
        assert(select('#', target:Assign(source)) == 0, 'Assign returns no values')
        assert(rawequal(target, identity) and target.marker == marker)
        assert(target:GetStartTime() == 10 and target:GetEndTime() == 20, 'Assign timing')
        assert(target:GetModRate() == 2 and target:GetTotalDuration() == 10)
        assert(target:GetTotalDuration(Enum.DurationTimeModifier.BaseTime) == 20)
        source:SetTimeFromStart(30, 12, 3)
        assert(target:GetStartTime() == 10 and target:GetEndTime() == 20)
        assert(target:GetModRate() == 2 and target:GetTotalDuration() == 10)
        target:SetTimeFromStart(50, 24, 4)
        assert(source:GetStartTime() == 30 and source:GetEndTime() == 34)
        assert(source:GetModRate() == 3 and source:GetTotalDuration() == 4)
        assert(source:GetTotalDuration(Enum.DurationTimeModifier.BaseTime) == 12)
        assert(target:GetStartTime() == 50 and target:GetEndTime() == 56)
        assert(rawequal(target, identity) and target.marker == marker)
        "#,
    )
    .expect("Assign updates timing while preserving receiver identity and custom fields");
}

#[test]
fn duration_copy_assign_self_keeps_state_and_returns_nothing() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local duration = C_DurationUtil.CreateDuration()
        local clock = C_DurationUtil.CreateManualClock(15)
        duration:SetTimeFromStart(10, 20, 2)
        duration:SetClock(clock)
        -- Clock/custom-field retention is simulator policy, not native evidence.
        local marker = {}
        duration.marker = marker
        assert(select('#', duration:Assign(duration)) == 0)
        assert(duration:GetStartTime() == 10 and duration:GetEndTime() == 20)
        assert(duration:GetModRate() == 2 and duration:GetTotalDuration() == 10)
        assert(duration:GetTotalDuration(Enum.DurationTimeModifier.BaseTime) == 20)
        assert(duration:GetClock() == clock and duration:GetClockTime() == 15)
        assert(duration:GetElapsedDuration() == 5 and duration:GetRemainingDuration() == 5)
        assert(duration.marker == marker)
        "#,
    )
    .expect("self-assignment preserves duration state and returns no values");
}

#[test]
fn duration_copy_shares_clock_reference_but_rebinds_independently() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        -- Clock reference copying (not cloning) is simulator policy, not native evidence.
        local clock = C_DurationUtil.CreateManualClock(12)
        local source = C_DurationUtil.CreateDuration()
        source:SetTimeFromStart(10, 20, 2)
        source:SetClock(clock)
        local copy = source:Copy()
        assert(copy:GetClock() == clock, 'Copy must retain the clock reference')
        assert(copy:GetElapsedDuration() == 2 and source:GetElapsedDuration() == 2)
        clock:AdvanceTime(3)
        assert(copy:GetClockTime() == 15 and source:GetClockTime() == 15)
        assert(copy:GetRemainingDuration() == 5 and source:GetRemainingDuration() == 5)
        local replacement = C_DurationUtil.CreateManualClock(18)
        copy:SetClock(replacement)
        clock:AdvanceTime(1)
        assert(copy:GetClock() == replacement and source:GetClock() == clock)
        assert(copy:GetElapsedDuration() == 8 and source:GetElapsedDuration() == 6)
        "#,
    )
    .expect("Copy shares the clock object but not its binding slot");
}

#[test]
fn duration_copy_assign_shares_clock_reference_but_rebinds_independently() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        -- Clock reference copying (not cloning) is simulator policy, not native evidence.
        local clock = C_DurationUtil.CreateManualClock(12)
        local source = C_DurationUtil.CreateDuration()
        source:SetTimeFromStart(10, 20, 2)
        source:SetClock(clock)
        local target = C_DurationUtil.CreateDuration()
        target:SetClock(C_DurationUtil.CreateManualClock(99))
        target:Assign(source)
        assert(target:GetClock() == clock, 'Assign must replace the clock reference')
        assert(target:GetElapsedDuration() == 2 and source:GetElapsedDuration() == 2)
        clock:AdvanceTime(3)
        assert(target:GetClockTime() == 15 and source:GetClockTime() == 15)
        assert(target:GetRemainingDuration() == 5 and source:GetRemainingDuration() == 5)
        local replacement = C_DurationUtil.CreateManualClock(18)
        source:SetClock(replacement)
        clock:AdvanceTime(1)
        assert(source:GetClock() == replacement and target:GetClock() == clock)
        assert(source:GetElapsedDuration() == 8 and target:GetElapsedDuration() == 6)
        "#,
    )
    .expect("Assign shares the clock object but leaves bindings independently mutable");
}

#[test]
fn duration_copy_assign_unbound_source_clears_target_clock() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        -- Clearing the previous clock binding is simulator policy, not native evidence.
        local source = C_DurationUtil.CreateDuration()
        source:SetTimeFromStart(10, 20, 2)
        local target = C_DurationUtil.CreateDuration()
        target:SetClock(C_DurationUtil.CreateManualClock(15))
        assert(source:GetClock() == nil and target:GetClock() ~= nil)
        target:Assign(source)
        assert(target:GetClock() == nil, 'unbound source must clear target clock')
        assert(target:GetStartTime() == 10 and target:GetEndTime() == 20)
        assert(target:GetModRate() == 2 and target:GetTotalDuration() == 10)
        assert(target:GetTotalDuration(Enum.DurationTimeModifier.BaseTime) == 20)
        "#,
    )
    .expect("Assign replaces a bound receiver with the source's unbound clock state");
}

#[test]
fn duration_copy_assign_rejects_missing_or_wrong_source_without_mutation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local target = C_DurationUtil.CreateDuration()
        local clock = C_DurationUtil.CreateManualClock(15)
        target:SetTimeFromStart(10, 20, 2)
        target:SetClock(clock)
        -- Identity/custom-field and clock preservation are simulator policy.
        local identity, marker = target, {}
        target.marker = marker
        local invalidCalls = {
            function() target:Assign() end,
            function() target:Assign(nil) end,
            function() target:Assign({}) end,
            function() target:Assign({startTime = 10, duration = 20, modRate = 2}) end,
            function() target:Assign(clock) end,
            function() target:Assign(false) end,
            function() target:Assign(10) end,
            function() target:Assign('duration') end,
        }
        for index, call in ipairs(invalidCalls) do
            local ok = pcall(call)
            assert(not ok, 'Assign must reject invalid source case ' .. index)
            assert(rawequal(target, identity) and target.marker == marker)
            assert(target:GetStartTime() == 10 and target:GetEndTime() == 20)
            assert(target:GetModRate() == 2 and target:GetTotalDuration() == 10)
            assert(target:GetTotalDuration(Enum.DurationTimeModifier.BaseTime) == 20)
            assert(target:GetClock() == clock and target:GetClockTime() == 15)
            assert(target:GetElapsedDuration() == 5 and target:GetRemainingDuration() == 5)
        end
        "#,
    )
    .expect("Assign requires a duration source and leaves the receiver unchanged on error");
}
