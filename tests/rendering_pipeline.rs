#![cfg(feature = "gui")]
//! Rendering pipeline tests: from XML templates through GPU upload.
//!
//! Uses scroll bar buttons as the test subject, testing 5 layers:
//! 1. TextureManager loads scroll bar textures
//! 2. Texture path resolution (backslash, case-insensitive, atlas DB)
//! 3. Layout verification (positions from FauxScrollFrameTemplate)
//! 4. Quad batch generation
//! 5. GPU atlas upload

use crate::common;

use common::env_with_shared_xml;
use wow_ui_sim::atlas::{ATLAS_DB, get_atlas_info};
use wow_ui_sim::iced_app::{
    RegistryQuadBatchParams, build_quad_batch_for_registry, compute_frame_rect,
};
use wow_ui_sim::render::{GpuTextureAtlas, QuadBatch};
use wow_ui_sim::texture::TextureManager;

// ============================================================================
// Helpers
// ============================================================================

fn make_texture_manager() -> TextureManager {
    TextureManager::new()
}

/// Build strata buckets from a WowLuaEnv (mutable borrow), then return a clone.
fn build_strata_buckets(env: &wow_ui_sim::lua_api::WowLuaEnv) -> Vec<Vec<u64>> {
    let mut state = env.state().borrow_mut();
    let _ = state.get_strata_buckets();
    state.strata_buckets.as_ref().unwrap().clone()
}

/// Scroll bar texture paths used by the classic FauxScrollFrameTemplate.
const SCROLL_UP_BUTTON: &str = "Interface/Buttons/UI-ScrollBar-ScrollUpButton-Up";
const SCROLL_DOWN_BUTTON: &str = "Interface/Buttons/UI-ScrollBar-ScrollDownButton-Up";
const SCROLL_KNOB: &str = "Interface/Buttons/UI-ScrollBar-Knob";

/// Atlas-based scroll bar texture (MinimalScrollBar).
const MINIMAL_SCROLLBAR_ATLAS: &str = "Interface/Buttons/ScrollBarProportional";

const GPU_ATLAS_MEDIUM_TEXTURE_SIZE: u32 = 100;
const GPU_ATLAS_LARGE_TEXTURE_SIZE: u32 = 200;
const GPU_ATLAS_XL_TEXTURE_SIZE: u32 = 400;
const GPU_ATLAS_TIER_2_CELL_SIZE: u32 = 256;

// ============================================================================
// Layer 1: TextureManager loads scroll bar textures
// ============================================================================

#[test]
fn layer1_load_scroll_up_button_texture() {
    let mut mgr = make_texture_manager();

    let data = mgr.load(SCROLL_UP_BUTTON);
    assert!(
        data.is_some(),
        "Should load scroll up button texture: {}",
        SCROLL_UP_BUTTON
    );
    let data = data.unwrap();
    assert!(
        data.width > 0 && data.height > 0,
        "Texture dimensions should be positive"
    );
    assert_eq!(
        data.pixels.len(),
        (data.width * data.height * 4) as usize,
        "Pixel data should be RGBA"
    );
}

#[test]
fn layer1_load_scroll_down_button_texture() {
    let mut mgr = make_texture_manager();

    let data = mgr.load(SCROLL_DOWN_BUTTON);
    assert!(
        data.is_some(),
        "Should load scroll down button texture: {}",
        SCROLL_DOWN_BUTTON
    );
    let data = data.unwrap();
    assert!(data.width > 0 && data.height > 0);
    assert_eq!(data.pixels.len(), (data.width * data.height * 4) as usize);
}

#[test]
fn layer1_load_scroll_knob_texture() {
    let mut mgr = make_texture_manager();

    let data = mgr.load(SCROLL_KNOB);
    assert!(
        data.is_some(),
        "Should load scroll knob texture: {}",
        SCROLL_KNOB
    );
    let data = data.unwrap();
    assert!(data.width > 0 && data.height > 0);
    assert_eq!(data.pixels.len(), (data.width * data.height * 4) as usize);
}

