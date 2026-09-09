use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn duration_core_manual_progress_rate_and_rewind() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
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
    "#).unwrap();
}

#[test]
fn duration_core_end_span_reset_and_validation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
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
    "#).unwrap();
}

#[test]
fn duration_core_default_clock_and_instances() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().start_time = std::time::Instant::now() - std::time::Duration::from_secs(40);
    env.exec(r#"
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
    "#).unwrap();
}
