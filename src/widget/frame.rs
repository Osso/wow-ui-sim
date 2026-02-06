//! Frame widget - the base container for UI elements.

use super::{next_widget_id, Anchor, AnchorPoint, WidgetType};
use std::collections::{HashMap, HashSet};

/// Attribute value stored on frames.
#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

/// RGBA color value.
#[derive(Debug, Clone, Copy, Default)]
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
        match s.to_uppercase().as_str() {
            "LEFT" | "TOP" => TextJustify::Left,
            "CENTER" | "MIDDLE" => TextJustify::Center,
            "RIGHT" | "BOTTOM" => TextJustify::Right,
            _ => TextJustify::Left,  // WoW defaults to LEFT
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
        let upper = s.to_uppercase();
        if upper.contains("THICKOUTLINE") {
            TextOutline::ThickOutline
        } else if upper.contains("OUTLINE") || upper.contains("NORMAL") {
            TextOutline::Outline
        } else {
            TextOutline::None
        }
    }
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

/// A Frame is the base widget type in WoW's UI system.
#[derive(Debug)]
pub struct Frame {
    /// Unique widget ID.
    pub id: u64,
    /// Widget type.
    pub widget_type: WidgetType,
    /// Global name (optional).
    pub name: Option<String>,
    /// Parent widget ID.
    pub parent_id: Option<u64>,
    /// Child widget IDs.
    pub children: Vec<u64>,
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
    /// Anchors defining position.
    pub anchors: Vec<Anchor>,
    /// Whether the frame is visible.
    pub visible: bool,
    /// Events this frame is registered to receive.
    pub registered_events: HashSet<String>,
    /// Frame level (draw order within strata).
    pub frame_level: i32,
    /// Whether frame level was explicitly set (not inherited from parent).
    pub has_fixed_frame_level: bool,
    /// Frame strata (major draw order).
    pub frame_strata: FrameStrata,
    /// Whether frame strata was explicitly set (not inherited from parent).
    pub has_fixed_frame_strata: bool,
    /// Alpha transparency (0.0 - 1.0).
    pub alpha: f32,
    /// Scale factor (affects visible size; default 1.0).
    pub scale: f32,
    /// Whether mouse is enabled.
    pub mouse_enabled: bool,
    /// Whether keyboard input is enabled for this frame.
    pub keyboard_enabled: bool,
    /// Whether keyboard input propagates to parent frames.
    pub propagate_keyboard_input: bool,
    /// Texture path (for Texture widgets).
    pub texture: Option<String>,
    /// Solid color texture (from SetColorTexture).
    pub color_texture: Option<Color>,
    /// Vertex color for textures (tinting).
    pub vertex_color: Option<Color>,
    /// Text content (for FontString widgets).
    pub text: Option<String>,
    /// Title text (for DefaultPanelTemplate frames).
    pub title: Option<String>,
    /// Text color for FontStrings.
    pub text_color: Color,
    /// Shadow color for FontStrings (defaults to transparent = no shadow).
    pub shadow_color: Color,
    /// Shadow offset (x, y) in pixels for FontStrings.
    pub shadow_offset: (f32, f32),
    /// Font name (for FontString widgets).
    pub font: Option<String>,
    /// Font size (for FontString widgets).
    pub font_size: f32,
    /// Font outline style (OUTLINE, THICKOUTLINE).
    pub font_outline: TextOutline,
    /// Horizontal text justification (LEFT, CENTER, RIGHT).
    pub justify_h: TextJustify,
    /// Vertical text justification (TOP, MIDDLE, BOTTOM).
    pub justify_v: TextJustify,
    /// Named attributes (for secure frames, unit frames, etc.).
    pub attributes: HashMap<String, AttributeValue>,
    /// Backdrop configuration.
    pub backdrop: Backdrop,
    /// Named child references (e.g., "Text" -> FontString child ID for CheckButtons).
    pub children_keys: HashMap<String, u64>,
    /// Whether the frame can be moved by the user.
    pub movable: bool,
    /// Whether the frame can be resized by the user.
    pub resizable: bool,
    /// Whether the frame is clamped to screen bounds.
    pub clamped_to_screen: bool,
    /// Whether the frame is currently being moved/dragged.
    pub is_moving: bool,
    /// Whether text should word-wrap (for FontString widgets).
    pub word_wrap: bool,
    /// Maximum number of lines to display (0 = unlimited, for FontString widgets).
    pub max_lines: u32,
    /// Text scale factor (for FontString widgets).
    pub text_scale: f64,
    /// Normal texture path (for Button widgets).
    pub normal_texture: Option<String>,
    /// Normal texture UV coords (left, right, top, bottom) for atlas-based buttons.
    pub normal_tex_coords: Option<(f32, f32, f32, f32)>,
    /// Pushed texture path (for Button widgets).
    pub pushed_texture: Option<String>,
    /// Pushed texture UV coords for atlas-based buttons.
    pub pushed_tex_coords: Option<(f32, f32, f32, f32)>,
    /// Highlight texture path (for Button widgets).
    pub highlight_texture: Option<String>,
    /// Highlight texture UV coords for atlas-based buttons.
    pub highlight_tex_coords: Option<(f32, f32, f32, f32)>,
    /// Disabled texture path (for Button widgets).
    pub disabled_texture: Option<String>,
    /// Disabled texture UV coords for atlas-based buttons.
    pub disabled_tex_coords: Option<(f32, f32, f32, f32)>,
    /// Checked texture path (for CheckButton widgets).
    pub checked_texture: Option<String>,
    /// Checked texture UV coords for atlas-based check buttons.
    pub checked_tex_coords: Option<(f32, f32, f32, f32)>,
    /// Disabled-checked texture path (for CheckButton widgets).
    pub disabled_checked_texture: Option<String>,
    /// Disabled-checked texture UV coords for atlas-based check buttons.
    pub disabled_checked_tex_coords: Option<(f32, f32, f32, f32)>,
    /// Left cap texture for 3-slice buttons.
    pub left_texture: Option<String>,
    /// Middle (stretchable) texture for 3-slice buttons.
    pub middle_texture: Option<String>,
    /// Right cap texture for 3-slice buttons.
    pub right_texture: Option<String>,
    /// Draw layer for regions (textures/fontstrings).
    pub draw_layer: DrawLayer,
    /// Sub-layer within draw layer (for fine-grained ordering).
    pub draw_sub_layer: i32,
    /// Tile texture horizontally.
    pub horiz_tile: bool,
    /// Tile texture vertically.
    pub vert_tile: bool,
    /// Texture coordinates (left, right, top, bottom) — final UV coords used for rendering.
    pub tex_coords: Option<(f32, f32, f32, f32)>,
    /// Atlas base texture coordinates — the sub-region on the texture file.
    /// SetTexCoord remaps relative to these when an atlas is active.
    pub atlas_tex_coords: Option<(f32, f32, f32, f32)>,
    /// Atlas name (if set via SetAtlas).
    pub atlas: Option<String>,
    /// NineSlice layout type (e.g., "PortraitFrameTemplate", "ButtonFrameTemplateNoPortrait").
    pub nine_slice_layout: Option<String>,
    /// Whether this frame receives ALL events (set by RegisterAllEvents).
    pub register_all_events: bool,
    /// Whether this frame clips its children to its bounds.
    pub clips_children: bool,
    /// Whether mouse motion events are enabled.
    pub mouse_motion_enabled: bool,

