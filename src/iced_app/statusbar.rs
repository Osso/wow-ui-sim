//! StatusBar fill rendering — computes fill fraction for bar texture children.

use std::collections::HashMap;
use crate::widget::{Color, WidgetType};

/// StatusBar fill info for a bar texture child.
pub(super) struct StatusBarFill {
    pub fraction: f32,
    pub reverse: bool,
    pub color: Option<Color>,
}

/// Collect fill info for StatusBar bar textures visible in the render list.
///
/// Only scans the render list (visible frames), not the entire registry.
pub(super) fn collect_statusbar_fills(
    render_list: &[(u64, crate::LayoutRect, f32)],
    registry: &crate::widget::WidgetRegistry,
) -> HashMap<u64, StatusBarFill> {
    let mut fills = HashMap::new();
    for &(id, _, _) in render_list {
        let Some(frame) = registry.get(id) else { continue };
        if frame.widget_type != WidgetType::StatusBar {
            continue;
        }
        let bar_id = frame.statusbar_bar_id
            .or_else(|| frame.children_keys.get("BarTexture").copied())
            .or_else(|| frame.children_keys.get("StatusBarTexture").copied())
            .or_else(|| frame.children_keys.get("Bar").copied());
        let Some(bar_id) = bar_id else { continue };
        let range = frame.statusbar_max - frame.statusbar_min;
        let fraction = if range > 0.0 {
            ((frame.statusbar_value - frame.statusbar_min) / range) as f32
        } else {
            0.0
        };
        fills.insert(bar_id, StatusBarFill {
            fraction: fraction.clamp(0.0, 1.0),
            reverse: frame.statusbar_reverse_fill,
            color: frame.statusbar_color,
        });
    }
    fills
}
