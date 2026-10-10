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
    initialize_and_load_env(args, screen, env)
}

fn initialize_and_load_env(args: &Args, screen: ScreenKind, env: WowLuaEnv) -> InitResult {
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    #[ignore = "private host-budget probe; external no-sound setting and 90s process bound required"]
    fn trusted_finite_budget_for_selected_addons() {
        // Explicit trusted test input, NOT a production default or native policy.
        const TEST_LIMIT: u64 = 100_000_000;
        const OWNERS: [&str; 2] = ["EnhanceQoL", "AllTheThings"];
        run_trusted_finite_budget_probe(&OWNERS, &OWNERS, TEST_LIMIT);
    }

    #[test]
    #[ignore = "private host-budget probe; run alone with external no-sound setting and 90s process bound"]
    fn trusted_finite_budget_for_selected_addons_with_shared_media() {
        // Explicit trusted test input, NOT a production default or native policy.
        const TEST_LIMIT: u64 = 100_000_000;
        const OWNERS: [&str; 3] = ["EnhanceQoL", "AllTheThings", "EnhanceQoLSharedMedia"];
        const EARLY_OWNERS: [&str; 2] = ["EnhanceQoL", "AllTheThings"];
        let env = run_trusted_finite_budget_probe(&OWNERS, &EARLY_OWNERS, TEST_LIMIT);
        // SharedMedia loads during PLAYER_LOGIN, not initial addon loading.
        // Settle ticks may reset usage: inspect actual loaded state instead.
        let sim = env.state().borrow();
        let shared_media = sim
            .addons
            .iter()
            .find(|addon| addon.folder_name == "EnhanceQoLSharedMedia");
        let encountered = shared_media.is_some();
        let loaded = shared_media.is_some_and(|addon| addon.loaded);
        eprintln!(
            "[trusted-budget-probe] phase=after_headless_settle owner=EnhanceQoLSharedMedia encountered={encountered} loaded={loaded}"
        );
        assert!(
            encountered,
            "SharedMedia addon missing from actual addon state"
        );
        assert!(
            loaded,
            "SharedMedia was not loaded after normal login settle"
        );
    }

    fn run_trusted_finite_budget_probe(
        owners: &[&str],
        early_owners: &[&str],
        test_limit: u64,
    ) -> WowLuaEnv {
        assert!(
            std::env::var("WOW_SIM_NO_SOUND")
                .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true")),
            "caller must supply WOW_SIM_NO_SOUND; probe never mutates process environment"
        );
        let args = Args::try_parse_from(["wow-sim", "--no-saved-vars", "dump-tree"])
            .unwrap_or_else(|_| panic!("probe arguments must parse"));
        assert!(args.no_saved_vars);
        assert!(!args.no_addons && !args.skip_addons() && !args.is_test_command());
        assert_eq!(command_screen_size(&args.command), (1600.0, 1200.0));
        let screen = args.effective_screen();
        assert_eq!(screen, ScreenKind::Game);
        let env = WowLuaEnv::new().unwrap_or_else(|_| panic!("probe environment creation failed"));
        env.loader_env()
            .with_state(|state| {
                for &owner in owners {
                    state.set_instruction_budget(owner, Some(test_limit));
                }
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap();

        let snapshot = |env: &WowLuaEnv, phase: &str| {
            env.loader_env()
                .with_state(|state| {
                    for &owner in owners {
                        match state.instruction_budget(owner) {
                            Some(budget) => {
                                eprintln!(
                                    "[trusted-budget-probe] test_input phase={phase} owner={owner} limit={:?} used={}",
                                    budget.limit, budget.used
                                );
                                assert_eq!(budget.limit, Some(test_limit));
                            }
                            None => {
                                eprintln!(
                                    "[trusted-budget-probe] phase={phase} owner={owner} limit=unavailable used=unavailable"
                                );
                                panic!("configured probe owner budget missing");
                            }
                        }
                    }
                    Ok::<_, std::convert::Infallible>(())
                })
                .unwrap();
            // Count recorded quota errors only; never publish messages or payloads.
            let sim = env.state().borrow();
            for &owner in owners {
                let marker = format!("instruction budget exhausted for owner '{owner}'");
                let quota_errors: usize = sim
                    .lua_error_counts
                    .iter()
                    .filter(|(message, _)| message.contains(&marker))
                    .map(|(_, count)| *count)
                    .sum();
                eprintln!(
                    "[trusted-budget-probe] phase={phase} owner={owner} quota_errors={quota_errors}"
                );
            }
        };
        snapshot(&env, "before_init");
        let (env, _font_system, _saved_vars) = initialize_and_load_env(&args, screen, env)
            .unwrap_or_else(|_| panic!("probe initialization pipeline failed"));
        snapshot(&env, "after_load_and_gc");
        // Check real startup consumption before frame ticks can reset owner usage.
        env.loader_env()
            .with_state(|state| {
                for &owner in early_owners {
                    assert!(
                        state
                            .instruction_budget(owner)
                            .is_some_and(|budget| budget.used > 0),
                        "selected probe owner was not encountered in a metered startup scope"
                    );
                }
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap();
        // EXACT helper used by normal run_dump_tree, including its settle ticks.
        super::super::settle_headless_startup(&env);
        snapshot(&env, "after_headless_settle");
        // Reaching this boundary proves pipeline return, not clean startup or native parity.
        env
    }
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
