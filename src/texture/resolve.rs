use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::paths::find_case_insensitive;

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
fn try_casc_resolve(normalized_path: &str) -> Option<PathBuf> {
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
fn try_casc_resolve(_normalized_path: &str) -> Option<PathBuf> {
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
        if let Some(addon_relative) = strip_addons_prefix(normalized_path) {
            for addons_path in &self.addons_paths {
                if let Some(result) = self.try_resolve_in_dir(addons_path, addon_relative) {
                    return Some(result);
                }
            }
        }

        if cfg!(feature = "client-mists")
            && is_legacy_paperdoll_slot_path(normalized_path)
            && let Some(result) = try_blizzard_interface_art_resolve(self, normalized_path)
        {
            return Some(result);
        }

        if let Some(result) = try_casc_resolve(normalized_path) {
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
        for ext in texture_extension_priority() {
            let file_path = base.join(format!("{}.{}", path, ext));
            if file_path.is_file() {
                return Some(file_path);
            }
        }

        let file_path = base.join(path);
        if file_path.is_file() {
            return Some(file_path);
        }

        if let Some(result) = self.resolve_case_insensitive_in(base, path) {
            return Some(result);
        }

        None
    }

    /// Resolve path with case-insensitive directory matching within a base directory.
    fn resolve_case_insensitive_in(&self, base: &Path, path: &str) -> Option<PathBuf> {
        let components: Vec<&str> = path.split('/').collect();
        let file_name = components.last()?;
        let mut current = base.to_path_buf();

        for component in components.iter().take(components.len().saturating_sub(1)) {
            current = find_case_insensitive_dir(&current, component)?;
        }
        find_case_insensitive_file(&current, file_name)
    }
}

fn strip_addons_prefix(path: &str) -> Option<&str> {
    let prefix_len = "Interface/AddOns/".len();
    let (prefix, rest) = path.split_at_checked(prefix_len)?;
    prefix
        .eq_ignore_ascii_case("Interface/AddOns/")
        .then_some(rest)
}

fn normalize_crop_request_key(path: &str) -> String {
    if let Some(index) = path.find("@crop:") {
        let base = normalize_wow_path(&path[..index]);
        return format!("{base}@crop:{}", &path[index + 6..]);
    }
    normalize_wow_path(path)
}

fn persistent_crop_cache_root() -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("wow-ui-sim").join("crop-cache");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn load_persistent_sub_region_cache(root: &Path, key: &str) -> Option<TextureData> {
    let path = persistent_sub_region_cache_path(root, key);
    let rgba = image::open(path).ok()?.to_rgba8();
    Some(TextureData {
        width: rgba.width(),
        height: rgba.height(),
        pixels: Arc::<[u8]>::from(rgba.into_raw()),
    })
}

fn store_persistent_sub_region_cache(root: &Path, key: &str, data: &TextureData) {
    let path = persistent_sub_region_cache_path(root, key);
    let Some(parent) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let Some(image) = image::RgbaImage::from_raw(data.width, data.height, data.pixels.to_vec())
    else {
        return;
    };
    let _ = image.save(path);
}

fn persistent_sub_region_cache_path(root: &Path, key: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    let prefix = safe_crop_cache_prefix(key);
    root.join(format!("{prefix}-{hash:016x}.png"))
}

fn safe_crop_cache_prefix(key: &str) -> String {
    let mut prefix = String::new();
    for ch in key.chars() {
        if prefix.len() >= 96 {
            break;
        }
        if ch.is_ascii_alphanumeric() {
            prefix.push(ch.to_ascii_lowercase());
        } else if !prefix.ends_with('_') {
            prefix.push('_');
        }
    }
    prefix.trim_matches('_').to_string()
}

fn texture_extension_priority() -> &'static [&'static str] {
    &[
        "blp", "BLP", "webp", "WEBP", "PNG", "png", "tga", "TGA", "jpg", "JPG",
    ]
}

fn find_case_insensitive_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for ext in texture_extension_priority() {
        let with_ext = format!("{name}.{ext}");
        if let Some(entry) = find_case_insensitive(dir, &with_ext).filter(|entry| entry.is_file()) {
            return Some(entry);
        }
    }
    find_case_insensitive(dir, name).filter(|entry| entry.is_file())
}

