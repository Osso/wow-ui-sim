use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_startup_addons_for_screen, load_startup_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::global_frames;
use wow_ui_sim::screen::ScreenKind;

use super::common;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap_or_else(|_| {
        wow_ui_sim::paths::default_blizzard_ui_addons_path()
            .expect("Blizzard UI cache should be available")
    })
}

pub(crate) fn env_with_full_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create env");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    for addon in discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game) {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
            .unwrap_or_else(|err| panic!("load {}: {err}", addon.toc_path.display()));
        env.apply_runtime_addon_load_workarounds(&addon.name);
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env.apply_post_event_workarounds();
    let _ = global_frames::hide_runtime_hidden_frames(&*env.rilua());
    env
}

fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for ev in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(ev);
    }
    common::fire_player_entering_world(env, true, false);
    let _ = env.fire_edit_mode_layouts_updated();
    for ev in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(ev);
    }
}

pub(crate) fn install_test_error_handler(env: &WowLuaEnv) {
    env.exec(
        r#"
        __test_errors = {}
        seterrorhandler(function(msg)
            table.insert(__test_errors, tostring(msg))
        end)
    "#,
    )
    .expect("install error handler");
}

pub(crate) fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}
