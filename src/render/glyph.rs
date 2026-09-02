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

use super::font::{WowFontSystem, line_height_for_font_size};
use super::shader::{BlendMode, QuadBatch};
use crate::widget::TextJustify;

mod text_emit;
pub use text_emit::emit_text_quads;

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
    /// Swash placement offset from pen position to image left edge.
    left: i32,
    /// Swash placement offset from pen position to image top edge.
    top: i32,
}

/// Compute a u64 hash key for the shape cache from all inputs that affect glyph layout.
fn shape_cache_hash(
    text: &str,
    font_path: Option<&str>,
    font_size: f32,
    shape_width: f32,
    bounds_height: f32,
    max_lines: u32,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::hash::DefaultHasher::new();
    text.hash(&mut h);
    font_path.unwrap_or("").hash(&mut h);
    font_size.to_bits().hash(&mut h);
    shape_width.to_bits().hash(&mut h);
    bounds_height.to_bits().hash(&mut h);
    max_lines.hash(&mut h);
    h.finish()
}

/// A single glyph position extracted from a layout run.
#[derive(Clone)]
struct CachedGlyph {
    cache_key: CacheKey,
    x: i32,
    y: i32,
}

/// Extracted layout run data for cache replay.
#[derive(Clone)]
struct CachedLayoutRun {
    line_y: f32,
    line_w: f32,
    glyphs: Vec<CachedGlyph>,
}

/// Cached shape result with LRU tracking.
struct ShapeCacheEntry {
    runs: Vec<CachedLayoutRun>,
    total_height: f32,
    last_used: u64,
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
    /// Cache of shaped text layout runs keyed by u64 hash of shaping inputs.
    shape_cache: HashMap<u64, ShapeCacheEntry>,
    /// Generation counter for LRU eviction.
    shape_cache_generation: u64,
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
            shape_cache: HashMap::new(),
            shape_cache_generation: 0,
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

    /// Advance the cache generation counter and sweep stale entries.
    ///
    /// Call once per frame. Sweeps entries unused for 120 generations (~2s)
    /// every 60 generations (~1s at 60fps).
    pub fn advance_generation(&mut self) {
        self.shape_cache_generation += 1;
        if self.shape_cache_generation.is_multiple_of(60) {
            let generation = self.shape_cache_generation;
            self.shape_cache
                .retain(|_, entry| generation - entry.last_used < 120);
        }
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

        let (atlas_x, atlas_y) = self.reserve_glyph_slot(width, height)?;
        let entry = self.insert_rasterized_glyph(cache_key, image, atlas_x, atlas_y);
        Some(entry)
    }

    fn insert_rasterized_glyph(
        &mut self,
        cache_key: CacheKey,
        image: &cosmic_text::SwashImage,
        atlas_x: u32,
        atlas_y: u32,
    ) -> GlyphEntry {
        let width = image.placement.width;
        let height = image.placement.height;
        write_glyph_pixels(
            &mut self.pixels,
            atlas_x,
            atlas_y,
            width,
            height,
            &image.data,
            image.content,
        );

        let entry = build_glyph_entry(
            atlas_x,
            atlas_y,
            width,
            height,
            image.placement.left,
            image.placement.top,
        );
        self.entries.insert(cache_key, entry);
        self.dirty = true;
        entry
    }

    fn reserve_glyph_slot(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if self.cursor_x + width > GLYPH_ATLAS_SIZE {
            self.start_next_glyph_row();
        }

        if self.cursor_y + height > GLYPH_ATLAS_SIZE {
            tracing::warn!("Glyph atlas full ({} glyphs)", self.entries.len());
            return None;
        }

        let slot = (self.cursor_x, self.cursor_y);
        self.cursor_x += width + 1; // 1px padding
        self.row_height = self.row_height.max(height);
        Some(slot)
    }

    fn start_next_glyph_row(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += self.row_height + 1; // 1px padding
        self.row_height = 0;
    }

    fn insert_shape_cache_entry(
        &mut self,
        key: u64,
        runs: Vec<CachedLayoutRun>,
        total_height: f32,
    ) {
        let generation = self.shape_cache_generation;
        self.shape_cache.insert(
            key,
            ShapeCacheEntry {
                runs,
                total_height,
                last_used: generation,
            },
        );
    }
}

