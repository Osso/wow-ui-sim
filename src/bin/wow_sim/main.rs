//! WoW UI Simulator CLI
//!
//! Multi-purpose CLI for the WoW UI simulator.
//!
//! Usage:
//!   wow-sim lua                      # Interactive Lua REPL
//!   wow-sim lua -e "print('hi')"     # Execute code and exit
//!   wow-sim extract-textures         # Extract textures to WebP

mod dump;

use clap::{Parser, Subcommand};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_server::client;

#[derive(Parser)]
#[command(name = "wow-sim")]
#[command(about = "WoW UI Simulator CLI tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lua REPL - connect to running wow-ui-sim and execute Lua code
    Lua {
        /// Execute code and exit
        #[arg(short = 'e', long)]
        exec: Option<String>,

        /// Execute file and exit
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,

        /// List running servers
        #[arg(short = 'l', long)]
        list: bool,
    },

    /// Extract textures referenced by addons to WebP format
    ExtractTextures {
        /// Path to addons directory to scan
        #[arg(long, default_value_os_t = default_addons_path())]
        addons: PathBuf,

        /// Path to WoW Interface directory (for BLP textures)
        #[arg(long, default_value_os_t = default_interface_path())]
        interface: PathBuf,

        /// Output directory for WebP textures
        #[arg(long, short, default_value = "./textures")]
        output: PathBuf,
    },

    /// Dump the rendered frame tree with absolute coordinates (requires running server)
    DumpTree {
        /// Filter by frame name (substring match)
        #[arg(short, long)]
        filter: Option<String>,

        /// Show only visible frames
        #[arg(long)]
        visible_only: bool,
    },

    /// Load UI and dump frame tree (standalone, no server needed)
    Dump {
        /// Filter by frame name (substring match)
        #[arg(short, long)]
        filter: Option<String>,

        /// Show only visible frames
        #[arg(long)]
        visible_only: bool,

        /// Skip loading third-party addons (faster startup)
        #[arg(long)]
        no_addons: bool,

        /// Skip loading SavedVariables (faster startup)
        #[arg(long)]
        no_saved_vars: bool,
    },

    /// Convert a BLP texture file to WebP format
    ConvertTexture {
        /// Input BLP file path
        input: PathBuf,

        /// Output WebP file path (defaults to input with .webp extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Render UI to an image file (standalone, no GUI needed)
    Screenshot {
        /// Output file path (lossy WebP at quality 15 by default; png/jpg detected from extension)
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

        /// Skip loading third-party addons (faster startup)
        #[arg(long)]
        no_addons: bool,

        /// Skip loading SavedVariables (faster startup)
        #[arg(long)]
        no_saved_vars: bool,
    },
}

fn default_addons_path() -> PathBuf {
    PathBuf::from("./Interface/AddOns")
}

fn default_interface_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join("Projects/wow/Interface")
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lua { exec, file, list } => {
            if list {
                list_servers();
            } else if let Some(code) = exec {
                execute_and_exit(&code);
            } else if let Some(path) = file {
                execute_file_and_exit(&path);
            } else {
                run_repl();
            }
        }
        Commands::ExtractTextures {
            addons,
            interface,
            output,
        } => {
            let (found, missing) =
                wow_ui_sim::extract_textures::extract_textures(&addons, &interface, &output);
            println!("\nSummary: {} converted, {} missing", found, missing);
        }
        Commands::DumpTree { filter, visible_only } => {
            dump_tree(filter, visible_only);
        }
        Commands::Dump {
            filter,
            visible_only,
            no_addons,
            no_saved_vars,
        } => {
            dump::dump_standalone(filter, visible_only, no_addons, no_saved_vars);
        }
        Commands::ConvertTexture { input, output } => {
            convert_texture(&input, output.as_ref());
        }
        Commands::Screenshot {
            output,
            width,
            height,
            filter,
            no_addons,
            no_saved_vars,
        } => {
            screenshot_standalone(output, width, height, filter, no_addons, no_saved_vars);
        }
    }
}

