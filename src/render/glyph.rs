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
    /// Swash placement offset from pen position to image top edge.
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

impl Default for GlyphAtlas {
    fn default() -> Self {
        Self::new()
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

        // Write glyph pixels into atlas
        write_glyph_pixels(
            &mut self.pixels,
            self.cursor_x,
            self.cursor_y,
            width,
            height,
            &image.data,
            image.content,
        );

        let entry = GlyphEntry {
            uv_x: self.cursor_x as f32 / GLYPH_ATLAS_SIZE as f32,
            uv_y: self.cursor_y as f32 / GLYPH_ATLAS_SIZE as f32,
            uv_w: width as f32 / GLYPH_ATLAS_SIZE as f32,
            uv_h: height as f32 / GLYPH_ATLAS_SIZE as f32,
            width,
            height,
            top: image.placement.top,
        };

        self.entries.insert(cache_key, entry);
        self.cursor_x += width + 1; // 1px padding
        self.row_height = self.row_height.max(height);
        self.dirty = true;

        Some(entry)
    }
}

/// Write glyph pixels into the atlas at the given cursor position.
///
/// Handles all swash content types: Mask (alpha-only), Color (RGBA), SubpixelMask (RGB).
fn write_glyph_pixels(
    pixels: &mut [u8],
    cursor_x: u32,
    cursor_y: u32,
    width: u32,
    height: u32,
    data: &[u8],
    content: SwashContent,
) {
    match content {
        SwashContent::Mask => {
            for y in 0..height {
                for x in 0..width {
                    let src_idx = (y * width + x) as usize;
                    let alpha = data.get(src_idx).copied().unwrap_or(0);
                    let dst_idx = (((cursor_y + y) * GLYPH_ATLAS_SIZE + cursor_x + x) * 4) as usize;
                    pixels[dst_idx] = 255;
                    pixels[dst_idx + 1] = 255;
                    pixels[dst_idx + 2] = 255;
                    pixels[dst_idx + 3] = alpha;
                }
            }
        }
        SwashContent::Color => {
            for y in 0..height {
                for x in 0..width {
                    let src_idx = ((y * width + x) * 4) as usize;
                    let dst_idx = (((cursor_y + y) * GLYPH_ATLAS_SIZE + cursor_x + x) * 4) as usize;
                    pixels[dst_idx] = data.get(src_idx).copied().unwrap_or(0);
                    pixels[dst_idx + 1] = data.get(src_idx + 1).copied().unwrap_or(0);
                    pixels[dst_idx + 2] = data.get(src_idx + 2).copied().unwrap_or(0);
                    pixels[dst_idx + 3] = data.get(src_idx + 3).copied().unwrap_or(0);
                }
            }
        }
        SwashContent::SubpixelMask => {
            for y in 0..height {
                for x in 0..width {
                    let src_idx = ((y * width + x) * 3) as usize;
                    let alpha = data.get(src_idx).copied().unwrap_or(0);
                    let dst_idx = (((cursor_y + y) * GLYPH_ATLAS_SIZE + cursor_x + x) * 4) as usize;
                    pixels[dst_idx] = 255;
                    pixels[dst_idx + 1] = 255;
                    pixels[dst_idx + 2] = 255;
                    pixels[dst_idx + 3] = alpha;
                }
            }
        }
    }
}

/// Shape text into a cosmic-text buffer and return total text height.
#[allow(clippy::too_many_arguments)]
fn shape_text_to_runs(
    font_system: &mut WowFontSystem,
    text: &str,
    font_path: Option<&str>,
    font_size: f32,
    bounds_width: f32,
    bounds_height: f32,
    word_wrap: bool,
    max_lines: u32,
) -> (Buffer, f32) {
    let line_height = (font_size * 1.2).ceil();
    let metrics = Metrics::new(font_size, line_height);
    let attrs = font_system.attrs_owned(font_path);

    let shape_width = if word_wrap && bounds_width > 0.0 { bounds_width } else { 10000.0 };

    let mut buffer = Buffer::new(&mut font_system.font_system, metrics);
    buffer.set_size(&mut font_system.font_system, Some(shape_width), Some(bounds_height));
    buffer.set_text(
        &mut font_system.font_system,
        text,
        &attrs.as_attrs(),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system.font_system, true);

    // Calculate total text height (for vertical justification).
    let mut runs: Vec<_> = buffer.layout_runs().collect();
    if max_lines > 0 {
        runs.truncate(max_lines as usize);
    }
    let total_height = if runs.len() <= 1 {
        line_height
    } else {
        let first_y = runs.first().map(|r| r.line_y).unwrap_or(0.0);
        runs.last().map(|run| run.line_y - first_y + line_height).unwrap_or(line_height)
    };

    // We only need total_height; the buffer is returned for glyph iteration.
    // The runs are re-collected from buffer later via layout_runs().
    drop(runs);
    (buffer, total_height)
}