    // --- Slider fields ---
    /// Current slider value.
    pub slider_value: f64,
    /// Slider minimum value.
    pub slider_min: f64,
    /// Slider maximum value.
    pub slider_max: f64,
    /// Slider step size.
    pub slider_step: f64,
    /// Slider orientation ("HORIZONTAL" or "VERTICAL").
    pub slider_orientation: String,
    /// Whether slider obeys step on drag.
    pub slider_obey_step_on_drag: bool,
    /// Number of steps per page for slider.
    pub slider_steps_per_page: i32,

    // --- StatusBar fields ---
    /// Current statusbar value.
    pub statusbar_value: f64,
    /// StatusBar minimum value.
    pub statusbar_min: f64,
    /// StatusBar maximum value.
    pub statusbar_max: f64,
    /// StatusBar color.
    pub statusbar_color: Option<Color>,
    /// StatusBar texture path.
    pub statusbar_texture_path: Option<String>,
    /// StatusBar fill style ("STANDARD", "CENTER", etc.).
    pub statusbar_fill_style: String,
    /// Whether statusbar fills in reverse.
    pub statusbar_reverse_fill: bool,
    /// StatusBar orientation ("HORIZONTAL" or "VERTICAL").
    pub statusbar_orientation: String,

    // --- EditBox fields ---
    /// Cursor position in editbox.
    pub editbox_cursor_pos: i32,
    /// Maximum letters allowed (0 = unlimited).
    pub editbox_max_letters: i32,
    /// Maximum bytes allowed (0 = unlimited).
    pub editbox_max_bytes: i32,
    /// Whether editbox is multi-line.
    pub editbox_multi_line: bool,
    /// Whether editbox auto-focuses.
    pub editbox_auto_focus: bool,
    /// Whether editbox is numeric-only.
    pub editbox_numeric: bool,
    /// Whether editbox masks input as password.
    pub editbox_password: bool,
    /// Cursor blink speed in seconds.
    pub editbox_blink_speed: f64,
    /// History lines.
    pub editbox_history: Vec<String>,
    /// Maximum history lines (0 = unlimited).
    pub editbox_history_max: i32,
    /// Text insets (left, right, top, bottom).
    pub editbox_text_insets: (f32, f32, f32, f32),
    /// Whether to count invisible letters.
    pub editbox_count_invisible_letters: bool,

