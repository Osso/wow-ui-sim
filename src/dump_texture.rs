//! Dump textures used by frames to disk for debugging atlas crops.

use std::path::Path;

use crate::render::shader::load_texture_or_crop;
use crate::render::QuadBatch;
use crate::texture::TextureManager;

/// Save all unique textures from a QuadBatch to disk as PNGs.
///
/// Filters texture paths by optional substring match (case-insensitive).
/// Saves both regular and mask textures.
pub fn dump_batch_textures(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
    output_dir: &Path,
    filter: Option<&str>,
) {
    std::fs::create_dir_all(output_dir).ok();
    let mut seen = std::collections::HashSet::new();
    let mut saved = 0;

    let all_requests = batch.texture_requests.iter()
        .chain(&batch.mask_texture_requests);

    for request in all_requests {
        if seen.contains(&request.path) {
            continue;
        }
        seen.insert(request.path.clone());

        if let Some(f) = filter {
            let fl = f.to_lowercase();
            if !request.path.to_lowercase().contains(&fl) {
                continue;
            }
        }

        let Some(gpu_data) = load_texture_or_crop(tex_mgr, &request.path) else {
            eprintln!("  FAILED: {}", request.path);
            continue;
        };

        let filename = sanitize_texture_filename(&request.path);
        let out_path = output_dir.join(&filename);
        match image::RgbaImage::from_raw(gpu_data.width, gpu_data.height, gpu_data.rgba) {
            Some(img) => {
                if let Err(e) = img.save(&out_path) {
                    eprintln!("  ERROR saving {filename}: {e}");
                } else {
                    eprintln!("  {}x{} → {}", gpu_data.width, gpu_data.height, out_path.display());
                    saved += 1;
                }
            }
            None => eprintln!("  BAD DATA: {} ({}x{}, expected {} bytes)",
                request.path, gpu_data.width, gpu_data.height,
                gpu_data.width * gpu_data.height * 4),
        }
    }
    eprintln!("Saved {saved} textures to {}", output_dir.display());
}

/// Convert a texture path (with @crop: suffix) to a safe filename.
fn sanitize_texture_filename(path: &str) -> String {
    let name = path
        .replace('\\', "_")
        .replace('/', "_")
        .replace('@', "_at_")
        .replace(':', "_")
        .replace(',', "_");
    // Strip leading underscores from Interface_ prefix
    let name = name.trim_start_matches('_');
    format!("{name}.png")
}
