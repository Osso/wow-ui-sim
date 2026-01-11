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
    // Frame-like widgets
    Frame(FrameXml),
    Button(FrameXml),
    CheckButton(FrameXml),
    EditBox(FrameXml),
    ScrollFrame(FrameXml),
    Slider(FrameXml),
    StatusBar(FrameXml),
    GameTooltip(FrameXml),
    ColorSelect(FrameXml),
    Model(FrameXml),
    ModelScene(FrameXml),
    EventFrame(FrameXml),
    CinematicModel(FrameXml),
    PlayerModel(FrameXml),
    DressUpModel(FrameXml),
    Browser(FrameXml),
    Minimap(FrameXml),
    MessageFrame(FrameXml),
    MovieFrame(FrameXml),
    ScrollingMessageFrame(FrameXml),
    SimpleHTML(FrameXml),
    WorldFrame(FrameXml),
    DropDownToggleButton(FrameXml),
    DropdownButton(FrameXml),
    EventButton(FrameXml),
    EventEditBox(FrameXml),
    Cooldown(FrameXml),
    // Texture/Font regions
    Texture(TextureXml),
    FontString(FontStringXml),
    // File references
    Script(ScriptXml),
    Include(IncludeXml),
    // Animation elements
    AnimationGroup(AnimationGroupXml),
    // ModelScene elements
    Actor(ActorXml),
    // Font definitions
    Font(FontXml),
}

/// Frame definition in XML.
#[derive(Debug, Deserialize, Default)]
pub struct FrameXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parent")]
    pub parent: Option<String>,
    #[serde(rename = "@parentKey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@hidden")]
    pub hidden: Option<bool>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@intrinsic")]
    pub intrinsic: Option<bool>,
    #[serde(rename = "@propagateMouseInput")]
    pub propagate_mouse_input: Option<String>,

    // Child elements collected via $value to allow multiples
    #[serde(rename = "$value", default)]
    pub children: Vec<FrameChildElement>,
}

impl FrameXml {
    /// Get the Size element if present.
    pub fn size(&self) -> Option<&SizeXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::Size(s) => Some(s),
            _ => None,
        })
    }

    /// Get the Anchors element if present.
    pub fn anchors(&self) -> Option<&AnchorsXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::Anchors(a) => Some(a),
            _ => None,
        })
    }

    /// Get the Scripts element if present.
    pub fn scripts(&self) -> Option<&ScriptsXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::Scripts(s) => Some(s),
            _ => None,
        })
    }

    /// Get all Layers elements (there can be multiple).
    pub fn layers(&self) -> impl Iterator<Item = &LayersXml> {
        self.children.iter().filter_map(|c| match c {
            FrameChildElement::Layers(l) => Some(l),
            _ => None,
        })
    }

    /// Get the Frames element if present.
    pub fn frames(&self) -> Option<&FramesXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::Frames(f) => Some(f),
            _ => None,
        })
    }

    /// Get the KeyValues element if present.
    pub fn key_values(&self) -> Option<&KeyValuesXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::KeyValues(k) => Some(k),
            _ => None,
        })
    }
}

/// Child elements that can appear inside a Frame.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FrameChildElement {
    Size(SizeXml),
    Anchors(AnchorsXml),
    Scripts(ScriptsXml),
    Layers(LayersXml),
    Frames(FramesXml),
    KeyValues(KeyValuesXml),
    Animations(AnimationsXml),
    // Button-specific
    NormalTexture(TextureXml),
    PushedTexture(TextureXml),
    DisabledTexture(TextureXml),
    HighlightTexture(TextureXml),
    CheckedTexture(TextureXml),
    ButtonText(FontStringXml),
    NormalFont(FontRefXml),
    HighlightFont(FontRefXml),
    DisabledFont(FontRefXml),
    // EditBox-specific
    FontString(FontStringXml),
    // ScrollFrame-specific
    ScrollChild(ScrollChildXml),
    // Slider-specific
    ThumbTexture(TextureXml),
    // StatusBar-specific
    BarTexture(TextureXml),
    BarColor(ColorXml),
    // Backdrop (legacy)
    Backdrop(BackdropXml),
    // Resize bounds
    ResizeBounds(ResizeBoundsXml),
    // Hit rect insets
    HitRectInsets(InsetsXml),
    // Additional elements we may encounter
    #[serde(other)]
    Unknown,
}

