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
    // Foundation (no new deps)
    ("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc"),
    ("Blizzard_Colors", "Blizzard_Colors_Mainline.toc"),
    ("Blizzard_SharedXML", "Blizzard_SharedXML_Mainline.toc"),
    ("Blizzard_SharedXMLGame", "Blizzard_SharedXMLGame_Mainline.toc"),
    ("Blizzard_UIPanelTemplates", "Blizzard_UIPanelTemplates_Mainline.toc"),
    ("Blizzard_FrameXMLBase", "Blizzard_FrameXMLBase.toc"),
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
    ("Blizzard_UIPanels_Game", "Blizzard_UIPanels_Game_Mainline.toc"),
    ("Blizzard_ActionBar", "Blizzard_ActionBar_Mainline.toc"),
    // Existing UI modules
    ("Blizzard_GameMenu", "Blizzard_GameMenu_Mainline.toc"),
    ("Blizzard_UIWidgets", "Blizzard_UIWidgets_Mainline.toc"),
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

    init_addon_list(&env);

    // Print addon list
    let _ = env.exec(
        r#"
        local num = C_AddOns.GetNumAddOns()
        if num > 0 then
            print("\n=== Addons (" .. num .. ") ===\n")
            for i = 1, num do
                local name, title, notes, loadable, reason, security = C_AddOns.GetAddOnInfo(i)
                local loaded = C_AddOns.IsAddOnLoaded(i)
                local enabled = C_AddOns.GetAddOnEnableState(i) > 0
                local status = loaded and "loaded" or (enabled and "enabled" or "disabled")
                print(string.format("  [%d] %s (%s) [%s]", i, tostring(title), tostring(name), status))
            end
        end
        "#,
    );

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

    let display_name = resolve_display_name(widgets, frame, id);
    let matches_filter = filter
        .as_ref()
        .map(|f| display_name.to_lowercase().contains(&f.to_lowercase()))
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
        let text_str = resolve_display_text(widgets, frame)
            .map(|t| format!(" text={:?}", t))
            .unwrap_or_default();
        let font_str = if frame.widget_type == wow_ui_sim::widget::WidgetType::FontString {
            format!(" font={:?} size={}", frame.font.as_deref().unwrap_or("(none)"), frame.font_size)
        } else {
            String::new()
        };
        println!(
            "{}{} [{:?}] ({}x{}) {}{}{}{}",
            indent, display_name, frame.widget_type, frame.width as i32, frame.height as i32, vis, text_str, font_str, keys_str
        );
    }

    for &child_id in &frame.children {
        print_frame(widgets, child_id, depth + 1, filter, visible_only);
    }
}

/// Resolve a display name for a frame: use its global name, or look up its
/// parentKey from the parent's children_keys, falling back to "(anonymous)".
fn resolve_display_name(
    widgets: &wow_ui_sim::widget::WidgetRegistry,
    frame: &wow_ui_sim::widget::Frame,
    id: u64,
) -> String {
    // Use global name if it's a real name (not auto-generated)
    if let Some(ref name) = frame.name {
        if !name.starts_with("__anon_")
            && !name.starts_with("__frame_")
            && !name.starts_with("__tex_")
            && !name.starts_with("__fs_")
        {
            return name.clone();
        }
    }

    // Look up parentKey from parent's children_keys
    if let Some(parent_id) = frame.parent_id {
        if let Some(parent) = widgets.get(parent_id) {
            for (key, &child_id) in &parent.children_keys {
                if child_id == id {
                    return format!(".{}", key);
                }
            }
        }
    }

    frame
        .name
        .as_deref()
        .unwrap_or("(anonymous)")
        .to_string()
}

/// Get display text for a frame: its own text, or text from a Title/TitleText child.
fn resolve_display_text(
    widgets: &wow_ui_sim::widget::WidgetRegistry,
    frame: &wow_ui_sim::widget::Frame,
) -> Option<String> {
    // Use frame's own text if present
    if let Some(ref t) = frame.text {
        if !t.is_empty() {
            return Some(strip_wow_escapes(t));
        }
    }

    // Check Title/TitleText children for text
    for key in &["Title", "TitleText"] {
        if let Some(&child_id) = frame.children_keys.get(*key) {
            if let Some(child) = widgets.get(child_id) {
                if let Some(ref t) = child.text {
                    if !t.is_empty() {
                        return Some(strip_wow_escapes(t));
                    }
                }
            }
        }
    }

    None
}

/// Strip WoW escape sequences (|T...|t texture, |c...|r color) for cleaner display.
fn strip_wow_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '|' {
            match chars.peek() {
                Some('T') | Some('t') => {
                    // |Tpath:h:w:x:y|t - skip texture escape
                    if chars.peek() == Some(&'T') {
                        chars.next();
                        while let Some(&ch) = chars.peek() {
                            chars.next();
                            if ch == '|' {
                                chars.next(); // skip 't'
                                break;
                            }
                        }
                    } else {
                        chars.next(); // lowercase t is end marker, already consumed
                    }
                }
                Some('c') => {
                    // |cFFRRGGBB - skip color code (10 chars total: |c + 8 hex)
                    chars.next();
                    for _ in 0..8 {
                        chars.next();
                    }
                }
                Some('r') => {
                    chars.next(); // |r = color reset
                }
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
    }
    result.trim().to_string()
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
    init_addon_list(&env);
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

/// Show and populate AddonList (matches main.rs GUI init).
fn init_addon_list(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if AddonList and AddonListMixin then
            pcall(function() AddonListMixin.OnLoad(AddonList) end)
            AddonList:Show()
            if AddonList_Update then pcall(AddonList_Update) end
        end
        "#,
    );
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
