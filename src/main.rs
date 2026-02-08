use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use wow_ui_sim::loader::{load_addon, load_addon_with_saved_vars, LoadResult, LoadTiming};
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

    /// Execute Lua code after startup (GUI mode only, runs after first frame)
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

        /// Show only visible frames
        #[arg(long)]
        visible_only: bool,
    },

    /// Render UI to an image file (no GUI needed)
    Screenshot {
        /// Output file path (always lossy WebP at quality 15, extension forced to .webp)
        #[arg(short, long, default_value = "screenshot.webp")]
        output: PathBuf,

        /// Image width in pixels
        #[arg(long, default_value_t = 1024)]
        width: u32,

        /// Image height in pixels
        #[arg(long, default_value_t = 768)]
        height: u32,

        /// Render only this frame subtree (name substring match)
        #[arg(short, long)]
        filter: Option<String>,
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

/// Find the best .toc file for an addon directory (prefer _Mainline.toc for retail)
fn find_toc_file(addon_dir: &Path) -> Option<PathBuf> {
    wow_ui_sim::loader::find_toc_file(addon_dir)
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
                    && let Some(toc) = find_toc_file(&path) {
                        addons.push((name, toc));
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

    // Set addon base paths for runtime on-demand loading (C_AddOns.LoadAddOn)
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![
            PathBuf::from("./Interface/BlizzardUI"),
            PathBuf::from("./Interface/AddOns"),
        ];
    }

    let mut saved_vars = configure_saved_vars(&args);
    load_blizzard_addons(&env);
    load_third_party_addons(&args, &env, &mut saved_vars);
    run_post_load_scripts(&env)?;

    match args.command {
        Some(Commands::DumpTree { filter, visible_only }) => {
            fire_startup_events(&env);
            let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());
            apply_delay(args.delay);
            let state = env.state().borrow();
            wow_ui_sim::dump::print_frame_tree(&state.widgets, filter.as_deref(), visible_only);
        }
        Some(Commands::Screenshot { output, width, height, filter }) => {
            run_screenshot(&env, &font_system, output, width, height, filter, args.delay);
        }
        None => {
            let debug = wow_ui_sim::DebugOptions {
                borders: args.debug_borders || args.debug_elements,
                anchors: args.debug_anchors || args.debug_elements,
            };
            wow_ui_sim::run_iced_ui(env, debug, Some(saved_vars), args.exec_lua)?;
        }
    }

    Ok(())
}

