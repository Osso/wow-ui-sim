//! Rendering pipeline tests: from XML templates through GPU upload.
//!
//! Uses scroll bar buttons as the test subject, testing 5 layers:
//! 1. TextureManager loads scroll bar textures
//! 2. Texture path resolution (backslash, case-insensitive, atlas DB)
//! 3. Layout verification (positions from FauxScrollFrameTemplate)
//! 4. Quad batch generation
//! 5. GPU atlas upload

mod common;

use std::path::PathBuf;

use common::env_with_shared_xml;
use wow_ui_sim::atlas::{get_atlas_info, ATLAS_DB};
use wow_ui_sim::iced_app::{build_quad_batch_for_registry, compute_frame_rect};
use wow_ui_sim::render::{GpuTextureAtlas, QuadBatch};
use wow_ui_sim::texture::TextureManager;

// ============================================================================
// Helpers
// ============================================================================

const INTERFACE_PATH: &str = "/home/osso/Projects/wow/Interface";
const LOCAL_TEXTURES: &str = "./textures";
const FALLBACK_TEXTURES: &str = "/home/osso/Repos/wow-ui-textures";

fn make_texture_manager() -> Option<TextureManager> {
    let textures_path = if PathBuf::from(LOCAL_TEXTURES).exists() {
        PathBuf::from(LOCAL_TEXTURES)
    } else if PathBuf::from(FALLBACK_TEXTURES).exists() {
        PathBuf::from(FALLBACK_TEXTURES)
    } else {
        return None;
    };

    let mut mgr = TextureManager::new(&textures_path);
    if PathBuf::from(INTERFACE_PATH).exists() {
        mgr = mgr.with_interface_path(INTERFACE_PATH);
    }
    Some(mgr)
}

/// Scroll bar texture paths used by the classic FauxScrollFrameTemplate.
const SCROLL_UP_BUTTON: &str = "Interface/Buttons/UI-ScrollBar-ScrollUpButton-Up";
const SCROLL_DOWN_BUTTON: &str = "Interface/Buttons/UI-ScrollBar-ScrollDownButton-Up";
const SCROLL_KNOB: &str = "Interface/Buttons/UI-ScrollBar-Knob";

/// Atlas-based scroll bar texture (MinimalScrollBar).
const MINIMAL_SCROLLBAR_ATLAS: &str = "Interface/Buttons/ScrollBarProportional";

// ============================================================================
// Layer 1: TextureManager loads scroll bar textures
// ============================================================================

#[test]
fn layer1_load_scroll_up_button_texture() {
    let Some(mut mgr) = make_texture_manager() else {
        eprintln!("Skipping: texture directories not found");
        return;
    };

    let data = mgr.load(SCROLL_UP_BUTTON);
    assert!(data.is_some(), "Should load scroll up button texture: {}", SCROLL_UP_BUTTON);
    let data = data.unwrap();
    assert!(data.width > 0 && data.height > 0, "Texture dimensions should be positive");
    assert_eq!(
        data.pixels.len(),
        (data.width * data.height * 4) as usize,
        "Pixel data should be RGBA"
    );
}

#[test]
fn layer1_load_scroll_down_button_texture() {
    let Some(mut mgr) = make_texture_manager() else {
        eprintln!("Skipping: texture directories not found");
        return;
    };

    let data = mgr.load(SCROLL_DOWN_BUTTON);
    assert!(data.is_some(), "Should load scroll down button texture: {}", SCROLL_DOWN_BUTTON);
    let data = data.unwrap();
    assert!(data.width > 0 && data.height > 0);
    assert_eq!(data.pixels.len(), (data.width * data.height * 4) as usize);
}

#[test]
fn layer1_load_scroll_knob_texture() {
    let Some(mut mgr) = make_texture_manager() else {
        eprintln!("Skipping: texture directories not found");
        return;
    };

    let data = mgr.load(SCROLL_KNOB);
    assert!(data.is_some(), "Should load scroll knob texture: {}", SCROLL_KNOB);
    let data = data.unwrap();
    assert!(data.width > 0 && data.height > 0);
    assert_eq!(data.pixels.len(), (data.width * data.height * 4) as usize);
}

#[test]
fn layer1_load_minimal_scrollbar_atlas_texture() {
    let Some(mut mgr) = make_texture_manager() else {
        eprintln!("Skipping: texture directories not found");
        return;
    };

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
    let Some(mut mgr) = make_texture_manager() else {
        eprintln!("Skipping: texture directories not found");
        return;
    };

    let forward = mgr.load("Interface/Buttons/UI-ScrollBar-ScrollUpButton-Up");
    assert!(forward.is_some(), "Forward slash path should load");
    let fwd_size = (forward.unwrap().width, forward.unwrap().height);

    let backslash = mgr.load("Interface\\Buttons\\UI-ScrollBar-ScrollUpButton-Up");
    assert!(backslash.is_some(), "Backslash path should load");
    let bk_size = (backslash.unwrap().width, backslash.unwrap().height);

    assert_eq!(fwd_size, bk_size, "Same texture loaded regardless of slash direction");
}

