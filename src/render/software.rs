//! CPU-based software rasterizer for producing screenshots.
//!
//! Renders a QuadBatch to an image::RgbaImage without requiring a GPU.
//! Supports solid color quads, textured quads with UV mapping, and alpha blending.
//! Text (FontString) is not rendered — this is for debugging frame layout and textures.

use image::RgbaImage;

use super::shader::{BlendMode, QuadBatch};
use crate::texture::TextureManager;

/// Render a QuadBatch to an RGBA image using CPU rasterization.
///
/// Iterates quads in batch order (already sorted by strata/level/layer),
/// resolves pending texture requests via TextureManager, samples textures
/// with nearest-neighbor filtering, and alpha-blends onto a black background.
pub fn render_to_image(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
    width: u32,
    height: u32,
) -> RgbaImage {
    let mut img = RgbaImage::from_pixel(width, height, image::Rgba([0, 0, 0, 255]));

    // Build a lookup from vertex index -> texture path for pending textures.
    // A single TextureRequest can span multiple quads (e.g. three-slice: 3 quads, 12 vertices).
    // Insert an entry for each quad's starting vertex within the request range.
    let mut tex_lookup: std::collections::HashMap<u32, String> =
        std::collections::HashMap::new();
    for req in &batch.texture_requests {
        let mut v = req.vertex_start;
        let end = req.vertex_start + req.vertex_count;
        while v < end {
            tex_lookup.insert(v, req.path.clone());
            v += 4; // Each quad is 4 vertices
        }
    }

    // Pre-load all requested textures
    for req in &batch.texture_requests {
        tex_mgr.load(&req.path);
    }

    // Process quads (every 4 vertices = 1 quad, every 6 indices = 1 quad)
    let num_quads = batch.indices.len() / 6;
    for qi in 0..num_quads {
        let idx_base = qi * 6;
        // First triangle: indices[0], indices[1], indices[2] → TL, TR, BR
        // The quad vertices are at indices[0]..indices[0]+3 (TL, TR, BR, BL)
        let v0 = batch.indices[idx_base] as usize; // TL

        let tl = &batch.vertices[v0];
        let br = &batch.vertices[v0 + 2];

        // Axis-aligned bounding rect
        let x0 = tl.position[0];
        let y0 = tl.position[1];
        let x1 = br.position[0];
        let y1 = br.position[1];

        // Clamp to image bounds
        let px0 = (x0.max(0.0) as u32).min(width);
        let py0 = (y0.max(0.0) as u32).min(height);
        let px1 = (x1.ceil().max(0.0) as u32).min(width);
        let py1 = (y1.ceil().max(0.0) as u32).min(height);

        if px0 >= px1 || py0 >= py1 {
            continue;
        }

        // UV coords from TL and BR vertices
        let uv_tl = tl.tex_coords;
        let uv_br = br.tex_coords;

        // Vertex color (same for all 4 vertices in our quads)
        let vc = tl.color;

        // Blend mode
        let blend_additive = tl.flags == BlendMode::Additive as u32;

        // Determine texture source
        let tex_data = if tl.tex_index == -2 {
            // Pending texture — look up path
            tex_lookup
                .get(&(v0 as u32))
                .and_then(|path| tex_mgr.get(path))
        } else if tl.tex_index >= 0 {
            // Pre-resolved texture index — not used in standalone mode
            None
        } else {
            // tex_index == -1 → solid color
            None
        };

        let is_solid = tl.tex_index == -1;

        for py in py0..py1 {
            for px in px0..px1 {
                // Compute source color
                let (sr, sg, sb, sa) = if is_solid {
                    // Solid color quad
                    (vc[0], vc[1], vc[2], vc[3])
                } else if let Some(tex) = tex_data {
                    // Textured quad — sample texture at interpolated UV
                    let t = if (x1 - x0).abs() > 0.001 {
                        (px as f32 + 0.5 - x0) / (x1 - x0)
                    } else {
                        0.0
                    };
                    let s = if (y1 - y0).abs() > 0.001 {
                        (py as f32 + 0.5 - y0) / (y1 - y0)
                    } else {
                        0.0
                    };

                    let u = uv_tl[0] + t * (uv_br[0] - uv_tl[0]);
                    let v = uv_tl[1] + s * (uv_br[1] - uv_tl[1]);

                    // Nearest-neighbor sample
                    let tx = ((u * tex.width as f32) as u32).min(tex.width.saturating_sub(1));
                    let ty = ((v * tex.height as f32) as u32).min(tex.height.saturating_sub(1));
                    let offset = ((ty * tex.width + tx) * 4) as usize;

                    if offset + 3 < tex.pixels.len() {
                        let tr = tex.pixels[offset] as f32 / 255.0;
                        let tg = tex.pixels[offset + 1] as f32 / 255.0;
                        let tb = tex.pixels[offset + 2] as f32 / 255.0;
                        let ta = tex.pixels[offset + 3] as f32 / 255.0;
                        // Multiply by vertex color (tint)
                        (tr * vc[0], tg * vc[1], tb * vc[2], ta * vc[3])
                    } else {
                        continue;
                    }
                } else {
                    // No texture resolved and not solid — skip
                    continue;
                };

                if sa <= 0.0 {
                    continue;
                }

                // Read destination pixel
                let dst = img.get_pixel(px, py);
                let dr = dst[0] as f32 / 255.0;
                let dg = dst[1] as f32 / 255.0;
                let db = dst[2] as f32 / 255.0;

                // Blend
                let (or, og, ob) = if blend_additive {
                    // Additive: dst + src * alpha
                    (dr + sr * sa, dg + sg * sa, db + sb * sa)
                } else {
                    // Alpha blend: src * alpha + dst * (1 - alpha)
                    (
                        sr * sa + dr * (1.0 - sa),
                        sg * sa + dg * (1.0 - sa),
                        sb * sa + db * (1.0 - sa),
                    )
                };

                let clamp = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
                img.put_pixel(px, py, image::Rgba([clamp(or), clamp(og), clamp(ob), 255]));
            }
        }
    }

    img
}