/// Configure SavedVariables from WTF directory based on args/env.
fn configure_saved_vars(args: &Args) -> SavedVariablesManager {
    let mut saved_vars = SavedVariablesManager::new();

    let skip = args.no_saved_vars
        || std::env::var("WOW_SIM_NO_SAVED_VARS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

    if skip {
        println!("SavedVariables loading disabled");
        return saved_vars;
    }

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

    saved_vars
}

/// Blizzard addons loaded in dependency order.
const BLIZZARD_ADDONS: &[(&str, &str)] = &[
    // Foundation (no new deps)
    ("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc"),
    ("Blizzard_Colors", "Blizzard_Colors_Mainline.toc"),
    ("Blizzard_SharedXML", "Blizzard_SharedXML_Mainline.toc"),
    ("Blizzard_SharedXMLGame", "Blizzard_SharedXMLGame_Mainline.toc"),
    ("Blizzard_UIPanelTemplates", "Blizzard_UIPanelTemplates_Mainline.toc"),
    ("Blizzard_FrameXMLBase", "Blizzard_FrameXMLBase_Mainline.toc"),
    // ActionBar dependency chain
    ("Blizzard_LoadLocale", "Blizzard_LoadLocale.toc"),
    ("Blizzard_Fonts_Shared", "Blizzard_Fonts_Shared.toc"),
    ("Blizzard_HelpPlate", "Blizzard_HelpPlate.toc"),
    ("Blizzard_AccessibilityTemplates", "Blizzard_AccessibilityTemplates.toc"),
    ("Blizzard_ObjectAPI", "Blizzard_ObjectAPI_Mainline.toc"),
    ("Blizzard_UIParent", "Blizzard_UIParent_Mainline.toc"),
    ("Blizzard_TextStatusBar", "Blizzard_TextStatusBar.toc"),
    ("Blizzard_MoneyFrame", "Blizzard_MoneyFrame_Mainline.toc"),
    ("Blizzard_POIButton", "Blizzard_POIButton.toc"),
    ("Blizzard_Flyout", "Blizzard_Flyout.toc"),
    ("Blizzard_StoreUI", "Blizzard_StoreUI_Mainline.toc"),
    ("Blizzard_MicroMenu", "Blizzard_MicroMenu_Mainline.toc"),
    ("Blizzard_EditMode", "Blizzard_EditMode.toc"),
    ("Blizzard_GarrisonBase", "Blizzard_GarrisonBase.toc"),
    ("Blizzard_GameTooltip", "Blizzard_GameTooltip_Mainline.toc"),
    ("Blizzard_UIParentPanelManager", "Blizzard_UIParentPanelManager_Mainline.toc"),
    ("Blizzard_Settings_Shared", "Blizzard_Settings_Shared_Mainline.toc"),
    ("Blizzard_SettingsDefinitions_Shared", "Blizzard_SettingsDefinitions_Shared.toc"),
    ("Blizzard_SettingsDefinitions_Frame", "Blizzard_SettingsDefinitions_Frame_Mainline.toc"),
    ("Blizzard_FrameXMLUtil", "Blizzard_FrameXMLUtil_Mainline.toc"),
    ("Blizzard_ItemButton", "Blizzard_ItemButton_Mainline.toc"),
    ("Blizzard_QuickKeybind", "Blizzard_QuickKeybind.toc"),
    ("Blizzard_FrameXML", "Blizzard_FrameXML_Mainline.toc"),
    // UIPanels_Game must load before WorldMap (QuestMapFrame needed by AttachQuestLog)
    ("Blizzard_UIPanels_Game", "Blizzard_UIPanels_Game_Mainline.toc"),
    // WorldMap dependency chain
    ("Blizzard_MapCanvasSecureUtil", "Blizzard_MapCanvasSecureUtil.toc"),
    ("Blizzard_MapCanvas", "Blizzard_MapCanvas.toc"),
    ("Blizzard_SharedMapDataProviders", "Blizzard_SharedMapDataProviders_Mainline.toc"),
    ("Blizzard_WorldMap", "Blizzard_WorldMap_Mainline.toc"),
    ("Blizzard_ActionBar", "Blizzard_ActionBar_Mainline.toc"),
    // Existing UI modules
    ("Blizzard_GameMenu", "Blizzard_GameMenu_Mainline.toc"),
    ("Blizzard_UIWidgets", "Blizzard_UIWidgets_Mainline.toc"),
    ("Blizzard_Minimap", "Blizzard_Minimap_Mainline.toc"),
    ("Blizzard_AddOnList", "Blizzard_AddOnList.toc"),
    // Communities (Guild micro button)
    ("Blizzard_TimerunningUtil", "Blizzard_TimerunningUtil.toc"),
    ("Blizzard_Communities", "Blizzard_Communities_Mainline.toc"),
];

/// Load Blizzard SharedXML and base UI addons.
fn load_blizzard_addons(env: &WowLuaEnv) {
    let blizzard_ui_path = PathBuf::from("./Interface/BlizzardUI");
    if !blizzard_ui_path.exists() {
        return;
    }

    println!("\nLoading Blizzard addons...");
    for (name, toc) in BLIZZARD_ADDONS {
        let toc_path = blizzard_ui_path.join(format!("{}/{}", name, toc));
        if !toc_path.exists() {
            continue;
        }
        match load_addon(&env.loader_env(), &toc_path) {
            Ok(r) => {
                println!("{} loaded: {} Lua, {} XML, {} warnings", name, r.lua_files, r.xml_files, r.warnings.len());
                for w in &r.warnings {
                    println!("  [!] {}", w);
                }
            }
            Err(e) => println!("{} failed: {}", name, e),
        }
    }

}

/// Scan, load, and register third-party addons; print summary.
fn load_third_party_addons(args: &Args, env: &WowLuaEnv, saved_vars: &mut SavedVariablesManager) {
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
    saved_vars: &mut SavedVariablesManager,
    stats: &mut LoadStats,
) {
    let (title, notes, load_on_demand) = parse_addon_metadata(name, toc_path);

    match load_addon_with_saved_vars(&env.loader_env(), toc_path, saved_vars) {
        Ok(r) => {
            let load_time_secs = r.timing.total().as_secs_f64();
            env.register_addon(AddonInfo {
                folder_name: name.to_string(), title: title.clone(),
                notes: notes.clone(), enabled: true, loaded: true, load_on_demand,
                load_time_secs,
            });
            record_addon_success(name, &r, stats);
        }
        Err(e) => {
            env.register_addon(AddonInfo {
                folder_name: name.to_string(), title, notes,
                enabled: true, loaded: false, load_on_demand,
                load_time_secs: 0.0,
            });
            println!("✗ {} failed: {}", name, e);
            stats.fail_count += 1;
        }
    }
}

/// Record a successful addon load: print status and accumulate stats.
fn record_addon_success(name: &str, r: &LoadResult, stats: &mut LoadStats) {
    let status = if r.warnings.is_empty() { "✓" } else { "⚠" };
    let t = &r.timing;
    println!("{} {} loaded: {} Lua, {} XML, {} warnings ({:.1?} total: io={:.1?} xml={:.1?} lua={:.1?} sv={:.1?})",
        status, name, r.lua_files, r.xml_files, r.warnings.len(),
        t.total(), t.io_time, t.xml_parse_time, t.lua_exec_time, t.saved_vars_time);
    stats.addon_times.push((name.to_string(), t.total()));
    print_addon_warnings(name, &r.warnings);
    stats.total_lua += r.lua_files;
    stats.total_xml += r.xml_files;
    stats.total_warnings += r.warnings.len();
    stats.total_timing.io_time += r.timing.io_time;
    stats.total_timing.xml_parse_time += r.timing.xml_parse_time;
    stats.total_timing.lua_exec_time += r.timing.lua_exec_time;
    stats.total_timing.saved_vars_time += r.timing.saved_vars_time;
    stats.success_count += 1;
}

/// Addons whose warnings are shown during loading.
const VERBOSE_WARNING_ADDONS: &[&str] = &[
    "BetterWardrobe", "Plumber", "BetterBlizzFrames", "Baganator", "Angleur",
    "ExtraQuestButton", "WaypointUI", "TomTom", "WorldQuestTracker", "SavedInstances",
    "Rarity", "SimpleItemLevel", "TalentLoadoutManager", "Simulationcraft", "TomCats",
    "RaiderIO", "!BugGrabber", "AdvancedInterfaceOptions", "CraftSim", "BlizzMove_Debug",
    "ClickableRaidBuffs", "Dejunk", "Cell", "AngryKeystones", "AutoPotion",
    "BigWigs_Plugins", "BugSack", "Clicked", "DeathNote", "DeModal",
    "ElvUI_OptionsUI", "DragonRaceTimes", "DynamicCam", "DialogueUI", "Chattynator",
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

/// Sleep for the given number of milliseconds (if specified).
fn apply_delay(delay: Option<u64>) {
    if let Some(ms) = delay {
        eprintln!("[Startup] Delaying {}ms", ms);
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

/// Fire startup events to simulate WoW login sequence.
fn fire_startup_events(env: &WowLuaEnv) {
    let fire = |name| {
        eprintln!("[Startup] Firing {}", name);
        if let Err(e) = env.fire_event(name) {
            eprintln!("Error firing {}: {}", name, e);
        }
    };

    eprintln!("[Startup] Firing ADDON_LOADED");
    if let Err(e) = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(env.lua().create_string("WoWUISim").unwrap())],
    ) {
        eprintln!("Error firing ADDON_LOADED: {}", e);
    }

    fire("VARIABLES_LOADED");
    fire("PLAYER_LOGIN");

    eprintln!("[Startup] Firing EDIT_MODE_LAYOUTS_UPDATED");
    for err in env.fire_edit_mode_layouts_updated() {
        eprintln!("  {}", err);
    }

    eprintln!("[Startup] Firing TIME_PLAYED_MSG via RequestTimePlayed");
    if let Err(e) = env.lua().globals().get::<mlua::Function>("RequestTimePlayed")
        .and_then(|f| f.call::<()>(()))
    {
        eprintln!("Error calling RequestTimePlayed: {}", e);
    }

    eprintln!("[Startup] Firing PLAYER_ENTERING_WORLD");
    if let Err(e) = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[mlua::Value::Boolean(true), mlua::Value::Boolean(false)],
    ) {
        eprintln!("Error firing PLAYER_ENTERING_WORLD: {}", e);
    }

    fire("UPDATE_BINDINGS");
    fire("DISPLAY_SIZE_CHANGED");
    fire("UI_SCALE_CHANGED");
    fire("PLAYER_LEAVING_WORLD");
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
            if let Some(&tid) = c.children_keys.get("Text") {
                if let Some(tf) = state.widgets.get(tid) {
                    eprintln!("      TextFS {tid}: text={:?} {}x{} vis={} strata={:?} lvl={} draw={:?} anch={}",
                        tf.text, tf.width, tf.height, tf.visible, tf.frame_strata,
                        tf.frame_level, tf.draw_layer, tf.anchors.len());
                }
            }
        }
    }
}

