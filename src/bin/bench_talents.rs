//! Benchmark binary: load UI then repeatedly open/close talent panel.
//! Opens the talent panel 10 times so it dominates the perf profile.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn main() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    let ui = PathBuf::from("./Interface/BlizzardUI");
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

    // First open demand-loads Blizzard_PlayerSpells
    eprintln!("=== Opening talent panel (first, demand-load) ===");
    let start = std::time::Instant::now();
    env.exec("PlayerSpellsUtil.ToggleClassTalentFrame()")
        .expect("Failed to open talent panel");
    eprintln!("First open: {:.2?}", start.elapsed());

    // Subsequent opens: close then re-open 9 more times
    for i in 2..=10 {
        env.exec("PlayerSpellsUtil.ToggleClassTalentFrame()").ok();
        let start = std::time::Instant::now();
        env.exec("PlayerSpellsUtil.ToggleClassTalentFrame()").ok();
        if i == 2 {
            eprintln!("Subsequent open: {:.2?}", start.elapsed());
        }
    }
    eprintln!("Done (10 opens total)");
}