    // --- ScrollFrame fields ---
    /// Scroll child frame ID.
    pub scroll_child_id: Option<u64>,
    /// Horizontal scroll offset.
    pub scroll_horizontal: f64,
    /// Vertical scroll offset.
    pub scroll_vertical: f64,

    // --- Cooldown fields ---
    /// Cooldown start time.
    pub cooldown_start: f64,
    /// Cooldown duration in seconds.
    pub cooldown_duration: f64,
    /// Whether cooldown is reversed.
    pub cooldown_reverse: bool,
    /// Whether to draw the swipe animation.
    pub cooldown_draw_swipe: bool,
    /// Whether to draw the edge highlight.
    pub cooldown_draw_edge: bool,
    /// Whether to draw the bling animation at end.
    pub cooldown_draw_bling: bool,
    /// Whether to hide countdown numbers.
    pub cooldown_hide_countdown: bool,
    /// Whether cooldown is paused.
    pub cooldown_paused: bool,
}

/// Build a `Frame` with all defaults. `$id` is the expression for the `id` field.
macro_rules! frame_defaults {
    ($id:expr) => {
        Frame {
            id: $id,
            widget_type: WidgetType::Frame,
            name: None,
            parent_id: None,
            children: Vec::new(),
            width: 0.0,
            height: 0.0,
            anchors: Vec::new(),
            visible: true,
            registered_events: HashSet::new(),
            frame_level: 0,
            has_fixed_frame_level: false,
            frame_strata: FrameStrata::Medium,
            has_fixed_frame_strata: false,
            alpha: 1.0,
            scale: 1.0,
            mouse_enabled: false,
            keyboard_enabled: false,
            propagate_keyboard_input: false,
            texture: None,
            color_texture: None,
            vertex_color: None,
            text: None,
            title: None,
            text_color: Color::new(1.0, 0.8, 0.2, 1.0),
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.0),
            shadow_offset: (0.0, 0.0),
            font: None,
            font_size: 14.0,
            font_outline: TextOutline::None,
            justify_h: TextJustify::Center,
            justify_v: TextJustify::Center,
            attributes: HashMap::new(),
            backdrop: Backdrop::default(),
            children_keys: HashMap::new(),
            movable: false,
            resizable: false,
            clamped_to_screen: false,
            is_moving: false,
            word_wrap: true,
            max_lines: 0,
            text_scale: 1.0,
            normal_texture: None,
            normal_tex_coords: None,
            pushed_texture: None,
            pushed_tex_coords: None,
            highlight_texture: None,
            highlight_tex_coords: None,
            disabled_texture: None,
            disabled_tex_coords: None,
            checked_texture: None,
            checked_tex_coords: None,
            disabled_checked_texture: None,
            disabled_checked_tex_coords: None,
            left_texture: None,
            middle_texture: None,
            right_texture: None,
            draw_layer: DrawLayer::Artwork,
            draw_sub_layer: 0,
            horiz_tile: false,
            vert_tile: false,
            tex_coords: None,
            atlas_tex_coords: None,
            atlas: None,
            nine_slice_layout: None,
            register_all_events: false,
            clips_children: false,
            mouse_motion_enabled: false,

            // Slider
            slider_value: 0.0,
            slider_min: 0.0,
            slider_max: 100.0,
            slider_step: 1.0,
            slider_orientation: "HORIZONTAL".to_string(),
            slider_obey_step_on_drag: false,
            slider_steps_per_page: 1,

            // StatusBar
            statusbar_value: 0.0,
            statusbar_min: 0.0,
            statusbar_max: 1.0,
            statusbar_color: None,
            statusbar_texture_path: None,
            statusbar_fill_style: "STANDARD".to_string(),
            statusbar_reverse_fill: false,
            statusbar_orientation: "HORIZONTAL".to_string(),

            // EditBox
            editbox_cursor_pos: 0,
            editbox_max_letters: 0,
            editbox_max_bytes: 0,
            editbox_multi_line: false,
            editbox_auto_focus: false,
            editbox_numeric: false,
            editbox_password: false,
            editbox_blink_speed: 0.5,
            editbox_history: Vec::new(),
            editbox_history_max: 0,
            editbox_text_insets: (0.0, 0.0, 0.0, 0.0),
            editbox_count_invisible_letters: false,

            // ScrollFrame
            scroll_child_id: None,
            scroll_horizontal: 0.0,
            scroll_vertical: 0.0,

            // Cooldown
            cooldown_start: 0.0,
            cooldown_duration: 0.0,
            cooldown_reverse: false,
            cooldown_draw_swipe: true,
            cooldown_draw_edge: false,
            cooldown_draw_bling: true,
            cooldown_hide_countdown: false,
            cooldown_paused: false,
        }
    };
}

