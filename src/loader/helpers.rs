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
    
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
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

    code.push_str(&format!(
        "\n        do\n        local __ag = {frame_ref}:CreateAnimationGroup({}, {})\n        ",
        lua_opt_str(anim_group.name.as_deref()),
        lua_opt_str(anim_group.inherits.as_deref()),
    ));

    if let Some(parent_key) = &anim_group.parent_key {
        code.push_str(&format!("\n        {frame_ref}.{parent_key} = __ag\n        "));
    }
    emit_str_call(&mut code, "__ag", "SetLooping", anim_group.looping.as_deref());
    if anim_group.set_to_final_alpha == Some(true) {
        code.push_str("\n        __ag:SetToFinalAlpha(true)\n        ");
    }

    emit_anim_group_children(&mut code, anim_group, frame_ref);
    emit_anim_group_mixin(&mut code, anim_group);

    code.push_str("\n        end\n        ");
    code
}

/// Emit child elements (scripts, keyvalues, animations) for an animation group.
fn emit_anim_group_children(
    code: &mut String,
    anim_group: &crate::xml::AnimationGroupXml,
    frame_ref: &str,
) {
    for element in &anim_group.elements {
        match element {
            crate::xml::AnimationElement::Scripts(scripts) => {
                code.push_str(&generate_anim_group_scripts_code(scripts, "__ag"));
            }
            crate::xml::AnimationElement::KeyValues(kv) => {
                emit_anim_key_values(code, kv);
            }
            crate::xml::AnimationElement::Unknown => {}
            _ => {
                if let Some((type_str, xml)) = resolve_animation_element(element) {
                    code.push_str(&generate_animation_code(xml, type_str, frame_ref));
                }
            }
        }
    }
}

/// Emit KeyValue assignments for an animation group.
fn emit_anim_key_values(code: &mut String, kv: &crate::xml::KeyValuesXml) {
    for key_value in &kv.values {
        let value = match key_value.value_type.as_deref() {
            Some("number") => key_value.value.clone(),
            Some("boolean") => key_value.value.to_lowercase(),
            _ => format!("\"{}\"", escape_lua_string(&key_value.value)),
        };
        code.push_str(&format!("\n        __ag.{} = {}\n        ", key_value.key, value));
    }
}

/// Resolve an AnimationElement variant to its type string and XML data.
fn resolve_animation_element(
    element: &crate::xml::AnimationElement,
) -> Option<(&str, &crate::xml::AnimationXml)> {
    match element {
        crate::xml::AnimationElement::Alpha(a) => Some(("Alpha", a)),
        crate::xml::AnimationElement::Translation(a) => Some(("Translation", a)),
        crate::xml::AnimationElement::LineTranslation(a) => Some(("LineTranslation", a)),
        crate::xml::AnimationElement::Rotation(a) => Some(("Rotation", a)),
        crate::xml::AnimationElement::Scale(a) => Some(("Scale", a)),
        crate::xml::AnimationElement::LineScale(a) => Some(("LineScale", a)),
        crate::xml::AnimationElement::Path(a) => Some(("Path", a)),
        crate::xml::AnimationElement::FlipBook(a) => Some(("FlipBook", a)),
        crate::xml::AnimationElement::VertexColor(a) => Some(("VertexColor", a)),
        crate::xml::AnimationElement::TextureCoordTranslation(a) => {
            Some(("TextureCoordTranslation", a))
        }
        crate::xml::AnimationElement::Animation(a) => Some(("Animation", a)),
        _ => None,
    }
}

/// Emit Mixin() calls for an animation group.
fn emit_anim_group_mixin(code: &mut String, anim_group: &crate::xml::AnimationGroupXml) {
    if let Some(mixin) = &anim_group.mixin {
        for m in mixin.split(',').map(|s| s.trim()) {
            if !m.is_empty() {
                code.push_str(&format!("\n        if {m} then Mixin(__ag, {m}) end\n        "));
            }
        }
    }
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
    if let Some(order) = anim.order {
        code.push_str(&format!("\n        __anim:SetOrder({order})\n        "));
    }
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
    let Some(new_handler) = build_handler_expr(script) else { return };

    match script.inherit.as_deref() {
        Some("prepend") => emit_chained_handler(code, target, handler_name, &new_handler, true),
        Some("append") => emit_chained_handler(code, target, handler_name, &new_handler, false),
        _ => {
            code.push_str(&format!(
                "\n        {target}:SetScript(\"{handler_name}\", {new_handler})\n        "
            ));
        }
    }
}

/// Emit a chained handler that wraps the existing handler (prepend=new first, else old first).
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
                        print("[script:{handler_name}] " .. name .. ": " .. tostring(__err1))
                    end
                    if not __ok2 then
                        local name = self.GetName and self:GetName() or "?"
                        print("[script:{handler_name}] " .. name .. ": " .. tostring(__err2))
                    end
                end)
            else
                {target}:SetScript("{handler_name}", __new)
            end
        end
        "#
    ));
}

/// Build the Lua expression for a script handler (without setting it).
fn build_handler_expr(script: &crate::xml::ScriptBodyXml) -> Option<String> {
    if let Some(func) = &script.function {
        if func.is_empty() { return None; }
        Some(func.clone())
    } else if let Some(method) = &script.method {
        Some(format!("function(self, ...) self:{method}(...) end"))
    } else {
        let body = script.body.as_deref()?.trim();
        if body.is_empty() { return None; }
        Some(format!("function(self, ...)\n            {body}\n        end"))
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
    fn test_lua_opt_str_some() {
        assert_eq!(lua_opt_str(Some("hello")), r#""hello""#);
    }

    #[test]
    fn test_lua_opt_str_none() {
        assert_eq!(lua_opt_str(None), "nil");
    }

    #[test]
    fn test_lua_opt_str_empty() {
        assert_eq!(lua_opt_str(Some("")), "nil");
    }

    #[test]
    fn test_emit_num_call_some() {
        let mut code = String::new();
        emit_num_call(&mut code, "__anim", "SetDuration", Some(0.5));
        assert!(code.contains("__anim:SetDuration(0.5)"));
    }

    #[test]
    fn test_emit_num_call_none() {
        let mut code = String::new();
        emit_num_call(&mut code, "__anim", "SetDuration", None);
        assert!(code.is_empty());
    }

    #[test]
    fn test_emit_str_call_some() {
        let mut code = String::new();
        emit_str_call(&mut code, "__anim", "SetSmoothing", Some("IN_OUT"));
        assert!(code.contains(r#"__anim:SetSmoothing("IN_OUT")"#));
    }

    #[test]
    fn test_emit_pair_call_both() {
        let mut code = String::new();
        emit_pair_call(&mut code, "__anim", "SetOffset", Some(10.0), Some(20.0), 0.0);
        assert!(code.contains("__anim:SetOffset(10, 20)"));
    }

    #[test]
    fn test_emit_pair_call_one_only() {
        let mut code = String::new();
        emit_pair_call(&mut code, "__anim", "SetScale", Some(2.0), None, 1.0);
        assert!(code.contains("__anim:SetScale(2, 1)"));
    }

    #[test]
    fn test_emit_pair_call_none() {
        let mut code = String::new();
        emit_pair_call(&mut code, "__anim", "SetScale", None, None, 1.0);
        assert!(code.is_empty());
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
