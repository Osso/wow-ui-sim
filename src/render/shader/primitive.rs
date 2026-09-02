//! WoW UI shader primitive implementation.

use super::quad::TextureRequestHandle;
use super::{QuadBatch, WowUiPipeline};
use iced::Rectangle;
use iced::widget::shader::{self, Viewport};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub use super::primitive_textures::{
    GpuBcTextureData, GpuTextureData, LoadedTexture, TextureLoadTelemetry, load_texture_or_crop,
    load_texture_prefer_bc, load_texture_prefer_bc_with_telemetry,
};
#[cfg(test)]
pub(crate) use super::primitive_textures::{
    bc_texture_dimensions_fit_gpu_atlas, decode_crop_request,
};

#[derive(Debug, Default)]
pub(crate) struct TextureRequestTracker {
    handles: HashMap<String, Vec<TextureRequestHandle>>,
}

impl TextureRequestTracker {
    fn handles_for_path_mut(&mut self, path: &str) -> &mut Vec<TextureRequestHandle> {
        self.handles.entry(path.to_string()).or_default()
    }

    pub(crate) fn register_request(&mut self, request: &crate::render::TextureRequest) {
        self.handles_for_path_mut(&request.path)
            .push(request.handle.clone());
    }

    pub(crate) fn register_batch(&mut self, batch: &QuadBatch) {
        for request in batch
            .texture_requests
            .iter()
            .chain(&batch.mask_texture_requests)
        {
            self.register_request(request);
        }
    }

    pub(crate) fn mark_failed(&mut self, path: &str) {
        if let Some(handles) = self.handles.get(path) {
            for handle in handles {
                if handle.is_pending() && !handle.is_staged() {
                    handle.mark_failed();
                }
            }
        }
    }

    pub(crate) fn mark_staged(&mut self, path: &str) {
        if let Some(handles) = self.handles.get(path) {
            for handle in handles {
                if handle.is_pending() {
                    handle.mark_staged();
                }
            }
        }
    }

    pub(crate) fn mark_ready(&mut self, path: &str) {
        if let Some(handles) = self.handles.get(path) {
            for handle in handles {
                if handle.is_pending() || handle.is_staged() {
                    handle.mark_ready();
                }
            }
        }
    }

    pub(crate) fn mark_upload_retry(&mut self, path: &str) {
        if let Some(handles) = self.handles.get(path) {
            for handle in handles {
                if handle.is_pending() || handle.is_staged() {
                    handle.mark_force_rgba();
                }
            }
        }
    }

    pub(crate) fn needs_force_rgba_retry(&self, path: &str) -> bool {
        self.handles
            .get(path)
            .is_some_and(|handles| handles.iter().any(TextureRequestHandle::needs_force_rgba))
    }

    pub(crate) fn ready_count(&self) -> usize {
        self.handles
            .values()
            .map(|handles| handles.iter().filter(|handle| handle.is_ready()).count())
            .sum()
    }

    pub(crate) fn staged_count(&self) -> usize {
        self.handles
            .values()
            .map(|handles| handles.iter().filter(|handle| handle.is_staged()).count())
            .sum()
    }
}

#[cfg(test)]
#[path = "primitive_tests.rs"]
mod tests;

use crate::widget::FrameStrata;

