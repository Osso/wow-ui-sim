//! Ordinary simulator query behavior; native/security equivalence is not asserted.

use wow_ui_sim::lua_api::WowLuaEnv;

fn setup() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        owner = CreateFrame("Frame", nil, UIParent)
        group = owner:CreateAnimationGroup()
        animation = group:CreateAnimation("Alpha")
        animation:SetDuration(1)
        animation:SetSmoothing("IN")
        idleGroup = owner:CreateAnimationGroup()
        idleAnimation = idleGroup:CreateAnimation("Alpha")
        idleAnimation:SetDuration(1)
        function near(actual, expected)
            assert(math.abs(actual - expected) < 0.000001,
                tostring(actual) .. " ~= " .. tostring(expected))
        end
        function state(playing, paused, done, pending)
            for _, object in ipairs({group, animation}) do
                assert(object:IsPlaying() == playing)
                assert(object:IsPaused() == paused)
                assert(object:IsDone() == done)
            end
            assert(group:IsPendingFinish() == pending)
            assert(animation:IsStopped() == not playing)
            assert(not idleAnimation:IsPlaying() and not idleAnimation:IsPaused())
            assert(not idleAnimation:IsDone() and idleAnimation:IsStopped())
        end
        "#,
    )
    .unwrap();
    env
}

