use iced::{Point, Rectangle, Size};

use crate::iced_app::layout::anchor_position;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::render::shader::primitive::{
    TextureLoadTelemetry, TextureRequestTracker, load_texture_or_crop,
    load_texture_prefer_bc_with_telemetry,
};
use crate::render::texture::UI_SCALE;
use crate::render::{GpuBcTextureData, GpuTextureData, QuadBatch};
use crate::widget::{Frame, FrameStrata, WidgetRegistry, WidgetType};

use super::super::app::App;
use super::super::quad_builders::{build_texture_quads, emit_button_highlight};

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TextureLoadBatchTelemetry {
    rgba_total_elapsed: std::time::Duration,
    rgba_mem_cache_hits: usize,
    rgba_resolve_elapsed: std::time::Duration,
    rgba_decode_elapsed: std::time::Duration,
    bc_total_elapsed: std::time::Duration,
    bc_resolve_elapsed: std::time::Duration,
    bc_parse_elapsed: std::time::Duration,
    bc_extract_elapsed: std::time::Duration,
    bc_cache_hits: usize,
    crop_decode_elapsed: std::time::Duration,
    crop_extract_elapsed: std::time::Duration,
}

type BudgetedTextureLoadResult = (
    Vec<GpuTextureData>,
    Vec<GpuBcTextureData>,
    std::time::Duration,
    std::time::Duration,
    TextureLoadBatchTelemetry,
    bool,
);

#[derive(Default)]
struct TextureLoadAccumulator {
    textures: Vec<GpuTextureData>,
    bc_textures: Vec<GpuBcTextureData>,
    scan_elapsed: std::time::Duration,
    load_elapsed: std::time::Duration,
    telemetry: TextureLoadBatchTelemetry,
    exhausted: bool,
}

struct PendingTextureQueueStep {
    scan_elapsed: std::time::Duration,
    load_elapsed: std::time::Duration,
    budget_hit: bool,
    exhausted: bool,
}

impl TextureLoadAccumulator {
    fn record_budgeted(&mut self, batch: BudgetedTextureLoadResult) {
        let (textures, bc_textures, scan_elapsed, load_elapsed, telemetry, exhausted) = batch;
        self.textures.extend(textures);
        self.bc_textures.extend(bc_textures);
        self.scan_elapsed += scan_elapsed;
        self.load_elapsed += load_elapsed;
        self.telemetry.record_batch(telemetry);
        self.exhausted |= exhausted;
    }
}

impl TextureLoadBatchTelemetry {
    fn record(&mut self, telemetry: TextureLoadTelemetry) {
        self.rgba_total_elapsed += telemetry.rgba.total_elapsed;
        self.rgba_mem_cache_hits += usize::from(telemetry.rgba.mem_cache_hit);
        self.rgba_resolve_elapsed += telemetry.rgba.resolve_elapsed;
        self.rgba_decode_elapsed += telemetry.rgba.decode_elapsed;
        self.bc_total_elapsed += telemetry.bc.total_elapsed;
        self.bc_resolve_elapsed += telemetry.bc.resolve_elapsed;
        self.bc_parse_elapsed += telemetry.bc.parse_elapsed;
        self.bc_extract_elapsed += telemetry.bc.extract_elapsed;
        self.bc_cache_hits += usize::from(telemetry.bc.cache_hit);
        self.crop_decode_elapsed += telemetry.crop_decode_elapsed;
        self.crop_extract_elapsed += telemetry.crop_extract_elapsed;
    }

    fn record_batch(&mut self, telemetry: Self) {
        self.rgba_total_elapsed += telemetry.rgba_total_elapsed;
        self.rgba_mem_cache_hits += telemetry.rgba_mem_cache_hits;
        self.rgba_resolve_elapsed += telemetry.rgba_resolve_elapsed;
        self.rgba_decode_elapsed += telemetry.rgba_decode_elapsed;
        self.bc_total_elapsed += telemetry.bc_total_elapsed;
        self.bc_resolve_elapsed += telemetry.bc_resolve_elapsed;
        self.bc_parse_elapsed += telemetry.bc_parse_elapsed;
        self.bc_extract_elapsed += telemetry.bc_extract_elapsed;
        self.bc_cache_hits += telemetry.bc_cache_hits;
        self.crop_decode_elapsed += telemetry.crop_decode_elapsed;
        self.crop_extract_elapsed += telemetry.crop_extract_elapsed;
    }
}

