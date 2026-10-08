#![cfg(feature = "gui")]

use crate::common;

#[cfg(not(feature = "client-retail"))]
use std::path::PathBuf;

use wow_ui_sim::iced_app::{
    RegistryQuadBatchParams, build_quad_batch_for_registry, compute_frame_rect,
};
#[cfg(not(feature = "client-retail"))]
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(not(feature = "client-retail"))]
use wow_ui_sim::screen::ScreenKind;
#[cfg(not(feature = "client-retail"))]
use wow_ui_sim::startup::settle_headless_startup;

#[cfg(not(feature = "client-retail"))]
fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

#[cfg(not(feature = "client-retail"))]
fn load_settled_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("Failed to load Blizzard addon {name}: {err}"));
    }

    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
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

fn dump_request(
    label: &str,
    batch: &wow_ui_sim::render::QuadBatch,
    request: &wow_ui_sim::render::TextureRequest,
) {
    let start = request.vertex_start as usize;
    let end = start + request.vertex_count as usize;
    let bounds = quad_bounds(batch, request);
    eprintln!(
        "{label} request: path={} vertex_start={} vertex_count={} bounds=({:.2}, {:.2}) -> ({:.2}, {:.2})",
        request.path,
        request.vertex_start,
        request.vertex_count,
        bounds.0,
        bounds.1,
        bounds.2,
        bounds.3
    );
    for (idx, vertex) in batch.vertices[start..end].iter().enumerate() {
        eprintln!(
            "{label} vertex[{idx}]: position={:?} tex_coords={:?} color={:?} tex_index={} flags={} local_uv={:?} mask_tex_index={} mask_tex_coords={:?}",
            vertex.position,
            vertex.tex_coords,
            vertex.color,
            vertex.tex_index,
            vertex.flags,
            vertex.local_uv,
            vertex.mask_tex_index,
            vertex.mask_tex_coords
        );
    }
}

fn bounds_match_rect(bounds: (f32, f32, f32, f32), rect: wow_ui_sim::LayoutRect) -> bool {
    let tolerance = 0.1;
    (bounds.0 - rect.x).abs() <= tolerance
        && (bounds.1 - rect.y).abs() <= tolerance
        && (bounds.2 - (rect.x + rect.width)).abs() <= tolerance
        && (bounds.3 - (rect.y + rect.height)).abs() <= tolerance
}

fn is_compact_raid_manager_texture(path: &str) -> bool {
    let texture_path = path.to_ascii_lowercase();
    if texture_path.starts_with("interface\\hud\\uigroupmanager") {
        return true;
    }

    matches!(
        texture_path.as_str(),
        "gm-bgopen-leads"
            | "gm-bgopen-assists"
            | "gm-bgopen-regulars"
            | "gm-bgopen-party-leads"
            | "gm-bgopen-party-regulars"
            | "gm-btnforward-normal"
            | "gm-btnforward-hover"
            | "gm-btnforward-pressed"
            | "gm-btnforward-disabled"
    )
}

