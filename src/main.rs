use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon, load_addon_with_saved_vars, LoadResult, LoadTiming};
use wow_ui_sim::lua_api::{AddonInfo, WowLuaEnv};
use wow_ui_sim::render::WowFontSystem;
use wow_ui_sim::saved_variables::{SavedVariablesManager, WtfConfig};
use wow_ui_sim::toc::TocFile;

#[derive(Parser)]
#[command(name = "wow-sim", about = "WoW UI Simulator")]
struct Args {
    /// Skip loading WTF SavedVariables (faster startup)
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

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Load UI and dump frame tree (no GUI needed)
    DumpTree {
        /// Filter by frame name (substring match)
        #[arg(short, long)]
        filter: Option<String>,

        /// Filter by parentKey name (prints full subtree of matching frames)
        #[arg(long)]
        filter_key: Option<String>,

        /// Show only visible frames
        #[arg(long)]
        visible_only: bool,
        /// Screen width for layout computation
        #[arg(long, default_value_t = 1600)]
        width: u32,
        /// Screen height for layout computation
        #[arg(long, default_value_t = 1200)]
        height: u32,
    },

    /// Render UI to an image file (no GUI needed)
    Screenshot {
        /// Output file path (always lossy WebP at quality 15, extension forced to .webp)
        #[arg(short, long, default_value = "screenshot.webp")]
        output: PathBuf,

        /// Image width in pixels
        #[arg(long, default_value_t = 1600)]
        width: u32,

        /// Image height in pixels
        #[arg(long, default_value_t = 1200)]
        height: u32,

        /// Render only this frame subtree (name substring match)
        #[arg(short, long)]
        filter: Option<String>,

        /// Crop the output image to WxH+X+Y (e.g., 700x150+400+650)
        #[arg(long, value_name = "WxH+X+Y")]
        crop: Option<String>,
    },
}

/// Apply resource limits to prevent runaway memory/CPU usage.
/// Defaults: 10GB memory, 1 CPU core.
fn apply_resource_limits() {
    // Memory limit via RLIMIT_AS (virtual address space)
    let max_mem_bytes: u64 = std::env::var("WOW_SIM_MAX_MEM_GB")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(10)
        * 1024
        * 1024
        * 1024;

    let mem_limit = libc::rlimit {
        rlim_cur: max_mem_bytes,
        rlim_max: max_mem_bytes,
    };
    unsafe {
        libc::setrlimit(libc::RLIMIT_AS, &mem_limit);
    }

    // CPU core limit via sched_setaffinity
    let max_cores: usize = std::env::var("WOW_SIM_MAX_CORES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);

    unsafe {
        let mut cpuset: libc::cpu_set_t = std::mem::zeroed();
        for i in 0..max_cores {
            libc::CPU_SET(i, &mut cpuset);
        }
        libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpuset);
    }

    println!(
        "Resource limits: {}GB memory, {} CPU core(s)",
        max_mem_bytes / 1024 / 1024 / 1024,
        max_cores
    );
}

