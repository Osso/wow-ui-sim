//! TOC file parser for WoW addons.
//!
//! Parses `.toc` files to extract addon metadata and file load order.

use crate::paths::find_case_insensitive;
use crate::screen::ScreenKind;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const RETAIL_INTERFACE_VERSION: u32 = crate::client_profile::RETAIL_API_INTERFACE_VERSION;
pub const ACTIVE_INTERFACE_VERSION: u32 = crate::client_profile::ACTIVE_INTERFACE_VERSION;

/// Parsed TOC file contents.
#[derive(Debug, Clone)]
pub struct TocFile {
    /// Addon directory path
    pub addon_dir: PathBuf,
    /// Addon name (from directory or Title metadata)
    pub name: String,
    /// Metadata key-value pairs (## Key: Value)
    pub metadata: HashMap<String, String>,
    /// Files to load in order during normal addon loading (relative paths).
    pub files: Vec<PathBuf>,
    /// Files annotated with `[Bootstrap]`, aligned with `files`.
    /// The annotation does not reorder files or create a separate load pass.
    pub file_is_bootstrap: Vec<bool>,
    /// Per-file environment override from `[LoadIntoEnvironment ...]`.
    /// `None` means inherit the addon's default environment.
    pub file_env_overrides: Vec<Option<bool>>,
    /// Per-file load-pass filter from `[AllowLoadEnvironment ...]`.
    /// `None` means the file can load in any environment pass.
    pub file_env_allows: Vec<Option<bool>>,
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

fn active_game_types() -> &'static [&'static str] {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::Ptr => &["mainline", "standard"],
        crate::client_profile::ClientProfile::Wrath => &["wrath", "wrath_classic", "classic"],
        crate::client_profile::ClientProfile::Mists => &["mists", "mists_classic", "classic"],
        crate::client_profile::ClientProfile::Era => &["vanilla", "classic_era", "classic"],
        crate::client_profile::ClientProfile::Anniversary => {
            &["vanilla", "classic_anniversary", "classic"]
        }
        crate::client_profile::ClientProfile::WowForever => &["camelot", "mainline"],
    }
}

fn matches_active_game_type(types: &str) -> bool {
    types
        .split(|character: char| character == ',' || character.is_whitespace())
        .any(|game_type| active_game_types().contains(&game_type))
}

fn game_type_annotation<'a>(line: &'a str, annotation: &str) -> Option<&'a str> {
    let start = line.find(annotation)? + annotation.len();
    let rest = &line[start..];
    Some(&rest[..rest.find(']')?])
}

/// Apply both inline game-type filters used by the profile's TOC files.
fn is_allowed_game_type(line: &str) -> bool {
    let allowed =
        game_type_annotation(line, "[AllowLoadGameType").is_none_or(matches_active_game_type);
    let excluded =
        game_type_annotation(line, "[ExcludeLoadGameType").is_some_and(matches_active_game_type);
    allowed && !excluded
}

