//! Helper functions for path resolution and code generation.

use std::path::{Path, PathBuf};

/// Normalize Windows-style paths (backslashes) to Unix-style (forward slashes).
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Find a directory entry case-insensitively.
pub fn find_case_insensitive(dir: &Path, name: &str) -> Option<PathBuf> {
    let name_lower = name.to_lowercase();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().to_lowercase() == name_lower {
                return Some(entry.path());
            }
        }
    }
    None
}

/// Resolve a path with case-insensitive matching (WoW is case-insensitive on Windows/macOS).
pub fn resolve_path_case_insensitive(base: &Path, path: &str) -> Option<PathBuf> {
    let components: Vec<&str> = path.split('/').collect();
    let mut current = base.to_path_buf();

    for component in &components {
        if component.is_empty() {
            continue;
        }
        // Try exact match first
        let exact = current.join(component);
        if exact.exists() {
            current = exact;
        } else if let Some(entry) = find_case_insensitive(&current, component) {
            current = entry;
        } else {
            return None;
        }
    }
    if current.exists() {
        Some(current)
    } else {
        None
    }
}

/// Resolve a path relative to xml_dir, with fallback to addon_root.
/// Some addons use paths relative to addon root instead of the XML file location.
/// Uses case-insensitive matching for compatibility with WoW (Windows/macOS).
pub fn resolve_path_with_fallback(xml_dir: &Path, addon_root: &Path, file: &str) -> PathBuf {
    let normalized = normalize_path(file);

    // Try case-sensitive first (faster)
    let primary = xml_dir.join(&normalized);
    if primary.exists() {
        return primary;
    }

    // Try case-insensitive in xml_dir
    if let Some(resolved) = resolve_path_case_insensitive(xml_dir, &normalized) {
        return resolved;
    }

    // Try case-sensitive fallback to addon root
    let fallback = addon_root.join(&normalized);
    if fallback.exists() {
        return fallback;
    }

    // Try case-insensitive in addon_root
    if let Some(resolved) = resolve_path_case_insensitive(addon_root, &normalized) {
        return resolved;
    }

    // Return primary path (will result in error with correct path)
    primary
}

/// Get size values from a SizeXml, checking both direct attributes and AbsDimension.
pub fn get_size_values(size: &crate::xml::SizeXml) -> (Option<f32>, Option<f32>) {
    if size.x.is_some() || size.y.is_some() {
        (size.x, size.y)
    } else if let Some(abs) = &size.abs_dimension {
        (abs.x, abs.y)
    } else {
        (None, None)
    }
}

/// Generate a unique ID for anonymous frames using an atomic counter.
pub fn rand_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Resolve Lua string escape sequences stored as literal text.
///
/// Global strings from WoW CSV contain Lua escape sequences like `\32` (space)
/// that are stored as literal backslash + digits in our Rust data. This function
/// interprets them the same way Lua would when parsing a string literal.
pub fn resolve_lua_escapes(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'a' => { result.push('\x07'); i += 2; }
                b'b' => { result.push('\x08'); i += 2; }
                b'f' => { result.push('\x0C'); i += 2; }
                b'n' => { result.push('\n'); i += 2; }
                b'r' => { result.push('\r'); i += 2; }
                b't' => { result.push('\t'); i += 2; }
                b'v' => { result.push('\x0B'); i += 2; }
                b'\\' => { result.push('\\'); i += 2; }
                b'"' => { result.push('"'); i += 2; }
                b'\'' => { result.push('\''); i += 2; }
                d if d.is_ascii_digit() => {
                    let mut val: u32 = 0;
                    let mut j = i + 1;
                    let end = (i + 4).min(bytes.len());
                    while j < end && bytes[j].is_ascii_digit() {
                        val = val * 10 + (bytes[j] - b'0') as u32;
                        j += 1;
                    }
                    if val <= 255 {
                        result.push(val as u8 as char);
                    }
                    i = j;
                }
                _ => {
                    result.push('\\');
                    i += 1;
                }
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

/// Escape a string for use in Lua code.
pub fn escape_lua_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Return a Lua expression that references a frame by its global name.
///
/// Uses `_G["name"]` to safely handle frame names containing characters
/// that aren't valid in Lua identifiers (e.g., `$TankMarkerCheckButton`).
pub fn lua_global_ref(name: &str) -> String {
    format!("_G[\"{}\"]", escape_lua_string(name))
}

/// Resolve a child widget name, replacing $parent with parent name.
/// Returns the resolved name, or generates a random one with the given prefix.
pub fn resolve_child_name(name: Option<&str>, parent_name: &str, prefix: &str) -> String {
    name.map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("{}{}", prefix, rand_id()))
}

