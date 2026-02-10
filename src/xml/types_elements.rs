//! XML element type definitions: layers, textures, fontstrings, animations, etc.

use serde::Deserialize;

use super::types::{
    AbsDimensionXml, AnchorsXml, AnimationsXml, ColorXml, FrameXml, KeyValuesXml, ScriptsXml,
    SizeXml,
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
    #[serde(rename = "$value", default)]
    pub elements: Vec<LayerElement>,
}

impl LayerXml {
    /// Get all Texture elements in this layer (includes MaskTextures — they
    /// need to exist as child widgets so Lua code can reference them via parentKey).
    /// Returns (texture, is_mask) pairs.
    pub fn textures(&self) -> impl Iterator<Item = (&TextureXml, bool)> {
        self.elements.iter().filter_map(|e| match e {
            LayerElement::Texture(t) | LayerElement::Line(t) => Some((t, false)),
            LayerElement::MaskTexture(t) => Some((t, true)),
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
    #[serde(rename = "@parentKey")]
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
    #[serde(rename = "@hidden")]
    pub hidden: Option<bool>,
    #[serde(rename = "@alpha")]
    pub alpha: Option<f32>,
    #[serde(rename = "@alphaMode")]
    pub alpha_mode: Option<String>,
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
    #[serde(rename = "Animations")]
    pub animations: Option<AnimationsXml>,
    #[serde(rename = "Scripts")]
    pub scripts: Option<ScriptsXml>,
    /// MaskedTextures — declares which sibling textures this mask applies to.
    #[serde(rename = "MaskedTextures")]
    pub masked_textures: Option<MaskedTexturesXml>,
}

/// Container for MaskedTexture entries inside a MaskTexture element.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct MaskedTexturesXml {
    #[serde(rename = "MaskedTexture", default)]
    pub entries: Vec<MaskedTextureEntryXml>,
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
    #[serde(rename = "@parentKey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
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
    #[serde(rename = "@setAllPoints")]
    pub set_all_points: Option<bool>,
    #[serde(rename = "@parentArray")]
    pub parent_array: Option<String>,
    #[serde(rename = "@wordwrap")]
    pub word_wrap: Option<bool>,
    #[serde(rename = "@maxLines")]
    pub max_lines: Option<u32>,
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
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@setToFinalAlpha")]
    pub set_to_final_alpha: Option<bool>,
    #[serde(rename = "@looping")]
    pub looping: Option<String>,
    #[serde(rename = "$value", default)]
    pub elements: Vec<AnimationElement>,
}

/// Elements that can appear inside an AnimationGroup.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum AnimationElement {
    Animation(AnimationXml),
    Alpha(AnimationXml),
    Translation(AnimationXml),
    LineTranslation(AnimationXml),
    Rotation(AnimationXml),
    Scale(AnimationXml),
    LineScale(AnimationXml),
    Path(AnimationXml),
    FlipBook(AnimationXml),
    VertexColor(AnimationXml),
    TextureCoordTranslation(AnimationXml),
    Scripts(Box<ScriptsXml>),
    KeyValues(KeyValuesXml),
    #[serde(other)]
    Unknown,
}

/// Common animation attributes. Since the simulator doesn't execute animations,
/// all type-specific attributes are optional on a single struct.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct AnimationXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parentKey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@childKey")]
    pub child_key: Option<String>,
    #[serde(rename = "@target")]
    pub target: Option<String>,
    #[serde(rename = "@targetKey")]
    pub target_key: Option<String>,
    #[serde(rename = "@duration")]
    pub duration: Option<f32>,
    #[serde(rename = "@order")]
    pub order: Option<u32>,
    #[serde(rename = "@startDelay")]
    pub start_delay: Option<f32>,
    #[serde(rename = "@endDelay")]
    pub end_delay: Option<f32>,
    #[serde(rename = "@smoothing")]
    pub smoothing: Option<String>,
    // Alpha
    #[serde(rename = "@fromAlpha")]
    pub from_alpha: Option<f32>,
    #[serde(rename = "@toAlpha")]
    pub to_alpha: Option<f32>,
    // Translation
    #[serde(rename = "@offsetX")]
    pub offset_x: Option<f32>,
    #[serde(rename = "@offsetY")]
    pub offset_y: Option<f32>,
    // Scale
    #[serde(rename = "@scaleX")]
    pub scale_x: Option<f32>,
    #[serde(rename = "@scaleY")]
    pub scale_y: Option<f32>,
    #[serde(rename = "@fromScaleX")]
    pub from_scale_x: Option<f32>,
    #[serde(rename = "@fromScaleY")]
    pub from_scale_y: Option<f32>,
    #[serde(rename = "@toScaleX")]
    pub to_scale_x: Option<f32>,
    #[serde(rename = "@toScaleY")]
    pub to_scale_y: Option<f32>,
    // Rotation
    #[serde(rename = "@degrees")]
    pub degrees: Option<f32>,
    #[serde(rename = "@radians")]
    pub radians: Option<f32>,
    // FlipBook
    #[serde(rename = "@flipBookRows")]
    pub flip_book_rows: Option<u32>,
    #[serde(rename = "@flipBookColumns")]
    pub flip_book_columns: Option<u32>,
    #[serde(rename = "@flipBookFrames")]
    pub flip_book_frames: Option<u32>,
    // TextureCoordTranslation
    #[serde(rename = "@offsetU")]
    pub offset_u: Option<f32>,
    #[serde(rename = "@offsetV")]
    pub offset_v: Option<f32>,
    // Path
    #[serde(rename = "@curve")]
    pub curve: Option<String>,
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
    #[serde(rename = "@justifyH")]
    pub justify_h: Option<String>,
    #[serde(rename = "@justifyV")]
    pub justify_v: Option<String>,
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
