//! XML parsing for WoW UI definition files.

use serde::Deserialize;

/// Root element of a WoW UI XML file.
#[derive(Debug, Deserialize)]
#[serde(rename = "Ui")]
pub struct UiXml {
    #[serde(rename = "$value", default)]
    pub elements: Vec<XmlElement>,
}

/// XML elements that can appear in a UI definition.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum XmlElement {
    Frame(FrameXml),
    Button(FrameXml),
    Script(ScriptXml),
    Include(IncludeXml),
}

/// Frame definition in XML.
#[derive(Debug, Deserialize, Default)]
pub struct FrameXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parent")]
    pub parent: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@hidden")]
    pub hidden: Option<bool>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,

    #[serde(rename = "Size")]
    pub size: Option<SizeXml>,
    #[serde(rename = "Anchors")]
    pub anchors: Option<AnchorsXml>,
    #[serde(rename = "Scripts")]
    pub scripts: Option<ScriptsXml>,
    #[serde(rename = "Layers")]
    pub layers: Option<LayersXml>,
    #[serde(rename = "Frames")]
    pub frames: Option<FramesXml>,
}

/// Size definition.
#[derive(Debug, Deserialize)]
pub struct SizeXml {
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@y")]
    pub y: Option<f32>,
    #[serde(rename = "AbsDimension")]
    pub abs_dimension: Option<AbsDimensionXml>,
}

#[derive(Debug, Deserialize)]
pub struct AbsDimensionXml {
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@y")]
    pub y: Option<f32>,
}

/// Anchors container.
#[derive(Debug, Deserialize)]
pub struct AnchorsXml {
    #[serde(rename = "Anchor", default)]
    pub anchors: Vec<AnchorXml>,
}

/// Single anchor definition.
#[derive(Debug, Deserialize)]
pub struct AnchorXml {
    #[serde(rename = "@point")]
    pub point: String,
    #[serde(rename = "@relativeTo")]
    pub relative_to: Option<String>,
    #[serde(rename = "@relativePoint")]
    pub relative_point: Option<String>,
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@y")]
    pub y: Option<f32>,
    #[serde(rename = "Offset")]
    pub offset: Option<OffsetXml>,
}

#[derive(Debug, Deserialize)]
pub struct OffsetXml {
    #[serde(rename = "AbsDimension")]
    pub abs_dimension: Option<AbsDimensionXml>,
}

/// Scripts container.
#[derive(Debug, Deserialize)]
pub struct ScriptsXml {
    #[serde(rename = "OnLoad")]
    pub on_load: Option<ScriptBodyXml>,
    #[serde(rename = "OnEvent")]
    pub on_event: Option<ScriptBodyXml>,
    #[serde(rename = "OnUpdate")]
    pub on_update: Option<ScriptBodyXml>,
    #[serde(rename = "OnClick")]
    pub on_click: Option<ScriptBodyXml>,
    #[serde(rename = "OnShow")]
    pub on_show: Option<ScriptBodyXml>,
    #[serde(rename = "OnHide")]
    pub on_hide: Option<ScriptBodyXml>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptBodyXml {
    #[serde(rename = "$text")]
    pub body: Option<String>,
    #[serde(rename = "@function")]
    pub function: Option<String>,
}

/// Layers container (for textures and font strings).
#[derive(Debug, Deserialize)]
pub struct LayersXml {
    #[serde(rename = "Layer", default)]
    pub layers: Vec<LayerXml>,
}

#[derive(Debug, Deserialize)]
pub struct LayerXml {
    #[serde(rename = "@level")]
    pub level: Option<String>,
    #[serde(rename = "Texture", default)]
    pub textures: Vec<TextureXml>,
    #[serde(rename = "FontString", default)]
    pub font_strings: Vec<FontStringXml>,
}

#[derive(Debug, Deserialize)]
pub struct TextureXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@file")]
    pub file: Option<String>,
    #[serde(rename = "Size")]
    pub size: Option<SizeXml>,
    #[serde(rename = "Anchors")]
    pub anchors: Option<AnchorsXml>,
}

#[derive(Debug, Deserialize)]
pub struct FontStringXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "Size")]
    pub size: Option<SizeXml>,
    #[serde(rename = "Anchors")]
    pub anchors: Option<AnchorsXml>,
}

/// Child frames container.
#[derive(Debug, Deserialize)]
pub struct FramesXml {
    #[serde(rename = "Frame", default)]
    pub frames: Vec<FrameXml>,
    #[serde(rename = "Button", default)]
    pub buttons: Vec<FrameXml>,
}

/// Script include.
#[derive(Debug, Deserialize)]
pub struct ScriptXml {
    #[serde(rename = "@file")]
    pub file: String,
}

/// XML include.
#[derive(Debug, Deserialize)]
pub struct IncludeXml {
    #[serde(rename = "@file")]
    pub file: String,
}

/// Parse a WoW UI XML file.
pub fn parse_xml(xml: &str) -> Result<UiXml, quick_xml::DeError> {
    quick_xml::de::from_str(xml)
}

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
