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

impl TocFile {
    /// Parse a TOC file from its contents.
    pub fn parse(addon_dir: &Path, contents: &str) -> Self {
        let mut metadata = HashMap::new();
        let mut files = Vec::new();

        for line in contents.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Metadata: ## Key: Value
            if let Some(rest) = line.strip_prefix("##") {
                let rest = rest.trim();
                if let Some((key, value)) = rest.split_once(':') {
                    metadata.insert(key.trim().to_string(), value.trim().to_string());
                }
                continue;
            }

            // Comments: # anything (including #@directives@)
            if line.starts_with('#') {
                continue;
            }

            // File path - normalize path separators and strip inline annotations
            // Blizzard TOC files use [...] annotations like [AllowLoadEnvironment Global]
            let file_path = if let Some(bracket_pos) = line.find('[') {
                line[..bracket_pos].trim()
            } else {
                line
            };
            let file_path = file_path.replace('\\', "/");
            if !file_path.is_empty() {
                files.push(PathBuf::from(file_path));
            }
        }

        // Get name from Title metadata or directory name
        let name = metadata
            .get("Title")
            .cloned()
            .unwrap_or_else(|| {
                addon_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            });

        TocFile {
            addon_dir: addon_dir.to_path_buf(),
            name,
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
    pub fn dependencies(&self) -> Vec<String> {
        self.metadata
            .get("Dependencies")
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

    /// Get saved variables names (account-wide).
    pub fn saved_variables(&self) -> Vec<String> {
        self.metadata
            .get("SavedVariables")
            .map(|s| {
                s.split(',')
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                    .collect()
            })
            .unwrap_or_default()
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
    pub fn file_paths(&self) -> Vec<PathBuf> {
        self.files
            .iter()
            .map(|f| self.addon_dir.join(f))
            .collect()
    }

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
}