fn find_case_insensitive_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    let entry = find_case_insensitive(dir, name)?;
    entry.is_dir().then_some(entry)
}

/// Extract a sub-region from texture data.
fn extract_sub_region(
    data: &TextureData,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Option<TextureData> {
    if x + width > data.width || y + height > data.height {
        return None;
    }

    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in y..(y + height) {
        let start = ((row * data.width + x) * 4) as usize;
        let end = start + (width * 4) as usize;
        pixels.extend_from_slice(&data.pixels[start..end]);
    }

    bleed_low_alpha_black_padding(&mut pixels, width, height);

    Some(TextureData {
        width,
        height,
        pixels: Arc::<[u8]>::from(pixels),
    })
}

/// Bumped whenever `extract_sub_region` changes what it writes.
const CROP_CACHE_VERSION: u32 = 2;

// Only fully transparent black texels take the neighbouring edge colour:
// they contribute nothing on their own and only matter through bilinear
// mixing at a crop edge. A black texel with ANY alpha is drawn art -- the
// micro-menu icons and the bag rings carry a pure-black drop shadow at
// alpha 2..128 on all four sides, and recolouring it with the icon's edge
// colour turned the shadow into a coloured halo around every icon.
const EDGE_BLEED_ALPHA_MAX: u8 = 0;
const EDGE_BLEED_RGB_MAX: u8 = 4;
const EDGE_SOURCE_ALPHA_MIN: u8 = 128;
const EDGE_SOURCE_RGB_MIN: u8 = 64;

fn bleed_low_alpha_black_padding(pixels: &mut [u8], width: u32, height: u32) {
    let source = pixels.to_vec();
    for y in 0..height {
        for x in 0..width {
            let offset = pixel_offset(width, x, y);
            if !is_low_alpha_black_padding(&source[offset..offset + 4]) {
                continue;
            }
            let Some(rgb) = nearest_edge_rgb(&source, width, height, x, y) else {
                continue;
            };
            pixels[offset..offset + 3].copy_from_slice(&rgb);
        }
    }
}

fn nearest_edge_rgb(source: &[u8], width: u32, height: u32, x: u32, y: u32) -> Option<[u8; 3]> {
    let max_radius = width.max(height);
    for radius in 1..=max_radius {
        if let Some(source_x) = x.checked_sub(radius)
            && let Some(rgb) = edge_rgb_at(source, width, source_x, y)
        {
            return Some(rgb);
        }
        if let Some(source_x) = x.checked_add(radius).filter(|&source_x| source_x < width)
            && let Some(rgb) = edge_rgb_at(source, width, source_x, y)
        {
            return Some(rgb);
        }
        if let Some(rgb) = y
            .checked_sub(radius)
            .and_then(|source_y| edge_rgb_at(source, width, x, source_y))
        {
            return Some(rgb);
        }
        if y + radius < height
            && let Some(rgb) = edge_rgb_at(source, width, x, y + radius)
        {
            return Some(rgb);
        }
    }
    None
}

fn edge_rgb_at(source: &[u8], width: u32, x: u32, y: u32) -> Option<[u8; 3]> {
    let offset = pixel_offset(width, x, y);
    let pixel = source.get(offset..offset + 4)?;
    is_edge_color_source(pixel).then_some([pixel[0], pixel[1], pixel[2]])
}

fn pixel_offset(width: u32, x: u32, y: u32) -> usize {
    ((y * width + x) * 4) as usize
}

fn is_low_alpha_black_padding(pixel: &[u8]) -> bool {
    pixel[3] <= EDGE_BLEED_ALPHA_MAX
        && pixel[0] <= EDGE_BLEED_RGB_MAX
        && pixel[1] <= EDGE_BLEED_RGB_MAX
        && pixel[2] <= EDGE_BLEED_RGB_MAX
}

fn is_edge_color_source(pixel: &[u8]) -> bool {
    pixel[3] > EDGE_SOURCE_ALPHA_MIN && pixel[0].max(pixel[1]).max(pixel[2]) > EDGE_SOURCE_RGB_MIN
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
