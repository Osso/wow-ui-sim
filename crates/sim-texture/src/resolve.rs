use crate::source_index::DirectorySearch;
use crate::{TextureData, normalize_wow_path};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

static CACHE_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
pub fn strip_addons_prefix(path: &str) -> Option<&str> {
    let prefix_len = "Interface/AddOns/".len();
    let (prefix, rest) = path.split_at_checked(prefix_len)?;
    prefix
        .eq_ignore_ascii_case("Interface/AddOns/")
        .then_some(rest)
}

pub fn normalize_crop_request_key(path: &str) -> String {
    if let Some(index) = path.find("@crop:") {
        let base = normalize_wow_path(&path[..index]);
        return format!("{base}@crop:{}", &path[index + 6..]);
    }
    normalize_wow_path(path)
}

pub fn persistent_crop_cache_root() -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("wow-ui-sim").join("crop-cache");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

pub fn load_persistent_sub_region_cache(root: &Path, key: &str) -> Option<TextureData> {
    let path = persistent_sub_region_cache_path(root, key);
    let rgba = image::open(path).ok()?.to_rgba8();
    Some(TextureData {
        width: rgba.width(),
        height: rgba.height(),
        pixels: Arc::<[u8]>::from(rgba.into_raw()),
    })
}

pub fn store_persistent_sub_region_cache(root: &Path, key: &str, data: &TextureData) {
    let path = persistent_sub_region_cache_path(root, key);
    let Some(image) = image::RgbaImage::from_raw(data.width, data.height, data.pixels.to_vec())
    else {
        return;
    };
    if let Err(error) = write_cache_file_atomically(&path, root, |temporary| {
        image.save(temporary).map_err(io::Error::other)
    }) {
        eprintln!("crop cache write {}: {error}", path.display());
    }
}

/// Publish only complete cache files. The private directory keeps the original extension
/// and avoids collisions between concurrent writes by the same process. `staging` must be
/// on the destination filesystem; callers whose destination directories are observed for
/// negative-cache validity stage outside them so failed writes leave those stamps unchanged.
pub fn write_cache_file_atomically(
    path: &Path,
    staging: &Path,
    write: impl FnOnce(&Path) -> io::Result<()>,
) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing cache parent"))?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::other("missing cache filename"))?;
    std::fs::create_dir_all(parent)?;
    std::fs::create_dir_all(staging)?;
    let directory = create_cache_write_directory(staging)?;
    let temporary = directory.join(name);
    let result = write(&temporary).and_then(|()| std::fs::rename(&temporary, path));
    if let Err(error) = std::fs::remove_dir_all(&directory) {
        eprintln!("cache temporary directory {}: {error}", directory.display());
    }
    result
}

fn create_cache_write_directory(parent: &Path) -> io::Result<PathBuf> {
    loop {
        let sequence = CACHE_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = parent.join(format!(".cache-write-{}-{sequence}", std::process::id()));
        match std::fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

pub fn persistent_sub_region_cache_path(root: &Path, key: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    let prefix = safe_crop_cache_prefix(key);
    root.join(format!("{prefix}-{hash:016x}.png"))
}

pub fn safe_crop_cache_prefix(key: &str) -> String {
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

pub fn texture_extension_priority() -> &'static [&'static str] {
    &[
        "blp", "BLP", "webp", "WEBP", "PNG", "png", "tga", "TGA", "jpg", "JPG",
    ]
}

pub fn find_case_insensitive_file(
    search: &mut DirectorySearch<'_>,
    dir: &Path,
    name: &str,
) -> Option<PathBuf> {
    for ext in texture_extension_priority() {
        let with_ext = format!("{name}.{ext}");
        if let Some(entry) = search
            .read_match(dir, &with_ext)
            .filter(|entry| entry.is_file())
        {
            return Some(entry);
        }
    }
    search.read_match(dir, name).filter(|entry| entry.is_file())
}

pub fn find_case_insensitive_dir(
    search: &mut DirectorySearch<'_>,
    dir: &Path,
    name: &str,
) -> Option<PathBuf> {
    let entry = search.read_match(dir, name)?;
    entry.is_dir().then_some(entry)
}

/// Extract a sub-region from texture data.
pub fn extract_sub_region(
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

/// Bumped whenever `extract_sub_region` changes what it writes; it is part
/// of the persistent crop cache key.
pub const CROP_CACHE_VERSION: u32 = 2;

// Only fully transparent black texels take the neighbouring edge colour:
// they contribute nothing on their own and only matter through bilinear
// mixing at a crop edge. A black texel with ANY alpha is drawn art -- the
// micro-menu icons and the bag rings carry a pure-black drop shadow at
// alpha 2..128 on all four sides, and recolouring it with the icon's edge
// colour turned the shadow into a coloured halo around every icon.
const EDGE_BLEED_ALPHA: u8 = 0;
const EDGE_BLEED_RGB_MAX: u8 = 4;
const EDGE_SOURCE_ALPHA_MIN: u8 = 128;
const EDGE_SOURCE_RGB_MIN: u8 = 64;

pub fn bleed_low_alpha_black_padding(pixels: &mut [u8], width: u32, height: u32) {
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

pub fn nearest_edge_rgb(source: &[u8], width: u32, height: u32, x: u32, y: u32) -> Option<[u8; 3]> {
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

pub fn edge_rgb_at(source: &[u8], width: u32, x: u32, y: u32) -> Option<[u8; 3]> {
    let offset = pixel_offset(width, x, y);
    let pixel = source.get(offset..offset + 4)?;
    is_edge_color_source(pixel).then_some([pixel[0], pixel[1], pixel[2]])
}

pub fn pixel_offset(width: u32, x: u32, y: u32) -> usize {
    ((y * width + x) * 4) as usize
}

pub fn is_low_alpha_black_padding(pixel: &[u8]) -> bool {
    pixel[3] == EDGE_BLEED_ALPHA
        && pixel[0] <= EDGE_BLEED_RGB_MAX
        && pixel[1] <= EDGE_BLEED_RGB_MAX
        && pixel[2] <= EDGE_BLEED_RGB_MAX
}

pub fn is_edge_color_source(pixel: &[u8]) -> bool {
    pixel[3] > EDGE_SOURCE_ALPHA_MIN && pixel[0].max(pixel[1]).max(pixel[2]) > EDGE_SOURCE_RGB_MIN
}