impl App {
    pub(super) fn build_overlay(&self) -> QuadBatch {
        let mut overlay = QuadBatch::new();
        self.append_debug_overlay(&mut overlay);
        self.append_hover_highlight(&mut overlay);
        if let Some(pos) = self.mouse_position {
            self.append_cursor_item_icon(&mut overlay, pos);
            const CURSOR_SIZE: f32 = 32.0;
            overlay.push_textured_path(
                Rectangle::new(
                    Point::new(pos.x, pos.y),
                    Size::new(CURSOR_SIZE, CURSOR_SIZE),
                ),
                r"Interface\Cursor\Point",
                [1.0, 1.0, 1.0, 1.0],
                crate::render::BlendMode::Alpha,
            );
        }
        overlay
    }

    fn append_debug_overlay(&self, overlay: &mut QuadBatch) {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let (show_borders, show_anchors) = self.debug_overlay_flags(&state);
        if !show_borders && !show_anchors {
            return;
        }

        for id in collect_debug_overlay_ids(&state) {
            self.append_frame_debug_overlay(overlay, &state, id, show_borders, show_anchors);
        }
    }

    fn debug_overlay_flags(&self, state: &crate::lua_api::SimState) -> (bool, bool) {
        (
            self.debug_borders || state.debug_borders,
            self.debug_anchors || state.debug_anchors,
        )
    }

    fn append_frame_debug_overlay(
        &self,
        overlay: &mut QuadBatch,
        state: &crate::lua_api::SimState,
        id: u64,
        show_borders: bool,
        show_anchors: bool,
    ) {
        let Some((frame, rect, bounds)) = debug_overlay_frame(state, id) else {
            return;
        };
        if show_borders {
            overlay.push_border(bounds, 1.0, [1.0, 0.1, 0.1, 0.9]);
        }
        if show_anchors {
            append_anchor_markers(overlay, frame, rect);
        }
    }

    /// Load textures with a small draw-thread budget (~10ms).
    /// Remaining work stays pending for the tick preloader.
    pub(super) fn load_all_textures(
        &self,
        dirty_strata: &[Option<Arc<QuadBatch>>; FrameStrata::COUNT],
        overlay: &QuadBatch,
    ) -> (
        Vec<GpuTextureData>,
        Vec<GpuBcTextureData>,
        std::time::Duration,
        Arc<Mutex<TextureRequestTracker>>,
    ) {
        let t = std::time::Instant::now();
        let deadline = t + std::time::Duration::from_millis(10);
        let mut texture_requests = texture_requests_for_dirty_strata(dirty_strata);
        let mut loaded = TextureLoadAccumulator::default();

        loaded.record_budgeted(
            self.load_pending_texture_queue_budgeted(deadline, &mut texture_requests),
        );

        if !loaded.exhausted {
            loaded.record_budgeted(self.load_new_textures_budgeted(
                overlay,
                deadline,
                &mut texture_requests,
            ));
        }

        self.textures_pending
            .set(loaded.exhausted || self.cached_render_requests_still_pending());
        let elapsed = t.elapsed();
        if should_log_slow_texture_load(&loaded, elapsed) {
            log_slow_texture_load(
                &loaded.textures,
                &loaded.bc_textures,
                elapsed,
                loaded.scan_elapsed,
                loaded.load_elapsed,
                loaded.telemetry,
            );
        }
        (
            loaded.textures,
            loaded.bc_textures,
            elapsed,
            Arc::new(Mutex::new(texture_requests)),
        )
    }

    /// Load new textures from the batch's requests within a time budget.
    /// Returns (RGBA textures, BC textures, scan_elapsed, load_elapsed, telemetry, deadline_reached).
    pub(super) fn load_new_textures_budgeted(
        &self,
        quads: &QuadBatch,
        deadline: std::time::Instant,
        texture_requests: &mut TextureRequestTracker,
    ) -> BudgetedTextureLoadResult {
        let mut tex_mgr = self.texture_manager.borrow_mut();
        let scan_start = std::time::Instant::now();
        texture_requests.register_batch(quads);
        let pending_paths = unresolved_texture_request_paths(quads);
        let scan_elapsed = scan_start.elapsed();
        let load_start = std::time::Instant::now();
        let (textures, bc_textures, telemetry, budget_hit) =
            load_texture_paths_budgeted(pending_paths, deadline, &mut tex_mgr, texture_requests);
        (
            textures,
            bc_textures,
            scan_elapsed,
            load_start.elapsed(),
            telemetry,
            budget_hit,
        )
    }

