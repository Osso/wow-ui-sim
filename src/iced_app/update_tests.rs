use super::*;
use crate::iced_app::app::AppInit;
use crate::iced_app::render;
use crate::lua_api::state::CursorInfo;
use crate::render::{FrameQuadSnapshot, QuadBatch, TextureRequest};
use crate::screen::ScreenKind;
use crate::texture::TextureManager;
use iced::Size;
use iced_runtime::Action;
use iced_runtime::futures::futures::StreamExt;
use iced_runtime::window::Action as WindowAction;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use tempfile::tempdir;
use tokio::sync::mpsc;

const ALL_STRATA_DIRTY: u16 = 0x3ff;

fn build_test_app(screen_kind: ScreenKind) -> App {
    build_test_app_with_texture_manager(screen_kind, TextureManager::new())
}

fn build_test_app_with_texture_manager(
    screen_kind: ScreenKind,
    texture_manager: TextureManager,
) -> App {
    let env = Rc::new(RefCell::new(
        WowLuaEnv::new().expect("Failed to create Lua environment"),
    ));
    env.borrow().set_screen_mode(screen_kind);

    let texture_manager = Rc::new(RefCell::new(texture_manager));
    let font_system = Rc::new(RefCell::new(crate::render::WowFontSystem::new()));
    let glyph_atlas = Rc::new(RefCell::new(crate::render::GlyphAtlas::new()));
    let (_cmd_tx, cmd_rx) = mpsc::channel(1);
    let (_lua_tx, lua_rx) = std::sync::mpsc::channel();

    let app = App::build_app(AppInit {
        env,
        log_messages: Vec::new(),
        texture_manager,
        font_system,
        glyph_atlas,
        cmd_rx,
        lua_rx,
        debug_borders: false,
        debug_anchors: false,
        saved_vars: None,
        config: crate::config::SimConfig::default(),
    });
    app.gui_startup_complete.set(true);
    app
}

fn build_test_app_with_addon_textures(screen_kind: ScreenKind, addons_path: &Path) -> App {
    build_test_app_with_texture_manager(
        screen_kind,
        TextureManager::new().with_addons_path(addons_path),
    )
}

fn addon_texture_path(name: &str) -> String {
    format!("Interface/AddOns/TestAddon/{name}")
}

fn write_addon_test_texture(addons_path: &Path, name: &str) {
    let texture_path = addons_path.join("TestAddon").join(format!("{name}.png"));
    if let Some(parent) = texture_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([0x44, 0x88, 0xcc, 0xff]));
    image.save(texture_path).unwrap();
}

#[test]
fn format_tick_stage_log_includes_nested_on_update_breakdown() {
    let log = format_tick_stage_log(
        &TickStageTimings {
            total: Duration::from_millis(100),
            exec_lua: Duration::from_millis(1),
            timers: Duration::from_millis(2),
            layout: Duration::from_millis(30),
            on_update: crate::lua_api::on_update::OnUpdateStageTimings {
                total: Duration::from_millis(40),
                dispatch_handlers: Duration::from_millis(10),
                animation_groups: Duration::from_millis(11),
                on_post_update: Duration::from_millis(12),
                finalize_metrics: Duration::from_millis(3),
                gc_step: Duration::from_millis(4),
            },
            party_health: Duration::from_millis(5),
            casting: Duration::from_millis(6),
            console: Duration::from_millis(7),
            mark_dirty: Duration::from_millis(8),
            preload: Duration::from_millis(9),
        },
        0x3,
        Some(4),
        true,
        42,
    );

    assert!(log.contains("total=100.000ms"));
    assert!(log.contains("layout=30.000"));
    assert!(log.contains("on_update=40.000"));
    assert!(log.contains("handlers=10.000"));
    assert!(log.contains("anim=11.000"));
    assert!(log.contains("post=12.000"));
    assert!(log.contains("metrics=3.000"));
    assert!(log.contains("gc=4.000"));
    assert!(log.contains("preload=9.000"));
    assert!(log.contains("dirty=0x3"));
    assert!(log.contains("ids=Some(4)"));
    assert!(log.contains("pending=true"));
    assert!(log.contains("ready=42"));
}