/// Primitive data for rendering WoW UI frames.
///
/// Per-strata batches: each `FrameStrata` gets its own vertex/index data on
/// the GPU.  Only dirty strata carry `Some(batch)` — clean strata are `None`
/// and the pipeline keeps their GPU buffers from the previous frame.
#[derive(Debug)]
pub struct WowUiPrimitive {
    /// Per-strata quad batches. Index = `FrameStrata::as_index()`.
    /// `Some` = dirty (re-upload), `None` = clean (pipeline keeps old buffer).
    pub strata_batches: [Option<Arc<QuadBatch>>; FrameStrata::COUNT],
    /// Small overlay batch (cursor, hover highlight) appended after world quads.
    pub overlay: QuadBatch,
    /// Background clear color.
    pub clear_color: [f32; 4],
    /// Texture data to upload (path -> image data).
    pub textures: Vec<GpuTextureData>,
    /// BC-compressed texture data to upload directly to the GPU BC atlas.
    pub bc_textures: Vec<GpuBcTextureData>,
    /// Glyph atlas RGBA data for text rendering (2048x2048).
    pub glyph_atlas_data: Option<Vec<u8>>,
    /// Size of the glyph atlas (width = height).
    pub glyph_atlas_size: u32,
    /// Shared app-side tracker for per-request texture upload handles.
    pub(crate) texture_requests: Option<Arc<Mutex<TextureRequestTracker>>>,
}

impl WowUiPrimitive {
    /// Create a primitive with a single merged batch placed in strata 0 (World).
    ///
    /// Used by the headless renderer and tests where per-strata separation
    /// isn't needed — all quads are already in draw order.
    pub fn new_merged(quads: Arc<QuadBatch>) -> Self {
        let mut strata_batches: [Option<Arc<QuadBatch>>; FrameStrata::COUNT] =
            std::array::from_fn(|_| None);
        strata_batches[0] = Some(quads);
        Self {
            strata_batches,
            overlay: QuadBatch::new(),
            clear_color: [0.10, 0.11, 0.14, 1.0],
            textures: Vec::new(),
            bc_textures: Vec::new(),
            glyph_atlas_data: None,
            glyph_atlas_size: 0,
            texture_requests: None,
        }
    }

    /// Create a merged primitive with texture data (headless path).
    pub fn new_merged_with_textures(
        quads: Arc<QuadBatch>,
        textures: Vec<GpuTextureData>,
        bc_textures: Vec<GpuBcTextureData>,
    ) -> Self {
        let mut p = Self::new_merged(quads);
        p.textures = textures;
        p.bc_textures = bc_textures;
        p
    }

    /// Create an empty primitive.
    pub fn empty() -> Self {
        Self {
            strata_batches: std::array::from_fn(|_| None),
            overlay: QuadBatch::new(),
            clear_color: [0.10, 0.11, 0.14, 1.0],
            textures: Vec::new(),
            bc_textures: Vec::new(),
            glyph_atlas_data: None,
            glyph_atlas_size: 0,
            texture_requests: None,
        }
    }

    fn dirty_strata_count(&self) -> usize {
        self.strata_batches
            .iter()
            .filter(|batch| batch.is_some())
            .count()
    }

    fn prepare_frame_textures(
        &self,
        pipeline: &mut WowUiPipeline,
        queue: &wgpu::Queue,
    ) -> TexturePrepareStats {
        crate::logging::set_blocking_phase("prepare_textures");
        let started = Instant::now();
        let before = texture_request_counts(self.texture_requests.as_ref());
        let upload_outcome = upload_pending_textures(
            pipeline,
            queue,
            &self.textures,
            &self.bc_textures,
            &self.glyph_atlas_data,
            self.glyph_atlas_size,
        );
        let retry_count = upload_outcome.retry_paths.len();
        let force_rgba_retry_count = upload_outcome.force_rgba_retry_paths.len();
        record_texture_upload_outcome(upload_outcome, self.texture_requests.as_ref());
        let after = texture_request_counts(self.texture_requests.as_ref());
        TexturePrepareStats {
            elapsed: started.elapsed(),
            before,
            after,
            retry_count,
            force_rgba_retry_count,
        }
    }

    fn trace_prepare_textures(&self, stats: TexturePrepareStats) {
        if !crate::logging::gui_trace_enabled() {
            return;
        }
        crate::logging::eprintln_gui_trace(&format!(
            "prepare ready_before={} ready_after={} staged_before={} staged_after={} retry={} force_rgba_retry={} dirty_strata={} new_rgba={} new_bc={}",
            stats.before.ready,
            stats.after.ready,
            stats.before.staged,
            stats.after.staged,
            stats.retry_count,
            stats.force_rgba_retry_count,
            self.dirty_strata_count(),
            self.textures.len(),
            self.bc_textures.len()
        ));
    }

