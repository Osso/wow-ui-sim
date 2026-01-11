//! Frame widget - the base container for UI elements.

use super::{next_widget_id, Anchor, AnchorPoint, WidgetType};
use std::collections::HashSet;

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
}
