//! The objective tracker's height comes from the managed container Blizzard
//! sizes for it. Headless startup used to force `SetHeight(836.5)` on every
//! OnUpdate tick, which at UI scales above ~1.3 made the (clamped-to-screen)
//! tracker taller than the space under the minimap and pushed it over it.
//!
//! Both tests mirror the screenshot command's order of operations: settle
//! startup, apply the UI scale, apply the canvas size, run the extra ticks.

use crate::common;

use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{apply_ui_scale, run_extra_update_ticks, settle_headless_startup};
use wow_ui_sim::xml::{clear_templates, register_intrinsic_templates};

/// Bottom padding `FramePositionDelegate:ManageRightFrameContainer` subtracts
/// from the space below the minimap cluster (UIParentPanelManager.lua).
const RIGHT_CONTAINER_BOTTOM_PADDING: f64 = 100.0;

struct RightSideGeometry {
    ui_height: f64,
    minimap_cluster_height: f64,
    container_height: f64,
    container_top: f64,
    tracker_height: f64,
    tracker_top: f64,
    tracker_bottom: f64,
}

fn load_settled_game_ui() -> common::LockedEnv {
    common::lock_env(|| {
        clear_templates();
        register_intrinsic_templates();
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        let ui_dir = wow_ui_sim::paths::default_blizzard_ui_addons_path()
            .expect("Blizzard UI cache should be available");
        env.set_screen_size(1024.0, 768.0);
        env.state().borrow_mut().addon_base_paths = vec![ui_dir.clone()];
        for (name, toc_path) in discover_blizzard_addons(&ui_dir) {
            if let Err(err) = load_addon(&env.loader_env(), &toc_path) {
                panic!("Failed to load Blizzard addon {name}: {err}");
            }
        }
        env.apply_post_load_workarounds();
        settle_headless_startup(&env);
        env
    })
}

fn lay_out_like_screenshot(env: &WowLuaEnv, width: f32, height: f32, ui_scale: f32) {
    apply_ui_scale(env, ui_scale);
    env.set_screen_size(width, height);
    run_extra_update_ticks(env, 3);
}

fn right_side_geometry(env: &WowLuaEnv) -> RightSideGeometry {
    let values: (f64, f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local container = RightManagedFrameContainer
            local tracker = ObjectiveTrackerFrame
            return UIParent:GetHeight(), MinimapCluster:GetHeight(),
                container:GetHeight(), container:GetTop(),
                tracker:GetHeight(), tracker:GetTop(), tracker:GetBottom()
            "#,
        )
        .expect("right-side frames should be laid out");
    let (ui_height, minimap_cluster_height, container_height, container_top, tracker_height, tracker_top, tracker_bottom) =
        values;
    RightSideGeometry {
        ui_height,
        minimap_cluster_height,
        container_height,
        container_top,
        tracker_height,
        tracker_top,
        tracker_bottom,
    }
}

fn assert_tracker_fills_managed_container(geo: &RightSideGeometry, label: &str) {
    let expected_container =
        geo.ui_height - geo.minimap_cluster_height - RIGHT_CONTAINER_BOTTOM_PADDING;
    assert!(
        (geo.container_height - expected_container).abs() < 0.5,
        "{label}: RightManagedFrameContainer height {} should be UIParent {} - MinimapCluster {} - {}",
        geo.container_height,
        geo.ui_height,
        geo.minimap_cluster_height,
        RIGHT_CONTAINER_BOTTOM_PADDING
    );
    assert!(
        (geo.tracker_height - geo.container_height).abs() < 0.5,
        "{label}: ObjectiveTrackerFrame height {} should match its container {}",
        geo.tracker_height,
        geo.container_height
    );
    assert!(
        (geo.tracker_top - geo.container_top).abs() < 0.5,
        "{label}: ObjectiveTrackerFrame top {} should sit at its container's top {}",
        geo.tracker_top,
        geo.container_top
    );
    assert!(
        geo.tracker_bottom >= -0.5,
        "{label}: ObjectiveTrackerFrame bottom {} is off screen",
        geo.tracker_bottom
    );
}

#[test]
fn objective_tracker_keeps_blizzard_height_through_headless_ticks() {
    let env = load_settled_game_ui();
    lay_out_like_screenshot(&env, 1600.0, 1200.0, 1.0);
    assert_tracker_fills_managed_container(&right_side_geometry(&env), "1600x1200 at scale 1");
}

#[test]
fn objective_tracker_fits_under_minimap_at_client_ui_scale() {
    let env = load_settled_game_ui();
    // 3440x1440 at uiScale 0.9: the client renders 1 unit as 1440/768 * 0.9 px.
    lay_out_like_screenshot(&env, 3440.0, 1440.0, 1.6875);
    let geo = right_side_geometry(&env);
    assert!(
        (geo.ui_height - 1440.0 / 1.6875).abs() < 0.5,
        "UIParent height {} should be the canvas height divided by the scale",
        geo.ui_height
    );
    assert_tracker_fills_managed_container(&geo, "3440x1440 at scale 1.6875");
}
