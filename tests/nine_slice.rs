//! Tests for NineSlice title bar textures and layout.

mod common;

use common::env_with_shared_xml;
use wow_ui_sim::iced_app::{build_quad_batch_for_registry, compute_frame_rect};

/// The NineSlice child of a ButtonFrameTemplate should have corner/edge
/// Texture children with atlas textures set by NineSliceUtil.ApplyLayout.
/// These render as normal Texture widgets -- no special NineSlice rendering needed.
#[test]
fn nine_slice_corner_textures_have_atlas() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestNineSlice", UIParent, "ButtonFrameTemplate")
        f:SetSize(400, 300)
        f:SetPoint("CENTER")
        f:Show()
    "#,
    )
    .unwrap();

    // The NineSlice child should exist (created by PortraitFrameBaseTemplate)
    let has_nine_slice: bool = env
        .eval("return TestNineSlice.NineSlice ~= nil")
        .unwrap();
    assert!(has_nine_slice, "ButtonFrameTemplate should have a NineSlice child");

    // NineSliceUtil.ApplyLayout should have set atlas on corner textures
    let tl_atlas: String = env
        .eval(
            r#"
        local ns = TestNineSlice.NineSlice
        if ns and ns.TopLeftCorner then
            return ns.TopLeftCorner:GetAtlas() or ""
        end
        return ""
    "#,
        )
        .unwrap();

    assert!(
        !tl_atlas.is_empty(),
        "NineSlice TopLeftCorner should have an atlas set (got empty string)"
    );
    assert!(
        tl_atlas.to_lowercase().contains("corner"),
        "TopLeftCorner atlas should contain 'corner', got: '{}'",
        tl_atlas
    );
}

/// NineSlice corner/edge textures should produce quads with atlas texture paths
/// in the rendering pipeline -- they render as normal Texture widgets.
#[test]
fn nine_slice_textures_produce_quads() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestNS9Quads", UIParent, "ButtonFrameTemplate")
        f:SetSize(400, 300)
        f:SetPoint("CENTER")
        f:Show()
    "#,
    )
    .unwrap();

    let buckets = {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(
        &state.widgets,
        (1024.0, 768.0),
        Some("TestNS9Quads"),
        None,
        None,
        None,
        None,
        None,
        &buckets,
    );

    // Collect all texture request paths
    let tex_paths: Vec<&str> = batch
        .texture_requests
        .iter()
        .map(|r| r.path.as_str())
        .collect();

    // NineSlice corner/edge textures use atlas entries from uiframemetal files
    let has_nine_slice_texture = tex_paths.iter().any(|p| {
        let lower = p.to_lowercase();
        lower.contains("uiframemetal") || lower.contains("uiframehorizontal") || lower.contains("uiframemetalvertical")
    });

    assert!(
        has_nine_slice_texture,
        "Quad batch should contain NineSlice atlas texture requests (uiframemetal/uiframehorizontal), \
         got: {:?}",
        tex_paths
    );
}

/// The NineSlice child frame created by SetAllPoints should match
/// the parent frame's bounds in the Rust layout computation.
#[test]
fn nine_slice_child_fills_parent_bounds() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestNSFill", UIParent, "ButtonFrameTemplate")
        f:SetSize(400, 300)
        f:SetPoint("CENTER")
        f:Show()
    "#,
    )
    .unwrap();

    // Check NineSlice anchor count from Lua side
    let ns_num_points: i32 = env.eval(r#"
        local ns = TestNSFill.NineSlice
        if ns then return ns:GetNumPoints() end
        return -1
    "#).unwrap();

    assert!(
        ns_num_points >= 2,
        "NineSlice should have at least 2 anchor points (from SetAllPoints), got {}",
        ns_num_points
    );

    // Verify the NineSlice Rust layout matches parent
    let state = env.state().borrow();
    let parent_id = state.widgets.get_id_by_name("TestNSFill").unwrap();
    let parent_rect = compute_frame_rect(&state.widgets, parent_id, 1024.0, 768.0);

    // Find the NineSlice child's Rust widget
    let parent = state.widgets.get(parent_id).unwrap();
    if let Some(&ns_id) = parent.children_keys.get("NineSlice") {
        let ns_rect = compute_frame_rect(&state.widgets, ns_id, 1024.0, 768.0);
        eprintln!("Parent: ({}, {}, {}x{})", parent_rect.x, parent_rect.y, parent_rect.width, parent_rect.height);
        eprintln!("NineSlice: ({}, {}, {}x{})", ns_rect.x, ns_rect.y, ns_rect.width, ns_rect.height);

        assert!(
            (ns_rect.width - parent_rect.width).abs() < 1.0,
            "NineSlice width {} should match parent width {}",
            ns_rect.width,
            parent_rect.width
        );
    } else {
        panic!("NineSlice not found in Rust children_keys");
    }
}

