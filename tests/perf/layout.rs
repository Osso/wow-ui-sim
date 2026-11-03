use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::{Point, Rectangle, Size};
use rustc_hash::FxHashSet;
use wow_ui_sim::iced_app::compute_frame_rect;
use wow_ui_sim::iced_app::tooltip::{
    TooltipRender, TooltipRenderData, build_tooltip_quads, collect_tooltip_data,
    prepare_tooltip_layout,
};
use wow_ui_sim::iced_app::{
    DirtyStrataRebuildParams, RegistryQuadBatchParams, build_quad_batch_for_registry,
    rebuild_dirty_strata_batches_for_registry,
};
use wow_ui_sim::lua_api::{SimState, WowLuaEnv};
use wow_ui_sim::render::{FrameQuadSnapshot, GlyphAtlas, QuadBatch, WowFontSystem};
use wow_ui_sim::widget::{Anchor, FrameStrata};

const PERF_SCREEN_SIZE: (f32, f32) = (1024.0, 768.0);
const PERF_DIRTY_TEXTURE_NAME: &str = "PerfDirtyTexture";
const PERF_TOOLTIP_OWNER_NAME: &str = "PerfTooltipOwner";
const PERF_TOOLTIP_HEADER: &str = "Performance Tooltip";

type PerfStrataCache = [Option<Arc<QuadBatch>>; FrameStrata::COUNT];
type PerfSnapshotCache = [Option<HashMap<u64, FrameQuadSnapshot>>; FrameStrata::COUNT];

pub fn measure_full_root_layout_pass(env: &WowLuaEnv) -> Duration {
    let ui_parent_id = {
        let state = env.state().borrow();
        state
            .widgets
            .get_id_by_name("UIParent")
            .expect("UIParent should exist in the settled game UI")
    };

    let started = Instant::now();
    {
        let mut state = env.state().borrow_mut();
        state.widgets.mark_rect_dirty(ui_parent_id);
        state.invalidate_layout(ui_parent_id);
    }
    started.elapsed()
}

pub fn measure_incremental_anchor_change_layout_pass(env: &WowLuaEnv) -> Duration {
    let player_frame_id = find_player_frame_id(env);
    let original_anchors = snapshot_frame_anchors(env, player_frame_id);

    let started = Instant::now();
    {
        let mut state = env.state().borrow_mut();
        shift_first_anchor_and_relayout(&mut state, player_frame_id);
        restore_anchors_and_relayout(&mut state, player_frame_id, original_anchors);
    }
    started.elapsed()
}

pub fn measure_strata_bucket_rebuild(env: &WowLuaEnv) -> Duration {
    let started = Instant::now();
    {
        let mut state = env.state().borrow_mut();
        state.strata_buckets = None;
        let _ = state
            .get_strata_buckets()
            .expect("strata buckets should rebuild for the settled game UI");
    }
    started.elapsed()
}

pub fn measure_full_quad_batch_build(env: &WowLuaEnv) -> Duration {
    let mut font_system = WowFontSystem::new();
    let mut glyph_atlas = GlyphAtlas::new();

    let started = Instant::now();
    {
        let mut state = env.state().borrow_mut();
        let strata_buckets = state
            .get_strata_buckets()
            .expect("strata buckets should exist for quad batch measurement")
            .clone();
        let tooltip_data = collect_tooltip_data(&state);
        let text_ctx = Some((&mut font_system, &mut glyph_atlas));

        let _batch = build_quad_batch_for_registry(
            RegistryQuadBatchParams::new(&state.widgets, PERF_SCREEN_SIZE, &strata_buckets)
                .text_ctx(text_ctx)
                .message_frames(Some(&state.message_frames))
                .tooltip_data(Some(&tooltip_data)),
        );
    }
    started.elapsed()
}

pub fn measure_dirty_tree_quad_rebuild(env: &WowLuaEnv) -> Duration {
    ensure_perf_dirty_texture_exists(env);
    let mut font_system = WowFontSystem::new();
    let mut glyph_atlas = GlyphAtlas::new();
    let mut strata_cache = empty_strata_cache();
    let mut snapshot_cache = empty_snapshot_cache();
    let mut state = env.state().borrow_mut();
    let strata_buckets = prepare_dirty_tree_strata(&mut state);
    let mut text_ctx = Some((&mut font_system, &mut glyph_atlas));
    prime_perf_strata_cache(
        &mut strata_cache,
        &mut snapshot_cache,
        &mut text_ctx,
        &state,
        &strata_buckets,
    );

    measure_primed_dirty_texture_rebuild(
        &mut strata_cache,
        &mut snapshot_cache,
        &mut text_ctx,
        &mut state,
        &strata_buckets,
    )
}

