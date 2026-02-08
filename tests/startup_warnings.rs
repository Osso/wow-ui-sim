//! Test that loading Blizzard addons and firing startup events produces no warnings.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

/// Install a Lua error handler that collects errors into `__test_errors`.
fn install_test_error_handler(env: &WowLuaEnv) {
    env.exec(
        r#"
        __test_errors = {}
        seterrorhandler(function(msg)
            table.insert(__test_errors, tostring(msg))
        end)
    "#,
    )
    .expect("Failed to install test error handler");
}

/// Read collected errors from `__test_errors` and clear it.
fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    let lua = env.lua();
    let table: mlua::Table = match lua.globals().get("__test_errors") {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut errors = Vec::new();
    for entry in table.sequence_values::<String>() {
        if let Ok(msg) = entry {
            errors.push(msg);
        }
    }
    // Clear the table for next batch
    let _ = lua.load("__test_errors = {}").exec();
    errors
}

/// Fire a single event, collecting handler errors via the Lua error handler.
fn fire(env: &WowLuaEnv, event: &str, args: &[mlua::Value]) -> Vec<String> {
    env.fire_event_with_args(event, args).ok();
    drain_test_errors(env)
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

    // Install error handler before firing events
    install_test_error_handler(&env);

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

    env.fire_edit_mode_layouts_updated().ok();
    warnings.extend(drain_test_errors(env));

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
    env.fire_on_update(0.016).ok();
    warnings.extend(drain_test_errors(env));
}

/// Known warning count from unimplemented APIs. Update this when adding stubs.
/// Goal: drive this to zero over time by implementing missing APIs.
const KNOWN_WARNING_COUNT: usize = 89;

#[test]
fn test_no_warnings_on_startup() {
    let warnings = load_and_startup();
    let count = warnings.len();

    if count > KNOWN_WARNING_COUNT {
        let mut msg = format!(
            "New warnings introduced! Expected at most {KNOWN_WARNING_COUNT}, got {count}.\n\
             All warnings:\n"
        );
        for w in &warnings {
            msg.push_str(&format!("  {w}\n"));
        }
        panic!("{msg}");
    }

    if count < KNOWN_WARNING_COUNT {
        panic!(
            "Warning count improved from {KNOWN_WARNING_COUNT} to {count}! \
             Update KNOWN_WARNING_COUNT to {count} to lock in the improvement."
        );
    }
}
