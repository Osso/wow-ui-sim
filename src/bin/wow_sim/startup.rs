use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::time::Instant;

use wow_ui_sim::font::WowFontSystem;
use wow_ui_sim::logging;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::SavedVariablesManager;
use wow_ui_sim::screen::ScreenKind;

use super::{
    Args, Commands, addon_loading, apply_post_load_workarounds, init_environment,
    restart_gc_after_bootstrap, saved_var_config, startup_trace,
};

type InitResult = Result<
    (
        WowLuaEnv,
        Rc<RefCell<WowFontSystem>>,
        Option<SavedVariablesManager>,
    ),
    Box<dyn Error>,
>;

pub(super) fn init_and_load(args: &Args, screen: ScreenKind) -> InitResult {
    let env = WowLuaEnv::new().expect("failed to create Lua env");
    configure_screen_size(&env, args);
    let font_system = create_font_system(args);
    init_environment(args, &env, &font_system)?;
    env.set_screen_mode(screen);

    let (mut saved_vars, edit_mode_cache_vars, snapshot_edit_mode_layout) =
        configure_startup_state(args, &env);
    load_startup_addons(
        args,
        screen,
        &env,
        &mut saved_vars,
        edit_mode_cache_vars.as_ref(),
        snapshot_edit_mode_layout.as_deref(),
    );

    restart_gc_after_bootstrap(&env);
    Ok((env, font_system, saved_vars))
}

fn configure_screen_size(env: &WowLuaEnv, args: &Args) {
    let (w, h) = command_screen_size(&args.command);
    let phase_start = Instant::now();
    env.set_screen_size(w, h);
    logging::eprintln_elapsed(&format!(
        "[Startup] screen size set to {w:.0}x{h:.0} in {:.2?}",
        phase_start.elapsed()
    ));
}

fn command_screen_size(command: &Option<Commands>) -> (f32, f32) {
    match command {
        #[cfg(feature = "gui")]
        Some(Commands::Screenshot { width, height, .. }) => (*width as f32, *height as f32),
        Some(Commands::DumpTree { width, height, .. }) => (*width as f32, *height as f32),
        None => (1024.0, 768.0),
        _ => (1600.0, 1200.0),
    }
}

fn create_font_system(args: &Args) -> Rc<RefCell<WowFontSystem>> {
    let phase_start = Instant::now();
    let font_system = Rc::new(RefCell::new(startup_trace::font_system_for_command(args)));
    logging::eprintln_elapsed(&format!(
        "[Startup] font system created in {:.2?}",
        phase_start.elapsed()
    ));
    font_system
}

fn configure_startup_state(
    args: &Args,
    env: &WowLuaEnv,
) -> (
    Option<SavedVariablesManager>,
    Option<SavedVariablesManager>,
    Option<String>,
) {
    // Pause GC across addon loading — addons allocate monotonically
    // (closures + frame tables + registry entries stay live), and we'd
    // rather walk them once in a final full_gc than mark them on every
    // threshold hit.
    startup_trace::time_load_step("stop GC", || env.gc_stop());
    let mut saved_vars = startup_trace::time_load_step("configure saved variables", || {
        saved_var_config::configure_saved_vars(args.no_saved_vars)
    });
    startup_trace::time_load_step("load keybindings", || {
        saved_var_config::load_keybindings_from_wtf(&env, saved_vars.as_ref())
    });
    let edit_mode_cache_vars = if saved_vars.is_none() {
        startup_trace::time_load_step(
            "configure edit mode cache",
            saved_var_config::configure_edit_mode_cache_vars,
        )
    } else {
        None
    };
    let snapshot_edit_mode_layout = read_snapshot_edit_mode_layout(env, &mut saved_vars);
    if let Some(layout) = snapshot_edit_mode_layout.as_deref() {
        logging::println_elapsed(&format!("ServerSnapshot active EditMode layout: {layout}"));
    }
    (saved_vars, edit_mode_cache_vars, snapshot_edit_mode_layout)
}

fn read_snapshot_edit_mode_layout(
    env: &WowLuaEnv,
    saved_vars: &mut Option<SavedVariablesManager>,
) -> Option<String> {
    // ServerSnapshot records the live client's active EditMode layout name.
    // Read it first so the cache loader can select that layout instead of the
    // sometimes stale WTF character-cache index.
    startup_trace::time_load_step("read ServerSnapshot edit mode layout", || {
        let saved_vars = saved_vars.as_mut()?;
        match wow_ui_sim::server_snapshot_import::load_edit_mode_layout(env, saved_vars) {
            Ok(layout) => layout,
            Err(error) => {
                logging::println_elapsed(&format!(
                    "ServerSnapshot edit mode layout import failed: {error}"
                ));
                None
            }
        }
    })
}

fn load_startup_addons(
    args: &Args,
    screen: ScreenKind,
    env: &WowLuaEnv,
    saved_vars: &mut Option<SavedVariablesManager>,
    edit_mode_cache_vars: Option<&SavedVariablesManager>,
    snapshot_edit_mode_layout: Option<&str>,
) {
    startup_trace::time_load_step("load edit mode cache", || {
        addon_loading::load_edit_mode_cache(
            env,
            saved_vars.as_ref().or(edit_mode_cache_vars),
            snapshot_edit_mode_layout,
        )
    });
    load_server_snapshot_state(env, saved_vars);
    startup_trace::time_load_step("load Blizzard addons", || {
        addon_loading::load_blizzard_addons(env, saved_vars, screen)
    });
    startup_trace::time_load_step("prepare chat frame for third-party addons", || {
        wow_ui_sim::lua_api::chat_init::prepare_for_third_party_addons(env)
    });
    #[cfg(any(feature = "client-mists", feature = "retail-12-1-0"))]
    startup_trace::time_load_step("apply post-Blizzard load workarounds", || {
        apply_post_load_workarounds(env)
    });
    startup_trace::time_load_step("load third-party addons", || {
        addon_loading::load_third_party_addons(
            args.skip_addons(),
            args.is_test_command(),
            env,
            saved_vars,
            screen,
        )
    });
    startup_trace::time_load_step("sync addon names to Lua", || env.sync_addon_names_to_lua());
    apply_post_load_workarounds(env);
}

fn load_server_snapshot_state(env: &WowLuaEnv, saved_vars: &mut Option<SavedVariablesManager>) {
    startup_trace::time_load_step("load ServerSnapshot character state", || {
        let Some(saved_vars) = saved_vars.as_mut() else {
            return;
        };
        match wow_ui_sim::server_snapshot_import::load_from_saved_variables(env, saved_vars) {
            Ok(imported) if imported > 0 => logging::println_elapsed(&format!(
                "ServerSnapshot imported {imported} action bar spell slot(s)"
            )),
            Ok(_) => {}
            Err(error) => {
                logging::println_elapsed(&format!("ServerSnapshot import failed: {error}"))
            }
        }
    });
}
