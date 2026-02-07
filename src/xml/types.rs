//! XML type definitions for WoW UI files.

use serde::Deserialize;

use super::types_elements::{
    ActorXml, ActorsXml, AnimationGroupXml, FontFamilyXml, FontStringXml, FontXml, FrameElement,
    FramesXml, IncludeXml, LayersXml, ScriptXml, TextureXml,
};

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
    ScopedModifier(FrameXml),
    Line(FrameXml),
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
    #[serde(rename = "@secureMixin")]
    pub secure_mixin: Option<String>,
    #[serde(rename = "@hidden")]
    pub hidden: Option<bool>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@intrinsic")]
    pub intrinsic: Option<bool>,
    #[serde(rename = "@propagateMouseInput")]
    pub propagate_mouse_input: Option<String>,
    #[serde(rename = "@setAllPoints")]
    pub set_all_points: Option<bool>,
    #[serde(rename = "@enableMouse")]
    pub enable_mouse: Option<bool>,
    /// Button text attribute (localization key or literal text).
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@parentArray")]
    pub parent_array: Option<String>,

    // Child elements collected via $value to allow multiples
    #[serde(rename = "$value", default)]
    pub children: Vec<FrameChildElement>,
}

impl FrameXml {
    /// Get combined mixin string (regular mixin + secureMixin).
    pub fn combined_mixin(&self) -> Option<String> {
        match (&self.mixin, &self.secure_mixin) {
            (Some(m), Some(sm)) => Some(format!("{}, {}", m, sm)),
            (Some(m), None) => Some(m.clone()),
            (None, Some(sm)) => Some(sm.clone()),
            (None, None) => None,
        }
    }

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

    /// Get all child frame elements across all `<Frames>` sections.
    pub fn all_frame_elements(&self) -> Vec<&FrameElement> {
        self.children
            .iter()
            .filter_map(|c| match c {
                FrameChildElement::Frames(f) => Some(f.elements.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Get the KeyValues element if present.
    pub fn key_values(&self) -> Option<&KeyValuesXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::KeyValues(k) => Some(k),
            _ => None,
        })
    }

    /// Get the Animations element if present.
    pub fn animations(&self) -> Option<&AnimationsXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::Animations(a) => Some(a),
            _ => None,
        })
    }

    pub fn scroll_child(&self) -> Option<&ScrollChildXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::ScrollChild(sc) => Some(sc),
            _ => None,
        })
    }

    /// Get the BarTexture element if present (StatusBar-specific).
    pub fn bar_texture(&self) -> Option<&TextureXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::BarTexture(t) => Some(t),
            _ => None,
        })
    }

    /// Get the NormalTexture element if present (Button-specific).
    pub fn normal_texture(&self) -> Option<&TextureXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::NormalTexture(t) => Some(t),
            _ => None,
        })
    }

    /// Get the PushedTexture element if present (Button-specific).
    pub fn pushed_texture(&self) -> Option<&TextureXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::PushedTexture(t) => Some(t),
            _ => None,
        })
    }

    /// Get the HighlightTexture element if present (Button-specific).
    pub fn highlight_texture(&self) -> Option<&TextureXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::HighlightTexture(t) => Some(t),
            _ => None,
        })
    }

    /// Get the DisabledTexture element if present (Button-specific).
    pub fn disabled_texture(&self) -> Option<&TextureXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::DisabledTexture(t) => Some(t),
            _ => None,
        })
    }

    /// Get the CheckedTexture element if present (CheckButton-specific).
    pub fn checked_texture(&self) -> Option<&TextureXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::CheckedTexture(t) => Some(t),
            _ => None,
        })
    }

    /// Get the DisabledCheckedTexture element if present (CheckButton-specific).
    pub fn disabled_checked_texture(&self) -> Option<&TextureXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::DisabledCheckedTexture(t) => Some(t),
            _ => None,
        })
    }

    /// Get the ThumbTexture element if present (Slider-specific).
    pub fn thumb_texture(&self) -> Option<&TextureXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::ThumbTexture(t) => Some(t),
            _ => None,
        })
    }

    /// Get the ButtonText fontstring if present (Button-specific).
    pub fn button_text(&self) -> Option<&FontStringXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::ButtonText(f) => Some(f),
            _ => None,
        })
    }

    /// Get the FontString child element if present (EditBox-specific).
    pub fn font_string_child(&self) -> Option<&FontStringXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::FontString(f) => Some(f),
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
    DisabledCheckedTexture(TextureXml),
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
    // EditBox/MessageFrame text padding
    TextInsets(InsetsXml),
    // Button pressed text offset
    PushedTextOffset(SizeXml),
    // Cooldown-specific textures
    SwipeTexture(TextureXml),
    EdgeTexture(TextureXml),
    BlingTexture(TextureXml),
    // ColorSelect-specific textures
    ColorWheelTexture(TextureXml),
    ColorWheelThumbTexture(TextureXml),
    ColorValueTexture(TextureXml),
    ColorValueThumbTexture(TextureXml),
    ColorAlphaTexture(TextureXml),
    ColorAlphaThumbTexture(TextureXml),
    // SimpleHTML header font strings
    FontStringHeader1(FontStringXml),
    FontStringHeader2(FontStringXml),
    FontStringHeader3(FontStringXml),
    // Button state colors
    NormalColor(ColorXml),
    HighlightColor(ColorXml),
    DisabledColor(ColorXml),
    // ModelScene actors container
    Actors(ActorsXml),
    // Model/MapScene elements
    FogColor(ColorXml),
    ViewInsets(InsetsXml),
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
    // Animation group scripts
    #[serde(rename = "OnPlay", default)]
    pub on_play: Vec<ScriptBodyXml>,
    #[serde(rename = "OnFinished", default)]
    pub on_finished: Vec<ScriptBodyXml>,
    #[serde(rename = "OnStop", default)]
    pub on_stop: Vec<ScriptBodyXml>,
    #[serde(rename = "OnLoop", default)]
    pub on_loop: Vec<ScriptBodyXml>,
    #[serde(rename = "OnPause", default)]
    pub on_pause: Vec<ScriptBodyXml>,
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
