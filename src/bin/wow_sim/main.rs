mod addon_loading;
mod cache_texture;
mod exec_lua;
#[cfg(feature = "gui")]
mod gui_commands;
mod saved_var_config;
mod startup;
mod startup_trace;
use cache_texture::run_cache_texture;
use clap::{Parser, Subcommand};
use startup::init_and_load;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use tracing_subscriber::EnvFilter;
use wow_ui_sim::font::WowFontSystem;
use wow_ui_sim::logging;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::SavedVariablesManager;
use wow_ui_sim::screen::ScreenKind;

#[derive(Parser)]
#[command(name = "wow-sim", about = "WoW UI Simulator", version)]
struct Args {
    /// Skip loading WTF Lua SavedVariables (EditMode cache still loads)
    #[arg(long)]
    no_saved_vars: bool,

    /// Skip loading third-party addons
    #[arg(long)]
    no_addons: bool,

    /// Show debug borders and anchor points on all elements
    #[arg(long)]
    debug_elements: bool,

    /// Show red debug borders around all elements
    #[arg(long)]
    debug_borders: bool,

    /// Show green anchor points on all elements
    #[arg(long)]
    debug_anchors: bool,

    /// Delay in milliseconds after firing startup events (for dump-tree/screenshot)
    #[arg(long, value_name = "MS")]
    delay: Option<u64>,

    /// Execute Lua code after startup (runs after first frame in GUI, after events in screenshot/dump-tree).
    /// Prefix with @ to load from file (e.g., --exec-lua @/tmp/debug.lua).
    #[arg(long, value_name = "CODE")]
    exec_lua: Option<String>,

    /// Run `--exec-lua` under secureenv (fenv retargeted via `mark_secure`).
    /// Handy for inspecting/setting secure-scope globals from an ad-hoc
    /// script without staging a TOC-annotated `.lua` file.
    #[arg(long)]
    exec_lua_secure: bool,

    /// Which top-level WoW screen to load.
    #[arg(long, value_enum, default_value_t = ScreenKind::Game, value_name = "SCREEN")]
    screen: ScreenKind,

    /// Compatibility alias for `--screen character-select`.
    #[arg(long, hide = true)]
    character_select: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Load UI and dump frame tree (no GUI needed)
    DumpTree {
        #[arg(short, long)]
        filter: Option<String>,
        #[arg(long)]
        filter_key: Option<String>,
        #[arg(long)]
        visible_only: bool,
        #[arg(short, long)]
        verbose: bool,
        #[arg(long, default_value_t = 1600)]
        width: u32,
        #[arg(long, default_value_t = 1200)]
        height: u32,
    },

    /// Render UI to an image file (no GUI needed)
    #[cfg(feature = "gui")]
    Screenshot {
        #[arg(short, long, default_value = "screenshot.webp")]
        output: PathBuf,
        #[arg(long, default_value_t = 1600)]
        width: u32,
        #[arg(long, default_value_t = 1200)]
        height: u32,
        #[arg(short, long)]
        filter: Option<String>,
        #[arg(long, value_name = "WxH+X+Y")]
        crop: Option<String>,
        #[arg(long, value_name = "FILTER")]
        dump_tree: Option<Option<String>>,
        /// WebP quality, 1-100. Ignored when --output ends in .png, which is
        /// written losslessly.
        #[arg(long, default_value_t = 90.0, value_name = "1-100")]
        quality: f32,
        /// UIParent scale, applied before the canvas size so the UI lays out
        /// for it the way the client does on a uiScale change. The client
        /// renders 1 UI unit as height/768 * uiScale pixels; 1440p at
        /// uiScale 0.9 is 1.6875. Above 1.0 the 2x atlas art is used, as the
        /// client does; WOW_SIM_HIRES_ATLASES=0/1 overrides that choice.
        #[arg(long, default_value_t = 1.0)]
        ui_scale: f32,
    },

    /// Show unique Lua errors as JSON (suppresses other output)
    LuaErrors,

    /// Run simulator self-tests (Wowless test suite)
    SelfTest {
        #[arg(long, default_value_t = 10000)]
        max_ticks: u32,
        #[arg(long)]
        categories: Option<String>,
    },

    /// Run test Lua files from Interface/AddOns/<name>/tests/ or Interface/TestAddOns/<name>/tests/
    RunTests { addon_name: String },

