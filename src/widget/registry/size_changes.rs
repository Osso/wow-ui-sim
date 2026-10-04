//! Resolved frame sizes and the OnSizeChanged change set fed by layout resolution.

use super::WidgetRegistry;
use crate::widget::{Frame, WidgetType};

/// Size deltas below this are float noise from rect/scale division, not resizes.
const SIZE_EPSILON: f32 = 1e-3;

/// Whether `GetSize` reads the resolved layout rect instead of the explicit size.
pub(crate) fn has_queryable_rect(frame: &Frame, id: u64) -> bool {
    !frame.anchors.is_empty() || frame.name.as_deref() == Some("UIParent") || id == 1
}

impl WidgetRegistry {
    /// The frame-unit size `GetSize` reports from the frame's current layout state.
    pub(crate) fn resolved_size(&self, id: u64) -> (f32, f32) {
        let Some(frame) = self.get(id) else {
            return (0.0, 0.0);
        };
        // FontString:GetWidth reports its text extent after SetText/SetWidth,
        // even when anchors also determine the eventual render rect.
        if frame.widget_type == WidgetType::FontString && frame.width > 0.0 {
            return (
                self.round_layout_value(frame, frame.width),
                self.round_layout_value(frame, frame.height),
            );
        }
        if has_queryable_rect(frame, id)
            && let Some(rect) = frame.layout_rect
        {
            let eff_scale = frame.effective_scale.max(1e-6);
            return (rect.width / eff_scale, rect.height / eff_scale);
        }
        (
            self.round_layout_value(frame, frame.width),
            self.round_layout_value(frame, frame.height),
        )
    }

    /// Called whenever layout resolution writes a frame's rect: queue OnSizeChanged
    /// when the resolved size differs from the last size reported to scripts.
    pub(crate) fn note_layout_size(&mut self, id: u64) {
        let size = self.resolved_size(id);
        let Some(frame) = self.widgets.get_mut(&id) else {
            return;
        };
        if !supports_scripts(frame.widget_type) {
            return;
        }
        let (old_width, old_height) = frame.reported_size;
        if (size.0 - old_width).abs() <= SIZE_EPSILON && (size.1 - old_height).abs() <= SIZE_EPSILON
        {
            return;
        }
        frame.reported_size = size;
        self.size_changed_ids.insert(id);
    }

    /// Take queued size changes in id order, with the size each reports.
    pub(crate) fn drain_size_changes(&mut self) -> Vec<(u64, (f32, f32))> {
        if self.size_changed_ids.is_empty() {
            return Vec::new();
        }
        let mut ids: Vec<u64> = self.size_changed_ids.drain().collect();
        ids.sort_unstable();
        ids.into_iter()
            .filter_map(|id| self.get(id).map(|frame| (id, frame.reported_size)))
            .collect()
    }
}

/// Regions (textures, font strings, lines) carry no script handlers.
fn supports_scripts(widget_type: WidgetType) -> bool {
    !matches!(
        widget_type,
        WidgetType::FontString | WidgetType::Texture | WidgetType::Line
    )
}
