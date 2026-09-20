//! XML type definitions for WoW UI files.

use serde::Deserialize;

use super::types_animation::{AnimationGroupXml, AnimationXml};
use super::types_elements::{
    ActorXml, ActorsXml, FontStringXml, FramesXml, IncludeXml, LayersXml, ScriptXml, TextureXml,
};
use super::types_fonts::{FontFamilyXml, FontXml};
use super::types_support::{
    AnchorsXml, AnimationsXml, AttributesXml, BackdropXml, BindingXml, ColorXml, FontRefXml,
    InsetsXml, KeyValuesXml, MixinsXml, ModifiedClickXml, ResizeBoundsXml, ScriptsXml,
    ScrollChildXml, SizeXml,
};

/// Root element of a WoW UI XML file.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename = "Ui")]
pub struct UiXml {
    #[serde(rename = "$value", default)]
    pub elements: Vec<XmlElement>,
}

/// ScopedModifier is a transparent container that wraps child elements.
/// When `forbidden="true"` or `fullLockdown="true"`, all contained frames are marked
/// as forbidden (secure-restricted).
#[derive(Debug, Deserialize, Clone)]
pub struct ScopedModifierXml {
    #[serde(rename = "@forbidden", default)]
    pub forbidden: Option<bool>,
    #[serde(rename = "@scriptsUseGivenEnv", default)]
    pub scripts_use_given_env: Option<bool>,
    #[serde(rename = "@addToSecureEnv", default)]
    pub add_to_secure_env: Option<bool>,
    #[serde(rename = "@hideFromGlobalEnv", default)]
    pub hide_from_global_env: Option<bool>,
    #[serde(rename = "@fullLockdown", default)]
    pub full_lockdown: Option<bool>,
    #[serde(rename = "@useForbiddenObjectTable", default)]
    pub use_forbidden_object_table: Option<bool>,
    #[serde(rename = "$value", default)]
    pub elements: Vec<XmlElement>,
}

/// XML elements that can appear in a UI definition.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum XmlElement {
    // Frame-like widgets
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
    ScopedModifier(ScopedModifierXml),
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
    AnimationGroup(AnimationGroupXml),
    // ModelScene elements
    Actor(ActorXml),
    // Font definitions
    Font(FontXml),
    FontFamily(FontFamilyXml),
    // Keybinding/click definitions (no-op, parsed for clean deserialization)
    Binding(BindingXml),
    ModifiedClick(ModifiedClickXml),
    // Text content (from malformed XML or comments)
    #[serde(rename = "$text")]
    Text(String),
    // Unknown elements (intrinsic types, custom elements)
    #[serde(other)]
    Unknown,
}