fn measure_primed_dirty_texture_rebuild(
    strata_cache: &mut PerfStrataCache,
    snapshot_cache: &mut PerfSnapshotCache,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    state: &mut SimState,
    strata_buckets: &[Vec<u64>],
) -> Duration {
    let _ = state.widgets.take_render_dirty_with_ids();
    let dirty_change = perturb_perf_dirty_texture(state);
    let dirty_frames = take_incremental_dirty_frames(state, dirty_change.texture_id);
    let tooltip_data = collect_tooltip_data(state);
    let elapsed = measure_perf_strata_rebuild(
        strata_cache,
        snapshot_cache,
        text_ctx,
        state,
        strata_buckets,
        &tooltip_data,
        dirty_frames.as_refs(),
    );
    restore_perf_dirty_texture(state, dirty_change);
    elapsed
}

pub fn measure_tooltip_collect_and_quad_emission(env: &WowLuaEnv) -> Duration {
    seed_perf_tooltip(env);

    let mut font_system = WowFontSystem::new();
    update_seeded_perf_tooltip_sizes(env, &mut font_system);

    let mut glyph_atlas = GlyphAtlas::new();
    measure_seeded_perf_tooltip_quad_emission(env, &mut font_system, &mut glyph_atlas)
}

fn update_seeded_perf_tooltip_sizes(env: &WowLuaEnv, font_system: &mut WowFontSystem) {
    let mut state = env.state().borrow_mut();
    prepare_tooltip_layout(&mut state, font_system);
}

fn measure_seeded_perf_tooltip_quad_emission(
    env: &WowLuaEnv,
    font_system: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
) -> Duration {
    let started = Instant::now();
    {
        let state = env.state().borrow();
        emit_perf_tooltip_quads(&state, font_system, glyph_atlas);
    }
    started.elapsed()
}

fn prepare_dirty_tree_strata(state: &mut SimState) -> Vec<Vec<u64>> {
    state.ensure_layout_rects();
    state
        .get_strata_buckets()
        .expect("strata buckets should exist for dirty-tree quad measurement")
        .clone()
}

fn prime_perf_strata_cache(
    strata_cache: &mut PerfStrataCache,
    snapshot_cache: &mut PerfSnapshotCache,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    state: &SimState,
    strata_buckets: &[Vec<u64>],
) {
    let tooltip_data = collect_tooltip_data(state);
    rebuild_perf_strata(
        strata_cache,
        snapshot_cache,
        text_ctx,
        state,
        strata_buckets,
        &tooltip_data,
        DirtyFrames::all(),
    );
}

fn rebuild_perf_strata(
    strata_cache: &mut PerfStrataCache,
    snapshot_cache: &mut PerfSnapshotCache,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    state: &SimState,
    strata_buckets: &[Vec<u64>],
    tooltip_data: &HashMap<u64, TooltipRenderData>,
    dirty_frames: DirtyFrameRefs<'_>,
) {
    let elapsed_secs = state.start_time.elapsed().as_secs_f64();
    rebuild_dirty_strata_batches_for_registry(
        strata_cache,
        snapshot_cache,
        text_ctx,
        dirty_strata_params(
            state,
            strata_buckets,
            tooltip_data,
            dirty_frames.mask,
            dirty_frames.ids,
            elapsed_secs,
        ),
    );
}

fn measure_perf_strata_rebuild(
    strata_cache: &mut PerfStrataCache,
    snapshot_cache: &mut PerfSnapshotCache,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    state: &SimState,
    strata_buckets: &[Vec<u64>],
    tooltip_data: &HashMap<u64, TooltipRenderData>,
    dirty_frames: DirtyFrameRefs<'_>,
) -> Duration {
    let started = Instant::now();
    rebuild_perf_strata(
        strata_cache,
        snapshot_cache,
        text_ctx,
        state,
        strata_buckets,
        tooltip_data,
        dirty_frames,
    );
    started.elapsed()
}