fn is_mists_game_menu_shared_file(addon_dir: &Path, line: &str) -> bool {
    crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Mists
        && addon_dir.file_name().and_then(|name| name.to_str()) == Some("Blizzard_GameMenu")
        && (line.contains("Shared\\GameMenuFrame.") || line.contains("Shared/GameMenuFrame."))
        && line.contains("standard")
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

/// Check if a metadata value consists entirely of unresolved packager template tokens.
///
/// Returns true when every comma-separated token looks like `@something@`.
/// Example: `@toc-version-retail@, @toc-version-cata@` → true.
/// Mixed: `@toc-version-retail@, 110000` → false.
fn is_all_template_versions(value: &str) -> bool {
    if !value.contains("@toc-version-") {
        return false;
    }
    value
        .split(',')
        .map(|v| v.trim())
        .all(|v| v.starts_with('@') && v.ends_with('@'))
}

/// Replace packager template variables in a metadata value.
/// - `@project-version@` → `dev`
fn replace_template_vars(value: &str) -> String {
    value.replace("@project-version@", "dev")
}

fn metadata_key_allows_repeats(key: &str) -> bool {
    matches!(
        key,
        "Dep" | "Dependencies" | "RequiredDep" | "RequiredDeps" | "OptionalDep" | "OptionalDeps"
    )
}

/// Process a `## Key: Value` metadata line into the map.
///
/// Skips `Interface` lines whose value consists entirely of unresolved
/// `@toc-version-*@` packager tokens — the `#@debug@` block in the TOC
/// provides the real fallback version for source-form TOC files.
fn insert_metadata(metadata: &mut HashMap<String, String>, rest: &str) {
    if !is_allowed_game_type(rest) {
        return;
    }
    let Some((key, value)) = rest.split_once(':') else {
        return;
    };
    let key = key.trim();
    let value = value.trim();
    if key == "Interface" && is_all_template_versions(value) {
        return;
    }
    let value = replace_template_vars(strip_annotations(value));
    if metadata_key_allows_repeats(key) {
        metadata
            .entry(key.to_string())
            .and_modify(|existing| {
                existing.push(',');
                existing.push_str(&value);
            })
            .or_insert(value);
    } else {
        metadata.insert(key.to_string(), value);
    }
}

fn parse_load_into_environment(line: &str) -> Option<bool> {
    let lower = line.to_ascii_lowercase();
    if lower.contains("[loadintoenvironment secure]") {
        Some(true)
    } else if lower.contains("[loadintoenvironment global]") {
        Some(false)
    } else {
        None
    }
}

fn parse_allow_load_environment(line: &str) -> Option<bool> {
    let lower = line.to_ascii_lowercase();
    if lower.contains("[allowloadenvironment secure]") {
        Some(true)
    } else if lower.contains("[allowloadenvironment global]") {
        Some(false)
    } else {
        None
    }
}

fn has_bootstrap_annotation(line: &str) -> bool {
    line.to_ascii_lowercase().contains("[bootstrap]")
}

fn split_metadata_list(value: &str) -> Vec<String> {
    if value.contains(',') {
        value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect()
    } else {
        value.split_whitespace().map(ToString::to_string).collect()
    }
}

pub(crate) fn collect_metadata_lists(
    metadata: &HashMap<String, String>,
    keys: &[&str],
) -> Vec<String> {
    keys.iter()
        .filter_map(|key| metadata.get(*key))
        .flat_map(|value| split_metadata_list(value))
        .collect()
}

/// Resolve the `[Family]` TOC substitution per active client profile. Retail
/// and Forever vendors ship a `Mainline/` subdir; mists ship `Classic/`. Wrath
/// FrameXML doesn't use the substitution, so the value there doesn't matter.
fn family_subdir() -> &'static str {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::Ptr
        | crate::client_profile::ClientProfile::WowForever => "Mainline",
        crate::client_profile::ClientProfile::Wrath
        | crate::client_profile::ClientProfile::Mists
        | crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => "Classic",
    }
}

/// Resolve the `[Game]` TOC substitution per active client profile.
fn game_subdir() -> &'static str {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::Ptr => "Standard",
        crate::client_profile::ClientProfile::Wrath => "Wrath",
        crate::client_profile::ClientProfile::Mists => "Mists",
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => "Vanilla",
        crate::client_profile::ClientProfile::WowForever => "Camelot",
    }
}

#[derive(Default)]
struct ParsedFileEntries {
    files: Vec<PathBuf>,
    file_is_bootstrap: Vec<bool>,
    file_env_overrides: Vec<Option<bool>>,
    file_env_allows: Vec<Option<bool>>,
}

impl ParsedFileEntries {
    /// Process a non-metadata, non-comment TOC line as a file path entry.
    fn push_file_entry(&mut self, addon_dir: &Path, line: &str) {
        if line.contains("[AllowLoadTextLocale") && !line.contains("enUS") {
            return;
        }
        if !is_allowed_game_type(line) && !is_mists_game_menu_shared_file(addon_dir, line) {
            return;
        }
        let line = line.replace("[TextLocale]", "enUS");
        let line = line.replace("[Family]", family_subdir());
        let line = line.replace("[Game]", game_subdir());
        let file_path = strip_annotations(&line).replace('\\', "/");
        if file_path.is_empty() {
            return;
        }

        self.file_is_bootstrap.push(has_bootstrap_annotation(&line));
        self.files.push(PathBuf::from(file_path));
        self.file_env_overrides
            .push(parse_load_into_environment(&line));
        self.file_env_allows
            .push(parse_allow_load_environment(&line));
    }
}