#[test]
fn collect_tick_dirty_preserves_preexisting_dirty_ids() {
    let mut app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);
    app.selected_rot_level = "Off".to_string();
    app.screen_size.set(Size::new(1024.0, 768.0));
    *app.pending_dirty_ids.borrow_mut() = Some(FxHashSet::default());

    {
        let env = app.env.borrow();
        env.exec("TickDirtyFrame = CreateFrame('Frame', 'TickDirtyFrame', UIParent)")
            .unwrap();
    }

    let frame_id = {
        let env = app.env.borrow();
        let state = env.state().borrow();
        state.widgets.get_id_by_name("TickDirtyFrame").unwrap()
    };

    {
        let env = app.env.borrow();
        let state = env.state().borrow();
        let _ = state.widgets.take_render_dirty_with_ids();
        state.widgets.mark_visual_dirty(frame_id);
    }

    let (dirty_mask, _tick_timings) = app.collect_tick_dirty();
    let pending = app.pending_dirty_ids.borrow().clone().unwrap();

    assert_ne!(
        dirty_mask, 0,
        "pre-tick dirty should contribute to the mask"
    );
    assert!(
        pending.contains(&frame_id),
        "pre-tick dirty frame should survive collect_tick_dirty"
    );
}

#[test]
fn invalidate_after_lua_mutation_keeps_incremental_dirty_ids() {
    let mut app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);
    *app.pending_dirty_ids.borrow_mut() = Some(FxHashSet::default());
    app.cached_frame_snapshots.borrow_mut()[0] =
        Some(HashMap::from([(1_u64, FrameQuadSnapshot::default())]));

    let frame_id = {
        let env = app.env.borrow();
        let state = env.state().borrow();
        state
            .widgets
            .get_id_by_name("UIParent")
            .expect("UIParent should exist")
    };

    {
        let env = app.env.borrow();
        let state = env.state().borrow();
        let _ = state.widgets.take_render_dirty_with_ids();
        state.widgets.mark_visual_dirty(frame_id);
    }

    app.invalidate_after_lua_mutation();

    let all_mask = (1u16 << crate::widget::FrameStrata::COUNT) - 1;
    assert_ne!(
        app.strata_dirty.get(),
        0,
        "dirty strata should be preserved"
    );
    assert_ne!(
        app.strata_dirty.get(),
        all_mask,
        "incremental invalidate should not force all strata dirty"
    );
    let pending = app.pending_dirty_ids.borrow();
    let ids = pending
        .as_ref()
        .expect("incremental invalidate should keep concrete dirty IDs");
    assert!(
        ids.contains(&frame_id),
        "pending dirty IDs should include the Lua-mutated frame"
    );
    assert!(
        app.cached_frame_snapshots.borrow()[0].is_some(),
        "incremental invalidate should not clear snapshot caches"
    );
}

#[test]
fn party_size_change_updates_group_state_and_selection() {
    let mut app = build_test_app(ScreenKind::Game);

    let _ = app.dispatch_simple_message(Message::PartySizeChanged("4".to_string()));

    assert_eq!(app.selected_party_size, "4");
    let env = app.env.borrow();
    let state = env.state().borrow();
    assert_eq!(state.party_members.len(), 4);
    assert!(
        state.party_group_active,
        "party size > 0 should mark the player as grouped"
    );
    assert_eq!(
        state.party_leader_index, None,
        "GUI party-size changes should default leadership to the local player like A_Admin.SetPartySize"
    );
}

#[test]
fn fast_timer_ticks_go_stale_quickly() {
    assert!(should_drop_stale_timer_tick(
        std::time::Duration::from_millis(150),
        std::time::Duration::from_millis(16),
    ));
}

