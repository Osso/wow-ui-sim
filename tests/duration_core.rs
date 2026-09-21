use wow_ui_sim::lua_api::WowLuaEnv;

// Secret lifecycle/access expectations below are explicit simulator guesses, not native proof.
#[cfg(feature = "client-wowforever")]
#[test]
fn duration_core_secret_inputs_store_wrappers_and_preserve_lifecycle() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local d = C_DurationUtil.CreateDuration()
        local clock = C_DurationUtil.CreateManualClock(25)
        d:SetClock(clock)
        assert(not d:HasSecretValues())
        d:SetTimeFromEnd(secretwrap(50, 30, nil))
        assert(d:HasSecretValues() and d:GetStartTime() == 20)
        assert(d:GetEndTime() == 50 and d:GetTotalDuration() == 30)
        assert(d:GetRemainingDuration() == 25 and d:GetModRate() == 1)
        for _, slot in ipairs({-1, -2, -3}) do
            assert(issecretvalue(rawget(d, slot)), 'timing must not leak through rawget')
        end
        local copy = d:Copy()
        local assigned = C_DurationUtil.CreateDuration()
        assigned:Assign(d)
        for _, other in ipairs({copy, assigned}) do
            assert(other ~= d and other:HasSecretValues())
            assert(other:GetClock() == clock and other:GetStartTime() == 20)
            for _, slot in ipairs({-1, -2, -3}) do
                assert(issecretvalue(rawget(other, slot)), 'copy must not expose plain timing')
            end
        end
        d:SetTimeFromStart(10, 20, 2)
        assert(d:HasSecretValues() and d:GetEndTime() == 20)
        d:SetTimeSpan(secretwrap(12), 18)
        assert(d:GetTotalDuration() == 6 and d:HasSecretValues())
        d:Reset()
        assert(d:IsZero() and d:HasSecretValues() and d:GetClock() == clock)
        for _, slot in ipairs({-1, -2, -3}) do assert(issecretvalue(rawget(d, slot))) end
        local plain = C_DurationUtil.CreateDuration()
        plain:SetTimeFromStart(3, 4)
        d:Assign(plain)
        assert(d:HasSecretValues() and d:GetStartTime() == 3)
        assert(d:GetClock() == nil and d:GetTotalDuration() == 4)
        d:SetToDefaults()
        assert(not d:HasSecretValues() and d:IsZero() and d:GetClock() == nil)
        for _, slot in ipairs({-1, -2, -3}) do assert(type(rawget(d, slot)) == 'number') end
        assert(copy:HasSecretValues() and copy:GetStartTime() == 20)
        assert(assigned:HasSecretValues() and assigned:GetTotalDuration() == 30)
    "#,
    )
    .expect("secret duration lifecycle preserves opaque timing until defaults clear it");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn duration_core_secret_tainted_queries_and_mutations_reject_atomically() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local d = C_DurationUtil.CreateDuration()
        local clock = C_DurationUtil.CreateManualClock(25)
        d:SetClock(clock)
        d:SetTimeFromStart(secretwrap(20, 30, 1))
        local wrapped = secretwrap(12)
        local plain = C_DurationUtil.CreateDuration()
        plain:SetTimeFromStart(2, 3)
        local saved = {rawget(d,-1), rawget(d,-2), rawget(d,-3)}
        local function tainted()
            assert(not issecure() and d:HasSecretValues())
            local copied = d:Copy()
            assert(copied:HasSecretValues())
            for _, name in ipairs({
                'GetStartTime', 'GetEndTime', 'GetTotalDuration', 'GetElapsedDuration',
                'GetRemainingDuration', 'GetElapsedPercent', 'GetRemainingPercent',
                'GetModRate', 'GetClockTime', 'IsZero', 'HasStarted', 'HasExpired', 'IsActive'
            }) do
                local ok = pcall(d[name], d)
                assert(not ok, name .. ' disclosed secret timing')
                assert(not pcall(copied[name], copied), name .. ' copy bypass')
            end
            for _, mutate in ipairs({
                function() d:SetTimeFromStart(1,2) end,
                function() d:SetTimeFromEnd(9,2) end,
                function() d:SetTimeSpan(1,2) end,
                function() plain:SetTimeFromStart(wrapped,2) end,
                function() plain:Assign(d) end,
                function() d:Assign(plain) end,
                function() d:Reset() end,
                function() d:SetToDefaults() end,
                function() d:SetClock({time=30}) end,
            }) do assert(not pcall(mutate), 'tainted mutation accepted') end
            assert(not plain:HasSecretValues() and plain:GetStartTime() == 2)
            assert(plain:GetTotalDuration() == 3)
            for index, slot in ipairs({-1,-2,-3}) do
                assert(rawget(d,slot) == saved[index], 'rejection mutated timing')
                assert(not pcall(secretunwrap, rawget(d,slot)), 'raw slot bypass')
            end
        end
        debug.setobjecttaint(tainted, 'DurationCoreProbe')
        tainted()
        assert(issecure() and d:GetStartTime() == 20 and d:GetTotalDuration() == 30)
        assert(d:GetClock() == clock and d:GetClockTime() == 25)
    "#,
    )
    .expect("tainted callers cannot disclose or mutate secret duration timing");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn duration_core_secret_invalid_arguments_leave_all_slots_unchanged() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local d = C_DurationUtil.CreateDuration()
        d:SetTimeFromStart(secretwrap(20,30,2))
        local saved = {rawget(d,-1),rawget(d,-2),rawget(d,-3)}
        local invalid = newproxy(true)
        for _, mutate in ipairs({
            function() d:SetTimeFromStart(invalid, 2) end,
            function() d:SetTimeFromEnd(secretwrap(40, -1, 1)) end,
            function() d:SetTimeFromStart(secretwrap(10, 20, 0)) end,
            function() d:SetTimeFromStart(secretwrap(10, 20, math.huge)) end,
            function() d:SetTimeFromEnd(secretwrap(0/0, 2, 1)) end,
            function() d:SetTimeSpan(secretwrap(5,4)) end,
            function() d:SetTimeFromStart(10,20,invalid) end,
        }) do
            assert(not pcall(mutate))
            assert(d:HasSecretValues())
            for index,slot in ipairs({-1,-2,-3}) do assert(rawget(d,slot) == saved[index]) end
            assert(d:GetStartTime() == 20 and d:GetTotalDuration() == 15 and d:GetModRate() == 2)
        end
        local plain = C_DurationUtil.CreateDuration()
        plain:SetTimeFromStart(1,2)
        assert(not pcall(plain.SetTimeFromStart, plain, secretwrap(3), -1))
        assert(not plain:HasSecretValues() and plain:GetStartTime() == 1)
        plain:SetTimeSpan(secretwrap(10,20))
        assert(plain:HasSecretValues() and plain:GetTotalDuration() == 10)
    "#,
    )
    .expect("invalid secret duration inputs preserve timing and secrecy atomically");
}

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

