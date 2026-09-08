#![cfg(feature = "gui")]

use crate::common;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use iced::{Point, Rectangle, Size};
use image::RgbaImage;
use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::headless::{render_batches_to_images, render_to_image};
use wow_ui_sim::render::shader::load_texture_or_crop;
use wow_ui_sim::render::{BlendMode, GlyphAtlas, QuadBatch, WowFontSystem};
use wow_ui_sim::texture::TextureManager;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn setup_full_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    let addons = discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if let Err(e) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events(&env);
    env.apply_post_event_workarounds();
    wow_ui_sim::startup::process_pending_timers(&env);
    wow_ui_sim::startup::fire_one_on_update_tick(&env);
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(&*env.rilua());
    env
}

fn open_class_talent_frame(env: &WowLuaEnv) {
    env.exec("PlayerSpellsUtil.ToggleClassTalentFrame()")
        .expect("Failed to open class talent frame");
}

fn make_texture_manager() -> TextureManager {
    TextureManager::new()
}

fn make_font_system() -> Rc<RefCell<WowFontSystem>> {
    Rc::new(RefCell::new(WowFontSystem::new()))
}

fn build_screenshot_like_batch(
    env: &WowLuaEnv,
    width: u32,
    height: u32,
    filter: Option<&str>,
) -> QuadBatch {
    let font_system = make_font_system();
    env.set_font_system(Rc::clone(&font_system));
    env.set_screen_size(width as f32, height as f32);
    wow_ui_sim::startup::run_extra_update_ticks(env, 3);

    let mut glyph_atlas = GlyphAtlas::new();
    let mut font_system = font_system.borrow_mut();
    let buckets = {
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut font_system);
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    let tooltip_data = wow_ui_sim::iced_app::tooltip::collect_tooltip_data(&state);
    build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(&state.widgets, (width as f32, height as f32), &buckets)
            .root_name(filter)
            .text_ctx(Some((&mut font_system, &mut glyph_atlas)))
            .message_frames(Some(&state.message_frames))
            .tooltip_data(Some(&tooltip_data)),
    )
}

fn sample_rect_pixel(image: &RgbaImage, rect: (f32, f32, f32, f32), u: f32, v: f32) -> [u8; 4] {
    let max_x = image.width().saturating_sub(1) as f32;
    let max_y = image.height().saturating_sub(1) as f32;
    let x = (rect.0 + rect.2 * u).round().clamp(0.0, max_x) as u32;
    let y = (rect.1 + rect.3 * v).round().clamp(0.0, max_y) as u32;
    image.get_pixel(x, y).0
}

fn assert_rgb_close(actual: [u8; 4], expected: [u8; 4], tolerance: u8, label: &str) {
    let tolerance = i16::from(tolerance);
    for channel in 0..3 {
        let delta = (i16::from(actual[channel]) - i16::from(expected[channel])).abs();
        assert!(
            delta <= tolerance,
            "{label} channel {channel} differs too much: actual={actual:?} expected={expected:?} tolerance={tolerance}"
        );
    }
}

fn max_rgb_channel_diff(lhs: [u8; 4], rhs: [u8; 4]) -> u8 {
    (0..3)
        .map(|channel| lhs[channel].abs_diff(rhs[channel]))
        .max()
        .unwrap_or(0)
}