    fn load_pending_texture_queue_budgeted(
        &self,
        deadline: std::time::Instant,
        texture_requests: &mut TextureRequestTracker,
    ) -> BudgetedTextureLoadResult {
        let mut textures = Vec::new();
        let mut bc_textures = Vec::new();
        let mut telemetry = TextureLoadBatchTelemetry::default();
        let mut tex_mgr = self.texture_manager.borrow_mut();
        let mut scan_elapsed = std::time::Duration::ZERO;
        let mut load_elapsed = std::time::Duration::ZERO;
        let mut budget_hit = false;
        let mut iterations_remaining = self.pending_texture_path_queue.borrow().len();

        while iterations_remaining != 0 {
            iterations_remaining -= 1;
            let step = self.load_pending_texture_queue_step(
                deadline,
                &mut tex_mgr,
                texture_requests,
                &mut textures,
                &mut bc_textures,
                &mut telemetry,
            );
            scan_elapsed += step.scan_elapsed;
            load_elapsed += step.load_elapsed;
            if step.exhausted || step.budget_hit {
                budget_hit = step.budget_hit;
                break;
            }
        }

        (
            textures,
            bc_textures,
            scan_elapsed,
            load_elapsed,
            telemetry,
            budget_hit,
        )
    }

    fn load_pending_texture_queue_step(
        &self,
        deadline: std::time::Instant,
        tex_mgr: &mut crate::texture::TextureManager,
        texture_requests: &mut TextureRequestTracker,
        textures: &mut Vec<GpuTextureData>,
        bc_textures: &mut Vec<GpuBcTextureData>,
        telemetry: &mut TextureLoadBatchTelemetry,
    ) -> PendingTextureQueueStep {
        let scan_started = std::time::Instant::now();
        let Some(path) = self.pop_next_pending_texture_path() else {
            return pending_texture_queue_step(scan_started.elapsed(), Duration::ZERO, false, true);
        };

        let (is_pending, should_load) = self.pending_path_state(&path);
        let scan_elapsed = scan_started.elapsed();
        if !is_pending {
            self.remove_pending_texture_path(&path);
            return pending_texture_queue_step(scan_elapsed, Duration::ZERO, false, false);
        }
        if !should_load {
            self.requeue_pending_texture_path(path);
            return pending_texture_queue_step(scan_elapsed, Duration::ZERO, false, false);
        }

        self.register_pending_texture_requests_for_path(&path, texture_requests);
        self.load_ready_pending_texture_path(
            deadline,
            path,
            tex_mgr,
            texture_requests,
            textures,
            bc_textures,
            telemetry,
            scan_elapsed,
        )
    }

    fn load_ready_pending_texture_path(
        &self,
        deadline: std::time::Instant,
        path: String,
        tex_mgr: &mut crate::texture::TextureManager,
        texture_requests: &mut TextureRequestTracker,
        textures: &mut Vec<GpuTextureData>,
        bc_textures: &mut Vec<GpuBcTextureData>,
        telemetry: &mut TextureLoadBatchTelemetry,
        scan_elapsed: std::time::Duration,
    ) -> PendingTextureQueueStep {
        let load_started = std::time::Instant::now();
        let budget_hit = process_budgeted_texture_request(
            deadline,
            &path,
            tex_mgr,
            textures,
            bc_textures,
            telemetry,
            texture_requests,
        );
        let load_elapsed = load_started.elapsed();
        if budget_hit {
            self.requeue_pending_texture_path(path);
            return pending_texture_queue_step(scan_elapsed, load_elapsed, true, false);
        }

        let (still_pending, _) = self.pending_path_state(&path);
        if still_pending {
            self.requeue_pending_texture_path(path);
        } else {
            self.remove_pending_texture_path(&path);
        }
        pending_texture_queue_step(scan_elapsed, load_elapsed, false, false)
    }