fn convert_texture(input: &PathBuf, output: Option<&PathBuf>) {
    use image_blp::{convert::blp_to_image, parser::load_blp};

    // Determine output path
    let output_path = match output {
        Some(p) => p.clone(),
        None => input.with_extension("webp"),
    };

    // Load BLP
    let blp = match load_blp(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error loading BLP {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };

    // Convert to image
    let img = match blp_to_image(&blp, 0) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error converting BLP: {}", e);
            std::process::exit(1);
        }
    };

    // Save as WebP
    let rgba = img.to_rgba8();
    if let Err(e) = rgba.save(&output_path) {
        eprintln!("Error saving WebP {}: {}", output_path.display(), e);
        std::process::exit(1);
    }

    println!(
        "Converted {} -> {} ({}x{})",
        input.display(),
        output_path.display(),
        rgba.width(),
        rgba.height()
    );
}

fn list_servers() {
    let servers = client::find_servers();
    if servers.is_empty() {
        println!("No wow-ui-sim servers found.");
        println!("Start wow-ui-sim first, then run wow-sim lua.");
    } else {
        println!("Running servers:");
        for server in &servers {
            let status = match client::ping(server) {
                Ok(()) => "OK",
                Err(_) => "ERROR",
            };
            println!("  {} [{}]", server.display(), status);
        }
    }
}

fn find_server() -> Option<PathBuf> {
    let servers = client::find_servers();
    if servers.is_empty() {
        eprintln!("Error: No wow-ui-sim server found.");
        eprintln!("Start wow-ui-sim first, then run wow-sim lua.");
        return None;
    }
    if servers.len() > 1 {
        eprintln!("Multiple servers found. Using first one.");
        eprintln!("Use --list to see all, or set WOW_LUA_SOCKET to specify.");
    }
    Some(servers.into_iter().next().unwrap())
}

fn execute_and_exit(code: &str) {
    let socket = match std::env::var("WOW_LUA_SOCKET") {
        Ok(s) => PathBuf::from(s),
        Err(_) => match find_server() {
            Some(s) => s,
            None => std::process::exit(1),
        },
    };

    match client::exec(&socket, code) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{}", output);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_file_and_exit(path: &PathBuf) {
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", path.display(), e);
            std::process::exit(1);
        }
    };
    execute_and_exit(&code);
}

fn dump_tree(filter: Option<String>, visible_only: bool) {
    let socket = match std::env::var("WOW_LUA_SOCKET") {
        Ok(s) => PathBuf::from(s),
        Err(_) => match find_server() {
            Some(s) => s,
            None => std::process::exit(1),
        },
    };

    match client::dump_tree(&socket, filter, visible_only) {
        Ok(tree) => {
            println!("{}", tree);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_repl() {
    let mut socket = match std::env::var("WOW_LUA_SOCKET") {
        Ok(s) => PathBuf::from(s),
        Err(_) => match find_server() {
            Some(s) => s,
            None => std::process::exit(1),
        },
    };

    println!("Connected to {}", socket.display());
    println!("Type Lua code to execute. Use .exit to quit.");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if !handle_repl_command(line, &mut socket) {
            break;
        }
    }

    println!("Goodbye!");
}

/// Handle a REPL input line. Returns false to exit the loop.
fn handle_repl_command(line: &str, socket: &mut PathBuf) -> bool {
    if line.starts_with('.') {
        match line {
            ".exit" | ".quit" | ".q" => return false,
            ".servers" => { list_servers(); }
            cmd if cmd.starts_with(".connect ") => {
                let path = cmd.strip_prefix(".connect ").unwrap().trim();
                *socket = PathBuf::from(path);
                println!("Switched to {}", socket.display());
            }
            _ => { eprintln!("Unknown command: {}", line); }
        }
        return true;
    }

    match client::exec(socket, line) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{}", output);
            }
        }
        Err(e) => { eprintln!("Error: {}", e); }
    }
    true
}

/// Blizzard addon TOC entries loaded before user addons.
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
    ("Blizzard_UnitFrame", "Blizzard_UnitFrame_Mainline.toc"),
    // Existing UI modules
    ("Blizzard_GameMenu", "Blizzard_GameMenu_Mainline.toc"),
    ("Blizzard_UIWidgets", "Blizzard_UIWidgets_Mainline.toc"),
    ("Blizzard_Minimap", "Blizzard_Minimap_Mainline.toc"),
    ("Blizzard_AddOnList", "Blizzard_AddOnList.toc"),
];

