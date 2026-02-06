//! WoW UI Simulator CLI
//!
//! Multi-purpose CLI for the WoW UI simulator.
//!
//! Usage:
//!   wow-sim lua                      # Interactive Lua REPL
//!   wow-sim lua -e "print('hi')"     # Execute code and exit
//!   wow-sim extract-textures         # Extract textures to WebP

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
        /// Output file path (format detected from extension: png, webp, jpg)
        #[arg(short, long, default_value = "screenshot.png")]
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
    dirs::home_dir()
        .unwrap_or_default()
        .join("Projects/wow/reference-addons")
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
            dump_standalone(filter, visible_only, no_addons, no_saved_vars);
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

    let addons_path = dirs::home_dir()
        .unwrap_or_default()
        .join("Projects/wow/reference-addons");
    env.scan_and_register_addons(&addons_path);

    (env, font_system)
}

/// Load Blizzard base UI addons from the wow-ui-source directory.
fn load_blizzard_addons(env: &WowLuaEnv) {
    let wow_ui_path = dirs::home_dir()
        .unwrap_or_default()
        .join("Projects/wow/reference-addons/wow-ui-source");

    if !wow_ui_path.exists() {
        eprintln!("Warning: Blizzard UI path not found: {}", wow_ui_path.display());
        return;
    }

    for (name, toc) in BLIZZARD_ADDONS {
        let toc_path = wow_ui_path.join(format!("Interface/AddOns/{}/{}", name, toc));
        if toc_path.exists() {
            match load_addon(env, &toc_path) {
                Ok(r) => {
                    eprintln!(
                        "{} loaded: {} Lua, {} XML, {} warnings",
                        name, r.lua_files, r.xml_files, r.warnings.len()
                    );
                }
                Err(e) => eprintln!("{} failed: {}", name, e),
            }
        }
    }
}

fn dump_standalone(
    filter: Option<String>,
    visible_only: bool,
    no_addons: bool,
    no_saved_vars: bool,
) {
    let (env, _font_system) = create_standalone_env(no_addons, no_saved_vars);

    let state = env.state().borrow();
    let widgets = &state.widgets;

    let mut roots = collect_root_frames(widgets);
    roots.sort_by(|a, b| {
        let name_a = a.1.as_deref().unwrap_or("");
        let name_b = b.1.as_deref().unwrap_or("");
        name_a.cmp(name_b)
    });

    let version_check = state.cvars.get_bool("checkAddonVersion");
    eprintln!("Load out of date addons: {}", if version_check { "off" } else { "on" });
    eprintln!("\n=== Frame Tree ===\n");

    for (id, _) in &roots {
        print_frame(widgets, *id, 0, &filter, visible_only);
    }
}

/// Collect root frames (no parent) sorted by name.
fn collect_root_frames(
    widgets: &wow_ui_sim::widget::WidgetRegistry,
) -> Vec<(u64, Option<String>)> {
    widgets
        .all_ids()
        .iter()
        .filter_map(|&id| {
            let w = widgets.get(id)?;
            if w.parent_id.is_none() {
                Some((id, w.name.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// Recursively print a frame and its children.
fn print_frame(
    widgets: &wow_ui_sim::widget::WidgetRegistry,
    id: u64,
    depth: usize,
    filter: &Option<String>,
    visible_only: bool,
) {
    let Some(frame) = widgets.get(id) else { return };

    if visible_only && !frame.visible {
        return;
    }

    let name = frame.name.as_deref().unwrap_or("(anonymous)");
    let matches_filter = filter
        .as_ref()
        .map(|f| name.to_lowercase().contains(&f.to_lowercase()))
        .unwrap_or(true);

    if matches_filter {
        let indent = "  ".repeat(depth);
        let vis = if frame.visible { "visible" } else { "hidden" };
        let keys: Vec<_> = frame.children_keys.keys().collect();
        let keys_str = if keys.is_empty() {
            String::new()
        } else {
            format!(" keys=[{}]", keys.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
        };
        println!(
            "{}{} [{:?}] ({}x{}) {}{}",
            indent, name, frame.widget_type, frame.width as i32, frame.height as i32, vis, keys_str
        );
    }

    for &child_id in &frame.children {
        print_frame(widgets, child_id, depth + 1, filter, visible_only);
    }
}

fn screenshot_standalone(
    output: PathBuf,
    width: u32,
    height: u32,
    filter: Option<String>,
    no_addons: bool,
    no_saved_vars: bool,
) {
    use wow_ui_sim::iced_app::build_quad_batch_for_registry;
    use wow_ui_sim::render::software::render_to_image;
    use wow_ui_sim::render::GlyphAtlas;
    use wow_ui_sim::texture::TextureManager;

    let (env, font_system) = create_standalone_env(no_addons, no_saved_vars);
    run_debug_script(&env);

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

    if let Err(e) = img.save(&output) {
        eprintln!("Failed to save image: {}", e);
        std::process::exit(1);
    }

    eprintln!("Saved {}x{} screenshot to {}", width, height, output.display());
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
        .with_addons_path(home.join("Projects/wow/reference-addons"))
}
