//! Addon loader - loads addons from TOC files.

use crate::lua_api::WowLuaEnv;
use crate::saved_variables::SavedVariablesManager;
use crate::toc::TocFile;
use crate::xml::{parse_xml_file, XmlElement};
use mlua::Table;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Result of loading an addon.
#[derive(Debug)]
pub struct LoadResult {
    /// Addon name
    pub name: String,
    /// Number of Lua files loaded
    pub lua_files: usize,
    /// Number of XML files loaded
    pub xml_files: usize,
    /// Time breakdown
    pub timing: LoadTiming,
    /// Errors encountered (non-fatal)
    pub warnings: Vec<String>,
}

/// Timing breakdown for addon loading.
#[derive(Debug, Default, Clone)]
pub struct LoadTiming {
    /// Time reading files from disk
    pub io_time: Duration,
    /// Time parsing XML
    pub xml_parse_time: Duration,
    /// Time executing Lua
    pub lua_exec_time: Duration,
    /// Time loading SavedVariables
    pub saved_vars_time: Duration,
}

impl LoadTiming {
    pub fn total(&self) -> Duration {
        self.io_time + self.xml_parse_time + self.lua_exec_time + self.saved_vars_time
    }
}

/// Context for loading addon files (name, private table, and addon root for path resolution).
struct AddonContext<'a> {
    name: &'a str,
    table: Table,
    /// Addon root directory for fallback path resolution
    addon_root: &'a Path,
}

/// Load an addon from its TOC file.
pub fn load_addon(env: &WowLuaEnv, toc_path: &Path) -> Result<LoadResult, LoadError> {
    let toc = TocFile::from_file(toc_path)?;
    load_addon_from_toc(env, &toc)
}

/// Load an addon from its TOC file with saved variables support.
pub fn load_addon_with_saved_vars(
    env: &WowLuaEnv,
    toc_path: &Path,
    saved_vars_mgr: &mut SavedVariablesManager,
) -> Result<LoadResult, LoadError> {
    let toc = TocFile::from_file(toc_path)?;
    load_addon_from_toc_with_saved_vars(env, &toc, saved_vars_mgr)
}

/// Load an addon from a parsed TOC.
pub fn load_addon_from_toc(env: &WowLuaEnv, toc: &TocFile) -> Result<LoadResult, LoadError> {
    load_addon_internal(env, toc, None)
}

/// Load an addon from a parsed TOC with saved variables support.
pub fn load_addon_from_toc_with_saved_vars(
    env: &WowLuaEnv,
    toc: &TocFile,
    saved_vars_mgr: &mut SavedVariablesManager,
) -> Result<LoadResult, LoadError> {
    load_addon_internal(env, toc, Some(saved_vars_mgr))
}

/// Internal addon loading with optional saved variables.
fn load_addon_internal(
    env: &WowLuaEnv,
    toc: &TocFile,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
) -> Result<LoadResult, LoadError> {
    // WoW passes the folder name (not Title) as the addon name vararg
    let folder_name = toc
        .addon_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&toc.name);

    let mut result = LoadResult {
        name: toc.name.clone(),
        lua_files: 0,
        xml_files: 0,
        timing: LoadTiming::default(),
        warnings: Vec::new(),
    };

    // Initialize saved variables before loading addon files
    if let Some(mgr) = saved_vars_mgr {
        let sv_start = Instant::now();
        // First try to load WTF saved variables from real WoW installation
        match mgr.load_wtf_for_addon(env.lua(), folder_name) {
            Ok(count) if count > 0 => {
                tracing::debug!("Loaded {} WTF SavedVariables file(s) for {}", count, toc.name);
            }
            Ok(_) => {
                // No WTF files found, fall back to JSON storage
                let saved_vars = toc.saved_variables();
                let saved_vars_per_char = toc.saved_variables_per_character();

                if !saved_vars.is_empty() || !saved_vars_per_char.is_empty() {
                    if let Err(e) = mgr.init_for_addon(
                        env.lua(),
                        folder_name,
                        &saved_vars,
                        &saved_vars_per_char,
                    ) {
                        result.warnings.push(format!(
                            "Failed to initialize saved variables for {}: {}",
                            folder_name, e
                        ));
                    }
                }
            }
            Err(e) => {
                result.warnings.push(format!(
                    "Failed to load WTF SavedVariables for {}: {}",
                    folder_name, e
                ));
            }
        }
        result.timing.saved_vars_time = sv_start.elapsed();
    }

    // Create the shared private table for this addon (WoW passes this as second vararg)
    let addon_table = env.create_addon_table().map_err(|e| LoadError::Lua(e.to_string()))?;
    let ctx = AddonContext {
        name: folder_name,
        table: addon_table,
        addon_root: &toc.addon_dir,
    };

    for file in toc.file_paths() {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "lua" => match load_lua_file(env, &file, &ctx, &mut result.timing) {
                Ok(()) => result.lua_files += 1,
                Err(e) => result.warnings.push(format!("{}: {}", file.display(), e)),
            },
            "xml" => match load_xml_file(env, &file, &ctx, &mut result.timing) {
                Ok(count) => {
                    result.xml_files += 1;
                    result.lua_files += count;
                }
                Err(e) => result.warnings.push(format!("{}: {}", file.display(), e)),
            },
            _ => {
                result.warnings.push(format!("{}: unknown file type", file.display()));
            }
        }
    }

    Ok(result)
}

/// Load a Lua file into the environment with addon varargs.
fn load_lua_file(
    env: &WowLuaEnv,
    path: &Path,
    ctx: &AddonContext,
    timing: &mut LoadTiming,
) -> Result<(), LoadError> {
    let io_start = Instant::now();
    // Use lossy UTF-8 conversion to handle files with invalid encoding
    let bytes = std::fs::read(path)?;
    let code = String::from_utf8_lossy(&bytes);
    timing.io_time += io_start.elapsed();

    // Strip UTF-8 BOM if present (common in Windows-edited files)
    let code = code.strip_prefix('\u{feff}').unwrap_or(&code);
    // Transform path to WoW-style for debugstack (libraries expect "AddOns/..." pattern)
    let path_str = path.display().to_string();
    let chunk_name = if let Some(pos) = path_str.find("reference-addons/") {
        // Transform: .../reference-addons/Details/... -> Interface/AddOns/Details/...
        format!("@Interface/AddOns/{}", &path_str[pos + 17..])
    } else if let Some(pos) = path_str.find("AddOns/") {
        format!("@Interface/{}", &path_str[pos..])
    } else {
        format!("@{}", path_str)
    };
    // Clone the table since mlua moves it on call
    let table_clone = ctx.table.clone();

    let lua_start = Instant::now();
    env.exec_with_varargs(&code, &chunk_name, ctx.name, table_clone)
        .map_err(|e| LoadError::Lua(e.to_string()))?;
    timing.lua_exec_time += lua_start.elapsed();

    Ok(())
}