    fn log_slow_prepare(
        &self,
        prepare_elapsed: Duration,
        texture_stats: TexturePrepareStats,
        strata_elapsed: Duration,
        overlay_elapsed: Duration,
    ) {
        if prepare_elapsed < Duration::from_millis(50) {
            return;
        }
        crate::logging::eprintln_elapsed(&format!(
            "[prepare] total={prepare_elapsed:.1?} textures={:.1?} strata={strata_elapsed:.1?} overlay={overlay_elapsed:.1?} dirty_strata={} new_rgba={} new_bc={}",
            texture_stats.elapsed,
            self.dirty_strata_count(),
            self.textures.len(),
            self.bc_textures.len()
        ));
    }
}

#[derive(Clone, Copy, Default)]
struct TextureRequestCounts {
    ready: usize,
    staged: usize,
}

#[derive(Clone, Copy)]
struct TexturePrepareStats {
    elapsed: Duration,
    before: TextureRequestCounts,
    after: TextureRequestCounts,
    retry_count: usize,
    force_rgba_retry_count: usize,
}

fn texture_request_counts(
    texture_requests: Option<&Arc<Mutex<TextureRequestTracker>>>,
) -> TextureRequestCounts {
    texture_requests
        .and_then(|tracker| {
            tracker.lock().ok().map(|tracker| TextureRequestCounts {
                ready: tracker.ready_count(),
                staged: tracker.staged_count(),
            })
        })
        .unwrap_or_default()
}

fn physical_prepare_bounds(bounds: &Rectangle, scale: f32) -> Rectangle {
    Rectangle::new(
        iced::Point::new(bounds.x * scale, bounds.y * scale),
        iced::Size::new(bounds.width * scale, bounds.height * scale),
    )
}

fn upload_dirty_strata_batches(
    pipeline: &mut WowUiPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scale: f32,
    strata_batches: &[Option<Arc<QuadBatch>>; FrameStrata::COUNT],
) -> Duration {
    crate::logging::set_blocking_phase("prepare_strata");
    let started = Instant::now();
    for (i, batch_opt) in strata_batches.iter().enumerate() {
        if let Some(batch) = batch_opt {
            let resolved = resolve_and_scale_quads(pipeline, batch, scale);
            pipeline.upload_strata(device, queue, i, &resolved);
        }
    }
    started.elapsed()
}

fn upload_overlay_batch(
    pipeline: &mut WowUiPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scale: f32,
    overlay: &QuadBatch,
) -> Duration {
    crate::logging::set_blocking_phase("prepare_overlay");
    let started = Instant::now();
    let overlay_idx = FrameStrata::COUNT;
    if overlay.vertices.is_empty() {
        pipeline.clear_strata(overlay_idx);
    } else {
        let resolved = resolve_and_scale_quads(pipeline, overlay, scale);
        pipeline.upload_strata(device, queue, overlay_idx, &resolved);
    }
    started.elapsed()
}

#[derive(Debug, Default)]
struct TextureUploadOutcome {
    ready_paths: HashSet<String>,
    retry_paths: HashSet<String>,
    force_rgba_retry_paths: HashSet<String>,
}

/// Upload pending textures and glyph atlas data to the GPU atlas.
fn upload_pending_textures(
    pipeline: &mut WowUiPipeline,
    queue: &wgpu::Queue,
    textures: &[GpuTextureData],
    bc_textures: &[GpuBcTextureData],
    glyph_atlas_data: &Option<Vec<u8>>,
    glyph_atlas_size: u32,
) -> TextureUploadOutcome {
    let atlas = pipeline.texture_atlas_mut();
    let mut outcome = TextureUploadOutcome::default();
    upload_rgba_textures(atlas, queue, textures, &mut outcome);
    upload_bc_textures(atlas, queue, bc_textures, &mut outcome);
    upload_glyph_atlas_if_present(atlas, queue, glyph_atlas_data, glyph_atlas_size);
    log_gpu_memory_once(atlas);
    outcome
}

