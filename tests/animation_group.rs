//! Tests for AnimationGroup lifecycle: Play, Stop, Pause, tick, OnFinished, looping, alpha.
//! Animation object property/state/target/script tests are in animation_anim.rs.

use wow_ui_sim::lua_api::WowLuaEnv;

fn setup() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn create_animation_group() {
    let env = setup();
    env.exec(r#"
        local f = CreateFrame("Frame", "TestAnimFrame", UIParent)
        local ag = f:CreateAnimationGroup("TestGroup")
        assert(ag ~= nil, "AnimationGroup should not be nil")
        assert(ag:GetName() == "TestGroup", "Name should be TestGroup, got " .. tostring(ag:GetName()))
    "#).unwrap();
}

#[test]
fn named_animation_group_registers_global() {
    let env = setup();
    let same_group: bool = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "TestAnimFrameNamedGlobal", UIParent)
            local ag = f:CreateAnimationGroup("TestAnimGroupGlobal")
            return _G.TestAnimGroupGlobal == ag
            "#,
        )
        .unwrap();
    assert!(
        same_group,
        "named animation groups should bind their name in _G like other named UI objects"
    );
}

#[test]
fn initial_state() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame2", UIParent)
        local ag = f:CreateAnimationGroup()
        assert(ag:IsPlaying() == false, "Should not be playing initially")
        assert(ag:IsPaused() == false, "Should not be paused initially")
        assert(ag:IsDone() == false, "Should not be done initially (never played)")
        assert(ag:GetLooping() == "NONE", "Default looping should be NONE")
        assert(ag:GetDuration() == 0, "Default duration should be 0")
    "#,
    )
    .unwrap();
}

#[test]
fn play_sets_playing() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame3", UIParent)
        local ag = f:CreateAnimationGroup()
        ag:Play()
        assert(ag:IsPlaying() == true, "Should be playing after Play()")
        assert(ag:IsPaused() == false)
        assert(ag:IsDone() == false)
    "#,
    )
    .unwrap();
}

#[test]
fn stop_sets_done() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame4", UIParent)
        local ag = f:CreateAnimationGroup()
        ag:Play()
        ag:Stop()
        assert(ag:IsPlaying() == false, "Should not be playing after Stop()")
        assert(ag:IsDone() == true, "Should be done after Stop()")
    "#,
    )
    .unwrap();
}

#[test]
fn stop_dispatches_on_stop_after_settling_state_and_allows_reentrant_play() {
    let env = setup();
    env.exec(
        r#"
        local owner = CreateFrame("Frame", nil, UIParent)
        local glow = CreateFrame("Frame", nil, owner)
        local group = owner:CreateAnimationGroup()
        local animation = group:CreateAnimation("Alpha")
        animation:SetDuration(2)
        local stops, finished = 0, 0
        group:SetScript("OnFinished", function() finished = finished + 1 end)
        group:SetScript("OnStop", function(self)
            stops = stops + 1
            assert(not self:IsPlaying() and not self:IsPaused() and self:IsDone())
            assert(self:GetElapsed() == 0)
            glow:Hide()
            self:Play()
        end)
        group:Play()
        assert(glow:IsShown())
        group:Stop()
        assert(stops == 1, "Stop should dispatch OnStop once")
        assert(not glow:IsShown(), "OnStop should hide the glow")
        assert(group:IsPlaying(), "reentrant Play in OnStop should survive Stop")
        assert(finished == 0, "Stop must not dispatch OnFinished")
    "#,
    )
    .unwrap();
}

