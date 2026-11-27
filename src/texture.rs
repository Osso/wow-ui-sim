//! Texture loading and caching for WoW UI textures.

#[cfg(test)]
use image_blp::types::BlpContent;
#[cfg(test)]
use sim_texture::{bc_texture_data, bc_texture_result};
#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::time::Duration;

mod preload;
mod resolve;

/// A texture given by file ID that the interface manifest does not list (world art such as
/// minimap tiles, which add-ons draw with SetTexture(fileID)): a path the texture manager
/// resolves by that ID straight from the local CASC.
pub(crate) const FILE_DATA_ID_PREFIX: &str = "FileDataID\\";

pub(crate) fn file_data_id_texture_path(file_data_id: u32) -> Option<String> {
    cfg!(feature = "casc").then(|| format!("{FILE_DATA_ID_PREFIX}{file_data_id}"))
}
pub(crate) use sim_texture::source_index;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(crate) use sim_texture::read_texture_dimensions;
pub use sim_texture::{
    BcLoadTelemetry, BcTextureFormat, BcTextureResult, RgbaLoadTelemetry, TextureData,
    fix_1bit_alpha, normalize_wow_path,
};
use sim_texture::{is_blp_file, load_bc_texture_from_blp, load_texture_file};

/// Texture manager that loads and caches textures.
#[derive(Debug)]
pub struct TextureManager {
    /// Base paths to addon directories (for addon textures).
    addons_paths: Vec<PathBuf>,
    source_index: std::cell::RefCell<source_index::SourceIndex>,
    /// Negative CPU loads keyed by normalized request in this root/profile context.
    rgba_unavailable: HashMap<String, source_index::SourceObservation>,
    bc_unavailable: HashMap<String, source_index::SourceObservation>,
    /// Cache of loaded texture data (path -> RGBA pixels).
    cache: HashMap<String, TextureData>,
    /// Cache of texture dimensions keyed by normalized WoW path.
    size_cache: HashMap<String, (u32, u32)>,
    /// Cache of raw BC-compressed texture data keyed by normalized WoW path.
    bc_cache: HashMap<String, BcTextureResult>,
    /// Cache of sub-region textures (path#region -> RGBA pixels).
    sub_cache: HashMap<String, TextureData>,
    /// Paths already logged as missing; never used to suppress source retries.
    not_found: HashSet<String>,
}

impl TextureManager {
    /// Create a new texture manager.
    pub fn new() -> Self {
        Self {
            addons_paths: Vec::new(),
            source_index: Default::default(),
            rgba_unavailable: HashMap::new(),
            bc_unavailable: HashMap::new(),
            cache: HashMap::new(),
            size_cache: HashMap::new(),
            bc_cache: HashMap::new(),
            sub_cache: HashMap::new(),
            not_found: HashSet::new(),
        }
    }