#[test]
fn layer1_load_minimal_scrollbar_atlas_texture() {
    let mut mgr = make_texture_manager();

    let data = mgr.load(MINIMAL_SCROLLBAR_ATLAS);
    assert!(
        data.is_some(),
        "Should load MinimalScrollBar atlas texture: {}",
        MINIMAL_SCROLLBAR_ATLAS,
    );
    let data = data.unwrap();
    assert!(data.width > 0 && data.height > 0);
    assert_eq!(data.pixels.len(), (data.width * data.height * 4) as usize);
}

// ============================================================================
// Layer 2: Texture path resolution
// ============================================================================

#[test]
fn layer2_backslash_resolves_same_as_forward_slash() {
    let mut mgr = make_texture_manager();

    let forward = mgr.load("Interface/Buttons/UI-ScrollBar-ScrollUpButton-Up");
    assert!(forward.is_some(), "Forward slash path should load");
    let fwd_size = (forward.unwrap().width, forward.unwrap().height);

    let backslash = mgr.load("Interface\\Buttons\\UI-ScrollBar-ScrollUpButton-Up");
    assert!(backslash.is_some(), "Backslash path should load");
    let bk_size = (backslash.unwrap().width, backslash.unwrap().height);

    assert_eq!(
        fwd_size, bk_size,
        "Same texture loaded regardless of slash direction"
    );
}

#[test]
fn layer2_case_insensitive_resolution() {
    let mut mgr = make_texture_manager();

    // The actual files use mixed case; try all-lowercase
    let result = mgr.load("interface/buttons/ui-scrollbar-scrollupbutton-up");
    assert!(result.is_some(), "Case-insensitive path should resolve");
}

#[test]
fn layer2_atlas_db_returns_scroll_bar_entries() {
    // Check that the atlas DB has entries for the proportional scroll bar textures
    let up = get_atlas_info("ui-scrollbar-scrollupbutton-up");
    assert!(up.is_some(), "Atlas DB should have scroll up button entry");
    let up = up.unwrap();
    assert!(
        up.width() > 0 && up.height() > 0,
        "Atlas entry should have dimensions"
    );
    assert!(
        up.info.file.to_lowercase().contains("scrollbar"),
        "Atlas file path should reference scrollbar: {}",
        up.info.file,
    );

    let down = get_atlas_info("ui-scrollbar-scrolldownbutton-up");
    assert!(
        down.is_some(),
        "Atlas DB should have scroll down button entry"
    );

    let center = get_atlas_info("ui-scrollbar-center");
    // This one has a ! prefix in the atlas DB
    // get_atlas_info should handle it
    if center.is_none() {
        // Try with the ! prefix directly via ATLAS_DB
        let alt = ATLAS_DB.get("!ui-scrollbar-center");
        assert!(
            alt.is_some(),
            "Atlas DB should have scroll bar center entry (with ! prefix)"
        );
    }
}

// ============================================================================
// Layer 3: Layout verification
// ============================================================================

/// Find scrollbar child IDs from a FauxScrollFrameTemplate.
/// Returns (scrollbar_id, up_button_id, down_button_id).
fn find_scrollbar_children(
    registry: &wow_ui_sim::widget::WidgetRegistry,
    sf_name: &str,
) -> (u64, u64, u64) {
    let child_id = |suffix| {
        let name = common::template_child_name(sf_name, suffix);
        registry
            .get_id_by_name(&name)
            .unwrap_or_else(|| panic!("native Faux template should create {name}"))
    };
    (
        child_id("ScrollBar"),
        child_id("ScrollBarScrollUpButton"),
        child_id("ScrollBarScrollDownButton"),
    )
}

