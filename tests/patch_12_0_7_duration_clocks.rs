#![cfg(feature = "retail-12-0-7")]
//! B06/B07 public clock behavior; cached signatures may postdate 12.0.7.
use wow_ui_sim::lua_api::WowLuaEnv;

fn execute(code: &str) {
    WowLuaEnv::new()
        .expect("duration clock environment")
        .exec(code)
        .expect("public duration clock behavior");
}

#[test]
fn manual_clock_fractional_transitions_independence_and_return_arity() {
    execute(
        r#"
        assert(select('#', C_DurationUtil.CreateManualClock()) == 1)
        local a = C_DurationUtil.CreateManualClock()
        -- Optional initial timestamp is an INFERRED existing simulator extension.
        local b = C_DurationUtil.CreateManualClock(20.5)
        assert(a ~= b and a:GetTime() == 0 and b:GetTime() == 20.5)
        assert(select('#', a:GetTime()) == 1)
        assert(select('#', a:SetTime(10.25)) == 0 and a:GetTime() == 10.25)
        assert(select('#', a:AdvanceTime(2.5)) == 0 and a:GetTime() == 12.75)
        assert(select('#', a:RewindTime(0.75)) == 0 and a:GetTime() == 12)
        assert(b:GetTime() == 20.5)
        b:AdvanceTime(0.25)
        assert(b:GetTime() == 20.75 and a:GetTime() == 12)
        assert(select('#', a:ResetTime()) == 0 and a:GetTime() == 0)
        assert(b:GetTime() == 20.75)
    "#,
    );
}

#[test]
fn manual_clock_state_is_host_owned_not_raw_writable_table_storage() {
    execute(
        r#"
        local clock = C_DurationUtil.CreateManualClock(8.25)
        assert(type(clock) == 'userdata')
        assert(not pcall(rawset, clock, 'time', 999))
        assert(not pcall(function() clock.time = 999 end))
        assert(not pcall(function() clock.GetTime = function() return 999 end end))
        clock:AdvanceTime(0.5)
        assert(clock:GetTime() == 8.75)
        local get = clock.GetTime
        assert(not pcall(get, {time=444}))
        assert(not pcall(get, newproxy(true)))
        assert(clock:GetTime() == 8.75)
    "#,
    );
}

#[test]
fn manual_clock_inferred_validation_rejects_before_mutating() {
    execute(
        r#"
        -- INFERRED: required finite numeric mutation inputs, finite results;
        -- signed finite timestamps/deltas remain supported, not native policy proof.
        local clock = C_DurationUtil.CreateManualClock(8.25)
        for _, name in ipairs({'SetTime', 'AdvanceTime', 'RewindTime'}) do
            assert(not pcall(clock[name], clock))
            assert(not pcall(clock[name], clock, nil))
            for _, bad in ipairs({false, {}, newproxy(true), '12', math.huge, -math.huge, 0/0}) do
                assert(not pcall(clock[name], clock, bad), name .. ' accepted invalid input')
                assert(clock:GetTime() == 8.25, name .. ' mutated after rejection')
            end
        end
        clock:SetTime(-2.5)
        clock:AdvanceTime(-0.25)
        assert(clock:GetTime() == -2.75)
        clock:RewindTime(-0.5)
        assert(clock:GetTime() == -2.25)
        clock:SetTime(1e308)
        assert(not pcall(clock.AdvanceTime, clock, 1e308))
        assert(clock:GetTime() == 1e308)
        clock:SetTime(-1e308)
        assert(not pcall(clock.RewindTime, clock, 1e308))
        assert(clock:GetTime() == -1e308)
    "#,
    );
}

