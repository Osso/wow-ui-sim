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
            Ok(0) => break, // EOF
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

        // Handle REPL commands
        if line.starts_with('.') {
            match line {
                ".exit" | ".quit" | ".q" => break,
                ".servers" => {
                    list_servers();
                    continue;
                }
                cmd if cmd.starts_with(".connect ") => {
                    let path = cmd.strip_prefix(".connect ").unwrap().trim();
                    socket = PathBuf::from(path);
                    println!("Switched to {}", socket.display());
                    continue;
                }
                _ => {
                    eprintln!("Unknown command: {}", line);
                    continue;
                }
            }
        }

        // Execute Lua code
        match client::exec(&socket, line) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    println!("Goodbye!");
}

fn dump_standalone(
    filter: Option<String>,
    visible_only: bool,
    no_addons: bool,
    no_saved_vars: bool,
) {
    // Set environment variables for the loader
    // SAFETY: We're single-threaded at this point
    if no_addons {
        unsafe { std::env::set_var("WOW_SIM_NO_ADDONS", "1") };
    }
    if no_saved_vars {
        unsafe { std::env::set_var("WOW_SIM_NO_SAVED_VARS", "1") };
    }

    // Create Lua environment
    let env = match WowLuaEnv::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to create Lua environment: {}", e);
            std::process::exit(1);
        }
    };

    // Load Blizzard SharedXML for base UI support
    let wow_ui_path = dirs::home_dir()
        .unwrap_or_default()
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
                        eprintln!(
                            "{} loaded: {} Lua, {} XML, {} warnings",
                            name,
                            r.lua_files,
                            r.xml_files,
                            r.warnings.len()
                        );
                    }
                    Err(e) => eprintln!("{} failed: {}", name, e),
                }
            }
        }
    } else {
        eprintln!("Warning: Blizzard UI path not found: {}", wow_ui_path.display());
    }

    // Dump the frame tree
    let state = env.state().borrow();
    let widgets = &state.widgets;

    // Collect root frames (no parent)
    let all_ids = widgets.all_ids();
    let mut roots: Vec<_> = all_ids
        .iter()
        .filter_map(|&id| {
            let w = widgets.get(id)?;
            if w.parent_id.is_none() {
                Some((id, w.name.clone()))
            } else {
                None
            }
        })
        .collect();

    // Sort by name for consistent output
    roots.sort_by(|a, b| {
        let name_a = a.1.as_deref().unwrap_or("");
        let name_b = b.1.as_deref().unwrap_or("");
        name_a.cmp(name_b)
    });

    fn print_frame(
        widgets: &wow_ui_sim::widget::WidgetRegistry,
        id: u64,
        depth: usize,
        filter: &Option<String>,
        visible_only: bool,
    ) {
        let Some(frame) = widgets.get(id) else {
            return;
        };

        // Filter by visibility
        if visible_only && !frame.visible {
            return;
        }

        // Filter by name
        let name = frame.name.as_deref().unwrap_or("(anonymous)");
        let matches_filter = filter
            .as_ref()
            .map(|f| name.to_lowercase().contains(&f.to_lowercase()))
            .unwrap_or(true);

        if matches_filter {
            let indent = "  ".repeat(depth);
            let vis = if frame.visible { "visible" } else { "hidden" };
            let size = format!("{}x{}", frame.width as i32, frame.height as i32);

            // Show parentKey children if any
            let keys: Vec<_> = frame.children_keys.keys().collect();
            let keys_str = if keys.is_empty() {
                String::new()
            } else {
                format!(" keys=[{}]", keys.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
            };

            println!(
                "{}{} [{:?}] ({}) {}{}",
                indent, name, frame.widget_type, size, vis, keys_str
            );
        }

        // Print children
        for &child_id in &frame.children {
            print_frame(widgets, child_id, depth + 1, filter, visible_only);
        }
    }

    eprintln!("\n=== Frame Tree ===\n");

    for (id, _) in roots {
        print_frame(widgets, id, 0, &filter, visible_only);
    }
}