fn record_texture_upload_outcome(
    outcome: TextureUploadOutcome,
    texture_requests: Option<&Arc<Mutex<TextureRequestTracker>>>,
) {
    let Some(texture_requests) = texture_requests else {
        return;
    };
    let Ok(mut texture_requests) = texture_requests.lock() else {
        return;
    };
    for path in outcome.ready_paths {
        texture_requests.mark_ready(&path);
    }
    for path in outcome.retry_paths {
        texture_requests.mark_upload_retry(&path);
    }
    for path in outcome.force_rgba_retry_paths {
        texture_requests.mark_upload_retry(&path);
    }
}

fn upload_rgba_textures(
    atlas: &mut crate::render::shader::atlas::GpuTextureAtlas,
    queue: &wgpu::Queue,
    textures: &[GpuTextureData],
    outcome: &mut TextureUploadOutcome,
) {
    for tex_data in textures {
        if atlas.get(&tex_data.path).is_some() {
            outcome.ready_paths.insert(tex_data.path.clone());
            continue;
        }
        if atlas
            .upload(
                queue,
                &tex_data.path,
                tex_data.width,
                tex_data.height,
                tex_data.rgba.as_ref(),
            )
            .is_some()
        {
            outcome.ready_paths.insert(tex_data.path.clone());
        } else {
            outcome.retry_paths.insert(tex_data.path.clone());
        }
    }
}

fn upload_bc_textures(
    atlas: &mut crate::render::shader::atlas::GpuTextureAtlas,
    queue: &wgpu::Queue,
    bc_textures: &[GpuBcTextureData],
    outcome: &mut TextureUploadOutcome,
) {
    for bc_data in bc_textures {
        let already_uploaded =
            atlas.get_bc(&bc_data.path).is_some() || atlas.get(&bc_data.path).is_some();
        if already_uploaded {
            outcome.ready_paths.insert(bc_data.path.clone());
            continue;
        }
        if atlas
            .upload_bc(
                queue,
                &bc_data.path,
                bc_data.width,
                bc_data.height,
                bc_data.bc_data.as_ref(),
                bc_data.bc_format,
            )
            .is_some()
        {
            outcome.ready_paths.insert(bc_data.path.clone());
        } else {
            outcome.retry_paths.insert(bc_data.path.clone());
            outcome.force_rgba_retry_paths.insert(bc_data.path.clone());
        }
    }
}

fn upload_glyph_atlas_if_present(
    atlas: &mut crate::render::shader::atlas::GpuTextureAtlas,
    queue: &wgpu::Queue,
    glyph_atlas_data: &Option<Vec<u8>>,
    glyph_atlas_size: u32,
) {
    let Some(glyph_data) = glyph_atlas_data else {
        return;
    };
    atlas.upload_glyph_atlas(queue, glyph_data, glyph_atlas_size);
}

/// Log GPU atlas memory usage once after the first batch of textures.
fn log_gpu_memory_once(atlas: &crate::render::shader::atlas::GpuTextureAtlas) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static LOGGED: AtomicBool = AtomicBool::new(false);
    if !crate::logging::texture_load_debug_enabled() {
        return;
    }
    if LOGGED.swap(true, Ordering::Relaxed) {
        return;
    }
    let stats = atlas.memory_stats();
    eprintln!(
        "{} [GPU] Atlas memory: {:.0} MB allocated, {:.1} MB used | slots: 64px={} 128px={} 256px={} 512px={} 2048px={}",
        crate::logging::global_elapsed_prefix(),
        stats.allocated_bytes as f64 / (1024.0 * 1024.0),
        stats.used_bytes as f64 / (1024.0 * 1024.0),
        stats.used_slots[0],
        stats.used_slots[1],
        stats.used_slots[2],
        stats.used_slots[3],
        stats.used_slots[4],
    );
}

