//! StatusBar fill rendering — computes fill fraction for bar texture children.

use crate::widget::{Color, WidgetType};
use std::collections::HashMap;

/// StatusBar fill info for a bar texture child.
pub(super) struct StatusBarFill {
    pub fraction: f32,
    pub reverse: bool,
    pub vertical: bool,
    pub color: Option<Color>,
}

/// Collect fill info for StatusBar bar textures visible in the render list.
///
/// Only scans the render list (visible frames), not the entire registry.
pub(super) fn collect_statusbar_fills(
    render_list: &[(u64, crate::LayoutRect, Option<crate::LayoutRect>, f32)],
    registry: &crate::widget::WidgetRegistry,
) -> HashMap<u64, StatusBarFill> {
    let mut fills = HashMap::new();
    for &(id, _, _, _) in render_list {
        let Some(frame) = registry.get(id) else {
            continue;
        };
        if frame.widget_type != WidgetType::StatusBar {
            continue;
        }
        let bar_id = frame
            .statusbar_bar_id
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
        fills.insert(
            bar_id,
            StatusBarFill {
                fraction: fraction.clamp(0.0, 1.0),
                reverse: frame.statusbar_reverse_fill,
                vertical: frame.slider_orientation == "VERTICAL",
                color: frame.statusbar_color,
            },
        );
    }
    fills
}

#[cfg(test)]
mod tests {
    use super::collect_statusbar_fills;
    use crate::LayoutRect;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn lua_statusbar_orientation_reaches_collected_fill() {
        let env = WowLuaEnv::new().unwrap();
        env.eval::<()>(
            r#"
            local bar = CreateFrame("StatusBar", "CollectedVerticalBar", UIParent)
            bar:SetOrientation("VERTICAL")
            bar:SetMinMaxValues(0, 100)
            bar:SetValue(25)
            bar:SetStatusBarTexture("Interface\\Buttons\\WHITE8X8")
        "#,
        )
        .unwrap();
        let state = env.state().borrow();
        let id = state
            .widgets
            .get_id_by_name("CollectedVerticalBar")
            .unwrap();
        let fills = collect_statusbar_fills(
            &[(
                id,
                LayoutRect {
                    x: 0.0,
                    y: 0.0,
                    width: 30.0,
                    height: 80.0,
                },
                None,
                1.0,
            )],
            &state.widgets,
        );
        let fill = fills
            .values()
            .next()
            .expect("Lua StatusBar texture has a collected fill");
        assert!(fill.vertical);
        assert_eq!(fill.fraction, 0.25);
    }
}