/// Set loader env vars and create a Lua environment with fonts and Blizzard addons loaded.
/// Returns the env and font system Rc (needed for screenshot quad batch building).
fn create_standalone_env(
    no_addons: bool,
    no_saved_vars: bool,
) -> (WowLuaEnv, std::rc::Rc<std::cell::RefCell<wow_ui_sim::render::WowFontSystem>>) {
    use std::cell::RefCell;
    use std::rc::Rc;

    if no_addons {
        unsafe { std::env::set_var("WOW_SIM_NO_ADDONS", "1") };
    }
    if no_saved_vars {
        unsafe { std::env::set_var("WOW_SIM_NO_SAVED_VARS", "1") };
    }

    let env = match WowLuaEnv::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to create Lua environment: {}", e);
            std::process::exit(1);
        }
    };

    let fonts_path = PathBuf::from("./fonts");
    let font_system = Rc::new(RefCell::new(wow_ui_sim::render::WowFontSystem::new(&fonts_path)));
    env.set_font_system(Rc::clone(&font_system));

    load_blizzard_addons(&env);

    // Override functions that Blizzard code defines but depend on unimplemented systems
    let _ = env.exec("UpdateMicroButtons = function() end");

    let addons_path = PathBuf::from("./Interface/AddOns");
    load_third_party_addons_standalone(&env, &addons_path, no_addons, no_saved_vars);

    (env, font_system)
}

/// Scan and load third-party addons for standalone commands.
fn load_third_party_addons_standalone(
    env: &WowLuaEnv,
    addons_path: &PathBuf,
    no_addons: bool,
    _no_saved_vars: bool,
) {
    use wow_ui_sim::loader::{find_toc_file, load_addon_with_saved_vars};
    use wow_ui_sim::lua_api::AddonInfo;
    use wow_ui_sim::saved_variables::SavedVariablesManager;
    use wow_ui_sim::toc::TocFile;

    // Always register addon metadata for C_AddOns API
    env.scan_and_register_addons(addons_path);

    let skip = no_addons
        || std::env::var("WOW_SIM_NO_ADDONS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    if skip {
        return;
    }

    let mut saved_vars = SavedVariablesManager::new();

    let Ok(entries) = std::fs::read_dir(addons_path) else { return };
    let mut addons: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if name.starts_with('.') || name == "BlizzardUI" { continue; }
        if let Some(toc) = find_toc_file(&path) {
            addons.push((name, toc));
        }
    }
    addons.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    for (name, toc_path) in &addons {
        let toc = TocFile::from_file(toc_path).ok();
        let (title, notes, lod) = toc.as_ref().map(|t| {
            let title = t.metadata.get("Title").cloned().unwrap_or_else(|| name.clone());
            let notes = t.metadata.get("Notes").cloned().unwrap_or_default();
            let lod = t.metadata.get("LoadOnDemand").map(|v| v == "1").unwrap_or(false);
            (title, notes, lod)
        }).unwrap_or_else(|| (name.clone(), String::new(), false));

        match load_addon_with_saved_vars(env, toc_path, &mut saved_vars) {
            Ok(r) => {
                eprintln!("{} loaded: {} Lua, {} XML, {} warnings",
                    name, r.lua_files, r.xml_files, r.warnings.len());
                env.register_addon(AddonInfo {
                    folder_name: name.clone(), title, notes,
                    enabled: true, loaded: true, load_on_demand: lod,
                    load_time_secs: r.timing.total().as_secs_f64(),
                });
            }
            Err(e) => {
                eprintln!("{} failed: {}", name, e);
                env.register_addon(AddonInfo {
                    folder_name: name.clone(), title, notes,
                    enabled: true, loaded: false, load_on_demand: lod,
                    load_time_secs: 0.0,
                });
            }
        }
    }
}

/// Load Blizzard base UI addons from Interface/BlizzardUI.
fn load_blizzard_addons(env: &WowLuaEnv) {
    let blizzard_ui_path = PathBuf::from("./Interface/BlizzardUI");

    if !blizzard_ui_path.exists() {
        eprintln!("Warning: Blizzard UI path not found: {}", blizzard_ui_path.display());
        return;
    }

    for (name, toc) in BLIZZARD_ADDONS {
        let toc_path = blizzard_ui_path.join(format!("{}/{}", name, toc));
        if toc_path.exists() {
            match load_addon(env, &toc_path) {
                Ok(r) => {
                    eprintln!(
                        "{} loaded: {} Lua, {} XML, {} warnings",
                        name, r.lua_files, r.xml_files, r.warnings.len()
                    );
                    for w in &r.warnings {
                        eprintln!("  [!] {}", w);
                    }
                }
                Err(e) => eprintln!("{} failed: {}", name, e),
            }
        }
    }
}

