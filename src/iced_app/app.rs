//! App struct definition and core initialization.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use iced::widget::canvas::Cache;
use iced::{Point, Rectangle, Size, Task};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::lua_api::WowLuaEnv;
use crate::lua_server;
use crate::render::{GlyphAtlas, WowFontSystem};
use crate::saved_variables::SavedVariablesManager;
use crate::texture::TextureManager;
use iced_layout_inspector::server::{self as debug_server, ScreenshotData};

use super::state::InspectorState;
use super::Message;

/// Default path to WoW TTF fonts.
pub const DEFAULT_FONTS_PATH: &str = "./fonts";

/// Default path to local WebP textures (preferred).
pub const LOCAL_TEXTURES_PATH: &str = "./textures";

/// Fallback path to wow-ui-textures repository.
pub const FALLBACK_TEXTURES_PATH: &str = "/home/osso/Repos/wow-ui-textures";

/// Default path to WoW Interface directory (extracted game files).
pub const DEFAULT_INTERFACE_PATH: &str = "/home/osso/Projects/wow/Interface";

/// Default path to addons directory.
pub const DEFAULT_ADDONS_PATH: &str = "./Interface/AddOns";

/// Debug visualization options.
#[derive(Default, Clone)]
pub struct DebugOptions {
    pub borders: bool,
    pub anchors: bool,
}

// Thread-local storage for init params
thread_local! {
    pub static INIT_ENV: RefCell<Option<WowLuaEnv>> = const { RefCell::new(None) };
    pub static INIT_TEXTURES: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    pub static INIT_DEBUG: RefCell<Option<DebugOptions>> = const { RefCell::new(None) };
    pub static INIT_SAVED_VARS: RefCell<Option<SavedVariablesManager>> = const { RefCell::new(None) };
    pub static INIT_EXEC_LUA: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Fire the standard WoW startup events.
pub fn fire_startup_events(env: &Rc<RefCell<WowLuaEnv>>) {
    let env = env.borrow();
    fire_login_events(&env);
    fire_world_and_ui_events(&env);
}

/// ADDON_LOADED, VARIABLES_LOADED, PLAYER_LOGIN, TIME_PLAYED_MSG, PLAYER_ENTERING_WORLD.
fn fire_login_events(env: &WowLuaEnv) {
    println!("[Startup] Firing ADDON_LOADED");
    if let Err(e) = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(env.lua().create_string("WoWUISim").unwrap())],
    ) {
        eprintln!("Error firing ADDON_LOADED: {}", e);
    }

    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        println!("[Startup] Firing {event}");
        if let Err(e) = env.fire_event(event) {
            eprintln!("Error firing {event}: {}", e);
        }
    }

    println!("[Startup] Firing EDIT_MODE_LAYOUTS_UPDATED");
    if let Err(e) = env.fire_edit_mode_layouts_updated() {
        eprintln!("  {}", e);
    }

    println!("[Startup] Firing TIME_PLAYED_MSG via RequestTimePlayed");
    if let Err(e) = env.lua().globals().get::<mlua::Function>("RequestTimePlayed")
        .and_then(|f| f.call::<()>(()))
    {
        eprintln!("Error calling RequestTimePlayed: {}", e);
    }

    println!("[Startup] Firing PLAYER_ENTERING_WORLD");
    if let Err(e) = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[mlua::Value::Boolean(true), mlua::Value::Boolean(false)],
    ) {
        eprintln!("Error firing PLAYER_ENTERING_WORLD: {}", e);
    }

    fire_unit_aura_event(env);
}

/// Fire UNIT_AURA("player", {isFullUpdate=true}) to trigger buff frame population.
fn fire_unit_aura_event(env: &WowLuaEnv) {
    println!("[Startup] Firing UNIT_AURA");
    let lua = env.lua();
    let update_info = match lua.create_table() {
        Ok(t) => {
            let _ = t.set("isFullUpdate", true);
            mlua::Value::Table(t)
        }
        Err(_) => return,
    };
    let unit = match lua.create_string("player") {
        Ok(s) => mlua::Value::String(s),
        Err(_) => return,
    };
    if let Err(e) = env.fire_event_with_args("UNIT_AURA", &[unit, update_info]) {
        eprintln!("Error firing UNIT_AURA: {}", e);
    }
}