// These assertions exercise existing simulator clock, modifier, and curve policies.
// Native interpolation/extrapolation, coercion/errors, and secret behavior are unproven.
fn duration_curve_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        clock = C_DurationUtil.CreateManualClock(15)
        duration = C_DurationUtil.CreateDuration()
        duration:SetTimeFromStart(10, 20, 2)
        duration:SetClock(clock)
        secondsCurve = C_CurveUtil.CreateCurve()
        secondsCurve:AddPoint(0, 100)
        secondsCurve:AddPoint(20, 300)
        percentCurve = C_CurveUtil.CreateCurve()
        percentCurve:AddPoint(0, 2)
        percentCurve:AddPoint(1, 6)
        methods = {
            {'EvaluateElapsedDuration', secondsCurve},
            {'EvaluateRemainingDuration', secondsCurve},
            {'EvaluateElapsedPercent', percentCurve},
            {'EvaluateRemainingPercent', percentCurve},
        }
        function AssertDurationCurveState()
            assert(duration:GetStartTime() == 10 and duration:GetEndTime() == 20)
            assert(duration:GetModRate() == 2 and duration:GetClock() == clock)
            assert(secondsCurve:GetPointCount() == 2 and percentCurve:GetPointCount() == 2)
            assert(secondsCurve:Evaluate(0) == 100 and secondsCurve:Evaluate(20) == 300)
            assert(percentCurve:Evaluate(0) == 2 and percentCurve:Evaluate(1) == 6)
        end
        function AssertDurationCurveSamples(method, curve, realValues, baseValues)
            local samples = {5, 10, 12.5, 15, 20, 25, 12.5}
            for index, time in ipairs(samples) do
                clock:SetTime(time)
                local function check(expected, ...)
                    local actual = duration[method](duration, curve, ...)
                    assert(type(actual) == 'number' and actual == expected,
                        method .. ' at ' .. time .. ': expected ' .. expected .. ', got ' .. tostring(actual))
                end
                check(realValues[index])
                check(realValues[index], nil)
                check(realValues[index], Enum.DurationTimeModifier.RealTime)
                check(baseValues[index], Enum.DurationTimeModifier.BaseTime)
                AssertDurationCurveState()
                assert(clock:GetTime() == time)
            end
        end
        function AssertDurationCurveColor(value, r, g, b, a)
            assert(type(value) == 'table' and type(value.GetRGBA) == 'function',
                'duration color evaluation must return a ColorMixin result')
            local vr, vg, vb, va = value:GetRGBA()
            assert(vr == r and vg == g and vb == b and va == a, 'unexpected evaluated RGBA')
        end
        function NewDurationColorCurve(maximum)
            local curve = C_CurveUtil.CreateColorCurve()
            curve:AddPoint(0, CreateColor(0, 0, 1, 0.5))
            curve:AddPoint(maximum, CreateColor(1, 0.5, 0, 1))
            return curve
        end
        "#,
    )
    .expect("real duration and curve fixtures should initialize");
    env
}

