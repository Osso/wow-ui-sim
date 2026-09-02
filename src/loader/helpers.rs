//! Helper functions for path resolution and code generation.

use std::path::{Path, PathBuf};

pub use crate::paths::find_case_insensitive;

/// Normalize Windows-style paths (backslashes) to Unix-style (forward slashes).
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
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

    if let Some(interface_path) = resolve_interface_addons_path(addon_root, &normalized) {
        return interface_path;
    }

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

fn resolve_interface_addons_path(addon_root: &Path, normalized: &str) -> Option<PathBuf> {
    let suffix = normalized.split("Interface/AddOns/").nth(1)?;
    let addons_root = find_addons_root(addon_root)?;
    let candidate = addons_root.join(suffix);

    if candidate.exists() {
        return Some(candidate);
    }

    resolve_path_case_insensitive(&addons_root, suffix)
}

fn find_addons_root(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|ancestor| ancestor.file_name().and_then(|name| name.to_str()) == Some("AddOns"))
        .map(Path::to_path_buf)
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
pub fn resolve_lua_escapes(s: &str) -> std::borrow::Cow<'_, str> {
    // Fast path: no backslashes means no escapes to resolve
    if !s.contains('\\') {
        return std::borrow::Cow::Borrowed(s);
    }
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i = apply_lua_escape(bytes, i, &mut result);
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    std::borrow::Cow::Owned(result)
}

/// Decode one Lua escape sequence starting at `i` (which must be `\\`).
/// Returns the index after the consumed escape.
fn apply_lua_escape(bytes: &[u8], i: usize, result: &mut String) -> usize {
    let escape = bytes[i + 1];
    if let Some(ch) = simple_lua_escape_char(escape) {
        result.push(ch);
        return i + 2;
    }

    if escape.is_ascii_digit() {
        return decode_decimal_escape(bytes, i, result);
    }

    result.push('\\');
    i + 1
}

fn simple_lua_escape_char(byte: u8) -> Option<char> {
    match byte {
        b'a' => Some('\x07'),
        b'b' => Some('\x08'),
        b'f' => Some('\x0C'),
        b'n' => Some('\n'),
        b'r' => Some('\r'),
        b't' => Some('\t'),
        b'v' => Some('\x0B'),
        b'\\' => Some('\\'),
        b'"' => Some('"'),
        b'\'' => Some('\''),
        _ => None,
    }
}

/// Decode a decimal escape like `\32` or `\255`. Returns index after consumed bytes.
fn decode_decimal_escape(bytes: &[u8], i: usize, result: &mut String) -> usize {
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
    j
}