/// Animations container.
#[derive(Debug, Deserialize, Default)]
pub struct AnimationsXml {
    #[serde(rename = "$value", default)]
    pub animations: Vec<AnimationGroupXml>,
}

/// Font reference (for NormalFont, HighlightFont, etc.)
#[derive(Debug, Deserialize, Default)]
pub struct FontRefXml {
    #[serde(rename = "@style")]
    pub style: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
}

/// ScrollChild element.
#[derive(Debug, Deserialize, Default)]
pub struct ScrollChildXml {
    #[serde(rename = "$value", default)]
    pub children: Vec<FrameElement>,
}

/// Backdrop element (legacy).
#[derive(Debug, Deserialize, Default)]
pub struct BackdropXml {
    #[serde(rename = "@bgFile")]
    pub bg_file: Option<String>,
    #[serde(rename = "@edgeFile")]
    pub edge_file: Option<String>,
    #[serde(rename = "@tile")]
    pub tile: Option<bool>,
}

/// ResizeBounds element.
#[derive(Debug, Deserialize, Default)]
pub struct ResizeBoundsXml {
    #[serde(rename = "@minWidth")]
    pub min_width: Option<f32>,
    #[serde(rename = "@minHeight")]
    pub min_height: Option<f32>,
    #[serde(rename = "@maxWidth")]
    pub max_width: Option<f32>,
    #[serde(rename = "@maxHeight")]
    pub max_height: Option<f32>,
}

/// Insets element.
#[derive(Debug, Deserialize, Default)]
pub struct InsetsXml {
    #[serde(rename = "@left")]
    pub left: Option<f32>,
    #[serde(rename = "@right")]
    pub right: Option<f32>,
    #[serde(rename = "@top")]
    pub top: Option<f32>,
    #[serde(rename = "@bottom")]
    pub bottom: Option<f32>,
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
    #[serde(rename = "@method")]
    pub method: Option<String>,
    #[serde(rename = "@inherit")]
    pub inherit: Option<String>,
    #[serde(rename = "@intrinsicOrder")]
    pub intrinsic_order: Option<String>,
}

/// KeyValues container for custom properties.
#[derive(Debug, Deserialize)]
pub struct KeyValuesXml {
    #[serde(rename = "KeyValue", default)]
    pub values: Vec<KeyValueXml>,
}

#[derive(Debug, Deserialize)]
pub struct KeyValueXml {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(rename = "@type")]
    pub value_type: Option<String>,
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
    #[serde(rename = "$value", default)]
    pub elements: Vec<LayerElement>,
}

impl LayerXml {
    /// Get all Texture elements in this layer.
    pub fn textures(&self) -> impl Iterator<Item = &TextureXml> {
        self.elements.iter().filter_map(|e| match e {
            LayerElement::Texture(t) => Some(t),
            _ => None,
        })
    }

    /// Get all FontString elements in this layer.
    pub fn font_strings(&self) -> impl Iterator<Item = &FontStringXml> {
        self.elements.iter().filter_map(|e| match e {
            LayerElement::FontString(f) => Some(f),
            _ => None,
        })
    }
}

/// Elements that can appear inside a Layer.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum LayerElement {
    Texture(TextureXml),
    FontString(FontStringXml),
    Line(TextureXml),
    MaskTexture(TextureXml),
}

#[derive(Debug, Deserialize, Default)]
pub struct TextureXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parentKey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@file")]
    pub file: Option<String>,
    #[serde(rename = "@atlas")]
    pub atlas: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@texelSnappingBias")]
    pub texel_snapping_bias: Option<String>,
    #[serde(rename = "@snapToPixelGrid")]
    pub snap_to_pixel_grid: Option<String>,
    #[serde(rename = "Size")]
    pub size: Option<SizeXml>,
    #[serde(rename = "Anchors")]
    pub anchors: Option<AnchorsXml>,
    #[serde(rename = "Color")]
    pub color: Option<ColorXml>,
    #[serde(rename = "Scripts")]
    pub scripts: Option<ScriptsXml>,
}

