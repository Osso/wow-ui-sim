//! Frame tree dump and diagnostic utilities.
//!
//! Single implementation used by both `wow-sim dump-tree` (headless) and
//! the connected `wow-cli dump-tree` (via iced_app debug server).

use crate::iced_app::layout::{anchor_position, compute_frame_rect};
use crate::widget::{Frame, WidgetRegistry, WidgetType};
use crate::LayoutRect;

// ── Public entry points ─────────────────────────────────────────────

/// Print the frame tree to stdout (headless subcommand).
pub fn print_frame_tree(
    widgets: &WidgetRegistry,
    filter: Option<&str>,
    filter_key: Option<&str>,
    visible_only: bool,
    screen_width: f32,
    screen_height: f32,
) {
    print_anchor_diagnostic(widgets);
    eprintln!("\n=== Frame Tree ===\n");
    let lines = build_tree(widgets, filter, filter_key, visible_only, screen_width, screen_height);
    for line in &lines {
        println!("{line}");
    }
}

/// Build the frame tree as lines (for connected dump-tree server).
pub fn build_tree(
    widgets: &WidgetRegistry,
    filter: Option<&str>,
    filter_key: Option<&str>,
    visible_only: bool,
    screen_width: f32,
    screen_height: f32,
) -> Vec<String> {
    let mut roots = collect_root_frames(widgets);
    roots.sort_by(|a, b| {
        let na = a.1.as_deref().unwrap_or("");
        let nb = b.1.as_deref().unwrap_or("");
        na.cmp(nb)
    });

    let mut lines = Vec::new();
    if let Some(key_filter) = filter_key {
        let key_lower = key_filter.to_lowercase();
        let matching = collect_key_matches(widgets, &roots, &key_lower);
        for id in matching {
            emit_subtree(widgets, id, 0, visible_only, screen_width, screen_height, &mut lines);
        }
    } else {
        for (id, _) in &roots {
            emit_filtered(widgets, *id, 0, filter, visible_only, screen_width, screen_height, &mut lines);
        }
    }
    lines
}

/// Build a compact dump with warning flags (for debug server Dump command).
pub fn build_warning_dump(
    widgets: &WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("WoW UI Simulator - Frame Dump".to_string());
    lines.push(format!("Screen: {}x{}", screen_width as i32, screen_height as i32));
    lines.push(String::new());

    let mut root_ids: Vec<u64> = widgets.iter_ids()
        .filter(|&id| {
            widgets.get(id)
                .map(|f| f.parent_id.is_none() || f.parent_id == Some(1))
                .unwrap_or(false)
        })
        .collect();
    root_ids.sort();

    for id in root_ids {
        emit_warning_recursive(widgets, id, 0, screen_width, screen_height, &mut lines);
    }
    lines
}

// ── Frame line formatting ───────────────────────────────────────────

/// Emit a single frame line with computed rect, stored size, anchors, texture.
fn emit_frame_line(
    frame: &Frame,
    id: u64,
    display_name: &str,
    depth: usize,
    widgets: &WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
    lines: &mut Vec<String>,
) {
    let indent = "  ".repeat(depth);
    let vis = if frame.visible { "visible" } else { "hidden" };
    let rect = compute_frame_rect(widgets, id, screen_width, screen_height);
    let size_str = format_size_str(frame, &rect);
    let stale_str = format_stale_str(frame, &rect);
    let info_str = format_info_str(frame, &rect);
    let text_str = format_text_str(widgets, frame);
    let font_str = format_font_str(frame);
    let strata_str = format!(" {}:{}", frame.frame_strata.as_str(), frame.frame_level);
    let mask_str = if frame.is_mask { " MASK" } else { "" };
    lines.push(format!(
        "{indent}{display_name} [{:?}] {size_str} {vis}{strata_str}{mask_str}{stale_str}{info_str}{text_str}{font_str}",
        frame.widget_type,
    ));
    emit_anchor_lines(widgets, frame, &indent, screen_width, screen_height, lines);
    if let Some(tex_path) = &frame.texture {
        lines.push(format!("{indent}  [texture] {tex_path}"));
    }
}

