#![cfg(feature = "gui")]

use std::path::PathBuf;

use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::{QuadBatch, QuadVertex, TextureRequest};
use wow_ui_sim::widget::WidgetRegistry;

pub fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

pub fn setup_full_ui() -> WowLuaEnv {
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
    crate::common::load_click_binding_bootstrap(&env, &ui);
    env.apply_post_load_workarounds();
    fire_startup_sequence(&env);
    env
}

/// Match exact run_screenshot sequence:
/// fire_startup_events → apply_post_event_workarounds → process_pending_timers
/// → fire_one_on_update_tick → hide_runtime_hidden_frames
pub fn fire_startup_sequence(env: &WowLuaEnv) {
    wow_ui_sim::startup::fire_startup_events(env);
    env.apply_post_event_workarounds();
    wow_ui_sim::startup::process_pending_timers(env);
    wow_ui_sim::startup::fire_one_on_update_tick(env);
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(&*env.rilua());
}

/// Open the spellbook once (first load, demand-loads Blizzard_PlayerSpells).
/// Does NOT process timers after toggle (matching the screenshot command flow
/// where no timer processing happens between exec-lua and quad building).
pub fn open_spellbook(env: &WowLuaEnv) {
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()")
        .expect("Failed to toggle spellbook");
}

/// Find spell item frame IDs by traversing the Rust registry.
/// Path: PlayerSpellsFrame -> SpellBookFrame -> PagedSpellsFrame -> ViewFrames -> items
pub fn find_spell_item_ids(registry: &WidgetRegistry) -> Vec<u64> {
    let psf_id = registry.get_id_by_name("PlayerSpellsFrame");
    let psf_id = match psf_id {
        Some(id) => id,
        None => return Vec::new(),
    };
    let psf = registry.get(psf_id).unwrap();

    // SpellBookFrame is a child key of PlayerSpellsFrame
    let sb_id = match psf.children_keys.get("SpellBookFrame") {
        Some(&id) => id,
        None => return Vec::new(),
    };
    let sb = registry.get(sb_id).unwrap();

    // PagedSpellsFrame is a child key of SpellBookFrame
    let paged_id = match sb.children_keys.get("PagedSpellsFrame") {
        Some(&id) => id,
        None => return Vec::new(),
    };
    collect_viewframe_children(registry, paged_id)
}

/// Collect visible children from all shown ViewFrames under a PagedSpellsFrame.
pub fn collect_viewframe_children(registry: &WidgetRegistry, paged_id: u64) -> Vec<u64> {
    let paged = match registry.get(paged_id) {
        Some(f) => f,
        None => return Vec::new(),
    };
    let mut items = Vec::new();
    for &child_id in &paged.children {
        let child = match registry.get(child_id) {
            Some(f) => f,
            None => continue,
        };
        // ViewFrames are Frame-type children that contain spell items
        if !child.visible {
            continue;
        }
        for &item_id in &child.children {
            if let Some(item) = registry.get(item_id)
                && item.visible
                && item.width > 0.0
                && item.height > 0.0
            {
                items.push(item_id);
            }
        }
    }
    items
}

/// Build strata buckets from a WowLuaEnv (mutable borrow), then return a clone.
pub fn build_strata_buckets(env: &WowLuaEnv) -> Vec<Vec<u64>> {
    let mut state = env.state().borrow_mut();
    let _ = state.get_strata_buckets();
    state.strata_buckets.as_ref().unwrap().clone()
}

/// Build quad batch for the full registry at 1024x768.
pub fn build_quads(env: &WowLuaEnv) -> usize {
    let buckets = build_strata_buckets(env);
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(RegistryQuadBatchParams::new(
        &state.widgets,
        (1024.0, 768.0),
        &buckets,
    ));
    batch.quad_count()
}

pub fn quad_bounds(batch: &QuadBatch, request: &TextureRequest) -> (f32, f32, f32, f32) {
    let start = request.vertex_start as usize;
    let end = start + request.vertex_count as usize;
    quad_bounds_from_vertices(&batch.vertices[start..end])
}

pub fn quad_bounds_from_vertices(verts: &[QuadVertex]) -> (f32, f32, f32, f32) {
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

pub fn bounds_match_rect(bounds: (f32, f32, f32, f32), rect: wow_ui_sim::LayoutRect) -> bool {
    let tolerance = 0.1;
    (bounds.0 - rect.x).abs() <= tolerance
        && (bounds.1 - rect.y).abs() <= tolerance
        && (bounds.2 - (rect.x + rect.width)).abs() <= tolerance
        && (bounds.3 - (rect.y + rect.height)).abs() <= tolerance
}
