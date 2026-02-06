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
/// * `shadow_color` - Optional shadow color (render shadow if alpha > 0)
/// * `shadow_offset` - Shadow offset (x, y) in pixels
/// * `outline` - Text outline style (None, Outline, ThickOutline)
/// * `word_wrap` - Whether to wrap text at bounds width
/// * `max_lines` - Maximum lines to render (0 = unlimited)
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

    // Shape text with cosmic-text
    let line_height = (font_size * 1.2).ceil();
    let metrics = Metrics::new(font_size, line_height);
    let attrs = font_system.attrs_owned(font_path);

    // Use bounds.width for wrapping only when word_wrap is enabled and bounds has width.
    // Otherwise use a large width so text stays on a single line.
    let shape_width = if word_wrap && bounds.width > 0.0 {
        bounds.width
    } else {
        10000.0
    };

    let mut buffer = Buffer::new(&mut font_system.font_system, metrics);
    buffer.set_size(&mut font_system.font_system, Some(shape_width), Some(bounds.height));
    buffer.set_text(
        &mut font_system.font_system,
        &stripped,
        &attrs.as_attrs(),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system.font_system, true);

    // Calculate total text height for vertical justification.
    // For single-line text, use line_height. For multi-line, use last line's position + line_height.
    let mut layout_runs: Vec<_> = buffer.layout_runs().collect();
    if max_lines > 0 {
        layout_runs.truncate(max_lines as usize);
    }
    let num_lines = layout_runs.len();
    let total_height = if num_lines <= 1 {
        line_height
    } else {
        layout_runs.last().map(|run| run.line_y + line_height).unwrap_or(line_height)
    };

    // Vertical offset based on justification.
    // Always use bounds.height for centering, even if smaller than total_height.
    // This ensures text is visually centered within the widget bounds.
    let y_offset = match justify_v {
        TextJustify::Left => 0.0,   // TOP
        TextJustify::Center => (bounds.height - total_height) / 2.0,
        TextJustify::Right => bounds.height - total_height, // BOTTOM
    };

    // Check if we should render a shadow
    let has_shadow = shadow_color.map(|c| c[3] > 0.0).unwrap_or(false);

    // Helper closure to emit glyph quads with given color and offset.
    // Iterates the (possibly truncated) layout_runs collected above.
    let emit_glyphs = |batch: &mut QuadBatch,
                       glyph_atlas: &mut GlyphAtlas,
                       font_system: &mut WowFontSystem,
                       glyph_color: [f32; 4],
                       offset_x: f32,
                       offset_y: f32| {
        for run in &layout_runs {
            // Horizontal offset based on justification.
            // For explicit width, apply justification within bounds.
            // For width=0 (auto-sized FontStrings), text starts at bounds.x (LEFT behavior).
            // This matches WoW where auto-sized FontStrings flow from their anchor point.
            let run_width = run.line_w;
            let x_offset = if bounds.width > 0.0 {
                match justify_h {
                    TextJustify::Left => 0.0,
                    TextJustify::Center => (bounds.width - run_width) / 2.0,
                    TextJustify::Right => bounds.width - run_width,
                }
            } else {
                // Width=0: text starts at bounds.x regardless of justify_h
                // This is correct for auto-sized FontStrings positioned by single anchor
                0.0
            };

            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0.0, 0.0), 1.0);

                if let Some(entry) =
                    glyph_atlas.ensure_glyph(font_system, physical_glyph.cache_key)
                {
                    // Glyph positioning:
                    // physical_glyph.x is pen position
                    // physical_glyph.y includes baseline offset, entry.top is bitmap offset
                    let glyph_x =
                        bounds.x + x_offset + physical_glyph.x as f32 + offset_x;
                    let glyph_y = bounds.y
                        + y_offset
                        + run.line_y
                        + physical_glyph.y as f32
                        - entry.top as f32
                        + offset_y;

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
    };

    // Render outline first (behind everything)
    if outline != crate::widget::TextOutline::None {
        let outline_color = [0.0_f32, 0.0, 0.0, color[3]]; // Black outline with same alpha
        let d = match outline {
            crate::widget::TextOutline::Outline => 1.0_f32,
            crate::widget::TextOutline::ThickOutline => 2.0,
            crate::widget::TextOutline::None => unreachable!(),
        };
        // 8 compass directions for outline
        for &(dx, dy) in &[
            (-d, 0.0), (d, 0.0), (0.0, -d), (0.0, d),
            (-d, -d), (d, -d), (-d, d), (d, d),
        ] {
            emit_glyphs(batch, glyph_atlas, font_system, outline_color, dx, dy);
        }
    }

    // Render shadow (behind main text, in front of outline)
    if has_shadow {
        emit_glyphs(
            batch,
            glyph_atlas,
            font_system,
            shadow_color.unwrap(),
            shadow_offset.0,
            shadow_offset.1,
        );
    }

    // Render main text
    emit_glyphs(batch, glyph_atlas, font_system, color, 0.0, 0.0);
}