    /// Append hover highlight quads for the currently hovered button.
    fn append_hover_highlight(&self, quads: &mut QuadBatch) {
        let Some(hovered_id) = self.hovered_frame else {
            return;
        };
        let env = self.env.borrow();
        let state = env.state().borrow();
        let registry = &state.widgets;
        let Some(f) = registry.get(hovered_id) else {
            return;
        };
        if !super::super::button_vis::should_append_hover_highlight(registry, f) {
            return;
        }

        let Some(bounds) = hovered_button_bounds(f) else {
            return;
        };

        let has_highlight_child = f.children_keys.contains_key("HighlightTexture");
        let is_pressed = self.pressed_frame == Some(hovered_id) || f.button_state == 1;
        if !is_pressed && !has_highlight_child {
            emit_button_highlight(quads, bounds, f, f.alpha);
        }

        append_hover_child_highlight(quads, registry, f, is_pressed);
    }

    /// Render the spell icon attached to the cursor when dragging.
    fn append_cursor_item_icon(&self, overlay: &mut QuadBatch, pos: Point) {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let spell_id = match &state.cursor_item {
            Some(crate::lua_api::state::CursorInfo::Action { spell_id, .. }) => *spell_id,
            Some(crate::lua_api::state::CursorInfo::Spell { spell_id }) => *spell_id,
            Some(crate::lua_api::state::CursorInfo::PetAction { spell_id, .. }) => *spell_id,
            Some(crate::lua_api::state::CursorInfo::Item { .. }) => return,
            Some(crate::lua_api::state::CursorInfo::Talent { .. }) => return,
            Some(crate::lua_api::state::CursorInfo::Macro { .. }) => return,
            Some(crate::lua_api::state::CursorInfo::Money { .. }) => return,
            None => return,
        };
        let Some(spell) = crate::spells::get_spell(spell_id) else {
            return;
        };
        let Some(path) = crate::manifest_interface_data::get_texture_path(spell.icon_file_data_id)
        else {
            return;
        };
        let tex_path = format!("Interface\\{}", path.replace('/', "\\"));

        const ICON_SIZE: f32 = 32.0;
        let icon_bounds = Rectangle::new(
            Point::new(pos.x - ICON_SIZE * 0.5, pos.y - ICON_SIZE * 0.5),
            Size::new(ICON_SIZE, ICON_SIZE),
        );
        overlay.push_textured_path(
            icon_bounds,
            &tex_path,
            [1.0, 1.0, 1.0, 1.0],
            crate::render::BlendMode::Alpha,
        );
    }
}

fn collect_debug_overlay_ids(state: &crate::lua_api::SimState) -> Vec<u64> {
    if let Some(buckets) = state.strata_buckets.as_ref() {
        return buckets
            .iter()
            .flat_map(|bucket| bucket.iter().copied())
            .collect();
    }

    state
        .widgets
        .iter_ids()
        .filter(|&id| state.widgets.is_ancestor_visible(id))
        .collect()
}

fn debug_overlay_frame(
    state: &crate::lua_api::SimState,
    id: u64,
) -> Option<(&Frame, crate::LayoutRect, Rectangle)> {
    if !state.widgets.is_ancestor_visible(id) {
        return None;
    }
    let frame = state.widgets.get(id)?;
    let rect = frame.layout_rect?;
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }
    let bounds = Rectangle::new(
        Point::new(rect.x * UI_SCALE, rect.y * UI_SCALE),
        Size::new(rect.width * UI_SCALE, rect.height * UI_SCALE),
    );
    Some((frame, rect, bounds))
}

fn hovered_button_bounds(frame: &Frame) -> Option<Rectangle> {
    if !matches!(
        frame.widget_type,
        WidgetType::Button | WidgetType::CheckButton
    ) {
        return None;
    }

    frame.layout_rect.and_then(layout_rect_bounds)
}

fn layout_rect_bounds(rect: crate::LayoutRect) -> Option<Rectangle> {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }

    Some(Rectangle::new(
        Point::new(rect.x * UI_SCALE, rect.y * UI_SCALE),
        Size::new(rect.width * UI_SCALE, rect.height * UI_SCALE),
    ))
}

fn append_hover_child_highlight(
    quads: &mut QuadBatch,
    registry: &WidgetRegistry,
    frame: &Frame,
    is_pressed: bool,
) {
    if is_pressed {
        return;
    }

    let Some(&highlight_id) = frame.children_keys.get("HighlightTexture") else {
        return;
    };
    let Some(highlight) = registry.get(highlight_id) else {
        return;
    };
    let Some(bounds) = highlight.layout_rect.and_then(layout_rect_bounds) else {
        return;
    };

    build_texture_quads(quads, bounds, highlight, None, highlight.alpha);
}