    /// Set the addons directory path for addon textures.
    pub fn with_addons_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.set_source_roots(&[path.into()]);
        self
    }

    /// Set all addon directory paths for addon textures.
    pub fn with_addons_paths(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.set_source_roots(&paths.into_iter().collect::<Vec<_>>());
        self
    }

    #[cfg(feature = "gui")]
    pub(crate) fn source_roots(&self) -> &[PathBuf] {
        &self.addons_paths
    }

    /// Root order is part of source identity; do not sort or canonicalize it.
    pub fn set_source_roots(&mut self, roots: &[PathBuf]) {
        if roots != self.addons_paths {
            self.addons_paths = roots.to_vec();
            self.invalidate_source_generation();
        }
    }

    /// Call at an explicit asset/root reload, even if root spelling is unchanged.
    /// This does not evict already rendered pixels or GPU atlas handles.
    pub fn invalidate_source_generation(&mut self) {
        self.source_index.get_mut().clear();
        self.rgba_unavailable.clear();
        self.bc_unavailable.clear();
        self.not_found.clear();
    }

    /// Load a texture by its WoW path (e.g., "Interface\\DialogFrame\\UI-DialogBox-Background").
    pub fn load(&mut self, wow_path: &str) -> Option<&TextureData> {
        self.load_with_telemetry(wow_path).0
    }

    pub fn load_with_telemetry(
        &mut self,
        wow_path: &str,
    ) -> (Option<&TextureData>, RgbaLoadTelemetry) {
        let start = Instant::now();
        let normalized = normalize_wow_path(wow_path);
        let mut telemetry = RgbaLoadTelemetry::default();

        if self.cache.contains_key(&normalized) {
            return self.cached_texture_load_result(&normalized, start);
        }
        if self
            .rgba_unavailable
            .get(&normalized)
            .is_some_and(|entry| entry.is_current())
        {
            return (
                None,
                RgbaLoadTelemetry {
                    total_elapsed: start.elapsed(),
                    ..Default::default()
                },
            );
        }
        self.rgba_unavailable.remove(&normalized);

        let loaded = self.load_resolved_texture(wow_path, &normalized, &mut telemetry);
        telemetry.total_elapsed = start.elapsed();
        let texture = loaded.then(|| self.cache.get(&normalized)).flatten();
        (texture, telemetry)
    }

    fn cached_texture_load_result<'a>(
        &'a self,
        normalized: &str,
        start: Instant,
    ) -> (Option<&'a TextureData>, RgbaLoadTelemetry) {
        let mut telemetry = RgbaLoadTelemetry {
            mem_cache_hit: true,
            ..RgbaLoadTelemetry::default()
        };
        telemetry.total_elapsed = start.elapsed();
        (self.cache.get(normalized), telemetry)
    }

    fn load_resolved_texture(
        &mut self,
        wow_path: &str,
        normalized: &str,
        telemetry: &mut RgbaLoadTelemetry,
    ) -> bool {
        let resolve_start = Instant::now();
        let Some(file_path) = self.resolve_path(normalized) else {
            telemetry.resolve_elapsed = resolve_start.elapsed();
            self.rgba_unavailable.insert(
                normalized.to_string(),
                self.source_index.borrow().observation(),
            );
            if self.not_found.insert(normalized.to_string()) {
                crate::logging::eprintln_elapsed(&format!("[TexMgr] Not found: {}", wow_path));
            }
            return false;
        };

        telemetry.resolve_elapsed = resolve_start.elapsed();
        if !file_path.is_file() {
            self.not_found.insert(normalized.to_string());
            return false;
        }

        let (loaded, load_telemetry) = self.load_texture_with_telemetry(&file_path);
        telemetry.decode_elapsed = load_telemetry.decode_elapsed;
        let Some(data) = loaded else {
            return false;
        };

        self.cache_texture_data(normalized, data);
        true
    }

    fn cache_texture_data(&mut self, normalized: &str, data: TextureData) {
        self.size_cache
            .insert(normalized.to_string(), (data.width, data.height));
        self.cache.insert(normalized.to_string(), data);
    }

    /// Get a cached texture without loading.
    pub fn get(&self, wow_path: &str) -> Option<&TextureData> {
        let normalized = normalize_wow_path(wow_path);
        self.cache.get(&normalized)
    }

    /// Check if a texture is already in the CPU cache (no disk I/O needed).
    pub fn is_cached(&self, wow_path: &str) -> bool {
        let normalized = normalize_wow_path(wow_path);
        self.cache.contains_key(&normalized) || self.bc_cache.contains_key(&normalized)
    }

    fn load_texture_with_telemetry(
        &self,
        file_path: &Path,
    ) -> (Option<TextureData>, RgbaLoadTelemetry) {
        let mut telemetry = RgbaLoadTelemetry::default();
        let decode_start = Instant::now();
        match load_texture_file(file_path) {
            Ok(data) => {
                telemetry.decode_elapsed = decode_start.elapsed();
                (Some(data), telemetry)
            }
            Err(e) => {
                telemetry.decode_elapsed = decode_start.elapsed();
                crate::logging::eprintln_elapsed(&format!(
                    "[TexMgr] Load error: {}: {}",
                    file_path.display(),
                    e
                ));
                (None, telemetry)
            }
        }
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureManager {
    /// Attempt to load a BLP texture's raw BC block data without CPU decoding.
    ///
    /// Returns `Some` only when the resolved file is a BLP with DXT1/DXT3/DXT5 content.
    /// Callers should fall back to `load()` (RGBA path) when this returns `None`.
    pub fn load_bc(&mut self, wow_path: &str) -> Option<&BcTextureResult> {
        self.load_bc_with_telemetry(wow_path).0
    }

    /// Attempt to load a BLP texture's raw BC block data and capture timing breakdowns.
    pub fn load_bc_with_telemetry(
        &mut self,
        wow_path: &str,
    ) -> (Option<&BcTextureResult>, BcLoadTelemetry) {
        let start = Instant::now();
        let normalized = normalize_wow_path(wow_path);
        if self.bc_cache.contains_key(&normalized) {
            return self.cached_bc_load_result(&normalized, start);
        }
        if self
            .bc_unavailable
            .get(&normalized)
            .is_some_and(|entry| entry.is_current())
        {
            return (
                None,
                BcLoadTelemetry {
                    total_elapsed: start.elapsed(),
                    ..Default::default()
                },
            );
        }
        self.bc_unavailable.remove(&normalized);

        let mut telemetry = BcLoadTelemetry::default();
        let loaded = self.load_resolved_bc_texture(&normalized, &mut telemetry);
        telemetry.total_elapsed = start.elapsed();
        let texture = loaded.then(|| self.bc_cache.get(&normalized)).flatten();
        (texture, telemetry)
    }

    fn cached_bc_load_result<'a>(
        &'a self,
        normalized: &str,
        start: Instant,
    ) -> (Option<&'a BcTextureResult>, BcLoadTelemetry) {
        let mut telemetry = BcLoadTelemetry {
            cache_hit: true,
            ..BcLoadTelemetry::default()
        };
        telemetry.total_elapsed = start.elapsed();
        (self.bc_cache.get(normalized), telemetry)
    }

    fn load_resolved_bc_texture(
        &mut self,
        normalized: &str,
        telemetry: &mut BcLoadTelemetry,
    ) -> bool {
        let resolve_start = Instant::now();
        let Some(file_path) = self.resolve_path(normalized) else {
            telemetry.resolve_elapsed = resolve_start.elapsed();
            self.mark_bc_unavailable(normalized);
            return false;
        };
        telemetry.resolve_elapsed = resolve_start.elapsed();

        if !is_blp_file(&file_path) {
            self.mark_bc_unavailable(normalized);
            return false;
        }
        telemetry.resolved_blp = true;

        let Some(bc_texture) = load_bc_texture_from_blp(&file_path, telemetry) else {
            self.mark_bc_unavailable(normalized);
            return false;
        };

        self.cache_bc_texture_data(normalized, bc_texture);
        true
    }

    fn mark_bc_unavailable(&mut self, normalized: &str) {
        self.bc_unavailable.insert(
            normalized.to_string(),
            self.source_index.borrow().observation(),
        );
    }

    fn cache_bc_texture_data(&mut self, normalized: &str, bc_texture: BcTextureResult) {
        self.bc_cache.insert(normalized.to_string(), bc_texture);
    }
}

#[cfg(test)]
mod tests;