/// Get the x/y offset from an anchor element.
pub fn resolve_anchor_offset(anchor: &crate::xml::AnchorXml) -> (f32, f32) {
    if let Some(offset) = &anchor.offset {
        if let Some(abs) = &offset.abs_dimension {
            (abs.x.unwrap_or(0.0), abs.y.unwrap_or(0.0))
        } else {
            (0.0, 0.0)
        }
    } else {
        (anchor.x.unwrap_or(0.0), anchor.y.unwrap_or(0.0))
    }
}

/// Resolve a relativeKey expression like "$parent.$parent.ScrollFrame" into a Lua expression.
///
/// Handles `$parent` both as a complete segment (`$parent.Foo`) and as a prefix
/// (`$parentPanelContainer`), matching WoW's substitution behavior.
fn resolve_relative_key(key: &str, parent_expr: &str) -> String {
    if !key.contains("$parent") && !key.contains("$Parent") && !key.contains("$parentKey") {
        return key.to_string();
    }
    let mut expr = String::new();
    for part in key.split('.') {
        if part == "$parent" || part == "$Parent" || part == "$parentKey" {
            if expr.is_empty() {
                expr = parent_expr.to_string();
            } else {
                expr = format!("{}:GetParent()", expr);
            }
        } else if let Some(suffix) = part.strip_prefix("$parent").or_else(|| part.strip_prefix("$Parent")) {
            // Handle $parent as a prefix: "$parentPanelContainer" → parent["PanelContainer"]
            if expr.is_empty() {
                expr = parent_expr.to_string();
            } else {
                expr = format!("{}:GetParent()", expr);
            }
            if !suffix.is_empty() {
                expr = format!("{}[\"{}\"]", expr, suffix);
            }
        } else if !part.is_empty() {
            expr = format!("{}[\"{}\"]", expr, part);
        }
    }
    if expr.is_empty() { parent_expr.to_string() } else { expr }
}

/// Resolve the relative target for an anchor.
///
/// - `parent_expr`: Lua expression for $parent in relativeKey (e.g. `"parent"` or a frame name)
/// - `parent_name`: actual parent name for $parent substitution in relativeTo strings
/// - `default_relative`: value when no relativeTo is specified (e.g. `"nil"` or `"parent"`)
pub fn resolve_anchor_relative(
    anchor: &crate::xml::AnchorXml,
    parent_expr: &str,
    parent_name: &str,
    default_relative: &str,
) -> String {
    if let Some(key) = anchor.relative_key.as_deref() {
        resolve_relative_key(key, parent_expr)
    } else {
        match anchor.relative_to.as_deref() {
            Some("$parent") => parent_expr.to_string(),
            Some(r) if r.contains("$parent") || r.contains("$Parent") => {
                lua_global_ref(&r.replace("$parent", parent_name).replace("$Parent", parent_name))
            }
            Some(r) => lua_global_ref(r),
            None => default_relative.to_string(),
        }
    }
}