fn dirty_strata_params<'a>(
    state: &'a SimState,
    strata_buckets: &'a [Vec<u64>],
    tooltip_data: &'a HashMap<u64, TooltipRenderData>,
    dirty: u16,
    dirty_ids: Option<&'a FxHashSet<u64>>,
    elapsed_secs: f64,
) -> DirtyStrataRebuildParams<'a> {
    DirtyStrataRebuildParams {
        dirty,
        dirty_ids,
        size: perf_screen_size(),
        strata_buckets,
        widgets: &state.widgets,
        pressed_frame: None,
        hovered_frame: None,
        message_frames: &state.message_frames,
        tooltip_data,
        quest_blobs: &state.quest_blobs,
        elapsed_secs,
    }
}

struct DirtyTextureChange {
    texture_id: u64,
    original_alpha: f32,
}

struct DirtyFrames {
    mask: u16,
    ids: Option<FxHashSet<u64>>,
}

impl DirtyFrames {
    fn all() -> DirtyFrameRefs<'static> {
        DirtyFrameRefs {
            mask: full_dirty_mask(),
            ids: None,
        }
    }

    fn as_refs(&self) -> DirtyFrameRefs<'_> {
        DirtyFrameRefs {
            mask: self.mask,
            ids: self.ids.as_ref(),
        }
    }
}

struct DirtyFrameRefs<'a> {
    mask: u16,
    ids: Option<&'a FxHashSet<u64>>,
}

fn take_incremental_dirty_frames(state: &mut SimState, expected_dirty_id: u64) -> DirtyFrames {
    let (mask, ids) = state.widgets.take_render_dirty_with_ids();
    assert_ne!(mask, 0, "single-frame visual change should dirty a strata");
    let ids = ids.expect("single-frame visual change should stay incremental");
    assert!(
        ids.contains(&expected_dirty_id),
        "dirty IDs should contain the changed texture frame"
    );
    DirtyFrames {
        mask,
        ids: Some(ids),
    }
}

fn perturb_perf_dirty_texture(state: &mut SimState) -> DirtyTextureChange {
    let texture_id = state
        .widgets
        .get_id_by_name(PERF_DIRTY_TEXTURE_NAME)
        .expect("perf dirty texture should resolve by name");
    let original_alpha = state
        .widgets
        .get(texture_id)
        .expect("perf dirty texture should exist")
        .alpha;
    state
        .widgets
        .get_mut_visual(texture_id)
        .expect("perf dirty texture should be mutable")
        .alpha = (original_alpha * 0.5).max(0.1);

    DirtyTextureChange {
        texture_id,
        original_alpha,
    }
}

fn restore_perf_dirty_texture(state: &mut SimState, change: DirtyTextureChange) {
    state
        .widgets
        .get_mut_visual(change.texture_id)
        .expect("perf dirty texture should still be mutable")
        .alpha = change.original_alpha;
    let _ = state.widgets.take_render_dirty_with_ids();
}

fn emit_perf_tooltip_quads(
    state: &SimState,
    font_system: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
) {
    let tooltip_data = collect_tooltip_data(state);
    let tooltip_id = state
        .widgets
        .get_id_by_name("GameTooltip")
        .expect("GameTooltip should exist in the settled game UI");
    let tooltip_frame = state
        .widgets
        .get(tooltip_id)
        .expect("GameTooltip frame should resolve");
    let bounds = perf_tooltip_bounds(state, tooltip_id);
    let mut batch = QuadBatch::new();
    let mut text_ctx = Some((font_system, glyph_atlas));

    build_tooltip_quads(
        TooltipRender {
            batch: &mut batch,
            bounds,
            tooltip_data: Some(&tooltip_data),
            id: tooltip_id,
            eff_alpha: tooltip_frame.alpha,
            eff_scale: tooltip_frame.effective_scale,
            draw_background: true,
        },
        &mut text_ctx,
    );

    assert!(
        !batch.vertices.is_empty(),
        "perf tooltip measurement should emit tooltip quads"
    );
}

fn perf_tooltip_bounds(state: &SimState, tooltip_id: u64) -> Rectangle {
    let bounds = compute_frame_rect(
        &state.widgets,
        tooltip_id,
        PERF_SCREEN_SIZE.0,
        PERF_SCREEN_SIZE.1,
    );
    Rectangle::new(
        Point::new(bounds.x, bounds.y),
        Size::new(bounds.width, bounds.height),
    )
}