#[test]
fn stop_routes_handler_errors_without_aborting_caller() {
    let env = setup();
    env.exec(
        r#"
        local owner = CreateFrame("Frame", nil, UIParent)
        local group = owner:CreateAnimationGroup()
        local animation = group:CreateAnimation("Alpha")
        animation:SetDuration(2)
        local errors = {}
        seterrorhandler(function(message) errors[#errors + 1] = tostring(message) end)
        group:SetScript("OnStop", function() error("stop callback failure") end)
        group:Play()
        local ok = pcall(function() group:Stop() end)
        assert(ok, "OnStop errors must not abort the Stop caller")
        assert(#errors == 1 and errors[1]:find("stop callback failure", 1, true))
        assert(group:IsDone() and not group:IsPlaying())
    "#,
    )
    .unwrap();
}

#[test]
fn pause_and_resume() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame5", UIParent)
        local ag = f:CreateAnimationGroup()
        ag:Play()
        ag:Pause()
        assert(ag:IsPlaying() == false, "Should not be playing when paused")
        assert(ag:IsPaused() == true, "Should be paused")
        -- Resume by calling Play again
        ag:Play()
        assert(ag:IsPlaying() == true, "Should be playing after resume")
        assert(ag:IsPaused() == false, "Should not be paused after resume")
    "#,
    )
    .unwrap();
}

#[test]
fn set_looping() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame6", UIParent)
        local ag = f:CreateAnimationGroup()
        ag:SetLooping("REPEAT")
        assert(ag:GetLooping() == "REPEAT", "Looping should be REPEAT")
        ag:SetLooping("BOUNCE")
        assert(ag:GetLooping() == "BOUNCE", "Looping should be BOUNCE")
        ag:SetLooping("NONE")
        assert(ag:GetLooping() == "NONE", "Looping should be NONE")
    "#,
    )
    .unwrap();
}

#[test]
fn create_animation_returns_handle() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame7", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        assert(anim ~= nil, "Animation should not be nil")
    "#,
    )
    .unwrap();
}

#[test]
fn animation_duration() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame8", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetDuration(0.5)
        assert(anim:GetDuration() == 0.5, "Duration should be 0.5")
    "#,
    )
    .unwrap();
}

#[test]
fn animation_from_to_alpha() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame9", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetFromAlpha(0.2)
        anim:SetToAlpha(0.8)
        assert(anim:GetFromAlpha() == 0.2, "FromAlpha should be 0.2, got " .. anim:GetFromAlpha())
        assert(anim:GetToAlpha() == 0.8, "ToAlpha should be 0.8, got " .. anim:GetToAlpha())
    "#,
    )
    .unwrap();
}

#[test]
fn animation_order() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame10", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetOrder(2)
        assert(anim:GetOrder() == 2, "Order should be 2")
    "#,
    )
    .unwrap();
}

#[test]
fn animation_smoothing() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame11", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetSmoothing("IN_OUT")
        assert(anim:GetSmoothing() == "IN_OUT", "Smoothing should be IN_OUT")
    "#,
    )
    .unwrap();
}

#[test]
fn animation_get_parent() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame12", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        local parent = anim:GetParent()
        assert(parent ~= nil, "GetParent should return the animation group")
    "#,
    )
    .unwrap();
}

#[test]
fn animation_get_region_parent() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame13", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        local region = anim:GetRegionParent()
        assert(region ~= nil, "GetRegionParent should return the owning frame")
        assert(region:GetName() == "TestAnimFrame13", "RegionParent should be the frame")
    "#,
    )
    .unwrap();
}

#[test]
fn group_get_animations() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame14", UIParent)
        local ag = f:CreateAnimationGroup()
        ag:CreateAnimation("Alpha")
        ag:CreateAnimation("Translation")
        local a1, a2 = ag:GetAnimations()
        assert(a1 ~= nil, "First animation should exist")
        assert(a2 ~= nil, "Second animation should exist")
    "#,
    )
    .unwrap();
}

#[test]
fn group_get_parent_returns_frame() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame15", UIParent)
        local ag = f:CreateAnimationGroup()
        local parent = ag:GetParent()
        assert(parent ~= nil, "GetParent should return the frame")
        assert(parent:GetName() == "TestAnimFrame15", "Parent should be TestAnimFrame15")
    "#,
    )
    .unwrap();
}

