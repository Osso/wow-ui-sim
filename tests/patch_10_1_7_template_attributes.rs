//! Runtime template attributes must exist before OnLoad, including inherited children.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_10_1_7_runtime_template_attributes_before_onload() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        P1017AttributeMixin = {
            OnAttributeChanged = function(self)
                assert(self.loadedCount ~= nil, 'attribute notification before OnLoad')
                self.attributeChanges = (self.attributeChanges or 0) + 1
            end,
        }
    "#).unwrap();
    let xml = r#"
        <Ui>
            <Frame name="P1017AttributeBase" virtual="true">
                <Attributes>
                    <Attribute name="ping-receiver" type="boolean" value="true"/>
                    <Attribute name="count" type="number" value="7"/>
                    <Attribute name="label" type="string" value="original"/>
                </Attributes>
            </Frame>
            <Frame name="P1017AttributeDerived" inherits="P1017AttributeBase" mixin="P1017AttributeMixin" virtual="true">
                <Attributes>
                    <Attribute name="count" type="number" value="11"/>
                    <Attribute name="label" type="nil"/>
                </Attributes>
                <Frames>
                    <Frame parentKey="Child" inherits="P1017AttributeBase">
                        <Attributes>
                            <Attribute name="ping-receiver" type="boolean" value="false"/>
                        </Attributes>
                        <Scripts><OnLoad>self.loadedCount = self:GetAttribute("count")</OnLoad></Scripts>
                    </Frame>
                </Frames>
                <Scripts><OnLoad>self.loadedCount = self:GetAttribute("count")</OnLoad></Scripts>
            </Frame>
        </Ui>
    "#;
    let ui = wow_ui_sim::xml::parse_xml(xml).unwrap();
    for element in ui.elements {
        if let wow_ui_sim::xml::XmlElement::Frame(frame) = element {
            let name = frame.name.clone().unwrap();
            wow_ui_sim::xml::register_template(&name, "Frame", frame);
        }
    }
    env.exec(r#"
        local frame = CreateFrame('Frame', nil, UIParent, 'P1017AttributeDerived')
        assert(frame:GetAttribute('ping-receiver') == true, 'base boolean')
        assert(frame:GetAttribute('count') == 11, 'derived numeric override')
        assert(frame:GetAttribute('label') == nil, 'derived nil removes base string')
        assert(frame.loadedCount == 11, 'root OnLoad observes attributes')
        assert(frame.attributeChanges == nil, 'initial attributes do not notify')
        frame:SetAttribute('count', 13)
        assert(frame.attributeChanges == 1 and frame:GetAttribute('count') == 13,
            'ordinary mutation notifies after construction')
        assert(frame.Child:GetAttribute('ping-receiver') == false, 'child boolean override')
        assert(frame.Child:GetAttribute('count') == 7, 'child inherits numeric')
        assert(frame.Child:GetAttribute('label') == 'original', 'child inherits string')
        assert(frame.Child.loadedCount == 7, 'child OnLoad observes attributes')
    "#).unwrap();
}
