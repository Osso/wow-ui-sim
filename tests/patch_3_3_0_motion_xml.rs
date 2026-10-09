//! Literal 3.3.0 XML attribute backed by the existing button motion flag.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_3_3_0_motion_xml_preserves_true_false_and_template_inheritance_before_onload() {
    let env = WowLuaEnv::new().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let toc = directory.path().join("P330Motion.toc");
    std::fs::write(&toc, "## Title: P330Motion\nfixture.xml\n").unwrap();
    std::fs::write(directory.path().join("fixture.xml"), r#"
        <Ui>
          <Button name="P330MotionBase" virtual="true" motionScriptsWhileDisabled="true"/>
          <Button name="P330MotionDerived" virtual="true" inherits="P330MotionBase" motionScriptsWhileDisabled="false"/>
          <Button name="P330MotionInherited" inherits="P330MotionBase">
            <Scripts><OnLoad>self.motionAtLoad = self:GetMotionScriptsWhileDisabled()</OnLoad></Scripts>
          </Button>
          <Button name="P330MotionFalse" inherits="P330MotionBase" motionScriptsWhileDisabled="false"/>
          <CheckButton name="P330MotionTrue" motionScriptsWhileDisabled="true"/>
          <Button name="P330MotionDefault"/>
        </Ui>
    "#).unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(r#"
        assert(P330MotionInherited:GetMotionScriptsWhileDisabled(), 'inherited true')
        assert(P330MotionInherited.motionAtLoad == true, 'applied before OnLoad')
        assert(not P330MotionFalse:GetMotionScriptsWhileDisabled(), 'explicit false overrides template')
        assert(P330MotionTrue:GetMotionScriptsWhileDisabled(), 'literal CheckButton true')
        assert(not P330MotionDefault:GetMotionScriptsWhileDisabled(), 'omitted stays false')
        local inherited = CreateFrame('Button', nil, UIParent, 'P330MotionBase')
        assert(inherited:GetMotionScriptsWhileDisabled(), 'Lua template creation inherits true')
        local derived = CreateFrame('Button', nil, UIParent, 'P330MotionDerived')
        assert(not derived:GetMotionScriptsWhileDisabled(), 'derived false overrides base true')
        inherited:SetMotionScriptsWhileDisabled(false)
        assert(not inherited:GetMotionScriptsWhileDisabled(), 'Lua mutation still owns same state')
    "#).unwrap();
}