#[test]
fn duration_curve_elapsed_seconds_follow_clock_and_modifiers() {
    duration_curve_env()
        .exec(
            r#"
        AssertDurationCurveSamples('EvaluateElapsedDuration', secondsCurve,
            {100, 100, 125, 150, 200, 200, 125},
            {100, 100, 150, 200, 300, 300, 150})
    "#,
        )
        .expect("elapsed seconds use 100 + 10 * seconds across clock boundaries and rewind");
}

#[test]
fn duration_curve_remaining_seconds_follow_clock_and_modifiers() {
    duration_curve_env()
        .exec(
            r#"
        AssertDurationCurveSamples('EvaluateRemainingDuration', secondsCurve,
            {200, 200, 175, 150, 100, 100, 175},
            {300, 300, 250, 200, 100, 100, 250})
    "#,
        )
        .expect("remaining seconds use 100 + 10 * seconds across clock boundaries and rewind");
}

#[test]
fn duration_curve_elapsed_percent_is_modifier_invariant() {
    duration_curve_env()
        .exec(
            r#"
        AssertDurationCurveSamples('EvaluateElapsedPercent', percentCurve,
            {2, 2, 3, 4, 6, 6, 3}, {2, 2, 3, 4, 6, 6, 3})
    "#,
        )
        .expect("elapsed percentages use 2 + 4 * fraction for every supported modifier");
}

#[test]
fn duration_curve_remaining_percent_is_modifier_invariant() {
    duration_curve_env()
        .exec(
            r#"
        AssertDurationCurveSamples('EvaluateRemainingPercent', percentCurve,
            {6, 6, 5, 4, 2, 2, 5}, {6, 6, 5, 4, 2, 2, 5})
    "#,
        )
        .expect("remaining percentages use 2 + 4 * fraction for every supported modifier");
}

#[test]
fn duration_curve_color_results_preserve_rgba_for_all_four_queries() {
    duration_curve_env()
        .exec(
            r#"
        local seconds = NewDurationColorCurve(10)
        local percent = NewDurationColorCurve(1)
        for index, entry in ipairs(methods) do
            local curve = index <= 2 and seconds or percent
            local evaluate = duration[entry[1]]
            AssertDurationCurveColor(evaluate(duration, curve), 0.5, 0.25, 0.5, 0.75)
            AssertDurationCurveColor(evaluate(duration, curve, nil), 0.5, 0.25, 0.5, 0.75)
            AssertDurationCurveColor(evaluate(duration, curve, Enum.DurationTimeModifier.RealTime),
                0.5, 0.25, 0.5, 0.75)
            local base = evaluate(duration, curve, Enum.DurationTimeModifier.BaseTime)
            if index <= 2 then
                AssertDurationCurveColor(base, 1, 0.5, 0, 1)
            else
                AssertDurationCurveColor(base, 0.5, 0.25, 0.5, 0.75)
            end
        end
        AssertDurationCurveState()
    "#,
        )
        .expect(
            "duration evaluation preserves real color curve results rather than returning numbers",
        );
}