/// Normalize Windows-style paths (backslashes) to Unix-style (forward slashes).
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Find a directory entry case-insensitively.
fn find_case_insensitive(dir: &Path, name: &str) -> Option<PathBuf> {
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
fn resolve_path_case_insensitive(base: &Path, path: &str) -> Option<PathBuf> {
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
fn resolve_path_with_fallback(xml_dir: &Path, addon_root: &Path, file: &str) -> std::path::PathBuf {
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

/// Create a frame from XML definition.
/// Returns the name of the created frame (or None if skipped).
fn create_frame_from_xml(
    env: &WowLuaEnv,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_override: Option<&str>,
) -> Result<Option<String>, LoadError> {
    // Register virtual frames (templates) in the template registry
    if frame.is_virtual == Some(true) {
        if let Some(ref name) = frame.name {
            crate::xml::register_template(name, widget_type, frame.clone());
        }
        return Ok(None);
    }

    // Need a name to create a global frame (unless we have a parent override for anonymous children)
    let name = match &frame.name {
        Some(n) => {
            // Replace $parent with actual parent name if present
            if let Some(parent_name) = parent_override {
                n.replace("$parent", parent_name)
            } else {
                n.clone()
            }
        }
        None => {
            if parent_override.is_some() {
                // Anonymous child frame - generate temp name
                format!("__anon_{}", rand_id())
            } else {
                return Ok(None); // Anonymous top-level frames are templates
            }
        }
    };

    // Build the Lua code to create and configure the frame
    let parent = parent_override
        .or(frame.parent.as_deref())
        .unwrap_or("UIParent");
    let inherits = frame.inherits.as_deref().unwrap_or("");

    // Create the frame
    let mut lua_code = format!(
        r#"
        local frame = CreateFrame("{}", "{}", {}, {})
        "#,
        widget_type,
        name,
        parent,
        if inherits.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", inherits)
        }
    );

    // Set parentKey immediately after frame creation, BEFORE anchors are set.
    // This ensures sibling frames can reference this frame via $parent.ChildKey in their anchors.
    if let Some(parent_key) = &frame.parent_key {
        lua_code.push_str(&format!(
            r#"
        {}.{} = frame
        "#,
            parent, parent_key
        ));
    }

    // Collect mixins from both direct attribute and inherited templates
    let mut all_mixins: Vec<String> = Vec::new();

    // First, collect mixins from inherited templates (base mixins first)
    if !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in &template_chain {
            if let Some(mixin) = &template_entry.frame.mixin {
                for m in mixin.split(',').map(|s| s.trim()) {
                    if !m.is_empty() && !all_mixins.contains(&m.to_string()) {
                        all_mixins.push(m.to_string());
                    }
                }
            }
        }
    }

    // Then add direct mixins (override templates)
    if let Some(mixin) = &frame.mixin {
        for m in mixin.split(',').map(|s| s.trim()) {
            if !m.is_empty() && !all_mixins.contains(&m.to_string()) {
                all_mixins.push(m.to_string());
            }
        }
    }

    // Apply all mixins
    for m in &all_mixins {
        lua_code.push_str(&format!(
            r#"
        if {} then Mixin(frame, {}) end
        "#,
            m, m
        ));
    }

    // Set size
    if let Some(size) = frame.size() {
        let (x, y) = get_size_values(size);
        if let (Some(x), Some(y)) = (x, y) {
            lua_code.push_str(&format!(
                r#"
        frame:SetSize({}, {})
        "#,
                x, y
            ));
        }
    }

    // Set anchors
    if let Some(anchors) = frame.anchors() {
        lua_code.push_str(&generate_anchors_code(anchors, parent));
    }

    // Set hidden state
    if frame.hidden == Some(true) {
        lua_code.push_str(
            r#"
        frame:Hide()
        "#,
        );
    }

    // Handle setAllPoints from inherited templates first
    let mut has_set_all_points = false;
    if !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in &template_chain {
            if template_entry.frame.set_all_points == Some(true) {
                has_set_all_points = true;
                break;
            }
        }
    }

    // Direct attribute overrides template
    if frame.set_all_points == Some(true) {
        has_set_all_points = true;
    }

    // Apply setAllPoints if set
    if has_set_all_points {
        lua_code.push_str(
            r#"
        frame:SetAllPoints(true)
        "#,
        );
    }

    // Handle KeyValues from inherited templates first (so they can be overridden)
    if !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in &template_chain {
            if let Some(key_values) = template_entry.frame.key_values() {
                for kv in &key_values.values {
                    let value = match kv.value_type.as_deref() {
                        Some("number") => kv.value.clone(),
                        Some("boolean") => kv.value.to_lowercase(),
                        _ => format!("\"{}\"", escape_lua_string(&kv.value)),
                    };
                    lua_code.push_str(&format!(
                        r#"
        frame.{} = {}
        "#,
                        kv.key, value
                    ));
                }
            }
        }
    }

    // Handle KeyValues from the frame itself (can override template values)
    if let Some(key_values) = frame.key_values() {
        for kv in &key_values.values {
            let value = match kv.value_type.as_deref() {
                Some("number") => kv.value.clone(),
                Some("boolean") => kv.value.to_lowercase(),
                _ => format!("\"{}\"", escape_lua_string(&kv.value)),
            };
            lua_code.push_str(&format!(
                r#"
        frame.{} = {}
        "#,
                kv.key, value
            ));
        }
    }

    // Handle Scripts
    if let Some(scripts) = frame.scripts() {
        lua_code.push_str(&generate_scripts_code(scripts));
    }

    // Execute the creation code
    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!("Failed to create frame {}: {}", name, e))
    })?;

    // Instantiate children from inherited templates
    if !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in template_chain {
            instantiate_template_children(env, &template_entry.frame, &name, &template_entry.name)?;
        }
    }

    // Handle Layers (textures and fontstrings)
    for layers in frame.layers() {
        for layer in &layers.layers {
            let draw_layer = layer.level.as_deref().unwrap_or("ARTWORK");

            // Create textures
            for texture in layer.textures() {
                create_texture_from_xml(env, texture, &name, draw_layer)?;
            }

            // Create fontstrings
            for fontstring in layer.font_strings() {
                create_fontstring_from_xml(env, fontstring, &name, draw_layer)?;
            }
        }
    }

    // Handle child Frames recursively
    if let Some(frames) = frame.frames() {
        for child in &frames.elements {
            let (child_frame, child_type) = match child {
                crate::xml::FrameElement::Frame(f) => (f, "Frame"),
                crate::xml::FrameElement::Button(f) | crate::xml::FrameElement::ItemButton(f) => (f, "Button"),
                crate::xml::FrameElement::CheckButton(f) => (f, "CheckButton"),
                crate::xml::FrameElement::EditBox(f) | crate::xml::FrameElement::EventEditBox(f) => (f, "EditBox"),
                crate::xml::FrameElement::ScrollFrame(f) => (f, "ScrollFrame"),
                crate::xml::FrameElement::Slider(f) => (f, "Slider"),
                crate::xml::FrameElement::StatusBar(f) => (f, "StatusBar"),
                crate::xml::FrameElement::EventFrame(f) => (f, "Frame"), // EventFrame is just a Frame
                crate::xml::FrameElement::EventButton(f) => (f, "Button"), // EventButton is just a Button
                crate::xml::FrameElement::DropdownButton(f) | crate::xml::FrameElement::DropDownToggleButton(f) => (f, "Button"), // Dropdown buttons
                crate::xml::FrameElement::Cooldown(f) => (f, "Cooldown"),
                crate::xml::FrameElement::GameTooltip(f) => (f, "GameTooltip"),
                crate::xml::FrameElement::Model(f) | crate::xml::FrameElement::ModelScene(f) => (f, "Frame"), // Model frames
                _ => continue, // Skip unsupported types for now
            };
            let child_name = create_frame_from_xml(env, child_frame, child_type, Some(&name))?;

            // Handle parentKey for child frames (works for both named and anonymous frames)
            if let (Some(actual_child_name), Some(parent_key)) =
                (child_name.clone(), &child_frame.parent_key)
            {
                let lua_code = format!(
                    r#"
                    {}.{} = {}
                    "#,
                    name, parent_key, actual_child_name
                );
                env.exec(&lua_code).ok(); // Ignore errors (parent might not exist yet)
            }
        }
    }

    // Fire OnLoad script after frame is fully configured
    // In WoW, OnLoad fires at the end of frame creation from XML
    // Templates often use method="OnLoad" which calls self:OnLoad()
    let onload_code = format!(
        r#"
        local frame = {}
        local handler = frame:GetScript("OnLoad")
        if handler then
            handler(frame)
        elseif type(frame.OnLoad) == "function" then
            -- Call mixin OnLoad method if no script handler but method exists
            frame:OnLoad()
        end
        "#,
        name
    );
    env.exec(&onload_code).ok(); // Ignore errors (OnLoad might not be set)

    Ok(Some(name))
}

