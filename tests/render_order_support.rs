// Shared test helpers. Individual test binaries that `mod` this file use
// a subset of the helpers, so per-binary dead_code warnings are expected.
#![cfg(feature = "gui")]
#![allow(dead_code)]

#[path = "common/blizzard_addon_harness.rs"]
mod blizzard_addon_harness;
#[path = "common/blizzard_addon_manifest.rs"]
mod blizzard_addon_manifest;
#[path = "common/event_helpers.rs"]
mod event_helpers;
#[path = "common/panel_fixtures.rs"]
mod panel_fixtures;

use image::RgbaImage;
use std::path::{Path, PathBuf};
use std::{cell::RefCell, rc::Rc};
use wow_ui_sim::iced_app::{
    RegistryQuadBatchParams, build_quad_batch_for_registry, compute_frame_rect,
};
use wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen as load_blizzard_addon_closure_for_screen;
use wow_ui_sim::lua_api::{SimState, WowLuaEnv};
use wow_ui_sim::render::{GlyphAtlas, QuadBatch, QuadVertex, TextureRequest, WowFontSystem};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::texture::TextureManager;
use wow_ui_sim::widget::WidgetRegistry;

pub(crate) fn build_strata_buckets(env: &WowLuaEnv) -> Vec<Vec<u64>> {
    let mut state = env.state().borrow_mut();
    let _ = state.get_strata_buckets();
    state.strata_buckets.as_ref().unwrap().clone()
}

fn make_font_system() -> Rc<RefCell<WowFontSystem>> {
    Rc::new(RefCell::new(WowFontSystem::new()))
}

pub(crate) fn make_texture_manager() -> TextureManager {
    TextureManager::new()
}