/// Emit anchor detail lines for a frame.
fn emit_anchor_lines(
    widgets: &WidgetRegistry,
    frame: &Frame,
    indent: &str,
    screen_width: f32,
    screen_height: f32,
    lines: &mut Vec<String>,
) {
    if frame.anchors.is_empty() {
        return;
    }
    let parent_rect = frame.parent_id
        .map(|pid| compute_frame_rect(widgets, pid, screen_width, screen_height))
        .unwrap_or(LayoutRect { x: 0.0, y: 0.0, width: screen_width, height: screen_height });

    for anchor in &frame.anchors {
        let (rel_name, rel_rect) = if let Some(rel_id) = anchor.relative_to_id {
            let rect = compute_frame_rect(widgets, rel_id as u64, screen_width, screen_height);
            let name = widgets.get(rel_id as u64)
                .and_then(|f| f.name.as_deref())
                .unwrap_or("(anon)");
            (name, rect)
        } else {
            (anchor.relative_to.as_deref().unwrap_or("$parent"), parent_rect)
        };
        let (ax, ay) = anchor_position(
            anchor.relative_point,
            rel_rect.x, rel_rect.y, rel_rect.width, rel_rect.height,
        );
        lines.push(format!(
            "{indent}  [anchor] {} -> {}:{} offset({:.0},{:.0}) -> ({:.0},{:.0})",
            anchor.point.as_str(), rel_name, anchor.relative_point.as_str(),
            anchor.x_offset, anchor.y_offset,
            ax + anchor.x_offset, ay - anchor.y_offset,
        ));
    }
}

// ── Formatters ──────────────────────────────────────────────────────

/// Computed rect, with stored size annotation when it differs.
fn format_size_str(frame: &Frame, rect: &LayoutRect) -> String {
    let differs = (frame.width - rect.width).abs() > 0.5
        || (frame.height - rect.height).abs() > 0.5;
    if differs && (frame.width > 0.0 || frame.height > 0.0) {
        format!(
            "({}x{}) [stored={}x{}]",
            rect.width as i32, rect.height as i32,
            frame.width as i32, frame.height as i32,
        )
    } else {
        format!("({}x{})", rect.width as i32, rect.height as i32)
    }
}

/// layout_rect staleness: show if cached rect diverges from computed rect.
fn format_stale_str(frame: &Frame, rect: &LayoutRect) -> String {
    match frame.layout_rect {
        Some(lr) if (lr.x - rect.x).abs() > 0.5
            || (lr.y - rect.y).abs() > 0.5
            || (lr.width - rect.width).abs() > 0.5
            || (lr.height - rect.height).abs() > 0.5 =>
        {
            format!(" [layout_rect=({:.0},{:.0}) {:.0}x{:.0}]", lr.x, lr.y, lr.width, lr.height)
        }
        None => " [layout_rect=None]".to_string(),
        _ => String::new(),
    }
}

fn format_info_str(frame: &Frame, rect: &LayoutRect) -> String {
    format!(
        " x={}, y={}, alpha={:.2}",
        rect.x as i32, rect.y as i32,
        frame.alpha,
    )
}

fn format_text_str(widgets: &WidgetRegistry, frame: &Frame) -> String {
    resolve_display_text(widgets, frame)
        .map(|t| format!(" text={:?}", t))
        .unwrap_or_default()
}

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

// ── Tree traversal ──────────────────────────────────────────────────

/// Emit a full subtree unconditionally (for filter_key matches).
fn emit_subtree(
    widgets: &WidgetRegistry,
    id: u64,
    depth: usize,
    visible_only: bool,
    screen_width: f32,
    screen_height: f32,
    lines: &mut Vec<String>,
) {
    let Some(frame) = widgets.get(id) else { return };
    if visible_only && !frame.visible {
        return;
    }
    let name = resolve_display_name(widgets, frame, id);
    emit_frame_line(frame, id, &name, depth, widgets, screen_width, screen_height, lines);
    for &child_id in &frame.children {
        emit_subtree(widgets, child_id, depth + 1, visible_only, screen_width, screen_height, lines);
    }
}

