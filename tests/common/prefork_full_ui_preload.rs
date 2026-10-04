use std::path::Path;

use wow_ui_sim::loader::{
    StartupAddonLoadKind, discover_blizzard_startup_addons_for_screen, load_addon,
    load_startup_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

pub(crate) fn frame_is_shown(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil and {frame_name}:IsShown() == true");
    env.eval::<bool>(&code).unwrap_or(false)
}

pub(crate) fn install_test_error_handler(env: &WowLuaEnv) {
    env.exec(
        r#"
        _G.__test_errors = {}
        seterrorhandler(function(msg)
            local trace = ""
            if type(debugstack) == "function" then
                trace = "\n" .. tostring(debugstack())
            end
            table.insert(_G.__test_errors, tostring(msg) .. trace)
        end)
        "#,
    )
    .expect("Failed to install test error handler");
}

pub(crate) fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    const SEP: char = '\u{1f}';
    let joined = env
        .eval::<String>(
            r#"
            local values = _G.__test_errors
            if type(values) ~= "table" then
                return ""
            end
            local parts = {}
            for i = 1, #values do
                parts[#parts + 1] = tostring(values[i])
            end
            _G.__test_errors = {}
            return table.concat(parts, string.char(31))
            "#,
        )
        .unwrap_or_default();
    if joined.is_empty() {
        return Vec::new();
    }
    joined.split(SEP).map(|entry| entry.to_string()).collect()
}

pub(crate) fn preload_full_game_ui() -> Result<WowLuaEnv, String> {
    wow_ui_sim::loader::enter_bytecode_cache_parent_bypass_mode();
    let env = WowLuaEnv::new().map_err(|error| format!("create full-UI environment: {error}"))?;
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    let blizzard_ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .map_err(|error| format!("resolve synced Blizzard UI addon path: {error}"))?;
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui.clone()];
    env.gc_stop();

    // Same discovery as the wow-sim Game startup, including LoD bootstrap-only nodes.
    for addon in discover_blizzard_startup_addons_for_screen(&blizzard_ui, ScreenKind::Game) {
        match addon.kind {
            StartupAddonLoadKind::Full => load_blizzard_addon(&env, &addon.name, &addon.toc_path)?,
            StartupAddonLoadKind::BootstrapOnly => {
                load_blizzard_bootstrap(&env, &addon.name, &addon.toc_path)?
            }
        }
    }

    env.sync_string_metatable_to_global_string();
    apply_post_load_workarounds(&env)?;
    env.gc_restart_after_bootstrap()
        .map_err(|error| format!("restart bootstrap GC: {error}"))?;
    fire_startup_events(&env)?;
    wow_ui_sim::loader::release_prefork_parent_bytecode_cache_memory()
        .map_err(|error| format!("release prefork parent bytecode cache memory: {error}"))?;
    Ok(env)
}

pub(crate) fn load_blizzard_addon(
    env: &WowLuaEnv,
    name: &str,
    toc_path: &Path,
) -> Result<(), String> {
    let error_count = lua_error_count(env);
    load_addon(&env.loader_env(), toc_path).map_err(|error| {
        format!(
            "load Blizzard addon `{name}` from {}: {error}",
            toc_path.display()
        )
    })?;
    ensure_no_new_lua_errors(
        env,
        error_count,
        &format!("loading Blizzard addon `{name}`"),
    )?;

    let error_count = lua_error_count(env);
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(name)])
        .map_err(|error| format!("fire ADDON_LOADED for `{name}`: {error}"))?;
    ensure_no_new_lua_errors(
        env,
        error_count,
        &format!("firing ADDON_LOADED for `{name}`"),
    )?;

    if name == "Blizzard_EnvironmentCleanup" {
        env.loader_env()
            .restore_post_cleanup_globals()
            .map_err(|error| {
                format!("restore post-cleanup globals after Blizzard addon `{name}`: {error}")
            })?;
    }
    Ok(())
}

fn load_blizzard_bootstrap(env: &WowLuaEnv, name: &str, toc_path: &Path) -> Result<(), String> {
    let error_count = lua_error_count(env);
    load_startup_addon(
        &env.loader_env(),
        toc_path,
        StartupAddonLoadKind::BootstrapOnly,
        None,
    )
    .map_err(|error| {
        format!(
            "load bootstrap of Blizzard addon `{name}` from {}: {error}",
            toc_path.display()
        )
    })?;
    ensure_no_new_lua_errors(
        env,
        error_count,
        &format!("loading bootstrap of Blizzard addon `{name}`"),
    )
}

fn apply_post_load_workarounds(env: &WowLuaEnv) -> Result<(), String> {
    let error_count = lua_error_count(env);
    env.apply_post_load_workarounds();
    ensure_no_new_lua_errors(env, error_count, "applying post-load workarounds")
}

fn fire_startup_events(env: &WowLuaEnv) -> Result<(), String> {
    let error_count = lua_error_count(env);
    fire_startup_events_for_screen(env, ScreenKind::Game);
    ensure_no_new_lua_errors(env, error_count, "firing game startup events")
}

fn lua_error_count(env: &WowLuaEnv) -> usize {
    env.state().borrow().lua_errors.len()
}

fn ensure_no_new_lua_errors(
    env: &WowLuaEnv,
    previous_count: usize,
    operation: &str,
) -> Result<(), String> {
    let state = env.state().borrow();
    let new_errors = &state.lua_errors[previous_count..];
    if new_errors.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{operation} produced {} Lua error(s):\n{}",
        new_errors.len(),
        new_errors.join("\n")
    ))
}