/// Scan addons directory and return sorted list of addon directories
fn scan_addons(base_path: &PathBuf) -> Vec<(String, PathBuf)> {
    let mut addons = Vec::new();

    if let Ok(entries) = std::fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                // Skip hidden directories and special directories
                let skip = name.starts_with('.')
                    || name == "BlizzardUI";
                if !skip
                    && let Some(toc_path) = wow_ui_sim::loader::find_toc_file(&path)
                        && let Ok(toc) = TocFile::from_file(&toc_path)
                            && !toc.is_glue_only() && !toc.is_ptr_only() && !toc.is_game_type_restricted() {
                                addons.push((name, toc_path));
                            }
            }
        }
    }

    // Sort alphabetically
    addons.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    addons
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    apply_resource_limits();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let env = WowLuaEnv::new()?;
    let font_system = Rc::new(RefCell::new(WowFontSystem::new(&PathBuf::from("./fonts"))));
    env.set_font_system(Rc::clone(&font_system));

    // Initialize sound manager (skip with WOW_SIM_NO_SOUND=1)
    init_sound(&env);

    // Set addon base paths for runtime on-demand loading (C_AddOns.LoadAddOn)
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![
            PathBuf::from("./Interface/BlizzardUI"),
            PathBuf::from("./Interface/AddOns"),
        ];
    }

    // Register synthetic templates for C++ intrinsic frame types (WoWScrollBoxList, etc.)
    // before any addons that reference them are loaded.
    wow_ui_sim::xml::register_intrinsic_templates();

    let mut saved_vars = configure_saved_vars(&args);
    load_blizzard_addons(&env);
    load_third_party_addons(&args, &env, &mut saved_vars);
    run_post_load_scripts(&env)?;

    let exec_lua = resolve_exec_lua(&args.exec_lua);

    match args.command {
        Some(Commands::DumpTree { filter, filter_key, visible_only, width, height }) => {
            fire_startup_events(&env);
            env.apply_post_event_workarounds();
            env.state().borrow_mut().widgets.rebuild_anchor_index();
            process_pending_timers(&env);
            fire_one_on_update_tick(&env);
            let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());
            if let Some(code) = &exec_lua
                && let Err(e) = env.exec(code) {
                    eprintln!("[exec-lua] error: {e}");
                }
            run_extra_update_ticks(&env, 3);
            apply_delay(args.delay);
            let state = env.state().borrow();
            wow_ui_sim::dump::print_frame_tree(&state.widgets, filter.as_deref(), filter_key.as_deref(), visible_only, width as f32, height as f32);
        }
        Some(Commands::Screenshot { output, width, height, filter, crop }) => {
            run_screenshot(&env, &font_system, output, width, height, filter, crop, args.delay, exec_lua.as_deref());
        }
        None => {
            let debug = wow_ui_sim::DebugOptions {
                borders: args.debug_borders || args.debug_elements,
                anchors: args.debug_anchors || args.debug_elements,
            };
            wow_ui_sim::run_iced_ui(env, debug, saved_vars, exec_lua)?;
        }
    }

    Ok(())
}

/// Resolve exec-lua argument: if prefixed with `@`, read the file contents.
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