fn fire_startup_events(env: &WowLuaEnv) {
    fire_core_startup_events(env);
    fire_post_login_events(env);
}

/// Fire the core login events: ADDON_LOADED through TIME_PLAYED_MSG.
fn fire_core_startup_events(env: &WowLuaEnv) {
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
}

/// Fire post-login events and trigger addon UI.
fn fire_post_login_events(env: &WowLuaEnv) {
    let fire = |name| {
        eprintln!("[Startup] Firing {}", name);
        if let Err(e) = env.fire_event(name) {
            eprintln!("Error firing {}: {}", name, e);
        }
    };

    fire("UPDATE_BINDINGS");
    fire("DISPLAY_SIZE_CHANGED");
    fire("UI_SCALE_CHANGED");

    // Show AccountPlayed popup on startup if the addon is loaded
    let _ = env.lua().load(r#"
        if SlashCmdList and SlashCmdList.ACCOUNTPLAYEDPOPUP then
            SlashCmdList.ACCOUNTPLAYEDPOPUP()
        end
    "#).exec();

    fire("PLAYER_LEAVING_WORLD");
}

fn screenshot_standalone(
    output: PathBuf,
    width: u32,
    height: u32,
    filter: Option<String>,
    no_addons: bool,
    no_saved_vars: bool,
) {
    use wow_ui_sim::render::software::render_to_image;
    use wow_ui_sim::render::GlyphAtlas;

    let (env, font_system) = create_standalone_env(no_addons, no_saved_vars);
    env.set_screen_size(width as f32, height as f32);
    run_debug_script(&env);
    fire_startup_events(&env);

    // Hide frames that WoW's C++ engine hides by default (no target, no group, etc.)
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());

    let mut glyph_atlas = GlyphAtlas::new();
    let batch = build_screenshot_batch(&env, &font_system, width, height, filter.as_deref(), &mut glyph_atlas);

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

    save_screenshot(&img, &output);
    eprintln!("Saved {}x{} screenshot to {}", width, height, output.display());
}

/// Save screenshot image. Uses lossy WebP (quality 15) for .webp, delegates to image crate otherwise.
fn save_screenshot(img: &image::RgbaImage, output: &std::path::Path) {
    let ext = output.extension().and_then(|e: &std::ffi::OsStr| e.to_str()).unwrap_or("webp");
    if ext.eq_ignore_ascii_case("webp") {
        let encoder = webp::Encoder::from_rgba(img.as_raw(), img.width(), img.height());
        let mem = encoder.encode(15.0);
        if let Err(e) = std::fs::write(output, &*mem) {
            eprintln!("Failed to save WebP: {}", e);
            std::process::exit(1);
        }
    } else if let Err(e) = img.save(output) {
        eprintln!("Failed to save image: {}", e);
        std::process::exit(1);
    }
}

/// Run optional debug Lua script for screenshot debugging.
fn run_debug_script(env: &WowLuaEnv) {
    let debug_script = PathBuf::from("/tmp/debug-screenshot.lua");
    if debug_script.exists() {
        let script = std::fs::read_to_string(&debug_script).unwrap_or_default();
        if let Err(e) = env.exec(&script) {
            eprintln!("[Debug] Script error: {}", e);
        }
    }
}

/// Build quad batch for screenshot rendering.
fn build_screenshot_batch(
    env: &WowLuaEnv,
    font_system: &std::rc::Rc<std::cell::RefCell<wow_ui_sim::render::WowFontSystem>>,
    width: u32,
    height: u32,
    filter: Option<&str>,
    glyph_atlas: &mut wow_ui_sim::render::GlyphAtlas,
) -> wow_ui_sim::render::QuadBatch {
    use wow_ui_sim::iced_app::build_quad_batch_for_registry;

    let state = env.state().borrow();
    let mut fs = font_system.borrow_mut();

    build_quad_batch_for_registry(
        &state.widgets,
        (width as f32, height as f32),
        filter, None, None,
        Some((&mut fs, glyph_atlas)),
    )
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
