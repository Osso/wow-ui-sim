use std::path::{Path, PathBuf};

use wow_ui_sim::loader::{
    BlizzardAddonOverride, discover_blizzard_addon_closure_for_screen_with_overrides, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

use super::panel_fixtures::{blizzard_ui_dir, clear_recorded_lua_errors, load_panel_addons};

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

pub(super) fn load_discovered_blizzard_addons_into_env(
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

fn new_blizzard_addon_env_for_screen(ui_dir: &Path, screen: ScreenKind) -> WowLuaEnv {
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

pub fn with_blizzard_addon_closure_in_dir<R, F>(
    ui_dir: &Path,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
    assertions: F,
) -> R
where
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    let (env, loaded) = build_blizzard_addon_closure_env(ui_dir, roots, overrides);
    assertions(&env, &loaded)
}

pub fn with_blizzard_addon_closure<R, F>(
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
    assertions: F,
) -> R
where
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    with_blizzard_addon_closure_in_dir(&blizzard_ui_dir(), roots, overrides, assertions)
}

pub fn with_blizzard_addon_startup_shape_in_dir<R, F>(
    ui_dir: &Path,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
    assertions: F,
) -> R
where
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    let env = new_blizzard_addon_env(ui_dir);
    load_panel_addons(&env);
    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }
    let loaded = load_blizzard_addon_closure_into_env(&env, ui_dir, roots, overrides);
    ensure_player_frame_stub(&env);
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    assertions(&env, &loaded)
}

pub fn with_blizzard_addon_startup_shape<R, F>(
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
    assertions: F,
) -> R
where
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    with_blizzard_addon_startup_shape_in_dir(&blizzard_ui_dir(), roots, overrides, assertions)
}

fn ensure_player_frame_stub(env: &WowLuaEnv) {
    env.exec(
        r#"
        if not PlayerFrame then
            PlayerFrame = CreateFrame("Frame", "PlayerFrame", UIParent)
        end
        PlayerFrame.unit = "player"
        "#,
    )
    .expect("failed to create PlayerFrame stub for startup smoke tests");
}

pub fn with_blizzard_addon_smoke_shape_in_dir<R, F>(
    ui_dir: &Path,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
    assertions: F,
) -> R
where
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    let env = new_blizzard_addon_env(ui_dir);
    load_panel_addons(&env);
    clear_recorded_lua_errors(&env);
    let loaded = load_blizzard_addon_closure_into_env(&env, ui_dir, roots, overrides);
    assertions(&env, &loaded)
}

pub fn with_blizzard_addon_smoke_shape<R, F>(
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
    assertions: F,
) -> R
where
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    with_blizzard_addon_smoke_shape_in_dir(&blizzard_ui_dir(), roots, overrides, assertions)
}

pub fn new_blizzard_addon_glue_env(ui_dir: &Path) -> WowLuaEnv {
    new_blizzard_addon_env_for_screen(ui_dir, ScreenKind::CharacterSelect)
}

// Glue counterpart of `with_blizzard_addon_smoke_shape`. Used by addons whose
// TOC carries `## AllowLoad: Glue` — those are filtered out by the game-screen
// closure walker (`allows_screen(Game)` is false for `Glue`), so the
// game-shape harness can never reach them. This variant pins the env to
// `ScreenKind::CharacterSelect`, walks the dependency closure with the same
// glue screen, and loads each addon in topological order.
//
// No "panel-addons" baseline is loaded first: the closure walker already
// pulls in `Blizzard_GlueXML` and its transitive deps from `roots` when the
// root TOC lists `## Dependencies: Blizzard_GlueXML`, so loading anything
// extra would either duplicate work or fight the dependency order.
pub fn with_blizzard_addon_glue_smoke_shape_in_dir<R, F>(
    ui_dir: &Path,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
    assertions: F,
) -> R
where
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    let env = new_blizzard_addon_glue_env(ui_dir);
    clear_recorded_lua_errors(&env);
    let loaded = load_blizzard_addon_closure_for_screen_into_env(
        &env,
        ui_dir,
        ScreenKind::CharacterSelect,
        roots,
        overrides,
    );
    assertions(&env, &loaded)
}

pub fn with_blizzard_addon_glue_smoke_shape<R, F>(
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
    assertions: F,
) -> R
where
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    with_blizzard_addon_glue_smoke_shape_in_dir(&blizzard_ui_dir(), roots, overrides, assertions)
}

type HarnessAssertions = fn(&WowLuaEnv, &[String]);

// Each integration test compiles this shared helper module as its own crate
// module, so some helpers are unused in any single target even though they are
// part of the cross-test harness API.
const _: () = {
    let _ = load_blizzard_addon_closure_into_env
        as for<'a> fn(&WowLuaEnv, &Path, &[&str], &[BlizzardAddonOverride<'a>]) -> Vec<String>;
    let _ = load_blizzard_addon_closure_for_screen_into_env
        as for<'a> fn(
            &WowLuaEnv,
            &Path,
            ScreenKind,
            &[&str],
            &[BlizzardAddonOverride<'a>],
        ) -> Vec<String>;
    let _ = build_blizzard_addon_closure_env
        as for<'a> fn(&Path, &[&str], &[BlizzardAddonOverride<'a>]) -> (WowLuaEnv, Vec<String>);
    let _ = new_blizzard_addon_env as fn(&Path) -> WowLuaEnv;
    let _ = with_blizzard_addon_closure_in_dir::<(), HarnessAssertions>
        as for<'a> fn(&Path, &[&str], &[BlizzardAddonOverride<'a>], HarnessAssertions);
    let _ = with_blizzard_addon_closure::<(), HarnessAssertions>
        as for<'a> fn(&[&str], &[BlizzardAddonOverride<'a>], HarnessAssertions);
    let _ = with_blizzard_addon_startup_shape_in_dir::<(), HarnessAssertions>
        as for<'a> fn(&Path, &[&str], &[BlizzardAddonOverride<'a>], HarnessAssertions);
    let _ = with_blizzard_addon_startup_shape::<(), HarnessAssertions>
        as for<'a> fn(&[&str], &[BlizzardAddonOverride<'a>], HarnessAssertions);
    let _ = with_blizzard_addon_smoke_shape_in_dir::<(), HarnessAssertions>
        as for<'a> fn(&Path, &[&str], &[BlizzardAddonOverride<'a>], HarnessAssertions);
    let _ = with_blizzard_addon_smoke_shape::<(), HarnessAssertions>
        as for<'a> fn(&[&str], &[BlizzardAddonOverride<'a>], HarnessAssertions);
    let _ = new_blizzard_addon_glue_env as fn(&Path) -> WowLuaEnv;
    let _ = with_blizzard_addon_glue_smoke_shape_in_dir::<(), HarnessAssertions>
        as for<'a> fn(&Path, &[&str], &[BlizzardAddonOverride<'a>], HarnessAssertions);
    let _ = with_blizzard_addon_glue_smoke_shape::<(), HarnessAssertions>
        as for<'a> fn(&[&str], &[BlizzardAddonOverride<'a>], HarnessAssertions);
};
