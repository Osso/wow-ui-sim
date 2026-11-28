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
        wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut state, &mut font_system);
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

post_wrath_ui_fixture! {
fn hero_icon_geometry(env: &WowLuaEnv) -> ((f32, f32, f32, f32), String) {
    let state = env.state().borrow();
    let registry = &state.widgets;
    let mut id = registry.get_id_by_name("PlayerSpellsFrame").unwrap();
    for key in [
        "TalentsFrame",
        "HeroTalentsContainer",
        "HeroSpecButton",
        "Icon1",
    ] {
        id = registry.get(id).unwrap().children_keys[key];
    }
    let rect = wow_ui_sim::iced_app::compute_frame_rect(registry, id, 1024.0, 768.0);
    let path = registry
        .get(id)
        .unwrap()
        .texture
        .clone()
        .expect("Icon1 source");
    ((rect.x, rect.y, rect.width, rect.height), path)
}

fn build_hero_render_batch(env: &WowLuaEnv) -> QuadBatch {
    let buckets = {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    build_quad_batch_for_registry(RegistryQuadBatchParams::new(
        &state.widgets,
        (1024.0, 768.0),
        &buckets,
    ))
}

fn hero_crop_request<'a>(
    batch: &'a QuadBatch,
    rect: (f32, f32, f32, f32),
    icon_path: &str,
) -> &'a wow_ui_sim::render::TextureRequest {
    let prefix = format!("{icon_path}@crop:");
    batch
        .texture_requests
        .iter()
        .find(|request| {
            request.path.starts_with(&prefix) && request_matches_rect(batch, request, rect)
        })
        .expect("HeroSpecButton.Icon1 should emit a cropped atlas request")
}

fn load_hero_crop(request: &wow_ui_sim::render::TextureRequest) -> RgbaImage {
    let mut manager = make_texture_manager();
    let crop = load_texture_or_crop(&mut manager, &request.path)
        .expect("cropped hero spec icon texture should load");
    RgbaImage::from_raw(crop.width, crop.height, crop.rgba.to_vec())
        .expect("cropped hero spec icon should decode into an image")
}

// A request can span several quads, and the overlap can contain solid triangles
// and multiple masked textures. Select draw triangles rather than first quads
// of requests, then preserve their exact draw order and all request ranges.
fn overlapping_triangles(batch: &QuadBatch, rect: (f32, f32, f32, f32)) -> Vec<[u32; 3]> {
    batch
        .indices
        .chunks_exact(3)
        .filter_map(|indices| {
            let triangle = [indices[0], indices[1], indices[2]];
            let vertices = triangle.map(|index| batch.vertices[index as usize]);
            bounds_overlap_rect(quad_bounds_from_vertices(&vertices), rect).then_some(triangle)
        })
        .collect()
}

fn compact_scene_vertices(
    batch: &QuadBatch,
    triangles: &[[u32; 3]],
) -> (Vec<wow_ui_sim::render::QuadVertex>, Vec<Option<u32>>) {
    let mut selected = vec![false; batch.vertices.len()];
    for index in triangles.iter().flatten() {
        selected[*index as usize] = true;
    }
    let mut vertices = Vec::new();
    let mut remap = Vec::with_capacity(batch.vertices.len());
    for (vertex, keep) in batch.vertices.iter().zip(selected) {
        if !keep {
            remap.push(None);
            continue;
        }
        remap.push(Some(vertices.len() as u32));
        vertices.push(*vertex);
    }
    (vertices, remap)
}

fn remap_scene_requests(
    requests: &[wow_ui_sim::render::TextureRequest],
    remap: &[Option<u32>],
) -> Vec<wow_ui_sim::render::TextureRequest> {
    requests
        .iter()
        .filter_map(|request| {
            let start = request.vertex_start as usize;
            let end = start + request.vertex_count as usize;
            let mut selected = remap[start..end].iter().flatten().copied();
            let new_start = selected.next()?;
            let vertex_count = 1 + selected.count() as u32;
            Some(wow_ui_sim::render::TextureRequest::new_with_uv_inset(
                request.path.clone(),
                new_start,
                vertex_count,
                request.use_uv_inset,
            ))
        })
        .collect()
}

fn isolate_scene_at_rect(batch: &QuadBatch, rect: (f32, f32, f32, f32)) -> QuadBatch {
    let triangles = overlapping_triangles(batch, rect);
    let (vertices, remap) = compact_scene_vertices(batch, &triangles);
    let indices = triangles
        .iter()
        .flatten()
        .map(|index| remap[*index as usize].unwrap())
        .collect();
    QuadBatch {
        vertices,
        indices,
        texture_requests: remap_scene_requests(&batch.texture_requests, &remap),
        mask_texture_requests: remap_scene_requests(&batch.mask_texture_requests, &remap),
    }
}

