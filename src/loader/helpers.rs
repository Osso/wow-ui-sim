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

/// Generate Lua code for creating an animation group and its child animations from XML.
pub fn generate_animation_group_code(
    anim_group: &crate::xml::AnimationGroupXml,
    frame_ref: &str,
) -> String {
    let mut code = String::new();

    let group_name = anim_group.name.as_deref().unwrap_or("");
    let inherits = anim_group.inherits.as_deref().unwrap_or("");

    code.push_str(&format!(
        r#"
        do
        local __ag = {}:CreateAnimationGroup({}, {})
        "#,
        frame_ref,
        if group_name.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", escape_lua_string(group_name))
        },
        if inherits.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", escape_lua_string(inherits))
        },
    ));

    // Set parentKey
    if let Some(parent_key) = &anim_group.parent_key {
        code.push_str(&format!(
            r#"
        {}.{} = __ag
        "#,
            frame_ref, parent_key
        ));
    }

    // Set looping
    if let Some(looping) = &anim_group.looping {
        code.push_str(&format!(
            r#"
        __ag:SetLooping("{}")
        "#,
            escape_lua_string(looping)
        ));
    }

    // Set setToFinalAlpha
    if anim_group.set_to_final_alpha == Some(true) {
        code.push_str(
            r#"
        __ag:SetToFinalAlpha(true)
        "#,
        );
    }

    // Process child elements
    for element in &anim_group.elements {
        match element {
            crate::xml::AnimationElement::Scripts(scripts) => {
                code.push_str(&generate_anim_group_scripts_code(scripts, "__ag"));
            }
            crate::xml::AnimationElement::KeyValues(kv) => {
                for key_value in &kv.values {
                    let value = match key_value.value_type.as_deref() {
                        Some("number") => key_value.value.clone(),
                        Some("boolean") => key_value.value.to_lowercase(),
                        _ => format!("\"{}\"", escape_lua_string(&key_value.value)),
                    };
                    code.push_str(&format!(
                        r#"
        __ag.{} = {}
        "#,
                        key_value.key, value
                    ));
                }
            }
            crate::xml::AnimationElement::Unknown => {}
            _ => {
                // Animation elements (Alpha, Translation, etc.)
                let (anim_type_str, anim_xml) = match element {
                    crate::xml::AnimationElement::Alpha(a) => ("Alpha", a),
                    crate::xml::AnimationElement::Translation(a) => ("Translation", a),
                    crate::xml::AnimationElement::LineTranslation(a) => ("LineTranslation", a),
                    crate::xml::AnimationElement::Rotation(a) => ("Rotation", a),
                    crate::xml::AnimationElement::Scale(a) => ("Scale", a),
                    crate::xml::AnimationElement::LineScale(a) => ("LineScale", a),
                    crate::xml::AnimationElement::Path(a) => ("Path", a),
                    crate::xml::AnimationElement::FlipBook(a) => ("FlipBook", a),
                    crate::xml::AnimationElement::VertexColor(a) => ("VertexColor", a),
                    crate::xml::AnimationElement::TextureCoordTranslation(a) => {
                        ("TextureCoordTranslation", a)
                    }
                    crate::xml::AnimationElement::Animation(a) => ("Animation", a),
                    _ => continue,
                };
                code.push_str(&generate_animation_code(anim_xml, anim_type_str, frame_ref));
            }
        }
    }

    // Apply mixin
    if let Some(mixin) = &anim_group.mixin {
        for m in mixin.split(',').map(|s| s.trim()) {
            if !m.is_empty() {
                code.push_str(&format!(
                    r#"
        if {} then Mixin(__ag, {}) end
        "#,
                    m, m
                ));
            }
        }
    }

    code.push_str(
        r#"
        end
        "#,
    );

    code
}