/// Emit frames matching a name filter.
#[allow(clippy::too_many_arguments)]
fn emit_filtered(
    widgets: &WidgetRegistry,
    id: u64,
    depth: usize,
    filter: Option<&str>,
    visible_only: bool,
    screen_width: f32,
    screen_height: f32,
    lines: &mut Vec<String>,
) {
    let Some(frame) = widgets.get(id) else { return };
    if visible_only && !frame.visible {
        return;
    }
    let name = resolve_display_name(widgets, frame, id);
    let matches = filter
        .map(|f| name.to_lowercase().contains(&f.to_lowercase()))
        .unwrap_or(true);
    if matches {
        emit_frame_line(frame, id, &name, depth, widgets, screen_width, screen_height, lines);
    }
    for &child_id in &frame.children {
        emit_filtered(widgets, child_id, depth + 1, filter, visible_only, screen_width, screen_height, lines);
    }
}

/// Emit a frame with warning flags (compact format for debug server).
fn emit_warning_recursive(
    widgets: &WidgetRegistry,
    id: u64,
    depth: usize,
    screen_width: f32,
    screen_height: f32,
    lines: &mut Vec<String>,
) {
    let Some(frame) = widgets.get(id) else { return };
    let rect = compute_frame_rect(widgets, id, screen_width, screen_height);
    let indent = "  ".repeat(depth);
    let name = frame.name.as_deref().unwrap_or("(anon)");
    let warnings = build_warnings(frame, &rect, screen_width, screen_height);
    let warn_str = if warnings.is_empty() {
        String::new()
    } else {
        format!(" ! {}", warnings.join(", "))
    };
    lines.push(format!(
        "{indent}{name} [{}] ({:.0},{:.0} {}x{}){warn_str}",
        frame.widget_type.as_str(),
        rect.x, rect.y, rect.width as i32, rect.height as i32,
    ));
    for &child_id in &frame.children {
        emit_warning_recursive(widgets, child_id, depth + 1, screen_width, screen_height, lines);
    }
}

fn build_warnings(frame: &Frame, rect: &LayoutRect, screen_width: f32, screen_height: f32) -> Vec<&'static str> {
    let mut w = Vec::new();
    if rect.width <= 0.0 { w.push("ZERO_WIDTH"); }
    if rect.height <= 0.0 { w.push("ZERO_HEIGHT"); }
    if rect.x + rect.width < 0.0 || rect.x > screen_width { w.push("OFFSCREEN_X"); }
    if rect.y + rect.height < 0.0 || rect.y > screen_height { w.push("OFFSCREEN_Y"); }
    if !frame.visible { w.push("HIDDEN"); }
    w
}

// ── Key-match filter ────────────────────────────────────────────────

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
        return;
    }
    for &child_id in &frame.children {
        collect_key_matches_recursive(widgets, child_id, key_lower, result);
    }
}

// ── Name / text resolution ──────────────────────────────────────────

fn collect_root_frames(widgets: &WidgetRegistry) -> Vec<(u64, Option<String>)> {
    widgets.iter_ids()
        .filter_map(|id| {
            let w = widgets.get(id)?;
            if w.parent_id.is_none() { Some((id, w.name.clone())) } else { None }
        })
        .collect()
}

/// Global name > parentKey > anonymous fallback.
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
                    return format!(".{key}");
                }
            }
        }
    // For anonymous frames with text, show a text preview
    if let Some(ref text) = frame.text {
        if text.len() > 20 {
            return format!("\"{}...\"", &text[..17]);
        }
        return format!("\"{text}\"");
    }
    frame.name.as_deref().unwrap_or("(anonymous)").to_string()
}

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

// ── Anchor diagnostic (headless only) ───────────────────────────────

fn print_anchor_diagnostic(widgets: &WidgetRegistry) {
    let mut anchored = 0;
    let mut unanchored = 0;
    let mut unanchored_keys: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for id in widgets.iter_ids() {
        let Some(w) = widgets.get(id) else { continue };
        if !w.anchors.is_empty() { anchored += 1; continue; }
        unanchored += 1;
        let parent_key = find_parent_key(widgets, w, id);
        let parent_name = w.parent_id
            .and_then(|pid| widgets.get(pid))
            .and_then(|p| p.name.clone())
            .unwrap_or_else(|| "(no parent)".into());
        let detail = format!("  {:?} on {} ({:?})", w.widget_type, parent_name, w.name);
        let key = parent_key.unwrap_or_else(|| "(no key)".into());
        unanchored_keys.entry(key).or_default().push(detail);
    }
    print_anchor_summary(&unanchored_keys, anchored, unanchored);
}

