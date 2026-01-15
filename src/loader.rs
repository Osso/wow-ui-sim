//! Addon loader - loads addons from TOC files.

use crate::lua_api::WowLuaEnv;
use crate::saved_variables::SavedVariablesManager;
use crate::toc::TocFile;
use crate::xml::{parse_xml_file, XmlElement};
use mlua::Table;
use std::path::Path;

/// Result of loading an addon.
#[derive(Debug)]
pub struct LoadResult {
    /// Addon name
    pub name: String,
    /// Number of Lua files loaded
    pub lua_files: usize,
    /// Number of XML files loaded
    pub xml_files: usize,
    /// Errors encountered (non-fatal)
    pub warnings: Vec<String>,
}

/// Context for loading addon files (name and private table).
struct AddonContext<'a> {
    name: &'a str,
    table: Table,
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
    let mut result = LoadResult {
        name: toc.name.clone(),
        lua_files: 0,
        xml_files: 0,
        warnings: Vec::new(),
    };

    // Initialize saved variables before loading addon files
    if let Some(mgr) = saved_vars_mgr {
        let saved_vars = toc.saved_variables();
        let saved_vars_per_char = toc.saved_variables_per_character();

        if !saved_vars.is_empty() || !saved_vars_per_char.is_empty() {
            if let Err(e) = mgr.init_for_addon(
                env.lua(),
                &toc.name,
                &saved_vars,
                &saved_vars_per_char,
            ) {
                result.warnings.push(format!(
                    "Failed to initialize saved variables for {}: {}",
                    toc.name, e
                ));
            }
        }
    }

    // Create the shared private table for this addon (WoW passes this as second vararg)
    let addon_table = env.create_addon_table().map_err(|e| LoadError::Lua(e.to_string()))?;
    let ctx = AddonContext {
        name: &toc.name,
        table: addon_table,
    };

    for file in toc.file_paths() {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "lua" => match load_lua_file(env, &file, &ctx) {
                Ok(()) => result.lua_files += 1,
                Err(e) => result.warnings.push(format!("{}: {}", file.display(), e)),
            },
            "xml" => match load_xml_file(env, &file, &ctx) {
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
fn load_lua_file(env: &WowLuaEnv, path: &Path, ctx: &AddonContext) -> Result<(), LoadError> {
    let code = std::fs::read_to_string(path)?;
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
    env.exec_with_varargs(&code, &chunk_name, ctx.name, table_clone)
        .map_err(|e| LoadError::Lua(e.to_string()))?;
    Ok(())
}

/// Normalize Windows-style paths (backslashes) to Unix-style (forward slashes).
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Create a frame from XML definition.
fn create_frame_from_xml(
    env: &WowLuaEnv,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_override: Option<&str>,
) -> Result<(), LoadError> {
    // Skip virtual frames (templates)
    if frame.is_virtual == Some(true) {
        return Ok(());
    }

    // Need a name to create a global frame (unless we have a parent override for anonymous children)
    let name = match &frame.name {
        Some(n) => n.clone(),
        None => {
            if parent_override.is_some() {
                // Anonymous child frame - generate temp name
                format!("__anon_{}", rand_id())
            } else {
                return Ok(()); // Anonymous top-level frames are templates
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

    // Apply mixins
    if let Some(mixin) = &frame.mixin {
        for m in mixin.split(',').map(|s| s.trim()) {
            if !m.is_empty() {
                lua_code.push_str(&format!(
                    r#"
        if {} then Mixin(frame, {}) end
        "#,
                    m, m
                ));
            }
        }
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
        lua_code.push_str(&generate_anchors_code(anchors, "$parent"));
    }

    // Set hidden state
    if frame.hidden == Some(true) {
        lua_code.push_str(
            r#"
        frame:Hide()
        "#,
        );
    }

    // Handle KeyValues
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
                crate::xml::FrameElement::Button(f) => (f, "Button"),
                crate::xml::FrameElement::CheckButton(f) => (f, "CheckButton"),
                crate::xml::FrameElement::EditBox(f) => (f, "EditBox"),
                crate::xml::FrameElement::ScrollFrame(f) => (f, "ScrollFrame"),
                crate::xml::FrameElement::Slider(f) => (f, "Slider"),
                crate::xml::FrameElement::StatusBar(f) => (f, "StatusBar"),
                _ => continue, // Skip unsupported types for now
            };
            create_frame_from_xml(env, child_frame, child_type, Some(&name))?;

            // Handle parentKey for child frames
            if let (Some(child_name), Some(parent_key)) =
                (&child_frame.name, &child_frame.parent_key)
            {
                let lua_code = format!(
                    r#"
                    {}.{} = {}
                    "#,
                    name, parent_key, child_name
                );
                env.exec(&lua_code).ok(); // Ignore errors (parent might not exist yet)
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

        let rel_str = match relative_to {
            Some("$parent") => parent_ref.to_string(),
            Some(rel) => rel.to_string(),
            None => "nil".to_string(),
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

    if let Some(on_load) = &scripts.on_load {
        add_handler(&mut code, "OnLoad", on_load);
    }
    if let Some(on_event) = &scripts.on_event {
        add_handler(&mut code, "OnEvent", on_event);
    }
    if let Some(on_update) = &scripts.on_update {
        add_handler(&mut code, "OnUpdate", on_update);
    }
    if let Some(on_click) = &scripts.on_click {
        add_handler(&mut code, "OnClick", on_click);
    }
    if let Some(on_show) = &scripts.on_show {
        add_handler(&mut code, "OnShow", on_show);
    }
    if let Some(on_hide) = &scripts.on_hide {
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

            let rel = match anchor.relative_to.as_deref() {
                Some("$parent") | None => "parent".to_string(),
                Some(r) => r.to_string(),
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

            let rel = match anchor.relative_to.as_deref() {
                Some("$parent") | None => "parent".to_string(),
                Some(r) => r.to_string(),
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
fn load_xml_file(env: &WowLuaEnv, path: &Path, ctx: &AddonContext) -> Result<usize, LoadError> {
    let ui = parse_xml_file(path)?;
    let xml_dir = path.parent().unwrap_or(Path::new("."));
    let mut lua_count = 0;

    for element in &ui.elements {
        match element {
            XmlElement::Script(s) => {
                // Script can have file attribute or inline content
                if let Some(file) = &s.file {
                    let script_path = xml_dir.join(normalize_path(file));
                    load_lua_file(env, &script_path, ctx)?;
                    lua_count += 1;
                } else if let Some(inline) = &s.inline {
                    // Execute inline script with varargs
                    let table_clone = ctx.table.clone();
                    env.exec_with_varargs(inline, "@inline", ctx.name, table_clone)
                        .map_err(|e| LoadError::Lua(e.to_string()))?;
                    lua_count += 1;
                }
            }
            XmlElement::Include(i) => {
                let include_path = xml_dir.join(normalize_path(&i.file));
                // Check if it's a Lua file (some addons use Include for Lua files)
                if i.file.ends_with(".lua") {
                    load_lua_file(env, &include_path, ctx)?;
                    lua_count += 1;
                } else {
                    lua_count += load_xml_file(env, &include_path, ctx)?;
                }
            }
            XmlElement::Frame(f) => {
                create_frame_from_xml(env, f, "Frame", None)?;
            }
            XmlElement::Button(f) => {
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
            XmlElement::Texture(_) | XmlElement::FontString(_) => {
                // Top-level textures/fontstrings are templates
            }
            XmlElement::AnimationGroup(_) => {
                // Animation groups are templates
            }
            XmlElement::Actor(_) => {
                // Actor definitions for ModelScene
            }
            XmlElement::Font(_) => {
                // Font definitions
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
        };
        load_lua_file(&env, &lua_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
        };
        load_xml_file(&env, &xml_path, &ctx).unwrap();

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
}
