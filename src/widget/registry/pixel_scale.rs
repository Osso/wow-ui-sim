use super::WidgetRegistry;
use crate::widget::Frame;

const UI_REFERENCE_HEIGHT: f32 = 768.0;

/// Cached physical-pixel conversion for the layout registry, before region scale.
#[derive(Debug)]
pub(super) struct LayoutPixelScale(f32);

impl Default for LayoutPixelScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl WidgetRegistry {
    pub(crate) fn with_physical_height(height: f32) -> Self {
        let mut registry = Self::default();
        registry.set_layout_physical_height(height);
        registry
    }

    pub(crate) fn set_layout_physical_height(&mut self, height: f32) {
        self.layout_pixel_scale = LayoutPixelScale(height / UI_REFERENCE_HEIGHT);
        self.clear_all_layout_rects();
    }

    pub(crate) fn round_layout_value(&self, frame: &Frame, value: f32) -> f32 {
        if !frame.round_layout_to_nearest_pixel {
            return value;
        }
        let pixels_per_unit = self.layout_pixel_scale.0 * frame.effective_scale;
        (value * pixels_per_unit).round() / pixels_per_unit
    }
}
