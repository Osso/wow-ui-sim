//! XML parsing for WoW UI definition files.

mod parse;
mod profile_templates;
mod template;
mod types;
mod types_animation;
mod types_elements;
mod types_fonts;
mod types_frame_data;
mod types_support;

// Re-export all public types and functions
pub use parse::{XmlLoadError, parse_xml, parse_xml_file};
pub use template::{
    TemplateEntry, TemplateInfo, TemplateKeyValueInfo, clear_templates, collect_anim_group_mixins,
    collect_font_string_mixins, collect_texture_key_values, collect_texture_mixins,
    get_anim_group_template, get_font_string_template, get_template, get_template_chain,
    get_template_info, get_template_lifecycle_flags, get_texture_template_size,
    register_anim_group_template, register_font_string_template, register_intrinsic_templates,
    register_template, register_template_with_local_source, register_texture_template,
    resolve_texture_inheritance,
};
pub use types::{FrameChildElement, FrameXml, ScopedModifierXml, UiXml, XmlElement};
pub use types_animation::{AnimationElement, AnimationGroupXml, AnimationXml};
pub use types_elements::{
    ActorXml, ActorsXml, FontStringXml, FrameElement, FramesXml, IncludeXml, LayerElement,
    LayerXml, LayersXml, ScriptXml, ShadowOffsetXml, ShadowXml, TextureXml, widget_type_for_tag,
};
pub use types_fonts::{FontFamilyMemberXml, FontFamilyXml, FontXml};
pub use types_support::{
    AbsDimensionXml, AnchorXml, AnchorsXml, AnimationsXml, AttributeXml, AttributesXml,
    BackdropXml, BindingXml, ColorXml, FontRefXml, GradientXml, InsetsXml, KeyValueXml,
    KeyValuesXml, MixinXml, MixinsXml, ModifiedClickXml, OffsetXml, ResizeBoundsXml, ScriptBodyXml,
    ScriptsXml, ScrollChildXml, SizeXml,
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

    #[test]
    fn test_parse_lowercase_parentkey_alias() {
        let xml = r#"
            <Ui>
                <Frame name="Parent">
                    <Frames>
                        <Button name="Child" parentkey="angleurKey"/>
                    </Frames>
                </Frame>
            </Ui>
        "#;

        let ui = parse_xml(xml).unwrap();
        let parent = match &ui.elements[0] {
            XmlElement::Frame(frame) => frame,
            other => panic!("expected frame, got {:?}", other),
        };
        let frames = match &parent.children[0] {
            FrameChildElement::Frames(frames) => frames,
            other => panic!("expected frames section, got {:?}", other),
        };
        let child = match &frames.elements[0] {
            FrameElement::Button(frame) => frame,
            other => panic!("expected button, got {:?}", other),
        };

        assert_eq!(child.parent_key.as_deref(), Some("angleurKey"));
    }

    #[test]
    fn test_parse_frame_mixins_block() {
        let xml = r#"
            <Ui>
                <Frame name="AuraContainer" intrinsic="true">
                    <Mixins>
                        <Mixin key="AuraContainerInboundMixin" source="secure" targetPartition="public" inboundPartition="forbidden" secureDelegates="true"/>
                        <Mixin key="AuraContainerPrivateMixin" source="secure"/>
                    </Mixins>
                </Frame>
            </Ui>
        "#;

        let ui = parse_xml(xml).unwrap();
        let frame = match &ui.elements[0] {
            XmlElement::Frame(frame) => frame,
            other => panic!("expected frame, got {:?}", other),
        };
        let mixins = frame.mixins().expect("Mixins block should parse");

        assert_eq!(mixins.entries.len(), 2);
        assert_eq!(mixins.entries[0].key, "AuraContainerInboundMixin");
        assert_eq!(mixins.entries[0].source.as_deref(), Some("secure"));
        assert_eq!(
            mixins.entries[0].target_partition.as_deref(),
            Some("public")
        );
        assert_eq!(
            mixins.entries[0].inbound_partition.as_deref(),
            Some("forbidden")
        );
        assert_eq!(mixins.entries[0].secure_delegates, Some(true));
        assert_eq!(mixins.entries[1].key, "AuraContainerPrivateMixin");
        assert_eq!(mixins.entries[1].source.as_deref(), Some("secure"));
    }

    #[test]
    fn test_parse_element_key_values() {
        let xml = r#"
            <Ui>
                <Frame name="TestFrame">
                    <Layers>
                        <Layer level="ARTWORK">
                            <FontString parentKey="Text">
                                <KeyValues>
                                    <KeyValue key="anchorSpacing" value="4" type="number"/>
                                </KeyValues>
                            </FontString>
                            <Texture parentKey="Icon">
                                <KeyValues>
                                    <KeyValue key="layoutIndex" value="2" type="number"/>
                                </KeyValues>
                            </Texture>
                        </Layer>
                    </Layers>
                </Frame>
            </Ui>
        "#;

        let ui = parse_xml(xml).unwrap();
        let frame = match &ui.elements[0] {
            XmlElement::Frame(frame) => frame,
            other => panic!("expected frame, got {:?}", other),
        };
        let layer = &frame.layers().next().unwrap().layers[0];
        let text = match &layer.elements[0] {
            LayerElement::FontString(text) => text,
            other => panic!("expected fontstring, got {:?}", other),
        };
        let texture = match &layer.elements[1] {
            LayerElement::Texture(texture) => texture,
            other => panic!("expected texture, got {:?}", other),
        };

        assert_eq!(
            text.key_values.as_ref().unwrap().values[0].key,
            "anchorSpacing"
        );
        assert_eq!(
            texture.key_values.as_ref().unwrap().values[0].key,
            "layoutIndex"
        );
    }

    #[test]
    fn test_parse_line_hwrapmode_repeat_as_horiz_tile_intent() {
        let xml = r#"
            <Ui>
                <Frame name="TestFrame">
                    <Layers>
                        <Layer level="ARTWORK">
                            <Line parentKey="MyLine" hWrapMode="REPEAT" vWrapMode="CLAMP"/>
                        </Layer>
                    </Layers>
                </Frame>
            </Ui>
        "#;

        let ui = parse_xml(xml).unwrap();
        let frame = match &ui.elements[0] {
            XmlElement::Frame(frame) => frame,
            other => panic!("expected frame, got {:?}", other),
        };
        let layer = &frame.layers().next().unwrap().layers[0];
        let line = match &layer.elements[0] {
            LayerElement::Line(line) => line,
            other => panic!("expected line, got {:?}", other),
        };
        assert_eq!(line.h_wrap_mode.as_deref(), Some("REPEAT"));
        assert!(line.wants_horiz_tile());
        assert!(!line.wants_vert_tile());
    }

    #[test]
    fn test_parse_masked_textures_ignores_text_nodes() {
        let xml = r#"
            <Ui>
                <Frame name="TestFrame">
                    <Layers>
                        <Layer level="ARTWORK">
                            <MaskTexture parentKey="CircleMask">
                                <MaskedTextures>
                                    \
                                    <MaskedTexture childKey="Portrait"/>
                                    \
                                </MaskedTextures>
                            </MaskTexture>
                        </Layer>
                    </Layers>
                </Frame>
            </Ui>
        "#;

        let ui = parse_xml(xml).unwrap();
        let frame = match &ui.elements[0] {
            XmlElement::Frame(frame) => frame,
            other => panic!("expected frame, got {:?}", other),
        };
        let layer = &frame.layers().next().unwrap().layers[0];
        let mask = match &layer.elements[0] {
            LayerElement::MaskTexture(mask) => mask,
            other => panic!("expected mask texture, got {:?}", other),
        };

        let entries = &mask.masked_textures.as_ref().unwrap().entries;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].child_key.as_deref(), Some("Portrait"));
    }
}
