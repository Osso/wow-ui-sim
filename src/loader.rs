//! Addon loader - loads addons from TOC files.

use crate::lua_api::WowLuaEnv;
use crate::toc::TocFile;
use crate::xml::{parse_xml_file, XmlElement};
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

/// Load an addon from its TOC file.
pub fn load_addon(env: &WowLuaEnv, toc_path: &Path) -> Result<LoadResult, LoadError> {
    let toc = TocFile::from_file(toc_path)?;
    load_addon_from_toc(env, &toc)
}

/// Load an addon from a parsed TOC.
pub fn load_addon_from_toc(env: &WowLuaEnv, toc: &TocFile) -> Result<LoadResult, LoadError> {
    let mut result = LoadResult {
        name: toc.name.clone(),
        lua_files: 0,
        xml_files: 0,
        warnings: Vec::new(),
    };

    for file in toc.file_paths() {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "lua" => {
                match load_lua_file(env, &file) {
                    Ok(()) => result.lua_files += 1,
                    Err(e) => result.warnings.push(format!("{}: {}", file.display(), e)),
                }
            }
            "xml" => {
                match load_xml_file(env, &file) {
                    Ok(count) => {
                        result.xml_files += 1;
                        result.lua_files += count;
                    }
                    Err(e) => result.warnings.push(format!("{}: {}", file.display(), e)),
                }
            }
            _ => {
                result.warnings.push(format!("{}: unknown file type", file.display()));
            }
        }
    }

    Ok(result)
}

/// Load a Lua file into the environment.
fn load_lua_file(env: &WowLuaEnv, path: &Path) -> Result<(), LoadError> {
    let code = std::fs::read_to_string(path)?;
    env.exec(&code).map_err(|e| LoadError::Lua(e.to_string()))?;
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
) -> Result<(), LoadError> {
    // Skip virtual frames (templates)
    if frame.is_virtual == Some(true) {
        return Ok(());
    }

    // Need a name to create a global frame
    let name = match &frame.name {
        Some(n) => n,
        None => return Ok(()), // Anonymous frames in XML are usually children
    };

    // Build the Lua code to create and configure the frame
    let parent = frame.parent.as_deref().unwrap_or("UIParent");
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
        if let (Some(x), Some(y)) = (size.x, size.y) {
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
        for anchor in &anchors.anchors {
            let point = &anchor.point;
            let relative_to = anchor.relative_to.as_deref();
            let relative_point = anchor.relative_point.as_deref().unwrap_or(point.as_str());
            let x = anchor.x.unwrap_or(0.0);
            let y = anchor.y.unwrap_or(0.0);

            if let Some(rel) = relative_to {
                lua_code.push_str(&format!(
                    r#"
        frame:SetPoint("{}", {}, "{}", {}, {})
        "#,
                    point, rel, relative_point, x, y
                ));
            } else {
                lua_code.push_str(&format!(
                    r#"
        frame:SetPoint("{}", nil, "{}", {}, {})
        "#,
                    point, relative_point, x, y
                ));
            }
        }
    }

    // Set hidden state
    if frame.hidden == Some(true) {
        lua_code.push_str(
            r#"
        frame:Hide()
        "#,
        );
    }

    // Execute the creation code
    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!("Failed to create frame {}: {}", name, e))
    })?;

    Ok(())
}

/// Load an XML file, processing its elements.
/// Returns the number of Lua files loaded from Script elements.
fn load_xml_file(env: &WowLuaEnv, path: &Path) -> Result<usize, LoadError> {
    let ui = parse_xml_file(path)?;
    let xml_dir = path.parent().unwrap_or(Path::new("."));
    let mut lua_count = 0;

    for element in &ui.elements {
        match element {
            XmlElement::Script(s) => {
                // Script can have file attribute or inline content
                if let Some(file) = &s.file {
                    let script_path = xml_dir.join(normalize_path(file));
                    load_lua_file(env, &script_path)?;
                    lua_count += 1;
                } else if let Some(inline) = &s.inline {
                    // Execute inline script
                    env.exec(inline).map_err(|e| LoadError::Lua(e.to_string()))?;
                    lua_count += 1;
                }
            }
            XmlElement::Include(i) => {
                let include_path = xml_dir.join(normalize_path(&i.file));
                lua_count += load_xml_file(env, &include_path)?;
            }
            XmlElement::Frame(f) => {
                create_frame_from_xml(env, f, "Frame")?;
            }
            XmlElement::Button(f) => {
                create_frame_from_xml(env, f, "Button")?;
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
            XmlElement::CheckButton(_)
            | XmlElement::EditBox(_)
            | XmlElement::ScrollFrame(_)
            | XmlElement::Slider(_)
            | XmlElement::StatusBar(_)
            | XmlElement::GameTooltip(_)
            | XmlElement::ColorSelect(_)
            | XmlElement::Model(_)
            | XmlElement::ModelScene(_)
            | XmlElement::EventFrame(_)
            | XmlElement::CinematicModel(_)
            | XmlElement::PlayerModel(_)
            | XmlElement::DressUpModel(_)
            | XmlElement::Browser(_)
            | XmlElement::Minimap(_)
            | XmlElement::MessageFrame(_)
            | XmlElement::MovieFrame(_)
            | XmlElement::ScrollingMessageFrame(_)
            | XmlElement::SimpleHTML(_)
            | XmlElement::WorldFrame(_)
            | XmlElement::DropDownToggleButton(_)
            | XmlElement::DropdownButton(_)
            | XmlElement::EventButton(_)
            | XmlElement::EventEditBox(_)
            | XmlElement::Cooldown(_) => {
                // All frame-like widgets
            }
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

        load_lua_file(&env, &lua_path).unwrap();

        let value: i32 = env.eval("return TEST_VAR").unwrap();
        assert_eq!(value, 42);

        // Cleanup
        std::fs::remove_file(&lua_path).ok();
    }
}
