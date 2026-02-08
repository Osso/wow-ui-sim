//! TOC file parser for WoW addons.
//!
//! Parses `.toc` files to extract addon metadata and file load order.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Parsed TOC file contents.
#[derive(Debug, Clone)]
pub struct TocFile {
    /// Addon directory path
    pub addon_dir: PathBuf,
    /// Addon name (from directory or Title metadata)
    pub name: String,
    /// Metadata key-value pairs (## Key: Value)
    pub metadata: HashMap<String, String>,
    /// Files to load in order (relative paths)
    pub files: Vec<PathBuf>,
}

/// Strip inline annotations like `[AllowLoadEnvironment Global]` from a TOC line.
fn strip_annotations(line: &str) -> &str {
    if let Some(pos) = line.find(" [") {
        line[..pos].trim()
    } else if line.ends_with(']') {
        if let Some(pos) = line.find('[') {
            line[..pos].trim()
        } else {
            line.trim()
        }
    } else {
        line.trim()
    }
}

/// Check if an inline `[AllowLoadGameType ...]` annotation includes a game type
/// compatible with mainline retail WoW (standard mode).
fn is_allowed_game_type(line: &str) -> bool {
    let Some(start) = line.find("[AllowLoadGameType") else {
        return true;
    };
    let rest = &line[start + "[AllowLoadGameType".len()..];
    let Some(end) = rest.find(']') else {
        return true;
    };
    let types = &rest[..end];
    types
        .split(',')
        .any(|t| matches!(t.trim(), "mainline" | "standard"))
}

/// Resolve addon name from Title metadata or directory name.
fn resolve_addon_name(metadata: &HashMap<String, String>, addon_dir: &Path) -> String {
    metadata.get("Title").cloned().unwrap_or_else(|| {
        addon_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    })
}

impl TocFile {
    /// Parse a TOC file from its contents.
    pub fn parse(addon_dir: &Path, contents: &str) -> Self {
        let mut metadata = HashMap::new();
        let mut files = Vec::new();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(rest) = line.strip_prefix("##") {
                let rest = rest.trim();
                if let Some((key, value)) = rest.split_once(':') {
                    metadata.insert(key.trim().to_string(), value.trim().to_string());
                }
                continue;
            }

            if line.starts_with('#') {
                continue;
            }

            // Skip locale-restricted files that don't include enUS
            if line.contains("[AllowLoadTextLocale") && !line.contains("enUS") {
                continue;
            }

            // Skip game-type-restricted files that aren't for mainline/standard retail
            if line.contains("[AllowLoadGameType") && !is_allowed_game_type(line) {
                continue;
            }

            // Replace placeholders and strip annotations
            let line = line.replace("[TextLocale]", "enUS");
            let line = line.replace("[Family]", "Mainline");
            let line = line.replace("[Game]", "Standard");
            let file_path = strip_annotations(&line).replace('\\', "/");
            if !file_path.is_empty() {
                files.push(PathBuf::from(file_path));
            }
        }

