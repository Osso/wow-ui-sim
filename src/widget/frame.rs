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

/// Backdrop configuration for frames.
#[derive(Debug, Clone, Default)]
pub struct Backdrop {
    /// Whether backdrop is enabled.
    pub enabled: bool,
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
    /// Vertex color for textures (tinting).
    pub vertex_color: Option<Color>,
    /// Text content (for FontString widgets).
    pub text: Option<String>,
    /// Text color for FontStrings.
    pub text_color: Color,
    /// Font name (for FontString widgets).
    pub font: Option<String>,
    /// Font size (for FontString widgets).
    pub font_size: f32,
    /// Named attributes (for secure frames, unit frames, etc.).
    pub attributes: HashMap<String, AttributeValue>,
    /// Backdrop configuration.
    pub backdrop: Backdrop,
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
            vertex_color: None,
            text: None,
            text_color: Color::new(1.0, 1.0, 1.0, 1.0), // Default white text
            font: None,
            font_size: 12.0,
            attributes: HashMap::new(),
            backdrop: Backdrop::default(),
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_point(
        &mut self,
        point: AnchorPoint,
        relative_to: Option<String>,
        relative_point: AnchorPoint,
        x_offset: f32,
        y_offset: f32,
    ) {
        self.anchors.push(Anchor {
            point,
            relative_to,
            relative_point,
            x_offset,
            y_offset,
        });
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