pub(crate) fn build_screenshot_like_batch(
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

pub(crate) fn is_descendant_of(
    widgets: &WidgetRegistry,
    mut frame_id: u64,
    ancestor_id: u64,
) -> bool {
    loop {
        if frame_id == ancestor_id {
            return true;
        }
        let Some(parent_id) = widgets.get(frame_id).and_then(|frame| frame.parent_id) else {
            return false;
        };
        frame_id = parent_id;
    }
}

pub(crate) fn diff_pixels_in_rect(
    before: &RgbaImage,
    after: &RgbaImage,
    rect: (u32, u32, u32, u32),
    per_channel_tolerance: u8,
) -> Vec<(u32, u32, [u8; 4], [u8; 4])> {
    let mut diffs = Vec::new();
    let max_x = (rect.0 + rect.2).min(before.width());
    let max_y = (rect.1 + rect.3).min(before.height());
    for y in rect.1..max_y {
        for x in rect.0..max_x {
            let lhs = before.get_pixel(x, y).0;
            let rhs = after.get_pixel(x, y).0;
            let differs =
                (0..4).any(|channel| lhs[channel].abs_diff(rhs[channel]) > per_channel_tolerance);
            if differs {
                diffs.push((x, y, lhs, rhs));
            }
        }
    }
    diffs
}

fn quad_bounds_from_vertices(verts: &[QuadVertex]) -> (f32, f32, f32, f32) {
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

pub(crate) fn quad_bounds(batch: &QuadBatch, request: &TextureRequest) -> (f32, f32, f32, f32) {
    let start = request.vertex_start as usize;
    let end = start + request.vertex_count as usize;
    quad_bounds_from_vertices(&batch.vertices[start..end])
}

pub(crate) fn request_overlaps_rect(
    batch: &QuadBatch,
    request: &TextureRequest,
    rect: (f32, f32, f32, f32),
) -> bool {
    let bounds = quad_bounds(batch, request);
    let rect_right = rect.0 + rect.2;
    let rect_bottom = rect.1 + rect.3;
    bounds.0 < rect_right && bounds.2 > rect.0 && bounds.1 < rect_bottom && bounds.3 > rect.1
}

pub(crate) fn vertex_range_bounds(
    batch: &QuadBatch,
    vertex_start: usize,
    vertex_count: usize,
) -> (f32, f32, f32, f32) {
    quad_bounds_from_vertices(&batch.vertices[vertex_start..vertex_start + vertex_count])
}

pub(crate) fn bounds_overlap_rect(
    bounds: (f32, f32, f32, f32),
    rect: (f32, f32, f32, f32),
) -> bool {
    let rect_right = rect.0 + rect.2;
    let rect_bottom = rect.1 + rect.3;
    bounds.0 < rect_right && bounds.2 > rect.0 && bounds.1 < rect_bottom && bounds.3 > rect.1
}

pub(crate) fn request_path_for_frame_texture(
    texture: &str,
    atlas_tex_coords: Option<(f32, f32, f32, f32)>,
) -> String {
    let Some((cl, cr, ct, cb)) = atlas_tex_coords else {
        return texture.to_string();
    };
    let is_full = (cl - 0.0).abs() < 0.001
        && (cr - 1.0).abs() < 0.001
        && (ct - 0.0).abs() < 0.001
        && (cb - 1.0).abs() < 0.001;
    if is_full {
        texture.to_string()
    } else {
        format!("{texture}@crop:{cl:.6},{cr:.6},{ct:.6},{cb:.6}")
    }
}

pub(crate) fn world_quest_pin_pairs(
    state: &SimState,
) -> Vec<(u64, u64, wow_ui_sim::LayoutRect, String, String)> {
    world_quest_icon_ids(state)
        .into_iter()
        .filter_map(|icon_id| world_quest_pin_pair(state, icon_id))
        .collect()
}

fn world_quest_icon_ids(state: &SimState) -> Vec<u64> {
    state
        .widgets
        .iter_ids()
        .filter(|&id| {
            state
                .widgets
                .get(id)
                .is_some_and(|frame| frame.atlas.as_deref() == Some("Worldquest-icon"))
        })
        .collect()
}

fn world_quest_pin_pair(
    state: &SimState,
    icon_id: u64,
) -> Option<(u64, u64, wow_ui_sim::LayoutRect, String, String)> {
    let button_id = world_quest_button_id(state, icon_id)?;
    let circle_id = world_quest_circle_id(state, button_id)?;
    let circle_request_path = frame_request_path(state, circle_id)?;
    let icon_request_path = frame_request_path(state, icon_id)?;

    Some((
        circle_id,
        icon_id,
        compute_frame_rect(&state.widgets, circle_id, 1024.0, 768.0),
        circle_request_path,
        icon_request_path,
    ))
}

fn world_quest_button_id(state: &SimState, icon_id: u64) -> Option<u64> {
    let display_frame_id = state.widgets.get(icon_id)?.parent_id?;
    state.widgets.get(display_frame_id)?.parent_id
}

fn world_quest_circle_id(state: &SimState, button_id: u64) -> Option<u64> {
    state.widgets.iter_ids().find(|&id| {
        state.widgets.get(id).is_some_and(|frame| {
            frame.parent_id == Some(button_id)
                && frame.atlas.as_deref() == Some("UI-QuestPoi-QuestNumber")
        })
    })
}

fn frame_request_path(state: &SimState, id: u64) -> Option<String> {
    let frame = state.widgets.get(id)?;
    let texture = frame.texture.as_deref()?;
    Some(request_path_for_frame_texture(
        texture,
        frame.atlas_tex_coords,
    ))
}

pub(crate) fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

pub(crate) const ISOLATED_WORLD_MAP_ROOT_ADDONS: &[&str] = &[
    "Blizzard_FrameEffects",
    "Blizzard_StoreUI",
    "Blizzard_UIPanels_Game",
    "Blizzard_MapCanvasSecureUtil",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_WorldMap",
    "Blizzard_GameMenu",
    "Blizzard_UIWidgets",
    "Blizzard_AddOnList",
    "Blizzard_TimerunningUtil",
];

pub(crate) fn discover_blizzard_addon_closure_for_screen(
    blizzard_ui_dir: &Path,
    screen: ScreenKind,
    roots: &[&str],
) -> Vec<(String, PathBuf)> {
    load_blizzard_addon_closure_for_screen(blizzard_ui_dir, screen, roots)
}

pub(crate) fn discover_blizzard_addon_closure_for_screen_with_overrides(
    blizzard_ui_dir: &Path,
    screen: ScreenKind,
    roots: &[&str],
    overrides: &[wow_ui_sim::loader::BlizzardAddonOverride<'_>],
) -> Vec<(String, PathBuf)> {
    wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen_with_overrides(
        blizzard_ui_dir,
        screen,
        roots,
        overrides,
    )
}

pub(crate) fn env_with_root_addons_ui(roots: &[&str]) -> WowLuaEnv {
    env_with_root_addons_ui_with_overrides(roots, &[])
}

pub(crate) fn env_with_root_addons_ui_with_overrides(
    roots: &[&str],
    overrides: &[wow_ui_sim::loader::BlizzardAddonOverride<'_>],
) -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let (env, _) = blizzard_addon_harness::build_blizzard_addon_closure_env(&ui, roots, overrides);
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

pub(crate) fn env_with_isolated_world_map_ui() -> WowLuaEnv {
    env_with_root_addons_ui_with_overrides(
        ISOLATED_WORLD_MAP_ROOT_ADDONS,
        blizzard_addon_manifest::WORLD_MAP_VOICE_CHAT_OVERRIDES,
    )
}

pub(crate) fn fire_addon_loaded(env: &WowLuaEnv, addon_name: &str) {
    let _ = env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(addon_name)]);
}

pub(crate) use event_helpers::fire_player_entering_world;

pub(crate) fn env_with_isolated_world_map() -> WowLuaEnv {
    let env = env_with_isolated_world_map_ui();
    open_world_map(&env);
    env
}

pub(crate) fn open_world_map(env: &WowLuaEnv) {
    env.exec("ToggleWorldMap()")
        .expect("failed to toggle world map after startup");
    wow_ui_sim::startup::process_pending_timers(env);
    wow_ui_sim::startup::fire_one_on_update_tick(env);
}

#[cfg(test)]
mod tests {
    use super::*;
    use wow_ui_sim::widget::{Frame, WidgetType};

    #[test]
    fn world_quest_pin_pairs_collect_matching_circle_and_icon() {
        let mut state = SimState::default();
        state.widgets.register(Frame {
            id: 10,
            widget_type: WidgetType::Frame,
            width: 32.0,
            height: 32.0,
            ..Default::default()
        });
        state.widgets.register(Frame {
            id: 20,
            widget_type: WidgetType::Frame,
            parent_id: Some(10),
            ..Default::default()
        });
        state.widgets.register(Frame {
            id: 30,
            widget_type: WidgetType::Texture,
            parent_id: Some(10),
            width: 32.0,
            height: 32.0,
            atlas: Some("UI-QuestPoi-QuestNumber".to_string()),
            texture: Some("Interface/Circle".to_string()),
            atlas_tex_coords: Some((0.1, 0.9, 0.2, 0.8)),
            ..Default::default()
        });
        state.widgets.register(Frame {
            id: 40,
            widget_type: WidgetType::Texture,
            parent_id: Some(20),
            width: 16.0,
            height: 16.0,
            atlas: Some("Worldquest-icon".to_string()),
            texture: Some("Interface/Icon".to_string()),
            ..Default::default()
        });
        state.widgets.add_child(10, 20);
        state.widgets.add_child(10, 30);
        state.widgets.add_child(20, 40);

        let pairs = world_quest_pin_pairs(&state);

        assert_eq!(pairs.len(), 1);
        let (circle_id, icon_id, circle_rect, circle_request, icon_request) = &pairs[0];
        assert_eq!((*circle_id, *icon_id), (30, 40));
        assert_eq!((circle_rect.width, circle_rect.height), (32.0, 32.0));
        assert_eq!(
            circle_request,
            "Interface/Circle@crop:0.100000,0.900000,0.200000,0.800000"
        );
        assert_eq!(icon_request, "Interface/Icon");
    }
}