#[test]
fn layer3_scrollbar_layout_positions() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestSF", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(300, 400)
        sf:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let registry = &state.widgets;
    let screen_w = 1024.0;
    let screen_h = 768.0;

    let sf_id = registry.get_id_by_name("TestSF").unwrap();
    let (scrollbar_id, up_id, down_id) = find_scrollbar_children(registry, "TestSF");

    let sf_rect = compute_frame_rect(registry, sf_id, screen_w, screen_h);
    let sb_rect = compute_frame_rect(registry, scrollbar_id, screen_w, screen_h);
    let up_rect = compute_frame_rect(registry, up_id, screen_w, screen_h);
    let down_rect = compute_frame_rect(registry, down_id, screen_w, screen_h);

    assert!(
        sf_rect.width > 0.0 && sf_rect.height > 0.0,
        "ScrollFrame should have positive size: {:?}",
        sf_rect
    );
    assert!(
        sb_rect.width > 0.0 && sb_rect.height > 0.0,
        "ScrollBar should have positive size: {:?}",
        sb_rect
    );

    assert!(
        up_rect.width > 0.0 && up_rect.height > 0.0,
        "ScrollUpButton should have positive size: {:?}",
        up_rect
    );
    assert!(
        up_rect.y <= sb_rect.y + sb_rect.height * 0.3,
        "ScrollUpButton (y={}) should be near top of ScrollBar (y={}, h={})",
        up_rect.y,
        sb_rect.y,
        sb_rect.height,
    );

    assert!(
        down_rect.width > 0.0 && down_rect.height > 0.0,
        "ScrollDownButton should have positive size: {:?}",
        down_rect
    );
    assert!(
        down_rect.y + down_rect.height >= sb_rect.y + sb_rect.height * 0.7,
        "ScrollDownButton bottom (y+h={}) should be near bottom of ScrollBar (y+h={})",
        down_rect.y + down_rect.height,
        sb_rect.y + sb_rect.height,
    );
}

// ============================================================================
// Layer 4: Quad batch generation
// ============================================================================

#[test]
fn layer4_quad_batch_has_quads_for_scroll_widgets() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestSFQuads", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(300, 400)
        sf:SetPoint("CENTER")
        sf:Show()
    "#,
    )
    .unwrap();

    let buckets = build_strata_buckets(&env);
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(&state.widgets, (1024.0, 768.0), &buckets)
            .root_name(Some("TestSFQuads")),
    );

    // Should have at least the background quad + some widget quads
    assert!(
        batch.quad_count() > 1,
        "Batch should have multiple quads, got {}",
        batch.quad_count(),
    );

    // Texture requests should include scroll bar texture paths (if textures are set)
    // The template may set textures via Lua, which creates texture requests
    if !batch.texture_requests.is_empty() {
        let paths: Vec<&str> = batch
            .texture_requests
            .iter()
            .map(|r| r.path.as_str())
            .collect();
        eprintln!("Texture requests in batch: {:?}", paths);
    }
}

#[test]
fn layer4_quad_batch_direct_push() {
    use iced::{Point, Rectangle, Size};
    use wow_ui_sim::render::BlendMode;

    let mut batch = QuadBatch::new();

    // Push a textured quad with known bounds
    let bounds = Rectangle::new(Point::new(100.0, 200.0), Size::new(50.0, 30.0));
    batch.push_textured_path(
        bounds,
        "Interface/Buttons/UI-ScrollBar-ScrollUpButton-Up",
        [1.0, 1.0, 1.0, 1.0],
        BlendMode::Alpha,
    );

    assert_eq!(batch.quad_count(), 1, "Should have exactly 1 quad");
    assert_eq!(batch.vertices.len(), 4, "Quad should have 4 vertices");
    assert_eq!(batch.indices.len(), 6, "Quad should have 6 indices");
    assert_eq!(
        batch.texture_requests.len(),
        1,
        "Should have 1 texture request"
    );

    // Verify vertex positions match the bounds
    let v0 = &batch.vertices[0]; // top-left
    assert!(
        (v0.position[0] - 100.0).abs() < 0.01,
        "TL x={}",
        v0.position[0]
    );
    assert!(
        (v0.position[1] - 200.0).abs() < 0.01,
        "TL y={}",
        v0.position[1]
    );

    let v2 = &batch.vertices[2]; // bottom-right
    assert!(
        (v2.position[0] - 150.0).abs() < 0.01,
        "BR x={}",
        v2.position[0]
    );
    assert!(
        (v2.position[1] - 230.0).abs() < 0.01,
        "BR y={}",
        v2.position[1]
    );

    // Verify texture request path
    assert_eq!(
        batch.texture_requests[0].path,
        "Interface/Buttons/UI-ScrollBar-ScrollUpButton-Up",
    );
    assert_eq!(batch.texture_requests[0].vertex_start, 0);
    assert_eq!(batch.texture_requests[0].vertex_count, 4);
}

