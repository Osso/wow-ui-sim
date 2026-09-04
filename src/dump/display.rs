use crate::widget::{Frame, WidgetRegistry, WidgetType};

/// Resolve owner addon index to folder name.
pub(crate) fn resolve_addon_name(addon_names: &[String], owner: Option<u16>) -> Option<&str> {
    owner.and_then(|idx| addon_names.get(idx as usize).map(|s| s.as_str()))
}

/// Prefer semantic child keys when the runtime name is a lowercase or
/// synthetic fallback.
pub(crate) fn resolve_display_name(widgets: &WidgetRegistry, frame: &Frame, id: u64) -> String {
    if let Some(parent_key) = frame.parent_key.as_deref()
        && parent_key_matches_current_parent(widgets, frame, id, parent_key)
        && should_prefer_parent_key(frame.name.as_deref(), parent_key)
    {
        return format!(".{parent_key}");
    }
    if let Some(parent_id) = frame.parent_id
        && let Some(parent) = widgets.get(parent_id)
    {
        for (key, &child_id) in &parent.children_keys {
            if child_id == id && should_prefer_parent_key(frame.name.as_deref(), key) {
                return format!(".{key}");
            }
        }
    }
    if let Some(ref name) = frame.name
        && !name.starts_with("__")
    {
        return name.clone();
    }
    if let Some(ref text) = frame.text {
        if text.chars().count() > 20 {
            return format!("\"{}...\"", first_chars(text, 17));
        }
        return format!("\"{text}\"");
    }
    frame.name.as_deref().unwrap_or("(anonymous)").to_string()
}

fn first_chars(value: &str, count: usize) -> String {
    value.chars().take(count).collect()
}

fn parent_key_matches_current_parent(
    widgets: &WidgetRegistry,
    frame: &Frame,
    id: u64,
    parent_key: &str,
) -> bool {
    frame
        .parent_id
        .and_then(|parent_id| widgets.get(parent_id))
        .and_then(|parent| parent.children_keys.get(parent_key).copied())
        == Some(id)
}

fn should_prefer_parent_key(name: Option<&str>, parent_key: &str) -> bool {
    let Some(name) = name else {
        return true;
    };
    if name.starts_with("__") {
        return true;
    }
    if name.eq_ignore_ascii_case(parent_key) && name != parent_key {
        return true;
    }
    name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && parent_key.chars().any(|c| c.is_ascii_uppercase())
}

pub(crate) fn resolve_display_text(widgets: &WidgetRegistry, frame: &Frame) -> Option<String> {
    if let Some(ref t) = frame.text
        && !t.is_empty()
    {
        return Some(strip_wow_escapes(t));
    }
    for key in &["Title", "TitleText"] {
        if let Some(&child_id) = frame.children_keys.get(*key)
            && let Some(child) = widgets.get(child_id)
            && let Some(ref t) = child.text
            && !t.is_empty()
        {
            return Some(strip_wow_escapes(t));
        }
    }
    None
}

pub(crate) fn print_anchor_diagnostic(widgets: &WidgetRegistry) {
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
    kv.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
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
    if let Some(parent_key) = w.parent_key.as_deref()
        && parent_key_matches_current_parent(widgets, w, id, parent_key)
    {
        return Some(parent_key.to_string());
    }
    let pid = w.parent_id?;
    let p = widgets.get(pid)?;
    p.children_keys
        .iter()
        .find(|(_, cid)| **cid == id)
        .map(|(k, _)| k.clone())
}

/// Strip WoW escape sequences (|T...|t texture, |c...|r color) for cleaner display.
pub fn strip_wow_escapes(s: &str) -> String {
    let mut result = String::new();
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
                if ch == '|' && chars.peek() == Some(&'h') {
                    chars.next();
                    break;
                }
            }
        }
        Some('h') => {
            chars.next();
        }
        Some('T') => {
            chars.next();
            while let Some(&ch) = chars.peek() {
                chars.next();
                if ch == '|' {
                    chars.next();
                    break;
                }
            }
        }
        Some('t') => {
            chars.next();
        }
        Some('c') => {
            chars.next();
            if chars.peek() == Some(&'n') {
                chars.next();
                // A named code ends at its colon. Stop at a `|` too: a color
                // name cannot contain one, so a code missing its colon costs
                // the code rather than the rest of the string.
                while let Some(&ch) = chars.peek() {
                    if ch == '|' {
                        break;
                    }
                    chars.next();
                    if ch == ':' {
                        break;
                    }
                }
            } else {
                for _ in 0..8 {
                    chars.next();
                }
            }
        }
        Some('r') => {
            chars.next();
        }
        _ => {}
    }
}

/// For Texture children with parentKey like NormalTexture/PushedTexture/etc.,
/// look up the texture path from the parent button's corresponding field.
pub(crate) fn resolve_button_state_texture<'a>(
    widgets: &'a WidgetRegistry,
    frame: &Frame,
    id: u64,
) -> Option<&'a str> {
    if frame.widget_type != WidgetType::Texture {
        return None;
    }
    let parent = widgets.get(frame.parent_id?)?;
    let key = parent
        .children_keys
        .iter()
        .find(|&(_, cid)| *cid == id)
        .map(|(k, _)| k.as_str())?;
    match key {
        "NormalTexture" => parent.normal_texture.as_deref(),
        "PushedTexture" => parent.pushed_texture.as_deref(),
        "HighlightTexture" => parent.highlight_texture.as_deref(),
        "DisabledTexture" => parent.disabled_texture.as_deref(),
        _ => None,
    }
}

/// Resolve a WoW texture path and return a suffix indicating the format found.
/// Returns e.g. " (webp)", " (BLP)", or " (MISSING)".
pub(crate) fn resolve_texture_format(wow_path: &str) -> String {
    use crate::texture::{TextureManager, normalize_wow_path};
    use std::sync::OnceLock;

    if !dump_texture_formats_enabled() {
        return String::new();
    }

    static TEX_MGR: OnceLock<TextureManager> = OnceLock::new();
    let mgr = TEX_MGR.get_or_init(|| {
        TextureManager::new().with_addons_paths(crate::paths::default_addons_paths())
    });

    let normalized = normalize_wow_path(wow_path);
    match mgr.resolve_path(&normalized) {
        Some(p) => {
            let ext = p
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            format!(" ({ext})")
        }
        None => " (MISSING)".to_string(),
    }
}

fn dump_texture_formats_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("WOW_SIM_DUMP_TEXTURE_FORMATS").is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{Frame, WidgetRegistry};

    #[test]
    fn display_name_truncates_utf8_text_on_char_boundary() {
        let mut widgets = WidgetRegistry::new();
        let mut frame = Frame::default();
        frame.text = Some("|cffff3333明天启程 |rwith a long suffix".to_string());
        let id = frame.id;
        widgets.register(frame);

        let display_name = resolve_display_name(&widgets, widgets.get(id).unwrap(), id);

        assert_eq!(display_name, "\"|cffff3333明天启程 |r...\"");
    }

    #[test]
    fn strip_wow_escapes_handles_named_color_before_hyperlink() {
        assert_eq!(
            super::strip_wow_escapes(
                "Use |cnIQ3:|Hitem:202046::::::::80:70:::::::::|h[Lucky Tortollan Charm]|h|r now"
            ),
            "Use [Lucky Tortollan Charm] now"
        );
    }

    #[test]
    fn strip_wow_escapes_named_color_without_colon_stops_at_the_next_escape() {
        assert_eq!(super::strip_wow_escapes("|cnBroken|r tail"), "tail");
    }
}