    /// Dump textures used by frames to disk (for debugging atlas crops)
    #[cfg(feature = "gui")]
    DumpTexture {
        #[arg(short, long, default_value = "/tmp/claude/textures")]
        output: PathBuf,
        #[arg(short, long)]
        filter: Option<String>,
        #[arg(long)]
        frame_filter: Option<String>,
    },

    /// Hidden regression helper: boot the GUI App headlessly and click named frames.
    #[cfg(feature = "gui")]
    #[command(hide = true)]
    HeadlessClickProbe {
        #[arg(value_parser = ["achievements", "talents", "mounts", "micromenu"])]
        panel: String,
        #[arg(long, default_value_t = 1024)]
        width: u32,
        #[arg(long, default_value_t = 768)]
        height: u32,
    },

    /// Resolve a WoW texture path through the CASC pipeline and pre-cache it on disk.
    ///
    /// Skips addon loading. Useful to extract a single texture (or retry a previously
    /// failed extraction) without launching the full simulator.
    CacheTexture {
        /// WoW texture path (backslash or forward slash; extension optional)
        path: String,
        /// Delete any existing `.missing` sentinel before retrying extraction
        #[arg(long)]
        force: bool,
    },
}

impl Args {
    fn effective_screen(&self) -> ScreenKind {
        if self.character_select {
            ScreenKind::CharacterSelect
        } else {
            self.screen
        }
    }

    fn is_test_command(&self) -> bool {
        matches!(
            self.command,
            Some(Commands::SelfTest { .. }) | Some(Commands::RunTests { .. })
        )
    }

    fn skip_addons(&self) -> bool {
        self.no_addons
            || std::env::var("WOW_SIM_NO_ADDONS")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
    }
}

fn main() {
    wow_ui_sim::stack::ensure_large_stack();
    startup_trace::print_process_started();
    startup_trace::apply_resource_limits();
    set_cwd_to_exe_dir_for_gui_launch();
    if let Err(error) = run_main() {
        report_fatal_error(error.as_ref());
        std::process::exit(1);
    }
}

fn set_cwd_to_exe_dir_for_gui_launch() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(parent) = exe.parent() else {
        return;
    };
    let _ = std::env::set_current_dir(parent);
}

/// The UI scale a command will render at; 1.0 for commands without one.
fn requested_ui_scale(command: &Option<Commands>) -> f32 {
    match command {
        #[cfg(feature = "gui")]
        Some(Commands::Screenshot { ui_scale, .. }) => *ui_scale,
        _ => 1.0,
    }
}

/// 2x atlas art is what the client draws once a UI unit spans more than one
/// pixel; `WOW_SIM_HIRES_ATLASES` forces either choice.
fn hires_atlases_wanted(ui_scale: f32, env_override: Option<&str>) -> bool {
    match env_override.map(str::trim) {
        Some("0") | Some("false") | Some("off") => false,
        Some("1") | Some("true") | Some("on") => true,
        _ => ui_scale > 1.0,
    }
}

/// Whether the command renders to an image file.
fn is_screenshot_command(command: &Option<Commands>) -> bool {
    match command {
        #[cfg(feature = "gui")]
        Some(Commands::Screenshot { .. }) => true,
        _ => false,
    }
}

/// A capture reproduces atlas texels 1:1 unless `WOW_SIM_BRIGHTNESS_BOOST`
/// asks for the on-screen lift: the `pow(rgb, 1/1.5)` aid lifts dark values
/// most, which flattens the dark outlines of small art (the calendar
/// button's day plate reads blurred with it) and matches no client texel.
/// The live window keeps the environment's choice.
fn screenshot_brightness_divisor(is_screenshot: bool, env_override: Option<&str>) -> Option<f32> {
    (is_screenshot && env_override.is_none()).then_some(1.0)
}

fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if let Some(Commands::CacheTexture { ref path, force }) = args.command {
        return run_cache_texture(path, force);
    }
    let screen = args.effective_screen();
    let saved_stdout = redirect_if_quiet(&args);
    wow_ui_sim::atlas::set_prefer_hires_atlases(hires_atlases_wanted(
        requested_ui_scale(&args.command),
        std::env::var("WOW_SIM_HIRES_ATLASES").ok().as_deref(),
    ));
    wow_ui_sim::render::set_brightness_boost_divisor(screenshot_brightness_divisor(
        is_screenshot_command(&args.command),
        std::env::var("WOW_SIM_BRIGHTNESS_BOOST").ok().as_deref(),
    ));
    let init = init_and_load(&args, screen)?;
    #[cfg(feature = "gui")]
    let (env, font_system, saved_vars) = init;
    #[cfg(not(feature = "gui"))]
    let (env, _font_system, _saved_vars) = init;

    let dispatch = CommandDispatch {
        command: args.command,
        env,
        #[cfg(feature = "gui")]
        font_system,
        delay: args.delay,
        exec_lua: resolve_exec_lua(&args.exec_lua),
        exec_lua_secure: args.exec_lua_secure,
        saved_stdout,
        #[cfg(feature = "gui")]
        saved_vars,
        #[cfg(feature = "gui")]
        debug_borders: args.debug_borders,
        #[cfg(feature = "gui")]
        debug_anchors: args.debug_anchors,
        #[cfg(feature = "gui")]
        debug_elements: args.debug_elements,
    };
    dispatch_command(dispatch)
}

fn redirect_if_quiet(args: &Args) -> Option<i32> {
    let quiet = matches!(
        args.command,
        Some(Commands::LuaErrors) | Some(Commands::SelfTest { .. })
    );
    if quiet {
        wow_ui_sim::lua_errors::redirect_stdout_to_stderr()
    } else {
        None
    }
}

fn apply_post_load_workarounds(env: &WowLuaEnv) {
    wow_ui_sim::logging::println_elapsed("[Startup] applying post-load workarounds");
    let post_load_started = Instant::now();
    env.apply_post_load_workarounds();
    wow_ui_sim::logging::println_elapsed(&format!(
        "[Startup] post-load workarounds complete in {:.2?}",
        post_load_started.elapsed()
    ));
}

fn restart_gc_after_bootstrap(env: &WowLuaEnv) {
    wow_ui_sim::logging::println_elapsed("[Startup] restarting GC after bootstrap");
    let gc_restart_started = Instant::now();
    env.gc_restart_after_bootstrap()
        .expect("post-bootstrap full_gc failed");
    wow_ui_sim::logging::println_elapsed(&format!(
        "[Startup] GC restart complete in {:.2?}",
        gc_restart_started.elapsed()
    ));
}

fn resolve_exec_lua(arg: &Option<String>) -> Option<String> {
    arg.as_ref().map(|s| {
        if let Some(path) = s.strip_prefix('@') {
            std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("[exec-lua] Failed to read {path}: {e}");
                String::new()
            })
        } else {
            s.clone()
        }
    })
}

fn init_sound(env: &WowLuaEnv) {
    let skip = std::env::var("WOW_SIM_NO_SOUND")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if skip {
        logging::println_elapsed("Sound disabled");
        return;
    }
    match wow_ui_sim::sound::SoundManager::new() {
        Some(mgr) => {
            logging::println_elapsed("Sound initialized");
            env.state().borrow_mut().sound_manager = Some(mgr);
        }
        None => logging::println_elapsed("Sound: no audio device available"),
    }
}