fn assert_isolated_crop_and_mask(
    batch: &QuadBatch,
    isolated: &QuadBatch,
    rect: (f32, f32, f32, f32),
    icon_path: &str,
) {
    assert!(
        isolated.indices.len() < batch.indices.len(),
        "isolated draw must be smaller than the full UI"
    );
    assert!(
        isolated.texture_requests.len() < batch.texture_requests.len(),
        "isolated atlas must omit unrelated textures"
    );
    let full_icon = hero_crop_request(batch, rect, icon_path);
    let isolated_icon = hero_crop_request(isolated, rect, icon_path);
    assert_eq!(full_icon.path, isolated_icon.path);
    assert_eq!(full_icon.vertex_count, isolated_icon.vertex_count);
    let full_mask = mask_covering_request(batch, full_icon);
    let isolated_mask = mask_covering_request(isolated, isolated_icon);
    assert_eq!(
        full_mask.path, isolated_mask.path,
        "preserve the real icon mask"
    );
}

fn mask_covering_request<'a>(
    batch: &'a QuadBatch,
    icon: &wow_ui_sim::render::TextureRequest,
) -> &'a wow_ui_sim::render::TextureRequest {
    batch
        .mask_texture_requests
        .iter()
        .find(|mask| {
            mask.vertex_start <= icon.vertex_start
                && mask.vertex_start + mask.vertex_count >= icon.vertex_start + icon.vertex_count
        })
        .expect("the actual hero icon must have a mask request")
}

fn report_overlap_layers(batch: &QuadBatch, rect: (f32, f32, f32, f32)) {
    let multi_quad_paths: Vec<_> = batch
        .texture_requests
        .iter()
        .filter(|request| request.vertex_count > 4 && request_overlaps_rect(batch, request, rect))
        .map(|request| (&request.path, request.vertex_count))
        .collect();
    let top_center = (rect.0 + 0.50 * rect.2, rect.1 + 0.35 * rect.3, 1.0, 1.0);
    let solid_colors: Vec<_> = overlapping_triangles(batch, top_center)
        .iter()
        .filter_map(|triangle| {
            let vertex = batch.vertices[triangle[0] as usize];
            (vertex.tex_index == -1).then_some((triangle[0], vertex.color))
        })
        .collect();
    eprintln!(
        "Hero overlap: top-center solid vertices={solid_colors:?}, multi-quad texture ranges={multi_quad_paths:?}"
    );
}

const HERO_SAMPLE_POINTS: [(f32, f32, &str); 5] = [
    (0.50, 0.35, "top-center"),
    (0.35, 0.50, "center-left"),
    (0.50, 0.50, "center"),
    (0.65, 0.50, "center-right"),
    (0.50, 0.65, "bottom-center"),
];

fn assert_hero_render_samples(
    crop_image: &RgbaImage,
    isolated_render: &RgbaImage,
    full_render: &RgbaImage,
    rendered_rect: (f32, f32, f32, f32),
) {
    let crop_rect = (
        0.0,
        0.0,
        crop_image.width() as f32,
        crop_image.height() as f32,
    );
    for (u, v, label) in HERO_SAMPLE_POINTS {
        let source = sample_rect_pixel(crop_image, crop_rect, u, v);
        assert!(
            source[3] >= 200,
            "{label} crop sample should be substantially opaque: sample={source:?}"
        );
        let isolated_pixel = sample_rect_pixel(isolated_render, rendered_rect, u, v);
        let full_render_pixel = sample_rect_pixel(full_render, rendered_rect, u, v);
        eprintln!("{label}: full={full_render_pixel:?} isolated={isolated_pixel:?}");
        assert_rgb_close(
            full_render_pixel,
            isolated_pixel,
            12,
            &format!("HeroSpecButton.Icon1 full render sample {label}"),
        );
    }
}

#[test]
fn hero_spec_icon_full_ui_render_matches_isolated_crop_render() {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping GPU sampling test: no adapter available");
        return;
    }
    let env = setup_full_ui();
    open_class_talent_frame(&env);
    let (rect, icon_path) = hero_icon_geometry(&env);
    let batch = build_hero_render_batch(&env);
    let crop_image = load_hero_crop(hero_crop_request(&batch, rect, &icon_path));
    let isolated_batch = isolate_scene_at_rect(&batch, rect);
    assert_isolated_crop_and_mask(&batch, &isolated_batch, rect, &icon_path);
    report_overlap_layers(&batch, rect);
    let isolated_render = render_to_image(
        &isolated_batch,
        &mut make_texture_manager(),
        1024,
        768,
        None,
    );
    let full_render = render_to_image(&batch, &mut make_texture_manager(), 1024, 768, None);
    assert_hero_render_samples(&crop_image, &isolated_render, &full_render, rect);
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

post_wrath_ui_fixture! {
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
}
