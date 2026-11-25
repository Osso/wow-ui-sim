//! Type definitions used by Frame: colors, gradients, model state, minimap blobs, text, backdrop.

use super::AnchorPoint;

/// Attribute value stored on frames.
///
/// Scalar types (`String`, `Number`, `Boolean`) are stored inline. Lua
/// reference types (tables, functions, userdata) that WoW attributes must
/// also support — Blizzard uses `SetAttribute` + `OnAttributeChanged` as a
/// secure message bus, passing tables and closures across the taint
/// barrier — are stored in a Lua registry table and referenced here by
/// key. See `text_attribute_event::val_to_attribute`.
#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    /// Key into the `__wow_attr_refs__` registry table.
    LuaRef(String),
    Nil,
}

/// RGBA color value.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSegment {
    pub text: String,
    pub color: Color,
}

/// Vertex color gradient (VERTICAL or HORIZONTAL).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    /// true = vertical (min at bottom, max at top), false = horizontal (min at left, max at right).
    pub vertical: bool,
    pub min_color: Color,
    pub max_color: Color,
}

/// Stored alpha-gradient span.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlphaGradient {
    pub start: f32,
    pub length: f32,
}

impl AlphaGradient {
    /// Simulator reveal policy: scalar positions in markup-stripped text.
    pub fn alpha_at(self, position: usize) -> f32 {
        let position = position as f32;
        if self.length <= 0.0 {
            return if position < self.start { 1.0 } else { 0.0 };
        }
        ((self.start + self.length - position) / self.length).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MinimapBlobLayerStyle {
    pub texture: Option<String>,
    pub alpha: f64,
}

#[derive(Debug, Clone, Default)]
pub struct MinimapBlobRingStyle {
    pub texture: Option<String>,
    pub alpha: f64,
    pub scalar: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelCameraState {
    pub distance: f32,
    pub facing: f32,
    pub target: (f32, f32, f32),
    pub roll: f32,
}

impl Default for ModelCameraState {
    fn default() -> Self {
        Self {
            distance: 0.0,
            facing: 0.0,
            target: (0.0, 0.0, 0.0),
            roll: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelTransformState {
    pub scale: f32,
    pub position: (f32, f32, f32),
    pub facing: f32,
    pub camera: ModelCameraState,
}

impl Default for ModelTransformState {
    fn default() -> Self {
        Self {
            scale: 1.0,
            position: (0.0, 0.0, 0.0),
            facing: 0.0,
            camera: ModelCameraState::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelAppearanceState {
    pub display_info: Option<i32>,
    pub creature_id: Option<i32>,
    pub animation_id: Option<i32>,
    pub sequence_id: Option<i32>,
    pub sequence_time_ms: Option<i32>,
    pub refresh_unit_count: u32,
    pub refresh_camera_count: u32,
    /// Set by `Actor:SetModelByCreatureDisplayID(displayID, useCached)`.
    /// Recorded so addons that round-trip the flag (e.g.
    /// `Blizzard_AlliedRacesFrameUI:UpdateModel`) can observe it; the
    /// simulator's 3D path never reads it because rendering is stubbed.
    pub use_cached_model: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelRenderingState {
    pub alpha: f32,
    pub shadow_effect: f32,
    pub particles_enabled: bool,
    pub use_gbuffer: bool,
}

impl Default for ModelRenderingState {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            shadow_effect: 0.0,
            particles_enabled: false,
            use_gbuffer: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelSceneCameraState {
    pub position: (f32, f32, f32),
    pub forward: (f32, f32, f32),
    pub right: (f32, f32, f32),
    pub up: (f32, f32, f32),
    pub field_of_view: f32,
    pub near_clip: f32,
    pub far_clip: f32,
}

impl Default for ModelSceneCameraState {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            forward: (0.0, 0.0, 1.0),
            right: (1.0, 0.0, 0.0),
            up: (0.0, 1.0, 0.0),
            field_of_view: 0.785,
            near_clip: 1.0,
            far_clip: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelSceneLightState {
    pub light_type: i32,
    pub position: (f32, f32, f32),
    pub direction: (f32, f32, f32),
    pub ambient_color: Color,
    pub diffuse_color: Color,
    pub visible: bool,
}

impl Default for ModelSceneLightState {
    fn default() -> Self {
        Self {
            light_type: 0,
            position: (0.0, 0.0, 0.0),
            direction: (0.0, -1.0, 0.0),
            ambient_color: Color::rgb(1.0, 1.0, 1.0),
            diffuse_color: Color::rgb(1.0, 1.0, 1.0),
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelSceneFogState {
    pub near: f32,
    pub far: f32,
    pub color: Color,
}

impl Default for ModelSceneFogState {
    fn default() -> Self {
        Self {
            near: 0.0,
            far: 0.0,
            color: Color::rgb(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ModelSceneState {
    pub paused: bool,
    pub allow_overlapped_models: bool,
    pub view_insets: (f32, f32, f32, f32),
    pub view_translation: (f32, f32),
    pub camera: ModelSceneCameraState,
    pub light: ModelSceneLightState,
    pub fog: ModelSceneFogState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerModelState {
    pub do_blend: bool,
    pub keep_model_on_hide: bool,
    pub last_unit: Option<String>,
    pub last_item: Option<String>,
    pub last_item_appearance: Option<String>,
    pub active_anim_kit: Option<i32>,
}

/// Lazily allocated state used only by model-family widget methods.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelWidgetState {
    pub model_path: Option<String>,
    pub model_file_id: Option<i64>,
    pub model_transform: ModelTransformState,
    pub actor_pitch: f32,
    pub actor_roll: f32,
    pub actor_yaw: f32,
    pub particle_override_scale: Option<f32>,
    pub model_appearance: ModelAppearanceState,
    pub model_rendering: ModelRenderingState,
    pub model_scene_state: ModelSceneState,
    pub model_scene_actor_ids: Vec<u64>,
    pub model_scene_actor_tags: Vec<(String, u64)>,
    pub player_model_state: PlayerModelState,
}

/// Text justification for FontStrings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextJustify {
    /// Left/Top alignment.
    Left,
    /// Center/Middle alignment (default).
    #[default]
    Center,
    /// Right/Bottom alignment.
    Right,
}

impl TextJustify {
    /// Parse from WoW string (case-insensitive).
    pub fn from_wow_str(s: &str) -> Self {
        if s.eq_ignore_ascii_case("LEFT") || s.eq_ignore_ascii_case("TOP") {
            TextJustify::Left
        } else if s.eq_ignore_ascii_case("CENTER") || s.eq_ignore_ascii_case("MIDDLE") {
            TextJustify::Center
        } else if s.eq_ignore_ascii_case("RIGHT") || s.eq_ignore_ascii_case("BOTTOM") {
            TextJustify::Right
        } else {
            TextJustify::Left
        }
    }

    /// Convert to the canonical horizontal WoW string ("LEFT", "CENTER", "RIGHT").
    pub fn as_h_str(self) -> &'static str {
        match self {
            TextJustify::Left => "LEFT",
            TextJustify::Center => "CENTER",
            TextJustify::Right => "RIGHT",
        }
    }

    /// Convert to the canonical vertical WoW string ("TOP", "MIDDLE", "BOTTOM").
    pub fn as_v_str(self) -> &'static str {
        match self {
            TextJustify::Left => "TOP",
            TextJustify::Center => "MIDDLE",
            TextJustify::Right => "BOTTOM",
        }
    }
}

/// Text outline style for FontStrings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextOutline {
    #[default]
    None,
    /// Normal outline (1px).
    Outline,
    /// Thick outline (2px).
    ThickOutline,
}

impl TextOutline {
    /// Parse from WoW flag string (e.g., "OUTLINE", "THICKOUTLINE", "OUTLINE, MONOCHROME").
    pub fn from_wow_str(s: &str) -> Self {
        if contains_ascii_case_insensitive(s, "THICKOUTLINE") {
            TextOutline::ThickOutline
        } else if contains_ascii_case_insensitive(s, "OUTLINE")
            || contains_ascii_case_insensitive(s, "NORMAL")
        {
            TextOutline::Outline
        } else {
            TextOutline::None
        }
    }
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack.len() >= needle.len()
        && haystack
            .as_bytes()
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

/// Backdrop configuration for frames.
#[derive(Debug, Clone, Default)]
pub struct Backdrop {
    /// Whether backdrop is enabled.
    pub enabled: bool,
    /// Background texture file path (WoW path format).
    pub bg_file: Option<String>,
    /// Edge/border texture file path (WoW path format).
    pub edge_file: Option<String>,
    /// Background color.
    pub bg_color: Color,
    /// Border color.
    pub border_color: Color,
    /// Edge size (border thickness).
    pub edge_size: f32,
    /// Insets from frame edges.
    pub insets: f32,
}

/// Anchor for Line widget start/end points.
#[derive(Debug, Clone, Default)]
pub struct LineAnchor {
    pub point: AnchorPoint,
    pub target_id: Option<u64>,
    pub x_offset: f32,
    pub y_offset: f32,
}
