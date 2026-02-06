//! XML parsing for WoW UI definition files.

mod parse;
mod template;
mod types;

// Re-export all public types and functions
pub use parse::{parse_xml, parse_xml_file, XmlLoadError};
pub use template::{
    clear_templates, get_template, get_template_chain, get_template_info, register_template,
    TemplateEntry, TemplateInfo,
};
pub use types::{
    AbsDimensionXml, ActorXml, ActorsXml, AnchorXml, AnchorsXml, AnimationElement,
    AnimationGroupXml, AnimationXml, AnimationsXml, BackdropXml, ColorXml, FontFamilyXml,
    FontRefXml, FontStringXml, FontXml, FrameChildElement, FrameElement, FrameXml, FramesXml,
    IncludeXml, InsetsXml, KeyValueXml, KeyValuesXml, LayerElement, LayerXml, LayersXml, OffsetXml,
    ResizeBoundsXml, ScriptBodyXml, ScriptXml, ScriptsXml, ScrollChildXml, SizeXml, TextureXml,
    UiXml, XmlElement,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_frame() {
        let xml = r#"
            <Ui>
                <Frame name="TestFrame" parent="UIParent">
                    <Size x="200" y="100"/>
                    <Anchors>
                        <Anchor point="CENTER"/>
                    </Anchors>
                </Frame>
            </Ui>
        "#;

        let ui = parse_xml(xml).unwrap();
        assert_eq!(ui.elements.len(), 1);
    }
}