/// Frame definition in XML.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct FrameXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parent")]
    pub parent: Option<String>,
    #[serde(rename = "@parentKey", alias = "@parentkey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@secureMixin")]
    pub secure_mixin: Option<String>,
    #[serde(rename = "@hidden")]
    pub hidden: Option<bool>,
    #[serde(rename = "@alpha")]
    pub alpha: Option<f32>,
    #[serde(rename = "@scale")]
    pub scale: Option<f32>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@intrinsic")]
    pub intrinsic: Option<bool>,
    #[serde(rename = "@propagateMouseInput")]
    pub propagate_mouse_input: Option<String>,
    #[serde(rename = "@propagateMouseInputMask")]
    pub propagate_mouse_input_mask: Option<String>,
    #[serde(rename = "@setAllPoints")]
    pub set_all_points: Option<bool>,
    #[serde(rename = "@clipChildren")]
    pub clip_children: Option<bool>,
    #[serde(rename = "@enableMouse")]
    pub enable_mouse: Option<bool>,
    #[serde(rename = "@enableKeyboard")]
    pub enable_keyboard: Option<bool>,
    #[serde(rename = "@clampedToScreen")]
    pub clamped_to_screen: Option<bool>,
    /// Button text attribute (localization key or literal text).
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@parentArray")]
    pub parent_array: Option<String>,
    #[serde(rename = "@id")]
    pub xml_id: Option<i32>,
    #[serde(rename = "@frameStrata")]
    pub frame_strata: Option<String>,
    #[serde(rename = "@frameLevel")]
    pub frame_level: Option<i32>,
    #[serde(rename = "@orientation")]
    pub orientation: Option<String>,
    #[serde(rename = "@fixedFrameLevel")]
    pub fixed_frame_level: Option<bool>,
    #[serde(rename = "@useParentLevel")]
    pub use_parent_level: Option<bool>,
    #[serde(rename = "@toplevel")]
    pub toplevel: Option<bool>,
    #[serde(rename = "@protected")]
    pub protected: Option<bool>,
    #[serde(rename = "@onUpdateMode")]
    pub on_update_mode: Option<String>,
    /// EditBox `letters` attribute — caps `SetMaxLetters`. Ignored on
    /// non-EditBox widgets.
    #[serde(rename = "@letters")]
    pub letters: Option<i32>,

    #[serde(skip)]
    pub use_forbidden_object_table: bool,

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

    /// Get the literal `<Mixins>` block if present.
    pub fn mixins(&self) -> Option<&MixinsXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::Mixins(m) => Some(m),
            _ => None,
        })
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

    pub fn forbidden_aspects(&self) -> Option<&ForbiddenAspectsXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::ForbiddenAspects(aspects) => Some(aspects),
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

    /// Collect all child frame elements into a Vec of `(frame, tag)` pairs.
    /// Convenience wrapper over `try_for_each_frame_element` for call sites
    /// that want materialised results (primarily tests).
    pub fn all_frame_elements(&self) -> Vec<(&FrameXml, &'static str)> {
        let mut out = Vec::new();
        for child in &self.children {
            collect_frame_child_elements(child, &mut out);
        }
        out
    }

    /// Visit all child frame elements across all `<Frames>` sections and
    /// standalone frame-type children (WoW XML allows frame elements outside
    /// `<Frames>` wrappers).
    pub fn try_for_each_frame_element<E>(
        &self,
        mut visit: impl FnMut(&FrameXml, &'static str) -> Result<(), E>,
    ) -> Result<(), E> {
        for child in &self.children {
            if let FrameChildElement::Frames(frames) = child {
                visit_frames_section(frames, &mut visit)?;
            } else if let Some((frame, tag)) = standalone_frame_child_tag(child) {
                visit(frame, tag)?;
            }
        }
        Ok(())
    }

    /// Get the Attributes element if present.
    pub fn xml_attributes(&self) -> Option<&AttributesXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::Attributes(a) => Some(a),
            _ => None,
        })
    }

    /// Get the first KeyValues element if present.
    pub fn key_values(&self) -> Option<&KeyValuesXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::KeyValues(k) => Some(k),
            _ => None,
        })
    }

    /// Iterate over all KeyValues elements (frames can have multiple `<KeyValues>` blocks).
    pub fn all_key_values(&self) -> impl Iterator<Item = &KeyValuesXml> {
        self.children.iter().filter_map(|c| match c {
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

    /// Get the HitRectInsets element if present.
    pub fn hit_rect_insets(&self) -> Option<&InsetsXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::HitRectInsets(i) => Some(i),
            _ => None,
        })
    }

    /// Get the TextInsets element if present.
    pub fn text_insets(&self) -> Option<&InsetsXml> {
        self.children.iter().find_map(|c| match c {
            FrameChildElement::TextInsets(i) => Some(i),
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

    /// Get font references for button state fonts (NormalFont, HighlightFont, DisabledFont).
    pub fn button_fonts(&self) -> [(&str, Option<&FontRefXml>); 3] {
        let normal = self.children.iter().find_map(|c| match c {
            FrameChildElement::NormalFont(f) => Some(f),
            _ => None,
        });
        let highlight = self.children.iter().find_map(|c| match c {
            FrameChildElement::HighlightFont(f) => Some(f),
            _ => None,
        });
        let disabled = self.children.iter().find_map(|c| match c {
            FrameChildElement::DisabledFont(f) => Some(f),
            _ => None,
        });
        [
            ("SetNormalFontObject", normal),
            ("SetHighlightFontObject", highlight),
            ("SetDisabledFontObject", disabled),
        ]
    }
}

fn visit_frames_section<E>(
    frames: &FramesXml,
    visit: &mut impl FnMut(&FrameXml, &'static str) -> Result<(), E>,
) -> Result<(), E> {
    for element in &frames.elements {
        if let Some((frame, tag)) = element.as_frame_data() {
            visit(frame, tag)?;
        }
    }
    Ok(())
}

fn collect_frame_child_elements<'a>(
    child: &'a FrameChildElement,
    out: &mut Vec<(&'a FrameXml, &'static str)>,
) {
    if let FrameChildElement::Frames(frames) = child {
        collect_frames_section(frames, out);
        return;
    }

    if let Some((frame, tag)) = standalone_frame_child_tag(child) {
        out.push((frame, tag));
    }
}

fn collect_frames_section<'a>(frames: &'a FramesXml, out: &mut Vec<(&'a FrameXml, &'static str)>) {
    for element in &frames.elements {
        if let Some((frame, tag)) = element.as_frame_data() {
            out.push((frame, tag));
        }
    }
}

fn standalone_frame_child_tag(child: &FrameChildElement) -> Option<(&FrameXml, &'static str)> {
    match child {
        FrameChildElement::Frame(f) => Some((f, "Frame")),
        FrameChildElement::Button(f) => Some((f, "Button")),
        FrameChildElement::StatusBar(f) => Some((f, "StatusBar")),
        FrameChildElement::CheckButton(f) => Some((f, "CheckButton")),
        FrameChildElement::EditBox(f) => Some((f, "EditBox")),
        FrameChildElement::ScrollFrame(f) => Some((f, "ScrollFrame")),
        FrameChildElement::Slider(f) => Some((f, "Slider")),
        FrameChildElement::Cooldown(f) => Some((f, "Cooldown")),
        FrameChildElement::GameTooltip(f) => Some((f, "GameTooltip")),
        FrameChildElement::Model(f) => Some((f, "Model")),
        FrameChildElement::ModelScene(f) => Some((f, "ModelScene")),
        FrameChildElement::PlayerModel(f) => Some((f, "PlayerModel")),
        FrameChildElement::MessageFrame(f) => Some((f, "MessageFrame")),
        FrameChildElement::ScrollingMessageFrame(f) => Some((f, "ScrollingMessageFrame")),
        FrameChildElement::SimpleHTML(f) => Some((f, "SimpleHTML")),
        FrameChildElement::ColorSelect(f) => Some((f, "ColorSelect")),
        FrameChildElement::ItemButton(f) => Some((f, "ItemButton")),
        FrameChildElement::EventFrame(f) => Some((f, "EventFrame")),
        _ => None,
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ForbiddenAspectsXml {
    #[serde(rename = "ForbiddenAspect", default)]
    pub aspects: Vec<ForbiddenAspectXml>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ForbiddenAspectXml {
    #[serde(rename = "@aspect")]
    pub aspect: String,
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
    Mixins(MixinsXml),
    ForbiddenAspects(ForbiddenAspectsXml),
    Attributes(AttributesXml),
    Animations(AnimationsXml),
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
    FontString(FontStringXml),
    ScrollChild(ScrollChildXml),
    ThumbTexture(TextureXml),
    BarTexture(TextureXml),
    BarColor(ColorXml),
    Backdrop(BackdropXml),
    ResizeBounds(ResizeBoundsXml),
    HitRectInsets(InsetsXml),
    TextInsets(InsetsXml),
    PushedTextOffset(SizeXml),
    SwipeTexture(TextureXml),
    EdgeTexture(TextureXml),
    BlingTexture(TextureXml),
    ColorWheelTexture(TextureXml),
    ColorWheelThumbTexture(TextureXml),
    ColorValueTexture(TextureXml),
    ColorValueThumbTexture(TextureXml),
    ColorAlphaTexture(TextureXml),
    ColorAlphaThumbTexture(TextureXml),
    FontStringHeader1(FontStringXml),
    FontStringHeader2(FontStringXml),
    FontStringHeader3(FontStringXml),
    NormalColor(ColorXml),
    HighlightColor(ColorXml),
    DisabledColor(ColorXml),
    Actors(ActorsXml),
    FogColor(ColorXml),
    ViewInsets(InsetsXml),
    Frame(FrameXml),
    Button(FrameXml),
    StatusBar(FrameXml),
    CheckButton(FrameXml),
    EditBox(FrameXml),
    ScrollFrame(FrameXml),
    Slider(FrameXml),
    Cooldown(FrameXml),
    GameTooltip(FrameXml),
    Model(FrameXml),
    ModelScene(FrameXml),
    PlayerModel(FrameXml),
    MessageFrame(FrameXml),
    ScrollingMessageFrame(FrameXml),
    SimpleHTML(FrameXml),
    ColorSelect(FrameXml),
    ItemButton(FrameXml),
    EventFrame(FrameXml),
    #[serde(other)]
    Unknown,
}
