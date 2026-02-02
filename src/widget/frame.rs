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
    /// Frame strata (major draw order).
    pub frame_strata: FrameStrata,
    /// Alpha transparency (0.0 - 1.0).
    pub alpha: f32,
    /// Whether mouse is enabled.
    pub mouse_enabled: bool,
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
    /// Font name (for FontString widgets).
    pub font: Option<String>,
    /// Font size (for FontString widgets).
    pub font_size: f32,
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
    /// Text scale factor (for FontString widgets).
    pub text_scale: f64,
    /// Normal texture path (for Button widgets).
    pub normal_texture: Option<String>,
    /// Pushed texture path (for Button widgets).
    pub pushed_texture: Option<String>,
    /// Highlight texture path (for Button widgets).
    pub highlight_texture: Option<String>,
    /// Disabled texture path (for Button widgets).
    pub disabled_texture: Option<String>,
}

impl Frame {
    pub fn new(widget_type: WidgetType, name: Option<String>, parent_id: Option<u64>) -> Self {
        Self {
            id: next_widget_id(),
            widget_type,
            name,
            parent_id,
            children: Vec::new(),
            width: 0.0,
            height: 0.0,
            anchors: Vec::new(),
            visible: true,
            registered_events: HashSet::new(),
            frame_level: 0,
            frame_strata: FrameStrata::Medium,
            alpha: 1.0,
            mouse_enabled: false,
            texture: None,
            color_texture: None,
            vertex_color: None,
            text: None,
            title: None,
            text_color: Color::new(1.0, 0.8, 0.2, 1.0), // Default gold text for visibility
            font: None,
            font_size: 14.0,
            justify_h: TextJustify::Left,  // WoW defaults to LEFT
            justify_v: TextJustify::Center,
            attributes: HashMap::new(),
            backdrop: Backdrop::default(),
            children_keys: HashMap::new(),
            movable: false,
            resizable: false,
            clamped_to_screen: false,
            is_moving: false,
            word_wrap: false,
            text_scale: 1.0,
            normal_texture: None,
            pushed_texture: None,
            highlight_texture: None,
            disabled_texture: None,
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
        self.registered_events.contains(event)
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
