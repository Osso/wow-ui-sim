//! Test that loading Blizzard addons and firing startup events produces no warnings.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

/// Fire a single event, collecting handler errors.
fn fire(env: &WowLuaEnv, event: &str, args: &[mlua::Value]) -> Vec<String> {
    env.fire_event_collecting_errors(event, args)
}

/// Load all Blizzard addons and fire startup events, collecting all warnings.
fn load_and_startup() -> Vec<String> {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons(&ui);
    let mut warnings = Vec::new();

    // Load addons
    for (name, toc_path) in &addons {
        match load_addon(&env.loader_env(), toc_path) {
            Ok(r) => {
                for w in r.warnings {
                    warnings.push(format!("[load {name}] {w}"));
                }
            }
            Err(e) => {
                warnings.push(format!("[load {name}] FAILED: {e}"));
            }
        }
    }

    // Apply workarounds (same as main.rs run_post_load_scripts)
    env.apply_post_load_workarounds();

    // Fire startup events (same sequence as main.rs)
    fire_startup_events(&env, &mut warnings);

    // Keep only the most recent 500 warnings
    if warnings.len() > 500 {
        warnings.drain(..warnings.len() - 500);
    }

    warnings
}

fn fire_startup_events(env: &WowLuaEnv, warnings: &mut Vec<String>) {
    let lua = env.lua();

    warnings.extend(fire(
        env,
        "ADDON_LOADED",
        &[mlua::Value::String(lua.create_string("WoWUISim").unwrap())],
    ));
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        warnings.extend(fire(env, event, &[]));
    }

    warnings.extend(env.fire_edit_mode_layouts_updated());

    if let Ok(f) = lua.globals().get::<mlua::Function>("RequestTimePlayed") {
        let _ = f.call::<()>(());
    }
    warnings.extend(fire(
        env,
        "PLAYER_ENTERING_WORLD",
        &[mlua::Value::Boolean(true), mlua::Value::Boolean(false)],
    ));
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
        "PLAYER_LEAVING_WORLD",
    ] {
        warnings.extend(fire(env, event, &[]));
    }

    // Fire one OnUpdate tick to catch handler errors
    warnings.extend(env.fire_on_update_collecting_errors(0.016));
}

#[test]
fn test_no_warnings_on_startup() {
    let warnings = load_and_startup();

    if !warnings.is_empty() {
        let mut msg = format!("Unexpected warnings during startup ({}):\n", warnings.len());
        for w in &warnings {
            msg.push_str(&format!("  {w}\n"));
        }
        panic!("{msg}");
    }
}
