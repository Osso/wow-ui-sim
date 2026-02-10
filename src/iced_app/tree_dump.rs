//! Frame tree dump and debug display methods for the App.

use crate::LayoutRect;

use super::app::App;
use super::layout::{anchor_position, compute_frame_rect};

impl App {
    /// Dump WoW frames for debug server.
    pub(crate) fn dump_wow_frames(&self) -> String {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let screen_width = self.screen_size.get().width;
        let screen_height = self.screen_size.get().height;

        let mut lines = Vec::new();
        lines.push("WoW UI Simulator - Frame Dump".to_string());
        lines.push(format!("Screen: {}x{}", screen_width as i32, screen_height as i32));
        lines.push(String::new());

        // Find root frames (no parent or parent is UIParent)
        let mut root_ids: Vec<u64> = state
            .widgets
            .iter_ids()
            .filter(|&id| {
                state
                    .widgets
                    .get(id)
                    .map(|f| f.parent_id.is_none() || f.parent_id == Some(1))
                    .unwrap_or(false)
            })
            .collect();
        root_ids.sort();

        for id in root_ids {
            dump_frame_recursive(&state.widgets, id, 0, screen_width, screen_height, &mut lines);
        }

        lines.join("\n")
    }

    /// Build a frame tree dump with absolute screen coordinates (WoW units).
    pub(crate) fn build_frame_tree_dump(&self, filter: Option<&str>, visible_only: bool) -> String {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let screen_width = self.screen_size.get().width;
        let screen_height = self.screen_size.get().height;

        let mut lines = Vec::new();

        let mut root_ids: Vec<u64> = state
            .widgets
            .iter_ids()
            .filter(|&id| {
                state
                    .widgets
                    .get(id)
                    .map(|f| f.parent_id.is_none())
                    .unwrap_or(false)
            })
            .collect();
        root_ids.sort();

        for id in root_ids {
            build_tree_recursive(
                &state.widgets, id, "", true,
                screen_width, screen_height,
                filter, visible_only, &mut lines,
            );
        }

        if lines.is_empty() {
            "No frames found".to_string()
        } else {
            lines.join("\n")
        }
    }
}

fn dump_frame_recursive(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    depth: usize,
    screen_width: f32,
    screen_height: f32,
    lines: &mut Vec<String>,
) {
    let Some(frame) = registry.get(id) else {
        return;
    };

    let rect = compute_frame_rect(registry, id, screen_width, screen_height);
    let indent = "  ".repeat(depth);

    let name = frame.name.as_deref().unwrap_or("(anon)");
    let type_str = frame.widget_type.as_str();

    // Build warning flags
    let mut warnings = Vec::new();
    if rect.width <= 0.0 {
        warnings.push("ZERO_WIDTH");
    }
    if rect.height <= 0.0 {
        warnings.push("ZERO_HEIGHT");
    }
    if rect.x + rect.width < 0.0 || rect.x > screen_width {
        warnings.push("OFFSCREEN_X");
    }
    if rect.y + rect.height < 0.0 || rect.y > screen_height {
        warnings.push("OFFSCREEN_Y");
    }
    if !frame.visible {
        warnings.push("HIDDEN");
    }

    let warning_str = if warnings.is_empty() {
        String::new()
    } else {
        format!(" ! {}", warnings.join(", "))
    };

    lines.push(format!(
        "{}{} [{}] ({:.0},{:.0} {}x{}){warning_str}",
        indent, name, type_str,
        rect.x, rect.y, rect.width as i32, rect.height as i32,
    ));

    for &child_id in &frame.children {
        dump_frame_recursive(registry, child_id, depth + 1, screen_width, screen_height, lines);
    }
}