fn build_glyph_entry(
    atlas_x: u32,
    atlas_y: u32,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
) -> GlyphEntry {
    GlyphEntry {
        uv_x: atlas_x as f32 / GLYPH_ATLAS_SIZE as f32,
        uv_y: atlas_y as f32 / GLYPH_ATLAS_SIZE as f32,
        uv_w: width as f32 / GLYPH_ATLAS_SIZE as f32,
        uv_h: height as f32 / GLYPH_ATLAS_SIZE as f32,
        width,
        height,
        left,
        top,
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
            write_mask_glyph_pixels(pixels, cursor_x, cursor_y, width, height, data)
        }
        SwashContent::Color => {
            write_color_glyph_pixels(pixels, cursor_x, cursor_y, width, height, data);
        }
        SwashContent::SubpixelMask => {
            write_subpixel_glyph_pixels(pixels, cursor_x, cursor_y, width, height, data);
        }
    }
}

fn write_mask_glyph_pixels(
    pixels: &mut [u8],
    cursor_x: u32,
    cursor_y: u32,
    width: u32,
    height: u32,
    data: &[u8],
) {
    for pixel in glyph_pixel_positions(cursor_x, cursor_y, width, height) {
        let alpha = data.get(pixel.src_idx).copied().unwrap_or(0);
        write_rgba_pixel(pixels, pixel.dst_idx, [255, 255, 255, alpha]);
    }
}

fn write_color_glyph_pixels(
    pixels: &mut [u8],
    cursor_x: u32,
    cursor_y: u32,
    width: u32,
    height: u32,
    data: &[u8],
) {
    for pixel in glyph_pixel_positions(cursor_x, cursor_y, width, height) {
        let src_idx = pixel.src_idx * 4;
        let rgba = [
            data.get(src_idx).copied().unwrap_or(0),
            data.get(src_idx + 1).copied().unwrap_or(0),
            data.get(src_idx + 2).copied().unwrap_or(0),
            data.get(src_idx + 3).copied().unwrap_or(0),
        ];
        write_rgba_pixel(pixels, pixel.dst_idx, rgba);
    }
}

fn write_subpixel_glyph_pixels(
    pixels: &mut [u8],
    cursor_x: u32,
    cursor_y: u32,
    width: u32,
    height: u32,
    data: &[u8],
) {
    for pixel in glyph_pixel_positions(cursor_x, cursor_y, width, height) {
        let src_idx = pixel.src_idx * 3;
        let r = data.get(src_idx).copied().unwrap_or(0);
        let g = data.get(src_idx + 1).copied().unwrap_or(0);
        let b = data.get(src_idx + 2).copied().unwrap_or(0);
        write_rgba_pixel(
            pixels,
            pixel.dst_idx,
            [255, 255, 255, subpixel_mask_alpha(r, g, b)],
        );
    }
}

struct GlyphPixelPosition {
    src_idx: usize,
    dst_idx: usize,
}

fn glyph_pixel_positions(
    cursor_x: u32,
    cursor_y: u32,
    width: u32,
    height: u32,
) -> impl Iterator<Item = GlyphPixelPosition> {
    (0..height).flat_map(move |y| {
        (0..width).map(move |x| GlyphPixelPosition {
            src_idx: (y * width + x) as usize,
            dst_idx: glyph_atlas_pixel_offset(cursor_x + x, cursor_y + y),
        })
    })
}

fn glyph_atlas_pixel_offset(x: u32, y: u32) -> usize {
    ((y * GLYPH_ATLAS_SIZE + x) * 4) as usize
}

fn write_rgba_pixel(pixels: &mut [u8], dst_idx: usize, rgba: [u8; 4]) {
    pixels[dst_idx..dst_idx + 4].copy_from_slice(&rgba);
}

fn subpixel_mask_alpha(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 + g as u16 + b as u16) / 3) as u8
}

