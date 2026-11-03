use sim_texture::resolve::*;
use std::path::{Path, PathBuf};

use super::{TextureData, TextureManager, normalize_wow_path};

#[cfg(feature = "casc")]
use std::sync::OnceLock;

#[cfg(feature = "casc")]
static CASC_INITIALIZED: OnceLock<bool> = OnceLock::new();

#[cfg(feature = "casc")]
fn casc_enabled() -> bool {
    *CASC_INITIALIZED.get_or_init(|| {
        // Opt-out: WOW_SIM_CASC=0 disables. Anything else (or unset) enables.
        if std::env::var("WOW_SIM_CASC").ok().as_deref() == Some("0") {
            return false;
        }
        // Require a discoverable WoW install, otherwise no point trying.
        asset_resolver::wow_install_path().is_some()
    })
}

#[cfg(feature = "casc")]
fn blizzard_interface_art_root() -> Option<PathBuf> {
    asset_resolver::wow_install_path()
        .and_then(|root| crate::paths::blizzard_interface_art_root_for_install_root(&root))
}

#[cfg(not(feature = "casc"))]
fn blizzard_interface_art_root() -> Option<PathBuf> {
    None
}

#[cfg(feature = "casc")]
fn casc_extract_dir() -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("wow-ui-sim/casc-extract");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

#[cfg(feature = "casc")]
fn try_casc_resolve(manager: &TextureManager, normalized_path: &str) -> Option<PathBuf> {
    if !casc_enabled() {
        return None;
    }

    let resolver = crate::asset_resolver_config::resolver();

    // Listfile entries always include the file extension; our normalized paths usually don't.
    // Try common UI asset extensions.
    let lower = normalized_path.to_ascii_lowercase();
    let prefixed =
        (!lower.starts_with("interface/")).then(|| format!("Interface/{normalized_path}"));
    let bases: Vec<String> = std::iter::once(normalized_path.to_string())
        .chain(prefixed)
        .collect();
    let candidates: &[&str] = &["blp", "BLP", "tga", "TGA", "ttf", "TTF", "otf", "OTF"];
    let (fdid, listfile_path) = bases
        .iter()
        .flat_map(|b| {
            std::iter::once(b.clone()).chain(candidates.iter().map(move |ext| format!("{b}.{ext}")))
        })
        .find_map(|p| lookup_casc_path(resolver, &p))?;

    let extract_dir = casc_extract_dir()?;
    let safe_relative = listfile_path.replace('\\', "/");
    let out_path = extract_dir.join(&safe_relative);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).ok()?;
    }
    // Observe after our own directory creation so an unchanged extraction miss
    // stays cheap. Native CASC availability retains the existing reload boundary.
    {
        let mut index = manager.source_index.borrow_mut();
        index.observe_source(&extract_dir, &safe_relative);
        index.observe_entry(&out_path);
    }

    if out_path.exists() {
        return Some(out_path);
    }

    resolver.ensure_cached(fdid, &out_path).or_else(|| {
        crate::casc_asset_fallback::ensure_known_asset_cached(&listfile_path, &out_path)
    })
}

#[cfg(feature = "casc")]
fn lookup_casc_path(
    resolver: &asset_resolver::CascListfileResolver,
    path: &str,
) -> Option<(u32, String)> {
    crate::limited_listfile::lookup_entry(path)
        .map(|entry| (entry.fdid, entry.path.to_string()))
        .or_else(|| {
            resolver
                .lookup_path(path)
                .map(|fdid| (fdid, path.to_string()))
        })
}

#[cfg(not(feature = "casc"))]
fn try_casc_resolve(_manager: &TextureManager, _normalized_path: &str) -> Option<PathBuf> {
    None
}

impl TextureManager {
    /// Number of entries in the texture cache.
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    /// Return all cached texture paths (for diagnostics/tests).
    pub fn cached_paths(&self) -> Vec<&str> {
        self.cache.keys().map(|path| path.as_str()).collect()
    }

    /// Get the dimensions of a cached texture.
    pub fn get_texture_size(&self, wow_path: &str) -> Option<(u32, u32)> {
        let normalized = normalize_wow_path(wow_path);
        self.cache
            .get(&normalized)
            .map(|d| (d.width, d.height))
            .or_else(|| self.size_cache.get(&normalized).copied())
    }

    /// Get dimensions for a texture, using cached metadata before falling back.
    pub fn get_or_load_texture_size(&mut self, wow_path: &str) -> Option<(u32, u32)> {
        if let Some((w, h)) = self.get_texture_size(wow_path) {
            return Some((w, h));
        }
        let normalized = normalize_wow_path(wow_path);
        let file_path = self.resolve_path(&normalized)?;
        let dims = super::read_texture_dimensions(&file_path).ok()?;
        self.size_cache.insert(normalized, dims);
        Some(dims)
    }