#[test]
fn duration_curve_live_point_reconfiguration_changes_all_queries() {
    duration_curve_env()
        .exec(
            r#"
        for index, entry in ipairs(methods) do
            local scalar = C_CurveUtil.CreateCurve()
            local maximum = index <= 2 and 10 or 1
            scalar:AddPoint(0, 10)
            scalar:AddPoint(maximum, 30)
            local evaluate = duration[entry[1]]
            assert(evaluate(duration, scalar) == 20, entry[1] .. ' initial scalar')
            scalar:ClearPoints()
            scalar:AddPoint(0, 50)
            scalar:AddPoint(maximum, 90)
            assert(evaluate(duration, scalar) == 70, entry[1] .. ' changed scalar')
            local color = NewDurationColorCurve(maximum)
            AssertDurationCurveColor(evaluate(duration, color), 0.5, 0.25, 0.5, 0.75)
            color:ClearPoints()
            color:AddPoint(0, CreateColor(1, 1, 0, 0))
            color:AddPoint(maximum, CreateColor(0, 0, 1, 0.5))
            AssertDurationCurveColor(evaluate(duration, color), 0.5, 0.5, 0.5, 0.25)
        end
        AssertDurationCurveState()
    "#,
        )
        .expect("each duration query evaluates the live scalar or color curve points");
}

#[test]
fn duration_curve_invalid_modifiers_preserve_duration_and_curve_state() {
    duration_curve_env()
        .exec(
            r#"
        for _, entry in ipairs(methods) do
            assert(not pcall(duration[entry[1]], duration, entry[2], 2),
                entry[1] .. ' must reject modifier 2')
            AssertDurationCurveState()
            assert(clock:GetTime() == 15)
        end
    "#,
        )
        .expect("invalid modifiers fail without changing duration state or curve points");
}

#[test]
fn duration_curve_invalid_bound_clock_preserves_duration_and_curve_state() {
    duration_curve_env()
        .exec(
            r#"
        for _, badTime in ipairs({math.huge, 'not a time'}) do
            clock.time = badTime
            for _, entry in ipairs(methods) do
                assert(not pcall(duration[entry[1]], duration, entry[2]),
                    entry[1] .. ' must reject an invalid bound clock')
                AssertDurationCurveState()
                assert(clock.time == badTime, 'evaluation must not repair the clock')
            end
        end
    "#,
        )
        .expect("invalid clock values propagate errors without changing configured state");
}

#[test]
fn duration_curve_missing_and_wrong_curves_are_rejected_without_mutation() {
    duration_curve_env()
        .exec(
            r#"
        local fakeCalled = false
        local fake = {Evaluate = function() fakeCalled = true; return 999 end}
        local wrong = {{}, fake, clock, duration, false, 12, 'curve'}
        for _, entry in ipairs(methods) do
            local evaluate = duration[entry[1]]
            assert(not pcall(evaluate, duration), entry[1] .. ' missing curve')
            AssertDurationCurveState()
            assert(not pcall(evaluate, duration, nil), entry[1] .. ' nil curve')
            AssertDurationCurveState()
            for _, curve in ipairs(wrong) do
                assert(not pcall(evaluate, duration, curve), entry[1] .. ' wrong curve')
                AssertDurationCurveState()
                assert(clock:GetTime() == 15 and not fakeCalled)
            end
        end
    "#,
        )
        .expect("only genuine curve objects are accepted; fake Evaluate tables are not invoked");
}

#[test]
fn duration_curve_unsupported_color_interpolation_propagates_evaluator_error() {
    duration_curve_env().exec(r#"
        local curve = NewDurationColorCurve(10)
        curve:SetType(Enum.LuaCurveType.Cubic)
        local expected = 'Color curve interpolation type is not modeled'
        local ok, directError = pcall(curve.Evaluate, curve, 5)
        assert(not ok and tostring(directError):find(expected, 1, true),
            'fixture must exercise the existing unsupported Cubic color evaluator')
        for _, entry in ipairs(methods) do
            local succeeded, message = pcall(duration[entry[1]], duration, curve)
            assert(not succeeded, entry[1] .. ' must propagate the curve error')
            assert(tostring(message):find(expected, 1, true), 'original evaluator error must survive')
            AssertDurationCurveState()
            assert(clock:GetTime() == 15 and curve:GetPointCount() == 2)
        end
        curve:SetType(Enum.LuaCurveType.Linear)
        AssertDurationCurveColor(curve:Evaluate(0), 0, 0, 1, 0.5)
        AssertDurationCurveColor(curve:Evaluate(10), 1, 0.5, 0, 1)
    "#).expect("unsupported color interpolation errors are not replaced by zero fallback results");
}