#[derive(Debug, Deserialize, Default)]
pub struct FontStringXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parentKey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@justifyH")]
    pub justify_h: Option<String>,
    #[serde(rename = "@justifyV")]
    pub justify_v: Option<String>,
    #[serde(rename = "Size")]
    pub size: Option<SizeXml>,
    #[serde(rename = "Anchors")]
    pub anchors: Option<AnchorsXml>,
    #[serde(rename = "Scripts")]
    pub scripts: Option<ScriptsXml>,
}

#[derive(Debug, Deserialize)]
pub struct ColorXml {
    #[serde(rename = "@r")]
    pub r: Option<f32>,
    #[serde(rename = "@g")]
    pub g: Option<f32>,
    #[serde(rename = "@b")]
    pub b: Option<f32>,
    #[serde(rename = "@a")]
    pub a: Option<f32>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
}

/// Child frames container - can contain any frame-like element.
#[derive(Debug, Deserialize)]
pub struct FramesXml {
    #[serde(rename = "$value", default)]
    pub elements: Vec<FrameElement>,
}

/// Frame-like elements that can appear inside a <Frames> container.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FrameElement {
    Frame(FrameXml),
    Button(FrameXml),
    CheckButton(FrameXml),
    EditBox(FrameXml),
    ScrollFrame(FrameXml),
    Slider(FrameXml),
    StatusBar(FrameXml),
    GameTooltip(FrameXml),
    ColorSelect(FrameXml),
    Model(FrameXml),
    ModelScene(FrameXml),
    EventFrame(FrameXml),
    CinematicModel(FrameXml),
    PlayerModel(FrameXml),
    DressUpModel(FrameXml),
    Browser(FrameXml),
    Minimap(FrameXml),
    MessageFrame(FrameXml),
    MovieFrame(FrameXml),
    ScrollingMessageFrame(FrameXml),
    SimpleHTML(FrameXml),
    WorldFrame(FrameXml),
    DropDownToggleButton(FrameXml),
    DropdownButton(FrameXml),
    EventButton(FrameXml),
    EventEditBox(FrameXml),
    Cooldown(FrameXml),
}

/// Script include (file attribute is optional for inline scripts).
#[derive(Debug, Deserialize)]
pub struct ScriptXml {
    #[serde(rename = "@file")]
    pub file: Option<String>,
    #[serde(rename = "$text")]
    pub inline: Option<String>,
}

/// XML include.
#[derive(Debug, Deserialize)]
pub struct IncludeXml {
    #[serde(rename = "@file")]
    pub file: String,
}

/// Animation group definition.
#[derive(Debug, Deserialize, Default)]
pub struct AnimationGroupXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parentKey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@setToFinalAlpha")]
    pub set_to_final_alpha: Option<bool>,
    #[serde(rename = "@looping")]
    pub looping: Option<String>,
    // Animation children would go here
}

/// Actor definition for ModelScene.
#[derive(Debug, Deserialize, Default)]
pub struct ActorXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parentKey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
}

/// Font definition.
#[derive(Debug, Deserialize, Default)]
pub struct FontXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@font")]
    pub font: Option<String>,
    #[serde(rename = "@height")]
    pub height: Option<f32>,
    #[serde(rename = "@outline")]
    pub outline: Option<String>,
}

/// Parse a WoW UI XML file from a string.
pub fn parse_xml(xml: &str) -> Result<UiXml, quick_xml::DeError> {
    quick_xml::de::from_str(xml)
}

/// Parse a WoW UI XML file from disk.
pub fn parse_xml_file(path: &std::path::Path) -> Result<UiXml, XmlLoadError> {
    let contents = std::fs::read_to_string(path)?;
    Ok(parse_xml(&contents)?)
}

/// Error type for XML loading.
#[derive(Debug)]
pub enum XmlLoadError {
    Io(std::io::Error),
    Parse(quick_xml::DeError),
}

impl From<std::io::Error> for XmlLoadError {
    fn from(e: std::io::Error) -> Self {
        XmlLoadError::Io(e)
    }
}

impl From<quick_xml::DeError> for XmlLoadError {
    fn from(e: quick_xml::DeError) -> Self {
        XmlLoadError::Parse(e)
    }
}

impl std::fmt::Display for XmlLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmlLoadError::Io(e) => write!(f, "IO error: {}", e),
            XmlLoadError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for XmlLoadError {}

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
