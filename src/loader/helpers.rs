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
fn resolve_relative_key(key: &str, parent_expr: &str) -> String {
    if !key.contains("$parent") && !key.contains("$Parent") {
        return key.to_string();
    }
    let mut expr = String::new();
    for part in key.split('.') {
        if part == "$parent" || part == "$Parent" {
            if expr.is_empty() {
                expr = parent_expr.to_string();
            } else {
                expr = format!("{}:GetParent()", expr);
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
                r.replace("$parent", parent_name).replace("$Parent", parent_name)
            }
            Some(r) => r.to_string(),
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
        code.push_str(&format!(
            r#"
        {}:SetPoint("{}", {}, "{}", {}, {})
        "#,
            target_var, point, rel, relative_point, x, y
        ));
    }
    code
}

/// Generate Lua code for setting anchors on a frame.
pub fn generate_anchors_code(anchors: &crate::xml::AnchorsXml, parent_ref: &str) -> String {
    generate_set_point_code(anchors, "frame", parent_ref, parent_ref, "nil")
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

/// Format an optional string as a Lua string literal or "nil".
fn lua_opt_str(s: Option<&str>) -> String {
    match s.filter(|s| !s.is_empty()) {
        Some(s) => format!("\"{}\"", escape_lua_string(s)),
        None => "nil".to_string(),
    }
}

/// Append a Lua method call with a single numeric argument if the value is Some.
fn emit_num_call(code: &mut String, target: &str, method: &str, val: Option<f32>) {
    if let Some(v) = val {
        code.push_str(&format!("\n        {target}:{method}({v})\n        "));
    }
}

/// Append a Lua method call with a single string argument if the value is Some.
fn emit_str_call(code: &mut String, target: &str, method: &str, val: Option<&str>) {
    if let Some(v) = val {
        code.push_str(&format!(
            "\n        {target}:{method}(\"{}\")\n        ",
            escape_lua_string(v)
        ));
    }
}

/// Append a Lua method call with two numeric arguments if either value is Some.
fn emit_pair_call(
    code: &mut String,
    target: &str,
    method: &str,
    a: Option<f32>,
    b: Option<f32>,
    default: f32,
) {
    if a.is_some() || b.is_some() {
        code.push_str(&format!(
            "\n        {target}:{method}({}, {})\n        ",
            a.unwrap_or(default),
            b.unwrap_or(default)
        ));
    }
}

/// Generate Lua code for a single animation element within a group.
fn generate_animation_code(
    anim: &crate::xml::AnimationXml,
    anim_type: &str,
    _frame_ref: &str,
) -> String {
    let mut code = String::new();

    code.push_str(&format!(
        "\n        local __anim = __ag:CreateAnimation(\"{anim_type}\", {})\n        ",
        lua_opt_str(anim.name.as_deref()),
    ));

    if let Some(parent_key) = &anim.parent_key {
        code.push_str(&format!("\n        __ag.{parent_key} = __anim\n        "));
    }

    emit_num_call(&mut code, "__anim", "SetDuration", anim.duration);
    emit_num_call(&mut code, "__anim", "SetOrder", anim.order.map(|o| o as f32));
    emit_num_call(&mut code, "__anim", "SetStartDelay", anim.start_delay);
    emit_num_call(&mut code, "__anim", "SetEndDelay", anim.end_delay);
    emit_str_call(&mut code, "__anim", "SetSmoothing", anim.smoothing.as_deref());
    emit_num_call(&mut code, "__anim", "SetFromAlpha", anim.from_alpha);
    emit_num_call(&mut code, "__anim", "SetToAlpha", anim.to_alpha);
    emit_pair_call(&mut code, "__anim", "SetOffset", anim.offset_x, anim.offset_y, 0.0);
    emit_pair_call(&mut code, "__anim", "SetScale", anim.scale_x, anim.scale_y, 1.0);
    emit_pair_call(&mut code, "__anim", "SetScaleFrom", anim.from_scale_x, anim.from_scale_y, 1.0);
    emit_pair_call(&mut code, "__anim", "SetScaleTo", anim.to_scale_x, anim.to_scale_y, 1.0);
    emit_num_call(&mut code, "__anim", "SetDegrees", anim.degrees);
    emit_str_call(&mut code, "__anim", "SetChildKey", anim.child_key.as_deref());
    emit_str_call(&mut code, "__anim", "SetTargetName", anim.target.as_deref());
    emit_str_call(&mut code, "__anim", "SetTargetKey", anim.target_key.as_deref());

    code
}

/// Append a single SetScript call for a script handler to the code buffer.
pub fn append_script_handler(
    code: &mut String,
    target: &str,
    handler_name: &str,
    script: &crate::xml::ScriptBodyXml,
) {
    if let Some(func) = &script.function {
        if func.is_empty() {
            return; // Empty function name = no-op (used to override parent scripts)
        }
        code.push_str(&format!(
            "\n        {target}:SetScript(\"{handler_name}\", {func})\n        "
        ));
    } else if let Some(method) = &script.method {
        code.push_str(&format!(
            "\n        {target}:SetScript(\"{handler_name}\", function(self, ...) self:{method}(...) end)\n        "
        ));
    } else if let Some(body) = &script.body {
        let body = body.trim();
        if !body.is_empty() {
            code.push_str(&format!(
                "\n        {target}:SetScript(\"{handler_name}\", function(self, ...)\n            {body}\n        end)\n        "
            ));
        }
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

/// Generate Lua code for animation group script handlers (OnPlay, OnFinished, etc.).
fn generate_anim_group_scripts_code(
    scripts: &crate::xml::ScriptsXml,
    group_ref: &str,
) -> String {
    apply_script_handlers(group_ref, &[
        ("OnPlay", scripts.on_play.last()),
        ("OnFinished", scripts.on_finished.last()),
        ("OnStop", scripts.on_stop.last()),
        ("OnLoop", scripts.on_loop.last()),
        ("OnPause", scripts.on_pause.last()),
    ])
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
    ])
}