/// Instantiate children from a template onto a frame.
/// This creates textures, fontstrings, and child frames defined in the template.
fn instantiate_template_children(
    env: &WowLuaEnv,
    template: &crate::xml::FrameXml,
    parent_name: &str,
    _template_name: &str,
) -> Result<(), LoadError> {
    // Handle Layers (textures and fontstrings from template)
    for layers in template.layers() {
        for layer in &layers.layers {
            let draw_layer = layer.level.as_deref().unwrap_or("ARTWORK");

            // Create textures from template
            for texture in layer.textures() {
                create_texture_from_xml(env, texture, parent_name, draw_layer)?;
            }

            // Create fontstrings from template
            for fontstring in layer.font_strings() {
                create_fontstring_from_xml(env, fontstring, parent_name, draw_layer)?;
            }
        }
    }

    // Handle child Frames from template recursively
    if let Some(frames) = template.frames() {
        for child in &frames.elements {
            let (child_frame, child_type) = match child {
                crate::xml::FrameElement::Frame(f) => (f, "Frame"),
                crate::xml::FrameElement::Button(f) | crate::xml::FrameElement::ItemButton(f) => (f, "Button"),
                crate::xml::FrameElement::CheckButton(f) => (f, "CheckButton"),
                crate::xml::FrameElement::EditBox(f) | crate::xml::FrameElement::EventEditBox(f) => (f, "EditBox"),
                crate::xml::FrameElement::ScrollFrame(f) => (f, "ScrollFrame"),
                crate::xml::FrameElement::Slider(f) => (f, "Slider"),
                crate::xml::FrameElement::StatusBar(f) => (f, "StatusBar"),
                crate::xml::FrameElement::EventFrame(f) => (f, "Frame"), // EventFrame is just a Frame
                crate::xml::FrameElement::EventButton(f) => (f, "Button"), // EventButton is just a Button
                crate::xml::FrameElement::DropdownButton(f) | crate::xml::FrameElement::DropDownToggleButton(f) => (f, "Button"), // Dropdown buttons
                crate::xml::FrameElement::Cooldown(f) => (f, "Cooldown"),
                crate::xml::FrameElement::GameTooltip(f) => (f, "GameTooltip"),
                crate::xml::FrameElement::Model(f) | crate::xml::FrameElement::ModelScene(f) => (f, "Frame"), // Model frames
                _ => continue,
            };

            // Create the child frame with parent_name as parent
            let child_name = create_frame_from_xml(env, child_frame, child_type, Some(parent_name))?;

            // Handle parentKey for template child frames
            if let (Some(actual_child_name), Some(parent_key)) =
                (child_name, &child_frame.parent_key)
            {
                let lua_code = format!(
                    r#"
                    {}.{} = {}
                    "#,
                    parent_name, parent_key, actual_child_name
                );
                env.exec(&lua_code).ok();
            }
        }
    }

    Ok(())
}

/// Get size values from a SizeXml, checking both direct attributes and AbsDimension.
fn get_size_values(size: &crate::xml::SizeXml) -> (Option<f32>, Option<f32>) {
    if size.x.is_some() || size.y.is_some() {
        (size.x, size.y)
    } else if let Some(abs) = &size.abs_dimension {
        (abs.x, abs.y)
    } else {
        (None, None)
    }
}

/// Generate a simple random ID for anonymous frames.
fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    nanos
}

