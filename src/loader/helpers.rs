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

/// Generate a simple random ID for anonymous frames.
pub fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    nanos
}

/// Escape a string for use in Lua code.
pub fn escape_lua_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Generate Lua code for setting anchors.
pub fn generate_anchors_code(anchors: &crate::xml::AnchorsXml, parent_ref: &str) -> String {
    let mut code = String::new();
    for anchor in &anchors.anchors {
        let point = &anchor.point;
        let relative_to = anchor.relative_to.as_deref();
        let relative_key = anchor.relative_key.as_deref();
        let relative_point = anchor.relative_point.as_deref().unwrap_or(point.as_str());

        // Get offset from either direct attributes or nested Offset element
        let (x, y) = if let Some(offset) = &anchor.offset {
            if let Some(abs) = &offset.abs_dimension {
                (abs.x.unwrap_or(0.0), abs.y.unwrap_or(0.0))
            } else {
                (0.0, 0.0)
            }
        } else {
            (anchor.x.unwrap_or(0.0), anchor.y.unwrap_or(0.0))
        };

        // Handle relativeKey (e.g., "$parent.Performance", "$parent.$parent.ScrollFrame") first, then relativeTo
        let rel_str = if let Some(key) = relative_key {
            // relativeKey format: "$parent", "$parent.ChildKey", "$parent.$parent.ChildKey", etc.
            if key.contains("$parent") || key.contains("$Parent") {
                // Split on dots and build expression
                let parts: Vec<&str> = key.split('.').collect();
                let mut expr = String::new();
                for part in parts {
                    if part == "$parent" || part == "$Parent" {
                        if expr.is_empty() {
                            expr = parent_ref.to_string();
                        } else {
                            expr = format!("{}:GetParent()", expr);
                        }
                    } else if !part.is_empty() {
                        // Access this as a property/child key
                        expr = format!("{}[\"{}\"]", expr, part);
                    }
                }
                if expr.is_empty() {
                    parent_ref.to_string()
                } else {
                    expr
                }
            } else {
                // No $parent pattern - use as global name
                key.to_string()
            }
        } else {
            match relative_to {
                Some("$parent") => parent_ref.to_string(),
                Some(rel) if rel.contains("$parent") || rel.contains("$Parent") => {
                    // Replace any $parent/$Parent pattern (e.g., $parent_Sibling, $parentColorSwatch)
                    rel.replace("$parent", parent_ref).replace("$Parent", parent_ref)
                }
                Some(rel) => rel.to_string(),
                None => "nil".to_string(),
            }
        };

        code.push_str(&format!(
            r#"
        frame:SetPoint("{}", {}, "{}", {}, {})
        "#,
            point, rel_str, relative_point, x, y
        ));
    }
    code
}

/// Generate Lua code for setting script handlers.
pub fn generate_scripts_code(scripts: &crate::xml::ScriptsXml) -> String {
    let mut code = String::new();

    // Helper to generate code for a single script handler
    let add_handler = |code: &mut String, handler_name: &str, script: &crate::xml::ScriptBodyXml| {
        if let Some(func) = &script.function {
            // Reference to a global function
            code.push_str(&format!(
                r#"
        frame:SetScript("{}", {})
        "#,
                handler_name, func
            ));
        } else if let Some(method) = &script.method {
            // Call a method on the frame
            code.push_str(&format!(
                r#"
        frame:SetScript("{}", function(self, ...) self:{}(...) end)
        "#,
                handler_name, method
            ));
        } else if let Some(body) = &script.body {
            let body = body.trim();
            if !body.is_empty() {
                // Inline script body
                code.push_str(&format!(
                    r#"
        frame:SetScript("{}", function(self, ...)
            {}
        end)
        "#,
                    handler_name, body
                ));
            }
        }
    };

    // Process scripts - use last handler if multiple are specified (WoW behavior)
    if let Some(on_load) = scripts.on_load.last() {
        add_handler(&mut code, "OnLoad", on_load);
    }
    if let Some(on_event) = scripts.on_event.last() {
        add_handler(&mut code, "OnEvent", on_event);
    }
    if let Some(on_update) = scripts.on_update.last() {
        add_handler(&mut code, "OnUpdate", on_update);
    }
    if let Some(on_click) = scripts.on_click.last() {
        add_handler(&mut code, "OnClick", on_click);
    }
    if let Some(on_show) = scripts.on_show.last() {
        add_handler(&mut code, "OnShow", on_show);
    }
    if let Some(on_hide) = scripts.on_hide.last() {
        add_handler(&mut code, "OnHide", on_hide);
    }

    code
}
