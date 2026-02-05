//! Font management using cosmic-text.
//!
//! Loads WoW TTF fonts into a cosmic-text FontSystem for text shaping,
//! measurement, and glyph rasterization. Provides mapping from WoW font
//! paths (e.g. `Fonts\\FRIZQT__.TTF`) to fontdb family names.

use std::collections::HashMap;
use std::path::Path;

use cosmic_text::fontdb;

/// WoW font path constants (as they appear in Lua/XML).
const WOW_FONT_FRIZ: &str = "Fonts\\FRIZQT__.TTF";
const WOW_FONT_ARIAL_NARROW: &str = "Fonts\\ARIALN.TTF";

/// Default WoW font (Friz Quadrata).
pub const DEFAULT_WOW_FONT: &str = WOW_FONT_FRIZ;

/// Font entry mapping a WoW path to a fontdb family name.
#[derive(Debug, Clone)]
struct FontEntry {
    family: String,
}

/// Manages WoW fonts via cosmic-text.
///
/// Holds a `FontSystem` with only the WoW TTF fonts loaded (no system fonts),
/// a `SwashCache` for glyph rasterization, and a mapping from WoW font paths
/// to fontdb family names.
pub struct WowFontSystem {
    pub font_system: cosmic_text::FontSystem,
    pub swash_cache: cosmic_text::SwashCache,
    /// Map from normalized WoW font path (uppercase) to family name.
    font_map: HashMap<String, FontEntry>,
}

impl std::fmt::Debug for WowFontSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WowFontSystem")
            .field("fonts", &self.font_map.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl WowFontSystem {
    /// Create a new font system with WoW fonts loaded from the given directory.
    ///
    /// `fonts_dir` should point to the `fonts/` directory containing TTF files.
    pub fn new(fonts_dir: &Path) -> Self {
        let mut db = fontdb::Database::new();
        let mut font_map = HashMap::new();

        // Load each TTF file and record its family name
        let font_files = [
            ("FRIZQT__.TTF", &[WOW_FONT_FRIZ, "Fonts\\frizqt__.ttf"][..]),
            ("ARIALN.ttf", &[WOW_FONT_ARIAL_NARROW, "Fonts\\arialn.ttf"][..]),
            ("frizqt___cyr.ttf", &["Fonts\\frizqt___cyr.ttf"][..]),
            ("TrajanPro3SemiBold.ttf", &["Fonts\\TrajanPro3SemiBold.ttf"][..]),
        ];

        for (filename, wow_paths) in &font_files {
            let path = fonts_dir.join(filename);
            if !path.exists() {
                tracing::warn!("Font file not found: {}", path.display());
                continue;
            }

            let data = match std::fs::read(&path) {
                Ok(d) => d,
                Err(e) => {
                    tracing::warn!("Failed to read font {}: {}", path.display(), e);
                    continue;
                }
            };

            // Query the family name before loading into fontdb
            let family_name = fontdb_family_name(&data).unwrap_or_else(|| filename.to_string());

            db.load_font_data(data);

            let entry = FontEntry {
                family: family_name.clone(),
            };

            for wow_path in *wow_paths {
                let key = normalize_wow_path(wow_path);
                font_map.insert(key, entry.clone());
            }

            tracing::debug!("Loaded font {} -> family '{}'", filename, family_name);
        }

        let font_system =
            cosmic_text::FontSystem::new_with_locale_and_db("en-US".to_string(), db);
        let swash_cache = cosmic_text::SwashCache::new();

        Self {
            font_system,
            swash_cache,
            font_map,
        }
    }

    /// Get the fontdb family name for a WoW font path.
    ///
    /// Falls back to the default WoW font (Friz Quadrata) if the path is
    /// unknown. Returns None only if no fonts were loaded at all.
    pub fn family_name(&self, wow_path: Option<&str>) -> Option<&str> {
        let key = normalize_wow_path(wow_path.unwrap_or(DEFAULT_WOW_FONT));
        if let Some(entry) = self.font_map.get(&key) {
            return Some(&entry.family);
        }
        // Fall back to default WoW font
        let default_key = normalize_wow_path(DEFAULT_WOW_FONT);
        self.font_map.get(&default_key).map(|e| e.family.as_str())
    }

    /// Create cosmic-text `Attrs` for a WoW font path.
    ///
    /// The returned `Attrs` borrows the family name from `self`, so it cannot
    /// be held across a `&mut self` call. For use with `Buffer::set_text`,
    /// prefer `attrs_owned()` which returns an `AttrsOwned`.
    pub fn attrs(&self, wow_path: Option<&str>) -> cosmic_text::Attrs<'_> {
        match self.family_name(wow_path) {
            Some(name) => cosmic_text::Attrs::new().family(cosmic_text::Family::Name(name)),
            None => cosmic_text::Attrs::new(),
        }
    }

    /// Create an owned `AttrsOwned` for a WoW font path.
    ///
    /// Use this when you need to pass attrs to functions that also take
    /// `&mut font_system`, since `AttrsOwned` doesn't borrow from self.
    pub fn attrs_owned(&self, wow_path: Option<&str>) -> cosmic_text::AttrsOwned {
        cosmic_text::AttrsOwned::new(&self.attrs(wow_path))
    }
}