#[test]
fn slow_timer_ticks_get_more_slack() {
    assert!(!should_drop_stale_timer_tick(
        std::time::Duration::from_millis(150),
        std::time::Duration::from_secs(1),
    ));
    assert!(should_drop_stale_timer_tick(
        std::time::Duration::from_secs(3),
        std::time::Duration::from_secs(1),
    ));
}

#[test]
fn queued_stale_timer_ticks_do_not_run_tick_work() {
    let mut app = build_test_app(ScreenKind::Game);
    app.screen_size.set(Size::new(1024.0, 768.0));
    app.textures_pending.set(true);
    app.selected_rot_level = "Off".to_string();
    let initial_on_update = app.last_on_update_time;
    let stale_captured_at = Instant::now() - std::time::Duration::from_secs(1);

    for _ in 0..8 {
        let _ = app.update(Message::ProcessTimers(stale_captured_at));
    }

    assert_eq!(app.dropped_stale_timer_ticks.get(), 8);
    assert!(
        app.oldest_dropped_timer_tick_age.get() >= std::time::Duration::from_secs(1),
        "stale tick age should track the queued backlog"
    );
    assert_eq!(
        app.last_on_update_time, initial_on_update,
        "stale timer ticks should not reach OnUpdate processing"
    );

    app.options_modal_visible = true;
    let _ = app.dispatch_simple_message(Message::KeyPress(
        "ESCAPE".to_string(),
        None,
        Instant::now(),
    ));

    assert!(
        !app.options_modal_visible,
        "escape should still be handled promptly"
    );
    assert_eq!(
        app.dropped_stale_timer_ticks.get(),
        0,
        "keypress log accounting should reset after reporting dropped ticks"
    );
    assert_eq!(
        app.oldest_dropped_timer_tick_age.get(),
        std::time::Duration::ZERO,
        "keypress log accounting should clear the recorded backlog age"
    );
}

#[test]
fn tick_warmup_keeps_draw_pending_when_it_decodes_visible_textures() {
    let temp_dir = tempdir().unwrap();
    let texture_path = addon_texture_path("tick-warmup");
    write_addon_test_texture(temp_dir.path(), "tick-warmup");

    let mut app = build_test_app_with_addon_textures(ScreenKind::Game, temp_dir.path());
    app.screen_size.set(Size::new(1024.0, 768.0));
    app.selected_rot_level = "Off".to_string();
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    {
        let env = app.env.borrow();
        env.exec(&format!(
            r#"
            local frame = CreateFrame("Frame", "TickWarmupFrame", UIParent)
            local texture = frame:CreateTexture(nil, "ARTWORK")
            texture:SetTexture("{texture_path}")
        "#,
        ))
        .unwrap();
        let _ = env.state().borrow().widgets.take_render_dirty_with_ids();
    }

    let task = app.handle_process_timers(Instant::now());
    let action = pollster::block_on(async {
        iced_runtime::task::into_stream(task)
            .expect("decode progress should request a redraw")
            .next()
            .await
            .expect("task should emit a redraw action")
    });

    assert_eq!(
        app.strata_dirty.get(),
        0,
        "warmup decode should not force a full strata rebuild by itself"
    );
    assert!(
        app.textures_pending.get(),
        "decoded textures should stay pending until draw uploads and resolves them"
    );
    assert!(
        app.texture_manager.borrow().get(&texture_path).is_some(),
        "tick-start warmup should decode the visible texture source"
    );
    assert!(
        matches!(action, Action::Window(WindowAction::RedrawAll)),
        "decode progress should request a redraw for the next draw pass"
    );
}

#[test]
fn process_timers_requests_redraw_when_strata_are_dirty() {
    let mut app = build_test_app(ScreenKind::Game);
    app.screen_size.set(Size::new(1024.0, 768.0));
    app.selected_rot_level = "Off".to_string();
    app.strata_dirty.set(1);

    let task = app.handle_process_timers(Instant::now());
    let action = pollster::block_on(async {
        iced_runtime::task::into_stream(task)
            .expect("redraw task should produce a runtime action")
            .next()
            .await
            .expect("task should emit a redraw action")
    });

    assert!(
        matches!(action, Action::Window(WindowAction::RedrawAll)),
        "dirty strata should still request a redraw"
    );
}