impl Default for Frame {
    fn default() -> Self {
        frame_defaults!(next_widget_id())
    }
}

impl Frame {
    pub fn new(widget_type: WidgetType, name: Option<String>, parent_id: Option<u64>) -> Self {
        Self {
            widget_type,
            name,
            parent_id,
            ..Default::default()
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_point(
        &mut self,
        point: AnchorPoint,
        relative_to_id: Option<usize>,
        relative_point: AnchorPoint,
        x_offset: f32,
        y_offset: f32,
    ) {
        // Replace existing anchor with same point, or add new one
        let new_anchor = Anchor {
            point,
            relative_to: None,
            relative_to_id,
            relative_point,
            x_offset,
            y_offset,
        };
        if let Some(existing) = self.anchors.iter_mut().find(|a| a.point == point) {
            *existing = new_anchor;
        } else {
            self.anchors.push(new_anchor);
        }
    }

    pub fn set_point_with_name(
        &mut self,
        point: AnchorPoint,
        relative_to: Option<String>,
        relative_point: AnchorPoint,
        x_offset: f32,
        y_offset: f32,
    ) {
        // Replace existing anchor with same point, or add new one
        let new_anchor = Anchor {
            point,
            relative_to,
            relative_to_id: None,
            relative_point,
            x_offset,
            y_offset,
        };
        if let Some(existing) = self.anchors.iter_mut().find(|a| a.point == point) {
            *existing = new_anchor;
        } else {
            self.anchors.push(new_anchor);
        }
    }

    pub fn clear_all_points(&mut self) {
        self.anchors.clear();
    }

    pub fn register_event(&mut self, event: &str) {
        self.registered_events.insert(event.to_string());
    }

    pub fn unregister_event(&mut self, event: &str) {
        self.registered_events.remove(event);
    }

    pub fn is_registered_for_event(&self, event: &str) -> bool {
        self.register_all_events || self.registered_events.contains(event)
    }
}

/// Frame strata (draw order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum FrameStrata {
    World,
    Background,
    Low,
    #[default]
    Medium,
    High,
    Dialog,
    Fullscreen,
    FullscreenDialog,
    Tooltip,
}

impl FrameStrata {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "WORLD" => Some(Self::World),
            "BACKGROUND" => Some(Self::Background),
            "LOW" => Some(Self::Low),
            "MEDIUM" => Some(Self::Medium),
            "HIGH" => Some(Self::High),
            "DIALOG" => Some(Self::Dialog),
            "FULLSCREEN" => Some(Self::Fullscreen),
            "FULLSCREEN_DIALOG" => Some(Self::FullscreenDialog),
            "TOOLTIP" => Some(Self::Tooltip),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::World => "WORLD",
            Self::Background => "BACKGROUND",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Dialog => "DIALOG",
            Self::Fullscreen => "FULLSCREEN",
            Self::FullscreenDialog => "FULLSCREEN_DIALOG",
            Self::Tooltip => "TOOLTIP",
        }
    }
}

/// Draw layer for regions (textures/fontstrings) within a frame.
/// Determines render order: BACKGROUND < BORDER < ARTWORK < OVERLAY < HIGHLIGHT
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum DrawLayer {
    Background = 1,
    Border = 2,
    #[default]
    Artwork = 3,
    Overlay = 4,
    Highlight = 5,
}

impl DrawLayer {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "BACKGROUND" => Some(Self::Background),
            "BORDER" => Some(Self::Border),
            "ARTWORK" => Some(Self::Artwork),
            "OVERLAY" => Some(Self::Overlay),
            "HIGHLIGHT" => Some(Self::Highlight),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Background => "BACKGROUND",
            Self::Border => "BORDER",
            Self::Artwork => "ARTWORK",
            Self::Overlay => "OVERLAY",
            Self::Highlight => "HIGHLIGHT",
        }
    }
}
