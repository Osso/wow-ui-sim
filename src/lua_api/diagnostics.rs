//! Diagnostic dump functions for OnUpdate handlers and frame trees.

use super::layout::{compute_frame_rect, get_parent_depth, LayoutRect};
use super::state::SimState;

/// Dump all frame positions for debugging.
/// Returns a formatted string similar to iced-debug output.
#[allow(clippy::format_push_string)]
pub fn dump_frames(state: &SimState) -> String {
    let screen_width = state.screen_width;
    let screen_height = state.screen_height;

    let mut output = format!("[WoW Frames: {}x{}]\n\n", screen_width, screen_height);

    let mut frames: Vec<_> = state.widgets.iter_ids().collect();
    frames.sort_by(|&a, &b| {
        let fa = state.widgets.get(a);
        let fb = state.widgets.get(b);
        match (fa, fb) {
            (Some(fa), Some(fb)) => fa
                .frame_strata
                .cmp(&fb.frame_strata)
                .then_with(|| fa.frame_level.cmp(&fb.frame_level)),
            _ => std::cmp::Ordering::Equal,
        }
    });

    for id in frames {
        let Some(frame) = state.widgets.get(id) else { continue };
        let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
        format_frame_entry(&mut output, &state.widgets, id, frame, &rect);
    }

    output
}

/// Format a single frame entry for the debug dump output.
fn format_frame_entry(
    output: &mut String,
    widgets: &crate::widget::WidgetRegistry,
    id: u64,
    frame: &crate::widget::Frame,
    rect: &LayoutRect,
) {
    use std::fmt::Write;

    let name = frame.name.as_deref().unwrap_or("(anon)");
    let vis = if frame.visible { "" } else { " HIDDEN" };
    let mouse = if frame.mouse_enabled { " mouse" } else { "" };
    let depth = get_parent_depth(widgets, id);
    let indent = "  ".repeat(depth);
    let parent_name = frame
        .parent_id
        .and_then(|pid| widgets.get(pid))
        .and_then(|p| p.name.as_deref())
        .unwrap_or("(root)");

    let _ = writeln!(
        output,
        "{}{} [{}] ({:.0},{:.0} {:.0}x{:.0}){}{} parent={}",
        indent, name, frame.widget_type.as_str(),
        rect.x, rect.y, rect.width, rect.height,
        vis, mouse, parent_name,
    );

    if !frame.anchors.is_empty() {
        let anchor = &frame.anchors[0];
        let _ = writeln!(
            output,
            "{}  └─ {:?} -> {:?} offset ({:.0},{:.0})",
            indent, anchor.point, anchor.relative_point, anchor.x_offset, anchor.y_offset
        );
    } else {
        let _ = writeln!(output, "{}  └─ (no anchors - topleft of parent)", indent);
    }
}

