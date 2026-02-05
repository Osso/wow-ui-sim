//! Glyph atlas and text quad emission.
//!
//! Rasterizes text via cosmic-text, packs glyph bitmaps into a texture atlas,
//! and emits textured quads through the existing QuadBatch pipeline.
//!
//! Glyphs are stored as RGBA (white + alpha) so the shader's `tex * color`
//! multiplication produces correctly tinted text from vertex color.

use std::collections::HashMap;

use cosmic_text::{Buffer, CacheKey, Metrics, Shaping, SwashContent};
use iced::Rectangle;

use super::font::WowFontSystem;
use super::shader::{BlendMode, QuadBatch};
use crate::widget::TextJustify;

/// Size of the glyph atlas texture in pixels.
const GLYPH_ATLAS_SIZE: u32 = 2048;

/// A rasterized glyph in the atlas.
#[derive(Debug, Clone, Copy)]
struct GlyphEntry {
    /// UV rectangle in the atlas.
    uv_x: f32,
    uv_y: f32,
    uv_w: f32,
    uv_h: f32,
    /// Glyph bitmap dimensions in pixels.
    width: u32,
    height: u32,
    /// Swash placement offsets from pen position to image top-left.
    left: i32,
    top: i32,
}

/// Atlas for rasterized glyph bitmaps.
///
/// Packs glyphs left-to-right, top-to-bottom into a single RGBA texture.
/// Uses a simple row packer: each row has the height of the tallest glyph in it.
pub struct GlyphAtlas {
    /// RGBA pixel data for the atlas texture.
    pixels: Vec<u8>,
    /// Current packing position.
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    /// Map from cosmic-text CacheKey to atlas entry.
    entries: HashMap<CacheKey, GlyphEntry>,
    /// Whether the atlas has new data since the last GPU upload.
    dirty: bool,
    /// Unique path used to register this atlas in the GpuTextureAtlas.
    atlas_path: String,
}

impl std::fmt::Debug for GlyphAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphAtlas")
            .field("glyphs", &self.entries.len())
            .field("cursor", &(self.cursor_x, self.cursor_y))
            .finish()
    }
}

impl GlyphAtlas {
    pub fn new() -> Self {
        Self {
            pixels: vec![0u8; (GLYPH_ATLAS_SIZE * GLYPH_ATLAS_SIZE * 4) as usize],
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            entries: HashMap::new(),
            dirty: false,
            atlas_path: "__glyph_atlas__".to_string(),
        }
    }

    /// The unique texture path used to identify this atlas in the GPU texture system.
    pub fn atlas_path(&self) -> &str {
        &self.atlas_path
    }

    /// Whether the atlas has new data that needs uploading.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark as clean after GPU upload.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get the atlas pixel data and dimensions for GPU upload.
    pub fn texture_data(&self) -> (&[u8], u32, u32) {
        (&self.pixels, GLYPH_ATLAS_SIZE, GLYPH_ATLAS_SIZE)
    }

    /// Rasterize a glyph and add it to the atlas if not already present.
    fn ensure_glyph(
        &mut self,
        font_system: &mut WowFontSystem,
        cache_key: CacheKey,
    ) -> Option<GlyphEntry> {
        if let Some(entry) = self.entries.get(&cache_key) {
            return Some(*entry);
        }

        let image = font_system
            .swash_cache
            .get_image(&mut font_system.font_system, cache_key)
            .as_ref()?;

        let width = image.placement.width;
        let height = image.placement.height;
        if width == 0 || height == 0 {
            return None;
        }

        // Advance to next row if no space
        if self.cursor_x + width > GLYPH_ATLAS_SIZE {
            self.cursor_x = 0;
            self.cursor_y += self.row_height + 1; // 1px padding
            self.row_height = 0;
        }

        // Check if we've run out of vertical space
        if self.cursor_y + height > GLYPH_ATLAS_SIZE {
            tracing::warn!("Glyph atlas full ({} glyphs)", self.entries.len());
            return None;
        }

        // Write glyph pixels into atlas as white + alpha
        match image.content {
            SwashContent::Mask => {
                for y in 0..height {
                    for x in 0..width {
                        let src_idx = (y * width + x) as usize;
                        let alpha = image.data.get(src_idx).copied().unwrap_or(0);
                        let dst_x = self.cursor_x + x;
                        let dst_y = self.cursor_y + y;
                        let dst_idx = ((dst_y * GLYPH_ATLAS_SIZE + dst_x) * 4) as usize;
                        self.pixels[dst_idx] = 255; // R
                        self.pixels[dst_idx + 1] = 255; // G
                        self.pixels[dst_idx + 2] = 255; // B
                        self.pixels[dst_idx + 3] = alpha; // A
                    }
                }
            }
            SwashContent::Color => {
                for y in 0..height {
                    for x in 0..width {
                        let src_idx = ((y * width + x) * 4) as usize;
                        let dst_x = self.cursor_x + x;
                        let dst_y = self.cursor_y + y;
                        let dst_idx = ((dst_y * GLYPH_ATLAS_SIZE + dst_x) * 4) as usize;
                        // Copy RGBA directly for color emoji/glyphs
                        self.pixels[dst_idx] = image.data.get(src_idx).copied().unwrap_or(0);
                        self.pixels[dst_idx + 1] =
                            image.data.get(src_idx + 1).copied().unwrap_or(0);
                        self.pixels[dst_idx + 2] =
                            image.data.get(src_idx + 2).copied().unwrap_or(0);
                        self.pixels[dst_idx + 3] =
                            image.data.get(src_idx + 3).copied().unwrap_or(0);
                    }
                }
            }
            SwashContent::SubpixelMask => {
                // Treat as regular mask using first channel
                for y in 0..height {
                    for x in 0..width {
                        let src_idx = ((y * width + x) * 3) as usize;
                        let alpha = image.data.get(src_idx).copied().unwrap_or(0);
                        let dst_x = self.cursor_x + x;
                        let dst_y = self.cursor_y + y;
                        let dst_idx = ((dst_y * GLYPH_ATLAS_SIZE + dst_x) * 4) as usize;
                        self.pixels[dst_idx] = 255;
                        self.pixels[dst_idx + 1] = 255;
                        self.pixels[dst_idx + 2] = 255;
                        self.pixels[dst_idx + 3] = alpha;
                    }
                }
            }
        }

        let entry = GlyphEntry {
            uv_x: self.cursor_x as f32 / GLYPH_ATLAS_SIZE as f32,
            uv_y: self.cursor_y as f32 / GLYPH_ATLAS_SIZE as f32,
            uv_w: width as f32 / GLYPH_ATLAS_SIZE as f32,
            uv_h: height as f32 / GLYPH_ATLAS_SIZE as f32,
            width,
            height,
            left: image.placement.left,
            top: image.placement.top,
        };

        self.entries.insert(cache_key, entry);
        self.cursor_x += width + 1; // 1px padding
        self.row_height = self.row_height.max(height);
        self.dirty = true;

        Some(entry)
    }
}