/// Shape text into a cosmic-text buffer and return total text height.
fn shape_text_to_runs(
    font_system: &mut WowFontSystem,
    shape: TextShapeRequest<'_>,
) -> (Buffer, f32) {
    let line_height = line_height_for_font_size(shape.font_size)
        .expect("shape_text_to_runs requires a positive font size");
    let shape_width = text_shape_width(&shape);
    let mut buffer = build_text_shape_buffer(font_system, &shape, line_height, shape_width);
    buffer.shape_until_scroll(&mut font_system.font_system, true);

    let total_height = shaped_text_total_height(&buffer, shape.max_lines, line_height);
    (buffer, total_height)
}

fn text_shape_width(shape: &TextShapeRequest<'_>) -> f32 {
    if shape.word_wrap && shape.bounds_width > 0.0 {
        shape.bounds_width
    } else {
        10000.0
    }
}

fn build_text_shape_buffer(
    font_system: &mut WowFontSystem,
    shape: &TextShapeRequest<'_>,
    line_height: f32,
    shape_width: f32,
) -> Buffer {
    let metrics = Metrics::new(shape.font_size, line_height);
    let attrs = font_system.attrs_owned(shape.font_path);
    let mut buffer = Buffer::new(&mut font_system.font_system, metrics);
    buffer.set_size(
        &mut font_system.font_system,
        Some(shape_width),
        Some(shape.bounds_height),
    );
    buffer.set_text(
        &mut font_system.font_system,
        shape.text,
        &attrs.as_attrs(),
        Shaping::Advanced,
        None,
    );
    buffer
}

fn shaped_text_total_height(buffer: &Buffer, max_lines: u32, line_height: f32) -> f32 {
    let mut runs: Vec<_> = buffer.layout_runs().collect();
    if max_lines > 0 {
        runs.truncate(max_lines as usize);
    }
    text_total_height(&runs, line_height)
}

fn text_total_height(runs: &[cosmic_text::LayoutRun<'_>], line_height: f32) -> f32 {
    if runs.len() <= 1 {
        return line_height;
    }

    let first_y = runs.first().map(|run| run.line_y).unwrap_or(0.0);
    runs.last()
        .map(|run| run.line_y - first_y + line_height)
        .unwrap_or(line_height)
}

struct TextShapeRequest<'a> {
    text: &'a str,
    font_path: Option<&'a str>,
    font_size: f32,
    bounds_width: f32,
    bounds_height: f32,
    word_wrap: bool,
    max_lines: u32,
}

/// Extract glyph positions from layout runs into cacheable data.
fn extract_layout_runs(buffer: &Buffer, max_lines: u32) -> Vec<CachedLayoutRun> {
    let runs: Vec<_> = buffer.layout_runs().collect();
    let runs_slice = if max_lines > 0 {
        &runs[..runs.len().min(max_lines as usize)]
    } else {
        &runs
    };
    runs_slice
        .iter()
        .map(|run| {
            let glyphs = run
                .glyphs
                .iter()
                .map(|g| {
                    let pg = g.physical((0.0, 0.0), 1.0);
                    CachedGlyph {
                        cache_key: pg.cache_key,
                        x: pg.x,
                        y: pg.y,
                    }
                })
                .collect();
            CachedLayoutRun {
                line_y: run.line_y,
                line_w: run.line_w,
                glyphs,
            }
        })
        .collect()
}

pub(super) struct GlyphCacheEmitRequest<'a> {
    runs: &'a [CachedLayoutRun],
    bounds: Rectangle,
    y_offset: f32,
    justify_h: TextJustify,
    glyph_color: [f32; 4],
    offset: (f32, f32),
    glyph_tex_index: i32,
}

/// Emit glyph quads from cached layout runs with a given color and offset.
fn emit_glyphs_from_cache(
    batch: &mut QuadBatch,
    glyph_atlas: &mut GlyphAtlas,
    font_system: &mut WowFontSystem,
    request: GlyphCacheEmitRequest<'_>,
) {
    for run in request.runs {
        let x_offset = cached_run_x_offset(&request, run);
        for glyph in &run.glyphs {
            emit_cached_glyph(
                batch,
                glyph_atlas,
                font_system,
                &request,
                run,
                x_offset,
                glyph,
            );
        }
    }
}

