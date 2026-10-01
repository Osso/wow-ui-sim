//! Font management using cosmic-text.
//!
//! Loads WoW TTF fonts into a cosmic-text FontSystem for text shaping,
//! measurement, and glyph rasterization. Provides mapping from WoW font
//! paths (e.g. `Fonts\\FRIZQT__.TTF`) to fontdb family names.

use std::collections::HashMap;

use cosmic_text::fontdb;

/// WoW font path constants (as they appear in Lua/XML).
const WOW_FONT_FRIZ: &str = "Fonts\\FRIZQT__.TTF";
const WOW_FONT_ARIAL_NARROW: &str = "Fonts\\ARIALN.TTF";
const LINE_HEIGHT_MULTIPLIER: f32 = 1.2;

/// Default WoW font (Friz Quadrata).
pub const DEFAULT_WOW_FONT: &str = WOW_FONT_FRIZ;

pub(crate) fn line_height_for_font_size(font_size: f32, smooth_scaling: bool) -> Option<f32> {
    if !font_size.is_finite() || font_size <= 0.0 {
        return None;
    }
    let height = font_size * LINE_HEIGHT_MULTIPLIER;
    Some(if smooth_scaling {
        height
    } else {
        height.ceil().max(1.0)
    })
}

pub(crate) fn spaced_text_height(base_height: f32, line_count: usize, spacing: f32) -> f32 {
    base_height + line_count.saturating_sub(1) as f32 * spacing
}

/// Font entry mapping a WoW path to a fontdb family name.
#[derive(Debug, Clone)]
struct FontEntry {
    family: String,
}

struct WowFontFile {
    filename: &'static str,
    wow_paths: &'static [&'static str],
    encoding_key_hex: Option<&'static str>,
}