fn print_anchor_summary(
    keys: &std::collections::HashMap<String, Vec<String>>,
    anchored: usize,
    unanchored: usize,
) {
    let mut kv: Vec<_> = keys.iter().map(|(k, v)| (k.clone(), v.len())).collect();
    kv.sort_by(|a, b| b.1.cmp(&a.1));
    eprintln!("Anchored: {anchored}, Unanchored: {unanchored}");
    eprintln!("Top unanchored keys: {:?}", &kv[..kv.len().min(15)]);
    for (key, _) in kv.iter().take(5) {
        if let Some(details) = keys.get(key) {
            eprintln!("  {key}:");
            for d in details.iter().take(3) {
                eprintln!("  {d}");
            }
        }
    }
    if let Some(no_key) = keys.get("(no key)") {
        print_no_key_breakdown(no_key);
    }
}

fn print_no_key_breakdown(no_key: &[String]) {
    let mut by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for d in no_key {
        let wtype = d.trim().split(' ').next().unwrap_or("?");
        *by_type.entry(wtype.to_string()).or_default() += 1;
    }
    let mut tv: Vec<_> = by_type.iter().collect();
    tv.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("  (no key) by type: {tv:?}");
}

fn find_parent_key(widgets: &WidgetRegistry, w: &Frame, id: u64) -> Option<String> {
    let pid = w.parent_id?;
    let p = widgets.get(pid)?;
    p.children_keys.iter()
        .find(|(_, cid)| **cid == id)
        .map(|(k, _)| k.clone())
}

// ── WoW escape stripping ───────────────────────────────────────────

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

