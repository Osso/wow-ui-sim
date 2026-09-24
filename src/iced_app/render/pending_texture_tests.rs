use super::*;
use crate::render::BlendMode;
use crate::render::shader::primitive::TextureRequestTracker;
use crate::render::{QuadBatch, TextureRequest};
use iced::{Point, Rectangle, Size};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

fn build_test_app() -> App {
    super::test_support::build_test_app()
}

#[test]
fn budgeted_preload_loads_explicitly_queued_texture_requests() {
    let temp_dir = tempdir().unwrap();
    let addons_dir = temp_dir.path();
    let texture_path = addons_dir.join("world-map-tile.png");
    std::fs::create_dir_all(texture_path.parent().unwrap()).unwrap();
    let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([0x44, 0x88, 0xcc, 0xff]));
    image.save(&texture_path).unwrap();

    let app = super::test_support::build_test_app_with_addons(Some(addons_dir));
    app.env
        .borrow()
        .state()
        .borrow_mut()
        .enqueue_texture_preloads(["Interface/AddOns/world-map-tile".to_string()]);

    app.preload_current_render_requests(Some(std::time::Duration::from_millis(50)));

    assert!(
        app.texture_manager
            .borrow()
            .get("Interface/AddOns/world-map-tile")
            .is_some(),
        "queued preload should decode the requested texture source"
    );
    assert!(
        !app.textures_pending.get(),
        "queue-driven preload should clear pending state once the queue drains"
    );
}

#[test]
fn budgeted_preload_requeues_tail_when_budget_hits() {
    let temp_dir = tempdir().unwrap();
    for name in ["alpha", "beta"] {
        let texture_path = temp_dir.path().join(format!("{name}.png"));
        let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([0x44, 0x88, 0xcc, 0xff]));
        image.save(&texture_path).unwrap();
    }

    let app = build_test_app();
    app.env
        .borrow()
        .state()
        .borrow_mut()
        .enqueue_texture_preloads(["alpha".to_string(), "beta".to_string()]);

    app.preload_current_render_requests(Some(std::time::Duration::ZERO));

    let queued_after = app
        .env
        .borrow()
        .state()
        .borrow()
        .pending_texture_preloads
        .len();

    assert!(
        app.textures_pending.get(),
        "budget hit should keep queue-driven preload pending"
    );
    assert_ne!(queued_after, 0, "unprocessed paths should be requeued");
}

#[test]
fn empty_queue_preload_clears_stale_pending_state_without_draw_owned_requests() {
    let app = build_test_app();
    app.textures_pending.set(true);

    app.preload_current_render_requests(Some(std::time::Duration::from_millis(50)));

    assert!(
        !app.textures_pending.get(),
        "an empty preload queue should clear stale pending state when draw requests are already resolved"
    );
}

#[test]
fn empty_queue_preload_keeps_draw_owned_pending_state() {
    let app = build_test_app();
    app.textures_pending.set(true);

    let request_path = "render-owned-pending";
    let mut batch = QuadBatch::new();
    batch
        .texture_requests
        .push(TextureRequest::new(request_path, 0, 4));
    batch.texture_requests[0].handle.mark_staged();
    app.cached_strata_quads.borrow_mut()[0] = Some(std::sync::Arc::new(batch));

    app.preload_current_render_requests(Some(std::time::Duration::from_millis(50)));

    assert!(
        app.textures_pending.get(),
        "an empty preload queue must preserve pending state while draw-owned requests are unresolved"
    );
}

#[test]
fn preload_current_render_requests_keeps_pending_until_draw_uploads_cached_requests() {
    let temp_dir = tempdir().unwrap();
    let addons_dir = temp_dir.path();
    let texture_path = addons_dir.join("render-owned-pending.png");
    std::fs::create_dir_all(texture_path.parent().unwrap()).unwrap();
    let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([0x44, 0x88, 0xcc, 0xff]));
    image.save(&texture_path).unwrap();

    let app = super::test_support::build_test_app_with_addons(Some(addons_dir));
    let request_path = "Interface/AddOns/render-owned-pending".to_string();
    app.env
        .borrow()
        .state()
        .borrow_mut()
        .enqueue_texture_preloads([request_path.clone()]);

    let mut batch = QuadBatch::new();
    batch
        .texture_requests
        .push(TextureRequest::new(&request_path, 0, 4));
    batch.texture_requests[0].handle.mark_staged();
    app.cached_strata_quads.borrow_mut()[0] = Some(std::sync::Arc::new(batch));
    app.seed_pending_texture_paths_from_cached_strata();

    app.preload_current_render_requests(Some(std::time::Duration::from_millis(50)));

    assert!(
        app.texture_manager.borrow().get(&request_path).is_some(),
        "queue-driven preload should decode the cached render request source"
    );
    assert!(
        app.textures_pending.get(),
        "queue drain must not clear pending state until the render request is GPU-uploaded"
    );
}