fn append_anchor_markers(overlay: &mut QuadBatch, frame: &Frame, rect: crate::LayoutRect) {
    const ANCHOR_MARKER_SIZE: f32 = 5.0;
    const ANCHOR_MARKER_OFFSET: f32 = ANCHOR_MARKER_SIZE * 0.5;
    const ANCHOR_MARKER_COLOR: [f32; 4] = [0.1, 1.0, 0.1, 1.0];

    for anchor in &frame.anchors {
        let (x, y) = anchor_position(
            anchor.point,
            rect.x * UI_SCALE,
            rect.y * UI_SCALE,
            rect.width * UI_SCALE,
            rect.height * UI_SCALE,
        );
        overlay.push_solid(
            Rectangle::new(
                Point::new(x - ANCHOR_MARKER_OFFSET, y - ANCHOR_MARKER_OFFSET),
                Size::new(ANCHOR_MARKER_SIZE, ANCHOR_MARKER_SIZE),
            ),
            ANCHOR_MARKER_COLOR,
        );
    }
}

fn texture_requests_for_dirty_strata(
    dirty_strata: &[Option<Arc<QuadBatch>>; FrameStrata::COUNT],
) -> TextureRequestTracker {
    let mut texture_requests = TextureRequestTracker::default();
    for batch in dirty_strata.iter().flatten() {
        texture_requests.register_batch(batch);
    }
    texture_requests
}

fn should_log_slow_texture_load(
    loaded: &TextureLoadAccumulator,
    elapsed: std::time::Duration,
) -> bool {
    elapsed.as_millis() > 10 && (!loaded.textures.is_empty() || !loaded.bc_textures.is_empty())
}

fn log_slow_texture_load(
    textures: &[GpuTextureData],
    bc_textures: &[GpuBcTextureData],
    elapsed: std::time::Duration,
    scan_elapsed: std::time::Duration,
    load_elapsed: std::time::Duration,
    telemetry: TextureLoadBatchTelemetry,
) {
    if !crate::logging::texture_load_debug_enabled() {
        return;
    }
    let mut preview: Vec<&str> = textures.iter().map(|tex| tex.path.as_str()).collect();
    preview.extend(bc_textures.iter().map(|tex| tex.path.as_str()));
    preview.truncate(12);
    eprintln!(
        "{} [textures] loaded {} in {elapsed:.1?} (scan={scan_elapsed:.1?} load={load_elapsed:.1?} rgba={:.1?} rgba_mem_hits={} rgba_resolve={:.1?} rgba_decode={:.1?} bc={:.1?} bc_resolve={:.1?} bc_parse={:.1?} bc_extract={:.1?} bc_cache_hits={} crop_decode={:.1?} crop_extract={:.1?}): {} (rgba={} bc={})",
        crate::logging::global_elapsed_prefix(),
        textures.len() + bc_textures.len(),
        telemetry.rgba_total_elapsed,
        telemetry.rgba_mem_cache_hits,
        telemetry.rgba_resolve_elapsed,
        telemetry.rgba_decode_elapsed,
        telemetry.bc_total_elapsed,
        telemetry.bc_resolve_elapsed,
        telemetry.bc_parse_elapsed,
        telemetry.bc_extract_elapsed,
        telemetry.bc_cache_hits,
        telemetry.crop_decode_elapsed,
        telemetry.crop_extract_elapsed,
        preview.join(", "),
        textures.len(),
        bc_textures.len(),
    );
}

fn should_pause_texture_loading(
    textures: &[GpuTextureData],
    bc_textures: &[GpuBcTextureData],
    deadline: std::time::Instant,
    path: &str,
    tex_mgr: &crate::texture::TextureManager,
) -> bool {
    let loaded_any_textures = !textures.is_empty() || !bc_textures.is_empty();
    let deadline_reached = std::time::Instant::now() >= deadline;
    let base_path_cached = tex_mgr.is_cached(texture_request_base_path(path));
    should_pause_texture_loading_state(loaded_any_textures, deadline_reached, base_path_cached)
}

fn should_pause_texture_loading_state(
    loaded_any_textures: bool,
    deadline_reached: bool,
    base_path_cached: bool,
) -> bool {
    loaded_any_textures && deadline_reached && !base_path_cached
}

