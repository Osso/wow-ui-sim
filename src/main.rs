use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use wow_ui_sim::loader::{load_addon, load_addon_with_saved_vars, LoadTiming};
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

    // Set up font system for text measurement during addon loading
    let font_system = Rc::new(RefCell::new(WowFontSystem::new(&PathBuf::from("./fonts"))));
    env.set_font_system(Rc::clone(&font_system));

    let mut saved_vars = SavedVariablesManager::new();

    let skip_saved_vars = args.no_saved_vars
        || std::env::var("WOW_SIM_NO_SAVED_VARS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    if skip_saved_vars {
        println!("SavedVariables loading disabled");
    } else {
        // Configure WTF loading for character "Haky" on "Burning Blade"
        let wtf_path = PathBuf::from("/syncthing/Sync/Projects/wow/WTF");
        if wtf_path.exists() {
            let wtf_config = WtfConfig::new(
                wtf_path,
                "50868465#2",
                "Burning Blade",
                "Haky",
            );
            println!("WTF config: {} @ {}/{}", wtf_config.account, wtf_config.realm, wtf_config.character);
            println!("  Account SavedVariables: {:?}", wtf_config.account_saved_vars_path());
            println!("  Character SavedVariables: {:?}", wtf_config.character_saved_vars_path());
            saved_vars.set_wtf_config(wtf_config);
        } else {
            println!("SavedVariables storage: {:?}", std::env::var("HOME").map(|h| format!("{}/.local/share/wow-ui-sim/SavedVariables", h)).unwrap_or_default());
        }
    }

    // Load Blizzard SharedXML for base UI support first
    let wow_ui_path = PathBuf::from(env!("HOME"))
        .join("Projects/wow/reference-addons/wow-ui-source");
    if wow_ui_path.exists() {
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
            if toc_path.exists() {
                match load_addon(&env, &toc_path) {
                    Ok(r) => {
                        println!("{} loaded: {} Lua, {} XML, {} warnings", name, r.lua_files, r.xml_files, r.warnings.len());
                        // Print warnings for Blizzard addons
                        for w in &r.warnings {
                            println!("  [!] {}", w);
                        }
                    }
                    Err(e) => println!("{} failed: {}", name, e),
                }
            }
        }
    }

    let skip_addons = args.no_addons
        || std::env::var("WOW_SIM_NO_ADDONS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

    if skip_addons {
        println!("\nAddon loading disabled");
    }

    // Scan and load all addons from reference-addons directory
    let addons_path = PathBuf::from(env!("HOME")).join("Projects/wow/reference-addons");
    let addons = if skip_addons {
        Vec::new()
    } else {
        scan_addons(&addons_path)
    };

    if !addons.is_empty() {
        println!("\n=== Loading {} addons ===\n", addons.len());
    }

    let mut total_lua = 0;
    let mut total_xml = 0;
    let mut total_warnings = 0;
    let mut total_timing = LoadTiming::default();
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut addon_times: Vec<(String, std::time::Duration)> = Vec::new();

    for (name, toc_path) in &addons {
        // Parse TOC to get metadata for addon info
        let toc = TocFile::from_file(toc_path).ok();
        let (title, notes, load_on_demand) = toc.as_ref().map(|t| {
            let title = t.metadata.get("Title").cloned().unwrap_or_else(|| name.clone());
            let notes = t.metadata.get("Notes").cloned().unwrap_or_default();
            let lod = t.metadata.get("LoadOnDemand").map(|v| v == "1").unwrap_or(false);
            (title, notes, lod)
        }).unwrap_or_else(|| (name.clone(), String::new(), false));

        match load_addon_with_saved_vars(&env, toc_path, &mut saved_vars) {
            Ok(r) => {
                // Register addon as loaded
                env.register_addon(AddonInfo {
                    folder_name: name.clone(),
                    title: title.clone(),
                    notes: notes.clone(),
                    enabled: true,
                    loaded: true,
                    load_on_demand,
                });

                let status = if r.warnings.is_empty() { "✓" } else { "⚠" };
                let t = &r.timing;
                println!("{} {} loaded: {} Lua, {} XML, {} warnings ({:.1?} total: io={:.1?} xml={:.1?} lua={:.1?} sv={:.1?})",
                    status, name, r.lua_files, r.xml_files, r.warnings.len(),
                    t.total(), t.io_time, t.xml_parse_time, t.lua_exec_time, t.saved_vars_time);
                addon_times.push((name.clone(), t.total()));
                // Show warnings for specific addons we're working on
                if !r.warnings.is_empty() && (name == "BetterWardrobe" || name == "Plumber" || name == "BetterBlizzFrames" || name == "Baganator" || name == "Angleur" || name == "ExtraQuestButton" || name == "WaypointUI" || name == "TomTom" || name == "WorldQuestTracker" || name == "SavedInstances" || name == "Rarity" || name == "SimpleItemLevel" || name == "TalentLoadoutManager" || name == "Simulationcraft" || name == "TomCats" || name == "RaiderIO" || name == "!BugGrabber" || name == "AdvancedInterfaceOptions" || name == "CraftSim" || name == "BlizzMove_Debug" || name == "ClickableRaidBuffs" || name == "Dejunk" || name == "Cell" || name == "AngryKeystones" || name == "AutoPotion" || name == "BigWigs_Plugins" || name == "BugSack" || name == "Clicked" || name == "DeathNote" || name == "DeModal" || name == "ElvUI_OptionsUI" || name == "DragonRaceTimes" || name == "DynamicCam" || name == "DialogueUI" || name == "Chattynator" || name == "AstralKeys" || name == "Leatrix_Plus" || name == "CooldownToGo_Options" || name == "HousingItemTracker" || name == "idTip" || name == "Macroriffic" || name == "NameplateSCT" || name == "Krowi_ExtendedVendorUI" || name == "OmniCD" || name == "Auctionator" || name == "EditModeExpanded" || name == "GlobalIgnoreList" || name == "AllTheThings" || name == "BigWigs_KhazAlgar" || name == "LegionRemixHelper" || name == "Collectionator" || name == "Syndicator" || name == "BigWigs" || name == "!KalielsTracker" || name == "KRaidSkipTracker" || name == "MacroToolkit" || name == "MinimapButtonButton" || name == "OribosExchange") {
                    for (i, w) in r.warnings.iter().take(10).enumerate() {
                        println!("  [{}] {}", i + 1, w);
                    }
                    if r.warnings.len() > 10 {
                        println!("  ... and {} more", r.warnings.len() - 10);
                    }
                }
                total_lua += r.lua_files;
                total_xml += r.xml_files;
                total_warnings += r.warnings.len();
                total_timing.io_time += r.timing.io_time;
                total_timing.xml_parse_time += r.timing.xml_parse_time;
                total_timing.lua_exec_time += r.timing.lua_exec_time;
                total_timing.saved_vars_time += r.timing.saved_vars_time;
                success_count += 1;
            }
            Err(e) => {
                // Register addon as failed to load
                env.register_addon(AddonInfo {
                    folder_name: name.clone(),
                    title,
                    notes,
                    enabled: true,
                    loaded: false,
                    load_on_demand,
                });

                println!("✗ {} failed: {}", name, e);
                fail_count += 1;
            }
        }
    }

    if !addons.is_empty() {
        println!("\n=== Summary ===");
        println!("Loaded: {}/{} addons", success_count, addons.len());
        println!("Failed: {}", fail_count);
        println!("Total: {} Lua files, {} XML files, {} warnings", total_lua, total_xml, total_warnings);
        let total_time = total_timing.total();
        if !total_time.is_zero() {
            println!("Total time: {:.2?}", total_time);
            println!("  IO:         {:.2?} ({:.1}%)", total_timing.io_time,
                100.0 * total_timing.io_time.as_secs_f64() / total_time.as_secs_f64());
            println!("  XML parse:  {:.2?} ({:.1}%)", total_timing.xml_parse_time,
                100.0 * total_timing.xml_parse_time.as_secs_f64() / total_time.as_secs_f64());
            println!("  Lua exec:   {:.2?} ({:.1}%)", total_timing.lua_exec_time,
                100.0 * total_timing.lua_exec_time.as_secs_f64() / total_time.as_secs_f64());
            println!("  SavedVars:  {:.2?} ({:.1}%)", total_timing.saved_vars_time,
                100.0 * total_timing.saved_vars_time.as_secs_f64() / total_time.as_secs_f64());
        }

        // Show slowest addons
        addon_times.sort_by(|a, b| b.1.cmp(&a.1));
        println!("\nSlowest addons:");
        for (name, time) in addon_times.iter().take(10) {
            println!("  {:>7.1?}  {}", time, name);
        }
    }

    // Test C_AddOns API and check AddonList state
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

        -- Check what AddonList-related globals exist
        print("AddonList type:", type(AddonList))
        print("AddonListMixin type:", type(AddonListMixin))
        print("AddonList_Update type:", type(AddonList_Update))
        print("CreateTreeDataProvider type:", type(CreateTreeDataProvider))
        print("CreateScrollBoxListTreeListView type:", type(CreateScrollBoxListTreeListView))
        print("ScrollUtil type:", type(ScrollUtil))
        "#,
    )?;

    // Initialize and show the AddonList frame
    env.exec(
        r#"
        if AddonList and AddonListMixin then
            -- Initialize the addon list
            local ok, err = pcall(function()
                AddonListMixin.OnLoad(AddonList)
            end)
            if not ok then
                print("[AddonList] OnLoad error:", err)
            end

            -- Show (use XML-defined anchor)
            AddonList:Show()

            -- Update the addon list
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

    // Run debug script if it exists
    let debug_script = PathBuf::from("/tmp/debug-scrollbox-update.lua");
    if debug_script.exists() {
        let script = std::fs::read_to_string(&debug_script)?;
        if let Err(e) = env.exec(&script) {
            println!("[Debug] Script error: {}", e);
        }
    }

    // Run the iced UI
    let debug = wow_ui_sim::DebugOptions {
        borders: args.debug_borders || args.debug_elements,
        anchors: args.debug_anchors || args.debug_elements,
    };
    wow_ui_sim::run_iced_ui(env, debug, Some(saved_vars))?;

    Ok(())
}
