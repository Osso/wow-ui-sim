//! XML parsing for WoW UI definition files.

use serde::Deserialize;

/// Root element of a WoW UI XML file.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename = "Ui")]
pub struct UiXml {
    #[serde(rename = "$value", default)]
    pub elements: Vec<XmlElement>,
}

/// XML elements that can appear in a UI definition.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum XmlElement {
    // Frame-like widgets
    Frame(FrameXml),
    Button(FrameXml),
    ItemButton(FrameXml),
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
    // File references (both uppercase and lowercase variants for compatibility)
    Script(ScriptXml),
    #[serde(rename = "script")]
    ScriptLower(ScriptXml),
    Include(IncludeXml),
    #[serde(rename = "include")]
    IncludeLower(IncludeXml),
    // Animation elements
    AnimationGroup(AnimationGroupXml),
    // ModelScene elements
    Actor(ActorXml),
    // Font definitions
    Font(FontXml),
    FontFamily(FontFamilyXml),
    // Text content (from malformed XML or comments)
    #[serde(rename = "$text")]
    Text(String),
}

/// Frame definition in XML.
#[derive(Debug, Deserialize, Default, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Default, Clone)]
pub struct AnimationsXml {
    #[serde(rename = "$value", default)]
    pub animations: Vec<AnimationGroupXml>,
}

/// Font reference (for NormalFont, HighlightFont, etc.)
#[derive(Debug, Deserialize, Default, Clone)]
pub struct FontRefXml {
    #[serde(rename = "@style")]
    pub style: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
}

/// ScrollChild element.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ScrollChildXml {
    #[serde(rename = "$value", default)]
    pub children: Vec<FrameElement>,
}

/// Backdrop element (legacy).
#[derive(Debug, Deserialize, Default, Clone)]
pub struct BackdropXml {
    #[serde(rename = "@bgFile")]
    pub bg_file: Option<String>,
    #[serde(rename = "@edgeFile")]
    pub edge_file: Option<String>,
    #[serde(rename = "@tile")]
    pub tile: Option<bool>,
}

