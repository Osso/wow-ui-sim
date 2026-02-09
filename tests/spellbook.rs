//! Tests for spellbook first-load rendering.
//!
//! Bug: Spellbook shows blank on first open, spells appear only on second open.
//! Lua state is correct on first open (IsVisible=true, anchors set, dimensions correct)
//! but the rendering pipeline skips the spell item frames.

mod common;

use std::path::PathBuf;
use wow_ui_sim::iced_app::frame_collect::collect_ancestor_visible_ids;
use wow_ui_sim::iced_app::{build_quad_batch_for_registry, compute_frame_rect};
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::WidgetRegistry;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
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
    fire_startup_sequence(&env);
    env
}

/// Match exact run_screenshot sequence:
/// fire_startup_events → apply_post_event_workarounds → process_pending_timers
/// → fire_one_on_update_tick → hide_runtime_hidden_frames
fn fire_startup_sequence(env: &WowLuaEnv) {
    wow_ui_sim::startup::fire_startup_events(env);
    env.apply_post_event_workarounds();
    wow_ui_sim::startup::process_pending_timers(env);
    wow_ui_sim::startup::fire_one_on_update_tick(env);
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());
}

/// Open the spellbook once (first load, demand-loads Blizzard_PlayerSpells).
/// Does NOT process timers after toggle (matching the screenshot command flow
/// where no timer processing happens between exec-lua and quad building).
fn open_spellbook(env: &WowLuaEnv) {
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()")
        .expect("Failed to toggle spellbook");
}

/// Find spell item frame IDs by traversing the Rust registry.
/// Path: PlayerSpellsFrame -> SpellBookFrame -> PagedSpellsFrame -> ViewFrames -> items
fn find_spell_item_ids(registry: &WidgetRegistry) -> Vec<u64> {
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
fn collect_viewframe_children(registry: &WidgetRegistry, paged_id: u64) -> Vec<u64> {
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
            if let Some(item) = registry.get(item_id) {
                if item.visible && item.width > 0.0 && item.height > 0.0 {
                    items.push(item_id);
                }
            }
        }
    }
    items
}