#[test]
fn layer4_texture_widget_with_file_data_id_emits_texture_request() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local tex = UIParent:CreateTexture("RenderFileDataTexture")
        tex:SetSize(32, 32)
        tex:SetPoint("CENTER")
        tex:SetTexture(136243)
        tex:Show()
    "#,
    )
    .unwrap();

    let expected_path = format!(
        "Interface\\{}",
        wow_ui_sim::manifest_interface_data::get_texture_path(136243)
            .expect("default icon should exist in manifest")
            .replace('/', "\\")
    );

    let buckets = build_strata_buckets(&env);
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(&state.widgets, (1024.0, 768.0), &buckets)
            .root_name(Some("RenderFileDataTexture")),
    );

    assert!(
        batch
            .texture_requests
            .iter()
            .any(|request| request.path == expected_path),
        "file-data-id texture should emit a texture request for {expected_path}, got {:?}",
        batch
            .texture_requests
            .iter()
            .map(|request| request.path.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn layer4_quad_batch_vertex_positions_match_layout() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestSFLayout", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(300, 400)
        sf:SetPoint("CENTER")
        sf:Show()
    "#,
    )
    .unwrap();

    let buckets = build_strata_buckets(&env);
    let state = env.state().borrow();
    let registry = &state.widgets;

    let screen_w = 1024.0;
    let screen_h = 768.0;

    // Get layout position of the scroll frame
    let sf_id = registry.get_id_by_name("TestSFLayout").unwrap();
    let sf_rect = compute_frame_rect(registry, sf_id, screen_w, screen_h);

    let batch = build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(registry, (screen_w, screen_h), &buckets)
            .root_name(Some("TestSFLayout")),
    );

    // The first quad is the background (full screen). Skip it.
    // Remaining quads should have positions within or near the scroll frame area.
    // UI_SCALE is 1.0, so layout coords == screen coords.
    let found_in_range = find_vertex_near_rect(&batch, &sf_rect);

    assert!(
        found_in_range || batch.quad_count() <= 1,
        "At least one widget quad should be near the scroll frame area",
    );
}

#[test]
fn layer5_additive_quads_preserve_destination_color_in_overlap() {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping GPU additive blend test: no adapter available");
        return;
    }

    use iced::{Point, Rectangle, Size};
    use wow_ui_sim::render::BlendMode;
    use wow_ui_sim::render::headless::render_to_image;

    let mut batch = QuadBatch::new();
    batch.push_quad(
        Rectangle::new(Point::new(0.0, 0.0), Size::new(64.0, 64.0)),
        Rectangle::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)),
        [0.0, 0.0, 0.0, 1.0],
        -1,
        BlendMode::Alpha,
    );
    batch.push_quad(
        Rectangle::new(Point::new(8.0, 8.0), Size::new(40.0, 40.0)),
        Rectangle::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)),
        [1.0, 0.0, 0.0, 1.0],
        -1,
        BlendMode::Alpha,
    );
    batch.push_quad(
        Rectangle::new(Point::new(24.0, 8.0), Size::new(24.0, 24.0)),
        Rectangle::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)),
        [0.0, 1.0, 0.0, 0.5],
        -1,
        BlendMode::Additive,
    );

    let mut tex_mgr = make_texture_manager();
    let image = render_to_image(&batch, &mut tex_mgr, 64, 64, None);
    let red_only = image.get_pixel(16, 16).0;
    let overlap = image.get_pixel(32, 16).0;

    assert!(
        overlap[0].abs_diff(red_only[0]) <= 2,
        "additive overlap should preserve destination red channel: red_only={red_only:?} overlap={overlap:?}"
    );
    assert!(
        overlap[1] > red_only[1],
        "additive overlap should add green light on top of the destination: red_only={red_only:?} overlap={overlap:?}"
    );
}