fn diff_bounds(
    before: &RgbaImage,
    after: &RgbaImage,
    per_channel_tolerance: u8,
) -> Option<(u32, u32, u32, u32)> {
    let mut min_x = u32::MAX;
    let mut min_y = u32::MAX;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    for y in 0..before.height() {
        for x in 0..before.width() {
            let lhs = before.get_pixel(x, y).0;
            let rhs = after.get_pixel(x, y).0;
            let differs =
                (0..4).any(|channel| lhs[channel].abs_diff(rhs[channel]) > per_channel_tolerance);
            if differs {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
    }

    found.then_some((min_x, min_y, max_x, max_y))
}

fn request_contains_point(
    batch: &wow_ui_sim::render::QuadBatch,
    request: &wow_ui_sim::render::TextureRequest,
    x: f32,
    y: f32,
) -> bool {
    let bounds = quad_bounds(batch, request);
    x >= bounds.0 && x <= bounds.2 && y >= bounds.1 && y <= bounds.3
}

fn request_overlaps_rect(
    batch: &wow_ui_sim::render::QuadBatch,
    request: &wow_ui_sim::render::TextureRequest,
    rect: (f32, f32, f32, f32),
) -> bool {
    let bounds = quad_bounds(batch, request);
    let rect_right = rect.0 + rect.2;
    let rect_bottom = rect.1 + rect.3;
    let bounds_right = bounds.2;
    let bounds_bottom = bounds.3;
    bounds.0 < rect_right
        && bounds_right > rect.0
        && bounds.1 < rect_bottom
        && bounds_bottom > rect.1
}

fn vertex_range_bounds(
    batch: &wow_ui_sim::render::QuadBatch,
    vertex_start: usize,
    vertex_count: usize,
) -> (f32, f32, f32, f32) {
    quad_bounds_from_vertices(&batch.vertices[vertex_start..vertex_start + vertex_count])
}

fn bounds_overlap_rect(bounds: (f32, f32, f32, f32), rect: (f32, f32, f32, f32)) -> bool {
    let rect_right = rect.0 + rect.2;
    let rect_bottom = rect.1 + rect.3;
    bounds.0 < rect_right && bounds.2 > rect.0 && bounds.1 < rect_bottom && bounds.3 > rect.1
}

fn marble_only_batch(width: u32, height: u32) -> QuadBatch {
    let mut batch = QuadBatch::default();
    batch.push_tiled_path(
        Rectangle::new(Point::ORIGIN, Size::new(width as f32, height as f32)),
        256.0,
        256.0,
        "framegeneral/ui-background-marble",
        [0.55, 0.55, 0.55, 1.0],
    );
    batch
}

fn assert_images_match_rect(
    actual: &RgbaImage,
    expected: &RgbaImage,
    rect: (u32, u32, u32, u32),
    label: &str,
) {
    for y in rect.1..rect.1 + rect.3 {
        for x in rect.0..rect.0 + rect.2 {
            assert_eq!(
                actual.get_pixel(x, y).0,
                expected.get_pixel(x, y).0,
                "{label} pixel mismatch at ({x}, {y})"
            );
        }
    }
}

fn quad_bounds_from_vertices(verts: &[wow_ui_sim::render::QuadVertex]) -> (f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for vert in verts {
        min_x = min_x.min(vert.position[0]);
        min_y = min_y.min(vert.position[1]);
        max_x = max_x.max(vert.position[0]);
        max_y = max_y.max(vert.position[1]);
    }
    (min_x, min_y, max_x, max_y)
}

fn quad_bounds(
    batch: &wow_ui_sim::render::QuadBatch,
    request: &wow_ui_sim::render::TextureRequest,
) -> (f32, f32, f32, f32) {
    let start = request.vertex_start as usize;
    let end = start + request.vertex_count as usize;
    quad_bounds_from_vertices(&batch.vertices[start..end])
}

fn request_matches_rect(
    batch: &wow_ui_sim::render::QuadBatch,
    request: &wow_ui_sim::render::TextureRequest,
    rect: (f32, f32, f32, f32),
) -> bool {
    let bounds = quad_bounds(batch, request);
    let tolerance = 0.1;
    (bounds.0 - rect.0).abs() <= tolerance
        && (bounds.1 - rect.1).abs() <= tolerance
        && (bounds.2 - (rect.0 + rect.2)).abs() <= tolerance
        && (bounds.3 - (rect.1 + rect.3)).abs() <= tolerance
}

#[test]
fn hero_spec_icon_full_ui_render_matches_isolated_crop_render() {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping GPU sampling test: no adapter available");
        return;
    }

    let env = setup_full_ui();
    open_class_talent_frame(&env);

    let (icon_rect, icon_path) = {
        let state = env.state().borrow();
        let player_spells_id = state
            .widgets
            .get_id_by_name("PlayerSpellsFrame")
            .expect("PlayerSpellsFrame should exist");
        let talents_frame_id = *state
            .widgets
            .get(player_spells_id)
            .and_then(|frame| frame.children_keys.get("TalentsFrame"))
            .expect("TalentsFrame child should exist");
        let hero_container_id = *state
            .widgets
            .get(talents_frame_id)
            .and_then(|frame| frame.children_keys.get("HeroTalentsContainer"))
            .expect("HeroTalentsContainer child should exist");
        let button_id = *state
            .widgets
            .get(hero_container_id)
            .and_then(|frame| frame.children_keys.get("HeroSpecButton"))
            .expect("HeroSpecButton child should exist");
        let button = state.widgets.get(button_id).unwrap();
        let icon_id = *button.children_keys.get("Icon1").expect("Icon1 child");
        let icon_rect =
            wow_ui_sim::iced_app::compute_frame_rect(&state.widgets, icon_id, 1024.0, 768.0);
        let icon_path = state
            .widgets
            .get(icon_id)
            .and_then(|frame| frame.texture.clone())
            .expect("Icon1 should have a texture path");
        (icon_rect, icon_path)
    };

    let buckets = {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(RegistryQuadBatchParams::new(
        &state.widgets,
        (1024.0, 768.0),
        &buckets,
    ));

    let icon_crop_prefix = format!("{icon_path}@crop:");
    let icon_request = batch
        .texture_requests
        .iter()
        .find(|request| {
            request.path.starts_with(&icon_crop_prefix)
                && request_matches_rect(
                    &batch,
                    request,
                    (icon_rect.x, icon_rect.y, icon_rect.width, icon_rect.height),
                )
        })
        .expect("HeroSpecButton.Icon1 should emit a cropped atlas request");

    let overlapping_requests: Vec<_> = batch
        .texture_requests
        .iter()
        .filter(|request| {
            request_overlaps_rect(
                &batch,
                request,
                (icon_rect.x, icon_rect.y, icon_rect.width, icon_rect.height),
            )
        })
        .collect();

    let mask_request = batch
        .mask_texture_requests
        .iter()
        .find(|request| {
            request_matches_rect(
                &batch,
                request,
                (icon_rect.x, icon_rect.y, icon_rect.width, icon_rect.height),
            )
        })
        .map(|request| request.path.clone());

    let mut crop_mgr = make_texture_manager();
    let crop = load_texture_or_crop(&mut crop_mgr, &icon_request.path)
        .expect("cropped hero spec icon texture should load");
    let crop_image = RgbaImage::from_raw(crop.width, crop.height, crop.rgba.to_vec())
        .expect("cropped hero spec icon should decode into an image");

    let mut isolated_batch = QuadBatch::default();
    let mut icon_isolated_start = None;
    for request in &overlapping_requests {
        let request_bounds = quad_bounds(&batch, request);
        isolated_batch.push_textured_path(
            Rectangle::new(
                Point::new(request_bounds.0, request_bounds.1),
                Size::new(
                    request_bounds.2 - request_bounds.0,
                    request_bounds.3 - request_bounds.1,
                ),
            ),
            &request.path,
            [1.0, 1.0, 1.0, 1.0],
            BlendMode::Alpha,
        );
        let isolated_start = isolated_batch.vertices.len() - 4;
        let source_start = request.vertex_start as usize;
        for offset in 0..4 {
            isolated_batch.vertices[isolated_start + offset] =
                batch.vertices[source_start + offset];
        }
        if request.vertex_start == icon_request.vertex_start && request.path == icon_request.path {
            icon_isolated_start = Some(isolated_start as u32);
        }
    }
    if let (Some(mask_path), Some(icon_isolated_start)) = (mask_request, icon_isolated_start) {
        isolated_batch
            .mask_texture_requests
            .push(wow_ui_sim::render::TextureRequest::new(
                mask_path,
                icon_isolated_start,
                4,
            ));
    }

    let mut isolated_mgr = make_texture_manager();
    let isolated_render = render_to_image(&isolated_batch, &mut isolated_mgr, 1024, 768, None);
    let mut full_render_mgr = make_texture_manager();
    let full_render = render_to_image(&batch, &mut full_render_mgr, 1024, 768, None);
    let rendered_rect = (icon_rect.x, icon_rect.y, icon_rect.width, icon_rect.height);
    let crop_rect = (
        0.0,
        0.0,
        crop_image.width() as f32,
        crop_image.height() as f32,
    );

    for (u, v, label) in [
        (0.50, 0.35, "top-center"),
        (0.35, 0.50, "center-left"),
        (0.50, 0.50, "center"),
        (0.65, 0.50, "center-right"),
        (0.50, 0.65, "bottom-center"),
    ] {
        let source = sample_rect_pixel(&crop_image, crop_rect, u, v);
        assert!(
            source[3] >= 200,
            "{label} crop sample should be substantially opaque: sample={source:?}"
        );

        let isolated_pixel = sample_rect_pixel(&isolated_render, rendered_rect, u, v);
        let full_render_pixel = sample_rect_pixel(&full_render, rendered_rect, u, v);
        assert_rgb_close(
            full_render_pixel,
            isolated_pixel,
            12,
            &format!("HeroSpecButton.Icon1 full render sample {label}"),
        );
    }
}

#[test]
fn hero_spec_icon_mask_clips_corners_but_preserves_center_pixels() {
    assert_mask_preserves_icon_pixels(r"Interface\\Masks\\CircleMask");
}

#[test]
fn active_spellbook_mask_preserves_black_rgb_opaque_alpha_center() {
    assert_mask_preserves_icon_pixels(r"Interface\\Spellbook\\SpellBookElementsIconMask");
}

#[test]
fn passive_spellbook_mask_preserves_white_rgb_opaque_alpha_center() {
    assert_mask_preserves_icon_pixels(r"Interface\\TalentFrame\\TalentsMaskNodeCircle");
}

fn assert_mask_preserves_icon_pixels(mask_path: &str) {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping GPU masking test: no adapter available");
        return;
    }

    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(128.0, 128.0);
    env.exec(&format!(
        r#"
        local frame = CreateFrame("Frame", "MaskPixelHarness", UIParent)
        frame:SetSize(64, 64)
        frame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 16, -16)

        local icon = frame:CreateTexture("MaskPixelHarnessIcon", "ARTWORK")
        icon:SetAllPoints()
        icon:SetColorTexture(1, 0, 0, 1)

        local mask = frame:CreateMaskTexture("MaskPixelHarnessMask", "ARTWORK")
        mask:SetAllPoints()
        mask:SetTexture("{mask_path}")

        icon:AddMaskTexture(mask)
    "#,
    ))
    .expect("failed to build mask pixel harness");

    let icon_id = {
        let state = env.state().borrow();
        state
            .widgets
            .get_id_by_name("MaskPixelHarnessIcon")
            .expect("MaskPixelHarnessIcon should exist")
    };
    let icon_rect = {
        let state = env.state().borrow();
        wow_ui_sim::iced_app::compute_frame_rect(&state.widgets, icon_id, 128.0, 128.0)
    };
    let rendered_rect = (icon_rect.x, icon_rect.y, icon_rect.width, icon_rect.height);

    let mut masked_mgr = make_texture_manager();
    let masked_batch = build_screenshot_like_batch(&env, 128, 128, None);
    let masked_render = render_to_image(&masked_batch, &mut masked_mgr, 128, 128, None);

    {
        let mut state = env.state().borrow_mut();
        let icon = state
            .widgets
            .get_mut(icon_id)
            .expect("MaskPixelHarnessIcon should exist");
        assert!(
            !icon.mask_textures.is_empty(),
            "MaskPixelHarnessIcon should start with a mask"
        );
        icon.mask_textures.clear();
    }

    let mut unmasked_mgr = make_texture_manager();
    let unmasked_batch = build_screenshot_like_batch(&env, 128, 128, None);
    let unmasked_render = render_to_image(&unmasked_batch, &mut unmasked_mgr, 128, 128, None);

    let masked_center = sample_rect_pixel(&masked_render, rendered_rect, 0.50, 0.50);
    let unmasked_center = sample_rect_pixel(&unmasked_render, rendered_rect, 0.50, 0.50);
    let expected_red = [255, 0, 0, 255];
    let expected_background = masked_render.get_pixel(4, 4).0;

    assert_rgb_close(
        masked_center,
        expected_red,
        16,
        "masked center should stay red",
    );
    assert!(
        max_rgb_channel_diff(unmasked_center, expected_red) <= 16,
        "unmasked center should also stay red: unmasked={unmasked_center:?}"
    );

    for (u, v, label) in [
        (0.03, 0.03, "top-left"),
        (0.97, 0.03, "top-right"),
        (0.97, 0.97, "bottom-right"),
        (0.03, 0.97, "bottom-left"),
    ] {
        let masked_corner = sample_rect_pixel(&masked_render, rendered_rect, u, v);
        let unmasked_corner = sample_rect_pixel(&unmasked_render, rendered_rect, u, v);

        assert_rgb_close(
            masked_corner,
            expected_background,
            10,
            &format!("masked {label} corner should reveal the cleared background"),
        );
        assert!(
            max_rgb_channel_diff(unmasked_corner, expected_red) <= 16,
            "unmasked {label} corner should stay red without the mask: unmasked={unmasked_corner:?}"
        );
    }
}

