use super::WidgetRegistry;
use crate::widget::Frame;
use crate::widget::clamp_geometry::{round_retail_layout_value, uses_retail_clamp_producers};

const UI_REFERENCE_HEIGHT: f32 = 768.0;

/// Cached physical-pixel conversion before region scale. Physical height is
/// the simulator's already selected output height (active or configured).
#[derive(Debug)]
pub(super) struct LayoutPixelScale {
    legacy_pixels_per_unit: f32,
    retail_display_scale: f32,
}

impl Default for LayoutPixelScale {
    fn default() -> Self {
        Self {
            legacy_pixels_per_unit: 1.0,
            retail_display_scale: 1.0,
        }
    }
}

impl WidgetRegistry {
    pub(crate) fn with_screen_size(width: f32, height: f32) -> Self {
        let mut registry = Self::default();
        registry.set_layout_physical_height(height);
        registry.set_layout_canvas_size(width, height);
        registry
    }

    pub(crate) fn set_layout_physical_height(&mut self, height: f32) {
        self.layout_pixel_scale = LayoutPixelScale {
            legacy_pixels_per_unit: height / UI_REFERENCE_HEIGHT,
            retail_display_scale: if height == 0.0 {
                1.0
            } else {
                UI_REFERENCE_HEIGHT / height
            },
        };
        self.clear_all_layout_rects();
    }

    pub(crate) fn round_layout_displacement(&self, frame: &Frame, value: f32) -> f32 {
        self.round_layout_value(frame, value, false)
    }

    pub(crate) fn round_layout_dimension(&self, frame: &Frame, value: f32) -> f32 {
        self.round_layout_value(frame, value, true)
    }

    fn round_layout_value(&self, frame: &Frame, value: f32, is_dimension: bool) -> f32 {
        if !frame.round_layout_to_nearest_pixel {
            return value;
        }
        if uses_retail_clamp_producers() {
            return round_retail_layout_value(
                value,
                frame.effective_scale,
                self.layout_pixel_scale.retail_display_scale,
                is_dimension,
            );
        }
        let pixels_per_unit =
            self.layout_pixel_scale.legacy_pixels_per_unit * frame.effective_scale;
        (value * pixels_per_unit).round() / pixels_per_unit
    }
}
