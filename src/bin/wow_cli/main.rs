//! WoW UI Simulator CLI - thin client for a running wow-sim server.
//!
//! All commands except extract-textures, convert-texture, and generate require a running wow-sim
//! instance.
//!
//! Usage:
//!   wow-cli lua                      # Interactive Lua REPL
//!   wow-cli lua -e "print('hi')"     # Execute code and exit
//!   wow-cli dump-tree                # Dump frame tree from running server
//!   wow-cli screenshot -o out.webp   # Render screenshot via running server
//!   wow-cli extract-textures         # Extract textures to WebP (standalone)
//!   wow-cli convert-texture foo.BLP  # Convert single BLP to WebP (standalone)
//!   wow-cli generate spells          # Regenerate data/spells.rs from CSVs

mod csv_util;
mod gen_atlas;
mod gen_global_strings;
mod gen_items;
mod gen_manifest;
mod gen_spells;
mod gen_traits;
mod gen_traits_emit;
mod gen_traits_load;

use clap::{Parser, Subcommand};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use wow_ui_sim::lua_server::client;

#[derive(Parser)]
#[command(name = "wow-cli")]
#[command(about = "WoW UI Simulator CLI tools (requires running wow-sim)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lua REPL - connect to running wow-sim and execute Lua code
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

    /// Dump the rendered frame tree (requires running server)
    DumpTree {
        /// Filter by frame name (substring match)
        #[arg(short, long)]
        filter: Option<String>,

        /// Show only visible frames
        #[arg(long)]
        visible_only: bool,
    },

    /// Render UI to an image file (requires running server)
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

    /// Extract textures referenced by addons to WebP format (standalone)
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

    /// Convert a BLP texture file to WebP format (standalone)
    ConvertTexture {
        /// Input BLP file path
        input: PathBuf,

        /// Output WebP file path (defaults to input with .webp extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate static data files from WoW CSV exports (standalone)
    Generate {
        #[command(subcommand)]
        what: GenerateTarget,
    },
}

#[derive(Subcommand)]
enum GenerateTarget {
    /// Generate data/spells.rs from SpellName/Spell/SpellMisc CSVs
    Spells,
    /// Generate data/items.rs from ItemSparse CSV
    Items,
    /// Generate data/atlas.rs from UiTextureAtlas CSVs
    Atlas,
    /// Generate data/global_strings.rs from GlobalStrings CSV
    GlobalStrings,
    /// Generate data/manifest_interface_data.rs from ManifestInterfaceData CSV
    Manifest,
    /// Generate data/traits.rs from Trait* CSVs
    Traits,
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
        Commands::DumpTree { filter, visible_only } => {
            dump_tree(filter, visible_only);
        }
        Commands::Screenshot { output, width, height, filter, crop } => {
            take_screenshot(&output, width, height, filter, crop);
        }
        Commands::ExtractTextures { addons, interface, output } => {
            let (found, missing) =
                wow_ui_sim::extract_textures::extract_textures(&addons, &interface, &output);
            println!("\nSummary: {} converted, {} missing", found, missing);
        }
        Commands::ConvertTexture { input, output } => {
            convert_texture(&input, output.as_ref());
        }
        Commands::Generate { what } => {
            run_generator(what);
        }
    }
}

fn run_generator(target: GenerateTarget) {
    let result = match target {
        GenerateTarget::Spells => gen_spells::run(),
        GenerateTarget::Items => gen_items::run(),
        GenerateTarget::Atlas => gen_atlas::run(),
        GenerateTarget::GlobalStrings => gen_global_strings::run(),
        GenerateTarget::Manifest => gen_manifest::run(),
        GenerateTarget::Traits => gen_traits::run(),
    };
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

// ── IPC helpers ─────────────────────────────────────────────────────

fn resolve_socket() -> PathBuf {
    match std::env::var("WOW_LUA_SOCKET") {
        Ok(s) => PathBuf::from(s),
        Err(_) => match find_server() {
            Some(s) => s,
            None => std::process::exit(1),
        },
    }
}

fn find_server() -> Option<PathBuf> {
    let servers = client::find_servers();
    if servers.is_empty() {
        eprintln!("Error: No wow-sim server found.");
        eprintln!("Start wow-sim first, then run wow-cli.");
        return None;
    }
    if servers.len() > 1 {
        eprintln!("Multiple servers found. Using first one.");
        eprintln!("Use --list to see all, or set WOW_LUA_SOCKET to specify.");
    }
    Some(servers.into_iter().next().unwrap())
}

fn list_servers() {
    let servers = client::find_servers();
    if servers.is_empty() {
        println!("No wow-sim servers found.");
        println!("Start wow-sim first, then run wow-cli lua.");
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

// ── Subcommand handlers ─────────────────────────────────────────────

fn execute_and_exit(code: &str) {
    let socket = resolve_socket();
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
    let socket = resolve_socket();
    match client::dump_tree(&socket, filter, visible_only) {
        Ok(tree) => println!("{}", tree),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn take_screenshot(output: &PathBuf, width: u32, height: u32, filter: Option<String>, crop: Option<String>) {
    let socket = resolve_socket();
    // Canonicalize output path so the server can write to the right location
    let abs_output = std::env::current_dir()
        .map(|cwd| cwd.join(output))
        .unwrap_or_else(|_| output.clone());
    match client::screenshot(&socket, &abs_output.to_string_lossy(), width, height, filter, crop) {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn convert_texture(input: &PathBuf, output: Option<&PathBuf>) {
    use image_blp::{convert::blp_to_image, parser::load_blp};

    let output_path = match output {
        Some(p) => p.clone(),
        None => input.with_extension("webp"),
    };

    let blp = match load_blp(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error loading BLP {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };

    let img = match blp_to_image(&blp, 0) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error converting BLP: {}", e);
            std::process::exit(1);
        }
    };

    let mut rgba = img.to_rgba8();
    // Fix 1-bit alpha: image-blp decodes 1-bit alpha as literal 0/1 byte values
    wow_ui_sim::texture::fix_1bit_alpha(rgba.as_mut());
    if let Err(e) = rgba.save(&output_path) {
        eprintln!("Error saving {}: {}", output_path.display(), e);
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

// ── REPL ────────────────────────────────────────────────────────────

fn run_repl() {
    let mut socket = resolve_socket();

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
            ".servers" => list_servers(),
            cmd if cmd.starts_with(".connect ") => {
                let path = cmd.strip_prefix(".connect ").unwrap().trim();
                *socket = PathBuf::from(path);
                println!("Switched to {}", socket.display());
            }
            _ => eprintln!("Unknown command: {}", line),
        }
        return true;
    }

    match client::exec(socket, line) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{}", output);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    true
}
