//! XML element type definitions: layers, textures, fontstrings, etc.

use serde::{Deserialize, Deserializer};

use super::types::FrameXml;
use super::types_support::{
    AbsDimensionXml, AnchorsXml, AnimationsXml, ColorXml, KeyValuesXml, ScriptsXml, SizeXml,
};

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
    #[serde(rename = "@textureSubLevel", default)]
    pub texture_sub_level: Option<i32>,
    #[serde(rename = "$value", default)]
    pub elements: Vec<LayerElement>,
}

impl LayerXml {
    /// Get all Texture elements in this layer (includes MaskTextures — they
    /// need to exist as child widgets so Lua code can reference them via parentKey).
    /// Returns (texture, is_mask, is_line) triples.
    pub fn textures(&self) -> impl Iterator<Item = (&TextureXml, bool, bool)> {
        self.elements.iter().filter_map(|e| match e {
            LayerElement::Texture(t) => Some((t, false, false)),
            LayerElement::Line(t) => Some((t, false, true)),
            LayerElement::MaskTexture(t) => Some((t, true, false)),
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
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@parentKey", alias = "@parentkey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@file")]
    pub file: Option<String>,
    #[serde(rename = "@atlas")]
    pub atlas: Option<String>,
    #[serde(rename = "@useAtlasSize")]
    pub use_atlas_size: Option<bool>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@texelSnappingBias")]
    pub texel_snapping_bias: Option<String>,
    #[serde(rename = "@snapToPixelGrid")]
    pub snap_to_pixel_grid: Option<String>,
    #[serde(rename = "@horizTile")]
    pub horiz_tile: Option<bool>,
    #[serde(rename = "@vertTile")]
    pub vert_tile: Option<bool>,
    #[serde(rename = "@hWrapMode")]
    pub h_wrap_mode: Option<String>,
    #[serde(rename = "@vWrapMode")]
    pub v_wrap_mode: Option<String>,
    #[serde(rename = "@thickness")]
    pub thickness: Option<f32>,
    #[serde(rename = "@hidden")]
    pub hidden: Option<bool>,
    #[serde(rename = "@alpha")]
    pub alpha: Option<f32>,
    #[serde(rename = "@alphaMode")]
    pub alpha_mode: Option<String>,
    #[serde(rename = "@blendMode")]
    pub blend_mode: Option<String>,
    #[serde(rename = "@setAllPoints")]
    pub set_all_points: Option<bool>,
    #[serde(rename = "@parentArray")]
    pub parent_array: Option<String>,
    #[serde(rename = "Size")]
    pub size: Option<SizeXml>,
    #[serde(rename = "Anchors")]
    pub anchors: Option<AnchorsXml>,
    #[serde(rename = "Color")]
    pub color: Option<ColorXml>,
    #[serde(rename = "Gradient")]
    pub gradient: Option<crate::xml::GradientXml>,
    #[serde(rename = "Animations")]
    pub animations: Option<AnimationsXml>,
    #[serde(rename = "Scripts")]
    pub scripts: Option<ScriptsXml>,
    #[serde(rename = "KeyValues")]
    pub key_values: Option<KeyValuesXml>,
    /// Texture coordinates (left, right, top, bottom) for UV mapping.
    #[serde(rename = "TexCoords")]
    pub tex_coords: Option<TexCoordsXml>,
    /// MaskedTextures — declares which sibling textures this mask applies to.
    #[serde(rename = "MaskedTextures")]
    pub masked_textures: Option<MaskedTexturesXml>,
}

impl TextureXml {
    /// Effective blend mode from either WoW XML spelling.
    pub fn effective_blend_mode(&self) -> Option<&str> {
        self.blend_mode.as_deref().or(self.alpha_mode.as_deref())
    }

    pub fn wants_horiz_tile(&self) -> bool {
        self.horiz_tile == Some(true)
            || self
                .h_wrap_mode
                .as_deref()
                .is_some_and(|mode| mode.eq_ignore_ascii_case("REPEAT"))
    }

    pub fn wants_vert_tile(&self) -> bool {
        self.vert_tile == Some(true)
            || self
                .v_wrap_mode
                .as_deref()
                .is_some_and(|mode| mode.eq_ignore_ascii_case("REPEAT"))
    }
}

/// TexCoords element with left/right/top/bottom UV coordinates.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct TexCoordsXml {
    #[serde(rename = "@left")]
    pub left: Option<f32>,
    #[serde(rename = "@right")]
    pub right: Option<f32>,
    #[serde(rename = "@top")]
    pub top: Option<f32>,
    #[serde(rename = "@bottom")]
    pub bottom: Option<f32>,
    #[serde(rename = "Rect")]
    pub rect: Option<TexCoordsRectXml>,
}

/// Rect child for TexCoords — corner-based UV coordinates.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct TexCoordsRectXml {
    #[serde(rename = "@ULx")]
    pub ul_x: Option<f32>,
    #[serde(rename = "@ULy")]
    pub ul_y: Option<f32>,
    #[serde(rename = "@LLx")]
    pub ll_x: Option<f32>,
    #[serde(rename = "@LLy")]
    pub ll_y: Option<f32>,
    #[serde(rename = "@URx")]
    pub ur_x: Option<f32>,
    #[serde(rename = "@URy")]
    pub ur_y: Option<f32>,
    #[serde(rename = "@LRx")]
    pub lr_x: Option<f32>,
    #[serde(rename = "@LRy")]
    pub lr_y: Option<f32>,
}

/// Container for MaskedTexture entries inside a MaskTexture element.
#[derive(Debug, Default, Clone)]
pub struct MaskedTexturesXml {
    pub entries: Vec<MaskedTextureEntryXml>,
}

impl<'de> Deserialize<'de> for MaskedTexturesXml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMaskedTexturesXml {
            #[serde(rename = "$value", default)]
            nodes: Vec<MaskedTextureNodeXml>,
        }

