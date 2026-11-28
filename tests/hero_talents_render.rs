#![cfg(feature = "gui")]

use crate::common;

use std::path::PathBuf;

use wow_ui_sim::atlas::get_atlas_info;
use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

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
    rect: wow_ui_sim::LayoutRect,
) -> bool {
    let bounds = quad_bounds(batch, request);
    let tolerance = 0.1;
    (bounds.0 - rect.x).abs() <= tolerance
        && (bounds.1 - rect.y).abs() <= tolerance
        && (bounds.2 - (rect.x + rect.width)).abs() <= tolerance
        && (bounds.3 - (rect.y + rect.height)).abs() <= tolerance
}

fn assert_bounds_match_rect(
    bounds: (f32, f32, f32, f32),
    rect: wow_ui_sim::LayoutRect,
    label: &str,
) {
    let tolerance = 0.1;
    assert!(
        (bounds.0 - rect.x).abs() <= tolerance,
        "{label} min_x should match rect.x: bounds={bounds:?} rect={rect:?}"
    );
    assert!(
        (bounds.1 - rect.y).abs() <= tolerance,
        "{label} min_y should match rect.y: bounds={bounds:?} rect={rect:?}"
    );
    assert!(
        (bounds.2 - (rect.x + rect.width)).abs() <= tolerance,
        "{label} max_x should match rect right edge: bounds={bounds:?} rect={rect:?}"
    );
    assert!(
        (bounds.3 - (rect.y + rect.height)).abs() <= tolerance,
        "{label} max_y should match rect bottom edge: bounds={bounds:?} rect={rect:?}"
    );
}

fn parse_crop_coords(path: &str) -> Option<(f32, f32, f32, f32)> {
    let (_, crop_str) = path.split_once("@crop:")?;
    let mut parts = crop_str
        .split(',')
        .filter_map(|part| part.parse::<f32>().ok());
    let left = parts.next()?;
    let right = parts.next()?;
    let top = parts.next()?;
    let bottom = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some((left, right, top, bottom))
}

post_wrath_ui_fixture! {
#[test]
fn hero_spec_icon_and_mask_quads_match_layout_rect() {
    let env = setup_full_ui();
    open_class_talent_frame(&env);

    let (icon_rect, icon_path, mask_path) = {
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
        let mask_id = *button
            .children_keys
            .get("IconMask")
            .expect("IconMask child");
        let icon_rect =
            wow_ui_sim::iced_app::compute_frame_rect(&state.widgets, icon_id, 1024.0, 768.0);
        let icon_path = state
            .widgets
            .get(icon_id)
            .and_then(|frame| frame.texture.clone())
            .expect("Icon1 should have a texture path");
        let mask_path = state
            .widgets
            .get(mask_id)
            .and_then(|frame| frame.texture.clone())
            .expect("IconMask should have a texture path");
        (icon_rect, icon_path, mask_path)
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
    let icon_requests: Vec<_> = batch
        .texture_requests
        .iter()
        .filter(|request| request.path == icon_path || request.path.starts_with(&icon_crop_prefix))
        .filter(|request| request_matches_rect(&batch, request, icon_rect))
        .collect();
    assert!(
        !icon_requests.is_empty(),
        "HeroSpecButton.Icon1 should emit at least one textured quad request"
    );
    let icon_request_path = &icon_requests[0].path;
    for request in &icon_requests {
        assert_eq!(
            &request.path, icon_request_path,
            "HeroSpecButton.Icon1 duplicate quads should share the same cropped atlas path"
        );
        assert_bounds_match_rect(quad_bounds(&batch, request), icon_rect, "hero spec icon");
    }

    let mask_crop_prefix = format!("{mask_path}@crop:");
    let mask_requests: Vec<_> = batch
        .mask_texture_requests
        .iter()
        .filter(|request| request.path == mask_path || request.path.starts_with(&mask_crop_prefix))
        .collect();
    assert!(
        !mask_requests.is_empty(),
        "HeroSpecButton.IconMask should emit at least one mask quad request"
    );
    let mask_request_path = &mask_requests[0].path;
    for request in &mask_requests {
        assert_eq!(
            &request.path, mask_request_path,
            "HeroSpecButton.IconMask duplicate quads should share the same mask path"
        );
        assert_bounds_match_rect(quad_bounds(&batch, request), icon_rect, "hero spec mask");
    }
}
}

post_wrath_ui_fixture! {
fn hero_icon_id(registry: &wow_ui_sim::widget::WidgetRegistry) -> u64 {
    let mut id = registry
        .get_id_by_name("PlayerSpellsFrame")
        .expect("PlayerSpellsFrame");
    for key in [
        "TalentsFrame",
        "HeroTalentsContainer",
        "HeroSpecButton",
        "Icon1",
    ] {
        id = registry.get(id).unwrap().children_keys[key];
    }
    id
}

fn require_icon_crop<'a>(
    batch: &'a wow_ui_sim::render::QuadBatch,
    icon: &wow_ui_sim::widget::Frame,
    rect: wow_ui_sim::LayoutRect,
    atlas_file: &str,
) -> &'a wow_ui_sim::render::TextureRequest {
    let prefix = format!("{atlas_file}@crop:");
    let candidates: Vec<_> = batch
        .texture_requests
        .iter()
        .filter(|request| request.path.starts_with(&prefix))
        .collect();
    if let Some(request) = candidates.iter().copied()
        .find(|request| request_matches_rect(batch, request, rect))
    {
        return request;
    }
    let bounds: Vec<_> = candidates.iter()
        .map(|request| (&request.path, quad_bounds(batch, request)))
        .collect();
    panic!(
        "HeroSpecButton.Icon1 should emit a cropped atlas request: rect={rect:?}, cached={:?}, candidates={bounds:?}",
        icon.layout_rect
    )
}