#[test]
fn canvas_event_requests_redraw_when_it_leaves_dirty_strata() {
    let mut app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(1);

    let task = app.handle_canvas_event(CanvasMessage::MouseLeave);
    let action = pollster::block_on(async {
        iced_runtime::task::into_stream(task)
            .expect("canvas event with dirty strata should request a redraw")
            .next()
            .await
            .expect("task should emit a redraw action")
    });

    assert!(
        matches!(action, Action::Window(WindowAction::RedrawAll)),
        "canvas event should request redraw when it leaves strata dirty"
    );
}

#[test]
fn mouse_move_ignores_pending_texture_redraw_when_nothing_visual_changed() {
    let mut app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);
    app.textures_pending.set(true);
    {
        let env = app.env.borrow();
        let _ = env.state().borrow().widgets.take_render_dirty_with_ids();
    }

    let task = app.handle_canvas_event(CanvasMessage::MouseMove(iced::Point::new(10.0, 10.0)));

    assert!(
        iced_runtime::task::into_stream(task).is_none(),
        "pending textures alone must not turn mouse movement into a redraw loop"
    );
}

#[test]
fn mouse_move_with_cursor_item_still_requests_redraw() {
    let mut app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);
    {
        let env = app.env.borrow();
        env.state().borrow_mut().cursor_item = Some(CursorInfo::Spell { spell_id: 123 });
        let _ = env.state().borrow().widgets.take_render_dirty_with_ids();
    }

    let task = app.handle_canvas_event(CanvasMessage::MouseMove(iced::Point::new(10.0, 10.0)));
    let action = pollster::block_on(async {
        iced_runtime::task::into_stream(task)
            .expect("cursor item movement should request a redraw")
            .next()
            .await
            .expect("task should emit a redraw action")
    });

    assert!(
        matches!(action, Action::Window(WindowAction::RedrawAll)),
        "cursor-carried overlay still needs to follow mouse movement"
    );
}

#[test]
fn tick_warmup_prefers_cached_render_crop_requests() {
    let temp_dir = tempdir().unwrap();
    let texture_path = addon_texture_path("tick-warmup-crop");
    write_addon_test_texture(temp_dir.path(), "tick-warmup-crop");

    let app = build_test_app_with_addon_textures(ScreenKind::Game, temp_dir.path());
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    let crop_path = format!("{texture_path}@crop:0.000000,0.500000,0.000000,0.500000");
    let mut batch = QuadBatch::new();
    batch
        .texture_requests
        .push(TextureRequest::new(&crop_path, 0, 4));
    app.cached_strata_quads.borrow_mut()[0] = Some(std::sync::Arc::new(batch));

    app.preload_visible_textures_with_budget(
        std::time::Duration::from_millis(50),
        crate::iced_app::app::ALL_STRATA_MASK,
    );

    let tex_mgr = app.texture_manager.borrow();
    assert!(
        tex_mgr.get_cached_crop_request(&crop_path).is_some(),
        "tick warmup should cache the current render crop request itself"
    );
    assert!(
        app.textures_pending.get(),
        "CPU-cached render requests must stay pending until the live draw uploads them"
    );
}