/// Emit glyph quads for all layout runs with a given color and offset.
#[allow(clippy::too_many_arguments)]
fn emit_glyphs_for_runs(
    batch: &mut QuadBatch,
    glyph_atlas: &mut GlyphAtlas,
    font_system: &mut WowFontSystem,
    buffer: &Buffer,
    bounds: Rectangle,
    y_offset: f32,
    justify_h: TextJustify,
    glyph_color: [f32; 4],
    offset_x: f32,
    offset_y: f32,
    glyph_tex_index: i32,
    max_lines: u32,
) {
    let runs: Vec<_> = buffer.layout_runs().collect();
    let runs = if max_lines > 0 { &runs[..runs.len().min(max_lines as usize)] } else { &runs };

    for run in runs {
        let x_offset = if bounds.width > 0.0 {
            match justify_h {
                TextJustify::Left => 0.0,
                TextJustify::Center => (bounds.width - run.line_w) / 2.0,
                TextJustify::Right => bounds.width - run.line_w,
            }
        } else {
            0.0
        };

        for glyph in run.glyphs.iter() {
            let pg = glyph.physical((0.0, 0.0), 1.0);
            if let Some(entry) = glyph_atlas.ensure_glyph(font_system, pg.cache_key) {
                let glyph_x = bounds.x + x_offset + pg.x as f32 + offset_x;
                let glyph_y = bounds.y + y_offset + run.line_y + pg.y as f32 - entry.top as f32 + offset_y;
                let glyph_bounds = Rectangle::new(
                    iced::Point::new(glyph_x, glyph_y),
                    iced::Size::new(entry.width as f32, entry.height as f32),
                );
                let uv = Rectangle::new(
                    iced::Point::new(entry.uv_x, entry.uv_y),
                    iced::Size::new(entry.uv_w, entry.uv_h),
                );
                batch.push_quad(glyph_bounds, uv, glyph_color, glyph_tex_index, BlendMode::Alpha);
            }
        }
    }
}

/// Measure the height of text after word-wrapping within the given width.
///
/// Returns the total pixel height the text would occupy when rendered with
/// the specified font, size, and wrapping constraints.
pub fn measure_text_height(
    font_system: &mut WowFontSystem,
    text: &str,
    font_path: Option<&str>,
    font_size: f32,
    bounds_width: f32,
    word_wrap: bool,
) -> f32 {
    let stripped = crate::render::text::strip_wow_markup(text);
    if stripped.is_empty() {
        return 0.0;
    }
    let (_, total_height) = shape_text_to_runs(
        font_system, &stripped, font_path, font_size,
        bounds_width, 10000.0, word_wrap, 0,
    );
    total_height
}

/// Emit text quads into a QuadBatch.
///
/// Shapes the text, rasterizes glyphs into the atlas, and pushes textured quads.
/// The glyph atlas texture must be uploaded separately via `GlyphAtlas::texture_data()`.
#[allow(clippy::too_many_arguments)]
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
    shadow_color: Option<[f32; 4]>,
    shadow_offset: (f32, f32),
    outline: crate::widget::TextOutline,
    word_wrap: bool,
    max_lines: u32,
) {
    if text.is_empty() || bounds.height <= 0.0 {
        return;
    }

    let stripped = crate::render::text::strip_wow_markup(text);
    if stripped.is_empty() {
        return;
    }

    let (buffer, total_height) = shape_text_to_runs(
        font_system, &stripped, font_path, font_size,
        bounds.width, bounds.height, word_wrap, max_lines,
    );

    let y_offset = match justify_v {
        TextJustify::Left => 0.0,   // TOP
        TextJustify::Center => (bounds.height - total_height) / 2.0,
        TextJustify::Right => bounds.height - total_height, // BOTTOM
    };

    let emit = |batch: &mut QuadBatch, ga: &mut GlyphAtlas, fs: &mut WowFontSystem,
                c: [f32; 4], ox: f32, oy: f32| {
        emit_glyphs_for_runs(batch, ga, fs, &buffer, bounds, y_offset, justify_h, c, ox, oy, glyph_tex_index, max_lines);
    };

    // Render outline first (behind everything)
    if outline != crate::widget::TextOutline::None {
        let outline_color = [0.0_f32, 0.0, 0.0, color[3]];
        let d = match outline {
            crate::widget::TextOutline::Outline => 1.0_f32,
            crate::widget::TextOutline::ThickOutline => 2.0,
            crate::widget::TextOutline::None => unreachable!(),
        };
        for &(dx, dy) in &[(-d, 0.0), (d, 0.0), (0.0, -d), (0.0, d), (-d, -d), (d, -d), (-d, d), (d, d)] {
            emit(batch, glyph_atlas, font_system, outline_color, dx, dy);
        }
    }

    // Render shadow (behind main text, in front of outline)
    if let Some(sc) = shadow_color
        && sc[3] > 0.0 {
            emit(batch, glyph_atlas, font_system, sc, shadow_offset.0, shadow_offset.1);
        }

    // Render main text
    emit(batch, glyph_atlas, font_system, color, 0.0, 0.0);
}