/// UPDATE_BINDINGS, DISPLAY_SIZE_CHANGED, UI_SCALE_CHANGED, addon hooks.
fn fire_world_and_ui_events(env: &WowLuaEnv) {
    for event in ["BAG_UPDATE_DELAYED", "UPDATE_BINDINGS", "DISPLAY_SIZE_CHANGED", "UI_SCALE_CHANGED", "UPDATE_CHAT_WINDOWS"] {
        println!("[Startup] Firing {event}");
        if let Err(e) = env.fire_event(event) {
            eprintln!("Error firing {event}: {}", e);
        }
    }

    let _ = env.lua().load(r#"
        if SlashCmdList and SlashCmdList.ACCOUNTPLAYEDPOPUP then
            SlashCmdList.ACCOUNTPLAYEDPOPUP()
        end
    "#).exec();
}

/// Application state.
pub struct App {
    pub(crate) env: Rc<RefCell<WowLuaEnv>>,
    pub(crate) log_messages: Vec<String>,
    pub(crate) command_input: String,
    pub(crate) texture_manager: Rc<RefCell<TextureManager>>,
    pub(crate) font_system: Rc<RefCell<WowFontSystem>>,
    pub(crate) glyph_atlas: Rc<RefCell<GlyphAtlas>>,
    pub(crate) frame_cache: Cache,
    pub(crate) hovered_frame: Option<u64>,
    pub(crate) pressed_frame: Option<u64>,
    pub(crate) mouse_down_frame: Option<u64>,
    pub(crate) scroll_offset: f32,
    /// Current canvas size (updated each frame for layout calculations).
    pub(crate) screen_size: std::cell::Cell<Size>,
    pub(crate) debug_rx: Option<mpsc::Receiver<debug_server::Command>>,
    pub(crate) pending_screenshot: Option<oneshot::Sender<Result<ScreenshotData, String>>>,
    pub(crate) lua_rx: Option<std::sync::mpsc::Receiver<lua_server::LuaCommand>>,
    /// Draw red debug borders around all frames when true.
    /// TODO: Re-implement as shader quads in build_quad_batch_for_registry.
    #[allow(dead_code)]
    pub(crate) debug_borders: bool,
    /// Draw green anchor points on all frames when true.
    /// TODO: Re-implement as shader quads in build_quad_batch_for_registry.
    #[allow(dead_code)]
    pub(crate) debug_anchors: bool,
    /// Track which textures have been uploaded to GPU atlas (avoid re-sending pixel data).
    pub(crate) gpu_uploaded_textures: RefCell<std::collections::HashSet<String>>,
    /// Cached quad batch for shader (avoids rebuilding every frame).
    pub(crate) cached_quads: RefCell<Option<(Size, crate::render::QuadBatch)>>,
    /// Cached sorted hit-test rects (rebuilt when layout changes).
    /// Pre-sorted top-to-bottom (highest strata first) with pre-scaled bounds.
    pub(crate) cached_hittable: RefCell<Option<Vec<(u64, Rectangle)>>>,
    /// Cached layout rects from the last quad build, shared with hit testing.
    pub(crate) cached_layout_rects: RefCell<Option<super::layout::LayoutCache>>,
    /// Flag to invalidate quad cache (set when content changes).
    pub(crate) quads_dirty: std::cell::Cell<bool>,
    /// Last quad rebuild timestamp (throttles rebuilds so cursor stays responsive).
    pub(crate) last_quad_rebuild: std::cell::Cell<std::time::Instant>,
    /// FPS counter: frame count since last update (interior mutability for draw()).
    pub(crate) frame_count: std::cell::Cell<u32>,
    /// FPS counter: last FPS calculation time.
    pub(crate) fps_last_time: std::time::Instant,
    /// Current FPS value.
    pub(crate) fps: f32,
    /// Frame render time in ms (interior mutability for draw()).
    pub(crate) frame_time_ms: std::cell::Cell<f32>,
    /// Smoothed frame time (5-second EMA).
    pub(crate) frame_time_avg: std::cell::Cell<f32>,
    /// Frame time for display (updated every 1 second with FPS).
    pub(crate) frame_time_display: f32,
    /// Current mouse position in canvas coordinates.
    pub(crate) mouse_position: Option<Point>,
    /// Currently inspected frame ID.
    pub(crate) inspected_frame: Option<u64>,
    /// Whether the inspector panel is visible.
    pub(crate) inspector_visible: bool,
    /// Position of the inspector panel.
    pub(crate) inspector_position: Point,
    /// Inspector panel state (editable fields).
    pub(crate) inspector_state: InspectorState,
    /// Whether the frames panel is collapsed.
    pub(crate) frames_panel_collapsed: bool,
    /// Last time OnUpdate handlers were fired (for elapsed calculation).
    pub(crate) last_on_update_time: std::time::Instant,
    /// SavedVariables manager for persisting addon data on exit.
    pub(crate) saved_vars: Option<SavedVariablesManager>,
    /// Lua code to execute after first frame (from --exec-lua).
    pub(crate) pending_exec_lua: Option<String>,
    /// Whether the XP bar is currently visible (toggled by UI checkbox).
    pub(crate) xp_bar_visible: bool,
    /// Last time party health was ticked (random walk every 2 seconds).
    pub(crate) last_party_health_tick: std::time::Instant,
    /// Whether rot damage is applied to party members every tick.
    pub(crate) rot_damage_enabled: bool,
    /// Currently selected player class name (for picker display).
    pub(crate) selected_class: String,
    /// Currently selected player race name (for picker display).
    pub(crate) selected_race: String,
    /// Currently selected rot damage level label.
    pub(crate) selected_rot_level: String,
}

impl App {
    pub fn title(_state: &Self) -> String {
        "WoW UI Simulator".to_string()
    }

    pub fn boot() -> (Self, Task<Message>) {
        let (env_rc, textures_path, saved_vars) = Self::take_init_params();
        let config = crate::config::SimConfig::load();
        Self::apply_config_to_state(&env_rc, &config);

        Self::run_startup_sequence(&env_rc);
        let log_messages = Self::collect_startup_logs(&env_rc);

        let (texture_manager, font_system, glyph_atlas) =
            Self::init_rendering(&env_rc, textures_path);
        let (cmd_rx, lua_rx) = Self::init_servers();
        let (debug_borders, debug_anchors) = Self::resolve_debug_flags();

        let app = Self::build_app(
            env_rc, log_messages, texture_manager, font_system, glyph_atlas,
            cmd_rx, lua_rx, debug_borders, debug_anchors, saved_vars, config,
        );

        (app, Task::none())
    }

    /// Construct the App struct from initialized components.
    #[allow(clippy::too_many_arguments)]
    fn build_app(
        env: Rc<RefCell<WowLuaEnv>>,
        log_messages: Vec<String>,
        texture_manager: Rc<RefCell<TextureManager>>,
        font_system: Rc<RefCell<WowFontSystem>>,
        glyph_atlas: Rc<RefCell<GlyphAtlas>>,
        cmd_rx: mpsc::Receiver<debug_server::Command>,
        lua_rx: std::sync::mpsc::Receiver<lua_server::LuaCommand>,
        debug_borders: bool,
        debug_anchors: bool,
        saved_vars: Option<SavedVariablesManager>,
        config: crate::config::SimConfig,
    ) -> Self {
        let now = std::time::Instant::now();
        App {
            env,
            log_messages,
            command_input: String::new(),
            texture_manager,
            font_system,
            glyph_atlas,
            frame_cache: Cache::new(),
            hovered_frame: None,
            pressed_frame: None,
            mouse_down_frame: None,
            scroll_offset: 0.0,
            screen_size: std::cell::Cell::new(Size::new(800.0, 600.0)),
            debug_rx: Some(cmd_rx),
            pending_screenshot: None,
            lua_rx: Some(lua_rx),
            debug_borders,
            debug_anchors,
            gpu_uploaded_textures: RefCell::new(std::collections::HashSet::new()),
            cached_quads: RefCell::new(None),
            cached_hittable: RefCell::new(None),
            cached_layout_rects: RefCell::new(None),
            quads_dirty: std::cell::Cell::new(true),
            last_quad_rebuild: std::cell::Cell::new(now),
            frame_count: std::cell::Cell::new(0),
            fps_last_time: now,
            fps: 0.0,
            frame_time_ms: std::cell::Cell::new(0.0),
            frame_time_avg: std::cell::Cell::new(0.0),
            frame_time_display: 0.0,
            mouse_position: None,
            inspected_frame: None,
            inspector_visible: false,
            inspector_position: Point::new(100.0, 100.0),
            inspector_state: InspectorState::default(),
            frames_panel_collapsed: true,
            last_on_update_time: now,
            saved_vars,
            pending_exec_lua: INIT_EXEC_LUA.with(|cell| cell.borrow_mut().take()),
            xp_bar_visible: config.xp_bar_visible,
            last_party_health_tick: now,
            rot_damage_enabled: config.rot_damage_enabled,
            selected_class: config.player_class,
            selected_race: config.player_race,
            selected_rot_level: config.rot_damage_level,
        }
    }

    /// Apply saved config to SimState before startup events fire.
    fn apply_config_to_state(env_rc: &Rc<RefCell<WowLuaEnv>>, config: &crate::config::SimConfig) {
        use crate::lua_api::state::{CLASS_LABELS, RACE_DATA, ROT_DAMAGE_LEVELS};
        let env = env_rc.borrow();
        let mut state = env.state().borrow_mut();
        state.player_class_index = CLASS_LABELS.iter()
            .position(|&n| n == config.player_class)
            .map(|i| (i + 1) as i32)
            .unwrap_or(1);
        state.player_race_index = RACE_DATA.iter()
            .position(|(n, _, _)| *n == config.player_race)
            .unwrap_or(0);
        state.rot_damage_level = ROT_DAMAGE_LEVELS.iter()
            .position(|(l, _)| *l == config.rot_damage_level)
            .unwrap_or(0);
    }

    /// Extract init params from thread-local storage.
    /// Fire startup events, apply post-event workarounds, and hide default-hidden frames.
    fn run_startup_sequence(env_rc: &Rc<RefCell<WowLuaEnv>>) {
        fire_startup_events(env_rc);
        let env_ref = env_rc.borrow();
        env_ref.apply_post_event_workarounds();
        let _ = crate::lua_api::hide_runtime_hidden_frames(env_ref.lua());
    }

    fn take_init_params() -> (
        Rc<RefCell<WowLuaEnv>>,
        PathBuf,
        Option<SavedVariablesManager>,
    ) {
        let env = INIT_ENV
            .with(|cell| cell.borrow_mut().take())
            .expect("WowLuaEnv not initialized");
        let textures_path = INIT_TEXTURES.with(|cell| cell.borrow_mut().take()).unwrap_or_else(|| {
            if PathBuf::from(LOCAL_TEXTURES_PATH).exists() {
                PathBuf::from(LOCAL_TEXTURES_PATH)
            } else {
                PathBuf::from(FALLBACK_TEXTURES_PATH)
            }
        });
        let saved_vars = INIT_SAVED_VARS.with(|cell| cell.borrow_mut().take());
        (Rc::new(RefCell::new(env)), textures_path, saved_vars)
    }

    /// Drain console output collected during startup.
    fn collect_startup_logs(env_rc: &Rc<RefCell<WowLuaEnv>>) -> Vec<String> {
        let mut log_messages = vec!["UI loaded. Press Ctrl+R to reload.".to_string()];
        let env = env_rc.borrow();
        let mut state = env.state().borrow_mut();
        log_messages.append(&mut state.console_output);
        log_messages
    }

    /// Create texture manager, font system, and glyph atlas.
    #[allow(clippy::type_complexity)]
    fn init_rendering(
        env_rc: &Rc<RefCell<WowLuaEnv>>,
        textures_path: PathBuf,
    ) -> (
        Rc<RefCell<TextureManager>>,
        Rc<RefCell<WowFontSystem>>,
        Rc<RefCell<GlyphAtlas>>,
    ) {
        let texture_manager = Rc::new(RefCell::new(
            TextureManager::new(textures_path)
                .with_interface_path(DEFAULT_INTERFACE_PATH)
                .with_addons_path(DEFAULT_ADDONS_PATH),
        ));
        let font_system = Rc::new(RefCell::new(
            WowFontSystem::new(&PathBuf::from(DEFAULT_FONTS_PATH)),
        ));
        env_rc.borrow().set_font_system(Rc::clone(&font_system));
        let glyph_atlas = Rc::new(RefCell::new(GlyphAtlas::new()));
        (texture_manager, font_system, glyph_atlas)
    }

    /// Start debug server and Lua REPL server.
    fn init_servers() -> (
        mpsc::Receiver<debug_server::Command>,
        std::sync::mpsc::Receiver<lua_server::LuaCommand>,
    ) {
        let (cmd_rx, _guard) = debug_server::init();
        std::mem::forget(_guard);

        let lua_rx = lua_server::init();
        (cmd_rx, lua_rx)
    }

    /// Resolve debug border/anchor flags from CLI and env vars.
    fn resolve_debug_flags() -> (bool, bool) {
        let init_debug = INIT_DEBUG.with(|cell| cell.borrow_mut().take()).unwrap_or_default();
        let debug_elements = std::env::var("WOW_SIM_DEBUG_ELEMENTS").is_ok();
        let debug_borders = init_debug.borders || debug_elements || std::env::var("WOW_SIM_DEBUG_BORDERS").is_ok();
        let debug_anchors = init_debug.anchors || debug_elements || std::env::var("WOW_SIM_DEBUG_ANCHORS").is_ok();

        if debug_borders || debug_anchors {
            eprintln!(
                "[wow-sim] Debug mode: borders={} anchors={}",
                debug_borders, debug_anchors
            );
        }
        (debug_borders, debug_anchors)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(ref saved_vars) = self.saved_vars {
            let env = self.env.borrow();
            match saved_vars.save_all(env.lua()) {
                Ok(()) => eprintln!("[wow-sim] SavedVariables saved"),
                Err(e) => eprintln!("[wow-sim] SavedVariables save error: {}", e),
            }
        }
    }
}