#[test]
fn set_script_has_script() {
    let env = setup();
    // HasScript reports type-support, not binding-presence (matches live client):
    // a group supports OnFinished regardless of whether a handler is bound. Use
    // GetScript to observe binding presence.
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame16", UIParent)
        local ag = f:CreateAnimationGroup()
        assert(ag:HasScript("OnFinished") == true)
        assert(ag:GetScript("OnFinished") == nil)
        ag:SetScript("OnFinished", function() end)
        assert(ag:HasScript("OnFinished") == true)
        assert(ag:GetScript("OnFinished") ~= nil, "Should have OnFinished handler after SetScript")
        -- Remove script
        ag:SetScript("OnFinished", nil)
        assert(ag:HasScript("OnFinished") == true)
        assert(ag:GetScript("OnFinished") == nil, "Handler should be cleared after removing")
    "#,
    )
    .unwrap();
}

#[test]
fn speed_multiplier() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame17", UIParent)
        local ag = f:CreateAnimationGroup()
        assert(ag:GetAnimationSpeedMultiplier() == 1.0)
        ag:SetAnimationSpeedMultiplier(2.0)
        assert(ag:GetAnimationSpeedMultiplier() == 2.0)
    "#,
    )
    .unwrap();
}

#[test]
fn set_to_final_alpha() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame18", UIParent)
        local ag = f:CreateAnimationGroup()
        assert(ag:IsSetToFinalAlpha() == false)
        ag:SetToFinalAlpha(true)
        assert(ag:IsSetToFinalAlpha() == true)
    "#,
    )
    .unwrap();
}

#[test]
fn tick_finishes_group() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrame19", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetDuration(0.1)
        anim:SetFromAlpha(0)
        anim:SetToAlpha(1)
        _G.testAG19 = ag
        ag:Play()
        assert(ag:IsPlaying() == true)
    "#,
    )
    .unwrap();

    // Tick enough to finish the animation (0.1s duration)
    env.fire_on_update(0.2).unwrap();
    let is_done: bool = env.eval("return _G.testAG19:IsDone()").unwrap();
    assert!(
        is_done,
        "Animation group should be done after ticking past duration"
    );
}

#[test]
fn tick_fires_on_finished() {
    let env = setup();
    env.exec(
        r#"
        _G.on_finished_called = false
        local f = CreateFrame("Frame", "TestAnimFrameOF", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetDuration(0.1)
        anim:SetFromAlpha(0)
        anim:SetToAlpha(1)
        ag:SetScript("OnFinished", function()
            _G.on_finished_called = true
        end)
        ag:Play()
    "#,
    )
    .unwrap();

    // Tick past the animation duration
    env.fire_on_update(0.2).unwrap();

    let called: bool = env.eval("_G.on_finished_called").unwrap();
    assert!(
        called,
        "OnFinished should have been called after animation completes"
    );
}

#[test]
fn tick_fires_child_animation_on_finished() {
    let env = setup();
    env.exec(
        r#"
        _G.child_on_finished_called = false
        local f = CreateFrame("Frame", "TestAnimChildFinished", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetDuration(0.1)
        anim:SetFromAlpha(1)
        anim:SetToAlpha(0)
        anim:SetScript("OnFinished", function(self)
            _G.child_on_finished_called = self:GetRegionParent() == f
        end)
        ag:Play()
    "#,
    )
    .unwrap();

    env.fire_on_update(0.2).unwrap();

    let called: bool = env.eval("_G.child_on_finished_called").unwrap();
    assert!(
        called,
        "child animation OnFinished should fire with the animation as self"
    );
}

#[test]
fn tick_alpha_animation_changes_frame_alpha() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrameAlpha", UIParent)
        f:SetAlpha(1.0)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetDuration(1.0)
        anim:SetFromAlpha(0)
        anim:SetToAlpha(1)
        ag:Play()
    "#,
    )
    .unwrap();

    // Tick halfway through
    env.fire_on_update(0.5).unwrap();

    let alpha: f64 = env
        .eval(
            r#"
        TestAnimFrameAlpha:GetAlpha()
    "#,
        )
        .unwrap();

    // At t=0.5 with linear smoothing, alpha should be ~0.5
    assert!(
        (alpha - 0.5).abs() < 0.05,
        "Alpha should be approximately 0.5 at halfway, got {}",
        alpha
    );
}