        let raw = RawMaskedTexturesXml::deserialize(deserializer)?;
        let entries = raw
            .nodes
            .into_iter()
            .filter_map(|node| match node {
                MaskedTextureNodeXml::MaskedTexture(entry) => Some(entry),
                MaskedTextureNodeXml::Text(text) => {
                    drop(text);
                    None
                }
            })
            .collect();

        Ok(Self { entries })
    }
}

#[derive(Debug, Deserialize, Clone)]
enum MaskedTextureNodeXml {
    #[serde(rename = "MaskedTexture")]
    MaskedTexture(MaskedTextureEntryXml),
    #[serde(rename = "$text")]
    Text(String),
}

/// A single MaskedTexture entry referencing a sibling texture by childKey.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct MaskedTextureEntryXml {
    #[serde(rename = "@childKey")]
    pub child_key: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FontStringXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parentKey", alias = "@parentkey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@font")]
    pub font: Option<String>,
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@hidden")]
    pub hidden: Option<bool>,
    #[serde(rename = "@alpha")]
    pub alpha: Option<f32>,
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@justifyH")]
    pub justify_h: Option<String>,
    #[serde(rename = "@justifyV")]
    pub justify_v: Option<String>,
    #[serde(rename = "Size", default)]
    pub size: Vec<SizeXml>,
    #[serde(rename = "Anchors")]
    pub anchors: Option<AnchorsXml>,
    #[serde(rename = "Color")]
    pub color: Option<ColorXml>,
    #[serde(rename = "Shadow")]
    pub shadow: Option<ShadowXml>,
    #[serde(rename = "Scripts")]
    pub scripts: Option<ScriptsXml>,
    #[serde(rename = "KeyValues")]
    pub key_values: Option<KeyValuesXml>,
    #[serde(rename = "@setAllPoints")]
    pub set_all_points: Option<bool>,
    #[serde(rename = "@parentArray")]
    pub parent_array: Option<String>,
    #[serde(rename = "@wordwrap")]
    pub word_wrap: Option<bool>,
    #[serde(rename = "@maxLines")]
    pub max_lines: Option<u32>,
    #[serde(rename = "FontHeight")]
    pub font_height: Option<FontHeightXml>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FontHeightXml {
    #[serde(rename = "@val")]
    pub val: Option<f32>,
    #[serde(rename = "AbsValue")]
    pub abs_value: Option<AbsValueXml>,
}