fn init_environment(
    args: &Args,
    env: &WowLuaEnv,
    font_system: &Rc<RefCell<WowFontSystem>>,
) -> Result<(), Box<dyn std::error::Error>> {
    logging::init_process_start_time(env.state().borrow().start_time);
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    env.set_font_system(Rc::clone(font_system));
    init_sound(env);
    {
        let mut state = env.state().borrow_mut();
        let mut addon_base_paths =
            vec![runtime_blizzard_ui_addons_path_with_setup().map_err(startup_blizzard_ui_help)?];
        addon_base_paths.extend(wow_ui_sim::paths::default_addons_paths());
        if args.is_test_command() {
            addon_base_paths.push(PathBuf::from("./Interface/TestAddOns"));
        }
        state.addon_base_paths = addon_base_paths;
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    Ok(())
}

fn runtime_blizzard_ui_addons_path_with_setup() -> wow_ui_sim::Result<PathBuf> {
    if let Some(path) = wow_ui_sim::blizzard_ui_sync::cached_blizzard_ui_addons_path() {
        return Ok(path);
    }

    let path = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()?;
    recover_missing_blizzard_ui_addons_path(wow_ui_sim::Error::Other(format!(
        "missing or stale Blizzard UI cache for {:?} profile at {}",
        wow_ui_sim::client_profile::ACTIVE,
        path.display()
    )))
}

fn recover_missing_blizzard_ui_addons_path(
    _missing_cache_error: wow_ui_sim::Error,
) -> wow_ui_sim::Result<PathBuf> {
    logging::println_elapsed("Blizzard UI source missing; syncing Blizzard UI from CASC");
    // Preserve the typed sync error so startup_blizzard_ui_help can pick a
    // tailored message for WowInstallNotFound / BlizzardUiPartial.
    wow_ui_sim::blizzard_ui_sync::sync_blizzard_ui()?;
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
}

fn startup_blizzard_ui_help(error: wow_ui_sim::Error) -> Box<dyn std::error::Error> {
    let message = match &error {
        wow_ui_sim::Error::WowInstallNotFound => format!(
            "Could not find a World of Warcraft installation.\n\n\
             wow-ui-sim needs a local WoW install to load Blizzard UI files via CASC.\n\n\
             Fix:\n  \
             - Install WoW via the Battle.net launcher, OR\n  \
             - Set WOW_INSTALL_PATH to your WoW directory before launching wow-ui-sim.\n\n\
             Default search paths include /Applications/World of Warcraft (macOS), \
             C:\\Program Files (x86)\\World of Warcraft (Windows), and common Wine/Lutris locations on Linux."
        ),
        wow_ui_sim::Error::BlizzardUiPartial {
            missing,
            total,
            last_error,
        } => format!(
            "Your WoW install appears incomplete: {missing} of {total} required files could not be extracted from CASC.\n\
             First missing entry: {last_error}\n\n\
             This usually means your WoW install is partially downloaded or stuck on an older build.\n\n\
             Fix:\n  \
             1. Open Battle.net\n  \
             2. Click the gear icon next to 'Play' on World of Warcraft\n  \
             3. Choose 'Scan and Repair'\n  \
             4. Re-launch wow-ui-sim after the repair completes."
        ),
        _ => format!(
            "{error}\n\nThe simulator stores Blizzard UI source in ~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns and tries to sync it from local WoW CASC automatically. Make sure WoW is installed or set WOW_INSTALL_PATH/WOW_DATA_PATH, or run `wow-cli casc sync-blizzard-ui` after configuring CASC."
        ),
    };
    Box::<dyn std::error::Error>::from(message)
}

fn report_fatal_error(error: &dyn std::error::Error) {
    eprintln!("{error}");
    #[cfg(all(windows, feature = "gui"))]
    show_windows_error_message(&error.to_string());
    #[cfg(all(target_os = "macos", feature = "gui"))]
    show_macos_error_message(&error.to_string());
    #[cfg(all(target_os = "linux", feature = "gui"))]
    show_linux_error_message(&error.to_string());
}

#[cfg(all(target_os = "macos", feature = "gui"))]
fn show_macos_error_message(message: &str) {
    // osascript is preinstalled on every macOS. Escape backslashes and
    // double quotes so the AppleScript string literal stays well-formed.
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        "display alert \"wow-ui-sim startup error\" message \"{escaped}\" as critical buttons {{\"OK\"}}"
    );
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status();
}

#[cfg(all(target_os = "linux", feature = "gui"))]
fn show_linux_error_message(message: &str) {
    // Try zenity (GNOME), then kdialog (KDE), then xmessage. Any of these
    // may be missing on a minimal install; failing silently is fine because
    // stderr already carries the message.
    let title = "wow-ui-sim startup error";
    let attempts: &[(&str, &[&str])] = &[
        ("zenity", &["--error", "--title", title, "--text", message]),
        ("kdialog", &["--error", message, "--title", title]),
        ("xmessage", &["-center", message]),
    ];
    for (cmd, args) in attempts {
        if std::process::Command::new(cmd)
            .args(*args)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return;
        }
    }
}