/// Generate Lua SetPoint calls for a list of anchors.
///
/// - `target_var`: the Lua variable to call SetPoint on (e.g. `"frame"`, `"fs"`, `"tex"`)
/// - `parent_expr`: Lua expression for $parent in relativeKey
/// - `parent_name`: actual parent name for $parent replacement in relativeTo
/// - `default_relative`: value when no relativeTo is specified
pub fn generate_set_point_code(
    anchors: &crate::xml::AnchorsXml,
    target_var: &str,
    parent_expr: &str,
    parent_name: &str,
    default_relative: &str,
) -> String {
    let mut code = String::new();
    for anchor in &anchors.anchors {
        let point = &anchor.point;
        let relative_point = anchor.relative_point.as_deref().unwrap_or(point.as_str());
        let (x, y) = resolve_anchor_offset(anchor);
        let rel = resolve_anchor_relative(anchor, parent_expr, parent_name, default_relative);
        // relativeKey chains can reference frames that don't exist yet at load
        // time (they get reparented later). Wrap in pcall to match WoW behavior
        // where unresolvable anchors are silently skipped.
        if anchor.relative_key.is_some() {
            code.push_str(&format!(
                r#"
        pcall(function() {}:SetPoint("{}", {}, "{}", {}, {}) end)
        "#,
                target_var, point, rel, relative_point, x, y
            ));
        } else {
            code.push_str(&format!(
                r#"
        {}:SetPoint("{}", {}, "{}", {}, {})
        "#,
                target_var, point, rel, relative_point, x, y
            ));
        }
    }
    code
}

/// Append a single SetScript call for a script handler to the code buffer.
pub fn append_script_handler(
    code: &mut String,
    target: &str,
    handler_name: &str,
    script: &crate::xml::ScriptBodyXml,
) {
    let Some(new_handler) = build_handler_expr(handler_name, script) else { return };

    match script.inherit.as_deref() {
        Some("prepend") => emit_chained_handler(code, target, handler_name, &new_handler, false),
        Some("append") => emit_chained_handler(code, target, handler_name, &new_handler, true),
        _ => {
            code.push_str(&format!(
                "\n        {target}:SetScript(\"{handler_name}\", {new_handler})\n        "
            ));
        }
    }
}

/// Emit a chained handler that wraps the existing handler (new_first=true → new runs first).
/// WoW semantics: "prepend"/"append" describe the INHERITED handler's position:
///   inherit="prepend" → inherited (old) runs first, instance (new) second → new_first=false
///   inherit="append"  → instance (new) runs first, inherited (old) second → new_first=true
fn emit_chained_handler(
    code: &mut String,
    target: &str,
    handler_name: &str,
    new_handler: &str,
    prepend: bool,
) {
    let (first, second) = if prepend { ("__new", "__old") } else { ("__old", "__new") };
    code.push_str(&format!(
        r#"
        do
            local __old = {target}:GetScript("{handler_name}")
            local __new = {new_handler}
            if __old then
                {target}:SetScript("{handler_name}", function(self, ...)
                    local __ok1, __err1 = pcall({first}, self, ...)
                    local __ok2, __err2 = pcall({second}, self, ...)
                    if not __ok1 then
                        local name = self.GetName and self:GetName() or "?"
                        __report_script_error("[script:{handler_name}] " .. name .. ": " .. tostring(__err1))
                    end
                    if not __ok2 then
                        local name = self.GetName and self:GetName() or "?"
                        __report_script_error("[script:{handler_name}] " .. name .. ": " .. tostring(__err2))
                    end
                end)
            else
                {target}:SetScript("{handler_name}", __new)
            end
        end
        "#
    ));
}

/// WoW implicit parameter names for inline XML script bodies.
fn handler_params(handler_name: &str) -> &'static str {
    match handler_name {
        "OnUpdate" => "self, elapsed",
        "OnEvent" => "self, event, ...",
        "OnClick" => "self, button, down",
        "OnEnter" | "OnLeave" => "self, motion",
        "OnMouseDown" | "OnMouseUp" => "self, button",
        "OnValueChanged" => "self, value",
        "OnTextChanged" => "self, userInput",
        "OnChar" => "self, text",
        _ => "self, ...",
    }
}

