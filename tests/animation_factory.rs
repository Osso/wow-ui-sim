//! Ordinary CreateAnimation behavior; forbidden-aspect enforcement is not covered.
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn create_animation_optional_arguments_return_one_owned_object() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local owner = CreateFrame("Frame", nil, UIParent)
        local group = owner:CreateAnimationGroup()
        local function capture(...)
            assert(select('#', ...) == 1, "factory must return exactly one object")
            return ...
        end
        local omitted = capture(group:CreateAnimation())
        local nils = capture(group:CreateAnimation(nil, nil, nil))
        local named = capture(group:CreateAnimation("Alpha", "FactoryNamedAlpha", nil))
        assert(omitted ~= nils and named ~= omitted and named ~= nils)
        assert(omitted:GetObjectType() == "Animation")
        assert(nils:GetObjectType() == "Animation")
        assert(named:GetObjectType() == "Alpha")
        assert(named:GetName() == "FactoryNamedAlpha")
        assert(omitted:GetName() == nil and nils:GetName() == nil)
        for _, animation in ipairs({omitted, nils, named}) do
            assert(animation:GetParent() == group)
            assert(animation:GetRegionParent() == owner)
            assert(animation:GetOrder() == 1)
            assert(animation:GetDuration() == 0)
        end
        local first, second, third = group:GetAnimations()
        assert(select('#', group:GetAnimations()) == 3)
        assert(first == omitted and second == nils and third == named)
        "#,
    )
    .unwrap();
}

#[test]
fn create_animation_named_result_uses_its_owner_timeline() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        owner = CreateFrame("Frame", nil, UIParent)
        group = owner:CreateAnimationGroup()
        idle = owner:CreateAnimationGroup()
        preceding = group:CreateAnimation("Animation")
        preceding:SetDuration(0.5)
        created = group:CreateAnimation("Alpha", "FactoryPlaybackAlpha")
        created:SetOrder(2)
        created:SetDuration(1)
        created:SetFromAlpha(0.25)
        created:SetToAlpha(0.75)
        assert(created:GetOrder() == 2 and created:GetDuration() == 1)
        assert(created:GetFromAlpha() == 0.25 and created:GetToAlpha() == 0.75)
        finished = 0
        created:SetScript("OnFinished", function(self)
            assert(self == created and self:GetParent() == group)
            finished = finished + 1
        end)
        group:Play()
        assert(group:IsPlaying() and not idle:IsPlaying())
        "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec("assert(created:GetElapsed() == 0 and finished == 0)")
        .unwrap();
    env.fire_on_update(0.75).unwrap();
    env.exec(
        "assert(created:GetElapsed() == 0.5 and created:GetProgress() == 0.5); \
         assert(finished == 0 and not idle:IsPlaying())",
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("assert(finished == 1 and group:IsDone()); assert(created:GetParent() == group)")
        .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec("assert(finished == 1)").unwrap();
}

fn load_animation_factory_fixture(env: &WowLuaEnv) -> tempfile::TempDir {
    let addon = tempfile::tempdir().unwrap();
    std::fs::write(
        addon.path().join("AnimationFactoryFixture.toc"),
        "## Title: AnimationFactoryFixture\nAnimationFactoryFixture.xml\n",
    )
    .unwrap();
    std::fs::write(
        addon.path().join("AnimationFactoryFixture.xml"),
        r#"<Ui>
            <Animation name="FactoryTimingTemplate" virtual="true" duration="1.75"
                order="4" startDelay="0.125" endDelay="0.25" smoothing="OUT"/>
            <Frame name="FactoryFixtureOwner" parent="UIParent">
                <Animations>
                    <AnimationGroup parentKey="ControlGroup">
                        <Animation parentKey="Control" duration="1.75" order="4"
                            startDelay="0.125" endDelay="0.25" smoothing="OUT"/>
                    </AnimationGroup>
                </Animations>
            </Frame>
        </Ui>"#,
    )
    .unwrap();
    load_addon(
        &env.loader_env(),
        &addon.path().join("AnimationFactoryFixture.toc"),
    )
    .unwrap();
    addon
}

#[test]
fn create_animation_applies_a_loader_declared_animation_template() {
    let env = WowLuaEnv::new().unwrap();
    let _addon = load_animation_factory_fixture(&env);
    env.exec(
        r#"
        local owner = FactoryFixtureOwner
        local control = owner.ControlGroup.Control
        assert(control:GetDuration() == 1.75 and control:GetOrder() == 4)
        assert(control:GetStartDelay() == 0.125 and control:GetEndDelay() == 0.25)
        assert(control:GetSmoothing() == "OUT")
        local group = owner:CreateAnimationGroup()
        local created = group:CreateAnimation("Animation", "FactoryTemplatedAnimation", "FactoryTimingTemplate")
        assert(created:GetObjectType() == "Animation")
        assert(created:GetName() == "FactoryTemplatedAnimation")
        assert(created:GetParent() == group and created:GetRegionParent() == owner)
        assert(created:GetDuration() == control:GetDuration(),
            "template duration: expected 1.75, got " .. tostring(created:GetDuration()))
        assert(created:GetOrder() == control:GetOrder())
        assert(created:GetStartDelay() == control:GetStartDelay())
        assert(created:GetEndDelay() == control:GetEndDelay())
        assert(created:GetSmoothing() == control:GetSmoothing())
        "#,
    )
    .unwrap();
}
