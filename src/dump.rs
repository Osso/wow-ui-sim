//! Frame tree dump and diagnostic utilities.

use crate::iced_app::layout::compute_frame_rect;
use crate::widget::{Frame, WidgetRegistry, WidgetType};

/// Dump the frame tree to stdout.
pub fn print_frame_tree(
    widgets: &WidgetRegistry,
    filter: Option<&str>,
    filter_key: Option<&str>,
    visible_only: bool,
    screen_width: f32,
    screen_height: f32,
) {
    let mut roots = collect_root_frames(widgets);
    roots.sort_by(|a, b| {
        let name_a = a.1.as_deref().unwrap_or("");
        let name_b = b.1.as_deref().unwrap_or("");
        name_a.cmp(name_b)
    });

    print_anchor_diagnostic(widgets);
    eprintln!("\n=== Frame Tree ===\n");

    if let Some(key_filter) = filter_key {
        print_subtrees_matching_key(widgets, &roots, key_filter, visible_only, screen_width, screen_height);
    } else {
        for (id, _) in &roots {
            print_frame(widgets, *id, 0, filter, visible_only, screen_width, screen_height);
        }
    }
}

/// Collect root frames (no parent).
fn collect_root_frames(widgets: &WidgetRegistry) -> Vec<(u64, Option<String>)> {
    widgets
        .iter_ids()
        .filter_map(|id| {
            let w = widgets.get(id)?;
            if w.parent_id.is_none() {
                Some((id, w.name.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// Print anchor diagnostic showing counts and details of unanchored frames.
fn print_anchor_diagnostic(widgets: &WidgetRegistry) {
    let mut anchored = 0;
    let mut unanchored = 0;
    let mut unanchored_keys: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for id in widgets.iter_ids() {
        let Some(w) = widgets.get(id) else { continue };
        if !w.anchors.is_empty() {
            anchored += 1;
            continue;
        }
        unanchored += 1;
        let parent_key = find_parent_key(widgets, w, id);
        let parent_name = w
            .parent_id
            .and_then(|pid| widgets.get(pid))
            .and_then(|p| p.name.clone())
            .unwrap_or_else(|| "(no parent)".into());
        let detail = format!(
            "  {:?} on {} ({:?})",
            w.widget_type, parent_name, w.name
        );
        let key = parent_key.unwrap_or_else(|| "(no key)".into());
        unanchored_keys.entry(key).or_default().push(detail);
    }
    print_anchor_summary(&unanchored_keys, anchored, unanchored);
}

/// Print anchor summary counts and top unanchored keys.
fn print_anchor_summary(
    unanchored_keys: &std::collections::HashMap<String, Vec<String>>,
    anchored: usize,
    unanchored: usize,
) {
    let mut kv: Vec<_> = unanchored_keys
        .iter()
        .map(|(k, v)| (k.clone(), v.len()))
        .collect();
    kv.sort_by(|a, b| b.1.cmp(&a.1));
    eprintln!("Anchored: {anchored}, Unanchored: {unanchored}");
    eprintln!("Top unanchored keys: {:?}", &kv[..kv.len().min(15)]);
    for (key, _) in kv.iter().take(5) {
        if let Some(details) = unanchored_keys.get(key) {
            eprintln!("  {}:", key);
            for d in details.iter().take(3) {
                eprintln!("  {}", d);
            }
        }
    }
    if let Some(no_key) = unanchored_keys.get("(no key)") {
        print_no_key_breakdown(no_key);
    }
}

/// Break down "(no key)" unanchored frames by widget type.
fn print_no_key_breakdown(no_key: &[String]) {
    let mut by_type: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for d in no_key {
        let wtype = d.trim().split(' ').next().unwrap_or("?");
        *by_type.entry(wtype.to_string()).or_default() += 1;
    }
    let mut tv: Vec<_> = by_type.iter().collect();
    tv.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("  (no key) by type: {:?}", tv);
}

/// Find the parentKey name for a widget in its parent's children_keys.
fn find_parent_key(widgets: &WidgetRegistry, w: &Frame, id: u64) -> Option<String> {
    let pid = w.parent_id?;
    let p = widgets.get(pid)?;
    p.children_keys
        .iter()
        .find(|(_, cid)| **cid == id)
        .map(|(k, _)| k.clone())
}

/// Find frames whose parentKey matches the filter and print their full subtrees.
fn print_subtrees_matching_key(
    widgets: &WidgetRegistry,
    roots: &[(u64, Option<String>)],
    key_filter: &str,
    visible_only: bool,
    sw: f32,
    sh: f32,
) {
    let key_lower = key_filter.to_lowercase();
    let matching_ids = collect_key_matches(widgets, roots, &key_lower);
    for id in matching_ids {
        print_frame_subtree(widgets, id, 0, visible_only, sw, sh);
    }
}

/// Recursively collect frame IDs whose parentKey or name matches the filter.
fn collect_key_matches(
    widgets: &WidgetRegistry,
    roots: &[(u64, Option<String>)],
    key_lower: &str,
) -> Vec<u64> {
    let mut result = Vec::new();
    for &(id, _) in roots {
        collect_key_matches_recursive(widgets, id, key_lower, &mut result);
    }
    result
}

fn collect_key_matches_recursive(
    widgets: &WidgetRegistry,
    id: u64,
    key_lower: &str,
    result: &mut Vec<u64>,
) {
    let Some(frame) = widgets.get(id) else { return };
    let display = resolve_display_name(widgets, frame, id);
    if display.to_lowercase().contains(key_lower) {
        result.push(id);
        return; // Don't recurse into children - they'll be printed as subtree
    }
    for &child_id in &frame.children {
        collect_key_matches_recursive(widgets, child_id, key_lower, result);
    }
}

/// Print a frame and its entire subtree unconditionally (no name filter).
fn print_frame_subtree(
    widgets: &WidgetRegistry,
    id: u64,
    depth: usize,
    visible_only: bool,
    sw: f32,
    sh: f32,
) {
    let Some(frame) = widgets.get(id) else { return };
    if visible_only && !frame.visible {
        return;
    }
    let display_name = resolve_display_name(widgets, frame, id);
    print_frame_line(frame, id, &display_name, depth, widgets, sw, sh);
    for &child_id in &frame.children {
        print_frame_subtree(widgets, child_id, depth + 1, visible_only, sw, sh);
    }
}

/// Recursively print a frame and its children.
#[allow(clippy::too_many_arguments)]
fn print_frame(
    widgets: &WidgetRegistry,
    id: u64,
    depth: usize,
    filter: Option<&str>,
    visible_only: bool,
    sw: f32,
    sh: f32,
) {
    let Some(frame) = widgets.get(id) else { return };

    if visible_only && !frame.visible {
        return;
    }

    let display_name = resolve_display_name(widgets, frame, id);
    let matches_filter = filter
        .map(|f| display_name.to_lowercase().contains(&f.to_lowercase()))
        .unwrap_or(true);

    if matches_filter {
        print_frame_line(frame, id, &display_name, depth, widgets, sw, sh);
    }

    for &child_id in &frame.children {
        print_frame(widgets, child_id, depth + 1, filter, visible_only, sw, sh);
    }
}

/// Format and print a single frame line with stored size and computed layout rect.
fn print_frame_line(
    frame: &Frame,
    id: u64,
    display_name: &str,
    depth: usize,
    widgets: &WidgetRegistry,
    sw: f32,
    sh: f32,
) {
    let indent = "  ".repeat(depth);
    let vis = if frame.visible { "visible" } else { "hidden" };
    let rect = compute_frame_rect(widgets, id, sw, sh);
    let size_str = format_size_str(frame, &rect);
    let keys_str = format_keys_str(frame);
    let text_str = resolve_display_text(widgets, frame)
        .map(|t| format!(" text={:?}", t))
        .unwrap_or_default();
    let font_str = format_font_str(frame);
    println!(
        "{indent}{display_name} [{:?}] {size_str} {vis}{text_str}{font_str}{keys_str}",
        frame.widget_type,
    );
}

/// Format size string: show computed rect, add stored size if it differs.
fn format_size_str(frame: &Frame, rect: &crate::LayoutRect) -> String {
    let stored_differs = (frame.width - rect.width).abs() > 0.5
        || (frame.height - rect.height).abs() > 0.5;
    if stored_differs && (frame.width > 0.0 || frame.height > 0.0) {
        format!(
            "({}x{}) [stored={}x{}]",
            rect.width as i32, rect.height as i32,
            frame.width as i32, frame.height as i32,
        )
    } else {
        format!("({}x{})", rect.width as i32, rect.height as i32)
    }
}

/// Format children_keys display string.
fn format_keys_str(frame: &Frame) -> String {
    let keys: Vec<_> = frame.children_keys.keys().collect();
    if keys.is_empty() {
        String::new()
    } else {
        format!(
            " keys=[{}]",
            keys.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        )
    }
}

/// Format font info for FontString widgets.
fn format_font_str(frame: &Frame) -> String {
    if frame.widget_type == WidgetType::FontString {
        format!(
            " font={:?} size={}",
            frame.font.as_deref().unwrap_or("(none)"),
            frame.font_size
        )
    } else {
        String::new()
    }
}

/// Resolve a display name for a frame: global name, parentKey, or "(anonymous)".
fn resolve_display_name(widgets: &WidgetRegistry, frame: &Frame, id: u64) -> String {
    if let Some(ref name) = frame.name
        && !name.starts_with("__anon_")
            && !name.starts_with("__frame_")
            && !name.starts_with("__tex_")
            && !name.starts_with("__fs_")
        {
            return name.clone();
        }

    if let Some(parent_id) = frame.parent_id
        && let Some(parent) = widgets.get(parent_id) {
            for (key, &child_id) in &parent.children_keys {
                if child_id == id {
                    return format!(".{}", key);
                }
            }
        }

    frame
        .name
        .as_deref()
        .unwrap_or("(anonymous)")
        .to_string()
}

/// Get display text for a frame: its own text, or text from a Title/TitleText child.
fn resolve_display_text(widgets: &WidgetRegistry, frame: &Frame) -> Option<String> {
    if let Some(ref t) = frame.text
        && !t.is_empty() {
            return Some(strip_wow_escapes(t));
        }

    for key in &["Title", "TitleText"] {
        if let Some(&child_id) = frame.children_keys.get(*key)
            && let Some(child) = widgets.get(child_id)
                && let Some(ref t) = child.text
                    && !t.is_empty() {
                        return Some(strip_wow_escapes(t));
                    }
    }

    None
}

/// Strip WoW escape sequences (|T...|t texture, |c...|r color) for cleaner display.
pub fn strip_wow_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '|' {
            skip_wow_escape(&mut chars);
        } else {
            result.push(c);
        }
    }
    result.trim().to_string()
}

/// Skip a single WoW escape sequence starting after the '|' character.
fn skip_wow_escape(chars: &mut std::iter::Peekable<std::str::Chars>) {
    match chars.peek() {
        // |Htype:data|h — hyperlink open tag, skip to closing |h
        Some('H') => {
            chars.next();
            while let Some(ch) = chars.next() {
                if ch == '|' && chars.peek() == Some(&'h') {
                    chars.next();
                    break;
                }
            }
        }
        // |h — hyperlink close tag
        Some('h') => {
            chars.next();
        }
        // |T...|t — texture
        Some('T') => {
            chars.next();
            while let Some(&ch) = chars.peek() {
                chars.next();
                if ch == '|' {
                    chars.next(); // skip 't'
                    break;
                }
            }
        }
        Some('t') => {
            chars.next();
        }
        // |cXXXXXXXX — color
        Some('c') => {
            chars.next();
            for _ in 0..8 {
                chars.next();
            }
        }
        // |r — color reset
        Some('r') => {
            chars.next();
        }
        _ => {}
    }
}