impl FontHeightXml {
    pub fn value(&self) -> Option<f32> {
        self.val.or_else(|| self.abs_value.as_ref()?.val)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AbsValueXml {
    #[serde(rename = "@val")]
    pub val: Option<f32>,
}

/// Shadow element for FontStrings - contains offset and color.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ShadowXml {
    #[serde(rename = "Offset")]
    pub offset: Option<ShadowOffsetXml>,
    #[serde(rename = "Color")]
    pub color: Option<ColorXml>,
}

/// Shadow offset - can have direct x/y attributes or nested AbsDimension.
#[derive(Debug, Deserialize, Clone)]
pub struct ShadowOffsetXml {
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@y")]
    pub y: Option<f32>,
    #[serde(rename = "AbsDimension")]
    pub abs_dimension: Option<AbsDimensionXml>,
}

impl ShadowOffsetXml {
    /// Get the x offset, preferring direct attribute over nested AbsDimension.
    pub fn x(&self) -> f32 {
        self.x
            .or_else(|| self.abs_dimension.as_ref().and_then(|d| d.x))
            .unwrap_or(0.0)
    }

    /// Get the y offset, preferring direct attribute over nested AbsDimension.
    pub fn y(&self) -> f32 {
        self.y
            .or_else(|| self.abs_dimension.as_ref().and_then(|d| d.y))
            .unwrap_or(0.0)
    }
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
    #[cfg(feature = "aura-xml-widgets")]
    AuraContainer(FrameXml),
    #[cfg(feature = "aura-xml-widgets")]
    ManagedAuraContainer(FrameXml),
    Button(FrameXml),
    #[cfg(feature = "aura-xml-widgets")]
    AuraButton(FrameXml),
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
    TaxiRouteFrame(FrameXml),
    ModelFFX(FrameXml),
    TabardModel(FrameXml),
    UiCamera(FrameXml),
    UnitPositionFrame(FrameXml),
    OffScreenFrame(FrameXml),
    Checkout(FrameXml),
    FogOfWarFrame(FrameXml),
    QuestPOIFrame(FrameXml),
    ArchaeologyDigSiteFrame(FrameXml),
    ScenarioPOIFrame(FrameXml),
    UIThemeContainerFrame(FrameXml),
    EventScrollFrame(FrameXml),
    ContainedAlertFrame(FrameXml),
    MapScene(FrameXml),
    ScopedModifier(super::types::ScopedModifierXml),
    Line(FrameXml),
}

fn preserved_frame_alias_type(tag: &str) -> Option<&'static str> {
    match tag {
        "EventFrame" => Some("EventFrame"),
        "UnitPositionFrame" => Some("UnitPositionFrame"),
        "OffScreenFrame" => Some("OffScreenFrame"),
        "Checkout" => Some("Checkout"),
        "FogOfWarFrame" => Some("FogOfWarFrame"),
        "QuestPOIFrame" => Some("QuestPOIFrame"),
        "ArchaeologyDigSiteFrame" => Some("ArchaeologyDigSiteFrame"),
        "ScenarioPOIFrame" => Some("ScenarioPOIFrame"),
        "Browser" => Some("Browser"),
        "MovieFrame" => Some("MovieFrame"),
        _ => None,
    }
}