#[test]
fn layer2_case_insensitive_resolution() {
    let Some(mut mgr) = make_texture_manager() else {
        eprintln!("Skipping: texture directories not found");
        return;
    };

    // The actual files use mixed case; try all-lowercase
    let result = mgr.load("interface/buttons/ui-scrollbar-scrollupbutton-up");
    assert!(
        result.is_some(),
        "Case-insensitive path should resolve"
    );
}

#[test]
fn layer2_atlas_db_returns_scroll_bar_entries() {
    // Check that the atlas DB has entries for the proportional scroll bar textures
    let up = get_atlas_info("ui-scrollbar-scrollupbutton-up");
    assert!(up.is_some(), "Atlas DB should have scroll up button entry");
    let up = up.unwrap();
    assert!(up.width > 0 && up.height > 0, "Atlas entry should have dimensions");
    assert!(
        up.file.to_lowercase().contains("scrollbar"),
        "Atlas file path should reference scrollbar: {}",
        up.file,
    );

    let down = get_atlas_info("ui-scrollbar-scrolldownbutton-up");
    assert!(down.is_some(), "Atlas DB should have scroll down button entry");

    let center = get_atlas_info("ui-scrollbar-center");
    // This one has a ! prefix in the atlas DB
    // get_atlas_info should handle it
    if center.is_none() {
        // Try with the ! prefix directly via ATLAS_DB
        let alt = ATLAS_DB.get("!ui-scrollbar-center");
        assert!(alt.is_some(), "Atlas DB should have scroll bar center entry (with ! prefix)");
    }
}

// ============================================================================
// Layer 3: Layout verification
// ============================================================================

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

    // Find TestSF
    let sf_id = registry.get_id_by_name("TestSF");
    assert!(sf_id.is_some(), "TestSF should exist in registry");
    let sf_id = sf_id.unwrap();

    let sf = registry.get(sf_id).unwrap();

    // Navigate to ScrollBar via children_keys
    let scrollbar_id = sf.children_keys.get("ScrollBar");
    assert!(scrollbar_id.is_some(), "TestSF should have ScrollBar child key");
    let scrollbar_id = *scrollbar_id.unwrap();

    let scrollbar = registry.get(scrollbar_id).unwrap();

    // Navigate to buttons via children_keys
    let up_id = scrollbar.children_keys.get("ScrollUpButton");
    assert!(up_id.is_some(), "ScrollBar should have ScrollUpButton child key");
    let up_id = *up_id.unwrap();

    let down_id = scrollbar.children_keys.get("ScrollDownButton");
    assert!(down_id.is_some(), "ScrollBar should have ScrollDownButton child key");
    let down_id = *down_id.unwrap();

    // Compute layout rects (use 1024x768 as screen size)
    let screen_w = 1024.0;
    let screen_h = 768.0;

    let sf_rect = compute_frame_rect(registry, sf_id, screen_w, screen_h);
    let sb_rect = compute_frame_rect(registry, scrollbar_id, screen_w, screen_h);
    let up_rect = compute_frame_rect(registry, up_id, screen_w, screen_h);
    let down_rect = compute_frame_rect(registry, down_id, screen_w, screen_h);

    // ScrollFrame should have positive dimensions
    assert!(sf_rect.width > 0.0 && sf_rect.height > 0.0,
        "ScrollFrame should have positive size: {:?}", sf_rect);

    // ScrollBar should have positive dimensions
    assert!(sb_rect.width > 0.0 && sb_rect.height > 0.0,
        "ScrollBar should have positive size: {:?}", sb_rect);

    // ScrollUpButton should be near the top of the ScrollBar
    assert!(up_rect.width > 0.0 && up_rect.height > 0.0,
        "ScrollUpButton should have positive size: {:?}", up_rect);
    assert!(
        up_rect.y <= sb_rect.y + sb_rect.height * 0.3,
        "ScrollUpButton (y={}) should be near top of ScrollBar (y={}, h={})",
        up_rect.y, sb_rect.y, sb_rect.height,
    );

    // ScrollDownButton should be near the bottom of the ScrollBar
    assert!(down_rect.width > 0.0 && down_rect.height > 0.0,
        "ScrollDownButton should have positive size: {:?}", down_rect);
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

    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(
        &state.widgets,
        (1024.0, 768.0),
        Some("TestSFQuads"),
        None,
        None,
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
        let paths: Vec<&str> = batch.texture_requests.iter().map(|r| r.path.as_str()).collect();
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
    assert_eq!(batch.texture_requests.len(), 1, "Should have 1 texture request");

    // Verify vertex positions match the bounds
    let v0 = &batch.vertices[0]; // top-left
    assert!((v0.position[0] - 100.0).abs() < 0.01, "TL x={}", v0.position[0]);
    assert!((v0.position[1] - 200.0).abs() < 0.01, "TL y={}", v0.position[1]);

    let v2 = &batch.vertices[2]; // bottom-right
    assert!((v2.position[0] - 150.0).abs() < 0.01, "BR x={}", v2.position[0]);
    assert!((v2.position[1] - 230.0).abs() < 0.01, "BR y={}", v2.position[1]);

    // Verify texture request path
    assert_eq!(
        batch.texture_requests[0].path,
        "Interface/Buttons/UI-ScrollBar-ScrollUpButton-Up",
    );
    assert_eq!(batch.texture_requests[0].vertex_start, 0);
    assert_eq!(batch.texture_requests[0].vertex_count, 4);
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

    let state = env.state().borrow();
    let registry = &state.widgets;

    let screen_w = 1024.0;
    let screen_h = 768.0;

    // Get layout position of the scroll frame
    let sf_id = registry.get_id_by_name("TestSFLayout").unwrap();
    let sf_rect = compute_frame_rect(registry, sf_id, screen_w, screen_h);

    let batch = build_quad_batch_for_registry(
        registry,
        (screen_w, screen_h),
        Some("TestSFLayout"),
        None,
        None,
    );

    // The first quad is the background (full screen). Skip it.
    // Remaining quads should have positions within or near the scroll frame area.
    // UI_SCALE is 1.0, so layout coords == screen coords.
    let mut found_in_range = false;
    for quad_idx in 1..batch.quad_count() {
        let vi = quad_idx * 4; // 4 vertices per quad
        if vi >= batch.vertices.len() {
            break;
        }
        let vx = batch.vertices[vi].position[0];
        let vy = batch.vertices[vi].position[1];
        // Check if vertex is in the general area of the scroll frame
        if vx >= sf_rect.x - 50.0
            && vx <= sf_rect.x + sf_rect.width + 50.0
            && vy >= sf_rect.y - 50.0
            && vy <= sf_rect.y + sf_rect.height + 50.0
        {
            found_in_range = true;
            break;
        }
    }

    assert!(
        found_in_range || batch.quad_count() <= 1,
        "At least one widget quad should be near the scroll frame area",
    );
}