#[cfg(all(windows, feature = "gui"))]
fn show_windows_error_message(message: &str) {
    use winapi::um::winuser::{MB_ICONERROR, MB_OK, MessageBoxW};

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let title = wide("wow-ui-sim startup error");
    let body = wide(message);
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

use wow_ui_sim::startup::{apply_delay_with_tick, run_extra_update_ticks, settle_headless_startup};

struct CommandDispatch {
    command: Option<Commands>,
    env: WowLuaEnv,
    #[cfg(feature = "gui")]
    font_system: Rc<RefCell<WowFontSystem>>,
    delay: Option<u64>,
    exec_lua: Option<String>,
    exec_lua_secure: bool,
    saved_stdout: Option<i32>,
    #[cfg(feature = "gui")]
    saved_vars: Option<SavedVariablesManager>,
    #[cfg(feature = "gui")]
    debug_borders: bool,
    #[cfg(feature = "gui")]
    debug_anchors: bool,
    #[cfg(feature = "gui")]
    debug_elements: bool,
}

#[cfg(feature = "gui")]
impl CommandDispatch {
    fn debug_options(&self) -> wow_ui_sim::DebugOptions {
        wow_ui_sim::DebugOptions {
            borders: self.debug_borders || self.debug_elements,
            anchors: self.debug_anchors || self.debug_elements,
        }
    }
}

fn dispatch_command(dispatch: CommandDispatch) -> Result<(), Box<dyn std::error::Error>> {
    match dispatch.command {
        Some(Commands::DumpTree { .. }) => dispatch_dump_tree(dispatch),
        #[cfg(feature = "gui")]
        Some(Commands::Screenshot { .. }) => gui_commands::dispatch_screenshot(dispatch),
        Some(Commands::LuaErrors) => run_lua_errors(&dispatch),
        Some(Commands::SelfTest {
            max_ticks,
            ref categories,
        }) => run_self_test(&dispatch, max_ticks, categories.as_deref()),
        Some(Commands::RunTests { ref addon_name }) => {
            run_addon_tests(&dispatch, addon_name);
        }
        #[cfg(feature = "gui")]
        Some(Commands::DumpTexture { .. }) => gui_commands::dispatch_dump_texture(dispatch),
        #[cfg(feature = "gui")]
        Some(Commands::HeadlessClickProbe { .. }) => {
            gui_commands::dispatch_headless_click_probe(dispatch)?
        }
        Some(Commands::CacheTexture { .. }) => {
            unreachable!("CacheTexture is handled before init_and_load");
        }
        #[cfg(feature = "gui")]
        None => return gui_commands::run_gui(dispatch),
        #[cfg(not(feature = "gui"))]
        None => {
            eprintln!("GUI not available (compiled without 'gui' feature).");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn dispatch_dump_tree(dispatch: CommandDispatch) {
    let Some(Commands::DumpTree {
        filter,
        filter_key,
        visible_only,
        verbose,
        width,
        height,
    }) = dispatch.command
    else {
        unreachable!("dispatch_dump_tree only fires for Commands::DumpTree");
    };
    run_dump_tree(
        &dispatch.env,
        DumpTreeCommand {
            filter,
            filter_key,
            visible_only,
            verbose,
            width,
            height,
            delay: dispatch.delay,
            exec_lua: dispatch.exec_lua.as_deref(),
            exec_lua_secure: dispatch.exec_lua_secure,
        },
    );
}

fn run_lua_errors(dispatch: &CommandDispatch) {
    wow_ui_sim::lua_errors::run_lua_errors(
        &dispatch.env,
        dispatch.saved_stdout,
        dispatch.exec_lua.as_deref(),
        dispatch.exec_lua_secure,
    );
}

fn run_self_test(dispatch: &CommandDispatch, max_ticks: u32, categories: Option<&str>) {
    if let Some(c) = categories {
        wow_ui_sim::self_test::inject_category_filter(&dispatch.env, c);
    }
    wow_ui_sim::self_test::run_startup(&dispatch.env);
    wow_ui_sim::self_test::run_test(
        &dispatch.env,
        max_ticks,
        dispatch.exec_lua.as_deref(),
        dispatch.exec_lua_secure,
        dispatch.saved_stdout,
    );
}

fn run_addon_tests(dispatch: &CommandDispatch, addon_name: &str) {
    settle_headless_startup(&dispatch.env);
    wow_ui_sim::addon_tests::run_addon_tests(
        &dispatch.env,
        addon_name,
        dispatch.exec_lua.as_deref(),
        dispatch.exec_lua_secure,
    );
}

struct DumpTreeCommand<'a> {
    filter: Option<String>,
    filter_key: Option<String>,
    visible_only: bool,
    verbose: bool,
    width: u32,
    height: u32,
    delay: Option<u64>,
    exec_lua: Option<&'a str>,
    exec_lua_secure: bool,
}

fn run_dump_tree(env: &WowLuaEnv, command: DumpTreeCommand<'_>) {
    settle_headless_startup(env);
    apply_delay_with_tick(env, command.delay);
    if let Some(code) = command.exec_lua {
        let code = exec_lua::wrap_headless_exec_lua(code);
        if let Err(e) = env.exec_maybe_secure(&code, command.exec_lua_secure) {
            eprintln!("[exec-lua] error: {e}");
        }
    }
    run_extra_update_ticks(env, 3);
    update_dump_layout_rects(env);
    let state = env.state().borrow();
    let addon_names: Vec<String> = state.addons.iter().map(|a| a.folder_name.clone()).collect();
    wow_ui_sim::dump::print_frame_tree(
        &state.widgets,
        &addon_names,
        command.filter.as_deref(),
        command.filter_key.as_deref(),
        command.visible_only,
        command.verbose,
        command.width as f32,
        command.height as f32,
    );
}

#[cfg(feature = "gui")]
fn update_dump_layout_rects(env: &WowLuaEnv) {
    let mut font_system = wow_ui_sim::render::font::WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut font_system);
    state.ensure_layout_rects();
}

#[cfg(not(feature = "gui"))]
fn update_dump_layout_rects(env: &WowLuaEnv) {
    env.state().borrow_mut().ensure_layout_rects();
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn legacy_character_select_flag_maps_to_character_select_screen() {
        let args = Args::try_parse_from(["wow-sim", "--character-select"])
            .expect("legacy character-select flag should parse");
        assert_eq!(args.effective_screen(), ScreenKind::CharacterSelect);
    }

    #[test]
    fn hires_atlases_follow_the_ui_scale_unless_overridden() {
        assert!(!hires_atlases_wanted(1.0, None));
        assert!(hires_atlases_wanted(1.6875, None));
        assert_eq!(screenshot_brightness_divisor(true, None), Some(1.0));
        assert_eq!(screenshot_brightness_divisor(true, Some("1.5")), None);
        assert_eq!(screenshot_brightness_divisor(false, None), None);
        assert!(!hires_atlases_wanted(1.6875, Some("0")));
        assert!(hires_atlases_wanted(1.0, Some("1")));
        assert!(hires_atlases_wanted(1.25, Some("garbage")));
    }

    #[cfg(feature = "gui")]
    #[test]
    fn screenshot_ui_scale_selects_hires_atlases() {
        let args = Args::try_parse_from(["wow-sim", "screenshot", "--ui-scale", "1.6875"])
            .expect("screenshot --ui-scale should parse");
        assert_eq!(requested_ui_scale(&args.command), 1.6875);
        let args = Args::try_parse_from(["wow-sim", "dump-tree"]).expect("dump-tree should parse");
        assert_eq!(requested_ui_scale(&args.command), 1.0);
    }

    #[cfg(feature = "gui")]
    #[test]
    fn screenshot_ui_scale_parses_and_defaults_to_one() {
        let args = Args::try_parse_from(["wow-sim", "screenshot", "--ui-scale", "1.6875"])
            .expect("screenshot --ui-scale should parse");
        let Some(Commands::Screenshot { ui_scale, .. }) = args.command else {
            panic!("expected the screenshot subcommand");
        };
        assert_eq!(ui_scale, 1.6875);

        let args =
            Args::try_parse_from(["wow-sim", "screenshot"]).expect("screenshot should parse");
        let Some(Commands::Screenshot { ui_scale, .. }) = args.command else {
            panic!("expected the screenshot subcommand");
        };
        assert_eq!(ui_scale, 1.0);
    }

    #[test]
    fn explicit_screen_still_parses_character_select() {
        let args = Args::try_parse_from(["wow-sim", "--screen", "character-select"])
            .expect("screen option should parse character-select");
        assert_eq!(args.effective_screen(), ScreenKind::CharacterSelect);
    }

    #[test]
    fn explicit_screen_parses_character_create() {
        let args = Args::try_parse_from(["wow-sim", "--screen", "character-create"])
            .expect("screen option should parse character-create");
        assert_eq!(args.effective_screen(), ScreenKind::CharacterCreate);
    }
}