/// ResizeBounds element.
#[derive(Debug, Deserialize, Default, Clone)]
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
#[derive(Debug, Deserialize, Default, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
pub struct SizeXml {
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@y")]
    pub y: Option<f32>,
    #[serde(rename = "AbsDimension")]
    pub abs_dimension: Option<AbsDimensionXml>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AbsDimensionXml {
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@y")]
    pub y: Option<f32>,
}

/// Anchors container.
#[derive(Debug, Deserialize, Clone)]
pub struct AnchorsXml {
    #[serde(rename = "Anchor", default)]
    pub anchors: Vec<AnchorXml>,
}

/// Single anchor definition.
#[derive(Debug, Deserialize, Clone)]
pub struct AnchorXml {
    #[serde(rename = "@point")]
    pub point: String,
    #[serde(rename = "@relativeTo")]
    pub relative_to: Option<String>,
    /// Relative key like "$parent.ScrollBox" or "$parent.Performance"
    #[serde(rename = "@relativeKey")]
    pub relative_key: Option<String>,
    #[serde(rename = "@relativePoint")]
    pub relative_point: Option<String>,
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@y")]
    pub y: Option<f32>,
    #[serde(rename = "Offset")]
    pub offset: Option<OffsetXml>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OffsetXml {
    #[serde(rename = "AbsDimension")]
    pub abs_dimension: Option<AbsDimensionXml>,
}

/// Scripts container.
/// Note: Some scripts can appear multiple times (e.g., duplicate OnClick in Baganator),
/// so we use Vec to allow this. WoW likely uses the last one or merges them.
#[derive(Debug, Deserialize, Clone)]
pub struct ScriptsXml {
    #[serde(rename = "OnLoad", default)]
    pub on_load: Vec<ScriptBodyXml>,
    #[serde(rename = "OnEvent", default)]
    pub on_event: Vec<ScriptBodyXml>,
    #[serde(rename = "OnUpdate", default)]
    pub on_update: Vec<ScriptBodyXml>,
    #[serde(rename = "OnClick", default)]
    pub on_click: Vec<ScriptBodyXml>,
    #[serde(rename = "OnShow", default)]
    pub on_show: Vec<ScriptBodyXml>,
    #[serde(rename = "OnHide", default)]
    pub on_hide: Vec<ScriptBodyXml>,
}

#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
pub struct KeyValuesXml {
    #[serde(rename = "KeyValue", default)]
    pub values: Vec<KeyValueXml>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeyValueXml {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(rename = "@type")]
    pub value_type: Option<String>,
}

/// Layers container (for textures and font strings).
#[derive(Debug, Deserialize, Clone)]
pub struct LayersXml {
    #[serde(rename = "Layer", default)]
    pub layers: Vec<LayerXml>,
}

#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum LayerElement {
    Texture(TextureXml),
    FontString(FontStringXml),
    Line(TextureXml),
    MaskTexture(TextureXml),
}

#[derive(Debug, Deserialize, Default, Clone)]
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

#[derive(Debug, Deserialize, Default, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
pub struct FramesXml {
    #[serde(rename = "$value", default)]
    pub elements: Vec<FrameElement>,
}

/// Frame-like elements that can appear inside a <Frames> container.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum FrameElement {
    Frame(FrameXml),
    Button(FrameXml),
    ItemButton(FrameXml),
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
#[derive(Debug, Deserialize, Clone)]
pub struct ScriptXml {
    #[serde(rename = "@file")]
    pub file: Option<String>,
    #[serde(rename = "$text")]
    pub inline: Option<String>,
}

/// XML include.
#[derive(Debug, Deserialize, Clone)]
pub struct IncludeXml {
    #[serde(rename = "@file")]
    pub file: String,
}

/// Animation group definition.
#[derive(Debug, Deserialize, Default, Clone)]
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
#[derive(Debug, Deserialize, Default, Clone)]
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
#[derive(Debug, Deserialize, Default, Clone)]
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

/// FontFamily definition - collection of fonts for different alphabets.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct FontFamilyXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    // Contains Member elements with Font children, which we ignore for simulation
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

// ========== Template Registry ==========

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// Stores a template (virtual frame) with its widget type.
#[derive(Debug, Clone)]
pub struct TemplateEntry {
    pub name: String,
    pub widget_type: String,
    pub frame: FrameXml,
}

/// Global registry of XML templates (virtual frames).
fn template_registry() -> &'static RwLock<HashMap<String, TemplateEntry>> {
    static REGISTRY: OnceLock<RwLock<HashMap<String, TemplateEntry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Register a template (virtual frame) in the global registry.
pub fn register_template(name: &str, widget_type: &str, frame: FrameXml) {
    let mut registry = template_registry().write().unwrap();
    registry.insert(name.to_string(), TemplateEntry {
        name: name.to_string(),
        widget_type: widget_type.to_string(),
        frame,
    });
}

/// Get a template by name from the registry.
pub fn get_template(name: &str) -> Option<TemplateEntry> {
    let registry = template_registry().read().unwrap();
    registry.get(name).cloned()
}

/// Template info for C_XMLUtil.GetTemplateInfo.
pub struct TemplateInfo {
    pub frame_type: String,
    pub width: f32,
    pub height: f32,
}

/// Get template info (type, width, height) by resolving inheritance chain.
pub fn get_template_info(name: &str) -> Option<TemplateInfo> {
    let chain = get_template_chain(name);
    if chain.is_empty() {
        return None;
    }

    // Get the widget type from the first entry that defines it
    let frame_type = chain.iter()
        .find(|e| !e.widget_type.is_empty())
        .map(|e| e.widget_type.clone())
        .unwrap_or_else(|| "Frame".to_string());

    // Resolve size by looking through inheritance chain (most derived wins)
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;

    for entry in &chain {
        if let Some(size) = entry.frame.size() {
            // Check AbsDimension first, then direct attributes
            if let Some(ref abs) = size.abs_dimension {
                if let Some(x) = abs.x {
                    width = x;
                }
                if let Some(y) = abs.y {
                    height = y;
                }
            }
            if let Some(x) = size.x {
                width = x;
            }
            if let Some(y) = size.y {
                height = y;
            }
        }
    }

    Some(TemplateInfo {
        frame_type,
        width,
        height,
    })
}

/// Get the full inheritance chain for a template (including the template itself).
/// Returns templates in order from most base to most derived.
pub fn get_template_chain(names: &str) -> Vec<TemplateEntry> {
    let mut chain = Vec::new();
    let mut visited = std::collections::HashSet::new();

    // Process comma-separated template names
    for name in names.split(',').map(|s| s.trim()) {
        if name.is_empty() || visited.contains(name) {
            continue;
        }
        collect_template_chain(name, &mut chain, &mut visited);
    }

    chain
}

/// Recursively collect templates in the inheritance chain.
fn collect_template_chain(
    name: &str,
    chain: &mut Vec<TemplateEntry>,
    visited: &mut std::collections::HashSet<String>,
) {
    if visited.contains(name) {
        return;
    }
    visited.insert(name.to_string());

    if let Some(entry) = get_template(name) {
        // First, process parent templates (if this template inherits from others)
        if let Some(ref inherits) = entry.frame.inherits {
            for parent in inherits.split(',').map(|s| s.trim()) {
                if !parent.is_empty() {
                    collect_template_chain(parent, chain, visited);
                }
            }
        }
        // Then add this template
        chain.push(entry);
    }
}

/// Clear the template registry (useful for testing).
#[allow(dead_code)]
pub fn clear_templates() {
    let mut registry = template_registry().write().unwrap();
    registry.clear();
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