const WOW_FONT_FILES: &[WowFontFile] = &[
    WowFontFile {
        filename: "FRIZQT__.TTF",
        wow_paths: &[WOW_FONT_FRIZ, "Fonts\\frizqt__.ttf"],
        encoding_key_hex: Some("DB472FF5CA74465BAA066021CD837645"),
    },
    WowFontFile {
        filename: "ARIALN.TTF",
        wow_paths: &[WOW_FONT_ARIAL_NARROW, "Fonts\\arialn.ttf"],
        encoding_key_hex: Some("B118D76FD2E2BDA9AAB0118B508D0FB1"),
    },
    WowFontFile {
        filename: "FRIZQT___CYR.TTF",
        wow_paths: &["Fonts\\FRIZQT___CYR.TTF", "Fonts\\frizqt___cyr.ttf"],
        encoding_key_hex: Some("78AEBA943ABCFF292438DA989CC1E728"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "MORPHEUS.TTF",
        wow_paths: &["Fonts\\MORPHEUS.TTF", "Fonts\\MORPHEUS.ttf"],
        encoding_key_hex: Some("81DFB1A8D2E8E5F88CC52B87184D1888"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "MORPHEUS_CYR.TTF",
        wow_paths: &["Fonts\\MORPHEUS_CYR.TTF", "Fonts\\MORPHEUS_CYR.ttf"],
        encoding_key_hex: Some("6F7F97CE27CC21B3CE3FB7DEC042EF6F"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "SKURRI.TTF",
        wow_paths: &["Fonts\\SKURRI.TTF", "Fonts\\skurri.ttf"],
        encoding_key_hex: Some("71642D3EFE9972983D3B4C5265402E93"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "SKURRI_CYR.TTF",
        wow_paths: &["Fonts\\SKURRI_CYR.TTF"],
        encoding_key_hex: Some("39512C75D1D6190C4318EB73365190E4"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "K_Pagetext.TTF",
        wow_paths: &["Fonts\\K_Pagetext.TTF", "Fonts\\K_Pagetext.ttf"],
        encoding_key_hex: Some("68B4590641A67D22C60A9C764AA8F639"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "K_Damage.TTF",
        wow_paths: &["Fonts\\K_Damage.TTF", "Fonts\\K_Damage.ttf"],
        encoding_key_hex: Some("267124985FE8E273FE9954DD59C57E2E"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "2002.TTF",
        wow_paths: &["Fonts\\2002.TTF", "Fonts\\2002.ttf"],
        encoding_key_hex: Some("D578A1888E2E5A893FBC6E28439233C6"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "ARHei.ttf",
        wow_paths: &["Fonts\\ARHei.ttf"],
        encoding_key_hex: Some("F622A11C55EE13A97DCEDD7D1E4CDBAD"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "ARKai_C.ttf",
        wow_paths: &["Fonts\\ARKai_C.ttf", "Fonts\\ARKai_C.TTF"],
        encoding_key_hex: Some("B06A317EAF8BD5166331CA2796015B73"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "ARKai_T.ttf",
        wow_paths: &["Fonts\\ARKai_T.ttf", "Fonts\\ARKai_T.TTF"],
        encoding_key_hex: Some("E2DFFF92C883E2A321CED396767E79DA"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "bHEI00M.TTF",
        wow_paths: &["Fonts\\bHEI00M.TTF"],
        encoding_key_hex: Some("6AD9B2C4A54D15683700CEB6263DFE05"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "bHEI01B.TTF",
        wow_paths: &["Fonts\\bHEI01B.TTF"],
        encoding_key_hex: Some("AEF77F819D6C1AB29C34B104BAFEC3AA"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "bKAI00M.TTF",
        wow_paths: &["Fonts\\bKAI00M.TTF", "Fonts\\bKAI00M.ttf"],
        encoding_key_hex: Some("1056AFA8FA4B3589FC2FD6869057D3AC"),
    },
    #[cfg(feature = "client-mists")]
    WowFontFile {
        filename: "blei00d.ttf",
        wow_paths: &[
            "Fonts\\blei00d.ttf",
            "Fonts\\blei00d.TTF",
            "Fonts\\BLEI00D.TTF",
        ],
        encoding_key_hex: Some("C0558AB41E6E62016686CF61AE0EC207"),
    },
];

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

#[cfg(feature = "casc")]
use std::sync::OnceLock;

#[cfg(feature = "casc")]
static FONT_CASC_INITIALIZED: OnceLock<bool> = OnceLock::new();
#[cfg(feature = "casc")]
static FONT_CASC_RESOLUTION_CACHE: OnceLock<
    Option<asset_resolver::casc_cache::CascResolutionCache>,
> = OnceLock::new();
#[cfg(feature = "casc")]
static FONT_CASC_READER: OnceLock<Option<CascFontReader>> = OnceLock::new();

#[cfg(feature = "casc")]
struct CascFontReader {
    install: cascette_client_storage::Installation,
    resolver: cascette_client_storage::resolver::ContentResolver,
}

#[cfg(feature = "casc")]
fn casc_enabled() -> bool {
    *FONT_CASC_INITIALIZED.get_or_init(|| {
        // Opt-out: WOW_SIM_CASC=0 disables. Anything else (or unset) enables.
        if std::env::var("WOW_SIM_CASC").ok().as_deref() == Some("0") {
            return false;
        }
        // Require a discoverable WoW install, otherwise no point trying.
        asset_resolver::wow_install_path().is_some()
    })
}

#[cfg(feature = "casc")]
fn try_casc_font_bytes(font_file: &WowFontFile) -> Option<Vec<u8>> {
    if !casc_enabled() {
        return None;
    }
    if let Some(bytes) = read_cached_font_bytes(font_file.filename) {
        return Some(bytes);
    }
    if let Some(encoding_key_hex) = font_file.encoding_key_hex {
        if let Some(bytes) = try_casc_font_bytes_by_encoding_key(encoding_key_hex) {
            write_cached_font_bytes(font_file.filename, &bytes);
            return Some(bytes);
        }
    }

    let path = format!("Fonts/{}", font_file.filename);
    if let Some(bytes) = try_casc_font_bytes_by_path(&path) {
        write_cached_font_bytes(font_file.filename, &bytes);
        return Some(bytes);
    }

    let resolver = crate::asset_resolver_config::resolver();
    let fdid =
        crate::limited_listfile::lookup_path(&path).or_else(|| resolver.lookup_path(&path))?;
    if !casc_resolution_contains_fdid(fdid) {
        return None;
    }
    let bytes = resolver.resolve_bytes(fdid)?;
    write_cached_font_bytes(font_file.filename, &bytes);
    Some(bytes)
}

#[cfg(feature = "casc")]
fn read_cached_font_bytes(filename: &str) -> Option<Vec<u8>> {
    std::fs::read(font_cache_path(filename)?).ok()
}

#[cfg(feature = "casc")]
fn write_cached_font_bytes(filename: &str, bytes: &[u8]) {
    let Some(path) = font_cache_path(filename) else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_ok() {
        let _ = std::fs::write(path, bytes);
    }
}

#[cfg(feature = "casc")]
fn font_cache_path(filename: &str) -> Option<std::path::PathBuf> {
    Some(dirs::cache_dir()?.join("wow-ui-sim/fonts").join(filename))
}

#[cfg(feature = "casc")]
fn try_casc_font_bytes_by_encoding_key(encoding_key_hex: &str) -> Option<Vec<u8>> {
    let reader = FONT_CASC_READER
        .get_or_init(init_casc_font_reader)
        .as_ref()?;
    let encoding_key = encoding_key_from_hex(encoding_key_hex)?;
    run_font_casc_async(reader.install.read_file_by_encoding_key(&encoding_key)).ok()
}

#[cfg(feature = "casc")]
fn try_casc_font_bytes_by_path(path: &str) -> Option<Vec<u8>> {
    let reader = FONT_CASC_READER
        .get_or_init(init_casc_font_reader)
        .as_ref()?;
    reader.read_path(path)
}

#[cfg(feature = "casc")]
fn init_casc_font_reader() -> Option<CascFontReader> {
    crate::logging::eprintln_elapsed("[Startup] font CASC reader init begin");
    let total_start = std::time::Instant::now();
    let install_root = asset_resolver::wow_install_path()?;
    let data_path = asset_resolver::wow_data_path()?;
    let casc_dir = asset_resolver::casc_resolver::casc_cache_dir_for_install(install_root).ok()?;

    let root_data = timed_font_phase("font CASC root.bin read", || {
        std::fs::read(casc_dir.join("root.bin"))
    })
    .ok()?;
    let encoding_data = timed_font_phase("font CASC encoding.bin read", || {
        std::fs::read(casc_dir.join("encoding.bin"))
    })
    .ok()?;

    let resolver = timed_font_phase("font CASC resolver built", || {
        build_font_casc_resolver(&root_data, &encoding_data)
    })?;

    let install = timed_font_phase("font CASC installation opened", || {
        cascette_client_storage::Installation::open(data_path)
    })
    .ok()?;
    timed_font_phase("font CASC installation initialized", || {
        run_font_casc_async(install.initialize())
    })
    .ok()?;

    crate::logging::eprintln_elapsed(&format!(
        "[Startup] font CASC reader init complete in {:.2?}",
        total_start.elapsed()
    ));
    Some(CascFontReader { install, resolver })
}

#[cfg(feature = "casc")]
fn build_font_casc_resolver(
    root_data: &[u8],
    encoding_data: &[u8],
) -> Option<cascette_client_storage::resolver::ContentResolver> {
    let resolver = cascette_client_storage::resolver::ContentResolver::new();
    resolver.load_root_file(root_data).ok()?;
    resolver.load_encoding_file(encoding_data).ok()?;
    Some(resolver)
}

#[cfg(feature = "casc")]
impl CascFontReader {
    fn read_path(&self, path: &str) -> Option<Vec<u8>> {
        font_path_candidates(path).find_map(|candidate| self.read_exact_path(&candidate))
    }

    fn read_exact_path(&self, path: &str) -> Option<Vec<u8>> {
        let encoding_key = self.resolver.resolve_path_to_encoding(path)?;
        run_font_casc_async(self.install.read_file_by_encoding_key(&encoding_key)).ok()
    }
}

#[cfg(feature = "casc")]
fn font_path_candidates(path: &str) -> impl Iterator<Item = String> + '_ {
    let slash_path = path.replace('\\', "/");
    let backslash_path = slash_path.replace('/', "\\");
    [slash_path, backslash_path].into_iter()
}

#[cfg(feature = "casc")]
fn run_font_casc_async<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("font CASC runtime")
        .block_on(future)
}

#[cfg(feature = "casc")]
fn encoding_key_from_hex(hex: &str) -> Option<cascette_crypto::EncodingKey> {
    let bytes = parse_hex_16(hex)?;
    Some(cascette_crypto::EncodingKey::from_bytes(bytes))
}

#[cfg(feature = "casc")]
fn parse_hex_16(hex: &str) -> Option<[u8; 16]> {
    if hex.len() != 32 {
        return None;
    }

    let mut bytes = [0u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&hex[start..start + 2], 16).ok()?;
    }
    Some(bytes)
}

#[cfg(feature = "casc")]
fn casc_resolution_contains_fdid(fdid: u32) -> bool {
    FONT_CASC_RESOLUTION_CACHE
        .get_or_init(|| {
            let install = asset_resolver::wow_install_path()?;
            asset_resolver::casc_resolver::open_resolution_cache_for_install(install).ok()
        })
        .as_ref()
        .is_some_and(|cache| cache.resolve_fdid(fdid).is_some())
}

#[cfg(not(feature = "casc"))]
fn try_casc_font_bytes(_font_file: &WowFontFile) -> Option<Vec<u8>> {
    None
}

impl WowFontSystem {
    /// Create a new font system with WoW fonts loaded from CASC (or system fallback).
    pub fn new() -> Self {
        Self::new_with_options(true)
    }

    /// Create a font system without opening CASC.
    ///
    /// Non-rendering commands use this to avoid paying CASC startup cost when
    /// they only need text metrics good enough for Lua-side layout.
    pub fn new_without_casc() -> Self {
        Self::new_with_options(false)
    }

    fn new_with_options(load_casc_fonts: bool) -> Self {
        crate::logging::eprintln_elapsed(&format!(
            "[Startup] WowFontSystem::new begin casc={load_casc_fonts}"
        ));
        let total_start = std::time::Instant::now();
        let mut db = fontdb::Database::new();
        let mut font_map = HashMap::new();

        if load_casc_fonts {
            load_wow_fonts(&mut db, &mut font_map);
        }
        if font_map.is_empty() {
            timed_font_phase("font system fallback loaded", || {
                load_system_font_fallback(&mut db, &mut font_map)
            });
        }

        if font_map.is_empty() {
            timed_font_phase("system fonts loaded", || db.load_system_fonts());
        }

        let font_system = timed_font_phase("cosmic font system built", || {
            cosmic_text::FontSystem::new_with_locale_and_db("en-US".to_string(), db)
        });
        let swash_cache = timed_font_phase("swash cache created", cosmic_text::SwashCache::new);

        let result = Self {
            font_system,
            swash_cache,
            font_map,
        };
        crate::logging::eprintln_elapsed(&format!(
            "[Startup] WowFontSystem::new complete in {:.2?}",
            total_start.elapsed()
        ));
        result
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

    /// Measure the pixel width of a text string using cosmic-text shaping.
    ///
    /// `font_path` is the WoW font path (e.g. `Fonts\\FRIZQT__.TTF`).
    /// Returns the width of the first layout line.
    pub fn measure_text_width(
        &mut self,
        text: &str,
        font_path: Option<&str>,
        font_size: f32,
    ) -> f32 {
        let Some(line_height) = line_height_for_font_size(font_size, false) else {
            return 0.0;
        };
        if text.is_empty() {
            return 0.0;
        }
        let metrics = cosmic_text::Metrics::new(font_size, line_height);
        let attrs = self.attrs_owned(font_path);
        let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(10000.0), Some(line_height));
        buffer.set_text(
            &mut self.font_system,
            text,
            &attrs.as_attrs(),
            cosmic_text::Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, true);

        buffer
            .layout_runs()
            .map(|run| run.line_w)
            .next()
            .unwrap_or(0.0)
    }

    /// Measure the pixel height of text, accounting for word wrapping.
    ///
    /// If `wrap_width` is Some and > 0, text wraps at that width.
    /// Returns the total height of all layout lines.
    pub fn measure_text_height(
        &mut self,
        text: &str,
        font_path: Option<&str>,
        font_size: f32,
        wrap_width: Option<f32>,
        spacing: f32,
        smooth_scaling: bool,
    ) -> f32 {
        self.measure_text_layout(
            text,
            font_path,
            font_size,
            wrap_width,
            spacing,
            smooth_scaling,
        )
        .0
    }

    pub(crate) fn measure_text_layout(
        &mut self,
        text: &str,
        font_path: Option<&str>,
        font_size: f32,
        wrap_width: Option<f32>,
        spacing: f32,
        smooth_scaling: bool,
    ) -> (f32, usize) {
        let Some(line_height) = line_height_for_font_size(font_size, smooth_scaling) else {
            return (0.0, 0);
        };
        if text.is_empty() {
            return (0.0, 0);
        }
        let metrics = cosmic_text::Metrics::new(font_size, line_height);
        let attrs = self.attrs_owned(font_path);
        let shape_width = match wrap_width {
            Some(w) if w > 0.0 => w,
            _ => 10000.0,
        };
        let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(shape_width), Some(10000.0));
        buffer.set_text(
            &mut self.font_system,
            text,
            &attrs.as_attrs(),
            cosmic_text::Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, true);

        let runs: Vec<_> = buffer.layout_runs().collect();
        let num_lines = runs.len();
        // Smooth measurement uses the same baseline span as glyph layout.
        // Keep the legacy false-mode first-baseline contribution unchanged.
        let first_y = if smooth_scaling {
            runs.first().map(|run| run.line_y).unwrap_or(0.0)
        } else {
            0.0
        };
        let base_height = if num_lines <= 1 {
            line_height
        } else {
            runs.last()
                .map(|run| run.line_y - first_y + line_height)
                .unwrap_or(line_height)
        };
        (
            spaced_text_height(base_height, num_lines, spacing),
            num_lines,
        )
    }
}

impl Default for WowFontSystem {
    fn default() -> Self {
        Self::new()
    }
}

fn load_wow_font(
    font_file: &WowFontFile,
    db: &mut fontdb::Database,
    font_map: &mut HashMap<String, FontEntry>,
) {
    let Some(data) = try_casc_font_bytes(font_file) else {
        tracing::warn!("Font {} not found in CASC, skipping", font_file.filename);
        return;
    };

    let family_name = fontdb_family_name(&data).unwrap_or_else(|| font_file.filename.to_string());
    db.load_font_data(data);
    register_font_aliases(font_file.wow_paths, &family_name, font_map);

    tracing::debug!(
        "Registered font {} from CASC -> family '{}'",
        font_file.filename,
        family_name
    );
}

fn load_wow_fonts(db: &mut fontdb::Database, font_map: &mut HashMap<String, FontEntry>) {
    for font_file in WOW_FONT_FILES {
        timed_font_phase(&format!("font {} loaded", font_file.filename), || {
            load_wow_font(font_file, db, font_map)
        });
    }
}

fn timed_font_phase<T>(label: &str, action: impl FnOnce() -> T) -> T {
    let phase_start = std::time::Instant::now();
    let result = action();
    crate::logging::eprintln_elapsed(&format!(
        "[Startup] {label} in {:.2?}",
        phase_start.elapsed()
    ));
    result
}

fn load_system_font_fallback(db: &mut fontdb::Database, font_map: &mut HashMap<String, FontEntry>) {
    db.load_system_fonts();
    let Some(family_name) = first_system_family_name(db) else {
        tracing::warn!("No CASC or system fonts available for text shaping");
        return;
    };

    for font_file in WOW_FONT_FILES {
        register_font_aliases(font_file.wow_paths, &family_name, font_map);
    }

    tracing::warn!("Using system font '{family_name}' as WoW font fallback");
}

fn first_system_family_name(db: &fontdb::Database) -> Option<String> {
    db.faces()
        .next()
        .and_then(|face| face.families.first())
        .map(|family| family.0.clone())
}

fn register_font_aliases(
    wow_paths: &[&str],
    family_name: &str,
    font_map: &mut HashMap<String, FontEntry>,
) {
    let entry = FontEntry {
        family: family_name.to_string(),
    };

    for wow_path in wow_paths {
        let key = normalize_wow_path(wow_path);
        font_map.insert(key, entry.clone());
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
    tmp_db.faces().next().map(|face| face.families[0].0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_wow_fonts() {
        let fs = WowFontSystem::new();
        // WoW UI font files come from CASC; without CASC the font_map is empty
        // and downstream shaping uses cosmic-text's default fonts.
        if asset_resolver_available() {
            assert!(
                !fs.font_map.is_empty(),
                "font_map should have CASC fonts: {:?}",
                fs.font_map.keys().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn no_casc_constructor_registers_font_aliases() {
        let fs = WowFontSystem::new_without_casc();

        assert!(
            fs.family_name(Some(WOW_FONT_FRIZ)).is_some(),
            "non-rendering commands should get font aliases without opening CASC"
        );
    }

    fn asset_resolver_available() -> bool {
        #[cfg(feature = "casc")]
        {
            asset_resolver::wow_install_path().is_some()
        }
        #[cfg(not(feature = "casc"))]
        {
            false
        }
    }

    #[cfg(feature = "casc")]
    #[test]
    fn loads_real_wow_fonts_from_casc() {
        if !asset_resolver_available() {
            return;
        }

        for font_file in WOW_FONT_FILES {
            let data = try_casc_font_bytes(font_file)
                .unwrap_or_else(|| panic!("{} CASC bytes", font_file.filename));
            let family = fontdb_family_name(&data)
                .unwrap_or_else(|| panic!("{} family name", font_file.filename));

            assert!(
                !family.is_empty(),
                "expected {} to load a real family name",
                font_file.filename
            );
        }

        let friz_data = try_casc_font_bytes(&WOW_FONT_FILES[0]).expect("FRIZQT__.TTF CASC bytes");
        let friz_family = fontdb_family_name(&friz_data).expect("FRIZQT__.TTF family name");
        assert!(
            friz_family.to_ascii_lowercase().contains("friz"),
            "expected Friz Quadrata family, got {friz_family}"
        );
    }

    #[test]
    fn resolves_friz_quadrata() {
        let fs = WowFontSystem::new();
        let name = fs.family_name(Some("Fonts\\FRIZQT__.TTF")).unwrap();
        assert!(!name.is_empty(), "Expected a font family fallback");
    }

    #[test]
    fn resolves_case_insensitive() {
        let fs = WowFontSystem::new();
        let upper = fs.family_name(Some("Fonts\\FRIZQT__.TTF"));
        let lower = fs.family_name(Some("Fonts\\frizqt__.ttf"));
        let mixed = fs.family_name(Some("fonts\\FrizQT__.TTF"));
        assert_eq!(upper, lower);
        assert_eq!(upper, mixed);
    }

    #[test]
    fn unknown_font_falls_back_to_default() {
        let fs = WowFontSystem::new();
        let name = fs.family_name(Some("Fonts\\NONEXISTENT.TTF")).unwrap();
        let default_name = fs.family_name(Some(DEFAULT_WOW_FONT)).unwrap();
        assert_eq!(name, default_name);
    }

    #[cfg(feature = "client-mists")]
    #[test]
    fn mists_specialty_fonts_resolve_without_default_fallback() {
        if !asset_resolver_available() {
            return;
        }

        let fs = WowFontSystem::new();
        let default_name = fs.family_name(Some(DEFAULT_WOW_FONT)).unwrap();

        for path in [
            "Fonts\\MORPHEUS.ttf",
            "Fonts\\skurri.ttf",
            "Fonts\\K_Pagetext.ttf",
            "Fonts\\K_Damage.ttf",
        ] {
            let family = fs.family_name(Some(path)).unwrap();
            assert_ne!(
                family, default_name,
                "{path} should resolve to its own loaded WoW font family, not the default font"
            );
        }
    }

    #[cfg(feature = "client-mists")]
    #[test]
    fn mists_font_paths_are_registered() {
        if !asset_resolver_available() {
            return;
        }

        let fs = WowFontSystem::new();

        for path in [
            "Fonts\\2002.TTF",
            "Fonts\\ARHei.ttf",
            "Fonts\\ARIALN.TTF",
            "Fonts\\ARKai_C.ttf",
            "Fonts\\ARKai_T.ttf",
            "Fonts\\bHEI00M.TTF",
            "Fonts\\bHEI01B.TTF",
            "Fonts\\bKAI00M.TTF",
            "Fonts\\blei00d.ttf",
            "Fonts\\FRIZQT___CYR.TTF",
            "Fonts\\FRIZQT__.TTF",
            "Fonts\\K_Damage.ttf",
            "Fonts\\K_Pagetext.ttf",
            "Fonts\\MORPHEUS_CYR.TTF",
            "Fonts\\MORPHEUS.ttf",
            "Fonts\\SKURRI_CYR.TTF",
            "Fonts\\skurri.ttf",
        ] {
            let key = normalize_wow_path(path);
            assert!(
                fs.font_map.contains_key(&key),
                "{path} should be registered for Mists font XML"
            );
        }
    }

    #[test]
    fn friz_font_bytes_resolve_when_font_fdid_cache_entry_is_missing() {
        if !asset_resolver_available() {
            return;
        }

        let bytes = try_casc_font_bytes(&WOW_FONT_FILES[0])
            .expect("FRIZQT__.TTF should resolve through path or known encoding key");

        assert!(
            fontdb_family_name(&bytes).is_some(),
            "FRIZQT__.TTF bytes should parse as a real font"
        );
    }

    #[test]
    fn none_font_uses_default() {
        let fs = WowFontSystem::new();
        let name = fs.family_name(None).unwrap();
        let default_name = fs.family_name(Some(DEFAULT_WOW_FONT)).unwrap();
        assert_eq!(name, default_name);
    }

    #[test]
    fn can_shape_text_with_loaded_font() {
        let mut fs = WowFontSystem::new();
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

    #[test]
    fn measure_text_width_returns_positive() {
        let mut fs = WowFontSystem::new();
        let w = fs.measure_text_width("Hello", Some(WOW_FONT_FRIZ), 14.0);
        assert!(w > 0.0, "Expected positive width, got {w}");
    }

    #[test]
    fn measure_text_width_empty_is_zero() {
        let mut fs = WowFontSystem::new();
        let w = fs.measure_text_width("", Some(WOW_FONT_FRIZ), 14.0);
        assert_eq!(w, 0.0);
    }

    #[test]
    fn zero_font_size_measures_as_empty_text() {
        let mut fs = WowFontSystem::new();

        assert_eq!(fs.measure_text_width("Collections", None, 0.0), 0.0);
        assert_eq!(
            fs.measure_text_height("Collections", None, 0.0, Some(100.0), 0.0, false),
            0.0
        );
    }

    #[test]
    fn measure_text_width_scales_with_length() {
        let mut fs = WowFontSystem::new();
        let short = fs.measure_text_width("Hi", Some(WOW_FONT_FRIZ), 14.0);
        let long = fs.measure_text_width("Hello World", Some(WOW_FONT_FRIZ), 14.0);
        assert!(
            long > short,
            "Longer text should be wider: {long} > {short}"
        );
    }

    #[test]
    fn measure_text_height_single_line() {
        let mut fs = WowFontSystem::new();
        let h = fs.measure_text_height("Hello", Some(WOW_FONT_FRIZ), 14.0, None, 0.0, false);
        let line_height = (14.0_f32 * 1.2).ceil();
        assert_eq!(h, line_height, "Single line should equal line_height");
    }

    #[test]
    fn measure_text_height_wraps_with_narrow_width() {
        let mut fs = WowFontSystem::new();
        let long_text =
            "This is a fairly long sentence that should wrap when given a narrow width constraint";
        let single = fs.measure_text_height(long_text, Some(WOW_FONT_FRIZ), 14.0, None, 0.0, false);
        let wrapped = fs.measure_text_height(
            long_text,
            Some(WOW_FONT_FRIZ),
            14.0,
            Some(100.0),
            0.0,
            false,
        );
        assert!(
            wrapped > single,
            "Wrapped text should be taller: {wrapped} > {single}"
        );
    }

    #[test]
    fn measure_text_height_empty_is_zero() {
        let mut fs = WowFontSystem::new();
        let h = fs.measure_text_height("", Some(WOW_FONT_FRIZ), 14.0, Some(200.0), 0.0, false);
        assert_eq!(h, 0.0);
    }
}