/// Build quad batch for the full registry at 1024x768.
fn build_quads(env: &WowLuaEnv) -> usize {
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(
        &state.widgets,
        (1024.0, 768.0),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    batch.quad_count()
}

fn build_quads_with_textures(env: &WowLuaEnv) -> (usize, Vec<String>) {
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(
        &state.widgets,
        (1024.0, 768.0),
        None, None, None, None, None, None,
    );
    let textures: Vec<String> = batch.texture_requests.iter()
        .map(|r| r.path.clone())
        .collect();
    (batch.quad_count(), textures)
}

/// Check that a frame is reachable from root via the children chain.
/// Returns (reachable, detail_string).
fn check_frame_reachability(registry: &WidgetRegistry, frame_id: u64) -> (bool, String) {
    let mut id = frame_id;
    let mut path = Vec::new();

    loop {
        let Some(frame) = registry.get(id) else {
            return (false, format!("Frame {} not found", id));
        };
        let name = frame.name.as_deref().unwrap_or("(anon)");
        path.push(format!("{}[{}]", name, id));

        let Some(parent_id) = frame.parent_id else {
            break;
        };
        let parent_has_child = registry
            .get(parent_id)
            .map(|p| p.children.contains(&id))
            .unwrap_or(false);
        if !parent_has_child {
            let pname = registry.get(parent_id)
                .and_then(|f| f.name.as_deref())
                .unwrap_or("?");
            return (false, format!(
                "BREAK: {}[{}] parent={}[{}] but NOT in children list. Path: {}",
                name, id, pname, parent_id, path.join(" -> ")
            ));
        }
        id = parent_id;
    }
    path.reverse();
    (true, path.join(" -> "))
}

/// Log the ancestor chain of a frame for debugging.
fn log_ancestor_chain(registry: &WidgetRegistry, frame_id: u64) {
    let mut id = frame_id;
    loop {
        let Some(frame) = registry.get(id) else {
            eprintln!("  Frame {} not found!", id);
            break;
        };
        let name = frame.name.as_deref().unwrap_or("(anon)");
        eprintln!(
            "  {} [{}]: visible={}, children={}, anchors={}",
            name, id, frame.visible, frame.children.len(), frame.anchors.len()
        );
        match frame.parent_id {
            Some(pid) => id = pid,
            None => break,
        }
    }
}

#[test]
fn spellbook_spells_visible_on_first_open() {
    let env = setup_full_ui();
    open_spellbook(&env);

    let sb_visible: bool = env
        .eval(
            "return PlayerSpellsFrame and PlayerSpellsFrame.SpellBookFrame \
             and PlayerSpellsFrame.SpellBookFrame:IsVisible() or false",
        )
        .unwrap();
    assert!(sb_visible, "SpellBookFrame should be visible after toggle");

    let item_ids = {
        let state = env.state().borrow();
        find_spell_item_ids(&state.widgets)
    };
    assert!(!item_ids.is_empty(), "Should have visible spell items");
    eprintln!("{} visible spell items on first open", item_ids.len());

    diagnose_missing_items(&env, &item_ids);

    let first_quads = build_quads(&env);
    eprintln!("First open quad count: {}", first_quads);

    // Close and reopen
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()").unwrap();
    let _ = env.process_timers();
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()").unwrap();
    let _ = env.process_timers();

    let second_quads = build_quads(&env);
    eprintln!("Second open quad count: {}", second_quads);

    assert_eq!(
        first_quads, second_quads,
        "First open should produce same quad count as second open.\n\
         First: {first_quads}, Second: {second_quads}, Diff: {}",
        second_quads as i64 - first_quads as i64,
    );
}

/// Check which items are missing from ancestor_visible and log details.
fn diagnose_missing_items(env: &WowLuaEnv, item_ids: &[u64]) {
    let state = env.state().borrow();
    let registry = &state.widgets;
    let ancestor_visible = collect_ancestor_visible_ids(registry);

    let mut in_set = 0;
    let mut missing = 0;
    for &item_id in item_ids {
        if ancestor_visible.contains_key(&item_id) {
            in_set += 1;
        } else {
            missing += 1;
            let (ok, detail) = check_frame_reachability(registry, item_id);
            eprintln!("Item {} NOT in ancestor_visible: ok={}, {}", item_id, ok, detail);
            log_ancestor_chain(registry, item_id);
        }
    }
    eprintln!("Ancestor-visible: {} in, {} missing", in_set, missing);
}

#[test]
fn spellbook_spell_items_in_ancestor_visible() {
    let env = setup_full_ui();
    open_spellbook(&env);

    let state = env.state().borrow();
    let item_ids = find_spell_item_ids(&state.widgets);
    assert!(!item_ids.is_empty(), "Should have spell items");

    let ancestor_visible = collect_ancestor_visible_ids(&state.widgets);

    let missing: Vec<_> = item_ids
        .iter()
        .filter(|id| !ancestor_visible.contains_key(id))
        .map(|&id| {
            let name = state.widgets.get(id)
                .and_then(|f| f.name.as_deref())
                .unwrap_or("(anon)");
            format!("{}[{}]", name, id)
        })
        .collect();

    assert!(
        missing.is_empty(),
        "All visible spell items should be in ancestor_visible.\n\
         Missing {} items: {:?}",
        missing.len(),
        &missing[..missing.len().min(10)]
    );
}

#[test]
fn spellbook_icon_textures_in_ancestor_visible() {
    let env = setup_full_ui();
    open_spellbook(&env);

    let state = env.state().borrow();
    let registry = &state.widgets;
    let item_ids = find_spell_item_ids(registry);
    assert!(!item_ids.is_empty(), "Should have spell items");

    let ancestor_visible = collect_ancestor_visible_ids(registry);

    // For each spell item, find its Button child, then Icon texture child
    let mut icons_found = 0u32;
    let mut icons_missing = 0u32;
    for &item_id in &item_ids {
        let Some(item) = registry.get(item_id) else { continue };
        let Some(&btn_id) = item.children_keys.get("Button") else { continue };
        let Some(btn) = registry.get(btn_id) else { continue };
        let Some(&icon_id) = btn.children_keys.get("Icon") else { continue };
        let Some(icon) = registry.get(icon_id) else { continue };

        if ancestor_visible.contains_key(&icon_id) {
            icons_found += 1;
        } else {
            icons_missing += 1;
            if icons_missing <= 3 {
                let (ok, detail) = check_frame_reachability(registry, icon_id);
                eprintln!("Icon {icon_id} NOT in ancestor_visible: ok={ok} {detail}");
                eprintln!("  icon: vis={} tex={:?} w={} h={}",
                    icon.visible, icon.texture, icon.width, icon.height);
                eprintln!("  btn {btn_id}: vis={} children={:?}",
                    btn.visible, btn.children);
                eprintln!("  item {item_id}: vis={} children={:?}",
                    item.visible, item.children);
            }
        }
    }
    eprintln!("Icons: found={icons_found} missing={icons_missing}");
    assert_eq!(icons_missing, 0,
        "All spell icon textures should be in ancestor_visible");
}

#[test]
fn spellbook_texture_requests_match_between_opens() {
    let env = setup_full_ui();
    open_spellbook(&env);

    let (q1, tex1) = build_quads_with_textures(&env);
    let icon_tex1: Vec<_> = tex1.iter()
        .filter(|t| t.to_lowercase().contains("icons"))
        .collect();

    // Close and reopen
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()").unwrap();
    let _ = env.process_timers();
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()").unwrap();
    let _ = env.process_timers();

    let (q2, tex2) = build_quads_with_textures(&env);
    let icon_tex2: Vec<_> = tex2.iter()
        .filter(|t| t.to_lowercase().contains("icons"))
        .collect();

    eprintln!("First open: {} quads, {} textures, {} icon textures",
        q1, tex1.len(), icon_tex1.len());
    eprintln!("Second open: {} quads, {} textures, {} icon textures",
        q2, tex2.len(), icon_tex2.len());

    // Show icon textures unique to second open
    let set1: std::collections::HashSet<_> = icon_tex1.iter().collect();
    let new_icons: Vec<_> = icon_tex2.iter()
        .filter(|t| !set1.contains(t))
        .collect();
    if !new_icons.is_empty() {
        eprintln!("NEW icon textures on second open: {:?}", &new_icons[..new_icons.len().min(5)]);
    }

    assert_eq!(icon_tex1.len(), icon_tex2.len(),
        "Should have same icon texture count between opens");
}

#[test]
fn spellbook_spell_items_have_nonzero_rect() {
    let env = setup_full_ui();
    open_spellbook(&env);

    let state = env.state().borrow();
    let registry = &state.widgets;
    let item_ids = find_spell_item_ids(registry);
    assert!(!item_ids.is_empty(), "Should have spell items");

    let zero_rect: Vec<_> = item_ids
        .iter()
        .filter_map(|&id| {
            let rect = compute_frame_rect(registry, id, 1024.0, 768.0);
            if rect.width <= 0.0 || rect.height <= 0.0 {
                let f = registry.get(id)?;
                let name = f.name.as_deref().unwrap_or("(anon)");
                Some(format!(
                    "{}[{}] rect={:?} fw={} fh={} anchors={}",
                    name, id, rect, f.width, f.height, f.anchors.len()
                ))
            } else {
                None
            }
        })
        .collect();

    assert!(
        zero_rect.is_empty(),
        "All visible spell items should have non-zero layout rects.\n\
         Zero-rect items ({}):\n{}",
        zero_rect.len(),
        zero_rect.join("\n")
    );
}
