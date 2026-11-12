//! Engine layout for a sized, bound Slider thumb without authored anchors.
//! Direction follows cached Blizzard scrollbar consumers. Contained travel,
//! collapsed-range centering and oversized-thumb centering are simulator policy,
//! not a claim about native Slider endpoint insets.

use crate::LayoutRect;
use crate::widget::{Frame, WidgetRegistry, WidgetType};

fn bound_slider_thumb_parent<'a>(registry: &'a WidgetRegistry, frame: &Frame) -> Option<&'a Frame> {
    if frame.widget_type != WidgetType::Texture || !frame.anchors.is_empty() {
        return None;
    }
    let parent = registry.get(frame.parent_id?)?;
    let bound = parent.children_keys.get("ThumbTexture") == Some(&frame.id);
    (parent.widget_type == WidgetType::Slider && bound).then_some(parent)
}

fn has_positive_finite_dimensions(width: f32, height: f32) -> bool {
    width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0
}

fn has_positive_finite_size(rect: LayoutRect) -> bool {
    has_positive_finite_dimensions(rect.width, rect.height)
}

pub(crate) fn has_sized_slider_thumb_binding(registry: &WidgetRegistry, frame: &Frame) -> bool {
    let is_bound = bound_slider_thumb_parent(registry, frame).is_some();
    let width = registry.round_layout_value(frame, frame.width) * frame.effective_scale;
    let height = registry.round_layout_value(frame, frame.height) * frame.effective_scale;
    is_bound && has_positive_finite_dimensions(width, height)
}

/// Admission uses the real stored layout result, never binding identity alone.
/// Both query and render paths resolve dirty layout before consulting this.
pub(crate) fn resolved_slider_thumb_rect(
    registry: &WidgetRegistry,
    frame: &Frame,
) -> Option<LayoutRect> {
    bound_slider_thumb_parent(registry, frame)?;
    let rect = frame.layout_rect?;
    let finite_origin = rect.x.is_finite() && rect.y.is_finite();
    (finite_origin && has_positive_finite_size(rect)).then_some(rect)
}

pub(super) fn slider_thumb_rect(
    registry: &WidgetRegistry,
    frame: &Frame,
    track: LayoutRect,
) -> Option<LayoutRect> {
    let slider = bound_slider_thumb_parent(registry, frame)?;
    let mut rect = LayoutRect {
        x: track.x,
        y: track.y,
        width: registry.round_layout_value(frame, frame.width) * frame.effective_scale,
        height: registry.round_layout_value(frame, frame.height) * frame.effective_scale,
    };
    if !has_positive_finite_size(rect) {
        return None;
    }
    let fraction = slider_fraction(slider);
    rect.x += (track.width - rect.width) * 0.5;
    rect.y += (track.height - rect.height) * 0.5;
    if slider.slider_orientation == "VERTICAL" {
        rect.y += (fraction - 0.5) * (track.height - rect.height).max(0.0);
    } else {
        rect.x += (fraction - 0.5) * (track.width - rect.width).max(0.0);
    }
    Some(rect)
}

fn slider_fraction(slider: &Frame) -> f32 {
    let span = slider.slider_max - slider.slider_min;
    if span <= 0.0 || !span.is_finite() {
        return 0.5;
    }
    let fraction = (slider.slider_value - slider.slider_min) / span;
    if !fraction.is_finite() {
        return 0.5;
    }
    fraction.clamp(0.0, 1.0) as f32
}