fn assert_atlas_crop_coords(
    request: &wow_ui_sim::render::TextureRequest,
    atlas: &wow_ui_sim::atlas::AtlasInfo,
) {
    let crop_coords =
        parse_crop_coords(&request.path).expect("HeroSpecButton.Icon1 crop request should parse");
    let expected = (
        atlas.left_tex_coord,
        atlas.right_tex_coord,
        atlas.top_tex_coord,
        atlas.bottom_tex_coord,
    );
    let tolerance = 0.000001;
    assert!(
        (crop_coords.0 - expected.0).abs() <= tolerance
            && (crop_coords.1 - expected.1).abs() <= tolerance
            && (crop_coords.2 - expected.2).abs() <= tolerance
            && (crop_coords.3 - expected.3).abs() <= tolerance,
        "HeroSpecButton.Icon1 crop coords should match atlas entry: crop={crop_coords:?} expected={expected:?}"
    );
}

#[test]
fn hero_spec_icon_crop_request_matches_atlas_entry() {
    let env = setup_full_ui();
    open_class_talent_frame(&env);
    let icon_id = hero_icon_id(&env.state().borrow().widgets);
    let buckets = {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    let icon = state.widgets.get(icon_id).unwrap();
    let atlas_name = icon.atlas.as_deref().expect("Icon1 should have an atlas");
    let atlas = get_atlas_info(atlas_name).expect("atlas entry should exist");
    assert_eq!(icon.texture.as_deref(), Some(atlas.info.file), "Icon1 source must be the atlas backing file");
    let rect = wow_ui_sim::iced_app::compute_frame_rect(&state.widgets, icon_id, 1024.0, 768.0);
    let batch = build_quad_batch_for_registry(RegistryQuadBatchParams::new(
        &state.widgets, (1024.0, 768.0), &buckets,
    ));
    let request = require_icon_crop(&batch, icon, rect, atlas.info.file);
    assert_atlas_crop_coords(request, atlas.info);
}

#[test]
fn hero_spec_icon_replaces_xml_placeholder_file_identity() {
    let env = setup_full_ui();
    open_class_talent_frame(&env);
    env.exec(r#"
        local button = PlayerSpellsFrame.TalentsFrame.HeroTalentsContainer.HeroSpecButton
        local icon = button.Icon1
        local placeholder = button.Icon2:GetTexture()
        assert(not button.Icon2:IsShown(), "single active subtree fixture")
        assert(placeholder == 134400 and button.Icon2:GetTextureFileID() == placeholder)
        assert(icon:IsVisible() and icon:GetAtlas() == "talents-heroclass-paladin-lightsmith")
        local atlas = C_Texture.GetAtlasInfo(icon:GetAtlas())
        assert(icon:GetTextureFilePath() == atlas.filename, "atlas backing path")
        assert(icon:GetTexture() ~= placeholder, "GetTexture retained the XML file ID")
        assert(icon:GetTextureFileID() ~= placeholder, "GetTextureFileID retained the XML file ID")
    "#).unwrap();
}
}
