//! Ordinary OnPlay transition policy; native repeated-call/security behavior is unverified.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn animation_on_play_observes_committed_group_and_preserves_reentrant_stop() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local owner = CreateFrame("Frame")
        local group = owner:CreateAnimationGroup()
        local animation = group:CreateAnimation("Alpha")
        animation:SetDuration(1)
        local calls = 0
        group:SetScript("OnPlay", function(self, ...)
            calls = calls + 1
            assert(self == group and select('#', ...) == 0)
            assert(self:IsPlaying() and not self:IsPaused() and not self:IsDone())
            assert(self:IsReverse())
            self:Play(true)
            assert(calls == 1, "already-playing calls must not redispatch")
            self:Stop()
            assert(not self:IsPlaying())
        end)
        animation:Play(true)
        assert(calls == 1, "OnPlay must target the resolved group")
        assert(not group:IsPlaying() and group:IsDone(), "callback state must survive Play return")
        finished = 0
        group:SetScript("OnFinished", function() finished = finished + 1 end)
    "#).unwrap();
    env.fire_on_update(2.0).unwrap();
    env.exec("assert(finished == 0)").unwrap();
}

#[test]
fn animation_on_play_routes_set_playing_and_play_synced() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local group = CreateFrame("Frame"):CreateAnimationGroup()
        group:CreateAnimation("Alpha"):SetDuration(1)
        local calls = 0
        group:SetScript("OnPlay", function(self)
            assert(self == group and self:IsPlaying() and not self:IsPaused())
            calls = calls + 1
        end)
        group:SetPlaying(true)
        assert(calls == 1)
        group:SetPlaying(true)
        assert(calls == 1)
        group:Pause()
        group:PlaySynced()
        assert(calls == 2)
        group:Stop()
        group:Play()
        assert(calls == 3)
    "#).unwrap();
}
