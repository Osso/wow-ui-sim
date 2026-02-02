//! Texture loading and caching for WoW UI textures.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use image_blp::convert::blp_to_image;
use image_blp::parser::load_blp;

/// Texture manager that loads and caches textures.
#[derive(Debug)]
pub struct TextureManager {
    /// Base path to wow-ui-textures repository (for game UI textures).
    textures_path: PathBuf,
    /// Base path to WoW Interface directory (for extracted game files).
    interface_path: Option<PathBuf>,
    /// Base path to addons directory (for addon textures).
    addons_path: Option<PathBuf>,
    /// Cache of loaded texture data (path -> RGBA pixels).
    cache: HashMap<String, TextureData>,
    /// Cache of sub-region textures (path#region -> RGBA pixels).
    sub_cache: HashMap<String, TextureData>,
}

/// Loaded texture data.
#[derive(Debug, Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA
}

impl TextureManager {
    /// Create a new texture manager with the given textures path.
    pub fn new(textures_path: impl Into<PathBuf>) -> Self {
        Self {
            textures_path: textures_path.into(),
            interface_path: None,
            addons_path: None,
            cache: HashMap::new(),
            sub_cache: HashMap::new(),
        }
    }

    /// Set the WoW Interface directory path for extracted game files.
    pub fn with_interface_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.interface_path = Some(path.into());
        self
    }

    /// Set the addons directory path for addon textures.
    pub fn with_addons_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.addons_path = Some(path.into());
        self
    }

    /// Load a texture by its WoW path (e.g., "Interface\\DialogFrame\\UI-DialogBox-Background").
    pub fn load(&mut self, wow_path: &str) -> Option<&TextureData> {
        // Normalize the path
        let normalized = normalize_wow_path(wow_path);

        // Check cache first
        if self.cache.contains_key(&normalized) {
            return self.cache.get(&normalized);
        }

        // Try to load from disk
        if let Some(file_path) = self.resolve_path(&normalized) {
            match load_texture_file(&file_path) {
                Ok(data) => {
                    self.cache.insert(normalized.clone(), data);
                    return self.cache.get(&normalized);
                }
                Err(e) => {
                    eprintln!("[TexMgr] Load error: {} -> {}: {}", wow_path, file_path.display(), e);
                }
            }
        } else {
            eprintln!("[TexMgr] Not found: {}", wow_path);
        }

        None
    }

    /// Get a cached texture without loading.
    pub fn get(&self, wow_path: &str) -> Option<&TextureData> {
        let normalized = normalize_wow_path(wow_path);
        self.cache.get(&normalized)
    }

    /// Get the dimensions of a cached texture.
    pub fn get_texture_size(&self, wow_path: &str) -> Option<(u32, u32)> {
        let normalized = normalize_wow_path(wow_path);
        self.cache.get(&normalized).map(|d| (d.width, d.height))
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
        let normalized = normalize_wow_path(wow_path);
        let key = format!("{}#{}_{}_{}_{}", normalized, x, y, width, height);

        // Check sub-region cache
        if self.sub_cache.contains_key(&key) {
            return self.sub_cache.get(&key);
        }

        // Load the full texture first
        if let Some(file_path) = self.resolve_path(&normalized) {
            if let Ok(full_data) = load_texture_file(&file_path) {
                // Extract sub-region
                if let Some(sub_data) = extract_sub_region(&full_data, x, y, width, height) {
                    self.sub_cache.insert(key.clone(), sub_data);
                    return self.sub_cache.get(&key);
                }
            }
        }

        None
    }

    /// Resolve a WoW texture path to a file system path.
    fn resolve_path(&self, normalized_path: &str) -> Option<PathBuf> {
        // Handle addon textures: Interface/AddOns/AddonName/path/texture
        if let Some(addon_relative) = normalized_path
            .strip_prefix("Interface/AddOns/")
            .or_else(|| normalized_path.strip_prefix("interface/Addons/"))
            .or_else(|| normalized_path.strip_prefix("interface/addons/"))
        {
            if let Some(addons_path) = &self.addons_path {
                if let Some(result) = self.try_resolve_in_dir(addons_path, addon_relative) {
                    return Some(result);
                }
            }
        }

        // Remove "Interface/" prefix if present for game textures
        let path = normalized_path
            .strip_prefix("Interface/")
            .or_else(|| normalized_path.strip_prefix("interface/"))
            .unwrap_or(normalized_path);

        // Try wow-ui-textures repo first
        if let Some(result) = self.try_resolve_in_dir(&self.textures_path, path) {
            return Some(result);
        }

        // Try WoW Interface directory (extracted game files)
        if let Some(interface_path) = &self.interface_path {
            if let Some(result) = self.try_resolve_in_dir(interface_path, path) {
                return Some(result);
            }
        }

        None
    }

    /// Try to resolve a path within a given base directory.
    fn try_resolve_in_dir(&self, base: &Path, path: &str) -> Option<PathBuf> {
        // Try different extensions
        for ext in &["PNG", "png", "tga", "TGA", "blp", "BLP", "jpg", "JPG"] {
            let file_path = base.join(format!("{}.{}", path, ext));
            if file_path.exists() {
                return Some(file_path);
            }
        }

        // Try without extension (file might already have it)
        let file_path = base.join(path);
        if file_path.exists() {
            return Some(file_path);
        }

        // Try case-insensitive directory matching
        if let Some(result) = self.resolve_case_insensitive_in(base, path) {
            return Some(result);
        }

        None
    }

    /// Resolve path with case-insensitive directory matching within a base directory.
    fn resolve_case_insensitive_in(&self, base: &Path, path: &str) -> Option<PathBuf> {
        let components: Vec<&str> = path.split('/').collect();
        let mut current = base.to_path_buf();

        for (i, component) in components.iter().enumerate() {
            let is_last = i == components.len() - 1;

            if is_last {
                // For the filename, try with different extensions
                for ext in &["PNG", "png", "tga", "TGA", "blp", "BLP", "jpg", "JPG"] {
                    let with_ext = format!("{}.{}", component, ext);
                    if let Some(entry) = self.find_case_insensitive(&current, &with_ext) {
                        return Some(entry);
                    }
                }
                // Try without extension
                if let Some(entry) = self.find_case_insensitive(&current, component) {
                    return Some(entry);
                }
            } else {
                // For directories, find case-insensitive match
                if let Some(entry) = self.find_case_insensitive(&current, component) {
                    if entry.is_dir() {
                        current = entry;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
        }
        None
    }

    /// Find a directory entry case-insensitively.
    fn find_case_insensitive(&self, dir: &Path, name: &str) -> Option<PathBuf> {
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
}

/// Normalize a WoW texture path.
fn normalize_wow_path(path: &str) -> String {
    // Replace backslashes with forward slashes
    let normalized = path.replace('\\', "/");
    // Remove file extension if present
    if let Some(pos) = normalized.rfind('.') {
        if normalized[pos..].len() <= 5 {
            return normalized[..pos].to_string();
        }
    }
    normalized
}

/// Load texture data from a file.
fn load_texture_file(path: &Path) -> Result<TextureData, Box<dyn std::error::Error + Send + Sync>> {
    // Check if it's a BLP file
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext.eq_ignore_ascii_case("blp") {
        // Use image-blp for BLP files
        // Note: image-blp uses image 0.24, we use 0.25, so extract raw pixels directly
        let blp = load_blp(path)?;
        let blp_img = blp_to_image(&blp, 0)?;
        // Get dimensions and convert to RGBA8 bytes
        let rgba = blp_img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        Ok(TextureData {
            width,
            height,
            pixels: rgba.into_raw(),
        })
    } else {
        // Use standard image crate for other formats
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(TextureData {
            width,
            height,
            pixels: rgba.into_raw(),
        })
    }
}

/// Extract a sub-region from texture data.
fn extract_sub_region(
    data: &TextureData,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Option<TextureData> {
    // Bounds check
    if x + width > data.width || y + height > data.height {
        return None;
    }

    let mut pixels = Vec::with_capacity((width * height * 4) as usize);

    for row in y..(y + height) {
        let start = ((row * data.width + x) * 4) as usize;
        let end = start + (width * 4) as usize;
        pixels.extend_from_slice(&data.pixels[start..end]);
    }

    Some(TextureData {
        width,
        height,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_wow_path() {
        assert_eq!(
            normalize_wow_path("Interface\\DialogFrame\\UI-DialogBox-Background"),
            "Interface/DialogFrame/UI-DialogBox-Background"
        );
        assert_eq!(
            normalize_wow_path("Interface\\BUTTONS\\UI-Panel-Button-Up.blp"),
            "Interface/BUTTONS/UI-Panel-Button-Up"
        );
    }
}