/// Render a headless screenshot.
fn run_screenshot(
    env: &WowLuaEnv,
    font_system: &Rc<RefCell<WowFontSystem>>,
    output: PathBuf,
    width: u32,
    height: u32,
    filter: Option<String>,
    delay: Option<u64>,
) {
    use wow_ui_sim::iced_app::build_quad_batch_for_registry;
    use wow_ui_sim::render::software::render_to_image;
    use wow_ui_sim::render::GlyphAtlas;

    env.set_screen_size(width as f32, height as f32);
    fire_startup_events(env);
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());
    debug_show_game_menu(env);
    apply_delay(delay);

    let mut glyph_atlas = GlyphAtlas::new();
    let batch = {
        let state = env.state().borrow();
        let mut fs = font_system.borrow_mut();
        build_quad_batch_for_registry(
            &state.widgets,
            (width as f32, height as f32),
            filter.as_deref(), None, None,
            Some((&mut fs, &mut glyph_atlas)),
        )
    };

    eprintln!(
        "QuadBatch: {} quads, {} texture requests",
        batch.quad_count(), batch.texture_requests.len()
    );

    let mut tex_mgr = create_texture_manager();

    let glyph_data = if glyph_atlas.is_dirty() {
        let (data, size, _) = glyph_atlas.texture_data();
        Some((data, size))
    } else {
        None
    };

    let img = render_to_image(&batch, &mut tex_mgr, width, height, glyph_data);
    let output = output.with_extension("webp");
    save_screenshot(&img, &output);
    eprintln!("Saved {}x{} screenshot to {}", width, height, output.display());
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