/// Resolve pending texture indices (-2) and scale vertex positions to physical pixels.
fn resolve_and_scale_quads(
    pipeline: &mut WowUiPipeline,
    quads: &QuadBatch,
    scale: f32,
) -> QuadBatch {
    let mut resolved = quads.clone();
    let atlas = pipeline.texture_atlas_mut();
    resolve_texture_requests(atlas, &quads.texture_requests, &mut resolved.vertices);
    resolve_mask_requests(atlas, &quads.mask_texture_requests, &mut resolved.vertices);
    clear_pending_and_scale(&mut resolved.vertices, scale);
    resolved
}

/// Remap primary texture UVs for resolved atlas entries (RGBA or BC).
fn resolve_texture_requests(
    atlas: &crate::render::shader::atlas::GpuTextureAtlas,
    requests: &[crate::render::TextureRequest],
    vertices: &mut [crate::render::QuadVertex],
) {
    for request in requests {
        if let Some(entry) = resolved_texture_entry(atlas, &request.path) {
            apply_resolved_texture_entry(
                request_vertices(request, vertices),
                entry,
                request.use_uv_inset,
            );
        }
    }
}

fn request_vertices<'a>(
    request: &crate::render::TextureRequest,
    vertices: &'a mut [crate::render::QuadVertex],
) -> &'a mut [crate::render::QuadVertex] {
    let start = request.vertex_start as usize;
    let end = start + request.vertex_count as usize;
    &mut vertices[start..end]
}

#[derive(Debug, Clone, Copy)]
enum ResolvedTextureEntry {
    Rgba(crate::render::shader::atlas::TextureEntry),
    Bc(crate::render::shader::atlas::BcTextureEntry),
}

fn resolved_texture_entry(
    atlas: &crate::render::shader::atlas::GpuTextureAtlas,
    path: &str,
) -> Option<ResolvedTextureEntry> {
    atlas
        .get(path)
        .copied()
        .map(ResolvedTextureEntry::Rgba)
        .or_else(|| atlas.get_bc(path).copied().map(ResolvedTextureEntry::Bc))
}

fn apply_resolved_texture_entry(
    vertices: &mut [crate::render::QuadVertex],
    entry: ResolvedTextureEntry,
    use_uv_inset: bool,
) {
    match entry {
        ResolvedTextureEntry::Rgba(entry) => apply_rgba_entry(vertices, &entry, use_uv_inset),
        ResolvedTextureEntry::Bc(entry) => apply_bc_entry(vertices, &entry),
    }
}

fn apply_rgba_entry(
    vertices: &mut [crate::render::QuadVertex],
    entry: &crate::render::shader::atlas::TextureEntry,
    use_uv_inset: bool,
) {
    let tex_idx = entry.tex_index();
    for vertex in vertices.iter_mut() {
        if vertex.tex_index == -2 {
            vertex.tex_index = tex_idx;
            vertex.tex_coords[0] = remap_entry_uv(
                vertex.tex_coords[0],
                UvRemap::entry_axis(entry.uv_x, entry.uv_width, entry.original_width, entry.tier)
                    .with_inset(use_uv_inset),
            );
            vertex.tex_coords[1] = remap_entry_uv(
                vertex.tex_coords[1],
                UvRemap::entry_axis(
                    entry.uv_y,
                    entry.uv_height,
                    entry.original_height,
                    entry.tier,
                )
                .with_inset(use_uv_inset),
            );
        }
    }
}