/// Normalize a WoW font path to uppercase with forward slashes for map lookup.
fn normalize_wow_path(path: &str) -> String {
    path.replace('/', "\\").to_uppercase()
}

/// Extract the font family name from raw TTF data using fontdb.
fn fontdb_family_name(data: &[u8]) -> Option<String> {
    // Parse the font to get its family name
    let mut tmp_db = fontdb::Database::new();
    tmp_db.load_font_data(data.to_vec());
    tmp_db
        .faces()
        .next()
        .map(|face| face.families[0].0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fonts_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts")
    }

    #[test]
    fn loads_wow_fonts() {
        let fs = WowFontSystem::new(&fonts_dir());
        // 4 font files loaded: FRIZQT__, ARIALN, frizqt___cyr, TrajanPro3SemiBold
        // Case-insensitive aliases collapse to 4 unique normalized keys
        assert_eq!(fs.font_map.len(), 4, "font_map: {:?}", fs.font_map.keys().collect::<Vec<_>>());
    }

    #[test]
    fn resolves_friz_quadrata() {
        let fs = WowFontSystem::new(&fonts_dir());
        let name = fs.family_name(Some("Fonts\\FRIZQT__.TTF")).unwrap();
        assert!(
            name.contains("Friz") || name.contains("Fritz"),
            "Unexpected family name: {name}"
        );
    }

    #[test]
    fn resolves_case_insensitive() {
        let fs = WowFontSystem::new(&fonts_dir());
        let upper = fs.family_name(Some("Fonts\\FRIZQT__.TTF"));
        let lower = fs.family_name(Some("Fonts\\frizqt__.ttf"));
        let mixed = fs.family_name(Some("fonts\\FrizQT__.TTF"));
        assert_eq!(upper, lower);
        assert_eq!(upper, mixed);
    }

    #[test]
    fn unknown_font_falls_back_to_default() {
        let fs = WowFontSystem::new(&fonts_dir());
        let name = fs.family_name(Some("Fonts\\NONEXISTENT.TTF")).unwrap();
        assert!(
            name.contains("Friz") || name.contains("Fritz"),
            "Expected Friz Quadrata fallback, got: {name}"
        );
    }

    #[test]
    fn none_font_uses_default() {
        let fs = WowFontSystem::new(&fonts_dir());
        let name = fs.family_name(None).unwrap();
        assert!(
            name.contains("Friz") || name.contains("Fritz"),
            "Expected Friz Quadrata for None, got: {name}"
        );
    }

    #[test]
    fn can_shape_text_with_loaded_font() {
        let mut fs = WowFontSystem::new(&fonts_dir());
        let attrs = fs.attrs_owned(Some("Fonts\\FRIZQT__.TTF"));
        let metrics = cosmic_text::Metrics::new(14.0, 18.0);
        let mut buffer = cosmic_text::Buffer::new(&mut fs.font_system, metrics);
        buffer.set_text(
            &mut fs.font_system,
            "Hello WoW",
            &attrs.as_attrs(),
            cosmic_text::Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut fs.font_system, true);

        // Should produce at least one layout run with glyphs
        let runs: Vec<_> = buffer.layout_runs().collect();
        assert!(!runs.is_empty(), "No layout runs produced");
        assert!(!runs[0].glyphs.is_empty(), "No glyphs in first run");
    }
}
