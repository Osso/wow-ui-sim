//! Texture loading and caching for WoW UI textures.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Texture manager that loads and caches textures.
#[derive(Debug)]
pub struct TextureManager {
    /// Base path to wow-ui-textures repository.
    textures_path: PathBuf,
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
            cache: HashMap::new(),
            sub_cache: HashMap::new(),
        }
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
            if let Ok(data) = load_texture_file(&file_path) {
                self.cache.insert(normalized.clone(), data);
                return self.cache.get(&normalized);
            }
        }

        None
    }

    /// Get a cached texture without loading.
    pub fn get(&self, wow_path: &str) -> Option<&TextureData> {
        let normalized = normalize_wow_path(wow_path);
        self.cache.get(&normalized)
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
        // WoW paths like "Interface/DialogFrame/UI-DialogBox-Background"
        // map to "DialogFrame/UI-DialogBox-Background.PNG" in the repo

        // Remove "Interface/" prefix if present
        let path = normalized_path
            .strip_prefix("Interface/")
            .unwrap_or(normalized_path);

        // Try different extensions
        for ext in &["PNG", "png", "tga", "TGA", "blp", "BLP"] {
            let file_path = self.textures_path.join(format!("{}.{}", path, ext));
            if file_path.exists() {
                return Some(file_path);
            }
        }

        // Try without extension (file might already have it)
        let file_path = self.textures_path.join(path);
        if file_path.exists() {
            return Some(file_path);
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
fn load_texture_file(path: &Path) -> Result<TextureData, image::ImageError> {
    let img = image::open(path)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(TextureData {
        width,
        height,
        pixels: rgba.into_raw(),
    })
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