fn apply_bc_entry(
    vertices: &mut [crate::render::QuadVertex],
    bc_entry: &crate::render::shader::atlas::BcTextureEntry,
) {
    let tex_idx = bc_entry.tex_index();
    for vertex in vertices.iter_mut() {
        if vertex.tex_index == -2 {
            vertex.tex_index = tex_idx;
            vertex.tex_coords[0] = remap_bc_entry_uv(
                vertex.tex_coords[0],
                bc_entry.uv_x,
                bc_entry.uv_width,
                bc_entry.original_width,
            );
            vertex.tex_coords[1] = remap_bc_entry_uv(
                vertex.tex_coords[1],
                bc_entry.uv_y,
                bc_entry.uv_height,
                bc_entry.original_height,
            );
        }
    }
}

/// Remap mask texture UVs for resolved atlas entries (RGBA or BC).
fn resolve_mask_requests(
    atlas: &crate::render::shader::atlas::GpuTextureAtlas,
    requests: &[crate::render::TextureRequest],
    vertices: &mut [crate::render::QuadVertex],
) {
    for request in requests {
        if let Some(entry) = resolved_texture_entry(atlas, &request.path) {
            apply_resolved_mask_entry(
                request_vertices(request, vertices),
                entry,
                request.use_uv_inset,
            );
        }
    }
}

fn apply_resolved_mask_entry(
    vertices: &mut [crate::render::QuadVertex],
    entry: ResolvedTextureEntry,
    use_uv_inset: bool,
) {
    match entry {
        ResolvedTextureEntry::Rgba(entry) => apply_rgba_mask_entry(vertices, &entry, use_uv_inset),
        ResolvedTextureEntry::Bc(entry) => apply_bc_mask_entry(vertices, &entry),
    }
}

fn apply_rgba_mask_entry(
    vertices: &mut [crate::render::QuadVertex],
    entry: &crate::render::shader::atlas::TextureEntry,
    use_uv_inset: bool,
) {
    let tex_idx = entry.tex_index();
    // The coverage encoding was guessed from the mask's file name before its
    // pixels existed. Now that the entry is resolved, decide it from the
    // data: a mask whose alpha is uniformly opaque carries coverage in RGB,
    // anything else carries it in alpha. Both branches are needed - WoW ships
    // both encodings - and the flag has to be cleared as well as set, or the
    // name-based guess wins where it was wrong. See
    // docs/wiki/investigations/mask-coverage-encoding.md.
    let alpha_coverage = !entry.alpha_uniformly_opaque;
    for vertex in vertices.iter_mut() {
        if vertex.mask_tex_index == -2 {
            vertex.mask_tex_index = tex_idx;
            if alpha_coverage {
                vertex.flags |= crate::render::shader::FLAG_MASK_ALPHA_COVERAGE;
            } else {
                vertex.flags &= !crate::render::shader::FLAG_MASK_ALPHA_COVERAGE;
            }
            vertex.mask_tex_coords[0] = remap_entry_uv(
                vertex.mask_tex_coords[0],
                UvRemap::entry_axis(entry.uv_x, entry.uv_width, entry.original_width, entry.tier)
                    .with_inset(use_uv_inset),
            );
            vertex.mask_tex_coords[1] = remap_entry_uv(
                vertex.mask_tex_coords[1],
                UvRemap::entry_axis(
                    entry.uv_y,
                    entry.uv_height,
                    entry.original_height,
                    entry.tier,
                )
                .with_inset(use_uv_inset),
            );
        }
    }
}

fn apply_bc_mask_entry(
    vertices: &mut [crate::render::QuadVertex],
    entry: &crate::render::shader::atlas::BcTextureEntry,
) {
    let tex_idx = entry.tex_index();
    for vertex in vertices.iter_mut() {
        if vertex.mask_tex_index == -2 {
            vertex.mask_tex_index = tex_idx;
            vertex.mask_tex_coords[0] = remap_bc_entry_uv(
                vertex.mask_tex_coords[0],
                entry.uv_x,
                entry.uv_width,
                entry.original_width,
            );
            vertex.mask_tex_coords[1] = remap_bc_entry_uv(
                vertex.mask_tex_coords[1],
                entry.uv_y,
                entry.uv_height,
                entry.original_height,
            );
        }
    }
}

