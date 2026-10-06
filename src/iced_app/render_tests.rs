#[cfg(test)]
mod tests {
    use super::*;
    use crate::iced_app::{build_hittable_rects, frame_collect::collect_hittable_frames};
    use crate::render::FrameQuadSnapshot;
    use crate::texture::{TextureManager, normalize_wow_path};
    use iced::Point;
    use rustc_hash::FxHashSet;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn dirty_mask(strata: usize) -> u16 {
        1u16 << strata
    }

    fn build_test_app() -> App {
        super::test_support::build_test_app()
    }

    #[test]
    fn cursor_moved_outside_canvas_publishes_mouse_leave() {
        let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(100.0, 80.0));
        let event = mouse::Event::CursorMoved {
            position: Point::new(150.0, 40.0),
        };

        let action = handle_mouse_event(&event, bounds, mouse::Cursor::Unavailable)
            .expect("outside canvas movement should publish a canvas leave event");
        let (message, _, _) = action.into_inner();

        assert!(
            matches!(
                message,
                Some(Message::CanvasEvent(CanvasMessage::MouseLeave))
            ),
            "outside canvas movement should clear hover state"
        );
    }

    #[test]
    fn window_unfocused_publishes_mouse_leave() {
        let app = build_test_app();
        let mut shader_state = ();
        let action = <&App as shader::Program<Message>>::update(
            &&app,
            &mut shader_state,
            &Event::Window(window::Event::Unfocused),
            Rectangle::new(Point::ORIGIN, Size::new(100.0, 80.0)),
            mouse::Cursor::Unavailable,
        )
        .expect("window unfocus should publish a canvas leave event");
        let (message, _, _) = action.into_inner();

        assert!(
            matches!(
                message,
                Some(Message::CanvasEvent(CanvasMessage::MouseLeave))
            ),
            "window unfocus should clear hover state"
        );
    }

    #[test]
    fn format_texture_preload_log_reports_budget_reason_and_samples() {
        let log = format_texture_preload_log(&TexturePreloadPassTelemetry {
            elapsed: std::time::Duration::from_millis(26),
            budget: Some(std::time::Duration::from_millis(25)),
            queued: 2,
            loaded: 1,
            remaining: 1,
            remaining_sample: vec!["queued-a".to_string()],
            pending: true,
        });

        assert!(log.contains("elapsed=26.000ms"));
        assert!(log.contains("budget_ms=25.000"));
        assert!(log.contains("queued=2"));
        assert!(log.contains("loaded=1"));
        assert!(log.contains("remaining=1"));
        assert!(log.contains("pending=true"));
        assert!(log.contains("reason=queued_budget"));
        assert!(log.contains("sample=queued-a"));
    }

    #[test]
    fn texture_preload_reason_reports_complete_after_queue_drains() {
        assert_eq!(
            texture_preload_reason(&TexturePreloadPassTelemetry::default()),
            "complete"
        );
    }

    fn file_data_id_to_wow_path(file_data_id: u32) -> Option<String> {
        let path = crate::manifest_interface_data::get_texture_path(file_data_id)?;
        Some(format!("Interface\\{}", path.replace('/', "\\")))
    }

    fn first_map_with_art_and_overlay_paths() -> Option<(u32, String, String)> {
        let mut texture_manager = TextureManager::new();
        for map_id in 1..=10_000 {
            let Some((art_paths, overlay_paths)) = map_preload_paths(map_id) else {
                continue;
            };
            if all_paths_loadable(&mut texture_manager, &art_paths, &overlay_paths) {
                return Some((map_id, art_paths[0].clone(), overlay_paths[0].clone()));
            }
        }
        None
    }

    fn map_preload_paths(map_id: u32) -> Option<(Vec<String>, Vec<String>)> {
        let art_paths: Vec<_> = crate::map_art::get_map_art(map_id)?
            .tiles
            .iter()
            .flat_map(|tiles| tiles.iter().copied())
            .filter_map(file_data_id_to_wow_path)
            .collect();
        let overlay_paths: Vec<_> = crate::map_exploration::get_overlays_for_map(map_id)?
            .iter()
            .flat_map(|overlay| overlay.file_data_ids.iter().copied())
            .filter_map(file_data_id_to_wow_path)
            .collect();
        (!art_paths.is_empty() && !overlay_paths.is_empty()).then_some((art_paths, overlay_paths))
    }

    fn all_paths_loadable(
        texture_manager: &mut TextureManager,
        art_paths: &[String],
        overlay_paths: &[String],
    ) -> bool {
        art_paths
            .iter()
            .chain(overlay_paths.iter())
            .all(|path| {
                preload_texture_request_source(texture_manager, path);
                texture_manager.is_cached(path)
            })
    }

    fn write_test_texture(base: &Path, wow_path: &str, color: [u8; 4]) {
        let normalized = normalize_wow_path(wow_path);
        let relative = normalized
            .strip_prefix("Interface/")
            .unwrap_or(normalized.as_str());
        let file_path = base.join(format!("{relative}.webp"));
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let image = image::RgbaImage::from_pixel(4, 4, image::Rgba(color));
        image.save(&file_path).unwrap();
    }

    fn rebuild_after_widget_dirty(app: &App, size: Size) -> u16 {
        let (dirty_mask, dirty_ids) = app
            .env
            .borrow()
            .state()
            .borrow()
            .widgets
            .take_render_dirty_with_ids();
        app.mark_strata_dirty(dirty_mask);
        app.merge_pending_dirty_ids(dirty_ids);
        app.rebuild_dirty_strata(size, dirty_mask)
    }

    fn texture_request_alphas(app: &App, needle: &str) -> Vec<f32> {
        let mut alphas = Vec::new();
        let strata = app.cached_strata_quads.borrow();
        for batch in strata.iter().flatten() {
            for request in &batch.texture_requests {
                if !request.path.contains(needle) {
                    continue;
                }
                let start = request.vertex_start as usize;
                let end = start + request.vertex_count as usize;
                alphas.extend(
                    batch.vertices[start..end]
                        .iter()
                        .map(|vertex| vertex.color[3]),
                );
            }
        }
        alphas
    }

    fn snapshot_texture_alphas(app: &App, frame_id: u64) -> Vec<f32> {
        let mut alphas = Vec::new();
        let snapshots = app.cached_frame_snapshots.borrow();
        for snapshot in snapshots.iter().flatten() {
            let Some(snapshot) = snapshot.get(&frame_id) else {
                continue;
            };
            for request in &snapshot.texture_requests {
                let start = request.vertex_start as usize;
                let end = start + request.vertex_count as usize;
                alphas.extend(
                    snapshot.vertices[start..end]
                        .iter()
                        .map(|vertex| vertex.color[3]),
                );
            }
        }
        alphas
    }

    fn snapshot_texture_paths(app: &App, frame_id: u64) -> Vec<String> {
        let mut paths = Vec::new();
        let snapshots = app.cached_frame_snapshots.borrow();
        for snapshot in snapshots.iter().flatten() {
            let Some(snapshot) = snapshot.get(&frame_id) else {
                continue;
            };
            paths.extend(
                snapshot
                    .texture_requests
                    .iter()
                    .map(|request| request.path.clone()),
            );
        }
        paths
    }

    fn overlay_texture_request_count(app: &App) -> usize {
        app.build_overlay().texture_requests.len()
    }

    #[test]
    fn cursor_talent_icon_renders_in_overlay() {
        const TALENT_DEFINITION_ID: u32 = 137759;
        let mut app = build_test_app();
        app.mouse_position = Some(Point::new(48.0, 48.0));
        app.env.borrow().state().borrow_mut().cursor_item =
            Some(crate::lua_api::state::CursorInfo::Talent {
                talent_id: TALENT_DEFINITION_ID,
                pvp: false,
            });

        let overlay = app.build_overlay();
        assert_eq!(overlay.texture_requests.len(), 2, "talent icon plus pointer");
        assert_ne!(overlay.texture_requests[0].path, r"Interface\Cursor\Point");
        assert_eq!(overlay.texture_requests[1].path, r"Interface\Cursor\Point");
    }

    fn mark_frames_dirty(app: &App, frame_ids: &[u64]) {
        let env = app.env.borrow();
        let state = env.state().borrow();
        for frame_id in frame_ids {
            state.widgets.mark_visual_dirty(*frame_id);
        }
    }

    fn rebuild_hittable_cache(app: &App, size: Size) {
        let env = app.env.borrow();
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let strata_buckets = state
            .get_strata_buckets()
            .expect("visible strata buckets should exist")
            .clone();
        let collected = collect_hittable_frames(&state.widgets, &strata_buckets);
        let hittable = build_hittable_rects(&collected, &state.widgets);
        let grid = super::super::hit_grid::HitGrid::new(hittable, size.width, size.height);
        *app.cached_hittable.borrow_mut() = Some(grid);
    }

    #[test]
    fn scaled_frames_scale_hit_rect_insets() {
        let app = build_test_app();
        app.env
            .borrow()
            .exec(
                r#"
                ScaledHitInsetButton = CreateFrame("Button", "ScaledHitInsetButton", UIParent)
                ScaledHitInsetButton:SetSize(32, 32)
                ScaledHitInsetButton:SetScale(0.625)
                ScaledHitInsetButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
                ScaledHitInsetButton:EnableMouse(true)
                ScaledHitInsetButton:SetHitRectInsets(6, 6, 7, 7)
            "#,
            )
            .expect("scaled hit inset frame setup should succeed");

        let env = app.env.borrow();
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let strata_buckets = state
            .get_strata_buckets()
            .expect("visible strata buckets should exist")
            .clone();
        let frame_id = state
            .widgets
            .get_id_by_name("ScaledHitInsetButton")
            .expect("scaled hit inset button should exist");
        let collected = collect_hittable_frames(&state.widgets, &strata_buckets);
        let hittable = build_hittable_rects(&collected, &state.widgets);
        let (_, rect, _) = hittable
            .iter()
            .find(|(id, _, _)| *id == frame_id)
            .expect("scaled hit inset button should be hittable");

        assert_eq!(rect.width, 12.5);
        assert_eq!(rect.height, 11.25);
    }

    #[test]
    fn cached_button_normal_texture_alpha_restores_after_hover_hide() {
        let temp_dir = tempdir().unwrap();
        let normal_path = "Interface/Buttons/UI-Panel-Button-Up";
        write_test_texture(temp_dir.path(), normal_path, [0xaa, 0x22, 0x22, 0xff]);

        let app = build_test_app();
        app.env
            .borrow()
            .exec(
                r#"
                CachedHoverButton = CreateFrame("Button", "CachedHoverButton", UIParent)
                CachedHoverButton:SetSize(100, 40)
                CachedHoverButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 20, -20)
                CachedHoverButton:SetNormalTexture("Interface/Buttons/UI-Panel-Button-Up")
            "#,
            )
            .expect("cached hover button setup should succeed");

        let size = Size::new(320.0, 240.0);
        *app.pending_dirty_ids.borrow_mut() = None;
        app.mark_all_strata_dirty();
        app.rebuild_dirty_strata(size, app.strata_dirty.get());

        let initial_alphas = texture_request_alphas(&app, "UI-Panel-Button-Up");
        assert!(
            initial_alphas.contains(&1.0),
            "initial normal texture should render opaque"
        );

        app.env
            .borrow()
            .exec("CachedHoverButton:GetNormalTexture():SetAlpha(0)")
            .expect("normal texture should hide");
        rebuild_after_widget_dirty(&app, size);
        let hidden_alphas = texture_request_alphas(&app, "UI-Panel-Button-Up");
        assert!(
            hidden_alphas.iter().all(|alpha| *alpha == 0.0),
            "normal texture should render transparent while hover code hides it"
        );

        app.env
            .borrow()
            .exec("CachedHoverButton:GetNormalTexture():SetAlpha(1)")
            .expect("normal texture should restore");
        rebuild_after_widget_dirty(&app, size);
        let restored_alphas = texture_request_alphas(&app, "UI-Panel-Button-Up");
        assert!(
            restored_alphas.contains(&1.0),
            "normal texture should render opaque again after OnLeave restores alpha"
        );
    }

    #[test]
    fn cached_button_state_texture_restores_normal_after_hover() {
        let mut app = build_test_app();
        app.env
            .borrow()
            .exec(
                r#"
                CachedMicroButton = CreateFrame("Button", "CachedMicroButton", UIParent)
                CachedMicroButton:SetSize(32, 40)
                CachedMicroButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 20, -20)
                CachedMicroButton:SetNormalAtlas("UI-HUD-MicroMenu-Professions-Up")
                CachedMicroButton:SetHighlightAtlas("UI-HUD-MicroMenu-Professions-Mouseover", "BLEND")
            "#,
            )
            .expect("cached micro button setup should succeed");

        let (button_id, normal_id, highlight_id) = {
            let env = app.env.borrow();
            let state = env.state().borrow();
            let button = state
                .widgets
                .get_by_name("CachedMicroButton")
                .expect("cached micro button should exist");
            let normal_id = *button
                .children_keys
                .get("NormalTexture")
                .expect("normal texture child should exist");
            let highlight_id = *button
                .children_keys
                .get("HighlightTexture")
                .expect("highlight texture child should exist");
            let normal = state
                .widgets
                .get(normal_id)
                .expect("normal texture child should resolve");
            assert_eq!(
                normal.atlas_tex_coords, normal.tex_coords,
                "button SetNormalAtlas should preserve atlas sub-region metadata on the child texture",
            );
            (button.id, normal_id, highlight_id)
        };

        let size = Size::new(320.0, 240.0);
        *app.pending_dirty_ids.borrow_mut() = None;
        app.mark_all_strata_dirty();
        app.rebuild_dirty_strata(size, app.strata_dirty.get());

        app.env
            .borrow()
            .exec("CachedMicroButton:GetNormalTexture():SetAlpha(0)")
            .expect("normal texture should hide on hover");
        {
            app.env.borrow().state().borrow_mut().hovered_frame = Some(button_id);
        }
        app.hovered_frame = Some(button_id);
        mark_frames_dirty(&app, &[button_id, normal_id, highlight_id]);
        rebuild_after_widget_dirty(&app, size);
        let hovered_overlay_requests = overlay_texture_request_count(&app);
        assert!(
            hovered_overlay_requests > 0,
            "hover should emit the highlight texture through the overlay batch \
             (append_hover_highlight)"
        );

        app.env
            .borrow()
            .exec("CachedMicroButton:GetNormalTexture():SetAlpha(1)")
            .expect("normal texture should restore after hover");
        {
            app.env.borrow().state().borrow_mut().hovered_frame = None;
        }
        app.hovered_frame = None;
        mark_frames_dirty(&app, &[button_id, normal_id, highlight_id]);
        rebuild_after_widget_dirty(&app, size);

        assert!(
            snapshot_texture_alphas(&app, normal_id).contains(&1.0),
            "leaving hover should re-emit the normal texture at full alpha"
        );
        assert!(
            snapshot_texture_paths(&app, normal_id)
                .iter()
                .any(|path| path.contains("@crop:")),
            "restored normal texture should render through an isolated atlas crop"
        );
        let unhovered_overlay_requests = overlay_texture_request_count(&app);
        assert!(
            unhovered_overlay_requests < hovered_overlay_requests,
            "leaving hover should drop highlight texture requests from the overlay batch \
             (was {hovered_overlay_requests}, now {unhovered_overlay_requests})"
        );
    }

    #[test]
    fn mouse_leave_rebuild_restores_button_normal_texture() {
        let mut app = build_test_app();
        app.env
            .borrow()
            .exec(
                r#"
                MouseLeaveMicroButton = CreateFrame("Button", "MouseLeaveMicroButton", UIParent)
                MouseLeaveMicroButton:SetSize(32, 40)
                MouseLeaveMicroButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 20, -20)
                MouseLeaveMicroButton:SetNormalAtlas("UI-HUD-MicroMenu-Professions-Up")
                MouseLeaveMicroButton:SetHighlightAtlas("UI-HUD-MicroMenu-Professions-Mouseover", "BLEND")
                MouseLeaveMicroButton:SetScript("OnEnter", function(self)
                    self:GetNormalTexture():SetAlpha(0)
                end)
                MouseLeaveMicroButton:SetScript("OnLeave", function(self)
                    self:GetNormalTexture():SetAlpha(1)
                end)
            "#,
            )
            .expect("mouse-leave micro button setup should succeed");

        let normal_id = {
            let env = app.env.borrow();
            let state = env.state().borrow();
            let button = state
                .widgets
                .get_by_name("MouseLeaveMicroButton")
                .expect("mouse-leave micro button should exist");
            let normal_id = *button
                .children_keys
                .get("NormalTexture")
                .expect("normal texture child should exist");
            let _ = *button
                .children_keys
                .get("HighlightTexture")
                .expect("highlight texture child should exist");
            normal_id
        };

        let size = Size::new(320.0, 240.0);
        app.mark_all_strata_dirty();
        app.rebuild_dirty_strata(size, app.strata_dirty.get());

        app.handle_mouse_move(Point::new(30.0, 40.0));
        app.rebuild_dirty_strata(size, app.strata_dirty.get());
        let hovered_overlay_requests = overlay_texture_request_count(&app);
        assert!(
            hovered_overlay_requests > 0,
            "hover should emit the highlight texture through the overlay batch \
             (append_hover_highlight) on the real mouse path"
        );

        app.handle_mouse_leave();
        app.rebuild_dirty_strata(size, app.strata_dirty.get());
        assert!(
            snapshot_texture_alphas(&app, normal_id).contains(&1.0),
            "mouse leave should re-emit the normal texture after OnLeave restores alpha"
        );
        let unhovered_overlay_requests = overlay_texture_request_count(&app);
        assert!(
            unhovered_overlay_requests < hovered_overlay_requests,
            "mouse leave should drop highlight texture requests from the overlay batch \
             (was {hovered_overlay_requests}, now {unhovered_overlay_requests})"
        );
    }

    #[test]
    fn mouse_up_rebuild_restores_pressed_button_normal_texture() {
        let mut app = build_test_app();
        app.env
            .borrow()
            .exec(
                r#"
                PressedMicroButton = CreateFrame("Button", "PressedMicroButton", UIParent)
                PressedMicroButton:SetSize(32, 40)
                PressedMicroButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 20, -20)
                PressedMicroButton:EnableMouse(true)
                PressedMicroButton:SetNormalAtlas("UI-HUD-MicroMenu-SpecTalents-Up")
                PressedMicroButton:SetPushedAtlas("UI-HUD-MicroMenu-SpecTalents-Down")
            "#,
            )
            .expect("pressed micro button setup should succeed");

        let (normal_id, pushed_id) = {
            let env = app.env.borrow();
            let state = env.state().borrow();
            let button = state
                .widgets
                .get_by_name("PressedMicroButton")
                .expect("pressed micro button should exist");
            let normal_id = *button
                .children_keys
                .get("NormalTexture")
                .expect("normal texture child should exist");
            let pushed_id = *button
                .children_keys
                .get("PushedTexture")
                .expect("pushed texture child should exist");
            (normal_id, pushed_id)
        };

        let size = Size::new(320.0, 240.0);
        app.screen_size.set(size);
        app.mark_all_strata_dirty();
        app.rebuild_dirty_strata(size, app.strata_dirty.get());
        rebuild_hittable_cache(&app, size);

        let click_pos = Point::new(30.0, 40.0);
        app.handle_mouse_down(click_pos);
        app.rebuild_dirty_strata(size, app.strata_dirty.get());
        assert!(
            snapshot_texture_alphas(&app, normal_id).is_empty(),
            "pressed button should remove the normal texture snapshot"
        );
        assert!(
            snapshot_texture_alphas(&app, pushed_id).contains(&1.0),
            "pressed button should emit the pushed texture snapshot"
        );

        app.handle_mouse_up(click_pos);
        app.rebuild_dirty_strata(size, app.strata_dirty.get());
        assert!(
            snapshot_texture_alphas(&app, normal_id).contains(&1.0),
            "mouse up should dirty and re-emit the normal texture snapshot"
        );
        assert!(
            snapshot_texture_alphas(&app, pushed_id).is_empty(),
            "mouse up should remove the pushed texture snapshot"
        );
    }

    #[test]
    fn mouse_leave_clears_pressed_button_texture_state() {
        let mut app = build_test_app();
        app.env
            .borrow()
            .exec(
                r#"
                PressedLeaveMicroButton = CreateFrame("Button", "PressedLeaveMicroButton", UIParent)
                PressedLeaveMicroButton:SetSize(32, 40)
                PressedLeaveMicroButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 20, -20)
                PressedLeaveMicroButton:EnableMouse(true)
                PressedLeaveMicroButton:SetNormalAtlas("UI-HUD-MicroMenu-SpecTalents-Up")
                PressedLeaveMicroButton:SetPushedAtlas("UI-HUD-MicroMenu-SpecTalents-Down")
            "#,
            )
            .expect("pressed leave micro button setup should succeed");

        let normal_id = {
            let env = app.env.borrow();
            let state = env.state().borrow();
            let button = state
                .widgets
                .get_by_name("PressedLeaveMicroButton")
                .expect("pressed leave micro button should exist");
            *button
                .children_keys
                .get("NormalTexture")
                .expect("normal texture child should exist")
        };

        let size = Size::new(320.0, 240.0);
        app.screen_size.set(size);
        app.mark_all_strata_dirty();
        app.rebuild_dirty_strata(size, app.strata_dirty.get());
        rebuild_hittable_cache(&app, size);

        app.handle_mouse_down(Point::new(30.0, 40.0));
        app.rebuild_dirty_strata(size, app.strata_dirty.get());
        app.handle_mouse_leave();
        app.rebuild_dirty_strata(size, app.strata_dirty.get());

        assert!(
            app.pressed_frame.is_none(),
            "mouse leave should clear the app pressed target"
        );
        assert!(
            snapshot_texture_alphas(&app, normal_id).contains(&1.0),
            "mouse leave should dirty and restore the normal texture snapshot"
        );
    }

    #[test]
    fn prune_irrelevant_dirty_strata_skips_cached_strata_without_bucket_or_snapshot_hits() {
        let registry = crate::widget::WidgetRegistry::new();
        let dirty_ids = FxHashSet::from_iter([99_u64]);
        let buckets = vec![vec![1_u64, 2_u64]];
        let cached = std::array::from_fn(|i| (i == 0).then(|| Arc::new(QuadBatch::new())));
        let snapshots = std::array::from_fn(|i| {
            (i == 0).then(|| HashMap::from([(1_u64, FrameQuadSnapshot::default())]))
        });

        let pruned = prune_irrelevant_dirty_strata(
            dirty_mask(0),
            Some(&dirty_ids),
            &registry,
            Some(&buckets),
            &cached,
            &snapshots,
        );

        assert_eq!(
            pruned, 0,
            "irrelevant dirty ids should not rebuild cached strata"
        );
    }

    #[test]
    fn prune_irrelevant_dirty_strata_keeps_strata_when_snapshot_must_be_removed() {
        let registry = crate::widget::WidgetRegistry::new();
        let dirty_ids = FxHashSet::from_iter([3_u64]);
        let buckets = vec![vec![1_u64, 2_u64]];
        let cached = std::array::from_fn(|i| (i == 0).then(|| Arc::new(QuadBatch::new())));
        let snapshots = std::array::from_fn(|i| {
            (i == 0).then(|| HashMap::from([(3_u64, FrameQuadSnapshot::default())]))
        });

        let pruned = prune_irrelevant_dirty_strata(
            dirty_mask(0),
            Some(&dirty_ids),
            &registry,
            Some(&buckets),
            &cached,
            &snapshots,
        );

        assert_eq!(
            pruned,
            dirty_mask(0),
            "dirty frames with cached snapshots still need a rebuild to clear old quads"
        );
    }

    #[test]
    fn prune_irrelevant_dirty_strata_keeps_strata_when_bucket_contains_dirty_frame() {
        let registry = crate::widget::WidgetRegistry::new();
        let dirty_ids = FxHashSet::from_iter([2_u64]);
        let buckets = vec![vec![1_u64, 2_u64]];
        let cached = std::array::from_fn(|i| (i == 0).then(|| Arc::new(QuadBatch::new())));
        let snapshots = std::array::from_fn(|i| {
            (i == 0).then(|| HashMap::from([(1_u64, FrameQuadSnapshot::default())]))
        });

        let pruned = prune_irrelevant_dirty_strata(
            dirty_mask(0),
            Some(&dirty_ids),
            &registry,
            Some(&buckets),
            &cached,
            &snapshots,
        );

        assert_eq!(
            pruned,
            dirty_mask(0),
            "dirty frames still present in the bucket must rebuild that strata"
        );
    }

    #[test]
    fn rebuild_dirty_strata_skips_irrelevant_cached_strata() {
        let app = build_test_app();
        app.cached_strata_quads.borrow_mut()[0] = Some(Arc::new(QuadBatch::new()));
        app.cached_frame_snapshots.borrow_mut()[0] =
            Some(HashMap::from([(1_u64, FrameQuadSnapshot::default())]));
        *app.pending_dirty_ids.borrow_mut() = Some(rustc_hash::FxHashSet::from_iter([99_u64]));
        app.env.borrow().state().borrow_mut().strata_buckets = Some(vec![vec![1_u64, 2_u64]]);

        let rebuilt = app.rebuild_dirty_strata(Size::new(64.0, 64.0), dirty_mask(0));

        assert_eq!(
            rebuilt, 0,
            "irrelevant dirty ids should short-circuit cached strata rebuilds"
        );
    }

    #[test]
    fn rebuild_dirty_strata_resets_consumed_full_rebuild_sentinel() {
        let app = build_test_app();
        app.pending_dirty_ids.borrow_mut().take();

        let _ = app.rebuild_dirty_strata(Size::new(64.0, 64.0), dirty_mask(0));

        let pending = app.pending_dirty_ids.borrow();
        assert!(
            pending.as_ref().is_some_and(FxHashSet::is_empty),
            "after consuming a full-rebuild sentinel, pending dirty IDs must reset to an empty concrete set"
        );
    }

    #[test]
    fn consumed_full_rebuild_sentinel_preserves_next_incremental_fast_path() {
        let app = build_test_app();
        app.cached_strata_quads.borrow_mut()[0] = Some(Arc::new(QuadBatch::new()));
        app.cached_frame_snapshots.borrow_mut()[0] =
            Some(HashMap::from([(1_u64, FrameQuadSnapshot::default())]));
        app.env.borrow().state().borrow_mut().strata_buckets = Some(vec![vec![1_u64, 2_u64]]);

        app.pending_dirty_ids.borrow_mut().take();
        let first = app.rebuild_dirty_strata(Size::new(64.0, 64.0), dirty_mask(0));
        assert_eq!(
            first,
            dirty_mask(0),
            "consuming a full-rebuild sentinel should allow one full rebuild pass"
        );

        app.merge_pending_dirty_ids(Some(FxHashSet::from_iter([99_u64])));
        let second = app.rebuild_dirty_strata(Size::new(64.0, 64.0), dirty_mask(0));
        assert_eq!(
            second, 0,
            "after the sentinel is consumed, unrelated dirty IDs must still prune cached strata rebuilds"
        );
    }

    include!("render_map_tests.rs");
}

#[cfg(test)]
#[path = "render/pending_texture_tests.rs"]
mod pending_texture_tests;