#[test]
fn owner_queries_follow_pause_resume_restart_and_stop() {
    let env = setup();
    env.exec("state(false, false, false, false); group:Play(); state(true, false, false, false)")
        .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec(
        r#"
        near(group:GetElapsed(), 0.25)
        near(group:GetProgress(), 0.25)
        near(animation:GetElapsed(), 0.25)
        near(animation:GetProgress(), 0.25)
        near(animation:GetSmoothProgress(), 0.25)
        animation:Pause()
        state(false, true, false, false)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec(
        r#"
        near(group:GetElapsed(), 0.25)
        near(animation:GetElapsed(), 0.25)
        group:Play()
        state(true, false, false, false)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec(
        r#"
        near(group:GetProgress(), 0.5)
        near(animation:GetSmoothProgress(), 0.5)
        animation:Restart()
        state(true, false, false, false)
        near(group:GetElapsed(), 0)
        near(animation:GetElapsed(), 0)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.125).unwrap();
    env.exec(
        r#"
        near(animation:GetProgress(), 0.125)
        animation:Stop()
        state(false, false, true, false)
        near(group:GetElapsed(), 0)
        near(animation:GetProgress(), 0)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("state(false, false, true, false); near(animation:GetElapsed(), 0)")
        .unwrap();
}

#[test]
fn pending_finish_queries_are_settled_before_callbacks() {
    let env = setup();
    env.exec(
        r#"
        finished = 0
        group:SetScript("OnFinished", function(self)
            assert(self == group)
            state(false, false, true, false)
            near(group:GetProgress(), 1)
            near(animation:GetElapsed(), 1)
            near(animation:GetSmoothProgress(), 1)
            finished = finished + 1
        end)
        group:Play()
        "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec(
        r#"
        animation:Pause()
        animation:Finish()
        state(true, false, false, true)
        near(group:GetElapsed(), 0.25)
        assert(finished == 0)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.0).unwrap();
    env.exec("state(false, false, true, false); assert(finished == 1)")
        .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("assert(finished == 1); group:Restart(); state(true, false, false, false)")
        .unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec("assert(finished == 2); state(false, false, true, false)")
        .unwrap();
}

#[test]
fn reverse_and_repeat_queries_follow_playback_direction() {
    let env = setup();
    env.exec("assert(group:GetLoopState() == 'NONE'); group:Play()")
        .unwrap();
    env.fire_on_update(0.75).unwrap();
    env.exec("group:Play(true); assert(group:IsReverse()); state(true, false, false, false)")
        .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec(
        r#"
        near(group:GetElapsed(), 0.5)
        near(animation:GetElapsed(), 0.5)
        near(group:GetProgress(), 0.5)
        near(animation:GetSmoothProgress(), 0.5)
        group:Stop()
        group:SetLooping("REPEAT")
        group:Play()
        assert(not group:IsReverse() and group:GetLoopState() == "REPEAT")
        "#,
    )
    .unwrap();
    env.fire_on_update(1.25).unwrap();
    env.exec(
        r#"
        state(true, false, false, false)
        assert(group:GetLoopState() == "REPEAT" and not group:IsReverse())
        near(group:GetElapsed(), 0.25)
        near(animation:GetProgress(), 0.25)
        group:Play(true)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec(
        r#"
        state(true, false, false, false)
        assert(group:IsReverse() and group:GetLoopState() == "REPEAT")
        near(group:GetProgress(), 0.75)
        near(animation:GetSmoothProgress(), 0.75)
        "#,
    )
    .unwrap();
}

#[test]
fn bounce_loop_callback_observes_direction_change() {
    let env = setup();
    env.exec(
        r#"
        loops = 0
        group:SetLooping("BOUNCE")
        group:SetScript("OnLoop", function(self)
            assert(self == group and self:GetLoopState() == "BOUNCE")
            state(true, false, false, false)
            loops = loops + 1
            assert(self:IsReverse() == (loops % 2 == 1))
        end)
        group:Play()
        "#,
    )
    .unwrap();
    env.fire_on_update(1.25).unwrap();
    env.exec("assert(loops == 1 and group:IsReverse())").unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("assert(loops == 2 and not group:IsReverse())")
        .unwrap();
}

#[test]
fn delayed_animation_queries_use_local_active_elapsed() {
    let env = setup();
    env.exec(
        r#"
        animation:SetStartDelay(0.5)
        animation:SetEndDelay(0.5)
        group:Play()
        assert(animation:IsDelaying())
        "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec(
        r#"
        assert(animation:IsDelaying())
        near(group:GetElapsed(), 0.25)
        near(group:GetProgress(), 0.125)
        near(animation:GetElapsed(), 0)
        near(animation:GetProgress(), 0)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec("near(group:GetElapsed(), 0.5); assert(not animation:IsDelaying())")
        .unwrap();
    env.fire_on_update(0.25).unwrap();
    let (elapsed, progress, delaying): (f64, f64, bool) = env
        .eval("return animation:GetElapsed(), animation:GetProgress(), animation:IsDelaying()")
        .unwrap();
    assert_eq!((elapsed, progress, delaying), (0.25, 0.25, false));
    env.fire_on_update(0.5).unwrap();
    env.exec(
        r#"
        assert(not animation:IsDelaying())
        near(group:GetProgress(), 0.625)
        near(animation:GetElapsed(), 0.75)
        near(animation:GetProgress(), 0.75)
        near(animation:GetSmoothProgress(), 0.75)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec(
        r#"
        state(true, false, false, false)
        assert(not animation:IsDelaying())
        near(group:GetProgress(), 0.875)
        near(animation:GetElapsed(), 1)
        near(animation:GetProgress(), 1)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec("state(false, false, true, false); near(group:GetProgress(), 1)")
        .unwrap();
}

#[test]
fn delaying_uses_order_start_and_longest_parallel_duration() {
    let env = setup();
    env.exec(
        r#"
        animation:SetOrder(9)
        animation:SetStartDelay(0.5)
        animation:SetEndDelay(0.5)
        local prior = group:CreateAnimation("Alpha")
        prior:SetOrder(2)
        prior:SetDuration(0.5)
        prior:SetEndDelay(0.5)
        local parallel = group:CreateAnimation("Alpha")
        parallel:SetOrder(2)
        parallel:SetDuration(0.25)
        group:Play()
        assert(not animation:IsDelaying())
        "#,
    )
    .unwrap();
    for (delta, time, delaying) in [
        (0.75, 0.75, false),
        (0.25, 1.0, true),
        (0.25, 1.25, true),
        (0.25, 1.5, false),
        (0.25, 1.75, false),
        (0.75, 2.5, false),
        (0.25, 2.75, false),
    ] {
        env.fire_on_update(delta).unwrap();
        let actual: (f64, bool) = env
            .eval("return group:GetElapsed(), animation:IsDelaying()")
            .unwrap();
        assert_eq!(actual, (time, delaying), "ordered timeline at {time}");
    }
    env.exec("group:Pause(); assert(not animation:IsDelaying()); group:Play(true)")
        .unwrap();
    for (delta, time, delaying) in [
        (1.25, 1.5, false),
        (0.25, 1.25, true),
        (0.25, 1.0, true),
        (0.25, 0.75, false),
    ] {
        env.fire_on_update(delta).unwrap();
        let actual: (f64, bool) = env
            .eval("return group:GetElapsed(), animation:IsDelaying()")
            .unwrap();
        assert_eq!(actual, (time, delaying), "reverse timeline at {time}");
    }
}