/// Escape a string for use in Lua code.
pub fn escape_lua_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn is_lua_identifier(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

/// Return a Lua field access expression for a string key.
///
/// Uses dot syntax for identifier-safe keys and bracket syntax otherwise.
pub fn lua_table_field_ref(table_expr: &str, key: &str) -> String {
    if is_lua_identifier(key) {
        format!("{table_expr}.{key}")
    } else {
        format!(r#"{table_expr}["{}"]"#, escape_lua_string(key))
    }
}

/// Return a Lua expression that references a frame by its global name.
///
/// Uses `_G["name"]` to safely handle frame names containing characters
/// that aren't valid in Lua identifiers (e.g., `$TankMarkerCheckButton`).
pub fn lua_frame_ref_by_id(id: u64) -> String {
    format!("debug.getregistry().__rilua_frame_refs[{id}]")
}

pub fn lua_global_ref(name: &str) -> String {
    if let Some(id) = name
        .strip_prefix("__frame_")
        .and_then(|suffix| suffix.parse::<u64>().ok())
    {
        return lua_frame_ref_by_id(id);
    }
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
            (offset.x.unwrap_or(0.0), offset.y.unwrap_or(0.0))
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
    let has_parent_ref =
        key.contains("$parent") || key.contains("$Parent") || key.contains("$parentKey");
    if !has_parent_ref {
        return key.to_string();
    }

    let mut expr = String::new();
    for segment in key.split('.') {
        resolve_segment(segment, parent_expr, &mut expr);
    }

    if expr.is_empty() {
        parent_expr.to_string()
    } else {
        expr
    }
}

/// Resolve a single dot-separated segment of a relativeKey expression.
///
/// `$parent` / `$Parent` / `$parentKey` → navigate to parent.
/// `$parentFoo` → navigate to parent, then index `["Foo"]`.
/// Anything else → index `["segment"]`.
fn resolve_segment(segment: &str, parent_expr: &str, expr: &mut String) {
    let parent_suffix = strip_parent_prefix(segment);

    if let Some(suffix) = parent_suffix {
        // Navigate to parent: first $parent uses parent_expr, subsequent use :GetParent()
        if expr.is_empty() {
            expr.push_str(parent_expr);
        } else {
            *expr = format!("{expr}:GetParent()");
        }
        if !suffix.is_empty() {
            *expr = format!("{expr}[\"{suffix}\"]");
        }
    } else if !segment.is_empty() {
        *expr = format!("{expr}[\"{segment}\"]");
    }
}

/// Strip a `$parent`/`$Parent`/`$parentKey` prefix from a segment, returning
/// the remaining suffix (empty string for exact matches like `$parent`).
/// Returns `None` if the segment doesn't start with a parent marker.
fn strip_parent_prefix(segment: &str) -> Option<&str> {
    if segment == "$parentKey" {
        return Some("");
    }
    segment
        .strip_prefix("$parent")
        .or_else(|| segment.strip_prefix("$Parent"))
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
            Some(r) if r.contains("$parent") || r.contains("$Parent") => lua_global_ref(
                &r.replace("$parent", parent_name)
                    .replace("$Parent", parent_name),
            ),
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
        let point = anchor.point.as_deref().unwrap_or("TOPLEFT");
        let relative_point = anchor.relative_point.as_deref().unwrap_or(point);
        let (x, y) = resolve_anchor_offset(anchor);
        let rel = resolve_anchor_relative(anchor, parent_expr, parent_name, default_relative);
        // relativeKey chains can reference frames that don't exist yet at load
        if anchor.relative_key.is_some() {
            let key = anchor.relative_key.as_deref().unwrap_or_default();
            code.push_str(&format!(
                r#"
        {}:SetPoint("{}", "{}", "{}", {}, {})
        "#,
                target_var,
                point,
                escape_lua_string(key),
                relative_point,
                x,
                y
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

fn append_script_handler_with_options(
    code: &mut String,
    target: &str,
    handler_name: &str,
    script: &crate::xml::ScriptBodyXml,
    default_binding: Option<u8>,
) {
    let script_binding =
        intrinsic_binding_index(script.intrinsic_order.as_deref()).or(default_binding);
    if script_clears_handler(script) {
        if let Some(binding) = script_binding {
            emit_set_script_binding(code, target, handler_name, binding, "nil");
        } else {
            code.push_str(&format!(
                "\n        {target}:SetScript(\"{handler_name}\", nil)\n        "
            ));
        }
        return;
    }

    let Some(new_handler) = build_handler_expr(target, handler_name, script) else {
        code.push_str(&format!(
            "\n        {target}:SetScript(\"{handler_name}\", nil)\n        "
        ));
        return;
    };

    if let Some(binding) = script_binding {
        emit_set_script_binding(code, target, handler_name, binding, &new_handler);
        return;
    }

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

fn intrinsic_binding_index(order: Option<&str>) -> Option<u8> {
    match order {
        Some("precall") => Some(0),
        Some("postcall") => Some(2),
        _ => None,
    }
}

fn emit_set_script_binding(
    code: &mut String,
    target: &str,
    handler_name: &str,
    binding: u8,
    handler_expr: &str,
) {
    let helper = crate::lua_api::globals::loader_script_bindings::SET_SCRIPT_BINDING_GLOBAL;
    code.push_str(&format!(
        "\n        {helper}({target}, \"{handler_name}\", {binding}, {handler_expr})\n        "
    ));
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
    let order = chained_handler_order(prepend);
    code.push_str(&chained_handler_lua(
        target,
        handler_name,
        new_handler,
        order,
    ));
}

fn chained_handler_order(prepend: bool) -> (&'static str, &'static str) {
    if prepend {
        ("__new", "__old")
    } else {
        ("__old", "__new")
    }
}

fn chained_handler_lua(
    target: &str,
    handler_name: &str,
    new_handler: &str,
    (first, second): (&str, &str),
) -> String {
    format!(
        r#"
        do
            local __old = {target}:GetScript("{handler_name}")
            local __new = {new_handler}
            local __report = debug.getregistry()["__report_script_error"]
            if __old then
                {target}:SetScript("{handler_name}", function(self, ...)
                    if securecall then
                        securecall({first}, self, ...)
                        securecall({second}, self, ...)
                    else
                        local __ok1, __err1 = pcall({first}, self, ...)
                        local __ok2, __err2 = pcall({second}, self, ...)
                        if not __ok1 then
                            local name = self.GetName and self:GetName() or "?"
                            __report("[script:{handler_name}] " .. name .. ": " .. tostring(__err1))
                        end
                        if not __ok2 then
                            local name = self.GetName and self:GetName() or "?"
                            __report("[script:{handler_name}] " .. name .. ": " .. tostring(__err2))
                        end
                    end
                end)
            else
                {target}:SetScript("{handler_name}", __new)
            end
        end
        "#
    )
}

fn script_clears_handler(script: &crate::xml::ScriptBodyXml) -> bool {
    if script.method.is_some() {
        return false;
    }
    if let Some(function_name) = script.function.as_deref() {
        return function_name.trim().is_empty();
    }
    script
        .body
        .as_deref()
        .map(|body| body.trim().is_empty())
        .unwrap_or(true)
}

/// WoW implicit parameter names for inline XML script bodies.
fn handler_params(handler_name: &str) -> &'static str {
    match handler_name {
        "OnUpdate" => "self, elapsed",
        "OnEvent" => "self, event, ...",
        "OnClick" | "OnDoubleClick" => "self, button, down",
        "OnEnter" | "OnLeave" => "self, motion",
        "OnMouseDown" | "OnMouseUp" => "self, button",
        "OnValueChanged" => "self, value",
        "OnTextChanged" => "self, userInput",
        "OnChar" => "self, text",
        _ => "self, ...",
    }
}

/// Build the Lua expression for a script handler (without setting it).
fn build_handler_expr(
    target: &str,
    handler_name: &str,
    script: &crate::xml::ScriptBodyXml,
) -> Option<String> {
    if let Some(func) = &script.function {
        if func.is_empty() {
            return None;
        }
        Some(func.clone())
    } else if let Some(method) = &script.method {
        Some(format!(
            "__wow_bind_xml_method({target}, \"{}\")",
            escape_lua_string(method)
        ))
    } else {
        let body = script.body.as_deref()?.trim();
        if body.is_empty() {
            return None;
        }
        let params = handler_params(handler_name);
        Some(format!(
            "function({params})\n            {body}\n        end"
        ))
    }
}

/// Apply a list of (handler_name, optional_script) pairs to a target.
pub fn apply_script_handlers(
    target: &str,
    handlers: &[(&'static str, Option<&crate::xml::ScriptBodyXml>)],
) -> String {
    apply_script_handlers_with_options(target, handlers, None)
}

fn apply_script_handlers_with_options(
    target: &str,
    handlers: &[(&'static str, Option<&crate::xml::ScriptBodyXml>)],
    default_binding: Option<u8>,
) -> String {
    let mut code = String::new();
    for (name, script) in handlers {
        if let Some(s) = script {
            append_script_handler_with_options(&mut code, target, name, s, default_binding);
        }
    }
    code
}

/// Generate Lua code for setting script handlers.
pub fn generate_scripts_code(scripts: &crate::xml::ScriptsXml) -> String {
    generate_scripts_code_for_target("frame", scripts)
}

pub fn generate_intrinsic_scripts_code(scripts: &crate::xml::ScriptsXml) -> String {
    generate_scripts_code_for_target_with_options("frame", scripts, Some(0))
}

/// Generate Lua code for setting script handlers on a named Lua variable.
pub fn generate_scripts_code_for_target(target: &str, scripts: &crate::xml::ScriptsXml) -> String {
    generate_scripts_code_for_target_with_options(target, scripts, None)
}

fn generate_scripts_code_for_target_with_options(
    target: &str,
    scripts: &crate::xml::ScriptsXml,
    default_binding: Option<u8>,
) -> String {
    let mut code = lifecycle_handlers_with_options(target, scripts, default_binding);
    code.push_str(&input_handlers_with_options(
        target,
        scripts,
        default_binding,
    ));
    code
}

fn lifecycle_handlers_with_options(
    target: &str,
    scripts: &crate::xml::ScriptsXml,
    default_binding: Option<u8>,
) -> String {
    apply_script_handlers_with_options(
        target,
        &[
            ("OnLoad", scripts.on_load.last()),
            ("OnEvent", scripts.on_event.last()),
            ("OnUpdate", scripts.on_update.last()),
            ("OnClick", scripts.on_click.last()),
            ("OnDoubleClick", scripts.on_double_click.last()),
            ("PreClick", scripts.pre_click.last()),
            ("PostClick", scripts.post_click.last()),
            ("OnShow", scripts.on_show.last()),
            ("OnHide", scripts.on_hide.last()),
            ("OnEnter", scripts.on_enter.last()),
            ("OnLeave", scripts.on_leave.last()),
            ("OnMouseDown", scripts.on_mouse_down.last()),
            ("OnMouseUp", scripts.on_mouse_up.last()),
            ("OnMouseWheel", scripts.on_mouse_wheel.last()),
            ("OnDragStart", scripts.on_drag_start.last()),
            ("OnDragStop", scripts.on_drag_stop.last()),
            ("OnReceiveDrag", scripts.on_receive_drag.last()),
        ],
        default_binding,
    )
}

fn input_handlers_with_options(
    target: &str,
    scripts: &crate::xml::ScriptsXml,
    default_binding: Option<u8>,
) -> String {
    apply_script_handlers_with_options(
        target,
        &[
            ("OnEnterPressed", scripts.on_enter_pressed.last()),
            ("OnEscapePressed", scripts.on_escape_pressed.last()),
            ("OnTabPressed", scripts.on_tab_pressed.last()),
            ("OnSpacePressed", scripts.on_space_pressed.last()),
            ("OnArrowPressed", scripts.on_arrow_pressed.last()),
            ("OnTextChanged", scripts.on_text_changed.last()),
            ("OnTextSet", scripts.on_text_set.last()),
            ("OnChar", scripts.on_char.last()),
            ("OnEditFocusGained", scripts.on_edit_focus_gained.last()),
            ("OnEditFocusLost", scripts.on_edit_focus_lost.last()),
            (
                "OnInputLanguageChanged",
                scripts.on_input_language_changed.last(),
            ),
            ("OnKeyDown", scripts.on_key_down.last()),
            ("OnKeyUp", scripts.on_key_up.last()),
            ("OnValueChanged", scripts.on_value_changed.last()),
            ("OnEnable", scripts.on_enable.last()),
            ("OnDisable", scripts.on_disable.last()),
            ("OnSizeChanged", scripts.on_size_changed.last()),
            ("OnAttributeChanged", scripts.on_attribute_changed.last()),
            ("OnHyperlinkClick", scripts.on_hyperlink_click.last()),
            ("OnHyperlinkEnter", scripts.on_hyperlink_enter.last()),
            ("OnHyperlinkLeave", scripts.on_hyperlink_leave.last()),
        ],
        default_binding,
    )
}

#[cfg(test)]
#[path = "helpers_tests.rs"]
mod helpers_tests;