/// Check if any quad vertex (after the first background quad) is near the given rect.
fn find_vertex_near_rect(batch: &QuadBatch, rect: &wow_ui_sim::LayoutRect) -> bool {
    for quad_idx in 1..batch.quad_count() {
        let vi = quad_idx * 4;
        if vi >= batch.vertices.len() {
            break;
        }
        let vx = batch.vertices[vi].position[0];
        let vy = batch.vertices[vi].position[1];
        if vx >= rect.x - 50.0
            && vx <= rect.x + rect.width + 50.0
            && vy >= rect.y - 50.0
            && vy <= rect.y + rect.height + 50.0
        {
            return true;
        }
    }
    false
}

// ============================================================================
// Layer 5: GPU atlas upload
// ============================================================================

/// Exercise both GPU entry points together, including creation and teardown.
#[test]
fn layer5_parallel_gpu_lifetimes_complete() {
    let start = std::sync::Barrier::new(4);
    std::thread::scope(|scope| {
        for worker in 0..4 {
            let start = &start;
            scope.spawn(move || {
                start.wait();
                for _ in 0..8 {
                    if worker % 2 == 0 {
                        layer5_gpu_atlas_upload_and_lookup();
                    } else {
                        layer5_additive_quads_preserve_destination_color_in_overlap();
                    }
                }
            });
        }
    });
}

#[test]
fn layer5_gpu_atlas_upload_and_lookup() {
    let Some(gpu) = common::try_create_gpu_device() else {
        eprintln!("Skipping GPU test: no adapter available");
        return;
    };
    let (device, queue) = (&gpu.device, &gpu.queue);

    let mut tex_mgr = make_texture_manager();

    let mut atlas = GpuTextureAtlas::new(&device);
    assert!(atlas.is_empty(), "Atlas should start empty");

    // Load and upload a scroll bar texture
    let tex_data = tex_mgr.load(SCROLL_UP_BUTTON);
    assert!(tex_data.is_some(), "Should load scroll up button texture");
    let tex_data = tex_data.unwrap();

    let entry = atlas.upload(
        &queue,
        SCROLL_UP_BUTTON,
        tex_data.width,
        tex_data.height,
        &tex_data.pixels,
    );
    assert!(entry.is_some(), "Upload should succeed");
    let entry = entry.unwrap();

    // Verify entry properties
    assert_eq!(entry.original_width, tex_data.width);
    assert_eq!(entry.original_height, tex_data.height);
    assert!(entry.uv_width > 0.0, "UV width should be positive");
    assert!(entry.uv_height > 0.0, "UV height should be positive");

    // Verify lookup works
    assert!(
        atlas.get(SCROLL_UP_BUTTON).is_some(),
        "Atlas lookup should find uploaded texture"
    );
    assert_eq!(atlas.len(), 1, "Atlas should have 1 texture");

    // Duplicate upload should return existing entry
    let dup = atlas.upload(
        &queue,
        SCROLL_UP_BUTTON,
        tex_data.width,
        tex_data.height,
        &tex_data.pixels,
    );
    assert!(dup.is_some());
    assert_eq!(atlas.len(), 1, "Duplicate upload should not increase count");
}