/// Emit text quads into a QuadBatch.
///
/// Shapes the text, rasterizes glyphs into the atlas, and pushes textured quads.
/// The glyph atlas texture must be uploaded separately via `GlyphAtlas::texture_data()`.
///
/// # Arguments
/// * `batch` - QuadBatch to append glyph quads to
/// * `font_system` - WoW font system for shaping and rasterization
/// * `glyph_atlas` - Glyph atlas for caching rasterized glyphs
/// * `text` - Text to render
/// * `bounds` - Screen rectangle for the text
/// * `font_path` - WoW font path (e.g. `Fonts\\FRIZQT__.TTF`)
/// * `font_size` - Font size in pixels
/// * `color` - RGBA text color (0.0-1.0)
/// * `justify_h` - Horizontal justification
/// * `justify_v` - Vertical justification
/// * `glyph_tex_index` - Texture index for the glyph atlas in the GPU atlas
pub fn emit_text_quads(
    batch: &mut QuadBatch,
    font_system: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    text: &str,
    bounds: Rectangle,
    font_path: Option<&str>,
    font_size: f32,
    color: [f32; 4],
    justify_h: TextJustify,
    justify_v: TextJustify,
    glyph_tex_index: i32,
) {
    if text.is_empty() || bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }

    let stripped = crate::render::text::strip_wow_markup(text);
    if stripped.is_empty() {
        return;
    }

    // Shape text with cosmic-text
    let line_height = (font_size * 1.2).ceil();
    let metrics = Metrics::new(font_size, line_height);
    let attrs = font_system.attrs_owned(font_path);

    let mut buffer = Buffer::new(&mut font_system.font_system, metrics);
    buffer.set_size(&mut font_system.font_system, Some(bounds.width), Some(bounds.height));
    buffer.set_text(
        &mut font_system.font_system,
        &stripped,
        &attrs.as_attrs(),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system.font_system, true);

    // Calculate total text height for vertical justification
    let layout_runs: Vec<_> = buffer.layout_runs().collect();
    let total_height = layout_runs
        .last()
        .map(|run| run.line_y + line_height)
        .unwrap_or(0.0);

    // Vertical offset based on justification
    let y_offset = match justify_v {
        TextJustify::Left => 0.0,   // TOP
        TextJustify::Center => (bounds.height - total_height) / 2.0,
        TextJustify::Right => bounds.height - total_height, // BOTTOM
    };

    // Emit quads for each glyph
    for run in buffer.layout_runs() {
        // Horizontal offset based on justification
        let run_width = run.line_w;
        let x_offset = match justify_h {
            TextJustify::Left => 0.0,
            TextJustify::Center => (bounds.width - run_width) / 2.0,
            TextJustify::Right => bounds.width - run_width,
        };

        for glyph in run.glyphs.iter() {
            let physical_glyph = glyph.physical((0.0, 0.0), 1.0);

            if let Some(entry) =
                glyph_atlas.ensure_glyph(font_system, physical_glyph.cache_key)
            {
                // Glyph positioning based on glyphon's approach:
                // x = physical_glyph.x + placement.left
                // y = line_y + physical_glyph.y - placement.top
                let glyph_x =
                    bounds.x + x_offset + physical_glyph.x as f32 + entry.left as f32;
                let glyph_y =
                    bounds.y + y_offset + run.line_y + physical_glyph.y as f32 - entry.top as f32;

                let glyph_bounds = Rectangle::new(
                    iced::Point::new(glyph_x, glyph_y),
                    iced::Size::new(entry.width as f32, entry.height as f32),
                );

                let uv = Rectangle::new(
                    iced::Point::new(entry.uv_x, entry.uv_y),
                    iced::Size::new(entry.uv_w, entry.uv_h),
                );

                batch.push_quad(glyph_bounds, uv, color, glyph_tex_index, BlendMode::Alpha);
            }
        }
    }
}