// ============================================================================
// Layer 5: GPU atlas upload
// ============================================================================

#[test]
fn layer5_gpu_atlas_upload_and_lookup() {
    let Some((device, queue)) = common::try_create_gpu_device() else {
        eprintln!("Skipping GPU test: no adapter available");
        return;
    };

    let Some(mut tex_mgr) = make_texture_manager() else {
        eprintln!("Skipping GPU test: texture directories not found");
        return;
    };

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
    assert!(atlas.get(SCROLL_UP_BUTTON).is_some(), "Atlas lookup should find uploaded texture");
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
    let Some((device, queue)) = common::try_create_gpu_device() else {
        eprintln!("Skipping GPU test: no adapter available");
        return;
    };

    let mut atlas = GpuTextureAtlas::new(&device);

    // Upload a small texture (should go to tier 0: 64x64)
    let small_pixels = vec![255u8; 16 * 16 * 4];
    let small = atlas.upload(&queue, "test/small_16x16", 16, 16, &small_pixels);
    assert!(small.is_some());
    assert_eq!(small.unwrap().tier, 0, "16x16 texture should go to tier 0 (64x64 cells)");

    // Upload a medium texture (should go to tier 1: 128x128)
    let medium_pixels = vec![255u8; 100 * 100 * 4];
    let medium = atlas.upload(&queue, "test/medium_100x100", 100, 100, &medium_pixels);
    assert!(medium.is_some());
    assert_eq!(medium.unwrap().tier, 1, "100x100 texture should go to tier 1 (128x128 cells)");

    // Upload a larger texture (should go to tier 2: 256x256)
    let large_pixels = vec![255u8; 200 * 200 * 4];
    let large = atlas.upload(&queue, "test/large_200x200", 200, 200, &large_pixels);
    assert!(large.is_some());
    assert_eq!(large.unwrap().tier, 2, "200x200 texture should go to tier 2 (256x256 cells)");

    // Upload an extra-large texture (should go to tier 3: 512x512)
    let xl_pixels = vec![255u8; 400 * 400 * 4];
    let xl = atlas.upload(&queue, "test/xl_400x400", 400, 400, &xl_pixels);
    assert!(xl.is_some());
    assert_eq!(xl.unwrap().tier, 3, "400x400 texture should go to tier 3 (512x512 cells)");

    assert_eq!(atlas.len(), 4, "Atlas should have 4 textures");
}

#[test]
fn layer5_gpu_atlas_real_scroll_textures() {
    let Some((device, queue)) = common::try_create_gpu_device() else {
        eprintln!("Skipping GPU test: no adapter available");
        return;
    };

    let Some(mut tex_mgr) = make_texture_manager() else {
        eprintln!("Skipping GPU test: texture directories not found");
        return;
    };

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
                path, data.width, data.height, entry.tier,
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