fn texture_request_base_path(path: &str) -> &str {
    path.find("@crop:").map_or(path, |index| &path[..index])
}

fn pending_texture_queue_step(
    scan_elapsed: std::time::Duration,
    load_elapsed: std::time::Duration,
    budget_hit: bool,
    exhausted: bool,
) -> PendingTextureQueueStep {
    PendingTextureQueueStep {
        scan_elapsed,
        load_elapsed,
        budget_hit,
        exhausted,
    }
}

fn load_texture_paths_budgeted(
    pending_paths: Vec<&str>,
    deadline: std::time::Instant,
    tex_mgr: &mut crate::texture::TextureManager,
    texture_requests: &mut TextureRequestTracker,
) -> (
    Vec<GpuTextureData>,
    Vec<GpuBcTextureData>,
    TextureLoadBatchTelemetry,
    bool,
) {
    let mut textures = Vec::new();
    let mut bc_textures = Vec::new();
    let mut telemetry = TextureLoadBatchTelemetry::default();

    for path in pending_paths {
        let budget_hit = process_budgeted_texture_request(
            deadline,
            path,
            tex_mgr,
            &mut textures,
            &mut bc_textures,
            &mut telemetry,
            texture_requests,
        );
        if budget_hit {
            return (textures, bc_textures, telemetry, true);
        }
    }

    (textures, bc_textures, telemetry, false)
}

fn process_budgeted_texture_request(
    deadline: std::time::Instant,
    path: &str,
    tex_mgr: &mut crate::texture::TextureManager,
    textures: &mut Vec<GpuTextureData>,
    bc_textures: &mut Vec<GpuBcTextureData>,
    telemetry: &mut TextureLoadBatchTelemetry,
    texture_requests: &mut TextureRequestTracker,
) -> bool {
    if should_pause_texture_loading(textures, bc_textures, deadline, path, tex_mgr) {
        return true;
    }
    load_pending_texture(
        tex_mgr,
        path,
        textures,
        bc_textures,
        telemetry,
        texture_requests,
    );
    false
}

fn load_pending_texture(
    tex_mgr: &mut crate::texture::TextureManager,
    path: &str,
    textures: &mut Vec<GpuTextureData>,
    bc_textures: &mut Vec<GpuBcTextureData>,
    telemetry: &mut TextureLoadBatchTelemetry,
    texture_requests: &mut TextureRequestTracker,
) {
    use crate::render::shader::primitive::LoadedTexture;

    if texture_requests.needs_force_rgba_retry(path) {
        if let Some(data) = load_texture_or_crop(tex_mgr, path) {
            texture_requests.mark_staged(path);
            textures.push(data);
            return;
        }
    } else {
        let (loaded, load_telemetry) = load_texture_prefer_bc_with_telemetry(tex_mgr, path);
        telemetry.record(load_telemetry);
        if let Some(loaded) = loaded {
            texture_requests.mark_staged(path);
            match loaded {
                LoadedTexture::Rgba(data) => textures.push(data),
                LoadedTexture::Bc(data) => bc_textures.push(data),
            }
            return;
        }
    }

    texture_requests.mark_failed(path);
}

fn unresolved_texture_request_paths(quads: &QuadBatch) -> Vec<&str> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();

    for request in quads
        .texture_requests
        .iter()
        .chain(&quads.mask_texture_requests)
    {
        let path = request.path.as_str();
        let is_duplicate = !seen.insert(path);
        if is_duplicate || !request.handle.should_load() {
            continue;
        }
        paths.push(path);
    }

    sort_texture_request_paths(&mut paths);
    paths
}

fn sort_texture_request_paths<T: AsRef<str>>(paths: &mut [T]) {
    paths.sort_by(|a, b| {
        texture_request_priority(a.as_ref())
            .cmp(&texture_request_priority(b.as_ref()))
            .then_with(|| a.as_ref().cmp(b.as_ref()))
    });
}

fn texture_request_priority(path: &str) -> (u8, u8) {
    let is_world_map = path
        .get(..19)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Interface\\WorldMap\\"))
        || path
            .get(..19)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Interface/WorldMap/"));
    let is_crop = path.contains("@crop:");
    (u8::from(!is_world_map), u8::from(is_crop))
}

#[cfg(test)]
#[path = "render_textures_tests.rs"]
mod tests;