fn skip_wow_escape(chars: &mut std::iter::Peekable<std::str::Chars>) {
    match chars.peek() {
        Some('H') => {
            chars.next();
            while let Some(ch) = chars.next() {
                if ch == '|' && chars.peek() == Some(&'h') { chars.next(); break; }
            }
        }
        Some('h') => { chars.next(); }
        Some('T') => {
            chars.next();
            while let Some(&ch) = chars.peek() {
                chars.next();
                if ch == '|' { chars.next(); break; }
            }
        }
        Some('t') => { chars.next(); }
        Some('c') => { chars.next(); for _ in 0..8 { chars.next(); } }
        Some('r') => { chars.next(); }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{Anchor, AnchorPoint, WidgetType};

    fn make_frame(id: u64, parent: Option<u64>, w: f32, h: f32) -> Frame {
        let mut f = Frame::default();
        f.id = id;
        f.parent_id = parent;
        f.width = w;
        f.height = h;
        f
    }

    fn anchor(point: AnchorPoint, rel_id: Option<usize>, rel_point: AnchorPoint) -> Anchor {
        Anchor { point, relative_to_id: rel_id, relative_to: None, relative_point: rel_point, x_offset: 0.0, y_offset: 0.0 }
    }

    fn build_basic_registry() -> WidgetRegistry {
        let mut reg = WidgetRegistry::new();
        let mut uip = make_frame(1, None, 1024.0, 768.0);
        uip.name = Some("UIParent".to_string());
        uip.children = vec![10, 11];
        reg.register(uip);

        let mut btn = make_frame(10, Some(1), 200.0, 36.0);
        btn.name = Some("MyButton".to_string());
        btn.visible = true;
        btn.anchors = vec![anchor(AnchorPoint::Center, None, AnchorPoint::Center)];
        btn.children = vec![20];
        btn.children_keys.insert("Icon".to_string(), 20);
        reg.register(btn);

        let mut tex = make_frame(20, Some(10), 32.0, 32.0);
        tex.widget_type = WidgetType::Texture;
        tex.name = Some("__tex_123".to_string());
        tex.visible = true;
        tex.texture = Some("Interface/Icons/foo".to_string());
        tex.anchors = vec![anchor(AnchorPoint::Center, None, AnchorPoint::Center)];
        reg.register(tex);

        let mut hidden = make_frame(11, Some(1), 100.0, 50.0);
        hidden.name = Some("HiddenFrame".to_string());
        hidden.visible = false;
        reg.register(hidden);

        reg
    }

    // ── strip_wow_escapes ───────────────────────────────────────

    #[test]
    fn test_strip_plain_text() {
        assert_eq!(strip_wow_escapes("Hello World"), "Hello World");
    }

    #[test]
    fn test_strip_color_codes() {
        assert_eq!(strip_wow_escapes("|cff00ff00Green|r Text"), "Green Text");
    }

    #[test]
    fn test_strip_texture_escape() {
        assert_eq!(strip_wow_escapes("Before |TInterface/Icons/foo:16|t After"), "Before  After");
    }

    #[test]
    fn test_strip_hyperlink() {
        assert_eq!(strip_wow_escapes("|Hitem:12345|h[Sword]|h"), "[Sword]");
    }

    #[test]
    fn test_strip_nested_escapes() {
        assert_eq!(
            strip_wow_escapes("|cffff0000|Hspell:1234|hFireball|h|r"),
            "Fireball"
        );
    }

    // ── resolve_display_name ────────────────────────────────────

    #[test]
    fn test_display_name_global() {
        let reg = build_basic_registry();
        let frame = reg.get(10).unwrap();
        assert_eq!(resolve_display_name(&reg, frame, 10), "MyButton");
    }

    #[test]
    fn test_display_name_parent_key() {
        let reg = build_basic_registry();
        let frame = reg.get(20).unwrap();
        // __tex_ prefix is filtered, falls through to parentKey
        assert_eq!(resolve_display_name(&reg, frame, 20), ".Icon");
    }

    #[test]
    fn test_display_name_text_preview() {
        let mut reg = WidgetRegistry::new();
        let mut f = make_frame(99, None, 10.0, 10.0);
        f.name = Some("__anon_42".to_string());
        f.text = Some("Short".to_string());
        reg.register(f);
        let frame = reg.get(99).unwrap();
        assert_eq!(resolve_display_name(&reg, frame, 99), "\"Short\"");
    }

    #[test]
    fn test_display_name_text_truncated() {
        let mut reg = WidgetRegistry::new();
        let mut f = make_frame(99, None, 10.0, 10.0);
        f.name = Some("__anon_42".to_string());
        f.text = Some("This is a very long text string".to_string());
        reg.register(f);
        let frame = reg.get(99).unwrap();
        assert_eq!(resolve_display_name(&reg, frame, 99), "\"This is a very lo...\"");
    }

    // ── format_size_str ─────────────────────────────────────────

    #[test]
    fn test_size_str_matching() {
        let f = make_frame(1, None, 200.0, 36.0);
        let rect = LayoutRect { x: 0.0, y: 0.0, width: 200.0, height: 36.0 };
        assert_eq!(format_size_str(&f, &rect), "(200x36)");
    }

    #[test]
    fn test_size_str_differs() {
        let f = make_frame(1, None, 100.0, 50.0);
        let rect = LayoutRect { x: 0.0, y: 0.0, width: 200.0, height: 36.0 };
        assert_eq!(format_size_str(&f, &rect), "(200x36) [stored=100x50]");
    }

    #[test]
    fn test_size_str_anchor_derived() {
        // stored=0x0 but computed from anchors — no [stored=] annotation
        let f = make_frame(1, None, 0.0, 0.0);
        let rect = LayoutRect { x: 0.0, y: 0.0, width: 136.0, height: 39.0 };
        assert_eq!(format_size_str(&f, &rect), "(136x39)");
    }

    // ── format_stale_str ────────────────────────────────────────

    #[test]
    fn test_stale_none() {
        let f = make_frame(1, None, 0.0, 0.0);
        let rect = LayoutRect { x: 0.0, y: 0.0, width: 100.0, height: 50.0 };
        assert_eq!(format_stale_str(&f, &rect), " [layout_rect=None]");
    }

    #[test]
    fn test_stale_matching() {
        let mut f = make_frame(1, None, 0.0, 0.0);
        let rect = LayoutRect { x: 10.0, y: 20.0, width: 100.0, height: 50.0 };
        f.layout_rect = Some(rect);
        assert_eq!(format_stale_str(&f, &rect), "");
    }

    #[test]
    fn test_stale_diverged() {
        let mut f = make_frame(1, None, 0.0, 0.0);
        f.layout_rect = Some(LayoutRect { x: 5.0, y: 20.0, width: 100.0, height: 50.0 });
        let rect = LayoutRect { x: 10.0, y: 20.0, width: 100.0, height: 50.0 };
        assert!(format_stale_str(&f, &rect).contains("layout_rect="));
    }

    // ── build_warnings ──────────────────────────────────────────

    #[test]
    fn test_warnings_zero_size() {
        let f = make_frame(1, None, 0.0, 0.0);
        let rect = LayoutRect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
        let w = build_warnings(&f, &rect, 1024.0, 768.0);
        assert!(w.contains(&"ZERO_WIDTH"));
        assert!(w.contains(&"ZERO_HEIGHT"));
    }

    #[test]
    fn test_warnings_offscreen() {
        let mut f = make_frame(1, None, 0.0, 0.0);
        f.visible = true;
        let rect = LayoutRect { x: 2000.0, y: -100.0, width: 50.0, height: 50.0 };
        let w = build_warnings(&f, &rect, 1024.0, 768.0);
        assert!(w.contains(&"OFFSCREEN_X"));
    }

    #[test]
    fn test_warnings_hidden() {
        let mut f = make_frame(1, None, 0.0, 0.0);
        f.visible = false;
        let rect = LayoutRect { x: 0.0, y: 0.0, width: 100.0, height: 50.0 };
        let w = build_warnings(&f, &rect, 1024.0, 768.0);
        assert!(w.contains(&"HIDDEN"));
    }

    #[test]
    fn test_warnings_normal_visible() {
        let mut f = make_frame(1, None, 100.0, 50.0);
        f.visible = true;
        let rect = LayoutRect { x: 10.0, y: 10.0, width: 100.0, height: 50.0 };
        let w = build_warnings(&f, &rect, 1024.0, 768.0);
        assert!(w.is_empty());
    }

    // ── build_tree integration ──────────────────────────────────

    #[test]
    fn test_build_tree_includes_children() {
        let reg = build_basic_registry();
        let lines = build_tree(&reg, None, None, false, 1024.0, 768.0);
        let has_button = lines.iter().any(|l| l.contains("MyButton"));
        let has_icon = lines.iter().any(|l| l.contains(".Icon"));
        assert!(has_button, "Should contain MyButton");
        assert!(has_icon, "Should contain .Icon (parentKey)");
    }

    #[test]
    fn test_build_tree_filter() {
        let reg = build_basic_registry();
        let lines = build_tree(&reg, Some("MyButton"), None, false, 1024.0, 768.0);
        assert!(lines.iter().any(|l| l.contains("MyButton")));
        assert!(!lines.iter().any(|l| l.contains("HiddenFrame")));
    }

    #[test]
    fn test_build_tree_visible_only() {
        let reg = build_basic_registry();
        let lines = build_tree(&reg, None, None, true, 1024.0, 768.0);
        assert!(!lines.iter().any(|l| l.contains("HiddenFrame")));
    }

    #[test]
    fn test_build_tree_shows_texture_path() {
        let reg = build_basic_registry();
        let lines = build_tree(&reg, None, None, false, 1024.0, 768.0);
        assert!(lines.iter().any(|l| l.contains("[texture] Interface/Icons/foo")));
    }

    #[test]
    fn test_build_tree_shows_anchor_lines() {
        let reg = build_basic_registry();
        let lines = build_tree(&reg, None, None, false, 1024.0, 768.0);
        assert!(lines.iter().any(|l| l.contains("[anchor]")));
    }

    #[test]
    fn test_build_warning_dump_includes_header() {
        let reg = build_basic_registry();
        let lines = build_warning_dump(&reg, 1024.0, 768.0);
        assert!(lines[0].contains("Frame Dump"));
        assert!(lines[1].contains("1024x768"));
    }
}