#[test]
fn tick_warmup_keeps_staged_but_unprepared_requests_pending() {
    let temp_dir = tempdir().unwrap();
    write_addon_test_texture(temp_dir.path(), "staged-not-prepared");

    let app = build_test_app_with_addon_textures(ScreenKind::Game, temp_dir.path());
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    let request_path = addon_texture_path("staged-not-prepared");
    let mut batch = QuadBatch::new();
    batch
        .texture_requests
        .push(TextureRequest::new(&request_path, 0, 4));
    batch.texture_requests[0].handle.mark_staged();
    app.cached_strata_quads.borrow_mut()[0] = Some(std::sync::Arc::new(batch));

    {
        let mut tex_mgr = app.texture_manager.borrow_mut();
        render::preload_texture_request_source(&mut tex_mgr, &request_path);
    }
    app.preload_visible_textures_with_budget(
        std::time::Duration::from_millis(50),
        crate::iced_app::app::ALL_STRATA_MASK,
    );

    assert!(
        app.textures_pending.get(),
        "draw-staged requests must stay pending until prepare uploads them into the atlas"
    );
}

#[test]
fn process_timers_requests_redraw_when_pending_is_newly_discovered() {
    let temp_dir = tempdir().unwrap();
    write_addon_test_texture(temp_dir.path(), "staged-discovered");

    let mut app = build_test_app_with_addon_textures(ScreenKind::Game, temp_dir.path());
    app.screen_size.set(Size::new(1024.0, 768.0));
    app.selected_rot_level = "Off".to_string();
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    let request_path = addon_texture_path("staged-discovered");
    let mut batch = QuadBatch::new();
    batch
        .texture_requests
        .push(TextureRequest::new(&request_path, 0, 4));
    batch.texture_requests[0].handle.mark_staged();
    app.cached_strata_quads.borrow_mut()[0] = Some(std::sync::Arc::new(batch));

    {
        let mut tex_mgr = app.texture_manager.borrow_mut();
        render::preload_texture_request_source(&mut tex_mgr, &request_path);
    }
    let task = app.update(Message::ProcessTimers(Instant::now()));
    let action = pollster::block_on(async {
        iced_runtime::task::into_stream(task)
            .expect("newly discovered pending draw work should request a redraw")
            .next()
            .await
            .expect("task should emit a redraw action")
    });

    assert!(
        app.textures_pending.get(),
        "staged-but-unprepared requests should become pending"
    );
    assert!(
        matches!(action, Action::Window(WindowAction::RedrawAll)),
        "first discovery of unresolved draw work should request a redraw"
    );
}

#[test]
fn process_timers_skips_redraw_when_pending_state_does_not_progress() {
    let mut app = build_test_app(ScreenKind::Game);
    app.screen_size.set(Size::new(1024.0, 768.0));
    app.selected_rot_level = "Off".to_string();
    app.strata_dirty.set(0);
    app.textures_pending.set(true);
    {
        let env = app.env.borrow();
        let _ = env.state().borrow().widgets.take_render_dirty_with_ids();
    }

    let task = app.handle_process_timers(Instant::now());

    assert!(
        iced_runtime::task::into_stream(task).is_none(),
        "pending state alone should not force redraws every tick"
    );
}

#[test]
fn process_timers_skips_visible_warmup_while_draw_work_is_pending() {
    let temp_dir = tempdir().unwrap();
    let texture_path = temp_dir.path().join("pending-skip-warmup.png");
    let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([0x44, 0x88, 0xcc, 0xff]));
    image.save(&texture_path).unwrap();

    let mut app = build_test_app(ScreenKind::Game);
    app.screen_size.set(Size::new(1024.0, 768.0));
    app.selected_rot_level = "Off".to_string();
    app.strata_dirty.set(0);
    app.textures_pending.set(true);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            local frame = CreateFrame("Frame", "PendingSkipWarmupFrame", UIParent)
            local texture = frame:CreateTexture(nil, "ARTWORK")
            texture:SetTexture("pending-skip-warmup")
        "#,
        )
        .unwrap();
        let _ = env.state().borrow().widgets.take_render_dirty_with_ids();
    }

    let task = app.handle_process_timers(Instant::now());
    assert!(
        iced_runtime::task::into_stream(task).is_none(),
        "draw-owned pending state should not trigger another warmup decode pass"
    );
    assert!(
        app.texture_manager
            .borrow()
            .get("pending-skip-warmup")
            .is_none(),
        "visible warmup should be skipped while draw work is already pending"
    );
}