/// Generate Lua code for a single animation element within a group.
fn generate_animation_code(
    anim: &crate::xml::AnimationXml,
    anim_type: &str,
    _frame_ref: &str,
) -> String {
    let mut code = String::new();

    let anim_name = anim.name.as_deref().unwrap_or("");

    code.push_str(&format!(
        r#"
        local __anim = __ag:CreateAnimation("{}", {})
        "#,
        anim_type,
        if anim_name.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", escape_lua_string(anim_name))
        },
    ));

    // Set parentKey
    if let Some(parent_key) = &anim.parent_key {
        code.push_str(&format!(
            r#"
        __ag.{} = __anim
        "#,
            parent_key
        ));
    }

    // Duration
    if let Some(dur) = anim.duration {
        code.push_str(&format!(
            r#"
        __anim:SetDuration({})
        "#,
            dur
        ));
    }

    // Order
    if let Some(order) = anim.order {
        code.push_str(&format!(
            r#"
        __anim:SetOrder({})
        "#,
            order
        ));
    }

    // Delays
    if let Some(delay) = anim.start_delay {
        code.push_str(&format!(
            r#"
        __anim:SetStartDelay({})
        "#,
            delay
        ));
    }
    if let Some(delay) = anim.end_delay {
        code.push_str(&format!(
            r#"
        __anim:SetEndDelay({})
        "#,
            delay
        ));
    }

    // Smoothing
    if let Some(smoothing) = &anim.smoothing {
        code.push_str(&format!(
            r#"
        __anim:SetSmoothing("{}")
        "#,
            escape_lua_string(smoothing)
        ));
    }

    // Alpha
    if let Some(val) = anim.from_alpha {
        code.push_str(&format!(
            r#"
        __anim:SetFromAlpha({})
        "#,
            val
        ));
    }
    if let Some(val) = anim.to_alpha {
        code.push_str(&format!(
            r#"
        __anim:SetToAlpha({})
        "#,
            val
        ));
    }

    // Translation
    if anim.offset_x.is_some() || anim.offset_y.is_some() {
        code.push_str(&format!(
            r#"
        __anim:SetOffset({}, {})
        "#,
            anim.offset_x.unwrap_or(0.0),
            anim.offset_y.unwrap_or(0.0)
        ));
    }

    // Scale
    if anim.scale_x.is_some() || anim.scale_y.is_some() {
        code.push_str(&format!(
            r#"
        __anim:SetScale({}, {})
        "#,
            anim.scale_x.unwrap_or(1.0),
            anim.scale_y.unwrap_or(1.0)
        ));
    }
    if anim.from_scale_x.is_some() || anim.from_scale_y.is_some() {
        code.push_str(&format!(
            r#"
        __anim:SetScaleFrom({}, {})
        "#,
            anim.from_scale_x.unwrap_or(1.0),
            anim.from_scale_y.unwrap_or(1.0)
        ));
    }
    if anim.to_scale_x.is_some() || anim.to_scale_y.is_some() {
        code.push_str(&format!(
            r#"
        __anim:SetScaleTo({}, {})
        "#,
            anim.to_scale_x.unwrap_or(1.0),
            anim.to_scale_y.unwrap_or(1.0)
        ));
    }

    // Rotation
    if let Some(deg) = anim.degrees {
        code.push_str(&format!(
            r#"
        __anim:SetDegrees({})
        "#,
            deg
        ));
    }

    // childKey / target / targetKey
    if let Some(child_key) = &anim.child_key {
        code.push_str(&format!(
            r#"
        __anim:SetChildKey("{}")
        "#,
            escape_lua_string(child_key)
        ));
    }
    if let Some(target) = &anim.target {
        code.push_str(&format!(
            r#"
        __anim:SetTargetName("{}")
        "#,
            escape_lua_string(target)
        ));
    }
    if let Some(target_key) = &anim.target_key {
        code.push_str(&format!(
            r#"
        __anim:SetTargetKey("{}")
        "#,
            escape_lua_string(target_key)
        ));
    }

    code
}

/// Generate Lua code for animation group script handlers (OnPlay, OnFinished, etc.).
fn generate_anim_group_scripts_code(
    scripts: &crate::xml::ScriptsXml,
    group_ref: &str,
) -> String {
    let mut code = String::new();

    let add_handler = |code: &mut String,
                       handler_name: &str,
                       script: &crate::xml::ScriptBodyXml,
                       target: &str| {
        if let Some(func) = &script.function {
            code.push_str(&format!(
                r#"
        {}:SetScript("{}", {})
        "#,
                target, handler_name, func
            ));
        } else if let Some(method) = &script.method {
            code.push_str(&format!(
                r#"
        {}:SetScript("{}", function(self, ...) self:{}(...) end)
        "#,
                target, handler_name, method
            ));
        } else if let Some(body) = &script.body {
            let body = body.trim();
            if !body.is_empty() {
                code.push_str(&format!(
                    r#"
        {}:SetScript("{}", function(self, ...)
            {}
        end)
        "#,
                    target, handler_name, body
                ));
            }
        }
    };

    if let Some(handler) = scripts.on_play.last() {
        add_handler(&mut code, "OnPlay", handler, group_ref);
    }
    if let Some(handler) = scripts.on_finished.last() {
        add_handler(&mut code, "OnFinished", handler, group_ref);
    }
    if let Some(handler) = scripts.on_stop.last() {
        add_handler(&mut code, "OnStop", handler, group_ref);
    }
    if let Some(handler) = scripts.on_loop.last() {
        add_handler(&mut code, "OnLoop", handler, group_ref);
    }
    if let Some(handler) = scripts.on_pause.last() {
        add_handler(&mut code, "OnPause", handler, group_ref);
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