#[test]
fn pending_transition_reinjects_clean_cached_strata_for_staged_requests() {
    let app = build_test_app();
    let request_path = "retained-reinject";

    let mut batch = QuadBatch::new();
    batch.push_textured_path(
        Rectangle::new(Point::ORIGIN, Size::new(8.0, 8.0)),
        request_path,
        [1.0, 1.0, 1.0, 1.0],
        BlendMode::Alpha,
    );
    let cached = std::sync::Arc::new(batch);
    app.cached_strata_quads.borrow_mut()[0] = Some(std::sync::Arc::clone(&cached));
    // Request-local state is now carried on the request itself.
    app.textures_pending.set(true);
    app.seed_pending_texture_paths_from_cached_strata();

    let mut dirty_strata = std::array::from_fn(|_| None);
    let texture_requests = Arc::new(Mutex::new(TextureRequestTracker::default()));

    app.recover_pending_textures(&mut dirty_strata, &texture_requests);

    assert!(
        dirty_strata[0]
            .as_ref()
            .is_some_and(|batch| std::sync::Arc::ptr_eq(batch, &cached)),
        "retained draw should resubmit the cached clean strata batch while textures are pending"
    );
    assert!(
        dirty_strata[1..].iter().all(Option::is_none),
        "only the cached strata with pending requests should be reinjected here"
    );
}

#[test]
fn pending_path_state_tracks_rebuilt_strata_deltas() {
    let app = build_test_app();

    let mut strata0 = QuadBatch::new();
    strata0
        .texture_requests
        .push(TextureRequest::new("delta-strata-0-a", 0, 4));
    app.cached_strata_quads.borrow_mut()[0] = Some(Arc::new(strata0));

    let mut strata1 = QuadBatch::new();
    strata1
        .texture_requests
        .push(TextureRequest::new("delta-strata-1", 0, 4));
    app.cached_strata_quads.borrow_mut()[1] = Some(Arc::new(strata1));

    app.refresh_pending_texture_requests_for_rebuilt_strata((1 << 0) | (1 << 1));
    {
        let pending = app.pending_texture_path_set.borrow();
        assert!(pending.contains("delta-strata-0-a"));
        assert!(pending.contains("delta-strata-1"));
    }

    let mut strata0_rebuilt = QuadBatch::new();
    strata0_rebuilt
        .texture_requests
        .push(TextureRequest::new("delta-strata-0-b", 0, 4));
    app.cached_strata_quads.borrow_mut()[0] = Some(Arc::new(strata0_rebuilt));
    app.refresh_pending_texture_requests_for_rebuilt_strata(1 << 0);

    let pending = app.pending_texture_path_set.borrow();
    assert!(
        !pending.contains("delta-strata-0-a"),
        "rebuilding one strata should evict stale paths from that strata only"
    );
    assert!(pending.contains("delta-strata-0-b"));
    assert!(
        pending.contains("delta-strata-1"),
        "unrebuilt strata paths should remain pending"
    );
}

#[test]
fn pending_path_queue_drains_when_request_is_marked_ready() {
    let app = build_test_app();

    let mut batch = QuadBatch::new();
    batch
        .texture_requests
        .push(TextureRequest::new("drain-on-ready", 0, 4));
    let handle = batch.texture_requests[0].handle.clone();
    app.cached_strata_quads.borrow_mut()[0] = Some(Arc::new(batch));

    app.refresh_pending_texture_requests_for_rebuilt_strata(1 << 0);
    assert!(
        app.cached_render_requests_still_pending(),
        "fresh pending request should be tracked"
    );

    handle.mark_ready();
    assert!(
        !app.cached_render_requests_still_pending(),
        "ready requests should be pruned from the persistent pending-path queue"
    );
    assert!(
        app.pending_texture_path_set.borrow().is_empty(),
        "drained queue should not keep stale pending paths"
    );
}