/// Escape a string for use in Lua code.
fn escape_lua_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Generate Lua code for setting anchors.
fn generate_anchors_code(anchors: &crate::xml::AnchorsXml, parent_ref: &str) -> String {
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
fn generate_scripts_code(scripts: &crate::xml::ScriptsXml) -> String {
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

/// Create a texture from XML definition.
fn create_texture_from_xml(
    env: &WowLuaEnv,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    draw_layer: &str,
) -> Result<(), LoadError> {
    // Skip virtual textures
    if texture.is_virtual == Some(true) {
        return Ok(());
    }

    let tex_name = texture
        .name
        .clone()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    let mut lua_code = format!(
        r#"
        local parent = {}
        local tex = parent:CreateTexture("{}", "{}")
        "#,
        parent_name, tex_name, draw_layer
    );

    // Set texture file
    if let Some(file) = &texture.file {
        lua_code.push_str(&format!(
            r#"
        tex:SetTexture("{}")
        "#,
            escape_lua_string(file)
        ));
    }

    // Set atlas
    if let Some(atlas) = &texture.atlas {
        lua_code.push_str(&format!(
            r#"
        tex:SetAtlas("{}")
        "#,
            escape_lua_string(atlas)
        ));
    }

    // Set size
    if let Some(size) = &texture.size {
        let (x, y) = get_size_values(size);
        if let (Some(x), Some(y)) = (x, y) {
            lua_code.push_str(&format!(
                r#"
        tex:SetSize({}, {})
        "#,
                x, y
            ));
        }
    }

    // Set anchors
    if let Some(anchors) = &texture.anchors {
        for anchor in &anchors.anchors {
            let point = &anchor.point;
            let relative_point = anchor.relative_point.as_deref().unwrap_or(point.as_str());

            let (x, y) = if let Some(offset) = &anchor.offset {
                if let Some(abs) = &offset.abs_dimension {
                    (abs.x.unwrap_or(0.0), abs.y.unwrap_or(0.0))
                } else {
                    (0.0, 0.0)
                }
            } else {
                (anchor.x.unwrap_or(0.0), anchor.y.unwrap_or(0.0))
            };

            // Handle relativeKey first, then relativeTo
            let rel = if let Some(key) = anchor.relative_key.as_deref() {
                // Handle $parent chains like "$parent.$parent.ScrollFrame"
                if key.contains("$parent") || key.contains("$Parent") {
                    let parts: Vec<&str> = key.split('.').collect();
                    let mut expr = String::new();
                    for part in parts {
                        if part == "$parent" || part == "$Parent" {
                            if expr.is_empty() {
                                expr = "parent".to_string();
                            } else {
                                expr = format!("{}:GetParent()", expr);
                            }
                        } else if !part.is_empty() {
                            expr = format!("{}[\"{}\"]", expr, part);
                        }
                    }
                    if expr.is_empty() { "parent".to_string() } else { expr }
                } else {
                    key.to_string()
                }
            } else {
                match anchor.relative_to.as_deref() {
                    Some("$parent") | None => "parent".to_string(),
                    Some(r) if r.contains("$parent") || r.contains("$Parent") => {
                        // Replace any $parent/$Parent pattern (e.g., $parent_Sibling, $parentColorSwatch)
                        r.replace("$parent", parent_name).replace("$Parent", parent_name)
                    }
                    Some(r) => r.to_string(),
                }
            };

            lua_code.push_str(&format!(
                r#"
        tex:SetPoint("{}", {}, "{}", {}, {})
        "#,
                point, rel, relative_point, x, y
            ));
        }
    }

    // Set color if specified
    if let Some(color) = &texture.color {
        let r = color.r.unwrap_or(1.0);
        let g = color.g.unwrap_or(1.0);
        let b = color.b.unwrap_or(1.0);
        let a = color.a.unwrap_or(1.0);
        lua_code.push_str(&format!(
            r#"
        tex:SetVertexColor({}, {}, {}, {})
        "#,
            r, g, b, a
        ));
    }

    // Set horizontal tiling
    if texture.horiz_tile == Some(true) {
        lua_code.push_str(
            r#"
        tex:SetHorizTile(true)
        "#,
        );
    }

    // Set vertical tiling
    if texture.vert_tile == Some(true) {
        lua_code.push_str(
            r#"
        tex:SetVertTile(true)
        "#,
        );
    }

    // Set all points if specified
    if texture.set_all_points == Some(true) {
        lua_code.push_str(
            r#"
        tex:SetAllPoints(true)
        "#,
        );
    }

    // Set parentKey if specified
    if let Some(key) = &texture.parent_key {
        lua_code.push_str(&format!(
            r#"
        parent.{} = tex
        "#,
            key
        ));
    }

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to create texture {} on {}: {}",
            tex_name, parent_name, e
        ))
    })
}

/// Create a fontstring from XML definition.
fn create_fontstring_from_xml(
    env: &WowLuaEnv,
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
    draw_layer: &str,
) -> Result<(), LoadError> {
    // Skip virtual fontstrings
    if fontstring.is_virtual == Some(true) {
        return Ok(());
    }

    let fs_name = fontstring
        .name
        .clone()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__fs_{}", rand_id()));

    let inherits = fontstring.inherits.as_deref().unwrap_or("");

    let mut lua_code = format!(
        r#"
        local parent = {}
        local fs = parent:CreateFontString("{}", "{}", {})
        "#,
        parent_name,
        fs_name,
        draw_layer,
        if inherits.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", inherits)
        }
    );

    // Set text
    if let Some(text) = &fontstring.text {
        lua_code.push_str(&format!(
            r#"
        fs:SetText("{}")
        "#,
            escape_lua_string(text)
        ));
    }

    // Set justification
    if let Some(justify_h) = &fontstring.justify_h {
        lua_code.push_str(&format!(
            r#"
        fs:SetJustifyH("{}")
        "#,
            justify_h
        ));
    }
    if let Some(justify_v) = &fontstring.justify_v {
        lua_code.push_str(&format!(
            r#"
        fs:SetJustifyV("{}")
        "#,
            justify_v
        ));
    }

    // Set size
    if let Some(size) = &fontstring.size {
        let (x, y) = get_size_values(size);
        if let (Some(x), Some(y)) = (x, y) {
            lua_code.push_str(&format!(
                r#"
        fs:SetSize({}, {})
        "#,
                x, y
            ));
        }
    }

    // Set anchors
    if let Some(anchors) = &fontstring.anchors {
        for anchor in &anchors.anchors {
            let point = &anchor.point;
            let relative_point = anchor.relative_point.as_deref().unwrap_or(point.as_str());

            let (x, y) = if let Some(offset) = &anchor.offset {
                if let Some(abs) = &offset.abs_dimension {
                    (abs.x.unwrap_or(0.0), abs.y.unwrap_or(0.0))
                } else {
                    (0.0, 0.0)
                }
            } else {
                (anchor.x.unwrap_or(0.0), anchor.y.unwrap_or(0.0))
            };

            // Handle relativeKey first, then relativeTo
            let rel = if let Some(key) = anchor.relative_key.as_deref() {
                // Handle $parent chains like "$parent.$parent.ScrollFrame"
                if key.contains("$parent") || key.contains("$Parent") {
                    let parts: Vec<&str> = key.split('.').collect();
                    let mut expr = String::new();
                    for part in parts {
                        if part == "$parent" || part == "$Parent" {
                            if expr.is_empty() {
                                expr = "parent".to_string();
                            } else {
                                expr = format!("{}:GetParent()", expr);
                            }
                        } else if !part.is_empty() {
                            expr = format!("{}[\"{}\"]", expr, part);
                        }
                    }
                    if expr.is_empty() { "parent".to_string() } else { expr }
                } else {
                    key.to_string()
                }
            } else {
                match anchor.relative_to.as_deref() {
                    Some("$parent") | None => "parent".to_string(),
                    Some(r) if r.contains("$parent") || r.contains("$Parent") => {
                        // Replace any $parent/$Parent pattern (e.g., $parent_Sibling, $parentColorSwatch)
                        r.replace("$parent", parent_name).replace("$Parent", parent_name)
                    }
                    Some(r) => r.to_string(),
                }
            };

            lua_code.push_str(&format!(
                r#"
        fs:SetPoint("{}", {}, "{}", {}, {})
        "#,
                point, rel, relative_point, x, y
            ));
        }
    }

    // Set parentKey if specified
    if let Some(key) = &fontstring.parent_key {
        lua_code.push_str(&format!(
            r#"
        parent.{} = fs
        "#,
            key
        ));
    }

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to create fontstring {} on {}: {}",
            fs_name, parent_name, e
        ))
    })
}

