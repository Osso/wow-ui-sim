use serde::Deserialize;

use super::types_animation::AnimationGroupXml;
use super::types_elements::FrameElement;

/// Animations container.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct AnimationsXml {
    #[serde(rename = "$value", default)]
    pub animations: Vec<AnimationGroupXml>,
}

/// Literal frame mixin list (`<Mixins><Mixin key="..."/></Mixins>`).
#[derive(Debug, Deserialize, Default, Clone)]
pub struct MixinsXml {
    #[serde(rename = "Mixin", default)]
    pub entries: Vec<MixinXml>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MixinXml {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@source")]
    pub source: Option<String>,
    #[serde(rename = "@targetPartition")]
    pub target_partition: Option<String>,
    #[serde(rename = "@inboundPartition")]
    pub inbound_partition: Option<String>,
    #[serde(rename = "@secureDelegates")]
    pub secure_delegates: Option<bool>,
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
    #[serde(rename = "AbsInset")]
    pub abs_inset: Option<AbsInsetXml>,
}

impl InsetsXml {
    pub fn left(&self) -> f32 {
        self.left
            .or_else(|| self.abs_inset.as_ref()?.left)
            .unwrap_or(0.0)
    }

    pub fn right(&self) -> f32 {
        self.right
            .or_else(|| self.abs_inset.as_ref()?.right)
            .unwrap_or(0.0)
    }

    pub fn top(&self) -> f32 {
        self.top
            .or_else(|| self.abs_inset.as_ref()?.top)
            .unwrap_or(0.0)
    }

    pub fn bottom(&self) -> f32 {
        self.bottom
            .or_else(|| self.abs_inset.as_ref()?.bottom)
            .unwrap_or(0.0)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AbsInsetXml {
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
    pub point: Option<String>,
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
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@y")]
    pub y: Option<f32>,
    #[serde(rename = "AbsDimension")]
    pub abs_dimension: Option<AbsDimensionXml>,
}

/// Scripts container.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ScriptsXml {
    #[serde(rename = "OnLoad", default)]
    pub on_load: Vec<ScriptBodyXml>,
    #[serde(rename = "OnEvent", default)]
    pub on_event: Vec<ScriptBodyXml>,
    #[serde(rename = "OnUpdate", default)]
    pub on_update: Vec<ScriptBodyXml>,
    #[serde(rename = "OnClick", default)]
    pub on_click: Vec<ScriptBodyXml>,
    #[serde(rename = "OnDoubleClick", default)]
    pub on_double_click: Vec<ScriptBodyXml>,
    #[serde(rename = "OnShow", default)]
    pub on_show: Vec<ScriptBodyXml>,
    #[serde(rename = "OnHide", default)]
    pub on_hide: Vec<ScriptBodyXml>,
    #[serde(rename = "OnEnter", default)]
    pub on_enter: Vec<ScriptBodyXml>,
    #[serde(rename = "OnLeave", default)]
    pub on_leave: Vec<ScriptBodyXml>,
    #[serde(rename = "OnMouseDown", default)]
    pub on_mouse_down: Vec<ScriptBodyXml>,
    #[serde(rename = "OnMouseUp", default)]
    pub on_mouse_up: Vec<ScriptBodyXml>,
    #[serde(rename = "OnMouseWheel", default)]
    pub on_mouse_wheel: Vec<ScriptBodyXml>,
    #[serde(rename = "OnDragStart", default)]
    pub on_drag_start: Vec<ScriptBodyXml>,
    #[serde(rename = "OnDragStop", default)]
    pub on_drag_stop: Vec<ScriptBodyXml>,
    #[serde(rename = "OnReceiveDrag", default)]
    pub on_receive_drag: Vec<ScriptBodyXml>,
    #[serde(rename = "OnEnterPressed", default)]
    pub on_enter_pressed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnEscapePressed", default)]
    pub on_escape_pressed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnTabPressed", default)]
    pub on_tab_pressed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnSpacePressed", default)]
    pub on_space_pressed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnArrowPressed", default)]
    pub on_arrow_pressed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnTextChanged", default)]
    pub on_text_changed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnTextSet", default)]
    pub on_text_set: Vec<ScriptBodyXml>,
    #[serde(rename = "OnChar", default)]
    pub on_char: Vec<ScriptBodyXml>,
    #[serde(rename = "OnEditFocusGained", default)]
    pub on_edit_focus_gained: Vec<ScriptBodyXml>,
    #[serde(rename = "OnEditFocusLost", default)]
    pub on_edit_focus_lost: Vec<ScriptBodyXml>,
    #[serde(rename = "OnInputLanguageChanged", default)]
    pub on_input_language_changed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnKeyDown", default)]
    pub on_key_down: Vec<ScriptBodyXml>,
    #[serde(rename = "OnKeyUp", default)]
    pub on_key_up: Vec<ScriptBodyXml>,
    #[serde(rename = "OnGamePadStick", default)]
    pub on_game_pad_stick: Vec<ScriptBodyXml>,
    #[serde(rename = "OnValueChanged", default)]
    pub on_value_changed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnEnable", default)]
    pub on_enable: Vec<ScriptBodyXml>,
    #[serde(rename = "OnDisable", default)]
    pub on_disable: Vec<ScriptBodyXml>,
    #[serde(rename = "OnSizeChanged", default)]
    pub on_size_changed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnAttributeChanged", default)]
    pub on_attribute_changed: Vec<ScriptBodyXml>,
    #[serde(rename = "OnHyperlinkClick", default)]
    pub on_hyperlink_click: Vec<ScriptBodyXml>,
    #[serde(rename = "OnHyperlinkEnter", default)]
    pub on_hyperlink_enter: Vec<ScriptBodyXml>,
    #[serde(rename = "OnHyperlinkLeave", default)]
    pub on_hyperlink_leave: Vec<ScriptBodyXml>,
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
    #[serde(rename = "PreClick", default)]
    pub pre_click: Vec<ScriptBodyXml>,
    #[serde(rename = "PostClick", default)]
    pub post_click: Vec<ScriptBodyXml>,
}

#[derive(Debug, Deserialize, Clone, Default)]
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

/// Attributes container for frame attributes set via XML.
#[derive(Debug, Deserialize, Clone)]
pub struct AttributesXml {
    #[serde(rename = "Attribute", default)]
    pub entries: Vec<AttributeXml>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AttributeXml {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@type")]
    pub attr_type: Option<String>,
    #[serde(rename = "@value")]
    pub value: Option<String>,
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
    #[serde(rename = "@value", default)]
    pub value: String,
    #[serde(rename = "@type")]
    pub value_type: Option<String>,
    #[serde(rename = "@source")]
    pub source: Option<String>,
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

#[derive(Debug, Deserialize, Default, Clone)]
pub struct BindingXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@category")]
    pub category: Option<String>,
    #[serde(rename = "@header")]
    pub header: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ModifiedClickXml {
    #[serde(rename = "@action")]
    pub action: Option<String>,
    #[serde(rename = "@default")]
    pub default: Option<String>,
}

/// `<Gradient orientation="VERTICAL"><MinColor .../><MaxColor .../></Gradient>`
#[derive(Debug, Deserialize, Clone)]
pub struct GradientXml {
    #[serde(rename = "@orientation")]
    pub orientation: Option<String>,
    #[serde(rename = "MinColor")]
    pub min_color: Option<ColorXml>,
    #[serde(rename = "MaxColor")]
    pub max_color: Option<ColorXml>,
}