/// The NineSlice corner textures should have non-zero size layout rects
/// so they actually render visibly on screen.
#[test]
fn nine_slice_corner_has_nonzero_layout() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestNSLayout", UIParent, "ButtonFrameTemplate")
        f:SetSize(400, 300)
        f:SetPoint("CENTER")
        f:Show()
    "#,
    )
    .unwrap();

    assert_corner_has_nonzero_size(&env);
    assert_corner_is_visible(&env);
    assert_top_edge_has_nonzero_height(&env);
}

fn assert_corner_has_nonzero_size(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (w, h): (f32, f32) = env
        .eval(
            r#"
        local ns = TestNSLayout.NineSlice
        local tl = ns and ns.TopLeftCorner
        if tl then
            return tl:GetWidth(), tl:GetHeight()
        end
        return 0, 0
    "#,
        )
        .unwrap();

    assert!(
        w > 0.0 && h > 0.0,
        "TopLeftCorner should have non-zero size, got {}x{}",
        w,
        h
    );
}

fn assert_corner_is_visible(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let visible: bool = env
        .eval(
            r#"
        local ns = TestNSLayout.NineSlice
        return ns.TopLeftCorner:IsVisible()
    "#,
        )
        .unwrap();

    assert!(visible, "TopLeftCorner should be visible");
}

fn assert_top_edge_has_nonzero_height(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (ew, eh): (f32, f32) = env
        .eval(
            r#"
        local ns = TestNSLayout.NineSlice
        local te = ns and ns.TopEdge
        if te then
            return te:GetWidth(), te:GetHeight()
        end
        return 0, 0
    "#,
        )
        .unwrap();

    assert!(
        eh > 0.0,
        "TopEdge (title bar) should have non-zero height, got {}x{}",
        ew,
        eh
    );
}

/// NineSlice corner textures should have non-zero layout rects in the Rust
/// layout computation (what actually drives rendering).
#[test]
fn nine_slice_corner_rust_layout_nonzero() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestNSRustLayout", UIParent, "ButtonFrameTemplate")
        f:SetSize(400, 300)
        f:SetPoint("CENTER")
        f:Show()
    "#,
    )
    .unwrap();

    let state = env.state().borrow();

    // Find the TopLeftCorner texture by checking atlas
    let mut found_corner = false;
    for id in state.widgets.iter_ids() {
        let f = state.widgets.get(id).unwrap();
        if let Some(ref atlas) = f.atlas {
            if atlas.to_lowercase().contains("cornertopleft") {
                let rect = compute_frame_rect(&state.widgets, id, 1024.0, 768.0);
                eprintln!(
                    "TopLeftCorner atlas='{}' layout=({}, {}, {}x{}), visible={}, anchors={}",
                    atlas, rect.x, rect.y, rect.width, rect.height,
                    f.visible, f.anchors.len()
                );
                assert!(
                    rect.width > 0.0 && rect.height > 0.0,
                    "TopLeftCorner Rust layout should have non-zero size, got {}x{}",
                    rect.width,
                    rect.height
                );
                found_corner = true;
            }
        }
    }
    assert!(found_corner, "Should find a TopLeftCorner texture with atlas set");
}