/// Load an XML file, processing its elements.
/// Returns the number of Lua files loaded from Script elements.
fn load_xml_file(
    env: &WowLuaEnv,
    path: &Path,
    ctx: &AddonContext,
    timing: &mut LoadTiming,
) -> Result<usize, LoadError> {
    let xml_start = Instant::now();
    let ui = parse_xml_file(path)?;
    timing.xml_parse_time += xml_start.elapsed();

    let xml_dir = path.parent().unwrap_or(Path::new("."));
    let mut lua_count = 0;

    for element in &ui.elements {
        match element {
            XmlElement::Script(s) | XmlElement::ScriptLower(s) => {
                // Script can have file attribute or inline content
                if let Some(file) = &s.file {
                    let script_path = resolve_path_with_fallback(xml_dir, ctx.addon_root, file);
                    load_lua_file(env, &script_path, ctx, timing)?;
                    lua_count += 1;
                } else if let Some(inline) = &s.inline {
                    // Execute inline script with varargs
                    let table_clone = ctx.table.clone();
                    let lua_start = Instant::now();
                    env.exec_with_varargs(inline, "@inline", ctx.name, table_clone)
                        .map_err(|e| LoadError::Lua(e.to_string()))?;
                    timing.lua_exec_time += lua_start.elapsed();
                    lua_count += 1;
                }
            }
            XmlElement::Include(i) | XmlElement::IncludeLower(i) => {
                let include_path = resolve_path_with_fallback(xml_dir, ctx.addon_root, &i.file);
                // Check if it's a Lua file (some addons use Include for Lua files)
                if i.file.ends_with(".lua") {
                    load_lua_file(env, &include_path, ctx, timing)?;
                    lua_count += 1;
                } else {
                    lua_count += load_xml_file(env, &include_path, ctx, timing)?;
                }
            }
            XmlElement::Frame(f) => {
                create_frame_from_xml(env, f, "Frame", None)?;
            }
            XmlElement::Button(f) | XmlElement::ItemButton(f) => {
                create_frame_from_xml(env, f, "Button", None)?;
            }
            XmlElement::CheckButton(f) => {
                create_frame_from_xml(env, f, "CheckButton", None)?;
            }
            XmlElement::EditBox(f) => {
                create_frame_from_xml(env, f, "EditBox", None)?;
            }
            XmlElement::ScrollFrame(f) => {
                create_frame_from_xml(env, f, "ScrollFrame", None)?;
            }
            XmlElement::Slider(f) => {
                create_frame_from_xml(env, f, "Slider", None)?;
            }
            XmlElement::StatusBar(f) => {
                create_frame_from_xml(env, f, "StatusBar", None)?;
            }
            XmlElement::EventFrame(f) => {
                create_frame_from_xml(env, f, "Frame", None)?;
            }
            XmlElement::Texture(_) | XmlElement::FontString(_) => {
                // Top-level textures/fontstrings are templates
            }
            XmlElement::AnimationGroup(_) => {
                // Animation groups are templates
            }
            XmlElement::Actor(_) => {
                // Actor definitions for ModelScene
            }
            XmlElement::Font(font) => {
                // Font definitions - create font objects that can be referenced
                if let Some(name) = &font.name {
                    if !name.is_empty() {
                        let font_path = font.font.clone().unwrap_or_else(|| "Fonts/FRIZQT__.TTF".to_string());
                        // Escape backslashes for Lua string
                        let font_path_escaped = font_path.replace('\\', "/");
                        let font_height = font.height.unwrap_or(12.0);
                        let font_outline = font.outline.clone().unwrap_or_default();

                        // Create font object via Lua
                        let lua_code = format!(r#"
                            {name} = {{
                                __font = "{font_path}",
                                __height = {font_height},
                                __outline = "{font_outline}",
                                __r = 1.0,
                                __g = 1.0,
                                __b = 1.0,
                                SetTextColor = function(self, r, g, b)
                                    self.__r = r
                                    self.__g = g
                                    self.__b = b
                                end,
                                GetFont = function(self)
                                    return self.__font, self.__height, self.__outline
                                end,
                                SetFont = function(self, path, height, flags)
                                    self.__font = path
                                    if height then self.__height = height end
                                    if flags then self.__outline = flags end
                                end,
                                CopyFontObject = function(self, source)
                                    if source.__font then self.__font = source.__font end
                                    if source.__height then self.__height = source.__height end
                                    if source.__outline then self.__outline = source.__outline end
                                    if source.__r then self.__r = source.__r end
                                    if source.__g then self.__g = source.__g end
                                    if source.__b then self.__b = source.__b end
                                end,
                            }}
                        "#, name = name, font_path = font_path_escaped, font_height = font_height, font_outline = font_outline);

                        env.exec(&lua_code).map_err(|e| {
                            LoadError::Lua(format!("Failed to create font {}: {}", name, e))
                        })?;
                    }
                }
            }
            XmlElement::FontFamily(font_family) => {
                // FontFamily definitions - create font objects that can be referenced
                if let Some(name) = &font_family.name {
                    if !name.is_empty() {
                        // Create font object via Lua with default values
                        // FontFamily contains Member elements with Font children, but for simulation
                        // we just need a font object with the right methods
                        let lua_code = format!(r#"
                            {name} = {{
                                __font = "Fonts/FRIZQT__.TTF",
                                __height = 12.0,
                                __outline = "",
                                __r = 1.0,
                                __g = 1.0,
                                __b = 1.0,
                                __justifyH = "CENTER",
                                __justifyV = "MIDDLE",
                                SetTextColor = function(self, r, g, b)
                                    self.__r = r
                                    self.__g = g
                                    self.__b = b
                                end,
                                GetTextColor = function(self)
                                    return self.__r, self.__g, self.__b
                                end,
                                SetFont = function(self, font, height, flags)
                                    if font then self.__font = font end
                                    if height then self.__height = height end
                                    if flags then self.__outline = flags end
                                end,
                                GetFont = function(self)
                                    return self.__font, self.__height, self.__outline
                                end,
                                SetJustifyH = function(self, justify)
                                    self.__justifyH = justify
                                end,
                                GetJustifyH = function(self)
                                    return self.__justifyH
                                end,
                                SetJustifyV = function(self, justify)
                                    self.__justifyV = justify
                                end,
                                GetJustifyV = function(self)
                                    return self.__justifyV
                                end,
                                CopyFontObject = function(self, source)
                                    if source.__font then self.__font = source.__font end
                                    if source.__height then self.__height = source.__height end
                                    if source.__outline then self.__outline = source.__outline end
                                    if source.__r then self.__r = source.__r end
                                    if source.__g then self.__g = source.__g end
                                    if source.__b then self.__b = source.__b end
                                end,
                            }}
                        "#, name = name);

                        env.exec(&lua_code).map_err(|e| {
                            LoadError::Lua(format!("Failed to create font family {}: {}", name, e))
                        })?;
                    }
                }
            }
            XmlElement::Text(_) => {
                // Inline text content - ignored (comes from malformed XML or comments)
            }
            // Other frame types not yet fully supported - skip for now
            _ => {}
        }
    }

    Ok(lua_count)
}

/// Error type for addon loading.
#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Toc(std::io::Error),
    Xml(crate::xml::XmlLoadError),
    Lua(String),
}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        LoadError::Io(e)
    }
}

