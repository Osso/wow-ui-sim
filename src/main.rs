use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use wow_ui_sim::loader::{load_addon, load_addon_with_saved_vars, LoadTiming};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::{SavedVariablesManager, WtfConfig};

/// Find the best .toc file for an addon directory (prefer _Mainline.toc for retail)
fn find_toc_file(addon_dir: &PathBuf) -> Option<PathBuf> {
    let addon_name = addon_dir.file_name()?.to_str()?;

    // Priority order for retail WoW
    let toc_variants = [
        format!("{}_Mainline.toc", addon_name),
        format!("{}.toc", addon_name),
    ];

    for variant in &toc_variants {
        let toc_path = addon_dir.join(variant);
        if toc_path.exists() {
            return Some(toc_path);
        }
    }

    // Fallback: find any .toc file
    if let Ok(entries) = std::fs::read_dir(addon_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toc").unwrap_or(false) {
                // Skip non-mainline variants
                let name = path.file_name().unwrap().to_str().unwrap();
                if !name.contains("_Cata") && !name.contains("_Wrath") &&
                   !name.contains("_TBC") && !name.contains("_Vanilla") &&
                   !name.contains("_Mists") {
                    return Some(path);
                }
            }
        }
    }

    None
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
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let env = WowLuaEnv::new()?;
    let mut saved_vars = SavedVariablesManager::new();

    // Check if SavedVariables loading is disabled (for faster startup)
    // Set WOW_SIM_NO_SAVED_VARS=1 to skip loading WTF files
    let skip_saved_vars = std::env::var("WOW_SIM_NO_SAVED_VARS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if skip_saved_vars {
        println!("SavedVariables loading disabled (WOW_SIM_NO_SAVED_VARS set)");
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
            ("Blizzard_SharedXML", "Blizzard_SharedXML_Mainline.toc"),
            ("Blizzard_GameMenu", "Blizzard_GameMenu_Mainline.toc"),
            ("Blizzard_UIWidgets", "Blizzard_UIWidgets_Mainline.toc"),
            ("Blizzard_FrameXMLBase", "Blizzard_FrameXMLBase.toc"),
        ];

        for (name, toc) in blizzard_addons {
            let toc_path = wow_ui_path.join(format!("Interface/AddOns/{}/{}", name, toc));
            if toc_path.exists() {
                match load_addon(&env, &toc_path) {
                    Ok(r) => println!("{} loaded: {} Lua, {} XML, {} warnings", name, r.lua_files, r.xml_files, r.warnings.len()),
                    Err(e) => println!("{} failed: {}", name, e),
                }
            }
        }
    }

    // Check if addon loading is disabled (for faster startup during texture testing)
    // Set WOW_SIM_NO_ADDONS=1 to skip loading third-party addons
    let skip_addons = std::env::var("WOW_SIM_NO_ADDONS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if skip_addons {
        println!("\nAddon loading disabled (WOW_SIM_NO_ADDONS set)");
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
        match load_addon_with_saved_vars(&env, toc_path, &mut saved_vars) {
            Ok(r) => {
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

    // Create a single WoW-style button centered on screen
    env.exec(
        r#"
        -- Hide ALL existing frames first
        local function hideAllFrames()
            local kids = {UIParent:GetChildren()}
            for _, child in ipairs(kids) do
                if child and child.Hide then
                    child:Hide()
                end
            end
        end
        hideAllFrames()

        -- Single WoW button with proper textures
        local btn = CreateFrame("Button", "TestButton", UIParent)
        btn:SetSize(128, 32)
        btn:SetPoint("CENTER", 0, 0)
        btn:SetText("Click Me")
        btn:EnableMouse(true)
        btn:Show()

        -- Set WoW button textures
        btn:SetNormalTexture("Interface\\Buttons\\UI-Panel-Button-Up")
        btn:SetPushedTexture("Interface\\Buttons\\UI-Panel-Button-Down")
        btn:SetHighlightTexture("Interface\\Buttons\\UI-Panel-Button-Highlight")

        btn:SetScript("OnClick", function() print("Button clicked!") end)
        btn:SetScript("OnEnter", function(self) self:SetText("> Click <") end)
        btn:SetScript("OnLeave", function(self) self:SetText("Click Me") end)
        "#,
    )?;

    // Run the GTK UI
    wow_ui_sim::run_gtk_ui(env)?;

    Ok(())
}