/// Build the Lua expression for a script handler (without setting it).
fn build_handler_expr(handler_name: &str, script: &crate::xml::ScriptBodyXml) -> Option<String> {
    if let Some(func) = &script.function {
        if func.is_empty() { return None; }
        Some(func.clone())
    } else if let Some(method) = &script.method {
        Some(format!("function(self, ...) self:{method}(...) end"))
    } else {
        let body = script.body.as_deref()?.trim();
        if body.is_empty() { return None; }
        let params = handler_params(handler_name);
        Some(format!("function({params})\n            {body}\n        end"))
    }
}

/// Apply a list of (handler_name, optional_script) pairs to a target.
pub fn apply_script_handlers(
    target: &str,
    handlers: &[(&str, Option<&crate::xml::ScriptBodyXml>)],
) -> String {
    let mut code = String::new();
    for (name, script) in handlers {
        if let Some(s) = script {
            append_script_handler(&mut code, target, name, s);
        }
    }
    code
}

/// Generate Lua code for setting script handlers.
pub fn generate_scripts_code(scripts: &crate::xml::ScriptsXml) -> String {
    apply_script_handlers("frame", &[
        ("OnLoad", scripts.on_load.last()),
        ("OnEvent", scripts.on_event.last()),
        ("OnUpdate", scripts.on_update.last()),
        ("OnClick", scripts.on_click.last()),
        ("OnShow", scripts.on_show.last()),
        ("OnHide", scripts.on_hide.last()),
        // Mouse
        ("OnEnter", scripts.on_enter.last()),
        ("OnLeave", scripts.on_leave.last()),
        ("OnMouseDown", scripts.on_mouse_down.last()),
        ("OnMouseUp", scripts.on_mouse_up.last()),
        ("OnMouseWheel", scripts.on_mouse_wheel.last()),
        ("OnDragStart", scripts.on_drag_start.last()),
        ("OnDragStop", scripts.on_drag_stop.last()),
        ("OnReceiveDrag", scripts.on_receive_drag.last()),
        // EditBox
        ("OnEnterPressed", scripts.on_enter_pressed.last()),
        ("OnEscapePressed", scripts.on_escape_pressed.last()),
        ("OnTabPressed", scripts.on_tab_pressed.last()),
        ("OnSpacePressed", scripts.on_space_pressed.last()),
        ("OnTextChanged", scripts.on_text_changed.last()),
        ("OnTextSet", scripts.on_text_set.last()),
        ("OnChar", scripts.on_char.last()),
        ("OnEditFocusGained", scripts.on_edit_focus_gained.last()),
        ("OnEditFocusLost", scripts.on_edit_focus_lost.last()),
        ("OnInputLanguageChanged", scripts.on_input_language_changed.last()),
        // Keyboard
        ("OnKeyDown", scripts.on_key_down.last()),
        ("OnKeyUp", scripts.on_key_up.last()),
        // Other
        ("OnValueChanged", scripts.on_value_changed.last()),
        ("OnEnable", scripts.on_enable.last()),
        ("OnDisable", scripts.on_disable.last()),
        ("OnSizeChanged", scripts.on_size_changed.last()),
        ("OnAttributeChanged", scripts.on_attribute_changed.last()),
        ("OnHyperlinkClick", scripts.on_hyperlink_click.last()),
        ("OnHyperlinkEnter", scripts.on_hyperlink_enter.last()),
        ("OnHyperlinkLeave", scripts.on_hyperlink_leave.last()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_backslashes() {
        assert_eq!(normalize_path("Interface\\Buttons\\UI-Button"), "Interface/Buttons/UI-Button");
    }

    #[test]
    fn test_normalize_path_already_forward() {
        assert_eq!(normalize_path("Interface/Buttons/UI-Button"), "Interface/Buttons/UI-Button");
    }

    #[test]
    fn test_normalize_path_empty() {
        assert_eq!(normalize_path(""), "");
    }

    #[test]
    fn test_resolve_lua_escapes_decimal() {
        // \32 = space (ASCII 32)
        assert_eq!(resolve_lua_escapes(r":\32"), ": ");
        assert_eq!(resolve_lua_escapes(r"Say:\32"), "Say: ");
    }

    #[test]
    fn test_resolve_lua_escapes_named() {
        assert_eq!(resolve_lua_escapes(r"\n"), "\n");
        assert_eq!(resolve_lua_escapes(r"\t"), "\t");
        assert_eq!(resolve_lua_escapes(r"\\"), "\\");
    }

    #[test]
    fn test_resolve_lua_escapes_no_escapes() {
        assert_eq!(resolve_lua_escapes("hello"), "hello");
        assert_eq!(resolve_lua_escapes(""), "");
    }

    #[test]
    fn test_resolve_lua_escapes_combined() {
        // \37 = '%', \32 = space
        assert_eq!(resolve_lua_escapes(r"%s\32"), "%s ");
    }

    #[test]
    fn test_escape_lua_string_backslash() {
        assert_eq!(escape_lua_string("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_escape_lua_string_quotes() {
        assert_eq!(escape_lua_string(r#"say "hello""#), r#"say \"hello\""#);
    }

    #[test]
    fn test_escape_lua_string_newlines() {
        assert_eq!(escape_lua_string("line1\nline2\rline3"), "line1\\nline2\\rline3");
    }

    #[test]
    fn test_escape_lua_string_combined() {
        assert_eq!(escape_lua_string("a\\b\n\"c\""), "a\\\\b\\n\\\"c\\\"");
    }

    #[test]
    fn test_resolve_child_name_with_parent() {
        let name = resolve_child_name(Some("$parentTitle"), "MyFrame", "anon_");
        assert_eq!(name, "MyFrameTitle");
    }

    #[test]
    fn test_resolve_child_name_no_parent_placeholder() {
        let name = resolve_child_name(Some("ExplicitName"), "MyFrame", "anon_");
        assert_eq!(name, "ExplicitName");
    }

    #[test]
    fn test_resolve_child_name_none_generates_prefix() {
        let name = resolve_child_name(None, "MyFrame", "anon_");
        assert!(name.starts_with("anon_"), "Should start with prefix, got: {}", name);
    }

    #[test]
    fn test_resolve_relative_key_simple_name() {
        let result = resolve_relative_key("ScrollFrame", "parent");
        assert_eq!(result, "ScrollFrame");
    }

    #[test]
    fn test_resolve_relative_key_parent() {
        let result = resolve_relative_key("$parent", "parent");
        assert_eq!(result, "parent");
    }

    #[test]
    fn test_resolve_relative_key_parent_child() {
        let result = resolve_relative_key("$parent.ScrollFrame", "parent");
        assert_eq!(result, r#"parent["ScrollFrame"]"#);
    }

    #[test]
    fn test_resolve_relative_key_double_parent() {
        let result = resolve_relative_key("$parent.$parent.ScrollFrame", "parent");
        assert_eq!(result, r#"parent:GetParent()["ScrollFrame"]"#);
    }

    #[test]
    fn test_get_size_values_direct() {
        let size = crate::xml::SizeXml {
            x: Some(100.0),
            y: Some(200.0),
            abs_dimension: None,
        };
        assert_eq!(get_size_values(&size), (Some(100.0), Some(200.0)));
    }

    #[test]
    fn test_get_size_values_abs_dimension() {
        let size = crate::xml::SizeXml {
            x: None,
            y: None,
            abs_dimension: Some(crate::xml::AbsDimensionXml {
                x: Some(50.0),
                y: Some(75.0),
            }),
        };
        assert_eq!(get_size_values(&size), (Some(50.0), Some(75.0)));
    }

    #[test]
    fn test_get_size_values_empty() {
        let size = crate::xml::SizeXml {
            x: None,
            y: None,
            abs_dimension: None,
        };
        assert_eq!(get_size_values(&size), (None, None));
    }
}