prefork_full_ui_case! {
fn compact_raid_manager_emits_background_and_forward_toggle_quad_bounds(env: &WowLuaEnv) {
    #[cfg(feature = "client-retail")]
    crate::objective_tracker_tree::settle_prefork_game_ui(env);

        env.exec(
            r#"
            A_Admin.SetPartySize(4)
            CompactRaidFrameManager_UpdateShown()
            CompactRaidFrameManager_UpdateOptionsFlowContainer()
            CompactRaidFrameManager_UpdateContainerVisibility()
            CompactRaidFrameManager_Collapse()
            "#,
        )
        .expect("Failed to configure collapsed compact raid manager fixture");

        let (background_rect, toggle_rect) = {
            let state = env.state().borrow();
            let manager_id = state
                .widgets
                .get_id_by_name("CompactRaidFrameManager")
                .expect("CompactRaidFrameManager should exist");
            let manager = state.widgets.get(manager_id).expect("manager widget");
            let background_id = *manager
                .children_keys
                .get("Background")
                .expect("manager background should exist");
            let toggle_id = *manager
                .children_keys
                .get("toggleButtonForward")
                .expect("forward toggle should exist");
            let background_rect = compute_frame_rect(&state.widgets, background_id, 1024.0, 768.0);
            let toggle_rect = compute_frame_rect(&state.widgets, toggle_id, 1024.0, 768.0);
            (background_rect, toggle_rect)
        };

        let buckets = {
            let mut state = env.state().borrow_mut();
            state.initialize_render_state();
            state.strata_buckets = None;
            let _ = state.get_strata_buckets();
            state
                .strata_buckets
                .as_ref()
                .expect("strata buckets should exist")
                .clone()
        };
        let state = env.state().borrow();
        let batch = build_quad_batch_for_registry(RegistryQuadBatchParams::new(
            &state.widgets,
            (1024.0, 768.0),
            &buckets,
        ));

        let mut uigroupmanager_requests = Vec::new();
        let mut background_requests = Vec::new();
        let mut toggle_requests = Vec::new();
        for request in &batch.texture_requests {
            let bounds = quad_bounds(&batch, request);
            if is_compact_raid_manager_texture(&request.path) {
                let record = (request.path.clone(), bounds);
                uigroupmanager_requests.push(record.clone());
                if bounds_match_rect(bounds, background_rect) {
                    background_requests.push(record.clone());
                }
                if bounds_match_rect(bounds, toggle_rect) {
                    toggle_requests.push(record);
                }
            }
        }

        eprintln!(
            "CompactRaidFrameManager.Background frame rect: ({:.2}, {:.2}) -> ({:.2}, {:.2})",
            background_rect.x,
            background_rect.y,
            background_rect.x + background_rect.width,
            background_rect.y + background_rect.height
        );
        for (path, bounds) in &uigroupmanager_requests {
            eprintln!(
                "UIGroupManager quad: path={} bounds=({:.2}, {:.2}) -> ({:.2}, {:.2})",
                path, bounds.0, bounds.1, bounds.2, bounds.3
            );
        }
        for (path, bounds) in &background_requests {
            eprintln!(
                "Background quad match: path={} bounds=({:.2}, {:.2}) -> ({:.2}, {:.2})",
                path, bounds.0, bounds.1, bounds.2, bounds.3
            );
        }

        eprintln!(
            "CompactRaidFrameManager.toggleButtonForward frame rect: ({:.2}, {:.2}) -> ({:.2}, {:.2})",
            toggle_rect.x,
            toggle_rect.y,
            toggle_rect.x + toggle_rect.width,
            toggle_rect.y + toggle_rect.height
        );
        for (path, bounds) in &toggle_requests {
            eprintln!(
                "Forward-toggle quad match: path={} bounds=({:.2}, {:.2}) -> ({:.2}, {:.2})",
                path, bounds.0, bounds.1, bounds.2, bounds.3
            );
        }

        assert!(
            !background_requests.is_empty(),
            "expected at least one CompactRaidFrameManager background texture request"
        );
        assert!(
            !toggle_requests.is_empty(),
            "expected at least one CompactRaidFrameManager forward-toggle texture request"
        );

        let background_request = batch
            .texture_requests
            .iter()
            .find(|request| {
                request.path == background_requests[0].0
                    && quad_bounds(&batch, request) == background_requests[0].1
            })
            .expect("background request should still exist");
        let toggle_request = batch
            .texture_requests
            .iter()
            .find(|request| {
                request.path == toggle_requests[0].0
                    && quad_bounds(&batch, request) == toggle_requests[0].1
            })
            .expect("toggle request should still exist");

        dump_request("Background", &batch, background_request);
        dump_request("Forward-toggle", &batch, toggle_request);

}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn compact_raid_manager_emits_background_and_forward_toggle_quad_bounds() {
    let env = load_settled_game_ui();
    compact_raid_manager_emits_background_and_forward_toggle_quad_bounds::run(&env);
}