#[test]
fn hiding_hero_talents_container_only_changes_top_center_region() {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping GPU diff test: no adapter available");
        return;
    }

    let env = setup_full_ui();
    open_class_talent_frame(&env);
    env.set_screen_size(1600.0, 1200.0);

    let (hero_container_id, hero_rect) = {
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let player_spells_id = state
            .widgets
            .get_id_by_name("PlayerSpellsFrame")
            .expect("PlayerSpellsFrame should exist");
        let talents_frame_id = *state
            .widgets
            .get(player_spells_id)
            .and_then(|frame| frame.children_keys.get("TalentsFrame"))
            .expect("TalentsFrame child should exist");
        let hero_container_id = *state
            .widgets
            .get(talents_frame_id)
            .and_then(|frame| frame.children_keys.get("HeroTalentsContainer"))
            .expect("HeroTalentsContainer child should exist");
        let hero_rect = wow_ui_sim::iced_app::compute_frame_rect(
            &state.widgets,
            hero_container_id,
            1600.0,
            1200.0,
        );
        (hero_container_id, hero_rect)
    };

    let visible_batch = {
        let buckets = {
            let mut state = env.state().borrow_mut();
            let _ = state.get_strata_buckets();
            state.strata_buckets.as_ref().unwrap().clone()
        };
        let state = env.state().borrow();
        build_quad_batch_for_registry(RegistryQuadBatchParams::new(
            &state.widgets,
            (1600.0, 1200.0),
            &buckets,
        ))
    };

    {
        let mut state = env.state().borrow_mut();
        state.set_frame_visible(hero_container_id, false);
        state.ensure_layout_rects();
    }

    let hidden_batch = {
        let buckets = {
            let mut state = env.state().borrow_mut();
            let _ = state.get_strata_buckets();
            state.strata_buckets.as_ref().unwrap().clone()
        };
        let state = env.state().borrow();
        build_quad_batch_for_registry(RegistryQuadBatchParams::new(
            &state.widgets,
            (1600.0, 1200.0),
            &buckets,
        ))
    };

    let mut texture_manager = make_texture_manager();
    let mut renders = render_batches_to_images(
        &[&visible_batch, &hidden_batch],
        &mut texture_manager,
        1600,
        1200,
        None,
    )
    .into_iter();
    let visible_render = renders.next().expect("visible render");
    let hidden_render = renders.next().expect("hidden render");
    assert!(renders.next().is_none(), "expected exactly two renders");

    let diff = diff_bounds(&visible_render, &hidden_render, 12)
        .expect("hiding HeroTalentsContainer should change rendered pixels");
    let expanded_left = (hero_rect.x - 160.0).max(0.0);
    let expanded_top = (hero_rect.y - 160.0).max(0.0);
    let expanded_right = hero_rect.x + hero_rect.width + 160.0;
    let expanded_bottom = hero_rect.y + hero_rect.height + 160.0;

    assert!(
        diff.0 as f32 >= expanded_left
            && diff.1 as f32 >= expanded_top
            && diff.2 as f32 <= expanded_right
            && diff.3 as f32 <= expanded_bottom,
        "Hiding HeroTalentsContainer should only affect the hero panel region: diff={diff:?} hero_rect={hero_rect:?}"
    );
}