#[allow(clippy::too_many_arguments)]
fn build_tree_recursive(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    prefix: &str,
    is_last: bool,
    screen_width: f32,
    screen_height: f32,
    filter: Option<&str>,
    visible_only: bool,
    lines: &mut Vec<String>,
) {
    let Some(frame) = registry.get(id) else {
        return;
    };

    if visible_only && !frame.visible {
        return;
    }

    let name = tree_display_name(frame);
    let matches_filter = filter.map(|f| name.to_lowercase().contains(&f.to_lowercase())).unwrap_or(true);
    let rect = compute_frame_rect(registry, id, screen_width, screen_height);

    let mut children: Vec<u64> = frame.children.to_vec();
    if filter.is_some() || visible_only {
        children.retain(|&child_id| {
            subtree_matches(registry, child_id, filter, visible_only)
        });
    }

    if !matches_filter && children.is_empty() {
        return;
    }

    let connector = "+- ";
    let size_info = tree_size_mismatch_info(frame, &rect);
    let vis_str = if frame.visible { "" } else { " [hidden]" };
    lines.push(format!(
        "{prefix}{connector}{name} ({}) @ ({:.0},{:.0}) {:.0}x{:.0}{size_info}{vis_str}",
        frame.widget_type.as_str(),
        rect.x, rect.y, rect.width, rect.height,
    ));

    let child_prefix = format!("{}{}", prefix, if is_last { "   " } else { "|  " });
    emit_anchor_lines(registry, frame, &child_prefix, screen_width, screen_height, lines);

    if let Some(tex_path) = &frame.texture {
        lines.push(format!("{}   [texture] {tex_path}", child_prefix));
    }

    for (i, &child_id) in children.iter().enumerate() {
        build_tree_recursive(
            registry, child_id, &child_prefix, i == children.len() - 1,
            screen_width, screen_height, filter, visible_only, lines,
        );
    }
}

/// Derive a display name for a frame in the tree dump.
fn tree_display_name(frame: &crate::widget::Frame) -> &str {
    let raw_name = frame.name.as_deref();
    let is_anon = raw_name
        .map(|n| n.starts_with("__anon_") || n.starts_with("__fs_") || n.starts_with("__tex_"))
        .unwrap_or(true);
    if let Some(text) = frame.text.as_ref().filter(|_| is_anon) {
        if text.len() > 20 {
            Box::leak(format!("\"{}...\"", &text[..17]).into_boxed_str())
        } else {
            Box::leak(format!("\"{text}\"").into_boxed_str())
        }
    } else {
        raw_name.unwrap_or("(anon)")
    }
}

/// Format size mismatch info if stored size differs from computed rect.
fn tree_size_mismatch_info(frame: &crate::widget::Frame, rect: &LayoutRect) -> String {
    if (frame.width - rect.width).abs() > 0.1 || (frame.height - rect.height).abs() > 0.1 {
        format!(" [stored={:.0}x{:.0}]", frame.width, frame.height)
    } else {
        String::new()
    }
}

/// Emit anchor detail lines for a frame in the tree dump.
fn emit_anchor_lines(
    registry: &crate::widget::WidgetRegistry,
    frame: &crate::widget::Frame,
    child_prefix: &str,
    screen_width: f32,
    screen_height: f32,
    lines: &mut Vec<String>,
) {
    let parent_rect = if let Some(parent_id) = frame.parent_id {
        compute_frame_rect(registry, parent_id, screen_width, screen_height)
    } else {
        LayoutRect { x: 0.0, y: 0.0, width: screen_width, height: screen_height }
    };

    for anchor in &frame.anchors {
        let (rel_name, relative_rect) = if let Some(rel_id) = anchor.relative_to_id {
            let rel_rect = compute_frame_rect(registry, rel_id as u64, screen_width, screen_height);
            let name = registry.get(rel_id as u64)
                .and_then(|f| f.name.as_deref())
                .unwrap_or("(anon)");
            (name, rel_rect)
        } else {
            (anchor.relative_to.as_deref().unwrap_or("$parent"), parent_rect)
        };

        let (anchor_x, anchor_y) = anchor_position(
            anchor.relative_point,
            relative_rect.x, relative_rect.y,
            relative_rect.width, relative_rect.height,
        );
        lines.push(format!(
            "{}   [anchor] {} -> {}:{} offset({:.0},{:.0}) -> ({:.0},{:.0})",
            child_prefix, anchor.point.as_str(),
            rel_name, anchor.relative_point.as_str(),
            anchor.x_offset, anchor.y_offset,
            anchor_x + anchor.x_offset, anchor_y - anchor.y_offset
        ));
    }
}

/// Check if a frame or any descendant matches the filter criteria.
fn subtree_matches(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    filter: Option<&str>,
    visible_only: bool,
) -> bool {
    let Some(frame) = registry.get(id) else {
        return false;
    };

    if visible_only && !frame.visible {
        return false;
    }

    let name = frame.name.as_deref().unwrap_or("(anon)");
    let matches = filter.map(|f| name.to_lowercase().contains(&f.to_lowercase())).unwrap_or(true);

    if matches {
        return true;
    }

    for &child_id in &frame.children {
        if subtree_matches(registry, child_id, filter, visible_only) {
            return true;
        }
    }

    false
}