type WidgetMapping = (&'static str, Option<&'static str>);
type WidgetTagMapping = (&'static str, WidgetMapping);

const DIRECT_WIDGET_MAPPINGS: &[WidgetTagMapping] = &[
    ("Frame", ("Frame", None)),
    #[cfg(feature = "aura-xml-widgets")]
    ("AuraContainer", ("Frame", Some("AuraContainer"))),
    #[cfg(feature = "aura-xml-widgets")]
    (
        "ManagedAuraContainer",
        ("Frame", Some("ManagedAuraContainer")),
    ),
    ("Button", ("Button", None)),
    #[cfg(feature = "aura-xml-widgets")]
    ("AuraButton", ("Button", Some("AuraButton"))),
    ("ItemButton", ("Button", Some("ItemButton"))),
    ("CheckButton", ("CheckButton", None)),
    ("EditBox", ("EditBox", None)),
    ("EventEditBox", ("EditBox", Some("EventEditBox"))),
    ("ScrollFrame", ("ScrollFrame", None)),
    (
        "EventScrollFrame",
        ("ScrollFrame", Some("EventScrollFrame")),
    ),
    ("Slider", ("Slider", None)),
    ("StatusBar", ("StatusBar", None)),
    ("Cooldown", ("Cooldown", None)),
    ("GameTooltip", ("GameTooltip", None)),
    ("ColorSelect", ("ColorSelect", None)),
    ("Model", ("Model", None)),
    ("ModelScene", ("ModelScene", None)),
    ("MessageFrame", ("MessageFrame", None)),
    (
        "ScrollingMessageFrame",
        ("MessageFrame", Some("ScrollingMessageFrame")),
    ),
    ("SimpleHTML", ("SimpleHTML", None)),
    ("Minimap", ("Minimap", None)),
    ("DropdownButton", ("Button", Some("DropdownButton"))),
    (
        "DropDownToggleButton",
        ("Button", Some("DropDownToggleButton")),
    ),
    ("EventButton", ("Button", Some("EventButton"))),
    (
        "ContainedAlertFrame",
        ("Button", Some("ContainedAlertFrame")),
    ),
];

fn direct_widget_mapping(tag: &str) -> Option<(&'static str, Option<&'static str>)> {
    DIRECT_WIDGET_MAPPINGS
        .iter()
        .find_map(|(mapped_tag, mapping)| (*mapped_tag == tag).then_some(*mapping))
}

fn is_player_model_family_tag(tag: &str) -> bool {
    matches!(
        tag,
        "PlayerModel" | "CinematicModel" | "TabardModel" | "DressUpModel"
    )
}

fn is_frame_fallback_tag(tag: &str) -> bool {
    matches!(
        tag,
        "TaxiRouteFrame"
            | "ModelFFX"
            | "UiCamera"
            | "UIThemeContainerFrame"
            | "MapScene"
            | "Line"
            | "WorldFrame"
    )
}

/// Shared mapping from XML element tag name to `(widget_type, intrinsic_name)`.
///
/// Covers the common mappings used by both `FrameElement` (inside `<Frames>`)
/// and `XmlElement` (top-level). Callers handle divergences before calling this.
pub fn widget_type_for_tag(tag: &str) -> Option<(&'static str, Option<&'static str>)> {
    if let Some(widget_type) = preserved_frame_alias_type(tag).or_else(|| {
        if is_player_model_family_tag(tag) {
            Some("PlayerModel")
        } else if is_frame_fallback_tag(tag) {
            Some("Frame")
        } else {
            None
        }
    }) {
        return Some((widget_type, None));
    }

    direct_widget_mapping(tag)
}

#[cfg(test)]
mod tests {
    use super::widget_type_for_tag;

    #[test]
    fn widget_type_for_tag_preserves_alias_widgets() {
        assert_eq!(
            widget_type_for_tag("EventFrame"),
            Some(("EventFrame", None))
        );
        assert_eq!(
            widget_type_for_tag("FogOfWarFrame"),
            Some(("FogOfWarFrame", None))
        );
    }

    #[test]
    fn widget_type_for_tag_maps_intrinsic_widget_aliases() {
        assert_eq!(
            widget_type_for_tag("ItemButton"),
            Some(("Button", Some("ItemButton")))
        );
        assert_eq!(
            widget_type_for_tag("EventScrollFrame"),
            Some(("ScrollFrame", Some("EventScrollFrame")))
        );
        assert_eq!(
            widget_type_for_tag("ContainedAlertFrame"),
            Some(("Button", Some("ContainedAlertFrame")))
        );
        assert_eq!(
            widget_type_for_tag("DropDownToggleButton"),
            Some(("Button", Some("DropDownToggleButton")))
        );
        assert_eq!(
            widget_type_for_tag("EventButton"),
            Some(("Button", Some("EventButton")))
        );
    }

    #[test]
    fn widget_type_for_tag_maps_widget_families_and_fallbacks() {
        assert_eq!(
            widget_type_for_tag("DressUpModel"),
            Some(("PlayerModel", None))
        );
        assert_eq!(widget_type_for_tag("MapScene"), Some(("Frame", None)));
        assert_eq!(widget_type_for_tag("UnknownTag"), None);
    }
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

/// Actors container for ModelScene.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ActorsXml {
    #[serde(rename = "Actor", default)]
    pub actors: Vec<ActorXml>,
}

/// Actor definition for ModelScene.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ActorXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parentKey", alias = "@parentkey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
}