#[test]
fn rebuilt_requests_reuse_ready_path_cache_without_redecode() {
    let app = build_test_app();

    let mut initial_batch = QuadBatch::new();
    initial_batch
        .texture_requests
        .push(TextureRequest::new("already-ready-path", 0, 4));
    let initial_handle = initial_batch.texture_requests[0].handle.clone();
    app.cached_strata_quads.borrow_mut()[0] = Some(Arc::new(initial_batch));
    app.refresh_pending_texture_requests_for_rebuilt_strata(1 << 0);

    assert!(app.cached_render_requests_still_pending());
    initial_handle.mark_ready();
    assert!(
        !app.cached_render_requests_still_pending(),
        "ready path should drain from pending queue and be cached as ready"
    );

    let mut rebuilt_batch = QuadBatch::new();
    rebuilt_batch
        .texture_requests
        .push(TextureRequest::new("already-ready-path", 0, 4));
    let rebuilt_handle = rebuilt_batch.texture_requests[0].handle.clone();
    app.cached_strata_quads.borrow_mut()[0] = Some(Arc::new(rebuilt_batch));
    app.refresh_pending_texture_requests_for_rebuilt_strata(1 << 0);

    assert!(
        rebuilt_handle.is_ready(),
        "newly rebuilt request should inherit ready state from persistent ready-path cache"
    );
    assert!(
        !app.cached_render_requests_still_pending(),
        "ready-cache hydrated request should not re-enter the pending queue"
    );
}

fn write_test_png(dir: &std::path::Path, name: &str) {
    let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([0x44, 0x88, 0xcc, 0xff]));
    image.save(dir.join(format!("{name}.png"))).unwrap();
}

/// Stand-in for the draw-side GPU upload: mark every cached request ready and
/// clear the pending flag, as a completed draw does.
fn complete_draw_uploads(app: &App) {
    for batch in app.cached_strata_quads.borrow().iter().flatten() {
        for request in batch
            .texture_requests
            .iter()
            .chain(&batch.mask_texture_requests)
        {
            request.handle.mark_ready();
        }
    }
    app.textures_pending.set(false);
}

fn run_idle_tick_warmup(app: &App) {
    let mut stage_timings = crate::iced_app::update::TickStageTimings::default();
    app.preload_visible_textures_for_tick(&mut stage_timings);
}

#[test]
fn settled_idle_warmup_still_loads_textures_from_rebuilt_strata() {
    let temp_dir = tempdir().unwrap();
    write_test_png(temp_dir.path(), "warmup-alpha");
    write_test_png(temp_dir.path(), "warmup-beta");
    let app = super::test_support::build_test_app_with_addons(Some(temp_dir.path()));
    let size = Size::new(800.0, 600.0);
    app.env
        .borrow()
        .exec(
            r#"
            WarmupTexture = UIParent:CreateTexture(nil, "ARTWORK")
            WarmupTexture:SetSize(32, 32)
            WarmupTexture:SetPoint("CENTER")
            WarmupTexture:SetTexture("Interface/AddOns/warmup-alpha")
        "#,
        )
        .unwrap();
    app.mark_all_strata_dirty();
    app.rebuild_draw_quads(size);
    complete_draw_uploads(&app);

    run_idle_tick_warmup(&app);
    assert!(
        app.texture_manager
            .borrow()
            .is_cached("Interface/AddOns/warmup-alpha")
    );
    complete_draw_uploads(&app);
    run_idle_tick_warmup(&app);

    app.env
        .borrow()
        .exec(r#"WarmupTexture:SetTexture("Interface/AddOns/warmup-beta")"#)
        .unwrap();
    app.mark_all_strata_dirty();
    app.rebuild_draw_quads(size);
    complete_draw_uploads(&app);
    run_idle_tick_warmup(&app);

    assert!(
        app.texture_manager
            .borrow()
            .is_cached("Interface/AddOns/warmup-beta"),
        "a settled warmup must rescan after strata rebuild introduces a new texture"
    );
}
