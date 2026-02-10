//! Tests for animation group target visibility (TargetsVisibleWhilePlayingAnimGroupTemplate).
//!
//! Verifies the OnLoad → Mixin Hide → SetTargetsShown → GetTarget → SetShown chain
//! that hides animation targets (like cast bar Flakes textures) on load.

use wow_ui_sim::lua_api::WowLuaEnv;

fn setup() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

/// Define the TargetsVisibleWhilePlayingAnimGroupMixin equivalent in Lua.
const DEFINE_MIXIN: &str = r##"
    __TestTargetMixin = {}
    function __TestTargetMixin:Hide()
        self:SetTargetsShown(false, self:GetAnimations())
    end
    function __TestTargetMixin:Show()
        self:SetTargetsShown(true, self:GetAnimations())
    end
    function __TestTargetMixin:SetTargetsShown(shown, ...)
        for i = 1, select("#", ...) do
            local anim = select(i, ...)
            if anim then
                local target = anim:GetTarget()
                if target and target.SetShown then
                    target:SetShown(shown)
                end
            end
        end
    end
"##;

/// Create a frame with two child textures and an animation group targeting them.
/// Returns Lua code that sets up the frame and fires OnLoad.
fn setup_cast_bar_with_onload(env: &WowLuaEnv) {
    env.exec(DEFINE_MIXIN).unwrap();
    env.exec(r#"
        local f = CreateFrame("Frame", "TestCastBar", UIParent)
        local flakes1 = f:CreateTexture("Flakes01", "ARTWORK")
        local flakes2 = f:CreateTexture("Flakes02", "ARTWORK")
        f.Flakes01 = flakes1
        f.Flakes02 = flakes2

        local ag = f:CreateAnimationGroup("StandardFinish")
        ag:SetScript("OnLoad", function(self, ...) self:Hide(...) end)
        local a1 = ag:CreateAnimation("Alpha")
        a1:SetChildKey("Flakes01")
        local a2 = ag:CreateAnimation("Alpha")
        a2:SetChildKey("Flakes02")
        Mixin(ag, __TestTargetMixin)

        do local onLoad = ag:GetScript("OnLoad")
        if onLoad then onLoad(ag) end end
    "#).unwrap();
}

/// Verify the full OnLoad chain hides animation targets.
#[test]
fn onload_hides_animation_targets() {
    let env = setup();
    setup_cast_bar_with_onload(&env);
    env.exec(r#"
        local f = TestCastBar
        assert(f.Flakes01:IsShown() == false,
            "Flakes01 should be hidden after OnLoad, got " .. tostring(f.Flakes01:IsShown()))
        assert(f.Flakes02:IsShown() == false,
            "Flakes02 should be hidden after OnLoad, got " .. tostring(f.Flakes02:IsShown()))
    "#).unwrap();
}

/// GetTarget returns the owner frame when no childKey is set.
#[test]
fn get_target_returns_owner_when_no_child_key() {
    let env = setup();
    env.exec(r#"
        local f = CreateFrame("Frame", "TestTargetOwner", UIParent)
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        local target = anim:GetTarget()
        assert(target ~= nil, "GetTarget should return owner frame")
        assert(target:GetName() == "TestTargetOwner",
            "Target should be owner frame, got " .. tostring(target:GetName()))
    "#).unwrap();
}

/// GetTarget resolves childKey to the correct child texture.
#[test]
fn get_target_resolves_child_key() {
    let env = setup();
    env.exec(r#"
        local f = CreateFrame("Frame", "TestTargetChild", UIParent)
        local tex = f:CreateTexture("MyTex", "ARTWORK")
        f.MyTex = tex
        local ag = f:CreateAnimationGroup()
        local anim = ag:CreateAnimation("Alpha")
        anim:SetChildKey("MyTex")
        local target = anim:GetTarget()
        assert(target ~= nil, "GetTarget should return child texture")
        assert(target:GetName() == "MyTex",
            "Target should be MyTex, got " .. tostring(target:GetName()))
    "#).unwrap();
}