#[test]
fn process_timers_skips_visible_warmup_while_strata_are_dirty() {
    let temp_dir = tempdir().unwrap();
    let texture_path = temp_dir.path().join("dirty-skip-warmup.png");
    let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([0xcc, 0x88, 0x44, 0xff]));
    image.save(&texture_path).unwrap();

    let mut app = build_test_app(ScreenKind::Game);
    app.screen_size.set(Size::new(1024.0, 768.0));
    app.selected_rot_level = "Off".to_string();
    app.strata_dirty.set(ALL_STRATA_DIRTY);
    app.textures_pending.set(false);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            local frame = CreateFrame("Frame", "DirtySkipWarmupFrame", UIParent)
            local texture = frame:CreateTexture(nil, "ARTWORK")
            texture:SetTexture("dirty-skip-warmup")
        "#,
        )
        .unwrap();
        let _ = env.state().borrow().widgets.take_render_dirty_with_ids();
    }

    let _task = app.handle_process_timers(Instant::now());
    assert!(
        app.texture_manager
            .borrow()
            .get("dirty-skip-warmup")
            .is_none(),
        "visible warmup should be skipped while strata need a render rebuild"
    );
}

#[test]
fn process_timers_treats_queued_preloads_as_background_work() {
    let temp_dir = tempdir().unwrap();
    let visible_texture = addon_texture_path("queued-visible");
    let queued_texture = addon_texture_path("queued-target");
    write_addon_test_texture(temp_dir.path(), "queued-visible");
    write_addon_test_texture(temp_dir.path(), "queued-target");

    let mut app = build_test_app_with_addon_textures(ScreenKind::Game, temp_dir.path());
    app.screen_size.set(Size::new(1024.0, 768.0));
    app.selected_rot_level = "Off".to_string();
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    {
        let env = app.env.borrow();
        env.exec(&format!(
            r#"
            local frame = CreateFrame("Frame", "QueuedPreloadVisibleFrame", UIParent)
            local texture = frame:CreateTexture(nil, "ARTWORK")
            texture:SetTexture("{visible_texture}")
        "#,
        ))
        .unwrap();
        env.state()
            .borrow_mut()
            .enqueue_texture_preloads([queued_texture.clone()]);
        let _ = env.state().borrow().widgets.take_render_dirty_with_ids();
    }

    let task = app.handle_process_timers(Instant::now());
    let action = pollster::block_on(async {
        iced_runtime::task::into_stream(task)
            .expect("visible warmup progress should request a redraw")
            .next()
            .await
            .expect("task should emit a redraw action")
    });
    assert!(
        matches!(action, Action::Window(WindowAction::RedrawAll)),
        "visible warmup progress should still request a redraw"
    );

    let tex_mgr = app.texture_manager.borrow();
    assert!(
        tex_mgr.get(&queued_texture).is_none(),
        "background queued preloads should not decode during the GUI timer tick"
    );
    assert!(
        tex_mgr.get(&visible_texture).is_some(),
        "visible texture warmup should still run when render work is otherwise idle"
    );
    drop(tex_mgr);
    assert!(
        app.env
            .borrow()
            .state()
            .borrow()
            .pending_texture_preloads
            .contains(&queued_texture),
        "background queued preloads should remain queued for explicit/background processing"
    );
}

#[test]
fn collect_tick_dirty_preserves_full_rebuild_sentinel() {
    let mut app = build_test_app(ScreenKind::Game);
    app.pending_dirty_ids.borrow_mut().take();
    app.mark_all_strata_dirty();

    {
        let env = app.env.borrow();
        let frame = env
            .state()
            .borrow()
            .widgets
            .get_id_by_name("UIParent")
            .expect("UIParent should exist");
        env.state().borrow().widgets.mark_visual_dirty(frame);
    }

    let _ = app.collect_tick_dirty();

    assert!(
        app.pending_dirty_ids.borrow().is_none(),
        "full rebuild sentinel must survive later per-frame dirty collection"
    );
}

