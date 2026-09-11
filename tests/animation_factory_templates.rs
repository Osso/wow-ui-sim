//! Real TOC/XML animation templates; native error/security behavior is unverified.
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_fixture(env: &WowLuaEnv, xml: &str) -> tempfile::TempDir {
    let addon = tempfile::tempdir().unwrap();
    std::fs::write(addon.path().join("FactoryTemplates.toc"), "## Title: FactoryTemplates\ntemplates.xml\n").unwrap();
    std::fs::write(addon.path().join("templates.xml"), xml).unwrap();
    load_addon(&env.loader_env(), &addon.path().join("FactoryTemplates.toc")).unwrap();
    addon
}

#[test]
fn animation_templates_inherit_properties_scripts_and_instance_identity() {
    let env = WowLuaEnv::new().unwrap();
    let _addon = load_fixture(&env, r#"<Ui>
      <Animation name="FactoryBase" virtual="true" duration="0.5" order="3" startDelay="0.125">
        <KeyValues><KeyValue key="marker" value="base" type="string"/></KeyValues>
        <Scripts><OnLoad>self.loaded = (self.loaded or 0) + 1</OnLoad></Scripts>
      </Animation>
      <Alpha name="FactoryAlpha" virtual="true" inherits="FactoryBase" parentKey="TemplateIdentity"
          duration="1.25" fromAlpha="0.25" toAlpha="0.75" childKey="Target">
        <KeyValues><KeyValue key="marker" value="derived" type="string"/></KeyValues>
        <Scripts>
          <OnLoad inherit="prepend">
            assert(self.loaded == 1 and self.marker == "derived")
            assert(self:GetDuration() == 1.25 and self:GetOrder() == 3)
            assert(self:GetTarget() == self:GetRegionParent().Target)
            self.loaded = self.loaded + 1
          </OnLoad>
          <OnFinished>self.finished = (self.finished or 0) + 1</OnFinished>
        </Scripts>
      </Alpha>
      <Frame name="FactoryTargetOwner" parent="UIParent">
        <Layers><Layer><Texture parentKey="Target"/></Layer></Layers>
      </Frame>
    </Ui>"#);
    env.exec(r#"
      local owner = FactoryTargetOwner
      local group = owner:CreateAnimationGroup()
      a = group:CreateAnimation("Alpha", "FactoryActualAlpha", "FactoryAlpha")
      local b = group:CreateAnimation("Alpha", nil, "FactoryAlpha")
      assert(a ~= b and a:GetName() == "FactoryActualAlpha" and b:GetName() == nil)
      assert(FactoryActualAlpha == a)
      assert(FactoryBase == nil and FactoryAlpha == nil and group.TemplateIdentity == nil)
      assert(a:GetObjectType() == "Alpha" and a:GetParent() == group)
      assert(a.loaded == 2 and b.loaded == 2)
      assert(a:GetFromAlpha() == 0.25 and a:GetToAlpha() == 0.75)
      assert(a:GetStartDelay() == 0.125)
      a:SetDuration(2)
      assert(b:GetDuration() == 1.25)
      a:SetDuration(1.25)
      group:Play()
    "#).unwrap();
    env.fire_on_update(2.0).unwrap();
    env.exec("assert(a.finished == 1)").unwrap();
}

#[test]
fn animation_templates_keep_inline_overrides_and_group_inheritance() {
    let env = WowLuaEnv::new().unwrap();
    let _addon = load_fixture(&env, r#"<Ui>
      <Animation name="InlineTiming" virtual="true" duration="4" order="2">
        <KeyValues><KeyValue key="value" value="7" type="number"/></KeyValues>
        <Scripts><OnLoad>self.templateLoads = (self.templateLoads or 0) + 1</OnLoad></Scripts>
      </Animation>
      <AnimationGroup name="InheritedAnimGroup" virtual="true">
        <Animation parentKey="FromGroup" inherits="InlineTiming" duration="0.5">
          <Scripts><OnLoad inherit="prepend">
            assert(self.templateLoads == 1 and self.value == 7)
            assert(self:GetDuration() == 0.5 and self:GetParent().FromGroup == self)
            self.ready = true
          </OnLoad></Scripts>
        </Animation>
      </AnimationGroup>
      <Frame name="InlineFactoryOwner" parent="UIParent">
        <Animations><AnimationGroup parentKey="Group" inherits="InheritedAnimGroup">
          <Animation parentKey="Own" inherits="InlineTiming" duration="0.75" order="5"/>
          <Scripts><OnLoad>
            assert(self.FromGroup.ready and self.Own.templateLoads == 1)
            assert(self.Own:GetDuration() == 0.75 and self.Own:GetOrder() == 5)
            self.ready = true
          </OnLoad></Scripts>
        </AnimationGroup></Animations>
      </Frame>
    </Ui>"#);
    env.exec(r#"
      local g = InlineFactoryOwner.Group
      assert(g.ready and select('#', g:GetAnimations()) == 2)
      assert(g.FromGroup:GetOrder() == 2 and g.Own.value == 7)
      assert(g.FromGroup:GetParent() == g and g.Own:GetParent() == g)
    "#).unwrap();
}

#[test]
fn animation_templates_reject_unknown_and_cycles_before_creation() {
    let env = WowLuaEnv::new().unwrap();
    let _addon = load_fixture(&env, r#"<Ui>
      <Animation name="CycleA" virtual="true" inherits="CycleB"/>
      <Animation name="CycleB" virtual="true" inherits="CycleA"/>
      <Animation name="MissingBase" virtual="true" inherits="MissingParent"/>
      <Animation name="FirstTiming" virtual="true" duration="0.25"/>
      <Animation name="SecondTiming" virtual="true" duration="0.75" order="4"/>
    </Ui>"#);
    env.exec(r#"
      local g = CreateFrame("Frame"):CreateAnimationGroup()
      local existing = g:CreateAnimation()
      for _, name in ipairs({"NotRegistered", "CycleA", "MissingBase", "FirstTiming, NotRegistered"}) do
        local ok, err = pcall(g.CreateAnimation, g, "Animation", "FailedTemplateInstance", name)
        assert(not ok and tostring(err):find("animation template"), tostring(err))
        assert(select('#', g:GetAnimations()) == 1 and g:GetAnimations() == existing)
        assert(FailedTemplateInstance == nil)
      end
      local a = g:CreateAnimation("Animation", nil, "FirstTiming, SecondTiming")
      assert(a:GetDuration() == 0.75 and a:GetOrder() == 4)
      local plain = g:CreateAnimation(nil, nil, nil)
      assert(plain:GetDuration() == 0 and plain:GetObjectType() == "Animation")
    "#).unwrap();
}

#[test]
fn animation_templates_apply_flipbook_properties_without_changing_requested_type() {
    let env = WowLuaEnv::new().unwrap();
    let _addon = load_fixture(&env, r#"<Ui>
      <FlipBook name="FactoryFlipbook" virtual="true" duration="2" flipBookRows="3"
         flipBookColumns="4" flipBookFrames="9" smoothing="IN"/>
    </Ui>"#);
    env.exec(r#"
      local g = CreateFrame("Frame"):CreateAnimationGroup()
      local a = g:CreateAnimation("FlipBook", "FlipInstance", "FactoryFlipbook")
      assert(a:GetObjectType() == "FlipBook" and a:GetName() == "FlipInstance")
      assert(a:GetFlipBookRows() == 3 and a:GetFlipBookColumns() == 4)
      assert(a:GetFlipBookFrames() == 9 and a:GetDuration() == 2)
      assert(a:GetSmoothing() == "IN")
      local generic = g:CreateAnimation(nil, nil, "FactoryFlipbook")
      assert(generic:GetObjectType() == "Animation" and generic:GetName() == nil)
    "#).unwrap();
}

#[test]
fn animation_template_registry_is_cleared_for_a_new_environment() {
    {
        let env = WowLuaEnv::new().unwrap();
        let _addon = load_fixture(&env, r#"<Ui><Animation name="EnvironmentLocalAnimation" virtual="true" duration="2"/></Ui>"#);
        env.exec("local g = CreateFrame('Frame'):CreateAnimationGroup(); assert(g:CreateAnimation(nil, nil, 'EnvironmentLocalAnimation'):GetDuration() == 2)").unwrap();
    }
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
      local g = CreateFrame("Frame"):CreateAnimationGroup()
      assert(not pcall(g.CreateAnimation, g, nil, nil, "EnvironmentLocalAnimation"))
      assert(select('#', g:GetAnimations()) == 0)
    "#).unwrap();
}