    /// Load a sub-region of a texture (for texture atlases).
    /// The key format is "path#x,y,w,h" where x,y is top-left and w,h is size.
    pub fn load_sub_region(
        &mut self,
        wow_path: &str,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Option<&TextureData> {
        self.load_sub_region_with_cache_root(
            wow_path,
            x,
            y,
            width,
            height,
            persistent_crop_cache_root().as_deref(),
        )
    }

    fn load_sub_region_with_cache_root(
        &mut self,
        wow_path: &str,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        cache_root: Option<&Path>,
    ) -> Option<&TextureData> {
        let normalized = normalize_wow_path(wow_path);
        // The version segment invalidates persistent crops written by an
        // earlier extract_sub_region; a code change there otherwise never
        // reaches an install whose crop cache is already populated.
        let key = format!(
            "{}#{}_{}_{}_{}#v{}",
            normalized, x, y, width, height, CROP_CACHE_VERSION
        );

        if self.sub_cache.contains_key(&key) {
            return self.sub_cache.get(&key);
        }
        if let Some(cache_root) = cache_root
            && let Some(cached) = load_persistent_sub_region_cache(cache_root, &key)
        {
            self.sub_cache.insert(key.clone(), cached);
            return self.sub_cache.get(&key);
        }

        if !self.cache.contains_key(&normalized) && self.load(wow_path).is_none() {
            return None;
        }
        if let Some(full_data) = self.cache.get(&normalized)
            && let Some(sub_data) = extract_sub_region(full_data, x, y, width, height)
        {
            if let Some(cache_root) = cache_root {
                store_persistent_sub_region_cache(cache_root, &key, &sub_data);
            }
            self.sub_cache.insert(key.clone(), sub_data);
            return self.sub_cache.get(&key);
        }

        None
    }

    pub fn get_cached_crop_request(&self, crop_request_path: &str) -> Option<&TextureData> {
        self.sub_cache
            .get(&normalize_crop_request_key(crop_request_path))
    }

    pub fn cache_crop_request_alias(
        &mut self,
        crop_request_path: &str,
        data: &TextureData,
    ) -> Option<&TextureData> {
        let key = normalize_crop_request_key(crop_request_path);
        self.sub_cache.insert(key.clone(), data.clone());
        self.sub_cache.get(&key)
    }

    #[cfg(test)]
    pub fn insert_test_texture(&mut self, wow_path: &str, data: TextureData) {
        let normalized = normalize_wow_path(wow_path);
        self.size_cache
            .insert(normalized.clone(), (data.width, data.height));
        self.cache.insert(normalized, data);
    }

    /// Resolve a WoW texture path to a file system path.
    pub fn resolve_path(&self, normalized_path: &str) -> Option<PathBuf> {
        self.source_index.borrow_mut().begin_resolution();
        if let Some(addon_relative) = strip_addons_prefix(normalized_path) {
            for addons_path in &self.addons_paths {
                if let Some(result) = self.try_resolve_in_dir(addons_path, addon_relative) {
                    return Some(result);
                }
            }
            // The renderer uses this bundled asset even with exclusive addon roots.
            if addon_relative.eq_ignore_ascii_case("SimCommands/textures/minimap-placeholder")
                && let Some(result) = crate::paths::bundled_addons_path_candidates()
                    .iter()
                    .find_map(|root| self.try_resolve_in_dir(root, addon_relative))
            {
                return Some(result);
            }
        }

        if cfg!(feature = "client-mists")
            && is_legacy_paperdoll_slot_path(normalized_path)
            && let Some(result) = try_blizzard_interface_art_resolve(self, normalized_path)
        {
            return Some(result);
        }

        if let Some(result) = try_casc_resolve(self, normalized_path) {
            return Some(result);
        }

        // Fallback: an extracted Interface dump. Covers entries the live CASC
        // has GC'd but the older dump still has on disk.
        if let Some(result) = try_blizzard_interface_art_resolve(self, normalized_path) {
            return Some(result);
        }

        None
    }
}

fn try_blizzard_interface_art_resolve(
    texture_manager: &TextureManager,
    normalized_path: &str,
) -> Option<PathBuf> {
    let blizzard_art_root = blizzard_interface_art_root()?;
    texture_manager
        .source_index
        .borrow_mut()
        .observe_entry(&blizzard_art_root);
    if !blizzard_art_root.exists() {
        return None;
    }
    if let Some(result) = texture_manager.try_resolve_in_dir(&blizzard_art_root, normalized_path) {
        return Some(result);
    }
    let lower = normalized_path.to_ascii_lowercase();
    if !lower.starts_with("interface/") {
        let prefixed = format!("Interface/{normalized_path}");
        if let Some(result) = texture_manager.try_resolve_in_dir(&blizzard_art_root, &prefixed) {
            return Some(result);
        }
    }
    None
}

fn is_legacy_paperdoll_slot_path(normalized_path: &str) -> bool {
    let lower = normalized_path.to_ascii_lowercase();
    lower.starts_with("interface/paperdoll/ui-paperdoll-slot-")
}

impl TextureManager {
    /// Try to resolve a path within a given base directory.
    fn try_resolve_in_dir(&self, base: &Path, path: &str) -> Option<PathBuf> {
        self.source_index.borrow_mut().observe_source(base, path);
        for ext in texture_extension_priority() {
            let file_path = base.join(format!("{}.{}", path, ext));
            if file_path.is_file() {
                self.source_index.borrow_mut().observe_entry(&file_path);
                return Some(file_path);
            }
        }

        let file_path = base.join(path);
        if file_path.is_file() {
            self.source_index.borrow_mut().observe_entry(&file_path);
            return Some(file_path);
        }

        if let Some(result) = self.resolve_case_insensitive_in(base, path) {
            self.source_index.borrow_mut().observe_entry(&result);
            return Some(result);
        }

        None
    }