#[derive(Clone, Copy)]
struct UvRemap {
    base_uv: f32,
    span_uv: f32,
    original_size: u32,
    use_uv_inset: bool,
    cell_size: u32,
}

impl UvRemap {
    fn entry_axis(base_uv: f32, span_uv: f32, original_size: u32, tier: u32) -> Self {
        Self {
            base_uv,
            span_uv,
            original_size,
            use_uv_inset: true,
            cell_size: crate::render::shader::atlas::TIER_SIZES[tier as usize],
        }
    }

    fn with_inset(self, use_uv_inset: bool) -> Self {
        Self {
            use_uv_inset,
            ..self
        }
    }
}

/// Half-texel UV inset for one atlas axis.
///
/// A single-texel axis has no interior: under the bilinear sampler its only
/// correct sample point is the texel centre, so the inset is half the span
/// regardless of `use_uv_inset` and the quad's whole width maps to that one
/// point. Spanning the texel edge to edge instead blends up to 50% with the
/// neighbouring atlas cell (the 1-px tiling strips of the character-frame
/// tabs showed this as a luminance ramp over the left half of the strip).
fn half_texel_inset(span_uv: f32, uploaded_size: f32, use_uv_inset: bool) -> f32 {
    if uploaded_size <= 1.0 {
        span_uv * 0.5
    } else if use_uv_inset {
        (span_uv * 0.5 / uploaded_size).min(span_uv * 0.5)
    } else {
        0.0
    }
}

fn remap_entry_uv(local_uv: f32, remap: UvRemap) -> f32 {
    let uploaded_size = remap.original_size.min(remap.cell_size).max(1) as f32;
    let inset = half_texel_inset(remap.span_uv, uploaded_size, remap.use_uv_inset);
    remap.base_uv + inset + local_uv * (remap.span_uv - inset * 2.0).max(0.0)
}

/// Remap UV for BC atlas entries (fixed 256x256 cell size).
fn remap_bc_entry_uv(local_uv: f32, base_uv: f32, span_uv: f32, original_size: u32) -> f32 {
    let cell_size = crate::render::shader::atlas::BC_CELL_SIZE;
    let uploaded_size = original_size.min(cell_size).max(1) as f32;
    let inset = half_texel_inset(span_uv, uploaded_size, true);
    base_uv + inset + local_uv * (span_uv - inset * 2.0).max(0.0)
}

/// Hide unresolved textures and scale positions to physical pixels.
fn clear_pending_and_scale(vertices: &mut [crate::render::QuadVertex], scale: f32) {
    for vertex in vertices.iter_mut() {
        if vertex.tex_index == -2 {
            vertex.color[3] = 0.0;
            vertex.tex_index = -1;
        }
        if vertex.mask_tex_index == -2 {
            vertex.mask_tex_index = -1;
        }
        vertex.position[0] *= scale;
        vertex.position[1] *= scale;
    }
}

impl shader::Primitive for WowUiPrimitive {
    type Pipeline = WowUiPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let scale = viewport.scale_factor();
        let physical_bounds = physical_prepare_bounds(bounds, scale);
        let prepare_started = Instant::now();
        let texture_stats = self.prepare_frame_textures(pipeline, queue);
        self.trace_prepare_textures(texture_stats);

        crate::logging::set_blocking_phase("prepare_projection");
        pipeline.update_projection(queue, &physical_bounds);
        let strata_elapsed =
            upload_dirty_strata_batches(pipeline, device, queue, scale, &self.strata_batches);
        let overlay_elapsed = upload_overlay_batch(pipeline, device, queue, scale, &self.overlay);
        let prepare_elapsed = prepare_started.elapsed();
        self.log_slow_prepare(
            prepare_elapsed,
            texture_stats,
            strata_elapsed,
            overlay_elapsed,
        );
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        pipeline.render(encoder, target, clip_bounds);
    }
}