#[test]
fn manual_clock_allowed_when_untainted_authenticates_inputs_before_receiver_validation() {
    execute(
        r#"
        local clock = C_DurationUtil.CreateManualClock(5)
        -- Construct every secret before entering tainted Lua.
        local timestamp, advance, rewind = secretwrap(12.5, 2.25, 0.5)
        clock:SetTime(timestamp)
        clock:AdvanceTime(advance)
        clock:RewindTime(rewind)
        assert(clock:GetTime() == 14.25)
        local function addon()
            assert(debug.getstacktaint() == 'DurationClockAudit')
            for _, item in ipairs({{'SetTime',timestamp}, {'AdvanceTime',advance}, {'RewindTime',rewind}}) do
                assert(not pcall(clock[item[1]], clock, item[2]))
                -- Explicit input must be authenticated even with a bad receiver.
                local ok, message = pcall(clock[item[1]], {}, item[2])
                assert(not ok and tostring(message):find('untainted', 1, true))
            end
            assert(clock:GetTime() == 14.25)
            clock:AdvanceTime(0.25)
            assert(clock:GetTime() == 14.5)
            assert(debug.getstacktaint() == 'DurationClockAudit')
        end
        debug.setobjecttaint(addon, 'DurationClockAudit')
        addon()
        assert(debug.getstacktaint() == nil and clock:GetTime() == 14.5)
    "#,
    );
}

#[test]
fn duration_clock_rebinding_reads_live_boundaries_and_roots_clock_identity() {
    execute(
        r#"
        local d = C_DurationUtil.CreateDuration()
        local a = C_DurationUtil.CreateManualClock(9.75)
        local b = C_DurationUtil.CreateManualClock(14)
        assert(select('#', d:GetClock()) == 1 and d:GetClock() == nil)
        assert(select('#', d:SetClock(a)) == 0 and d:GetClock() == a)
        d:SetTimeFromStart(10, 8, 2) -- real span [10,14)
        local function check(time, started, active, elapsed, remaining)
            assert(d:GetClockTime() == time)
            assert(select('#', d:HasStarted()) == 1 and d:HasStarted() == started)
            assert(select('#', d:IsActive()) == 1 and d:IsActive() == active)
            assert(d:HasStarted(0) == started and d:HasStarted(1) == started)
            assert(d:IsActive(nil) == active and d:IsActive(1) == active)
            assert(d:GetElapsedDuration() == elapsed and d:GetRemainingDuration() == remaining)
            assert(d:GetStartTime() == 10 and d:GetEndTime() == 14 and d:GetModRate() == 2)
        end
        check(9.75, false, false, 0, 4)
        a:AdvanceTime(0.25)
        check(10, true, true, 0, 4)
        a:AdvanceTime(0.5)
        check(10.5, true, true, 0.5, 3.5)
        assert(select('#', d:SetClock(b)) == 0 and d:GetClock() == b)
        check(14, true, false, 4, 0)
        a:SetTime(99)
        check(14, true, false, 4, 0)
        b:RewindTime(0.25)
        check(13.75, true, true, 3.75, 0.25)
        b:ResetTime()
        check(0, false, false, 0, 4)
        b = nil
        collectgarbage('collect')
        local retained = d:GetClock()
        retained:SetTime(11.25)
        check(11.25, true, true, 1.25, 2.75)
        assert(d:GetClock() == retained)
        assert(select('#', d:SetClock(nil)) == 0 and d:GetClock() == nil)
        -- Default clock equivalence without assuming a frozen wall-clock sample.
        local before = GetTime()
        local sample = d:GetClockTime()
        local after = GetTime()
        assert(before <= sample and sample <= after)
    "#,
    );
}

#[test]
fn duration_set_clock_authenticates_wrapped_clock_and_nil_without_rebinding_on_denial() {
    execute(
        r#"
        local a = C_DurationUtil.CreateManualClock(12.5)
        local b = C_DurationUtil.CreateManualClock(30.25)
        local d = C_DurationUtil.CreateDuration()
        d:SetTimeFromStart(10, 40)
        local wrappedClock, wrappedNil = secretwrap(b, nil)
        d:SetClock(wrappedClock)
        assert(d:GetClock() == b and d:GetElapsedDuration() == 20.25)
        d:SetClock(wrappedNil)
        assert(d:GetClock() == nil)
        d:SetClock(a)
        local function addon()
            for _, clock in ipairs({wrappedClock, wrappedNil}) do
                assert(not pcall(d.SetClock, d, clock))
                local ok, message = pcall(d.SetClock, false, clock)
                assert(not ok and tostring(message):find('untainted', 1, true))
                assert(d:GetClock() == a and d:GetElapsedDuration() == 2.5)
            end
            assert(debug.getstacktaint() == 'DurationBindingAudit')
        end
        debug.setobjecttaint(addon, 'DurationBindingAudit')
        addon()
        assert(debug.getstacktaint() == nil and d:GetClock() == a)
    "#,
    );
}

