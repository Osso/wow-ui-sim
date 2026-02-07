//! Frame tree dump and diagnostic utilities.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::WidgetRegistry;

/// Load the UI and dump the frame tree (standalone, no server needed).
pub fn dump_standalone(
    filter: Option<String>,
    visible_only: bool,
    no_addons: bool,
    no_saved_vars: bool,
) {
    let (env, _font_system) = super::create_standalone_env(no_addons, no_saved_vars);

    // Load debug script if present
    if let Ok(script) = std::fs::read_to_string("/tmp/debug-scrollbox-update.lua") {
        let _ = env.exec(&script);
    }

    super::fire_startup_events(&env);

    // Hide frames that WoW's C++ engine hides by default (no target, no group, etc.).
    // Must run AFTER startup events since PLAYER_ENTERING_WORLD handlers may re-show frames.
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());

    print_addon_list(&env);

    let state = env.state().borrow();
    let widgets = &state.widgets;

    let mut roots = collect_root_frames(widgets);
    roots.sort_by(|a, b| {
        let name_a = a.1.as_deref().unwrap_or("");
        let name_b = b.1.as_deref().unwrap_or("");
        name_a.cmp(name_b)
    });

    let version_check = state.cvars.get_bool("checkAddonVersion");
    eprintln!(
        "Load out of date addons: {}",
        if version_check { "off" } else { "on" }
    );

    print_anchor_diagnostic(widgets);
    eprintln!("\n=== Frame Tree ===\n");

    for (id, _) in &roots {
        print_frame(widgets, *id, 0, &filter, visible_only);
    }
}

/// Print the addon list via Lua.
fn print_addon_list(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        local num = C_AddOns.GetNumAddOns()
        if num > 0 then
            print("\n=== Addons (" .. num .. ") ===\n")
            for i = 1, num do
                local name, title, notes, loadable, reason, security = C_AddOns.GetAddOnInfo(i)
                local loaded = C_AddOns.IsAddOnLoaded(i)
                local enabled = C_AddOns.GetAddOnEnableState(i) > 0
                local status = loaded and "loaded" or (enabled and "enabled" or "disabled")
                print(string.format("  [%d] %s (%s) [%s]", i, tostring(title), tostring(name), status))
            end
        end
        "#,
    );
}

/// Collect root frames (no parent).
fn collect_root_frames(widgets: &WidgetRegistry) -> Vec<(u64, Option<String>)> {
    widgets
        .all_ids()
        .iter()
        .filter_map(|&id| {
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
    for id in widgets.all_ids() {
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
fn find_parent_key(
    widgets: &WidgetRegistry,
    w: &wow_ui_sim::widget::Frame,
    id: u64,
) -> Option<String> {
    let pid = w.parent_id?;
    let p = widgets.get(pid)?;
    p.children_keys
        .iter()
        .find(|(_, cid)| **cid == id)
        .map(|(k, _)| k.clone())
}

/// Recursively print a frame and its children.
fn print_frame(
    widgets: &WidgetRegistry,
    id: u64,
    depth: usize,
    filter: &Option<String>,
    visible_only: bool,
) {
    let Some(frame) = widgets.get(id) else { return };

    if visible_only && !frame.visible {
        return;
    }

    let display_name = resolve_display_name(widgets, frame, id);
    let matches_filter = filter
        .as_ref()
        .map(|f| display_name.to_lowercase().contains(&f.to_lowercase()))
        .unwrap_or(true);

    if matches_filter {
        print_frame_line(frame, &display_name, depth, widgets);
    }

    for &child_id in &frame.children {
        print_frame(widgets, child_id, depth + 1, filter, visible_only);
    }
}

/// Format and print a single frame line.
fn print_frame_line(
    frame: &wow_ui_sim::widget::Frame,
    display_name: &str,
    depth: usize,
    widgets: &WidgetRegistry,
) {
    let indent = "  ".repeat(depth);
    let vis = if frame.visible { "visible" } else { "hidden" };
    let keys: Vec<_> = frame.children_keys.keys().collect();
    let keys_str = if keys.is_empty() {
        String::new()
    } else {
        format!(
            " keys=[{}]",
            keys.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        )
    };
    let text_str = resolve_display_text(widgets, frame)
        .map(|t| format!(" text={:?}", t))
        .unwrap_or_default();
    let font_str = if frame.widget_type == wow_ui_sim::widget::WidgetType::FontString {
        format!(
            " font={:?} size={}",
            frame.font.as_deref().unwrap_or("(none)"),
            frame.font_size
        )
    } else {
        String::new()
    };
    println!(
        "{}{} [{:?}] ({}x{}) {}{}{}{}",
        indent,
        display_name,
        frame.widget_type,
        frame.width as i32,
        frame.height as i32,
        vis,
        text_str,
        font_str,
        keys_str
    );
}

/// Resolve a display name for a frame: global name, parentKey, or "(anonymous)".
fn resolve_display_name(
    widgets: &WidgetRegistry,
    frame: &wow_ui_sim::widget::Frame,
    id: u64,
) -> String {
    if let Some(ref name) = frame.name {
        if !name.starts_with("__anon_")
            && !name.starts_with("__frame_")
            && !name.starts_with("__tex_")
            && !name.starts_with("__fs_")
        {
            return name.clone();
        }
    }

    if let Some(parent_id) = frame.parent_id {
        if let Some(parent) = widgets.get(parent_id) {
            for (key, &child_id) in &parent.children_keys {
                if child_id == id {
                    return format!(".{}", key);
                }
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
fn resolve_display_text(
    widgets: &WidgetRegistry,
    frame: &wow_ui_sim::widget::Frame,
) -> Option<String> {
    if let Some(ref t) = frame.text {
        if !t.is_empty() {
            return Some(strip_wow_escapes(t));
        }
    }

    for key in &["Title", "TitleText"] {
        if let Some(&child_id) = frame.children_keys.get(*key) {
            if let Some(child) = widgets.get(child_id) {
                if let Some(ref t) = child.text {
                    if !t.is_empty() {
                        return Some(strip_wow_escapes(t));
                    }
                }
            }
        }
    }

    None
}

/// Strip WoW escape sequences (|T...|t texture, |c...|r color) for cleaner display.
fn strip_wow_escapes(s: &str) -> String {
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
        Some('c') => {
            chars.next();
            for _ in 0..8 {
                chars.next();
            }
        }
        Some('r') => {
            chars.next();
        }
        _ => {}
    }
}