/// Configure SavedVariables from WTF directory based on args/env.
fn configure_saved_vars(args: &Args) -> Option<SavedVariablesManager> {
    let skip = args.no_saved_vars
        || std::env::var("WOW_SIM_NO_SAVED_VARS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

    if skip {
        println!("SavedVariables loading disabled");
        return None;
    }

    let mut saved_vars = SavedVariablesManager::new();

    let wtf_path = PathBuf::from("/syncthing/Sync/Projects/wow/WTF");
    if wtf_path.exists() {
        let wtf_config = WtfConfig::new(wtf_path, "50868465#2", "Burning Blade", "Haky");
        println!("WTF config: {} @ {}/{}", wtf_config.account, wtf_config.realm, wtf_config.character);
        println!("  Account SavedVariables: {:?}", wtf_config.account_saved_vars_path());
        println!("  Character SavedVariables: {:?}", wtf_config.character_saved_vars_path());
        saved_vars.set_wtf_config(wtf_config);
    } else {
        println!("SavedVariables storage: {:?}", std::env::var("HOME")
            .map(|h| format!("{}/.local/share/wow-sim/SavedVariables", h)).unwrap_or_default());
    }

    Some(saved_vars)
}

/// Initialize sound manager unless WOW_SIM_NO_SOUND=1 or --no-sound.
fn init_sound(env: &WowLuaEnv) {
    let skip = std::env::var("WOW_SIM_NO_SOUND")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if skip {
        println!("Sound disabled");
        return;
    }

    let sound_dir = PathBuf::from("./sounds");
    match wow_ui_sim::sound::SoundManager::new(sound_dir) {
        Some(mgr) => {
            println!("Sound initialized");
            env.state().borrow_mut().sound_manager = Some(mgr);
        }
        None => {
            println!("Sound: no audio device available");
        }
    }
}

/// Load Blizzard SharedXML and base UI addons (auto-discovered, dependency-sorted).
fn load_blizzard_addons(env: &WowLuaEnv) {
    let blizzard_ui_path = PathBuf::from("./Interface/BlizzardUI");
    if !blizzard_ui_path.exists() {
        return;
    }

    let addons = discover_blizzard_addons(&blizzard_ui_path);
    let verbose = std::env::var("WOW_SIM_VERBOSE").is_ok();
    println!("\nLoading {} Blizzard addons...", addons.len());
    let blizzard_start = std::time::Instant::now();
    let mut total_timing = LoadTiming::default();
    for (name, toc_path) in &addons {
        match load_addon(&env.loader_env(), toc_path) {
            Ok(r) => {
                if verbose {
                    println!("{} loaded: {} Lua, {} XML, {} warnings", name, r.lua_files, r.xml_files, r.warnings.len());
                }
                for w in &r.warnings {
                    println!("  [!] {}", w);
                }
                total_timing.io_time += r.timing.io_time;
                total_timing.xml_parse_time += r.timing.xml_parse_time;
                total_timing.lua_exec_time += r.timing.lua_exec_time;
                total_timing.cache_hits += r.timing.cache_hits;
                total_timing.cache_misses += r.timing.cache_misses;
            }
            Err(e) => println!("{} failed: {}", name, e),
        }
        // Blizzard_EnvironmentCleanup nils secure C_* namespaces that are normally
        // provided by the C++ engine in a separate secure environment. Re-register
        // the stubs our sim needs after that addon runs.
        if name == "Blizzard_EnvironmentCleanup" {
            wow_ui_sim::lua_api::globals::restore_secure_stubs(env);
        }
    }
    let elapsed = blizzard_start.elapsed();
    let cache_total = total_timing.cache_hits + total_timing.cache_misses;
    let cache_info = if cache_total > 0 {
        format!(", bytecode cache: {}/{} hits", total_timing.cache_hits, cache_total)
    } else {
        String::new()
    };
    println!(
        "Blizzard addons loaded in {elapsed:.2?} (io={:.2?} xml={:.2?} lua={:.2?}{cache_info})",
        total_timing.io_time, total_timing.xml_parse_time, total_timing.lua_exec_time
    );
}

/// Scan, load, and register third-party addons; print summary.
fn load_third_party_addons(args: &Args, env: &WowLuaEnv, saved_vars: &mut Option<SavedVariablesManager>) {
    let skip_addons = args.no_addons
        || std::env::var("WOW_SIM_NO_ADDONS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

    if skip_addons {
        println!("\nAddon loading disabled");
        return;
    }

    let addons_path = PathBuf::from("./Interface/AddOns");
    let addons = scan_addons(&addons_path);

    if addons.is_empty() {
        return;
    }

    println!("\n=== Loading {} addons ===\n", addons.len());

    let mut stats = LoadStats::default();

    for (name, toc_path) in &addons {
        load_single_addon(env, name, toc_path, saved_vars, &mut stats);
    }

    print_load_summary(&addons, &stats);
}

/// Accumulated statistics from loading addons.
#[derive(Default)]
struct LoadStats {
    total_lua: usize,
    total_xml: usize,
    total_warnings: usize,
    total_timing: LoadTiming,
    success_count: usize,
    fail_count: usize,
    addon_times: Vec<(String, std::time::Duration)>,
    cache_hits: u32,
    cache_misses: u32,
}

/// Load a single third-party addon and update stats.
/// Parse TOC metadata for an addon.
fn parse_addon_metadata(name: &str, toc_path: &Path) -> (String, String, bool) {
    let toc = TocFile::from_file(toc_path).ok();
    toc.as_ref().map(|t| {
        let title = t.metadata.get("Title").cloned().unwrap_or_else(|| name.to_string());
        let notes = t.metadata.get("Notes").cloned().unwrap_or_default();
        let lod = t.metadata.get("LoadOnDemand").map(|v| v == "1").unwrap_or(false);
        (title, notes, lod)
    }).unwrap_or_else(|| (name.to_string(), String::new(), false))
}

/// Load a single third-party addon and update stats.
fn load_single_addon(
    env: &WowLuaEnv,
    name: &str,
    toc_path: &Path,
    saved_vars: &mut Option<SavedVariablesManager>,
    stats: &mut LoadStats,
) {
    let (title, notes, load_on_demand) = parse_addon_metadata(name, toc_path);

    // Pre-register so loading_addon_index attributes frames to this addon.
    env.register_addon(AddonInfo {
        folder_name: name.to_string(), title, notes,
        enabled: true, loaded: false, load_on_demand, ..Default::default()
    });

    let result = match saved_vars.as_mut() {
        Some(sv) => load_addon_with_saved_vars(&env.loader_env(), toc_path, sv),
        None => load_addon(&env.loader_env(), toc_path),
    };
    match result {
        Ok(r) => {
            let t = r.timing.total().as_secs_f64();
            let mut s = env.state().borrow_mut();
            if let Some(a) = s.addons.iter_mut().find(|a| a.folder_name == name) {
                a.loaded = true;
                a.load_time_secs = t;
            }
            drop(s);
            record_addon_success(name, &r, stats);
        }
        Err(e) => { println!("✗ {} failed: {}", name, e); stats.fail_count += 1; }
    }
}

/// Record a successful addon load: print status and accumulate stats.
fn record_addon_success(name: &str, r: &LoadResult, stats: &mut LoadStats) {
    if std::env::var("WOW_SIM_VERBOSE").is_ok() {
        let status = if r.warnings.is_empty() { "✓" } else { "⚠" };
        let t = &r.timing;
        println!("{} {} loaded: {} Lua, {} XML, {} warnings ({:.1?} total: io={:.1?} xml={:.1?} lua={:.1?} sv={:.1?})",
            status, name, r.lua_files, r.xml_files, r.warnings.len(),
            t.total(), t.io_time, t.xml_parse_time, t.lua_exec_time, t.saved_vars_time);
    }
    stats.addon_times.push((name.to_string(), r.timing.total()));
    print_addon_warnings(name, &r.warnings);
    stats.total_lua += r.lua_files;
    stats.total_xml += r.xml_files;
    stats.total_warnings += r.warnings.len();
    stats.total_timing.io_time += r.timing.io_time;
    stats.total_timing.xml_parse_time += r.timing.xml_parse_time;
    stats.total_timing.lua_exec_time += r.timing.lua_exec_time;
    stats.total_timing.saved_vars_time += r.timing.saved_vars_time;
    stats.cache_hits += r.timing.cache_hits;
    stats.cache_misses += r.timing.cache_misses;
    stats.success_count += 1;
}

/// Addons whose warnings are shown during loading.
const VERBOSE_WARNING_ADDONS: &[&str] = &[
    "BetterWardrobe", "Plumber", "BetterBlizzFrames", "Baganator", "Angleur", "ExtraQuestButton",
    "WaypointUI", "TomTom", "WorldQuestTracker", "SavedInstances", "Rarity", "SimpleItemLevel",
    "TalentLoadoutManager", "Simulationcraft", "TomCats", "RaiderIO", "!BugGrabber",
    "AdvancedInterfaceOptions", "CraftSim", "BlizzMove_Debug", "ClickableRaidBuffs", "Dejunk",
    "Cell", "AngryKeystones", "AutoPotion", "BigWigs_Plugins", "BugSack", "Clicked", "DeathNote",
    "DeModal", "ElvUI_OptionsUI", "DragonRaceTimes", "DynamicCam", "DialogueUI", "Chattynator",
    "AstralKeys", "Leatrix_Plus", "CooldownToGo_Options", "HousingItemTracker", "idTip",
    "Macroriffic", "NameplateSCT", "Krowi_ExtendedVendorUI", "OmniCD", "Auctionator",
    "EditModeExpanded", "GlobalIgnoreList", "AllTheThings", "BigWigs_KhazAlgar",
    "LegionRemixHelper", "Collectionator", "Syndicator", "BigWigs", "!KalielsTracker",
    "KRaidSkipTracker", "MacroToolkit", "MinimapButtonButton", "OribosExchange",
];

/// Print warnings for addons in the verbose list.
fn print_addon_warnings(name: &str, warnings: &[String]) {
    if warnings.is_empty() || !VERBOSE_WARNING_ADDONS.contains(&name) {
        return;
    }
    for (i, w) in warnings.iter().take(10).enumerate() {
        println!("  [{}] {}", i + 1, w);
    }
    if warnings.len() > 10 {
        println!("  ... and {} more", warnings.len() - 10);
    }
}

/// Print loading summary with timing breakdown and slowest addons.
fn print_load_summary(addons: &[(String, PathBuf)], stats: &LoadStats) {
    println!("\n=== Summary ===");
    println!("Loaded: {}/{} addons", stats.success_count, addons.len());
    println!("Failed: {}", stats.fail_count);
    println!("Total: {} Lua files, {} XML files, {} warnings",
        stats.total_lua, stats.total_xml, stats.total_warnings);

    let total_time = stats.total_timing.total();
    if !total_time.is_zero() {
        let pct = |d: std::time::Duration| 100.0 * d.as_secs_f64() / total_time.as_secs_f64();
        println!("Total time: {:.2?}", total_time);
        println!("  IO:         {:.2?} ({:.1}%)", stats.total_timing.io_time, pct(stats.total_timing.io_time));
        println!("  XML parse:  {:.2?} ({:.1}%)", stats.total_timing.xml_parse_time, pct(stats.total_timing.xml_parse_time));
        println!("  Lua exec:   {:.2?} ({:.1}%)", stats.total_timing.lua_exec_time, pct(stats.total_timing.lua_exec_time));
        println!("  SavedVars:  {:.2?} ({:.1}%)", stats.total_timing.saved_vars_time, pct(stats.total_timing.saved_vars_time));
    }

    if stats.cache_hits > 0 || stats.cache_misses > 0 {
        let total = stats.cache_hits + stats.cache_misses;
        let pct = 100.0 * stats.cache_hits as f64 / total as f64;
        println!("Bytecode cache: {}/{} hits ({:.0}%)", stats.cache_hits, total, pct);
    }

    let mut sorted_times = stats.addon_times.clone();
    sorted_times.sort_by(|a, b| b.1.cmp(&a.1));
    println!("\nSlowest addons:");
    for (name, time) in sorted_times.iter().take(10) {
        println!("  {:>7.1?}  {}", time, name);
    }
}

/// Run post-load Lua test scripts and debug hooks.
fn run_post_load_scripts(env: &WowLuaEnv) -> Result<(), Box<dyn std::error::Error>> {
    env.apply_post_load_workarounds();

    let debug_script = PathBuf::from("/tmp/debug-scrollbox-update.lua");
    if debug_script.exists() {
        let script = std::fs::read_to_string(&debug_script)?;
        if let Err(e) = env.exec(&script) {
            println!("[Debug] Script error: {}", e);
        }
    }

    Ok(())
}

/// Process pending timers (deferred template wiring, addon callbacks, etc.).
///
/// In GUI mode timers fire every frame, but headless paths (screenshot, dump-tree)
/// must explicitly drain them. Loops until no more timers fire (handles chaining).
use wow_ui_sim::startup::{
    apply_delay, fire_one_on_update_tick, fire_startup_events, process_pending_timers,
};

/// Fire extra OnUpdate ticks so deferred UI (talent frame, pool-created frames) can process.
fn run_extra_update_ticks(env: &WowLuaEnv, n: usize) {
    for _ in 0..n {
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(env);
        process_pending_timers(env);
    }
}

/// Debug: open game menu via micro button click for screenshot testing.
fn debug_show_game_menu(env: &WowLuaEnv) {
    if std::env::var("WOW_SIM_SHOW_GAME_MENU").is_err() {
        return;
    }
    if let Err(e) = env.exec(r#"
        local btn = MainMenuMicroButton
        if btn then
            local onclick = btn:GetScript("OnClick")
            if onclick then onclick(btn, "LeftButton", false) end
        end
    "#) {
        eprintln!("[debug_game_menu] click error: {e}");
    }
    // Check what SetText resolves to for a game menu button
    if let Err(e) = env.exec(r#"if GameMenuFrame and GameMenuFrame.buttonPool then
        for button in GameMenuFrame.buttonPool:EnumerateActive() do
            local text, st = button:GetText() or "(nil)", button.SetText
            io.stderr:write(("[lua_debug] text=%q type(SetText)=%s\n"):format(text, type(st)))
            if type(st) == "function" then
                local info = debug.getinfo(st, "S")
                io.stderr:write(("[lua_debug] SetText source=%s\n"):format(info and info.source or "unknown"))
            end
            break
        end end"#) { eprintln!("[debug_game_menu] lua debug error: {e}"); }
    dump_game_menu_buttons(env);
}

fn dump_game_menu_buttons(env: &WowLuaEnv) {
    use wow_ui_sim::widget::WidgetType;
    let state = env.state().borrow();
    let gmf_id = state.widgets.get_id_by_name("GameMenuFrame");
    eprintln!("[debug] GameMenuFrame id={gmf_id:?}");
    let Some(gmf_id) = gmf_id else { return };
    let Some(gmf) = state.widgets.get(gmf_id) else { return };
    eprintln!("  vis={} strata={:?} lvl={} {}x{} children={}",
        gmf.visible, gmf.frame_strata, gmf.frame_level, gmf.width, gmf.height, gmf.children.len());
    // Show all children, not just buttons
    for (i, &cid) in gmf.children.iter().enumerate() {
        let Some(c) = state.widgets.get(cid) else { continue };
        let nm = c.name.as_deref().unwrap_or("(anon)");
        eprintln!("  [{i}] {cid} {nm} [{:?}] {}x{} strata={:?} lvl={} vis={} text={:?}",
            c.widget_type, c.width, c.height, c.frame_strata, c.frame_level, c.visible, c.text);
        if c.widget_type == WidgetType::Button {
            eprintln!("      font={:?} fsz={} color={:?}",
                c.font, c.font_size, c.text_color);
            if let Some(&tid) = c.children_keys.get("Text")
                && let Some(tf) = state.widgets.get(tid) {
                    eprintln!("      TextFS {tid}: text={:?} {}x{} vis={} strata={:?} lvl={} draw={:?} anch={}",
                        tf.text, tf.width, tf.height, tf.visible, tf.frame_strata,
                        tf.frame_level, tf.draw_layer, tf.anchors.len());
                }
        }
    }
}

/// Parse a crop string in WxH+X+Y format (e.g., "700x150+400+650").
/// Returns (width, height, x, y) or None if the format is invalid.
fn parse_crop(s: &str) -> Option<(u32, u32, u32, u32)> {
    let (dims, rest) = s.split_once('+')?;
    let (x_str, y_str) = rest.split_once('+')?;
    let (w_str, h_str) = dims.split_once('x')?;
    let w = w_str.parse().ok()?;
    let h = h_str.parse().ok()?;
    let x = x_str.parse().ok()?;
    let y = y_str.parse().ok()?;
    Some((w, h, x, y))
}

/// Apply crop to an image, exiting on invalid input.
fn apply_crop(img: image::RgbaImage, crop_str: &str) -> image::RgbaImage {
    use image::GenericImageView;
    let (cw, ch, cx, cy) = parse_crop(crop_str).unwrap_or_else(|| {
        eprintln!("Invalid crop format '{}', expected WxH+X+Y (e.g., 700x150+400+650)", crop_str);
        std::process::exit(1);
    });
    if cx + cw > img.width() || cy + ch > img.height() {
        eprintln!(
            "Crop region {}x{}+{}+{} exceeds image bounds {}x{}",
            cw, ch, cx, cy, img.width(), img.height()
        );
        std::process::exit(1);
    }
    img.view(cx, cy, cw, ch).to_image()
}

/// Build the quad batch and glyph atlas for headless rendering.
fn build_screenshot_batch(
    env: &WowLuaEnv,
    font_system: &Rc<RefCell<WowFontSystem>>,
    width: u32,
    height: u32,
    filter: Option<&str>,
) -> (wow_ui_sim::render::QuadBatch, wow_ui_sim::render::GlyphAtlas) {
    use wow_ui_sim::iced_app::build_quad_batch_for_registry;
    use wow_ui_sim::render::GlyphAtlas;
    let mut glyph_atlas = GlyphAtlas::new();
    let batch = {
        let mut fs = font_system.borrow_mut();
        let buckets = {
            let mut state = env.state().borrow_mut();
            state.ensure_layout_rects();
            wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut fs);
            let _ = state.get_strata_buckets();
            state.strata_buckets.as_ref().unwrap().clone()
        };
        let state = env.state().borrow();
        let tooltip_data = wow_ui_sim::iced_app::tooltip::collect_tooltip_data(&state);
        build_quad_batch_for_registry(
            &state.widgets,
            (width as f32, height as f32),
            filter, None, None,
            Some((&mut fs, &mut glyph_atlas)),
            Some(&state.message_frames),
            Some(&tooltip_data),
            &buckets,
        )
    };
    (batch, glyph_atlas)
}

/// Render a headless screenshot.
#[allow(clippy::too_many_arguments)]
fn run_screenshot(
    env: &WowLuaEnv,
    font_system: &Rc<RefCell<WowFontSystem>>,
    output: PathBuf,
    width: u32,
    height: u32,
    filter: Option<String>,
    crop: Option<String>,
    delay: Option<u64>,
    exec_lua: Option<&str>,
) {
    use wow_ui_sim::render::headless::render_to_image;

    env.set_screen_size(width as f32, height as f32);
    fire_startup_events(env);
    env.apply_post_event_workarounds();
    env.state().borrow_mut().widgets.rebuild_anchor_index();
    process_pending_timers(env);
    fire_one_on_update_tick(env);
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());
    debug_show_game_menu(env);
    if let Some(code) = exec_lua
        && let Err(e) = env.exec(code) {
            eprintln!("[exec-lua] error: {e}");
        }
    run_extra_update_ticks(env, 3);
    apply_delay(delay);
    let (batch, glyph_atlas) = build_screenshot_batch(env, font_system, width, height, filter.as_deref());
    eprintln!("QuadBatch: {} quads, {} texture requests", batch.quad_count(), batch.texture_requests.len());

    let mut tex_mgr = create_texture_manager();
    let glyph_data = if glyph_atlas.is_dirty() {
        let (data, size, _) = glyph_atlas.texture_data();
        Some((data, size))
    } else {
        None
    };

    let img = render_to_image(&batch, &mut tex_mgr, width, height, glyph_data);
    let img = match crop.as_deref() {
        Some(crop_str) => apply_crop(img, crop_str),
        None => img,
    };

    let output = output.with_extension("webp");
    save_screenshot(&img, &output);
    eprintln!("Saved {}x{} screenshot to {}", img.width(), img.height(), output.with_extension("webp").display());
}

/// Save screenshot image as lossy WebP (quality 15). Extension is forced to .webp.
fn save_screenshot(img: &image::RgbaImage, output: &std::path::Path) {
    let output = output.with_extension("webp");
    let encoder = webp::Encoder::from_rgba(img.as_raw(), img.width(), img.height());
    let mem = encoder.encode(15.0);
    if let Err(e) = std::fs::write(&output, &*mem) {
        eprintln!("Failed to save WebP: {}", e);
        std::process::exit(1);
    }
}

/// Create a TextureManager with local and fallback texture paths.
fn create_texture_manager() -> wow_ui_sim::texture::TextureManager {
    use wow_ui_sim::texture::TextureManager;

    let home = dirs::home_dir().unwrap_or_default();
    let local_textures = PathBuf::from("./textures");
    let textures_path = if local_textures.exists() {
        local_textures
    } else {
        home.join("Repos/wow-ui-textures")
    };
    TextureManager::new(textures_path)
        .with_interface_path(home.join("Projects/wow/Interface"))
        .with_addons_path(PathBuf::from("./Interface/AddOns"))
}