    /// Resolve path with case-insensitive directory matching within a base directory.
    fn resolve_case_insensitive_in(&self, base: &Path, path: &str) -> Option<PathBuf> {
        let mut index = self.source_index.borrow_mut();
        let mut search = index.search();
        let components: Vec<&str> = path.split('/').collect();
        let file_name = components.last()?;
        let mut current = base.to_path_buf();

        for component in components.iter().take(components.len().saturating_sub(1)) {
            current = find_case_insensitive_dir(&mut search, &current, component)?;
        }
        find_case_insensitive_file(&mut search, &current, file_name)
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_sub_region, persistent_sub_region_cache_path};
    use crate::texture::TextureData;
    use std::path::Path;
    use std::sync::Arc;

    #[test]
    fn cropped_sub_region_bleeds_edge_rgb_only_into_fully_transparent_padding() {
        // A black texel with alpha is a drawn drop shadow (the micro-menu
        // icons carry one at alpha 2..128); only alpha 0 is padding.
        let data = TextureData {
            width: 5,
            height: 1,
            pixels: Arc::from([
                220, 180, 40, 255, // opaque edge color
                0, 0, 0, 93, // drop shadow
                0, 0, 0, 55, // drop shadow
                0, 0, 0, 30, // drop shadow
                0, 0, 0, 0, // fully transparent padding
            ]),
        };

        let cropped = extract_sub_region(&data, 0, 0, 5, 1).unwrap();

        assert_eq!(
            &cropped.pixels[4..8],
            &[0, 0, 0, 93],
            "a shadow texel keeps its black"
        );
        assert_eq!(&cropped.pixels[8..12], &[0, 0, 0, 55]);
        assert_eq!(&cropped.pixels[12..16], &[0, 0, 0, 30]);
        assert_eq!(
            &cropped.pixels[16..20],
            &[220, 180, 40, 0],
            "padding takes the edge colour"
        );
    }

    #[test]
    fn cropped_sub_region_uses_persistent_cache_without_base_texture() {
        let root = tempfile::tempdir().unwrap();
        let mut source_mgr = crate::texture::TextureManager::new();
        source_mgr.insert_test_texture(
            "Interface/Foo/Atlas",
            TextureData {
                width: 2,
                height: 2,
                pixels: Arc::from([1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255]),
            },
        );

        let stored = source_mgr
            .load_sub_region_with_cache_root("Interface/Foo/Atlas", 1, 0, 1, 2, Some(root.path()))
            .unwrap()
            .clone();

        let mut cached_mgr = crate::texture::TextureManager::new();
        let cached = cached_mgr
            .load_sub_region_with_cache_root("Interface/Foo/Atlas", 1, 0, 1, 2, Some(root.path()))
            .unwrap();

        assert_eq!(cached.width, 1);
        assert_eq!(cached.height, 2);
        assert_eq!(cached.pixels, stored.pixels);
    }

    #[test]
    fn persistent_crop_cache_path_includes_stable_hash_suffix() {
        let root = Path::new("/tmp/crops");
        let path = persistent_sub_region_cache_path(root, "Interface/Foo#1_2_3_4");
        let file_name = path.file_name().unwrap().to_string_lossy();

        assert!(file_name.starts_with("interface_foo_1_2_3_4-"));
        assert!(file_name.ends_with(".png"));
    }
}