fn cached_run_x_offset(request: &GlyphCacheEmitRequest<'_>, run: &CachedLayoutRun) -> f32 {
    if request.bounds.width <= 0.0 {
        return 0.0;
    }

    match request.justify_h {
        TextJustify::Left => 0.0,
        TextJustify::Center => (request.bounds.width - run.line_w) / 2.0,
        TextJustify::Right => request.bounds.width - run.line_w,
    }
}

fn emit_cached_glyph(
    batch: &mut QuadBatch,
    glyph_atlas: &mut GlyphAtlas,
    font_system: &mut WowFontSystem,
    request: &GlyphCacheEmitRequest<'_>,
    run: &CachedLayoutRun,
    x_offset: f32,
    glyph: &CachedGlyph,
) {
    let Some(entry) = glyph_atlas.ensure_glyph(font_system, glyph.cache_key) else {
        return;
    };

    batch.push_quad(
        cached_glyph_bounds(request, run, x_offset, glyph, entry),
        cached_glyph_uv(entry),
        request.glyph_color,
        request.glyph_tex_index,
        BlendMode::Alpha,
    );
}

fn cached_glyph_bounds(
    request: &GlyphCacheEmitRequest<'_>,
    run: &CachedLayoutRun,
    x_offset: f32,
    glyph: &CachedGlyph,
    entry: GlyphEntry,
) -> Rectangle {
    // The bitmap was rasterized for whole device pixels (plus the glyph's
    // sub-pixel bin within its run); the frame origin, the justification
    // offset, the line position and the shadow offset are fractional at any
    // UI scale, and drawing the bitmap at a fractional position lets the
    // bilinear glyph sampler smear every edge over two pixels. The client
    // snaps text to the pixel grid; so does this. The rounding is the same
    // for every glyph of a run, so the run keeps its shape.
    let glyph_x =
        (request.bounds.x + x_offset + glyph.x as f32 + entry.left as f32 + request.offset.0)
            .round();
    let glyph_y = (request.bounds.y + request.y_offset + run.line_y + glyph.y as f32
        - entry.top as f32
        + request.offset.1)
        .round();

    Rectangle::new(
        iced::Point::new(glyph_x, glyph_y),
        iced::Size::new(entry.width as f32, entry.height as f32),
    )
}

fn cached_glyph_uv(entry: GlyphEntry) -> Rectangle {
    Rectangle::new(
        iced::Point::new(entry.uv_x, entry.uv_y),
        iced::Size::new(entry.uv_w, entry.uv_h),
    )
}

/// Measure the height of text after word-wrapping within the given width.
///
/// Returns the total pixel height the text would occupy when rendered with
/// the specified font, size, and wrapping constraints. Uses the shape cache
/// to avoid re-shaping text that has already been measured.
pub fn measure_text_height(
    font_system: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    text: &str,
    font_path: Option<&str>,
    font_size: f32,
    bounds_width: f32,
    word_wrap: bool,
) -> f32 {
    let stripped = crate::render::strip_wow_markup(text);
    if stripped.is_empty() {
        return 0.0;
    }
    let shape = TextShapeRequest {
        text: &stripped,
        font_path,
        font_size,
        bounds_width,
        bounds_height: 10000.0,
        word_wrap,
        max_lines: 0,
    };
    let key = text_measure_cache_key(&shape);
    if let Some(entry) = glyph_atlas.shape_cache.get_mut(&key) {
        entry.last_used = glyph_atlas.shape_cache_generation;
        return entry.total_height;
    }
    let (buffer, total_height) = shape_text_to_runs(font_system, shape);
    let runs = extract_layout_runs(&buffer, 0);
    glyph_atlas.insert_shape_cache_entry(key, runs, total_height);
    total_height
}

fn text_measure_cache_key(shape: &TextShapeRequest<'_>) -> u64 {
    let shape_width = text_shape_width(shape);
    shape_cache_hash(
        shape.text,
        shape.font_path,
        shape.font_size,
        shape_width,
        shape.bounds_height,
        shape.max_lines,
    )
}

#[cfg(test)]
mod tests;