        TocFile {
            addon_dir: addon_dir.to_path_buf(),
            name: resolve_addon_name(&metadata, addon_dir),
            metadata,
            files,
        }
    }

    /// Parse a TOC file from disk.
    pub fn from_file(toc_path: &Path) -> std::io::Result<Self> {
        let contents = std::fs::read_to_string(toc_path)?;
        let addon_dir = toc_path.parent().unwrap_or(Path::new("."));
        Ok(Self::parse(addon_dir, &contents))
    }

    /// Get interface version(s) from metadata.
    pub fn interface_versions(&self) -> Vec<u32> {
        self.metadata
            .get("Interface")
            .map(|s| {
                s.split(',')
                    .filter_map(|v| v.trim().parse().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get required dependencies.
    ///
    /// WoW TOC files use three variant keys: `RequiredDep`, `RequiredDeps`, `Dependencies`.
    pub fn dependencies(&self) -> Vec<String> {
        self.metadata
            .get("RequiredDep")
            .or_else(|| self.metadata.get("Dependencies"))
            .or_else(|| self.metadata.get("RequiredDeps"))
            .map(|s| {
                s.split(',')
                    .map(|d| d.trim().to_string())
                    .filter(|d| !d.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get optional dependencies.
    pub fn optional_deps(&self) -> Vec<String> {
        self.metadata
            .get("OptionalDeps")
            .map(|s| {
                s.split(',')
                    .map(|d| d.trim().to_string())
                    .filter(|d| !d.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if addon is load-on-demand.
    pub fn is_load_on_demand(&self) -> bool {
        self.metadata
            .get("LoadOnDemand")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    /// Check if addon is glue-only (login/character-select screen).
    /// These addons have `AllowLoad: Glue` and should not load in game mode.
    pub fn is_glue_only(&self) -> bool {
        self.metadata
            .get("AllowLoad")
            .map(|v| v.eq_ignore_ascii_case("glue"))
            .unwrap_or(false)
    }

    /// Check if addon is PTR/Beta-only (e.g. Blizzard_PTRFeedback).
    /// These addons have `OnlyBetaAndPTR: 1` and should not load on live clients.
    pub fn is_ptr_only(&self) -> bool {
        self.metadata
            .get("OnlyBetaAndPTR")
            .map(|v| v == "1")
            .unwrap_or(false)
    }

    /// Check if addon is restricted to a non-mainline game type (e.g. plunderstorm, classic).
    /// These addons have `AllowLoadGameType: <type>` and should only load in that mode.
    pub fn is_game_type_restricted(&self) -> bool {
        self.metadata
            .get("AllowLoadGameType")
            .map(|v| {
                !v.split(',')
                    .any(|t| matches!(t.trim(), "mainline" | "standard"))
            })
            .unwrap_or(false)
    }

    /// Get saved variables names (account-wide + machine-specific).
    pub fn saved_variables(&self) -> Vec<String> {
        let mut vars: Vec<String> = Vec::new();
        for key in ["SavedVariables", "SavedVariablesMachine"] {
            if let Some(s) = self.metadata.get(key) {
                vars.extend(
                    s.split(',')
                        .map(|v| v.trim().to_string())
                        .filter(|v| !v.is_empty()),
                );
            }
        }
        vars
    }

    /// Get saved variables per character names.
    pub fn saved_variables_per_character(&self) -> Vec<String> {
        self.metadata
            .get("SavedVariablesPerCharacter")
            .map(|s| {
                s.split(',')
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get absolute paths for all files to load.
    /// Uses case-insensitive matching for compatibility with WoW (Windows/macOS).
    pub fn file_paths(&self) -> Vec<PathBuf> {
        self.files
            .iter()
            .map(|f| resolve_path_case_insensitive(&self.addon_dir, f))
            .collect()
    }
}

/// Resolve a path with case-insensitive matching (WoW is case-insensitive on Windows/macOS).
fn resolve_path_case_insensitive(base: &Path, path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy().replace('\\', "/");
    let components: Vec<&str> = path_str.split('/').collect();
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
            // Fall back to exact path (will fail later with proper error)
            current = exact;
        }
    }
    current
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

impl TocFile {
    /// Check if this is a Blizzard addon (AllowLoad metadata present).
    pub fn is_blizzard_addon(&self) -> bool {
        self.metadata.contains_key("AllowLoad")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_toc() {
        let contents = r#"
## Title: MyAddon
## Interface: 110000
## Dependencies: Ace3, LibStub

Core.lua
UI/Main.lua
UI/Options.xml
"#;
        let toc = TocFile::parse(Path::new("/addons/MyAddon"), contents);

        assert_eq!(toc.name, "MyAddon");
        assert_eq!(toc.interface_versions(), vec![110000]);
        assert_eq!(toc.dependencies(), vec!["Ace3", "LibStub"]);
        assert_eq!(toc.files.len(), 3);
        assert_eq!(toc.files[0], PathBuf::from("Core.lua"));
        assert_eq!(toc.files[1], PathBuf::from("UI/Main.lua"));
        assert_eq!(toc.files[2], PathBuf::from("UI/Options.xml"));
    }

    #[test]
    fn test_parse_blizzard_toc() {
        let contents = r#"
## Title: Blizzard_SharedXMLBase
## AllowLoad: Both
Compat.lua
Mixin.lua
TableUtil.lua
"#;
        let toc = TocFile::parse(Path::new("/Interface/AddOns/Blizzard_SharedXMLBase"), contents);

        assert_eq!(toc.name, "Blizzard_SharedXMLBase");
        assert!(toc.is_blizzard_addon());
        assert_eq!(toc.files.len(), 3);
    }

    #[test]
    fn test_parse_with_comments() {
        let contents = r#"
## Title: TestAddon
# This is a comment
#@no-lib-strip@
Libs/LibStub.lua
#@end-no-lib-strip@
Core.lua
"#;
        let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

        // Comments and directives should be skipped
        assert_eq!(toc.files.len(), 2);
        assert_eq!(toc.files[0], PathBuf::from("Libs/LibStub.lua"));
        assert_eq!(toc.files[1], PathBuf::from("Core.lua"));
    }

    #[test]
    fn test_parse_backslash_paths() {
        let contents = r#"
## Title: TestAddon
Libs\LibStub\LibStub.lua
Core\Init.lua
"#;
        let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

        // Backslashes should be normalized to forward slashes
        assert_eq!(toc.files[0], PathBuf::from("Libs/LibStub/LibStub.lua"));
        assert_eq!(toc.files[1], PathBuf::from("Core/Init.lua"));
    }

    #[test]
    fn test_optional_deps() {
        let contents = r#"
## Title: TestAddon
## OptionalDeps: Ace3, LibDBIcon-1.0, LibSharedMedia-3.0
Core.lua
"#;
        let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

        assert_eq!(
            toc.optional_deps(),
            vec!["Ace3", "LibDBIcon-1.0", "LibSharedMedia-3.0"]
        );
    }

    #[test]
    fn test_saved_variables() {
        let contents = r#"
## Title: TestAddon
## SavedVariables: TestAddonDB, TestAddonPerCharDB
Core.lua
"#;
        let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

        assert_eq!(
            toc.saved_variables(),
            vec!["TestAddonDB", "TestAddonPerCharDB"]
        );
    }

    #[test]
    fn test_multiple_interface_versions() {
        let contents = r#"
## Title: TestAddon
## Interface: 110107, 50500, 11507
Core.lua
"#;
        let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

        assert_eq!(toc.interface_versions(), vec![110107, 50500, 11507]);
    }

    #[test]
    fn test_parse_inline_annotations() {
        let contents = r#"
## Title: TestAddon
Core.lua
Dump.lua [AllowLoadEnvironment Global]
Debug.lua [AllowLoadEnvironment Global, SomeFlag]
"#;
        let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

        // Annotations should be stripped, only filenames kept
        assert_eq!(toc.files.len(), 3);
        assert_eq!(toc.files[0], PathBuf::from("Core.lua"));
        assert_eq!(toc.files[1], PathBuf::from("Dump.lua"));
        assert_eq!(toc.files[2], PathBuf::from("Debug.lua"));
    }

    #[test]
    fn test_family_placeholder_resolves_to_mainline() {
        let contents = r#"
## Title: Blizzard_Colors
Shared\ColorOverrides.lua
[Family]\ColorConstants.lua
[Family]\ColorManager.lua
"#;
        let toc = TocFile::parse(Path::new("/addons/Blizzard_Colors"), contents);

        assert_eq!(toc.files.len(), 3);
        assert_eq!(toc.files[0], PathBuf::from("Shared/ColorOverrides.lua"));
        assert_eq!(toc.files[1], PathBuf::from("Mainline/ColorConstants.lua"));
        assert_eq!(toc.files[2], PathBuf::from("Mainline/ColorManager.lua"));
    }

    #[test]
    fn test_game_type_filter_skips_plunderstorm() {
        let contents = r#"
## Title: Blizzard_FrameXMLBase
Constants.lua
[Game]\GameModeConstants.lua [AllowLoadGameType plunderstorm]
"#;
        let toc = TocFile::parse(Path::new("/addons/Blizzard_FrameXMLBase"), contents);

        assert_eq!(toc.files.len(), 1);
        assert_eq!(toc.files[0], PathBuf::from("Constants.lua"));
    }

    #[test]
    fn test_game_type_filter_allows_mainline_and_standard() {
        let contents = r#"
## Title: TestAddon
Core.lua
Mainline\Override.lua [AllowLoadGameType mainline]
Standard\Mode.lua [AllowLoadGameType standard]
Standard\Multi.lua [AllowLoadGameType standard, wowhack, plunderstorm]
WoWLabs\Mode.lua [AllowLoadGameType plunderstorm]
Classic\Mode.lua [AllowLoadGameType classic]
Cata\Mode.lua [AllowLoadGameType wrath, cata, mists]
"#;
        let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

        assert_eq!(toc.files.len(), 4);
        assert_eq!(toc.files[0], PathBuf::from("Core.lua"));
        assert_eq!(toc.files[1], PathBuf::from("Mainline/Override.lua"));
        assert_eq!(toc.files[2], PathBuf::from("Standard/Mode.lua"));
        assert_eq!(toc.files[3], PathBuf::from("Standard/Multi.lua"));
    }

    #[test]
    fn test_is_allowed_game_type() {
        assert!(is_allowed_game_type("Core.lua"));
        assert!(is_allowed_game_type("File.lua [AllowLoadGameType mainline]"));
        assert!(is_allowed_game_type("File.lua [AllowLoadGameType standard]"));
        assert!(is_allowed_game_type(
            "File.lua [AllowLoadGameType standard, wowhack]"
        ));
        assert!(!is_allowed_game_type(
            "File.lua [AllowLoadGameType plunderstorm]"
        ));
        assert!(!is_allowed_game_type("File.lua [AllowLoadGameType classic]"));
        assert!(!is_allowed_game_type(
            "File.lua [AllowLoadGameType wrath, cata, mists]"
        ));
    }

    #[test]
    fn test_is_game_type_restricted() {
        let plunderstorm = TocFile::parse(
            Path::new("/addons/Test"),
            "## AllowLoadGameType: plunderstorm\nCore.lua",
        );
        assert!(plunderstorm.is_game_type_restricted());

        let mainline = TocFile::parse(
            Path::new("/addons/Test"),
            "## AllowLoadGameType: mainline\nCore.lua",
        );
        assert!(!mainline.is_game_type_restricted());

        let standard = TocFile::parse(
            Path::new("/addons/Test"),
            "## AllowLoadGameType: standard\nCore.lua",
        );
        assert!(!standard.is_game_type_restricted());

        let mixed = TocFile::parse(
            Path::new("/addons/Test"),
            "## AllowLoadGameType: plunderstorm, wowhack\nCore.lua",
        );
        assert!(mixed.is_game_type_restricted());

        let no_restriction = TocFile::parse(
            Path::new("/addons/Test"),
            "## Title: TestAddon\nCore.lua",
        );
        assert!(!no_restriction.is_game_type_restricted());
    }
}