impl TocFile {
    /// Parse a TOC file from its contents.
    ///
    /// Handles CurseForge/BigWigs packager template tags in source form:
    /// - `#@debug@` / `#@end-debug@` block markers: skipped as `#` comments;
    ///   inner lines like `## Interface: 120000` are active.
    /// - `## Interface: @toc-version-*@, ...` with only template tokens: skipped
    ///   so the `#@debug@` block entry takes precedence.
    /// - `@project-version@` in any value: replaced with `dev`.
    pub fn parse(addon_dir: &Path, contents: &str) -> Self {
        let mut metadata = HashMap::new();
        let mut file_entries = ParsedFileEntries::default();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix("##") {
                insert_metadata(&mut metadata, rest.trim());
                continue;
            }
            if line.starts_with('#') {
                continue;
            }
            file_entries.push_file_entry(addon_dir, line);
        }

        TocFile {
            addon_dir: addon_dir.to_path_buf(),
            name: resolve_addon_name(&metadata, addon_dir),
            metadata,
            files: file_entries.files,
            file_is_bootstrap: file_entries.file_is_bootstrap,
            file_env_overrides: file_entries.file_env_overrides,
            file_env_allows: file_entries.file_env_allows,
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
            .map(|s| s.split(',').filter_map(|v| v.trim().parse().ok()).collect())
            .unwrap_or_default()
    }

    /// Whether this TOC is compatible with the requested interface version.
    ///
    /// Addons without a parseable Interface line are allowed so source-form
    /// test fixtures and simulator helper addons can still load.
    pub fn supports_interface_version(&self, interface_version: u32) -> bool {
        let versions = self.interface_versions();
        versions.is_empty() || versions.contains(&interface_version)
    }

    /// Get required dependencies.
    ///
    /// WoW TOC files use variant keys including Blizzard's repeated `Dep`.
    pub fn dependencies(&self) -> Vec<String> {
        collect_metadata_lists(
            &self.metadata,
            &["Dep", "RequiredDep", "Dependencies", "RequiredDeps"],
        )
    }

    /// Get `LoadWith` triggers — addon names that, when loaded, should trigger
    /// loading this addon immediately inline.
    pub fn load_with(&self) -> Vec<String> {
        self.metadata
            .get("LoadWith")
            .map(|s| split_metadata_list(s))
            .unwrap_or_default()
    }

    /// Get optional dependencies.
    pub fn optional_deps(&self) -> Vec<String> {
        collect_metadata_lists(&self.metadata, &["OptionalDep", "OptionalDeps"])
    }

    /// Check if addon uses the secure Lua environment (UseSecureEnvironment: 1).
    pub fn is_secure_env(&self) -> bool {
        self.metadata
            .get("UseSecureEnvironment")
            .map(|v| v == "1")
            .unwrap_or(false)
    }

    /// Whether a TOC file entry is annotated with `[Bootstrap]`.
    pub fn file_is_bootstrap(&self, index: usize) -> bool {
        self.file_is_bootstrap.get(index).copied().unwrap_or(false)
    }

    /// Whether this TOC has any entries annotated with `[Bootstrap]`.
    pub fn has_bootstrap_files(&self) -> bool {
        self.file_is_bootstrap
            .iter()
            .any(|is_bootstrap| *is_bootstrap)
    }

    /// Get files annotated with `[Bootstrap]` in normal TOC order.
    pub fn bootstrap_files(&self) -> Vec<&PathBuf> {
        self.files
            .iter()
            .enumerate()
            .filter_map(|(index, file)| self.file_is_bootstrap(index).then_some(file))
            .collect()
    }

    /// Get the per-file environment override for a TOC entry.
    pub fn file_use_secure_env(&self, index: usize) -> Option<bool> {
        self.file_env_overrides.get(index).copied().flatten()
    }

    /// Get the environment pass filter for a TOC entry.
    pub fn file_allow_load_environment(&self, index: usize) -> Option<bool> {
        self.file_env_allows.get(index).copied().flatten()
    }

    /// Whether a file should load in the requested environment pass.
    pub fn file_allows_environment(&self, index: usize, use_secure_env: bool) -> bool {
        self.file_allow_load_environment(index)
            .map(|allowed_secure_env| allowed_secure_env == use_secure_env)
            .unwrap_or(true)
    }