#[test]
fn duration_activity_predicates_authenticate_and_validate_modifier_without_changing_state() {
    execute(
        r#"
        local clock = C_DurationUtil.CreateManualClock(12.5)
        local d = C_DurationUtil.CreateDuration()
        d:SetClock(clock)
        d:SetTimeFromStart(10, 20, 2)
        local real, base, default = secretwrap(0, 1, nil)
        for _, name in ipairs({'HasStarted', 'IsActive'}) do
            assert(d[name](d, real) and d[name](d, base) and d[name](d, default))
            for _, bad in ipairs({2, false, {}, newproxy(true), 0.5}) do
                assert(not pcall(d[name], d, bad))
            end
        end
        local function addon()
            for _, name in ipairs({'HasStarted', 'IsActive'}) do
                for _, modifier in ipairs({real, base, default}) do
                    assert(not pcall(d[name], d, modifier))
                    local ok, message = pcall(d[name], {}, modifier)
                    assert(not ok and tostring(message):find('untainted', 1, true))
                end
                assert(d[name](d, 0))
            end
            assert(debug.getstacktaint() == 'DurationPredicateAudit')
        end
        debug.setobjecttaint(addon, 'DurationPredicateAudit')
        addon()
        assert(debug.getstacktaint() == nil)
        assert(d:GetClock() == clock and clock:GetTime() == 12.5)
        assert(d:GetStartTime() == 10 and d:GetEndTime() == 20)
        clock:SetTime(20)
        assert(d:HasStarted() and not d:IsActive())
    "#,
    );
}

#[test]
fn duration_clocks_and_bindings_are_independent_between_environments() {
    let first = WowLuaEnv::new().expect("first clock environment");
    let second = WowLuaEnv::new().expect("second clock environment");
    let setup = r#"
        clock = C_DurationUtil.CreateManualClock(10.5)
        duration = C_DurationUtil.CreateDuration()
        duration:SetClock(clock)
        duration:SetTimeFromStart(10, 8)
    "#;
    first.exec(setup).unwrap();
    second.exec(setup).unwrap();
    first.exec("clock:AdvanceTime(2.25)").unwrap();
    let first_elapsed: f64 = first.eval("return duration:GetElapsedDuration()").unwrap();
    let second_elapsed: f64 = second.eval("return duration:GetElapsedDuration()").unwrap();
    assert_eq!(first_elapsed, 2.75);
    assert_eq!(second_elapsed, 0.5);
    second.exec("clock:ResetTime()").unwrap();
    assert_eq!(first.eval::<f64>("return clock:GetTime()").unwrap(), 12.75);
    assert_eq!(second.eval::<f64>("return clock:GetTime()").unwrap(), 0.0);
}

#[test]
fn allowed_when_untainted_authenticates_ignored_extras_before_any_validation() {
    execute(
        r#"
        local clock = C_DurationUtil.CreateManualClock(12)
        local d = C_DurationUtil.CreateDuration()
        d:SetClock(clock)
        d:SetTimeFromStart(10, 10)
        local extra = secretwrap(42)
        local function addon()
            for _, name in ipairs({'SetTime', 'AdvanceTime', 'RewindTime'}) do
                local ok, message = pcall(clock[name], clock, 1, extra)
                assert(not ok and tostring(message):find('untainted', 1, true), name .. ' ignored secret extra')
                ok, message = pcall(clock[name], {}, false, extra)
                assert(not ok and tostring(message):find('untainted', 1, true), name .. ' validated before extra authentication')
            end
            for _, name in ipairs({'SetClock', 'HasStarted', 'IsActive'}) do
                local input = name == 'SetClock' and clock or 0
                local ok, message = pcall(d[name], d, input, extra)
                assert(not ok and tostring(message):find('untainted', 1, true), name .. ' ignored secret extra')
                ok, message = pcall(d[name], {}, false, extra)
                assert(not ok and tostring(message):find('untainted', 1, true), name .. ' validated before extra authentication')
            end
            assert(clock:GetTime() == 12 and d:GetClock() == clock)
        end
        debug.setobjecttaint(addon, 'DurationExtraArguments')
        addon()
        assert(clock:GetTime() == 12 and d:GetClock() == clock)
        "#,
    );
}
