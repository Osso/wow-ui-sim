//! Dependency-closure loading without panel, startup, or process fixtures.

use std::path::{Path, PathBuf};

use wow_ui_sim::loader::{
    BlizzardAddonOverride, discover_blizzard_addon_closure_for_screen_with_overrides, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

pub fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

pub fn load_blizzard_addon_closure_into_env(
    env: &WowLuaEnv,
    ui_dir: &Path,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
) -> Vec<String> {
    load_blizzard_addon_closure_for_screen_into_env(env, ui_dir, ScreenKind::Game, roots, overrides)
}

pub fn load_blizzard_addon_closure_for_screen_into_env(
    env: &WowLuaEnv,
    ui_dir: &Path,
    screen: ScreenKind,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
) -> Vec<String> {
    let addons =
        discover_blizzard_addon_closure_for_screen_with_overrides(ui_dir, screen, roots, overrides);
    load_discovered_blizzard_addons_into_env(env, addons)
}

pub fn load_discovered_blizzard_addons_into_env(
    env: &WowLuaEnv,
    addons: Vec<(String, PathBuf)>,
) -> Vec<String> {
    let mut loaded = Vec::new();
    for (name, toc_path) in addons {
        if !is_addon_loaded(env, &name) {
            if let Err(error) = load_addon(&env.loader_env(), &toc_path) {
                panic!("{name} should load in the Blizzard addon closure harness: {error}");
            }
        }
        loaded.push(name);
    }
    loaded
}

fn is_addon_loaded(env: &WowLuaEnv, name: &str) -> bool {
    env.state()
        .borrow()
        .addons
        .iter()
        .any(|addon| addon.folder_name == name && addon.loaded)
}

pub fn build_blizzard_addon_closure_env(
    ui_dir: &Path,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
) -> (WowLuaEnv, Vec<String>) {
    let env = new_blizzard_addon_env(ui_dir);
    let loaded = load_blizzard_addon_closure_into_env(&env, ui_dir, roots, overrides);
    (env, loaded)
}

pub fn new_blizzard_addon_env(ui_dir: &Path) -> WowLuaEnv {
    new_blizzard_addon_env_for_screen(ui_dir, ScreenKind::Game)
}

pub(super) fn new_blizzard_addon_env_for_screen(ui_dir: &Path, screen: ScreenKind) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(screen);

    let ui = ui_dir.to_path_buf();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui];
    }

    env
}