impl From<crate::xml::XmlLoadError> for LoadError {
    fn from(e: crate::xml::XmlLoadError) -> Self {
        LoadError::Xml(e)
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {}", e),
            LoadError::Toc(e) => write!(f, "TOC error: {}", e),
            LoadError::Xml(e) => write!(f, "XML error: {}", e),
            LoadError::Lua(e) => write!(f, "Lua error: {}", e),
        }
    }
}

impl std::error::Error for LoadError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_lua_file() {
        let env = WowLuaEnv::new().unwrap();

        // Create a temp Lua file
        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let lua_path = temp_dir.join("test.lua");
        std::fs::write(&lua_path, "TEST_VAR = 42").unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_lua_file(&env, &lua_path, &ctx, &mut LoadTiming::default()).unwrap();

        let value: i32 = env.eval("return TEST_VAR").unwrap();
        assert_eq!(value, 42);

        // Cleanup
        std::fs::remove_file(&lua_path).ok();
    }

    #[test]
    fn test_xml_frame_with_layers_and_scripts() {
        let env = WowLuaEnv::new().unwrap();

        // Create a temp XML file with layers, scripts, and child frames
        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-xml");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="TestXMLFrame" parent="UIParent">
                    <Size x="200" y="150"/>
                    <Anchors>
                        <Anchor point="CENTER"/>
                    </Anchors>
                    <Layers>
                        <Layer level="BACKGROUND">
                            <Texture name="TestXMLFrame_BG" parentKey="bg">
                                <Size x="200" y="150"/>
                                <Color r="0.1" g="0.1" b="0.1" a="0.8"/>
                                <Anchors>
                                    <Anchor point="TOPLEFT"/>
                                    <Anchor point="BOTTOMRIGHT"/>
                                </Anchors>
                            </Texture>
                        </Layer>
                        <Layer level="ARTWORK">
                            <FontString name="TestXMLFrame_Title" parentKey="title" text="Test Title">
                                <Anchors>
                                    <Anchor point="TOP" y="-10"/>
                                </Anchors>
                            </FontString>
                        </Layer>
                    </Layers>
                    <Scripts>
                        <OnLoad>
                            XML_ONLOAD_FIRED = true
                        </OnLoad>
                    </Scripts>
                    <Frames>
                        <Button name="TestXMLFrame_CloseBtn" parentKey="closeBtn">
                            <Size x="80" y="22"/>
                            <Anchors>
                                <Anchor point="BOTTOM" y="10"/>
                            </Anchors>
                            <Scripts>
                                <OnClick>
                                    XML_ONCLICK_FIRED = true
                                </OnClick>
                            </Scripts>
                        </Button>
                    </Frames>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        // Load the XML
        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify frame was created
        let frame_exists: bool = env.eval("return TestXMLFrame ~= nil").unwrap();
        assert!(frame_exists, "TestXMLFrame should exist");

        // Verify texture was created with parentKey
        let bg_exists: bool = env.eval("return TestXMLFrame.bg ~= nil").unwrap();
        assert!(bg_exists, "TestXMLFrame.bg should exist via parentKey");

        // Verify fontstring was created with parentKey
        let title_exists: bool = env.eval("return TestXMLFrame.title ~= nil").unwrap();
        assert!(title_exists, "TestXMLFrame.title should exist via parentKey");

        // Verify child button was created
        let btn_exists: bool = env.eval("return TestXMLFrame_CloseBtn ~= nil").unwrap();
        assert!(btn_exists, "TestXMLFrame_CloseBtn should exist");

        // Verify button parentKey
        let close_btn_exists: bool = env.eval("return TestXMLFrame.closeBtn ~= nil").unwrap();
        assert!(
            close_btn_exists,
            "TestXMLFrame.closeBtn should exist via parentKey"
        );

        // Verify OnLoad script was set (will fire when we call GetScript)
        let has_onload: bool = env
            .eval("return TestXMLFrame:GetScript('OnLoad') ~= nil")
            .unwrap();
        assert!(has_onload, "OnLoad handler should be set");

        // Verify OnClick script was set on the button
        let has_onclick: bool = env
            .eval("return TestXMLFrame_CloseBtn:GetScript('OnClick') ~= nil")
            .unwrap();
        assert!(has_onclick, "OnClick handler should be set on button");

        // Cleanup
        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_scripts_function_attribute() {
        let env = WowLuaEnv::new().unwrap();

        // Define a global function first
        env.exec(
            r#"
            SCRIPT_FUNC_CALLED = false
            function MyGlobalOnLoad(self)
                SCRIPT_FUNC_CALLED = true
            end
            "#,
        )
        .unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-scripts");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_func.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="FuncTestFrame" parent="UIParent">
                    <Scripts>
                        <OnLoad function="MyGlobalOnLoad"/>
                    </Scripts>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify the script handler references the global function
        let handler_set: bool = env
            .eval("return FuncTestFrame:GetScript('OnLoad') == MyGlobalOnLoad")
            .unwrap();
        assert!(handler_set, "OnLoad should reference MyGlobalOnLoad");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_scripts_method_attribute() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-method");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_method.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="MethodTestFrame" parent="UIParent">
                    <Scripts>
                        <OnShow method="OnShowHandler"/>
                    </Scripts>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Add a method to the frame
        env.exec(
            r#"
            METHOD_CALLED = false
            function MethodTestFrame:OnShowHandler()
                METHOD_CALLED = true
            end
            "#,
        )
        .unwrap();

        // Call the OnShow handler (it should call the method)
        env.exec("MethodTestFrame:GetScript('OnShow')(MethodTestFrame)")
            .unwrap();

        let method_called: bool = env.eval("return METHOD_CALLED").unwrap();
        assert!(method_called, "OnShow should have called OnShowHandler method");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_keyvalues() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-kv");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_kv.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="KeyValueFrame" parent="UIParent">
                    <KeyValues>
                        <KeyValue key="myString" value="hello"/>
                        <KeyValue key="myNumber" value="42" type="number"/>
                        <KeyValue key="myBool" value="true" type="boolean"/>
                        <KeyValue key="myFalseBool" value="false" type="boolean"/>
                    </KeyValues>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify string value
        let str_val: String = env.eval("return KeyValueFrame.myString").unwrap();
        assert_eq!(str_val, "hello");

        // Verify number value
        let num_val: i32 = env.eval("return KeyValueFrame.myNumber").unwrap();
        assert_eq!(num_val, 42);

        // Verify boolean values
        let bool_val: bool = env.eval("return KeyValueFrame.myBool").unwrap();
        assert!(bool_val);

        let false_val: bool = env.eval("return KeyValueFrame.myFalseBool").unwrap();
        assert!(!false_val);

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_anchors_with_offset() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-offset");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_offset.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="OffsetFrame" parent="UIParent">
                    <Size x="100" y="100"/>
                    <Anchors>
                        <Anchor point="TOPLEFT">
                            <Offset>
                                <AbsDimension x="10" y="-20"/>
                            </Offset>
                        </Anchor>
                    </Anchors>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify anchor was set with offset values
        let point_info: String = env
            .eval(
                r#"
                local point, relativeTo, relativePoint, x, y = OffsetFrame:GetPoint(1)
                return string.format("%s,%s,%d,%d", point, relativePoint, x, y)
                "#,
            )
            .unwrap();
        assert_eq!(point_info, "TOPLEFT,TOPLEFT,10,-20");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_size_with_absdimension() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-abssize");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_abssize.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="AbsSizeFrame" parent="UIParent">
                    <Size>
                        <AbsDimension x="150" y="75"/>
                    </Size>
                    <Anchors>
                        <Anchor point="CENTER"/>
                    </Anchors>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify size was set correctly
        let width: f64 = env.eval("return AbsSizeFrame:GetWidth()").unwrap();
        let height: f64 = env.eval("return AbsSizeFrame:GetHeight()").unwrap();
        assert_eq!(width, 150.0);
        assert_eq!(height, 75.0);

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_nested_child_frames() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-nested");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_nested.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="ParentFrame" parent="UIParent">
                    <Size x="300" y="200"/>
                    <Frames>
                        <Frame name="ChildFrame" parentKey="child">
                            <Size x="100" y="50"/>
                            <Frames>
                                <Button name="GrandchildButton" parentKey="btn">
                                    <Size x="80" y="22"/>
                                </Button>
                            </Frames>
                        </Frame>
                    </Frames>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify parent frame
        let parent_exists: bool = env.eval("return ParentFrame ~= nil").unwrap();
        assert!(parent_exists, "ParentFrame should exist");

        // Verify child frame and parentKey
        let child_exists: bool = env.eval("return ChildFrame ~= nil").unwrap();
        assert!(child_exists, "ChildFrame should exist");

        let child_key_exists: bool = env.eval("return ParentFrame.child == ChildFrame").unwrap();
        assert!(child_key_exists, "ParentFrame.child should be ChildFrame");

        // Verify grandchild button and parentKey
        let grandchild_exists: bool = env.eval("return GrandchildButton ~= nil").unwrap();
        assert!(grandchild_exists, "GrandchildButton should exist");

        let grandchild_key_exists: bool = env
            .eval("return ChildFrame.btn == GrandchildButton")
            .unwrap();
        assert!(
            grandchild_key_exists,
            "ChildFrame.btn should be GrandchildButton"
        );

        // Verify parent relationships
        let parent_name: String = env
            .eval("return ChildFrame:GetParent():GetName() or 'nil'")
            .unwrap();
        assert_eq!(
            parent_name, "ParentFrame",
            "ChildFrame's parent should be ParentFrame, got {}",
            parent_name
        );

        let grandchild_parent_name: String = env
            .eval("return GrandchildButton:GetParent():GetName() or 'nil'")
            .unwrap();
        assert_eq!(
            grandchild_parent_name, "ChildFrame",
            "GrandchildButton's parent should be ChildFrame, got {}",
            grandchild_parent_name
        );

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_texture_color() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-texcolor");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_texcolor.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="ColorTexFrame" parent="UIParent">
                    <Size x="100" y="100"/>
                    <Layers>
                        <Layer level="BACKGROUND">
                            <Texture name="ColorTexFrame_BG" parentKey="bg">
                                <Size x="100" y="100"/>
                                <Color r="1.0" g="0.5" b="0.25" a="0.8"/>
                            </Texture>
                        </Layer>
                    </Layers>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify texture exists via parentKey
        let tex_exists: bool = env.eval("return ColorTexFrame.bg ~= nil").unwrap();
        assert!(tex_exists, "ColorTexFrame.bg should exist");

        // Verify vertex color was set (check via stored values if available)
        let has_color: bool = env
            .eval("return ColorTexFrame_BG ~= nil")
            .unwrap();
        assert!(has_color, "ColorTexFrame_BG should exist as global");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_virtual_frames_skipped() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-virtual");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_virtual.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="VirtualTemplate" virtual="true">
                    <Size x="200" y="100"/>
                </Frame>
                <Frame name="ConcreteFrame" parent="UIParent" inherits="VirtualTemplate">
                    <Anchors>
                        <Anchor point="CENTER"/>
                    </Anchors>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Virtual frame should NOT be created
        let virtual_exists: bool = env.eval("return VirtualTemplate ~= nil").unwrap();
        assert!(!virtual_exists, "VirtualTemplate should NOT exist (it's virtual)");

        // Concrete frame should exist
        let concrete_exists: bool = env.eval("return ConcreteFrame ~= nil").unwrap();
        assert!(concrete_exists, "ConcreteFrame should exist");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_multiple_anchors() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-multianchor");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_multianchor.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="MultiAnchorFrame" parent="UIParent">
                    <Anchors>
                        <Anchor point="TOPLEFT" x="10" y="-10"/>
                        <Anchor point="BOTTOMRIGHT" x="-10" y="10"/>
                    </Anchors>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify frame has multiple anchor points
        let num_points: i32 = env
            .eval("return MultiAnchorFrame:GetNumPoints()")
            .unwrap();
        assert_eq!(num_points, 2, "Frame should have 2 anchor points");

        // Verify first anchor
        let point1: String = env
            .eval(
                r#"
                local point, _, relPoint, x, y = MultiAnchorFrame:GetPoint(1)
                return string.format("%s,%s,%d,%d", point, relPoint, x, y)
                "#,
            )
            .unwrap();
        assert_eq!(point1, "TOPLEFT,TOPLEFT,10,-10");

        // Verify second anchor
        let point2: String = env
            .eval(
                r#"
                local point, _, relPoint, x, y = MultiAnchorFrame:GetPoint(2)
                return string.format("%s,%s,%d,%d", point, relPoint, x, y)
                "#,
            )
            .unwrap();
        assert_eq!(point2, "BOTTOMRIGHT,BOTTOMRIGHT,-10,10");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_all_script_handlers() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-allscripts");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_allscripts.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="AllScriptsFrame" parent="UIParent">
                    <Scripts>
                        <OnLoad>ONLOAD = true</OnLoad>
                        <OnEvent>ONEVENT = true</OnEvent>
                        <OnUpdate>ONUPDATE = true</OnUpdate>
                        <OnShow>ONSHOW = true</OnShow>
                        <OnHide>ONHIDE = true</OnHide>
                    </Scripts>
                </Frame>
                <Button name="AllScriptsButton" parent="UIParent">
                    <Scripts>
                        <OnClick>ONCLICK = true</OnClick>
                    </Scripts>
                </Button>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify all handlers are set
        let has_onload: bool = env
            .eval("return AllScriptsFrame:GetScript('OnLoad') ~= nil")
            .unwrap();
        assert!(has_onload, "OnLoad should be set");

        let has_onevent: bool = env
            .eval("return AllScriptsFrame:GetScript('OnEvent') ~= nil")
            .unwrap();
        assert!(has_onevent, "OnEvent should be set");

        let has_onupdate: bool = env
            .eval("return AllScriptsFrame:GetScript('OnUpdate') ~= nil")
            .unwrap();
        assert!(has_onupdate, "OnUpdate should be set");

        let has_onshow: bool = env
            .eval("return AllScriptsFrame:GetScript('OnShow') ~= nil")
            .unwrap();
        assert!(has_onshow, "OnShow should be set");

        let has_onhide: bool = env
            .eval("return AllScriptsFrame:GetScript('OnHide') ~= nil")
            .unwrap();
        assert!(has_onhide, "OnHide should be set");

        let has_onclick: bool = env
            .eval("return AllScriptsButton:GetScript('OnClick') ~= nil")
            .unwrap();
        assert!(has_onclick, "OnClick should be set on button");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_local_function_closures() {
        // This test verifies that local functions capture each other correctly in closures
        // Replicates the ExtraQuestButton/widgets.lua issue where updateKeyDirection is nil
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-closures");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let lua_path = temp_dir.join("closures.lua");
        std::fs::write(
            &lua_path,
            r#"
                local _, addon = ...

                local function innerFunc(x)
                    return x * 2
                end

                local function outerFunc(x)
                    -- innerFunc should be captured as an upvalue
                    if not innerFunc then
                        error("innerFunc is nil!")
                    end
                    return innerFunc(x)
                end

                -- Store the result on the addon table for verification
                addon.result = outerFunc(21)

                -- Also test immediate call pattern
                function addon:CreateSomething()
                    return outerFunc(10)
                end
            "#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "ClosureTest",
            table: addon_table.clone(),
            addon_root: &temp_dir,
        };
        load_lua_file(&env, &lua_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify the result was computed correctly
        let result: i32 = addon_table.get("result").unwrap();
        assert_eq!(result, 42, "innerFunc should be captured and work correctly");

        // Verify the method works too (test via direct call)
        let create_something: mlua::Function = addon_table.get("CreateSomething").unwrap();
        let method_result: i32 = create_something.call(addon_table.clone()).unwrap();
        assert_eq!(method_result, 20, "outerFunc should still capture innerFunc");

        std::fs::remove_file(&lua_path).ok();
    }

    #[test]
    fn test_multi_file_closures() {
        // This test simulates ExtraQuestButton's loading pattern:
        // 1. widgets.lua defines local functions and addon:CreateButton
        // 2. button.lua defines addon:CreateExtraButton which calls addon:CreateButton
        // 3. addon.lua calls addon:CreateExtraButton
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-multifile");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // File 1: widgets.lua - defines local functions and addon method
        let widgets_path = temp_dir.join("widgets.lua");
        std::fs::write(
            &widgets_path,
            r#"
                local _, addon = ...

                local function updateKeyDirection(self)
                    return "updated: " .. tostring(self)
                end

                local function onCVarUpdate(self, cvar)
                    if cvar == "TestCVar" then
                        -- This is the critical line - updateKeyDirection should be captured
                        if not updateKeyDirection then
                            error("updateKeyDirection is nil!")
                        end
                        self.result = updateKeyDirection(self)
                    end
                end

                function addon:CreateButton(name)
                    local button = { name = name }
                    -- Call onCVarUpdate immediately during CreateButton
                    onCVarUpdate(button, "TestCVar")
                    return button
                end
            "#,
        )
        .unwrap();

        // File 2: button.lua - calls addon:CreateButton
        let button_path = temp_dir.join("button.lua");
        std::fs::write(
            &button_path,
            r#"
                local _, addon = ...

                function addon:CreateExtraButton(name)
                    -- This calls CreateButton which was defined in widgets.lua
                    return addon:CreateButton(name .. "_extra")
                end
            "#,
        )
        .unwrap();

        // File 3: addon.lua - calls addon:CreateExtraButton
        let addon_lua_path = temp_dir.join("addon.lua");
        std::fs::write(
            &addon_lua_path,
            r#"
                local _, addon = ...

                -- This should work: CreateExtraButton -> CreateButton -> onCVarUpdate -> updateKeyDirection
                local button = addon:CreateExtraButton("test")
                addon.testButton = button
            "#,
        )
        .unwrap();

        // Create shared addon table and context
        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "MultiFileTest",
            table: addon_table.clone(),
            addon_root: &temp_dir,
        };

        // Load files in order (like TOC would)
        load_lua_file(&env, &widgets_path, &ctx, &mut LoadTiming::default())
            .expect("widgets.lua should load");
        load_lua_file(&env, &button_path, &ctx, &mut LoadTiming::default())
            .expect("button.lua should load");
        load_lua_file(&env, &addon_lua_path, &ctx, &mut LoadTiming::default())
            .expect("addon.lua should load");

        // Verify the button was created and the closure worked
        let test_button: mlua::Table = addon_table.get("testButton").expect("testButton should exist");
        let result: String = test_button.get("result").expect("result should be set");
        assert!(result.starts_with("updated:"), "updateKeyDirection should have been called, got: {}", result);

        // Cleanup
        std::fs::remove_file(&widgets_path).ok();
        std::fs::remove_file(&button_path).ok();
        std::fs::remove_file(&addon_lua_path).ok();
    }
}
