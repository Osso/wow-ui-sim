//! Reusable texture decoding, crop operations and directory observations.
mod blp;
pub mod logging;
pub mod resolve;
pub mod source_index;
use image_blp::{convert::blp_to_image, types::BlpContent};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Loaded texture data.
#[derive(Debug, Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub pixels: Arc<[u8]>, // RGBA
}

/// Result of a BC-compressed texture load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcTextureFormat {
    Bc1,
    Bc3,
}

/// Result of a BC-compressed texture load.
#[derive(Debug, Clone)]
pub struct BcTextureResult {
    pub width: u32,
    pub height: u32,
    /// Raw BC block data (mip level 0 only).
    pub bc_data: Arc<[u8]>,
    /// BC compression format.
    pub format: BcTextureFormat,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BcLoadTelemetry {
    pub cache_hit: bool,
    pub resolved_blp: bool,
    pub resolve_elapsed: Duration,
    pub parse_elapsed: Duration,
    pub extract_elapsed: Duration,
    pub total_elapsed: Duration,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RgbaLoadTelemetry {
    pub mem_cache_hit: bool,
    pub resolve_elapsed: Duration,
    pub decode_elapsed: Duration,
    pub total_elapsed: Duration,
}

pub fn is_blp_file(file_path: &Path) -> bool {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    ext.eq_ignore_ascii_case("blp")
}

pub fn load_bc_texture_from_blp(
    file_path: &Path,
    telemetry: &mut BcLoadTelemetry,
) -> Option<BcTextureResult> {
    let parse_start = Instant::now();
    let blp = blp::load(file_path).ok();
    telemetry.parse_elapsed = parse_start.elapsed();

    let blp = blp?;
    let extract_start = Instant::now();
    let bc_texture = bc_texture_result(blp.header.width, blp.header.height, blp.content);
    telemetry.extract_elapsed = extract_start.elapsed();
    bc_texture
}

pub fn bc_texture_result(width: u32, height: u32, content: BlpContent) -> Option<BcTextureResult> {
    let (bc_data, format) = bc_texture_data(content)?;
    Some(BcTextureResult {
        width,
        height,
        bc_data,
        format,
    })
}

pub fn bc_texture_data(content: BlpContent) -> Option<(Arc<[u8]>, BcTextureFormat)> {
    match content {
        BlpContent::Dxt1(dxtn) => {
            first_bc_image(dxtn).map(|bc_data| (bc_data, BcTextureFormat::Bc1))
        }
        // DXT3 is BC2, not BC3. The renderer has no BC2 atlas yet, so let
        // DXT3 fall back through the RGBA path instead of decoding alpha as BC3.
        BlpContent::Dxt3(_) => None,
        BlpContent::Dxt5(dxtn) => {
            first_bc_image(dxtn).map(|bc_data| (bc_data, BcTextureFormat::Bc3))
        }
        _ => None,
    }
}

fn first_bc_image(dxtn: image_blp::types::direct::dxtn::BlpDxtn) -> Option<Arc<[u8]>> {
    let image = dxtn.images.into_iter().next()?;
    if image.content.is_empty() {
        return None;
    }
    Some(Arc::<[u8]>::from(image.content))
}

/// Normalize a WoW texture path.
pub fn normalize_wow_path(path: &str) -> String {
    let normalized = collapse_path_separators(&path.replace('\\', "/"));
    // Remove file extension if present
    if let Some(pos) = normalized.rfind('.')
        && normalized[pos..].len() <= 5
    {
        return normalized[..pos].to_string();
    }
    normalized
}

fn collapse_path_separators(path: &str) -> String {
    let mut collapsed = String::with_capacity(path.len());
    let mut last_was_separator = false;
    for ch in path.chars() {
        if ch == '/' {
            if !last_was_separator {
                collapsed.push(ch);
            }
            last_was_separator = true;
        } else {
            collapsed.push(ch);
            last_was_separator = false;
        }
    }
    collapsed
}

/// Fix 1-bit alpha decoded by image-blp as literal 0/1 byte values.
///
/// BLP files with `alphaDepth=1` store alpha as a single bit per pixel.
/// The image-blp crate decodes this as byte values 0 or 1 instead of 0 or
/// 255, making textures nearly invisible. This remaps: 0 stays 0, any
/// non-zero alpha becomes 255.
pub fn fix_1bit_alpha(pixels: &mut [u8]) {
    // Check if alpha looks like 1-bit (max alpha value <= 1)
    let max_alpha = pixels.iter().skip(3).step_by(4).copied().max().unwrap_or(0);
    if max_alpha > 1 {
        return; // Normal 8-bit alpha, no fix needed
    }
    for alpha in pixels.iter_mut().skip(3).step_by(4) {
        if *alpha > 0 {
            *alpha = 255;
        }
    }
}

/// Load texture data from a file.
pub fn load_texture_file(
    path: &Path,
) -> Result<TextureData, Box<dyn std::error::Error + Send + Sync>> {
    if !path.is_file() {
        return Err(format!("texture path is not a file: {}", path.display()).into());
    }

    // Check if it's a BLP file
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext.eq_ignore_ascii_case("blp") {
        // Use image-blp for BLP files
        // Note: image-blp uses image 0.24, we use 0.25, so extract raw pixels directly
        let blp = blp::load(path)?;
        let blp_img = blp_to_image(&blp, 0)?;
        // Get dimensions and convert to RGBA8 bytes
        let rgba = blp_img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        let mut pixels = rgba.into_raw();
        // Fix 1-bit alpha: image-blp decodes 1-bit alpha as literal 0/1 byte values
        // instead of 0/255. Remap any alpha > 0 to 255 for correct rendering.
        fix_1bit_alpha(&mut pixels);
        Ok(TextureData {
            width,
            height,
            pixels: Arc::<[u8]>::from(pixels),
        })
    } else {
        // Use standard image crate for other formats. Guess from file
        // contents so mislabeled addon assets still load.
        let img = image::ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(TextureData {
            width,
            height,
            pixels: Arc::<[u8]>::from(rgba.into_raw()),
        })
    }
}

pub fn read_texture_dimensions(
    path: &Path,
) -> Result<(u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("blp") {
        return read_blp_dimensions(path);
    }
    let dimensions = image::ImageReader::open(path)?
        .with_guessed_format()?
        .into_dimensions()?;
    Ok(dimensions)
}

fn read_blp_dimensions(
    path: &Path,
) -> Result<(u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    if &magic != b"BLP2" && &magic != b"BLP1" {
        return Err(format!("unsupported BLP magic in {}", path.display()).into());
    }
    file.seek(SeekFrom::Start(12))?;
    let mut dims = [0u8; 8];
    file.read_exact(&mut dims)?;
    let width = u32::from_le_bytes(dims[0..4].try_into()?);
    let height = u32::from_le_bytes(dims[4..8].try_into()?);
    Ok((width, height))
}
