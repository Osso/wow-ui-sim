//! App struct definition and core initialization.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use iced::widget::canvas::Cache;
use iced::widget::image::Handle as ImageHandle;
use iced::{Point, Size, Task};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::lua_api::WowLuaEnv;
use crate::lua_server;
use crate::texture::TextureManager;
use iced_layout_inspector::server::{self as debug_server, ScreenshotData};

use super::state::InspectorState;
use super::Message;

/// Default path to local WebP textures (preferred).
pub const LOCAL_TEXTURES_PATH: &str = "./textures";

/// Fallback path to wow-ui-textures repository.
pub const FALLBACK_TEXTURES_PATH: &str = "/home/osso/Repos/wow-ui-textures";

/// Default path to WoW Interface directory (extracted game files).
pub const DEFAULT_INTERFACE_PATH: &str = "/home/osso/Projects/wow/Interface";

/// Default path to addons directory.
pub const DEFAULT_ADDONS_PATH: &str = "/home/osso/Projects/wow/reference-addons";

// Thread-local storage for init params
thread_local! {
    pub static INIT_ENV: RefCell<Option<WowLuaEnv>> = const { RefCell::new(None) };
    pub static INIT_TEXTURES: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Fire the standard WoW startup events.
pub fn fire_startup_events(env: &Rc<RefCell<WowLuaEnv>>) {
    let env = env.borrow();

    println!("[Startup] Firing ADDON_LOADED");
    if let Err(e) = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(env.lua().create_string("WoWUISim").unwrap())],
    ) {
        eprintln!("Error firing ADDON_LOADED: {}", e);
    }

    println!("[Startup] Firing PLAYER_LOGIN");
    if let Err(e) = env.fire_event("PLAYER_LOGIN") {
        eprintln!("Error firing PLAYER_LOGIN: {}", e);
    }

    println!("[Startup] Firing PLAYER_ENTERING_WORLD");
    if let Err(e) = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[
            mlua::Value::Boolean(true),  // isInitialLogin
            mlua::Value::Boolean(false), // isReloadingUi
        ],
    ) {
        eprintln!("Error firing PLAYER_ENTERING_WORLD: {}", e);
    }
}

/// Application state.
pub struct App {
    pub(crate) env: Rc<RefCell<WowLuaEnv>>,
    pub(crate) log_messages: Vec<String>,
    pub(crate) command_input: String,
    pub(crate) texture_manager: Rc<RefCell<TextureManager>>,
    /// Cache of loaded texture image handles (uses RefCell for interior mutability during draw).
    pub(crate) image_handles: Rc<RefCell<HashMap<String, ImageHandle>>>,
    pub(crate) frame_cache: Cache,
    /// Cache for text-only overlay (used in shader mode).
    pub(crate) text_cache: Cache,
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
    pub(crate) debug_borders: bool,
    /// Draw green anchor points on all frames when true.
    pub(crate) debug_anchors: bool,
    /// Track which textures have been uploaded to GPU atlas (avoid re-sending pixel data).
    pub(crate) gpu_uploaded_textures: RefCell<std::collections::HashSet<String>>,
    /// Cached quad batch for shader (avoids rebuilding every frame).
    pub(crate) cached_quads: RefCell<Option<(Size, crate::render::QuadBatch)>>,
    /// Flag to invalidate quad cache (set when content changes).
    pub(crate) quads_dirty: std::cell::Cell<bool>,
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
}

impl App {
    pub fn title(_state: &Self) -> String {
        "WoW UI Simulator".to_string()
    }

    pub fn boot() -> (Self, Task<Message>) {
        // Take init params from thread-local
        let env = INIT_ENV
            .with(|cell| cell.borrow_mut().take())
            .expect("WowLuaEnv not initialized");
        let textures_path = INIT_TEXTURES.with(|cell| cell.borrow_mut().take()).unwrap_or_else(|| {
            // Prefer local WebP textures, fall back to full repo
            if PathBuf::from(LOCAL_TEXTURES_PATH).exists() {
                PathBuf::from(LOCAL_TEXTURES_PATH)
            } else {
                PathBuf::from(FALLBACK_TEXTURES_PATH)
            }
        });

        let env_rc = Rc::new(RefCell::new(env));

        // Fire startup events
        fire_startup_events(&env_rc);

        // Collect console output from startup
        let mut log_messages = vec!["UI loaded. Press Ctrl+R to reload.".to_string()];
        {
            let env = env_rc.borrow();
            let mut state = env.state().borrow_mut();
            log_messages.append(&mut state.console_output);
        }

        let texture_manager = Rc::new(RefCell::new(
            TextureManager::new(textures_path)
                .with_interface_path(DEFAULT_INTERFACE_PATH)
                .with_addons_path(DEFAULT_ADDONS_PATH),
        ));

        // Initialize debug server
        let (cmd_rx, _guard) = debug_server::init();
        eprintln!(
            "[wow-ui-sim] Debug server at {}",
            debug_server::socket_path().display()
        );
        // Keep guard alive by leaking it
        std::mem::forget(_guard);

        // Initialize Lua REPL server
        let lua_rx = lua_server::init();
        eprintln!(
            "[wow-ui-sim] Lua server at {}",
            lua_server::socket_path().display()
        );

        // Debug modes: WOW_SIM_DEBUG_ELEMENTS enables both borders and anchors
        let debug_elements = std::env::var("WOW_SIM_DEBUG_ELEMENTS").is_ok();
        let debug_borders = debug_elements || std::env::var("WOW_SIM_DEBUG_BORDERS").is_ok();
        let debug_anchors = debug_elements || std::env::var("WOW_SIM_DEBUG_ANCHORS").is_ok();

        if debug_borders || debug_anchors {
            eprintln!(
                "[wow-ui-sim] Debug mode: borders={} anchors={}",
                debug_borders, debug_anchors
            );
        }

        let app = App {
            env: env_rc,
            log_messages,
            command_input: String::new(),
            texture_manager,
            image_handles: Rc::new(RefCell::new(HashMap::new())),
            frame_cache: Cache::new(),
            text_cache: Cache::new(),
            hovered_frame: None,
            pressed_frame: None,
            mouse_down_frame: None,
            scroll_offset: 0.0,
            screen_size: std::cell::Cell::new(Size::new(800.0, 600.0)), // Initial, updated each frame
            debug_rx: Some(cmd_rx),
            pending_screenshot: None,
            lua_rx: Some(lua_rx),
            debug_borders,
            debug_anchors,
            gpu_uploaded_textures: RefCell::new(std::collections::HashSet::new()),
            cached_quads: RefCell::new(None),
            quads_dirty: std::cell::Cell::new(true),
            frame_count: std::cell::Cell::new(0),
            fps_last_time: std::time::Instant::now(),
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
        };

        (app, Task::none())
    }
}