#[test]
fn sample_display_metrics_split_frame_budget() {
    let metrics =
        sample_display_metrics(std::time::Duration::from_secs_f32(1.0), 10, 10, 25.0, 15.0);

    assert!((metrics.fps - 10.0).abs() < 0.001);
    assert!((metrics.draw_ms - 2.5).abs() < 0.001);
    assert!((metrics.tick_ms - 1.5).abs() < 0.001);
    assert!((metrics.other_ms - 96.0).abs() < 0.001);
}

// ── Tick interval stability ──────────────────────────────────────────────────

#[test]
fn stable_timer_interval_uses_fixed_buckets() {
    use crate::iced_app::app::stable_timer_interval;
    use std::time::Duration;
    assert_eq!(
        stable_timer_interval(Duration::from_millis(0)),
        Duration::from_millis(16)
    );
    assert_eq!(
        stable_timer_interval(Duration::from_millis(40)),
        Duration::from_millis(16)
    );
    assert_eq!(
        stable_timer_interval(Duration::from_millis(120)),
        Duration::from_millis(50)
    );
    assert_eq!(
        stable_timer_interval(Duration::from_millis(800)),
        Duration::from_millis(250)
    );
    assert_eq!(
        stable_timer_interval(Duration::from_secs(600)),
        Duration::from_millis(1000)
    );
}

#[test]
fn compute_tick_interval_is_stable_while_pending_timer_shrinks() {
    // iced keys the time::every subscription by the interval value and the
    // subscription set is rebuilt after every update. If this returns the raw
    // remaining time, every update recreates the tick stream (resetting its
    // first fire), and under continuous input ProcessTimers starves: pending
    // C_Timer chains stall and the FPS display freezes at its last value.
    let mut app = build_test_app(ScreenKind::Game);
    app.gui_startup_complete.set(true);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);
    app.selected_rot_level = "Off".to_string();

    {
        let env = app.env.borrow();
        env.exec("C_Timer.After(0.8, function() end)").unwrap();
    }

    let first = app.compute_tick_interval();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let second = app.compute_tick_interval();

    assert!(first.is_some(), "pending timer must schedule a tick");
    assert_eq!(
        first, second,
        "tick interval must stay identical across updates while a pending \
         timer's remaining time shrinks, or the iced subscription is torn \
         down and recreated before it can ever fire"
    );
}

#[test]
fn stale_timer_ticks_cannot_starve_processing_indefinitely() {
    // Under sustained main-thread saturation every tick arrives older than
    // the stale threshold. Dropping all of them starves OnUpdate / C_Timer
    // processing and freezes the FPS display at its last value. After
    // STALE_TICK_PROGRESS_BUDGET without a processed tick, a stale tick must
    // be processed anyway.
    let mut app = build_test_app(ScreenKind::Game);
    app.gui_startup_complete.set(true);
    app.strata_dirty.set(1); // fast 16ms interval -> 100ms stale threshold
    let stale = std::time::Instant::now() - std::time::Duration::from_secs(1);

    // Budget exceeded: stale tick is processed, not dropped.
    app.last_timer_tick_processed
        .set(std::time::Instant::now() - std::time::Duration::from_millis(300));
    let ticks_before = app.tick_count.get();
    let _ = app.handle_process_timers(stale);
    assert_eq!(
        app.tick_count.get(),
        ticks_before + 1,
        "stale tick must be force-processed after the progress budget"
    );
    assert_eq!(app.dropped_stale_timer_ticks.get(), 0);

    // Budget fresh (just processed): the next stale tick is dropped.
    let _ = app.handle_process_timers(stale);
    assert_eq!(
        app.tick_count.get(),
        ticks_before + 1,
        "stale tick within the budget must still be dropped"
    );
    assert_eq!(app.dropped_stale_timer_ticks.get(), 1);
}