#[test]
fn layer5_gpu_atlas_tier_selection() {
    let Some(gpu) = common::try_create_gpu_device() else {
        eprintln!("Skipping GPU test: no adapter available");
        return;
    };
    let (device, queue) = (&gpu.device, &gpu.queue);

    let mut atlas = GpuTextureAtlas::new(&device);

    // Upload a small texture (should go to tier 0: 64 by 64)
    let small_pixels = vec![255u8; 16 * 16 * 4];
    let small = atlas.upload(&queue, "test/small_16x16", 16, 16, &small_pixels);
    assert!(small.is_some());
    assert_eq!(
        small.unwrap().tier,
        0,
        "16 by 16 texture should go to tier 0 (64 by 64 cells)"
    );

    // Upload a medium texture (should go to tier 1: 128 by 128)
    let medium_pixels =
        vec![255u8; (GPU_ATLAS_MEDIUM_TEXTURE_SIZE * GPU_ATLAS_MEDIUM_TEXTURE_SIZE * 4) as usize];
    let medium = atlas.upload(
        &queue,
        "test/medium_square",
        GPU_ATLAS_MEDIUM_TEXTURE_SIZE,
        GPU_ATLAS_MEDIUM_TEXTURE_SIZE,
        &medium_pixels,
    );
    assert!(medium.is_some());
    assert_eq!(
        medium.unwrap().tier,
        1,
        "{} by {} texture should go to tier 1 (128 by 128 cells)",
        GPU_ATLAS_MEDIUM_TEXTURE_SIZE,
        GPU_ATLAS_MEDIUM_TEXTURE_SIZE,
    );

    // Upload a larger texture (should go to tier 2)
    let large_pixels =
        vec![255u8; (GPU_ATLAS_LARGE_TEXTURE_SIZE * GPU_ATLAS_LARGE_TEXTURE_SIZE * 4) as usize];
    let large = atlas.upload(
        &queue,
        "test/large_square",
        GPU_ATLAS_LARGE_TEXTURE_SIZE,
        GPU_ATLAS_LARGE_TEXTURE_SIZE,
        &large_pixels,
    );
    assert!(large.is_some());
    assert_eq!(
        large.unwrap().tier,
        2,
        "{} by {} texture should go to tier 2 ({} by {} cells)",
        GPU_ATLAS_LARGE_TEXTURE_SIZE,
        GPU_ATLAS_LARGE_TEXTURE_SIZE,
        GPU_ATLAS_TIER_2_CELL_SIZE,
        GPU_ATLAS_TIER_2_CELL_SIZE,
    );

    // Upload an extra-large texture (should go to tier 3: 512 by 512)
    let xl_pixels =
        vec![255u8; (GPU_ATLAS_XL_TEXTURE_SIZE * GPU_ATLAS_XL_TEXTURE_SIZE * 4) as usize];
    let xl = atlas.upload(
        &queue,
        "test/xl_square",
        GPU_ATLAS_XL_TEXTURE_SIZE,
        GPU_ATLAS_XL_TEXTURE_SIZE,
        &xl_pixels,
    );
    assert!(xl.is_some());
    assert_eq!(
        xl.unwrap().tier,
        3,
        "{} by {} texture should go to tier 3 (512 by 512 cells)",
        GPU_ATLAS_XL_TEXTURE_SIZE,
        GPU_ATLAS_XL_TEXTURE_SIZE,
    );

    assert_eq!(atlas.len(), 4, "Atlas should have 4 textures");
}

#[test]
fn layer5_gpu_atlas_real_scroll_textures() {
    let Some(gpu) = common::try_create_gpu_device() else {
        eprintln!("Skipping GPU test: no adapter available");
        return;
    };
    let (device, queue) = (&gpu.device, &gpu.queue);

    let mut tex_mgr = make_texture_manager();

    let mut atlas = GpuTextureAtlas::new(&device);

    // Upload all three scroll bar textures
    let paths = [SCROLL_UP_BUTTON, SCROLL_DOWN_BUTTON, SCROLL_KNOB];
    for path in &paths {
        let data = tex_mgr.load(path);
        if let Some(data) = data {
            let entry = atlas.upload(&queue, path, data.width, data.height, &data.pixels);
            assert!(entry.is_some(), "Should upload {}", path);
            let entry = entry.unwrap();
            // Scroll bar button textures are small, should fit in tier 0 or 1
            assert!(
                entry.tier <= 1,
                "Scroll bar texture {} ({}x{}) should fit in tier 0 or 1, got tier {}",
                path,
                data.width,
                data.height,
                entry.tier,
            );
        } else {
            eprintln!("Warning: Could not load {}", path);
        }
    }

    // Verify all can be looked up
    for path in &paths {
        if atlas.get(path).is_some() {
            let entry = atlas.get(path).unwrap();
            assert!(entry.uv_width > 0.0);
            assert!(entry.uv_height > 0.0);
        }
    }
}