fn find_player_frame_id(env: &WowLuaEnv) -> u64 {
    let state = env.state().borrow();
    state
        .widgets
        .get_id_by_name("PlayerFrame")
        .expect("PlayerFrame should exist in the settled game UI")
}

fn snapshot_frame_anchors(env: &WowLuaEnv, frame_id: u64) -> Vec<Anchor> {
    let state = env.state().borrow();
    let anchors = state
        .widgets
        .get(frame_id)
        .expect("frame should resolve from its widget id")
        .anchors
        .clone();
    assert!(
        !anchors.is_empty(),
        "frame should have at least one anchor for incremental layout measurement"
    );
    anchors
}

fn shift_first_anchor_and_relayout(state: &mut wow_ui_sim::lua_api::SimState, frame_id: u64) {
    state
        .widgets
        .get_mut(frame_id)
        .expect("frame should resolve mutably from its widget id")
        .anchors[0]
        .x_offset += 1.0;
    state.widgets.mark_rect_dirty(frame_id);
    state.invalidate_layout(frame_id);
}

fn ensure_perf_dirty_texture_exists(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        if not _G.PerfDirtyFrame then
            local frame = CreateFrame("Frame", "PerfDirtyFrame", UIParent)
            frame:SetSize(32, 32)
            frame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 12, -12)
            frame:Show()

            local texture = frame:CreateTexture("PerfDirtyTexture", "ARTWORK")
            texture:SetAllPoints()
            texture:SetColorTexture(1, 0.25, 0.25, 1)
            texture:Show()
        end
    "#,
    )
    .expect("perf dirty texture setup should succeed");
}

fn seed_perf_tooltip(env: &WowLuaEnv) {
    let script = perf_tooltip_seed_lua();
    env.exec(&script)
        .expect("perf tooltip setup should succeed");
}

fn perf_tooltip_seed_lua() -> String {
    format!("{}\n{}", perf_tooltip_owner_lua(), perf_tooltip_lines_lua())
}

fn perf_tooltip_owner_lua() -> String {
    format!(
        r#"
        if not _G.{owner_name} then
            local owner = CreateFrame("Frame", "{owner_name}", UIParent)
            owner:SetSize(16, 16)
            owner:SetPoint("CENTER")
            owner:Show()
        end
"#,
        owner_name = PERF_TOOLTIP_OWNER_NAME,
    )
}

fn perf_tooltip_lines_lua() -> String {
    format!(
        r#"
        GameTooltip:SetOwner(_G.{owner_name}, "ANCHOR_NONE")
        GameTooltip:ClearLines()
        GameTooltip:AddLine("{header}")
        {body_lines}
        GameTooltip:Show()
    "#,
        owner_name = PERF_TOOLTIP_OWNER_NAME,
        header = PERF_TOOLTIP_HEADER,
        body_lines = perf_tooltip_body_lines_lua(),
    )
}

fn perf_tooltip_body_lines_lua() -> &'static str {
    r#"GameTooltip:AddLine("This is a deliberately long tooltip body line used to exercise wrapped tooltip text measurement and quad emission in the render path.", 1, 1, 1, true)
        GameTooltip:AddDoubleLine("Spell", "Avenger's Shield")
        GameTooltip:AddLine("Second wrapped line to keep the body realistic and ensure multiple glyph rows are emitted into the tooltip batch.", 0.9, 0.82, 0.5, true)
        GameTooltip:AddDoubleLine("Cooldown", "15 sec")"#
}

fn full_dirty_mask() -> u16 {
    (1u16 << FrameStrata::COUNT) - 1
}

fn perf_screen_size() -> Size {
    Size::new(PERF_SCREEN_SIZE.0, PERF_SCREEN_SIZE.1)
}

fn empty_strata_cache() -> [Option<Arc<QuadBatch>>; FrameStrata::COUNT] {
    std::array::from_fn(|_| None)
}

fn empty_snapshot_cache()
-> [Option<std::collections::HashMap<u64, FrameQuadSnapshot>>; FrameStrata::COUNT] {
    std::array::from_fn(|_| None)
}

fn restore_anchors_and_relayout(
    state: &mut wow_ui_sim::lua_api::SimState,
    frame_id: u64,
    original_anchors: Vec<Anchor>,
) {
    state
        .widgets
        .get_mut(frame_id)
        .expect("frame should still exist after incremental layout")
        .anchors = original_anchors;
    state.widgets.mark_rect_dirty(frame_id);
    state.invalidate_layout(frame_id);
}