    /// Default enabled state — `## DefaultState: disabled` ships the addon
    /// disabled out of the box; any other value (or absence) ships it enabled.
    pub fn default_enabled(&self) -> bool {
        self.metadata
            .get("DefaultState")
            .map(|v| !v.eq_ignore_ascii_case("disabled"))
            .unwrap_or(true)
    }

    /// Check if addon is load-on-demand.
    pub fn is_load_on_demand(&self) -> bool {
        self.metadata
            .get("LoadOnDemand")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    /// Check if addon requests early loading via `## LoadFirst: 1`.
    pub fn is_load_first(&self) -> bool {
        self.metadata
            .get("LoadFirst")
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

    /// Check if addon is PTR/Beta-only and should not load on this profile.
    /// These addons have `OnlyBetaAndPTR: 1` and should not load on live clients.
    pub fn is_ptr_only(&self) -> bool {
        self.metadata
            .get("OnlyBetaAndPTR")
            .map(|v| v == "1")
            .unwrap_or(false)
            && !matches!(
                crate::client_profile::ACTIVE,
                crate::client_profile::ClientProfile::Ptr
                    | crate::client_profile::ClientProfile::WowForever
            )
    }

    /// Check if addon is restricted to a game type incompatible with the active client profile.
    /// Tocs with `AllowLoadGameType: <type>` only load when that type matches the active profile.
    pub fn is_game_type_restricted(&self) -> bool {
        if self.is_mists_legacy_craft_ui_toc() {
            return false;
        }

        let excluded_by_allow = self
            .metadata
            .get("AllowLoadGameType")
            .is_some_and(|value| !matches_active_game_type(value));
        let explicitly_excluded = self
            .metadata
            .get("ExcludeLoadGameType")
            .is_some_and(|value| matches_active_game_type(value));
        excluded_by_allow || explicitly_excluded
    }

    fn is_mists_legacy_craft_ui_toc(&self) -> bool {
        crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Mists
            && self.folder_name() == Some("Blizzard_CraftUI")
            && self
                .metadata
                .get("AllowLoadGameType")
                .is_some_and(|value| value.split(',').any(|token| token.trim() == "wrath"))
    }

    /// Whether this addon should load for the requested screen kind.
    pub fn allows_screen(&self, screen: ScreenKind) -> bool {
        match self.metadata.get("AllowLoad").map(|v| v.trim()) {
            Some(v) if v.eq_ignore_ascii_case("both") => true,
            Some(v) if v.eq_ignore_ascii_case("game") => screen == ScreenKind::Game,
            Some(v) if v.eq_ignore_ascii_case("glue") => screen.is_glue(),
            Some(_) => screen == ScreenKind::Game,
            None => screen == ScreenKind::Game,
        }
    }

    /// Get saved variables names (account-wide + machine-specific).
    pub fn saved_variables(&self) -> Vec<String> {
        let mut vars: Vec<String> = Vec::new();
        for key in ["SavedVariables", "SavedVariablesMachine"] {
            if let Some(s) = self.metadata.get(key) {
                vars.extend(split_metadata_list(s));
            }
        }
        vars
    }

    /// Get saved variables per character names.
    pub fn saved_variables_per_character(&self) -> Vec<String> {
        self.metadata
            .get("SavedVariablesPerCharacter")
            .map(|s| split_metadata_list(s))
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

impl TocFile {
    /// Check if this is a Blizzard addon (AllowLoad metadata present).
    pub fn is_blizzard_addon(&self) -> bool {
        self.metadata.contains_key("AllowLoad")
    }

    /// Check if this TOC should execute without addon taint.
    ///
    /// Most Blizzard UI TOCs advertise `AllowLoad`; a few secure helper TOCs
    /// only advertise `UseSecureEnvironment`; other internal Blizzard addons
    /// rely on the signed `Blizzard_` folder-name convention that also drives
    /// `C_AddOns.GetAddOnSecurity`.
    pub fn loads_as_blizzard_code(&self) -> bool {
        self.is_blizzard_addon() || self.is_secure_env() || self.folder_name_starts_with_blizzard()
    }

    fn folder_name_starts_with_blizzard(&self) -> bool {
        self.folder_name()
            .is_some_and(|name| name.starts_with("Blizzard_"))
    }

    fn folder_name(&self) -> Option<&str> {
        self.addon_dir.file_name().and_then(|name| name.to_str())
    }
}

#[cfg(test)]
mod tests;
