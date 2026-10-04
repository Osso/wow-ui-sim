//! XML private lookup via real Lua/XML addon loading, not bridge helper calls.
#![cfg(feature = "retail-12-1-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_12_1_0_xml_preserves_private_identity_nested_mixins_and_addon_isolation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        AuditGlobal = {Nested = {Mixin = {GlobalValue = function() return 73 end}}}
    "#).unwrap();
    let root = tempfile::tempdir().unwrap();
    for (addon, value) in [("AuditXmlA", 17), ("AuditXmlB", 29)] {
        let dir = root.path().join(addon);
        std::fs::create_dir(&dir).unwrap();
        let toc = dir.join(format!("{addon}.toc"));
        std::fs::write(&toc, "Init.lua\nFrames.xml\nCheck.lua\n").unwrap();
        std::fs::write(dir.join("Init.lua"), format!(r#"
            local _, private = ...
            private.payload = {{value={value}}}
            private.Nested = {{First={{Describe=function(self) return self.payload.value, 'first' end}},
                Last={{Describe=function(self) return self.payload.value, 'last' end}}}}
        "#)).unwrap();
        std::fs::write(dir.join("Frames.xml"), format!(r#"<Ui>
            <Frame name="{addon}Template" virtual="true" mixin="AuditGlobal.Nested.Mixin">
                <Mixins>
                    <Mixin key="Nested.First" source="local"/>
                    <Mixin key="Nested.Last" source="local"/>
                </Mixins>
                <KeyValues><KeyValue key="payload" type="local"/></KeyValues>
            </Frame>
            <Frame name="{addon}Literal" inherits="{addon}Template"/>
        </Ui>"#)).unwrap();
        std::fs::write(dir.join("Check.lua"), format!(r#"
            local _, private = ...
            local literal = _G['{addon}Literal']
            local runtime = CreateFrame('Frame', '{addon}Runtime', UIParent, '{addon}Template')
            assert(rawequal(literal.payload, private.payload))
            assert(rawequal(runtime.payload, private.payload))
            local value, winner = runtime:Describe()
            assert(value == {value} and winner == 'last')
            assert(literal:GlobalValue() == 73 and runtime:GlobalValue() == 73)
            private.payload.value = {value} + 100
            assert(literal:Describe() == {value} + 100 and runtime:Describe() == {value} + 100)
        "#)).unwrap();
        let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
        assert!(loaded.warnings.is_empty(), "{addon}: {:?}", loaded.warnings);
    }
    env.exec(r#"
        assert(not rawequal(AuditXmlALiteral.payload, AuditXmlBLiteral.payload))
        AuditXmlARuntime.payload.value = 501
        assert(AuditXmlALiteral:Describe() == 501)
        assert(AuditXmlBLiteral:Describe() == 129)
        assert(AuditXmlBRuntime:Describe() == 129)
        assert(payload == nil and Nested == nil)
    "#).unwrap();
}
