use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use wow_ui_sim::loader::{load_addon, load_addon_with_saved_vars, LoadResult, LoadTiming};
use wow_ui_sim::lua_api::{AddonInfo, WowLuaEnv};
use wow_ui_sim::render::WowFontSystem;
use wow_ui_sim::saved_variables::{SavedVariablesManager, WtfConfig};
use wow_ui_sim::toc::TocFile;

#[derive(Parser)]
#[command(name = "wow-ui-sim", about = "WoW UI Simulator")]
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
fn find_toc_file(addon_dir: &PathBuf) -> Option<PathBuf> {
    wow_ui_sim::loader::find_toc_file(addon_dir)
}

/// Scan reference-addons directory and return sorted list of addon directories
fn scan_addons(base_path: &PathBuf) -> Vec<(String, PathBuf)> {
    let mut addons = Vec::new();

    if let Ok(entries) = std::fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                // Skip hidden directories and special directories
                let skip = name.starts_with('.')
                    || name == "wow-ui-source";
                if !skip {
                    if let Some(toc) = find_toc_file(&path) {
                        addons.push((name, toc));
                    }
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

    let mut saved_vars = configure_saved_vars(&args);
    load_blizzard_addons(&env);
    load_third_party_addons(&args, &env, &mut saved_vars);
    run_post_load_scripts(&env)?;

    let debug = wow_ui_sim::DebugOptions {
        borders: args.debug_borders || args.debug_elements,
        anchors: args.debug_anchors || args.debug_elements,
    };
    wow_ui_sim::run_iced_ui(env, debug, Some(saved_vars))?;

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
            .map(|h| format!("{}/.local/share/wow-ui-sim/SavedVariables", h)).unwrap_or_default());
    }

    saved_vars
}

/// Load Blizzard SharedXML and base UI addons.
fn load_blizzard_addons(env: &WowLuaEnv) {
    let wow_ui_path = PathBuf::from(env!("HOME"))
        .join("Projects/wow/reference-addons/wow-ui-source");
    if !wow_ui_path.exists() {
        return;
    }

    let blizzard_addons = [
        ("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc"),
        ("Blizzard_Colors", "Blizzard_Colors_Mainline.toc"),
        ("Blizzard_SharedXML", "Blizzard_SharedXML_Mainline.toc"),
        ("Blizzard_SharedXMLGame", "Blizzard_SharedXMLGame_Mainline.toc"),
        ("Blizzard_UIPanelTemplates", "Blizzard_UIPanelTemplates_Mainline.toc"),
        ("Blizzard_GameMenu", "Blizzard_GameMenu_Mainline.toc"),
        ("Blizzard_UIWidgets", "Blizzard_UIWidgets_Mainline.toc"),
        ("Blizzard_FrameXMLBase", "Blizzard_FrameXMLBase.toc"),
        ("Blizzard_AddOnList", "Blizzard_AddOnList.toc"),
    ];

    for (name, toc) in blizzard_addons {
        let toc_path = wow_ui_path.join(format!("Interface/AddOns/{}/{}", name, toc));
        if !toc_path.exists() {
            continue;
        }
        match load_addon(env, &toc_path) {
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

    let addons_path = PathBuf::from(env!("HOME")).join("Projects/wow/reference-addons");
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
fn parse_addon_metadata(name: &str, toc_path: &PathBuf) -> (String, String, bool) {
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
    toc_path: &PathBuf,
    saved_vars: &mut SavedVariablesManager,
    stats: &mut LoadStats,
) {
    let (title, notes, load_on_demand) = parse_addon_metadata(name, toc_path);

    match load_addon_with_saved_vars(env, toc_path, saved_vars) {
        Ok(r) => {
            env.register_addon(AddonInfo {
                folder_name: name.to_string(), title: title.clone(),
                notes: notes.clone(), enabled: true, loaded: true, load_on_demand,
            });
            record_addon_success(name, &r, stats);
        }
        Err(e) => {
            env.register_addon(AddonInfo {
                folder_name: name.to_string(), title, notes,
                enabled: true, loaded: false, load_on_demand,
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
    env.exec(
        r#"
        local num = C_AddOns.GetNumAddOns()
        print("C_AddOns.GetNumAddOns() =", num)
        if num > 0 then
            for i = 1, math.min(5, num) do
                local name, title = C_AddOns.GetAddOnInfo(i)
                print(string.format("  [%d] %s - %s", i, tostring(name), tostring(title)))
            end
        end

        print("AddonList type:", type(AddonList))
        print("AddonListMixin type:", type(AddonListMixin))
        print("AddonList_Update type:", type(AddonList_Update))
        print("CreateTreeDataProvider type:", type(CreateTreeDataProvider))
        print("CreateScrollBoxListTreeListView type:", type(CreateScrollBoxListTreeListView))
        print("ScrollUtil type:", type(ScrollUtil))
        "#,
    )?;

    env.exec(
        r#"
        if AddonList and AddonListMixin then
            local ok, err = pcall(function()
                AddonListMixin.OnLoad(AddonList)
            end)
            if not ok then
                print("[AddonList] OnLoad error:", err)
            end
            AddonList:Show()
            if AddonList_Update then
                local updateOk, updateErr = pcall(AddonList_Update)
                if updateOk then
                    print("[AddonList] Initialized and updated successfully")
                else
                    print("[AddonList] Update error:", updateErr)
                end
            end
        else
            print("[AddonList] AddonList or AddonListMixin not found")
        end
        "#,
    )?;

    let debug_script = PathBuf::from("/tmp/debug-scrollbox-update.lua");
    if debug_script.exists() {
        let script = std::fs::read_to_string(&debug_script)?;
        if let Err(e) = env.exec(&script) {
            println!("[Debug] Script error: {}", e);
        }
    }

    Ok(())
}