#[test]
fn tick_looping_repeat_restarts() {
    let env = setup();
    env.exec(
        r#"
        _G.loop_count = 0
        local f = CreateFrame("Frame", "TestAnimFrameLoop", UIParent)
        local ag = f:CreateAnimationGroup()
        ag:SetLooping("REPEAT")
        local anim = ag:CreateAnimation("Alpha")
        anim:SetDuration(0.1)
        anim:SetFromAlpha(0)
        anim:SetToAlpha(1)
        ag:SetScript("OnLoop", function()
            _G.loop_count = _G.loop_count + 1
        end)
        ag:Play()
    "#,
    )
    .unwrap();

    // Tick through several loops
    for _ in 0..5 {
        env.fire_on_update(0.11).unwrap();
    }

    let loop_count: i32 = env.eval("_G.loop_count").unwrap();
    assert!(
        loop_count >= 3,
        "Should have looped at least 3 times, got {}",
        loop_count
    );

    // Should still be playing (not finished)
    let still_playing: bool = env
        .eval(
            r#"
        return true
    "#,
        )
        .unwrap();
    assert!(still_playing);
}

#[test]
fn animation_order_sequencing() {
    let env = setup();
    env.exec(
        r#"
        _G.on_finished_called = false
        local f = CreateFrame("Frame", "TestAnimFrameOrder", UIParent)
        local ag = f:CreateAnimationGroup()
        -- Order 1: 0.1s
        local a1 = ag:CreateAnimation("Alpha")
        a1:SetDuration(0.1)
        a1:SetFromAlpha(0)
        a1:SetToAlpha(0.5)
        a1:SetOrder(1)
        -- Order 2: 0.1s
        local a2 = ag:CreateAnimation("Alpha")
        a2:SetDuration(0.1)
        a2:SetFromAlpha(0.5)
        a2:SetToAlpha(1.0)
        a2:SetOrder(2)
        ag:SetScript("OnFinished", function()
            _G.on_finished_called = true
        end)
        ag:Play()
    "#,
    )
    .unwrap();

    // Total duration should be 0.2s (0.1 + 0.1 sequential)
    // Tick 0.15s - should be in order 2 but not finished
    env.fire_on_update(0.15).unwrap();

    let finished_early: bool = env.eval("_G.on_finished_called").unwrap();
    assert!(
        !finished_early,
        "Should not have finished at 0.15s when total is 0.2s"
    );

    // Tick another 0.1s - should now be finished
    env.fire_on_update(0.1).unwrap();

    let finished: bool = env.eval("_G.on_finished_called").unwrap();
    assert!(finished, "Should have finished after total 0.25s");
}

#[test]
fn group_duration_matches_order_groups() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrameDur", UIParent)
        local ag = f:CreateAnimationGroup()
        -- Order 1: two anims, max 0.3s
        local a1 = ag:CreateAnimation("Alpha")
        a1:SetDuration(0.2)
        a1:SetOrder(1)
        local a2 = ag:CreateAnimation("Translation")
        a2:SetDuration(0.3)
        a2:SetOrder(1)
        -- Order 2: one anim 0.1s
        local a3 = ag:CreateAnimation("Alpha")
        a3:SetDuration(0.1)
        a3:SetOrder(2)
        local dur = ag:GetDuration()
        assert(dur == 0.4, "Duration should be 0.4 (0.3 + 0.1), got " .. dur)
    "#,
    )
    .unwrap();
}

#[test]
fn finish_method_completes_immediately() {
    let env = setup();
    env.exec(
        r#"
        _G.on_finished_fired = false
        local f = CreateFrame("Frame", "TestAnimFrameFinish", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetDuration(10.0)
        anim:SetFromAlpha(0)
        anim:SetToAlpha(1)
        ag:Play()
        ag:Finish()
        assert(ag:IsPlaying() == true, "Should still be playing after Finish()")
        assert(ag:IsDone() == false, "Should not be done yet after Finish()")
    "#,
    )
    .unwrap();
}

#[test]
fn restart_resets_and_plays() {
    let env = setup();
    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestAnimFrameRestart", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetDuration(1.0)
        ag:Play()
        ag:Stop()
        assert(ag:IsDone() == true)
        ag:Restart()
        assert(ag:IsPlaying() == true, "Should be playing after Restart()")
        assert(ag:IsDone() == false, "Should not be done after Restart()")
    "#,
    )
    .unwrap();
}
