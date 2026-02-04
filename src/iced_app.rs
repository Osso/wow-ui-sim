//! Iced-based UI for rendering WoW frames.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Geometry, Path, Stroke};
use iced::widget::image::Handle as ImageHandle;
use iced::widget::shader::{self, Shader};
use iced::widget::{button, column, container, row, scrollable, stack, text, text_input, Column};
use iced::window::screenshot::Screenshot;
use iced::{window, Border, Color, Element, Event, Font, Length, Point, Rectangle, Size, Theme};
use iced::{Subscription, Task};

use crate::render::{BlendMode, GpuTextureData, QuadBatch, WowUiPrimitive};

use iced_layout_inspector::server::{self as debug_server, Command as DebugCommand, ScreenshotData};
use tokio::sync::oneshot;
use tokio::sync::mpsc;

use crate::lua_api::WowLuaEnv;
use crate::lua_server::{self, LuaCommand, Response as LuaResponse};
use crate::render::text::{strip_wow_markup, wow_font_to_iced, TextRenderer};
use crate::render::texture::{draw_horizontal_slice_texture, UI_SCALE};
use crate::texture::TextureManager;
use crate::widget::{AnchorPoint, TextJustify, WidgetType};
use crate::LayoutRect;

/// Default path to local WebP textures (preferred).
const LOCAL_TEXTURES_PATH: &str = "./textures";

/// Fallback path to wow-ui-textures repository.
const FALLBACK_TEXTURES_PATH: &str = "/home/osso/Repos/wow-ui-textures";

/// Default path to WoW Interface directory (extracted game files).
const DEFAULT_INTERFACE_PATH: &str = "/home/osso/Projects/wow/Interface";

/// Default path to addons directory.
const DEFAULT_ADDONS_PATH: &str = "/home/osso/Projects/wow/reference-addons";

// WoW-inspired color palette
mod palette {
    use iced::Color;

    pub const BG_DARK: Color = Color::from_rgb(0.05, 0.05, 0.08);
    pub const BG_PANEL: Color = Color::from_rgb(0.12, 0.12, 0.14);
    pub const BG_INPUT: Color = Color::from_rgb(0.06, 0.06, 0.08);
    pub const GOLD: Color = Color::from_rgb(0.85, 0.65, 0.13);
    pub const GOLD_DIM: Color = Color::from_rgb(0.55, 0.42, 0.10);
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.90, 0.85);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.60, 0.58, 0.55);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.43, 0.40);
    pub const BORDER: Color = Color::from_rgb(0.25, 0.23, 0.20);
    pub const BORDER_HIGHLIGHT: Color = Color::from_rgb(0.40, 0.35, 0.25);
    pub const CONSOLE_TEXT: Color = Color::from_rgb(0.70, 0.85, 0.70);
}

/// Run the iced UI with the given Lua environment.
pub fn run_iced_ui(env: WowLuaEnv) -> Result<(), Box<dyn std::error::Error>> {
    // Prefer local WebP textures, fall back to full repo
    let textures_path = if PathBuf::from(LOCAL_TEXTURES_PATH).exists() {
        PathBuf::from(LOCAL_TEXTURES_PATH)
    } else {
        PathBuf::from(FALLBACK_TEXTURES_PATH)
    };
    run_iced_ui_with_textures(env, textures_path)
}

/// Run the iced UI with the given Lua environment and textures path.
pub fn run_iced_ui_with_textures(
    env: WowLuaEnv,
    textures_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Store in thread-local for the boot function
    INIT_ENV.with(|cell| *cell.borrow_mut() = Some(env));
    INIT_TEXTURES.with(|cell| *cell.borrow_mut() = Some(textures_path));

    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .window_size((1024.0, 768.0))
        .run()?;

    Ok(())
}

// Thread-local storage for init params
thread_local! {
    static INIT_ENV: RefCell<Option<WowLuaEnv>> = const { RefCell::new(None) };
    static INIT_TEXTURES: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Fire the standard WoW startup events.
fn fire_startup_events(env: &Rc<RefCell<WowLuaEnv>>) {
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


/// Application messages.
#[derive(Debug, Clone)]
pub enum Message {
    FireEvent(String),
    Scroll(f32, f32),
    ReloadUI,
    CommandInputChanged(String),
    ExecuteCommand,
    ProcessTimers,
    CanvasEvent(CanvasMessage),
    ScreenshotTaken(Screenshot),
    /// Tick for FPS display refresh.
    FpsTick,
    /// Close the inspector panel.
    InspectorClose,
    /// Inspector width input changed.
    InspectorWidthChanged(String),
    /// Inspector height input changed.
    InspectorHeightChanged(String),
    /// Inspector alpha input changed.
    InspectorAlphaChanged(String),
    /// Inspector frame level input changed.
    InspectorLevelChanged(String),
    /// Inspector visible checkbox toggled.
    InspectorVisibleToggled(bool),
    /// Inspector mouse enabled checkbox toggled.
    InspectorMouseEnabledToggled(bool),
    /// Apply inspector changes to the frame.
    InspectorApply,
    /// Toggle frames panel collapsed state.
    ToggleFramesPanel,
}

/// Canvas-specific messages.
#[derive(Debug, Clone)]
pub enum CanvasMessage {
    MouseMove(Point),
    MouseDown(Point),
    MouseUp(Point),
    MiddleClick(Point),
}

/// Text overlay wrapper for shader mode.
///
/// This renders only text (FontStrings) on a transparent background,
/// layered on top of the shader which renders textures/backgrounds.
pub struct TextOverlay<'a> {
    app: &'a App,
}

impl<'a> TextOverlay<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

/// Inspector panel state for editing frame properties.
#[derive(Default, Clone)]
pub struct InspectorState {
    pub width: String,
    pub height: String,
    pub alpha: String,
    pub frame_level: String,
    pub visible: bool,
    pub mouse_enabled: bool,
}

/// Application state.
pub struct App {
    env: Rc<RefCell<WowLuaEnv>>,
    log_messages: Vec<String>,
    command_input: String,
    texture_manager: Rc<RefCell<TextureManager>>,
    /// Cache of loaded texture image handles (uses RefCell for interior mutability during draw).
    image_handles: Rc<RefCell<HashMap<String, ImageHandle>>>,
    frame_cache: Cache,
    /// Cache for text-only overlay (used in shader mode).
    text_cache: Cache,
    hovered_frame: Option<u64>,
    pressed_frame: Option<u64>,
    mouse_down_frame: Option<u64>,
    scroll_offset: f32,
    /// Current canvas size (updated each frame for layout calculations).
    screen_size: std::cell::Cell<Size>,
    debug_rx: Option<mpsc::Receiver<DebugCommand>>,
    pending_screenshot: Option<oneshot::Sender<Result<ScreenshotData, String>>>,
    lua_rx: Option<std::sync::mpsc::Receiver<LuaCommand>>,
    /// Draw red debug borders around all frames when true.
    debug_borders: bool,
    /// Draw green anchor points on all frames when true.
    debug_anchors: bool,
    /// Track which textures have been uploaded to GPU atlas (avoid re-sending pixel data).
    gpu_uploaded_textures: RefCell<std::collections::HashSet<String>>,
    /// Cached quad batch for shader (avoids rebuilding every frame).
    cached_quads: RefCell<Option<(Size, QuadBatch)>>,
    /// Flag to invalidate quad cache (set when content changes).
    quads_dirty: std::cell::Cell<bool>,
    /// FPS counter: frame count since last update (interior mutability for draw()).
    frame_count: std::cell::Cell<u32>,
    /// FPS counter: last FPS calculation time.
    fps_last_time: std::time::Instant,
    /// Current FPS value.
    fps: f32,
    /// Frame render time in ms (interior mutability for draw()).
    frame_time_ms: std::cell::Cell<f32>,
    /// Smoothed frame time (5-second EMA).
    frame_time_avg: std::cell::Cell<f32>,
    /// Frame time for display (updated every 1 second with FPS).
    frame_time_display: f32,
    /// Current mouse position in canvas coordinates.
    mouse_position: Option<Point>,
    /// Currently inspected frame ID.
    inspected_frame: Option<u64>,
    /// Whether the inspector panel is visible.
    inspector_visible: bool,
    /// Position of the inspector panel.
    inspector_position: Point,
    /// Inspector panel state (editable fields).
    inspector_state: InspectorState,
    /// Whether the frames panel is collapsed.
    frames_panel_collapsed: bool,
}

impl App {
    fn title(_state: &Self) -> String {
        "WoW UI Simulator".to_string()
    }

    fn boot() -> (Self, Task<Message>) {
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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FireEvent(event) => {
                {
                    let env = self.env.borrow();
                    if let Err(e) = env.fire_event(&event) {
                        self.log_messages.push(format!("Event error: {}", e));
                    } else {
                        self.log_messages.push(format!("Fired: {}", event));
                    }
                }
                self.drain_console();
                self.frame_cache.clear();
                self.quads_dirty.set(true);
            }
            Message::CanvasEvent(canvas_msg) => match canvas_msg {
                CanvasMessage::MouseMove(pos) => {
                    self.mouse_position = Some(pos);
                    let new_hovered = self.hit_test(pos);
                    if new_hovered != self.hovered_frame {
                        let env = self.env.borrow();
                        if let Some(old_id) = self.hovered_frame {
                            let _ = env.fire_script_handler(old_id, "OnLeave", vec![]);
                        }
                        if let Some(new_id) = new_hovered {
                            let _ = env.fire_script_handler(new_id, "OnEnter", vec![]);
                        }
                        drop(env);
                        self.hovered_frame = new_hovered;
                        self.drain_console();
                        self.frame_cache.clear();
                        self.quads_dirty.set(true);
                    }
                }
                CanvasMessage::MouseDown(pos) => {
                    if let Some(frame_id) = self.hit_test(pos) {
                        self.mouse_down_frame = Some(frame_id);
                        self.pressed_frame = Some(frame_id);
                        let env = self.env.borrow();
                        let button_val =
                            mlua::Value::String(env.lua().create_string("LeftButton").unwrap());
                        let _ = env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val]);
                        drop(env);
                        self.drain_console();
                        self.frame_cache.clear();
                        self.quads_dirty.set(true);
                    }
                }
                CanvasMessage::MouseUp(pos) => {
                    // Check if click is on addon list checkbox
                    if self.handle_addon_checkbox_click(pos) {
                        self.frame_cache.clear();
                        self.quads_dirty.set(true);
                    } else {
                        let released_on = self.hit_test(pos);
                        if let Some(frame_id) = released_on {
                            let env = self.env.borrow();
                            let button_val =
                                mlua::Value::String(env.lua().create_string("LeftButton").unwrap());

                            if self.mouse_down_frame == Some(frame_id) {
                                let down_val = mlua::Value::Boolean(false);
                                let _ = env.fire_script_handler(
                                    frame_id,
                                    "OnClick",
                                    vec![button_val.clone(), down_val],
                                );
                            }

                            let _ = env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val]);
                            drop(env);
                            self.drain_console();
                            self.frame_cache.clear();
                            self.quads_dirty.set(true);
                        }
                    }
                    self.mouse_down_frame = None;
                    self.pressed_frame = None;
                }
                CanvasMessage::MiddleClick(pos) => {
                    // Open inspector for the frame under cursor
                    if let Some(frame_id) = self.hit_test(pos) {
                        self.populate_inspector(frame_id);
                        self.inspected_frame = Some(frame_id);
                        self.inspector_visible = true;
                        self.inspector_position = Point::new(pos.x + 10.0, pos.y + 10.0);
                    }
                }
            },
            Message::Scroll(_dx, dy) => {
                let scroll_speed = 30.0;
                // Negate dy: positive dy means scroll up, which should decrease offset
                self.scroll_offset -= dy * scroll_speed;
                let max_scroll = 2600.0;
                self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
                self.frame_cache.clear();
                self.quads_dirty.set(true);
            }
            Message::ReloadUI => {
                self.log_messages.push("Reloading UI...".to_string());
                {
                    let env = self.env.borrow();
                    if let Ok(s) = env.lua().create_string("WoWUISim") {
                        let _ = env.fire_event_with_args("ADDON_LOADED", &[mlua::Value::String(s)]);
                    }
                    let _ = env.fire_event("PLAYER_LOGIN");
                    let _ = env.fire_event_with_args(
                        "PLAYER_ENTERING_WORLD",
                        &[mlua::Value::Boolean(false), mlua::Value::Boolean(true)],
                    );
                }
                self.drain_console();
                self.log_messages.push("UI reloaded.".to_string());
                self.frame_cache.clear();
                self.quads_dirty.set(true);
            }
            Message::CommandInputChanged(input) => {
                self.command_input = input;
            }
            Message::ExecuteCommand => {
                let cmd = self.command_input.clone();
                if !cmd.is_empty() {
                    self.log_messages.push(format!("> {}", cmd));

                    let cmd_lower = cmd.to_lowercase();
                    if cmd_lower == "/frames" || cmd_lower == "/f" {
                        let env = self.env.borrow();
                        let dump = env.dump_frames();
                        eprintln!("{}", dump);
                        let line_count = dump.lines().count();
                        self.log_messages
                            .push(format!("Dumped {} frames to stderr", line_count / 2));
                    } else {
                        let env = self.env.borrow();
                        match env.dispatch_slash_command(&cmd) {
                            Ok(true) => {}
                            Ok(false) => {
                                self.log_messages.push(format!("Unknown command: {}", cmd));
                            }
                            Err(e) => {
                                self.log_messages.push(format!("Command error: {}", e));
                            }
                        }
                    }
                    self.drain_console();
                    self.command_input.clear();
                    self.frame_cache.clear();
                    self.quads_dirty.set(true);
                }
            }
            Message::ProcessTimers => {
                // Update FPS counter (every ~1 second)
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(self.fps_last_time);
                if elapsed >= std::time::Duration::from_secs(1) {
                    let frames = self.frame_count.get();
                    self.fps = frames as f32 / elapsed.as_secs_f32();
                    self.frame_time_display = self.frame_time_avg.get();
                    self.frame_count.set(0);
                    self.fps_last_time = now;
                }

                // Process WoW timers
                let timer_result = {
                    let env = self.env.borrow();
                    env.process_timers()
                };
                match timer_result {
                    Ok(count) if count > 0 => {
                        self.drain_console();
                        self.frame_cache.clear();
                        self.quads_dirty.set(true);
                    }
                    Err(e) => {
                        eprintln!("Timer error: {}", e);
                    }
                    _ => {}
                }

                // Process debug commands (using try_recv in blocking context)
                return self.process_debug_commands();
            }
            Message::ScreenshotTaken(screenshot) => {
                if let Some(respond) = self.pending_screenshot.take() {
                    let data = ScreenshotData {
                        width: screenshot.size.width,
                        height: screenshot.size.height,
                        pixels: screenshot.rgba.to_vec(),
                    };
                    let _ = respond.send(Ok(data));
                }
            }
            Message::FpsTick => {
                // FPS display is updated via ProcessTimers, this is unused
            }
            Message::InspectorClose => {
                self.inspector_visible = false;
                self.inspected_frame = None;
            }
            Message::InspectorWidthChanged(val) => {
                self.inspector_state.width = val;
            }
            Message::InspectorHeightChanged(val) => {
                self.inspector_state.height = val;
            }
            Message::InspectorAlphaChanged(val) => {
                self.inspector_state.alpha = val;
            }
            Message::InspectorLevelChanged(val) => {
                self.inspector_state.frame_level = val;
            }
            Message::InspectorVisibleToggled(val) => {
                self.inspector_state.visible = val;
            }
            Message::InspectorMouseEnabledToggled(val) => {
                self.inspector_state.mouse_enabled = val;
            }
            Message::InspectorApply => {
                if let Some(frame_id) = self.inspected_frame {
                    self.apply_inspector_changes(frame_id);
                    self.frame_cache.clear();
                    self.text_cache.clear();
                    self.quads_dirty.set(true);
                }
            }
            Message::ToggleFramesPanel => {
                self.frames_panel_collapsed = !self.frames_panel_collapsed;
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Title with FPS counter, frame time, canvas size, and mouse coords (raw canvas pixels)
        let mouse_str = match self.mouse_position {
            Some(pos) => format!(" | mouse:({:.0},{:.0})", pos.x, pos.y),
            None => String::new(),
        };
        // Show screen size (WoW logical coords = canvas size)
        let screen = self.screen_size.get();
        let screen_str = format!(" | screen:{}x{}", screen.width as i32, screen.height as i32);
        let title_text = format!(
            "WoW UI Simulator  [{:.1} FPS | {:.2}ms{}{}]",
            self.fps,
            self.frame_time_display,
            screen_str,
            mouse_str
        );
        let title = text(title_text).size(20).color(palette::GOLD);

        // GPU shader rendering with text overlay
        // Layer 1: Shader for textures/backgrounds
        let shader: Shader<Message, &App> = Shader::new(self)
            .width(Length::Fill)
            .height(Length::Fill);

        // Layer 2: Canvas for text and debug overlays (transparent background)
        let text_overlay: Canvas<TextOverlay<'_>, Message> = Canvas::new(TextOverlay::new(self))
            .width(Length::Fill)
            .height(Length::Fill);

        // Stack shader and text overlay, optionally add inspector panel
        let stacked: Element<'_, Message> = if self.inspector_visible {
            let inspector = self.build_inspector_panel();
            stack![shader, text_overlay, inspector].into()
        } else {
            stack![shader, text_overlay].into()
        };

        let render_container = container(stacked)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_DARK)),
                border: Border {
                    color: palette::BORDER_HIGHLIGHT,
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        // Frames sidebar with collapse toggle (floats over canvas)
        let toggle_label = if self.frames_panel_collapsed {
            "▶ Frames"
        } else {
            "▼ Frames"
        };
        let toggle_btn = button(text(toggle_label).size(12))
            .on_press(Message::ToggleFramesPanel)
            .padding([2, 6])
            .style(|_, _| button::Style {
                background: None,
                text_color: palette::TEXT_PRIMARY,
                ..Default::default()
            });

        let sidebar_panel = if self.frames_panel_collapsed {
            container(toggle_btn)
                .padding(6)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(palette::BG_PANEL)),
                    border: Border {
                        color: palette::BORDER,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
        } else {
            let frames_list = self.build_frames_sidebar();
            container(
                column![
                    toggle_btn,
                    scrollable(frames_list)
                        .width(Length::Fill)
                        .height(600),
                ]
                .spacing(4),
            )
            .width(240)
            .padding(6)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_PANEL)),
                border: Border {
                    color: palette::BORDER,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
        };

        // Position sidebar at top-right corner
        let sidebar_positioned = container(sidebar_panel)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right);

        // Stack canvas with floating sidebar
        let content_row = stack![render_container, sidebar_positioned];

        // Event buttons
        let event_buttons = row![
            button(text("ADDON_LOADED").size(12))
                .on_press(Message::FireEvent("ADDON_LOADED".to_string()))
                .style(event_button_style),
            button(text("PLAYER_LOGIN").size(12))
                .on_press(Message::FireEvent("PLAYER_LOGIN".to_string()))
                .style(event_button_style),
            button(text("PLAYER_ENTERING_WORLD").size(12))
                .on_press(Message::FireEvent("PLAYER_ENTERING_WORLD".to_string()))
                .style(event_button_style),
        ]
        .spacing(6);

        // Command input
        let command_row = row![
            text_input("/command", &self.command_input)
                .on_input(Message::CommandInputChanged)
                .on_submit(Message::ExecuteCommand)
                .width(Length::Fill)
                .style(input_style),
            button(text("Run").size(12))
                .on_press(Message::ExecuteCommand)
                .style(run_button_style),
        ]
        .spacing(6);

        // Console output
        let console_text: String = self
            .log_messages
            .iter()
            .rev()
            .take(5)
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        let console = container(
            scrollable(
                text(console_text)
                    .size(12)
                    .color(palette::CONSOLE_TEXT)
                    .font(Font::MONOSPACE),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(80)
        .padding(6)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(palette::BG_INPUT)),
            border: Border {
                color: palette::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

        // Main layout
        let main_column = column![title, content_row, event_buttons, command_row, console,]
            .spacing(5)
            .padding(7);

        container(main_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_DARK)),
                ..Default::default()
            })
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Timer for processing WoW timers and debug commands (~60fps)
        iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::ProcessTimers)
    }

    fn drain_console(&mut self) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        self.log_messages.append(&mut state.console_output);
    }

    fn process_debug_commands(&mut self) -> Task<Message> {
        // Collect debug commands first to avoid borrow issues
        let commands: Vec<_> = if let Some(ref mut rx) = self.debug_rx {
            let mut cmds = Vec::new();
            while let Ok(cmd) = rx.try_recv() {
                cmds.push(cmd);
            }
            cmds
        } else {
            Vec::new()
        };

        // Then handle them, collecting any tasks
        let mut tasks = Vec::new();
        for cmd in commands {
            if let Some(task) = self.handle_debug_command(cmd) {
                tasks.push(task);
            }
        }

        // Process Lua commands
        self.process_lua_commands();

        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    fn process_lua_commands(&mut self) {
        // Collect lua commands
        let commands: Vec<_> = if let Some(ref rx) = self.lua_rx {
            let mut cmds = Vec::new();
            while let Ok(cmd) = rx.try_recv() {
                cmds.push(cmd);
            }
            cmds
        } else {
            Vec::new()
        };

        // Handle each command
        for cmd in commands {
            match cmd {
                LuaCommand::Exec { code, respond } => {
                    // Clear console output before execution
                    {
                        let env = self.env.borrow();
                        env.state().borrow_mut().console_output.clear();
                    }

                    // Execute the Lua code
                    let result = {
                        let env = self.env.borrow();
                        env.exec(&code)
                    };

                    // Collect output and send response
                    let response = match result {
                        Ok(()) => {
                            let env = self.env.borrow();
                            let mut state = env.state().borrow_mut();
                            let output = state.console_output.join("\n");
                            state.console_output.clear();
                            LuaResponse::Output(output)
                        }
                        Err(e) => LuaResponse::Error(e.to_string()),
                    };

                    let _ = respond.send(response);

                    // Refresh display
                    self.drain_console();
                    self.frame_cache.clear();
                    self.quads_dirty.set(true);
                }
                LuaCommand::DumpTree { filter, visible_only, respond } => {
                    let tree = self.build_frame_tree_dump(filter.as_deref(), visible_only);
                    let _ = respond.send(LuaResponse::Tree(tree));
                }
            }
        }
    }

    fn handle_debug_command(&mut self, cmd: DebugCommand) -> Option<Task<Message>> {
        match cmd {
            DebugCommand::Dump { respond } => {
                let dump = self.dump_wow_frames();
                let _ = respond.send(dump);
                None
            }
            DebugCommand::Click { label, respond } => {
                let _ = respond.send(Err(format!("Click not implemented for '{}'", label)));
                None
            }
            DebugCommand::Input {
                field,
                value: _,
                respond,
            } => {
                let _ = respond.send(Err(format!("Input not implemented for '{}'", field)));
                None
            }
            DebugCommand::Submit { respond } => {
                let _ = respond.send(Err("Submit not implemented".to_string()));
                None
            }
            DebugCommand::Screenshot { respond } => {
                // Store the responder and initiate screenshot
                self.pending_screenshot = Some(respond);
                Some(
                    window::latest()
                        .and_then(window::screenshot)
                        .map(Message::ScreenshotTaken),
                )
            }
        }
    }

    fn dump_wow_frames(&self) -> String {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let screen_width = self.screen_size.get().width;
        let screen_height = self.screen_size.get().height;

        let mut lines = Vec::new();
        lines.push("WoW UI Simulator - Frame Dump".to_string());
        lines.push(format!("Screen: {}x{}", screen_width as i32, screen_height as i32));
        lines.push(String::new());

        // Find root frames (no parent or parent is UIParent)
        let mut root_ids: Vec<u64> = state
            .widgets
            .all_ids()
            .into_iter()
            .filter(|&id| {
                state
                    .widgets
                    .get(id)
                    .map(|f| f.parent_id.is_none() || f.parent_id == Some(1))
                    .unwrap_or(false)
            })
            .collect();
        root_ids.sort();

        for id in root_ids {
            self.dump_frame_recursive(&state.widgets, id, 0, screen_width, screen_height, &mut lines);
        }

        lines.join("\n")
    }

    fn dump_frame_recursive(
        &self,
        registry: &crate::widget::WidgetRegistry,
        id: u64,
        depth: usize,
        screen_width: f32,
        screen_height: f32,
        lines: &mut Vec<String>,
    ) {
        let Some(frame) = registry.get(id) else {
            return;
        };

        let rect = compute_frame_rect(registry, id, screen_width, screen_height);
        let indent = "  ".repeat(depth);

        let name = frame.name.as_deref().unwrap_or("(anon)");
        let type_str = frame.widget_type.as_str();

        // Build warning flags
        let mut warnings = Vec::new();
        if rect.width <= 0.0 {
            warnings.push("ZERO_WIDTH");
        }
        if rect.height <= 0.0 {
            warnings.push("ZERO_HEIGHT");
        }
        if rect.x + rect.width < 0.0 || rect.x > screen_width {
            warnings.push("OFFSCREEN_X");
        }
        if rect.y + rect.height < 0.0 || rect.y > screen_height {
            warnings.push("OFFSCREEN_Y");
        }
        if !frame.visible {
            warnings.push("HIDDEN");
        }

        let warning_str = if warnings.is_empty() {
            String::new()
        } else {
            format!(" ! {}", warnings.join(", "))
        };

        lines.push(format!(
            "{}{} [{}] ({:.0},{:.0} {}x{}){}",
            indent, name, type_str,
            rect.x, rect.y, rect.width as i32, rect.height as i32,
            warning_str
        ));

        // Recurse into children
        for &child_id in &frame.children {
            self.dump_frame_recursive(registry, child_id, depth + 1, screen_width, screen_height, lines);
        }
    }

    /// Build a frame tree dump with absolute screen coordinates (WoW units).
    fn build_frame_tree_dump(&self, filter: Option<&str>, visible_only: bool) -> String {
        let env = self.env.borrow();
        let state = env.state().borrow();
        // Use WoW logical screen size for layout calculation
        let screen_width = self.screen_size.get().width;
        let screen_height = self.screen_size.get().height;

        let mut lines = Vec::new();

        // Find root frames (no parent) - UIParent children are shown under UIParent
        let mut root_ids: Vec<u64> = state
            .widgets
            .all_ids()
            .into_iter()
            .filter(|&id| {
                state
                    .widgets
                    .get(id)
                    .map(|f| f.parent_id.is_none())
                    .unwrap_or(false)
            })
            .collect();
        root_ids.sort();

        for id in root_ids {
            self.build_tree_recursive(
                &state.widgets,
                id,
                "",
                true,
                screen_width,
                screen_height,
                filter,
                visible_only,
                &mut lines,
            );
        }

        if lines.is_empty() {
            "No frames found".to_string()
        } else {
            lines.join("\n")
        }
    }

    fn build_tree_recursive(
        &self,
        registry: &crate::widget::WidgetRegistry,
        id: u64,
        prefix: &str,
        is_last: bool,
        screen_width: f32,
        screen_height: f32,
        filter: Option<&str>,
        visible_only: bool,
        lines: &mut Vec<String>,
    ) {
        let Some(frame) = registry.get(id) else {
            return;
        };

        // Check visibility filter
        if visible_only && !frame.visible {
            return;
        }

        // Check name filter - use truncated text for anonymous frames with text
        let raw_name = frame.name.as_deref();
        let is_anon = raw_name.map(|n| n.starts_with("__anon_") || n.starts_with("__fs_") || n.starts_with("__tex_")).unwrap_or(true);
        let name = if is_anon && frame.text.is_some() {
            let text = frame.text.as_ref().unwrap();
            // Return truncated text for display (stored in a leaked string for lifetime)
            if text.len() > 20 {
                Box::leak(format!("\"{}...\"", &text[..17]).into_boxed_str())
            } else {
                Box::leak(format!("\"{}\"", text).into_boxed_str())
            }
        } else {
            raw_name.unwrap_or("(anon)")
        };
        let matches_filter = filter.map(|f| name.to_lowercase().contains(&f.to_lowercase())).unwrap_or(true);

        // Compute absolute coordinates in WoW units (not scaled for display)
        let rect = compute_frame_rect(registry, id, screen_width, screen_height);
        let abs_x = rect.x;
        let abs_y = rect.y;
        let abs_w = rect.width;
        let abs_h = rect.height;

        let type_str = frame.widget_type.as_str();
        let vis_str = if frame.visible { "" } else { " [hidden]" };

        // Get children that match the filter
        let mut children: Vec<u64> = frame.children.iter().copied().collect();
        if filter.is_some() || visible_only {
            children.retain(|&child_id| {
                self.subtree_matches(registry, child_id, screen_width, screen_height, filter, visible_only)
            });
        }

        // Only output if matches filter or has matching children
        if matches_filter || !children.is_empty() {
            let connector = if is_last { "└─ " } else { "├─ " };
            // Show size mismatch if stored size differs from computed
            let size_info = if (frame.width - rect.width).abs() > 0.1 || (frame.height - rect.height).abs() > 0.1 {
                format!(" [stored={:.0}x{:.0}]", frame.width, frame.height)
            } else {
                String::new()
            };
            lines.push(format!(
                "{}{}{} ({}) @ ({:.0},{:.0}) {:.0}x{:.0}{}{}",
                prefix, connector, name, type_str, abs_x, abs_y, abs_w, abs_h, size_info, vis_str
            ));

            // Show anchor information with computed absolute coordinates
            let child_prefix = format!("{}{}", prefix, if is_last { "   " } else { "│  " });

            // Get parent rect for anchor calculations
            let parent_rect = if let Some(parent_id) = frame.parent_id {
                compute_frame_rect(registry, parent_id, screen_width, screen_height)
            } else {
                LayoutRect {
                    x: 0.0,
                    y: 0.0,
                    width: screen_width,
                    height: screen_height,
                }
            };

            for anchor in &frame.anchors {
                let (rel_name, relative_rect) = if let Some(rel_id) = anchor.relative_to_id {
                    let rel_rect = compute_frame_rect(registry, rel_id as u64, screen_width, screen_height);
                    let name = registry.get(rel_id as u64)
                        .and_then(|f| f.name.as_deref())
                        .unwrap_or("(anon)");
                    (name, rel_rect)
                } else {
                    (anchor.relative_to.as_deref().unwrap_or("$parent"), parent_rect)
                };

                // Calculate the absolute position where this anchor resolves to
                let (anchor_x, anchor_y) = anchor_position(
                    anchor.relative_point,
                    relative_rect.x,
                    relative_rect.y,
                    relative_rect.width,
                    relative_rect.height,
                );
                let target_x = anchor_x + anchor.x_offset;
                let target_y = anchor_y - anchor.y_offset;

                lines.push(format!(
                    "{}   📍 {} → {}:{} offset({:.0},{:.0}) → ({:.0},{:.0})",
                    child_prefix,
                    anchor.point.as_str(),
                    rel_name,
                    anchor.relative_point.as_str(),
                    anchor.x_offset,
                    anchor.y_offset,
                    target_x,
                    target_y
                ));
            }

            // Show texture path for Texture widgets
            if let Some(tex_path) = &frame.texture {
                lines.push(format!("{}   🖼️ {}", child_prefix, tex_path));
            }

            // Recurse into children with updated prefix
            for (i, &child_id) in children.iter().enumerate() {
                let is_last_child = i == children.len() - 1;
                self.build_tree_recursive(
                    registry,
                    child_id,
                    &child_prefix,
                    is_last_child,
                    screen_width,
                    screen_height,
                    filter,
                    visible_only,
                    lines,
                );
            }
        }
    }

    /// Check if a frame or any descendant matches the filter criteria.
    fn subtree_matches(
        &self,
        registry: &crate::widget::WidgetRegistry,
        id: u64,
        screen_width: f32,
        screen_height: f32,
        filter: Option<&str>,
        visible_only: bool,
    ) -> bool {
        let Some(frame) = registry.get(id) else {
            return false;
        };

        if visible_only && !frame.visible {
            return false;
        }

        let name = frame.name.as_deref().unwrap_or("(anon)");
        let matches = filter.map(|f| name.to_lowercase().contains(&f.to_lowercase())).unwrap_or(true);

        if matches {
            return true;
        }

        // Check children
        for &child_id in &frame.children {
            if self.subtree_matches(registry, child_id, screen_width, screen_height, filter, visible_only) {
                return true;
            }
        }

        false
    }

    fn build_frames_sidebar(&self) -> Column<'_, Message> {
        let mut col = Column::new().spacing(2);

        let env = self.env.borrow();
        let state = env.state().borrow();

        let mut count = 0;
        for id in state.widgets.all_ids() {
            if let Some(frame) = state.widgets.get(id) {
                let name = match &frame.name {
                    Some(n) => n.as_str(),
                    None => continue,
                };
                if name.starts_with("__")
                    || name.starts_with("DBM")
                    || name.starts_with("Details")
                    || name.starts_with("Avatar")
                    || name.starts_with("Plater")
                    || name.starts_with("WeakAuras")
                    || name.starts_with("UIWidget")
                    || name.starts_with("GameMenu")
                {
                    continue;
                }
                if frame.width <= 0.0 || frame.height <= 0.0 {
                    continue;
                }

                let visible = if frame.visible { "visible" } else { "hidden" };
                let display = format!(
                    "{} [{}] {}x{} ({})",
                    name,
                    frame.widget_type.as_str(),
                    frame.width as i32,
                    frame.height as i32,
                    visible
                );

                let display = if display.len() > 30 {
                    format!("{}...", &display[..27])
                } else {
                    display
                };

                col = col.push(text(display).size(14).color(palette::TEXT_MUTED));

                count += 1;
                if count >= 15 {
                    break;
                }
            }
        }

        col
    }

    fn hit_test(&self, pos: Point) -> Option<u64> {
        let env = self.env.borrow();
        let state = env.state().borrow();

        let scale_x = UI_SCALE;
        let scale_y = UI_SCALE;

        // Use WoW logical screen size for layout calculation
        let screen_width = self.screen_size.get().width;
        let screen_height = self.screen_size.get().height;

        let mut frames: Vec<_> = state
            .widgets
            .all_ids()
            .into_iter()
            .filter_map(|id| {
                let frame = state.widgets.get(id)?;
                if !frame.visible || !frame.mouse_enabled {
                    return None;
                }
                if matches!(
                    frame.name.as_deref(),
                    Some("UIParent")
                        | Some("Minimap")
                        | Some("WorldFrame")
                        | Some("DEFAULT_CHAT_FRAME")
                        | Some("ChatFrame1")
                        | Some("EventToastManagerFrame")
                        | Some("EditModeManagerFrame")
                ) {
                    return None;
                }
                let rect = compute_frame_rect(
                    &state.widgets,
                    id,
                    screen_width,
                    screen_height,
                );
                Some((id, frame.frame_strata, frame.frame_level, rect))
            })
            .collect();

        frames.sort_by(|a, b| {
            a.1.cmp(&b.1)
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| a.0.cmp(&b.0))
        });

        for (id, _, _, rect) in frames.iter().rev() {
            let scaled_x = rect.x * scale_x;
            let scaled_y = rect.y * scale_y;
            let scaled_w = rect.width * scale_x;
            let scaled_h = rect.height * scale_y;

            if pos.x >= scaled_x
                && pos.x <= scaled_x + scaled_w
                && pos.y >= scaled_y
                && pos.y <= scaled_y + scaled_h
            {
                return Some(*id);
            }
        }
        None
    }

    /// Populate inspector state from a frame's properties.
    fn populate_inspector(&mut self, frame_id: u64) {
        let env = self.env.borrow();
        let state = env.state().borrow();
        if let Some(frame) = state.widgets.get(frame_id) {
            self.inspector_state = InspectorState {
                width: format!("{:.0}", frame.width),
                height: format!("{:.0}", frame.height),
                alpha: format!("{:.2}", frame.alpha),
                frame_level: format!("{}", frame.frame_level),
                visible: frame.visible,
                mouse_enabled: frame.mouse_enabled,
            };
        }
    }

    /// Apply inspector changes to the frame.
    fn apply_inspector_changes(&mut self, frame_id: u64) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        if let Some(frame) = state.widgets.get_mut(frame_id) {
            if let Ok(w) = self.inspector_state.width.parse::<f32>() {
                frame.width = w;
            }
            if let Ok(h) = self.inspector_state.height.parse::<f32>() {
                frame.height = h;
            }
            if let Ok(a) = self.inspector_state.alpha.parse::<f32>() {
                frame.alpha = a.clamp(0.0, 1.0);
            }
            if let Ok(l) = self.inspector_state.frame_level.parse::<i32>() {
                frame.frame_level = l;
            }
            frame.visible = self.inspector_state.visible;
            frame.mouse_enabled = self.inspector_state.mouse_enabled;
        }
    }

    /// Build the inspector panel widget.
    fn build_inspector_panel(&self) -> Element<'_, Message> {
        use iced::widget::{checkbox, space, Container};
        use iced::Padding;

        let env = self.env.borrow();
        let state = env.state().borrow();

        let frame_id = self.inspected_frame.unwrap_or(0);
        let frame = state.widgets.get(frame_id);

        // Header with frame info
        let (name, widget_type, computed_rect) = match frame {
            Some(f) => {
                let rect = compute_frame_rect(
                    &state.widgets,
                    frame_id,
                    self.screen_size.get().width,
                    self.screen_size.get().height,
                );
                (
                    f.name.clone().unwrap_or_else(|| "(anon)".to_string()),
                    f.widget_type.as_str().to_string(),
                    rect,
                )
            }
            None => ("(none)".to_string(), "".to_string(), LayoutRect::default()),
        };

        // Title bar with close button
        let title = row![
            text(format!("{} [{}]", name, widget_type))
                .size(14)
                .color(palette::GOLD),
            space::horizontal(),
            button(text("×").size(14))
                .on_press(Message::InspectorClose)
                .padding(2)
                .style(|_, _| button::Style {
                    background: Some(iced::Background::Color(Color::TRANSPARENT)),
                    text_color: palette::TEXT_SECONDARY,
                    ..Default::default()
                }),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center);

        // ID and position (read-only)
        let id_row = text(format!("ID: {}  Pos: ({:.0}, {:.0})", frame_id, computed_rect.x, computed_rect.y))
            .size(11)
            .color(palette::TEXT_SECONDARY);

        // Width/Height inputs
        let size_row = row![
            text("W:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.width)
                .on_input(Message::InspectorWidthChanged)
                .size(11)
                .width(50),
            text("H:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.height)
                .on_input(Message::InspectorHeightChanged)
                .size(11)
                .width(50),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center);

        // Alpha and Level inputs
        let alpha_level_row = row![
            text("Alpha:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.alpha)
                .on_input(Message::InspectorAlphaChanged)
                .size(11)
                .width(40),
            text("Level:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.frame_level)
                .on_input(Message::InspectorLevelChanged)
                .size(11)
                .width(40),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center);

        // Checkboxes
        let checkbox_row = row![
            checkbox(self.inspector_state.visible)
                .label("Visible")
                .on_toggle(Message::InspectorVisibleToggled)
                .size(14)
                .text_size(11),
            checkbox(self.inspector_state.mouse_enabled)
                .label("Mouse")
                .on_toggle(Message::InspectorMouseEnabledToggled)
                .size(14)
                .text_size(11),
        ]
        .spacing(10);

        // Anchors display (read-only)
        let anchors_text = match frame {
            Some(f) if !f.anchors.is_empty() => {
                let anchor_strs: Vec<String> = f
                    .anchors
                    .iter()
                    .map(|a| {
                        let rel = a.relative_to.as_deref().unwrap_or("$parent");
                        format!(
                            "{:?}→{} {:?} ({:.0},{:.0})",
                            a.point, rel, a.relative_point, a.x_offset, a.y_offset
                        )
                    })
                    .collect();
                anchor_strs.join("\n")
            }
            _ => "No anchors".to_string(),
        };
        let anchors_display = text(anchors_text).size(10).color(palette::TEXT_MUTED);

        // Apply button
        let apply_btn = button(text("Apply").size(12))
            .on_press(Message::InspectorApply)
            .padding(Padding::from([4, 12]));

        let content = column![
            title,
            id_row,
            size_row,
            alpha_level_row,
            checkbox_row,
            text("Anchors:").size(11).color(palette::TEXT_SECONDARY),
            anchors_display,
            apply_btn,
        ]
        .spacing(6)
        .padding(8);

        let panel: Container<'_, Message> = container(content)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_PANEL)),
                border: Border {
                    color: palette::BORDER_HIGHLIGHT,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .width(220);

        // Position the panel at the inspector_position
        // We use a container with padding to offset the panel
        let x_pad = self.inspector_position.x.max(0.0);
        let y_pad = self.inspector_position.y.max(0.0);

        container(panel)
            .padding(Padding::new(0.0).top(y_pad).left(x_pad))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Canvas program implementation for text overlay.
///
/// This renders only FontString widgets with a transparent background,
/// allowing it to be layered on top of the shader which handles textures.
impl canvas::Program<Message> for TextOverlay<'_> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // Text overlay doesn't handle events - the shader layer handles them
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.app.text_cache.draw(renderer, bounds.size(), |frame| {
            // Transparent background - let shader show through
            // Pass canvas origin offset so coordinates can be adjusted
            self.app.draw_text_overlay(frame, bounds.size());
        });

        vec![geometry]
    }
}

/// Shader program implementation for GPU rendering of WoW frames.
impl shader::Program<Message> for &App {
    type State = ();
    type Primitive = WowUiPrimitive;

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { position } => {
                    if bounds.contains(*position) {
                        let local = Point::new(position.x - bounds.x, position.y - bounds.y);
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseMove(local),
                        )));
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseDown(pos),
                        )));
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseUp(pos),
                        )));
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MiddleClick(pos),
                        )));
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    let dy = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => *y,
                        mouse::ScrollDelta::Pixels { y, .. } => *y / 30.0,
                    };
                    return Some(shader::Action::publish(Message::Scroll(0.0, dy)));
                }
                _ => {}
            },
            Event::Keyboard(keyboard_event) => {
                use iced::keyboard;
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = keyboard_event {
                    if modifiers.control() && *key == keyboard::Key::Character("r".into()) {
                        return Some(shader::Action::publish(Message::ReloadUI));
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        let start = std::time::Instant::now();

        // Increment frame counter for FPS calculation
        self.frame_count.set(self.frame_count.get() + 1);

        // Update screen size from canvas bounds (used by other functions)
        let size = bounds.size();
        self.screen_size.set(size);
        let mut cache = self.cached_quads.borrow_mut();
        let quads = if self.quads_dirty.get() || cache.as_ref().map(|(s, _)| *s != size).unwrap_or(true) {
            // Rebuild quad batch
            let new_quads = self.build_quad_batch(size);
            *cache = Some((size, new_quads.clone()));
            self.quads_dirty.set(false);
            new_quads
        } else {
            // Use cached quads
            cache.as_ref().unwrap().1.clone()
        };

        // Load ONLY NEW textures (skip ones already uploaded to GPU atlas)
        let mut textures = Vec::new();
        let mut uploaded = self.gpu_uploaded_textures.borrow_mut();
        {
            let mut tex_mgr = self.texture_manager.borrow_mut();
            for request in &quads.texture_requests {
                // Skip if already uploaded to GPU
                if uploaded.contains(&request.path) {
                    continue;
                }
                // Skip if already in this batch
                if textures.iter().any(|t: &GpuTextureData| t.path == request.path) {
                    continue;
                }
                if let Some(tex_data) = tex_mgr.load(&request.path) {
                    textures.push(GpuTextureData {
                        path: request.path.clone(),
                        width: tex_data.width,
                        height: tex_data.height,
                        rgba: tex_data.pixels.clone(),
                    });
                    // Mark as uploaded (will be uploaded in prepare())
                    uploaded.insert(request.path.clone());
                }
            }
        }

        // Update frame time with EMA (alpha = 0.33 for ~5 sample smoothing)
        let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
        self.frame_time_ms.set(elapsed_ms);
        let avg = self.frame_time_avg.get();
        self.frame_time_avg.set(0.33 * elapsed_ms + 0.67 * avg);

        WowUiPrimitive::with_textures(quads, textures)
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl App {
    /// Build a QuadBatch for GPU shader rendering.
    ///
    /// This mirrors build_quad_batch but builds a QuadBatch instead of drawing to canvas.
    /// Currently renders solid colors only - texture support requires atlas integration.
    fn build_quad_batch(&self, size: Size) -> QuadBatch {
        let mut batch = QuadBatch::with_capacity(1000);

        let env = self.env.borrow();
        let state = env.state().borrow();

        // Use canvas size for layout - WoW coords map 1:1 to canvas pixels
        let screen_width = size.width;
        let screen_height = size.height;

        // Add background quad first (replaces LoadOp::Clear to preserve iced UI)
        batch.push_solid(
            Rectangle::new(Point::ORIGIN, size),
            [palette::BG_DARK.r, palette::BG_DARK.g, palette::BG_DARK.b, 1.0],
        );

        // Find AddonList frame and collect descendant IDs
        let mut addonlist_ids = std::collections::HashSet::new();
        let addonlist_id = state.widgets.all_ids().into_iter().find(|&id| {
            state
                .widgets
                .get(id)
                .map(|f| f.name.as_deref() == Some("AddonList"))
                .unwrap_or(false)
        });

        if let Some(root_id) = addonlist_id {
            let mut queue = vec![root_id];
            while let Some(id) = queue.pop() {
                addonlist_ids.insert(id);
                if let Some(f) = state.widgets.get(id) {
                    queue.extend(f.children.iter().copied());
                }
            }
        }

        // Collect and sort frames (same sorting as build_quad_batch)
        let mut frames: Vec<_> = state
            .widgets
            .all_ids()
            .into_iter()
            .filter_map(|id| {
                let f = state.widgets.get(id)?;
                let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
                Some((id, f, rect))
            })
            .collect();

        frames.sort_by(|a, b| {
            a.1.frame_strata
                .cmp(&b.1.frame_strata)
                .then_with(|| a.1.frame_level.cmp(&b.1.frame_level))
                .then_with(|| {
                    let is_region = |t: &WidgetType| {
                        matches!(t, WidgetType::Texture | WidgetType::FontString)
                    };
                    match (is_region(&a.1.widget_type), is_region(&b.1.widget_type)) {
                        (true, true) => a.1.draw_layer.cmp(&b.1.draw_layer)
                            .then_with(|| a.1.draw_sub_layer.cmp(&b.1.draw_sub_layer)),
                        (false, true) => std::cmp::Ordering::Less,
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, false) => std::cmp::Ordering::Equal,
                    }
                })
                .then_with(|| a.0.cmp(&b.0))
        });

        for (id, f, rect) in frames {
            // Only show AddonList frame and children
            if !addonlist_ids.contains(&id) {
                continue;
            }
            if !f.visible {
                continue;
            }
            if rect.width <= 0.0 || rect.height <= 0.0 {
                continue;
            }

            let x = rect.x * UI_SCALE;
            let y = rect.y * UI_SCALE;
            let w = rect.width * UI_SCALE;
            let h = rect.height * UI_SCALE;
            let bounds = Rectangle::new(Point::new(x, y), Size::new(w, h));

            match f.widget_type {
                WidgetType::Frame => {
                    self.build_frame_quads(&mut batch, bounds, f);
                }
                WidgetType::Button => {
                    let is_pressed = self.pressed_frame == Some(id);
                    let is_hovered = self.hovered_frame == Some(id);
                    self.build_button_quads(&mut batch, bounds, f, is_pressed, is_hovered);
                }
                WidgetType::Texture => {
                    self.build_texture_quads(&mut batch, bounds, f);
                }
                WidgetType::FontString => {
                    // Text is handled separately (not in quad batch)
                    // For now, skip - will use iced text overlay
                }
                WidgetType::CheckButton => {
                    let is_pressed = self.pressed_frame == Some(id);
                    let is_hovered = self.hovered_frame == Some(id);
                    self.build_button_quads(&mut batch, bounds, f, is_pressed, is_hovered);
                }
                WidgetType::EditBox => {
                    self.build_editbox_quads(&mut batch, bounds, f);
                }
                _ => {}
            }
        }

        batch
    }

    /// Build quads for a Frame widget (backdrop).
    fn build_frame_quads(&self, batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
        // Draw backdrop if enabled
        if f.backdrop.enabled {
            let bg = &f.backdrop.bg_color;
            batch.push_solid(bounds, [bg.r, bg.g, bg.b, bg.a * f.alpha]);

            // Border
            if f.backdrop.edge_size > 0.0 {
                let bc = &f.backdrop.border_color;
                batch.push_border(bounds, f.backdrop.edge_size.max(1.0), [bc.r, bc.g, bc.b, bc.a * f.alpha]);
            }
        }

        // NineSlice rendering - for now, just draw a placeholder border
        if f.nine_slice_layout.is_some() {
            // TODO: Implement NineSlice with texture atlas
            // For now, draw a gold border to indicate NineSlice frame
            batch.push_border(bounds, 2.0, [0.6, 0.45, 0.15, f.alpha]);
        }
    }

    /// Build quads for a Button widget.
    fn build_button_quads(
        &self,
        batch: &mut QuadBatch,
        bounds: Rectangle,
        f: &crate::widget::Frame,
        is_pressed: bool,
        is_hovered: bool,
    ) {
        // Determine which texture to use based on state
        let texture_path = if is_pressed {
            f.pushed_texture.as_ref().or(f.normal_texture.as_ref())
        } else {
            f.normal_texture.as_ref()
        };

        // Render button texture or fallback to solid color
        if let Some(tex_path) = texture_path {
            // WoW button textures are 128x32 with thin gold borders (~3px)
            // Use 3-slice rendering to preserve the end caps while stretching the middle
            const BUTTON_TEX_WIDTH: f32 = 128.0;
            const BUTTON_CAP_WIDTH: f32 = 4.0;
            batch.push_three_slice_h_path(
                bounds,
                BUTTON_CAP_WIDTH,
                BUTTON_CAP_WIDTH,
                tex_path,
                BUTTON_TEX_WIDTH,
                [1.0, 1.0, 1.0, f.alpha],
            );
        } else {
            // Fallback solid color
            let bg_color = if is_pressed {
                [0.20, 0.08, 0.08, 0.95 * f.alpha]
            } else if is_hovered {
                [0.18, 0.07, 0.07, 0.95 * f.alpha]
            } else {
                [0.15, 0.05, 0.05, 0.95 * f.alpha]
            };
            batch.push_solid(bounds, bg_color);

            // Border for solid color fallback
            let border_color = if is_hovered || is_pressed {
                [0.8, 0.6, 0.2, f.alpha]
            } else {
                [0.6, 0.45, 0.15, f.alpha]
            };
            batch.push_border(bounds, 1.5, border_color);
        }

        // Highlight texture overlay on hover (also uses 3-slice since it's 128x32)
        if is_hovered && !is_pressed {
            if let Some(ref highlight_path) = f.highlight_texture {
                const BUTTON_TEX_WIDTH: f32 = 128.0;
                const BUTTON_CAP_WIDTH: f32 = 4.0;
                batch.push_three_slice_h_path_blend(
                    bounds,
                    BUTTON_CAP_WIDTH,
                    BUTTON_CAP_WIDTH,
                    highlight_path,
                    BUTTON_TEX_WIDTH,
                    [1.0, 1.0, 1.0, 0.5 * f.alpha],
                    BlendMode::Additive,
                );
            } else {
                // Fallback highlight
                batch.push_quad(
                    bounds,
                    Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0)),
                    [1.0, 0.9, 0.6, 0.15 * f.alpha],
                    -1,
                    BlendMode::Additive,
                );
            }
        }
    }

    /// Build quads for a Texture widget.
    fn build_texture_quads(&self, batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
        // Color texture
        if let Some(color) = f.color_texture {
            batch.push_solid(bounds, [color.r, color.g, color.b, color.a * f.alpha]);
            return;
        }

        // File texture
        if let Some(ref tex_path) = f.texture {
            // Use white tint by default, let the texture show through
            batch.push_textured_path(
                bounds,
                tex_path,
                [1.0, 1.0, 1.0, f.alpha],
                BlendMode::Alpha,
            );
        }
    }

    /// Build quads for an EditBox widget.
    fn build_editbox_quads(&self, batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
        // Background
        batch.push_solid(bounds, [0.06, 0.06, 0.08, 0.9 * f.alpha]);
        // Border
        batch.push_border(bounds, 1.0, [0.3, 0.25, 0.15, f.alpha]);
    }

    /// Draw text elements and debug overlays (borders, anchor points).
    fn draw_text_overlay(&self, frame: &mut canvas::Frame, size: Size) {
        let env = self.env.borrow();
        let state = env.state().borrow();

        let screen_width = size.width;
        let screen_height = size.height;

        // Find AddonList frame and collect descendant IDs (same as build_quad_batch)
        let mut addonlist_ids = std::collections::HashSet::new();
        let addonlist_id = state.widgets.all_ids().into_iter().find(|&id| {
            state
                .widgets
                .get(id)
                .map(|f| f.name.as_deref() == Some("AddonList"))
                .unwrap_or(false)
        });

        if let Some(root_id) = addonlist_id {
            let mut queue = vec![root_id];
            while let Some(id) = queue.pop() {
                addonlist_ids.insert(id);
                if let Some(f) = state.widgets.get(id) {
                    queue.extend(f.children.iter().copied());
                }
            }
        }

        // Collect and sort frames (same sorting as build_quad_batch)
        let mut frames: Vec<_> = state
            .widgets
            .all_ids()
            .into_iter()
            .filter_map(|id| {
                let f = state.widgets.get(id)?;
                let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
                Some((id, f, rect))
            })
            .collect();

        frames.sort_by(|a, b| {
            a.1.frame_strata
                .cmp(&b.1.frame_strata)
                .then_with(|| a.1.frame_level.cmp(&b.1.frame_level))
                .then_with(|| {
                    let is_region = |t: &WidgetType| {
                        matches!(t, WidgetType::Texture | WidgetType::FontString)
                    };
                    match (is_region(&a.1.widget_type), is_region(&b.1.widget_type)) {
                        (true, true) => {
                            a.1.draw_layer
                                .cmp(&b.1.draw_layer)
                                .then_with(|| a.1.draw_sub_layer.cmp(&b.1.draw_sub_layer))
                        }
                        (false, true) => std::cmp::Ordering::Less,
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, false) => std::cmp::Ordering::Equal,
                    }
                })
                .then_with(|| a.0.cmp(&b.0))
        });

        // Draw only text elements
        for (id, f, rect) in frames {
            if !addonlist_ids.contains(&id) {
                continue;
            }
            if !f.visible {
                continue;
            }
            if rect.width <= 0.0 || rect.height <= 0.0 {
                continue;
            }

            let x = rect.x * UI_SCALE;
            let y = rect.y * UI_SCALE;
            let w = rect.width * UI_SCALE;
            let h = rect.height * UI_SCALE;
            let bounds = Rectangle::new(Point::new(x, y), Size::new(w, h));

            match f.widget_type {
                WidgetType::FontString => {
                    self.draw_fontstring_widget(frame, bounds, f);
                }
                WidgetType::Button => {
                    self.draw_button_text(frame, bounds, f);
                }
                WidgetType::EditBox => {
                    // Draw editbox text only (no background)
                    self.draw_editbox_text(frame, bounds, f);
                }
                WidgetType::CheckButton => {
                    // Draw checkbox label text only
                    self.draw_checkbutton_text(frame, bounds, f);
                }
                _ => {}
            }

            // Draw debug border if enabled
            if self.debug_borders {
                let border_path = Path::rectangle(bounds.position(), bounds.size());
                frame.stroke(
                    &border_path,
                    Stroke::default()
                        .with_color(Color::from_rgb(1.0, 0.0, 0.0))
                        .with_width(1.0),
                );
            }

            // Draw debug anchor points if enabled
            if self.debug_anchors {
                let anchor_color = Color::from_rgb(0.0, 1.0, 0.0);
                let dot_radius = 4.0;

                for anchor in &f.anchors {
                    // Get the position of this anchor point on the element
                    let (ax, ay) = anchor_position(
                        anchor.point,
                        bounds.x,
                        bounds.y,
                        bounds.width,
                        bounds.height,
                    );
                    // Draw a filled circle at the anchor point
                    let dot = Path::circle(Point::new(ax, ay), dot_radius);
                    frame.fill(&dot, anchor_color);
                }
            }
        }
    }

    /// Draw only the text portion of a button (for text overlay).
    fn draw_button_text(&self, frame: &mut canvas::Frame, bounds: Rectangle, f: &crate::widget::Frame) {
        if let Some(ref txt) = f.text {
            let clean_text = strip_wow_markup(txt);
            TextRenderer::draw_justified_text(
                frame,
                &clean_text,
                bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                wow_font_to_iced(f.font.as_deref()),
                TextJustify::Center,
                TextJustify::Center,
            );
        }
    }

    /// Draw only the text portion of an editbox (for text overlay).
    fn draw_editbox_text(&self, frame: &mut canvas::Frame, bounds: Rectangle, f: &crate::widget::Frame) {
        if let Some(ref txt) = f.text {
            let padding = 4.0;
            let text_bounds = Rectangle::new(
                Point::new(bounds.x + padding, bounds.y),
                Size::new(bounds.width - padding * 2.0, bounds.height),
            );
            TextRenderer::draw_justified_text(
                frame,
                txt,
                text_bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                Font::DEFAULT,
                TextJustify::Left,
                TextJustify::Center,
            );
        }
    }

    /// Draw only the text portion of a checkbox (for text overlay).
    fn draw_checkbutton_text(&self, frame: &mut canvas::Frame, bounds: Rectangle, f: &crate::widget::Frame) {
        // Most checkbuttons have their label as a separate FontString child, not f.text
        // This handles any direct text on the checkbox itself
        if let Some(ref txt) = f.text {
            let clean_text = strip_wow_markup(txt);
            let label_x = bounds.x + 20.0; // Offset past checkbox
            let label_bounds = Rectangle::new(
                Point::new(label_x, bounds.y),
                Size::new(bounds.width - 20.0, bounds.height),
            );
            TextRenderer::draw_justified_text(
                frame,
                &clean_text,
                label_bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                wow_font_to_iced(f.font.as_deref()),
                TextJustify::Left,
                TextJustify::Center,
            );
        }
    }

    /// Draw addon list entries text only (for text overlay).
    fn draw_addon_list_entries_text(
        &self,
        frame: &mut canvas::Frame,
        addon_list_rect: &LayoutRect,
        state: &crate::lua_api::SimState,
    ) {
        let addons = &state.addons;
        if addons.is_empty() {
            return;
        }

        // Content area bounds (same as draw_addon_list_entries)
        let list_x = addon_list_rect.x * UI_SCALE;
        let list_y = addon_list_rect.y * UI_SCALE;
        let list_width = addon_list_rect.width * UI_SCALE;
        let list_height = addon_list_rect.height * UI_SCALE;

        let content_left = list_x + 12.0;
        let content_right = list_x + list_width - 40.0;
        let content_top = list_y + 65.0;
        let content_bottom = list_y + list_height - 32.0;
        let content_width = content_right - content_left;

        let entry_height = 20.0;
        let checkbox_size = 14.0;
        let checkbox_margin = 4.0;

        let visible_height = content_bottom - content_top;
        let first_visible = (self.scroll_offset / entry_height).floor() as usize;
        let visible_count = ((visible_height / entry_height).ceil() as usize) + 1;

        for (i, addon) in addons.iter().enumerate().skip(first_visible).take(visible_count) {
            let relative_y = (i as f32 * entry_height) - self.scroll_offset;
            let entry_y = content_top + relative_y;

            if entry_y + entry_height < content_top || entry_y > content_bottom {
                continue;
            }

            // Addon title text (positioned after checkbox)
            let text_x = content_left + checkbox_size + checkbox_margin;
            let text_width = content_width - checkbox_size - checkbox_margin;

            // Text color based on load status
            let text_color = if addon.loaded {
                Color::from_rgba(1.0, 0.82, 0.0, 1.0) // Gold for loaded
            } else if addon.enabled {
                Color::from_rgba(1.0, 0.3, 0.3, 1.0) // Red for failed to load
            } else {
                Color::from_rgba(0.5, 0.5, 0.5, 1.0) // Gray for disabled
            };

            TextRenderer::draw_justified_text(
                frame,
                &addon.title,
                Rectangle::new(
                    Point::new(text_x, entry_y),
                    Size::new(text_width, entry_height),
                ),
                12.0,
                text_color,
                Font::DEFAULT,
                crate::widget::TextJustify::Left,
                crate::widget::TextJustify::Center,
            );
        }
    }

    fn draw_frame_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        // Draw NineSlice border if this is a NineSlice frame
        if let Some(ref layout) = f.nine_slice_layout {
            self.draw_nine_slice_border(frame, bounds, layout, f.alpha);
        }

        if f.backdrop.enabled {
            // Draw backdrop background
            let bg = &f.backdrop.bg_color;
            frame.fill_rectangle(
                bounds.position(),
                bounds.size(),
                Color::from_rgba(bg.r, bg.g, bg.b, bg.a * f.alpha),
            );

            // Draw border
            let bc = &f.backdrop.border_color;
            frame.stroke(
                &Path::rectangle(bounds.position(), bounds.size()),
                Stroke::default()
                    .with_color(Color::from_rgba(bc.r, bc.g, bc.b, bc.a * f.alpha))
                    .with_width(f.backdrop.edge_size.max(1.0)),
            );
        }

    }

    /// Draw NineSlice panel border using atlas textures.
    fn draw_nine_slice_border(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        _layout: &str,
        alpha: f32,
    ) {
        // Use the 2x hi-res atlas textures for the metal frame
        // Corner sizes from atlas: 150x150 for 2x textures, scaled down to ~75 for rendering
        let corner_size = 32.0; // Scaled down corner size for rendering
        let edge_thickness = 32.0; // Edge thickness

        // Atlas names for ButtonFrameTemplateNoPortrait (also works for PortraitFrameTemplate)
        let tl_atlas = "ui-frame-metal-cornertopleft-2x";
        let tr_atlas = "ui-frame-metal-cornertopright-2x";
        let bl_atlas = "ui-frame-metal-cornerbottomleft-2x";
        let br_atlas = "ui-frame-metal-cornerbottomright-2x";
        let top_atlas = "_ui-frame-metal-edgetop-2x";
        let bottom_atlas = "_ui-frame-metal-edgebottom-2x";
        let left_atlas = "!ui-frame-metal-edgeleft-2x";
        let right_atlas = "!ui-frame-metal-edgeright-2x";

        // Draw corners
        if let Some(handle) = self.get_atlas_texture(tl_atlas) {
            let corner_bounds = Rectangle::new(
                Point::new(bounds.x - 8.0, bounds.y - 8.0),
                Size::new(corner_size, corner_size),
            );
            frame.draw_image(corner_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        if let Some(handle) = self.get_atlas_texture(tr_atlas) {
            let corner_bounds = Rectangle::new(
                Point::new(bounds.x + bounds.width - corner_size + 8.0, bounds.y - 8.0),
                Size::new(corner_size, corner_size),
            );
            frame.draw_image(corner_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        if let Some(handle) = self.get_atlas_texture(bl_atlas) {
            let corner_bounds = Rectangle::new(
                Point::new(bounds.x - 8.0, bounds.y + bounds.height - corner_size + 8.0),
                Size::new(corner_size, corner_size),
            );
            frame.draw_image(corner_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        if let Some(handle) = self.get_atlas_texture(br_atlas) {
            let corner_bounds = Rectangle::new(
                Point::new(bounds.x + bounds.width - corner_size + 8.0, bounds.y + bounds.height - corner_size + 8.0),
                Size::new(corner_size, corner_size),
            );
            frame.draw_image(corner_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        // Draw edges (stretched between corners)
        // Top edge
        if let Some(handle) = self.get_atlas_texture(top_atlas) {
            let edge_bounds = Rectangle::new(
                Point::new(bounds.x + corner_size - 8.0, bounds.y - 8.0),
                Size::new(bounds.width - corner_size * 2.0 + 16.0, edge_thickness),
            );
            frame.draw_image(edge_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        // Bottom edge
        if let Some(handle) = self.get_atlas_texture(bottom_atlas) {
            let edge_bounds = Rectangle::new(
                Point::new(bounds.x + corner_size - 8.0, bounds.y + bounds.height - edge_thickness + 8.0),
                Size::new(bounds.width - corner_size * 2.0 + 16.0, edge_thickness),
            );
            frame.draw_image(edge_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        // Left edge
        if let Some(handle) = self.get_atlas_texture(left_atlas) {
            let edge_bounds = Rectangle::new(
                Point::new(bounds.x - 8.0, bounds.y + corner_size - 8.0),
                Size::new(edge_thickness, bounds.height - corner_size * 2.0 + 16.0),
            );
            frame.draw_image(edge_bounds, canvas::Image::new(handle).opacity(alpha));
        }

        // Right edge
        if let Some(handle) = self.get_atlas_texture(right_atlas) {
            let edge_bounds = Rectangle::new(
                Point::new(bounds.x + bounds.width - edge_thickness + 8.0, bounds.y + corner_size - 8.0),
                Size::new(edge_thickness, bounds.height - corner_size * 2.0 + 16.0),
            );
            frame.draw_image(edge_bounds, canvas::Image::new(handle).opacity(alpha));
        }
    }

    /// Load an atlas texture by name, extracting the region from the atlas file.
    fn get_atlas_texture(&self, atlas_name: &str) -> Option<ImageHandle> {
        use crate::atlas::ATLAS_DB;

        let info = ATLAS_DB.get(atlas_name)?;

        // Get the full texture first
        let _ = self.get_or_load_texture(info.file);
        if let Some((tex_w, tex_h)) = self.get_texture_size(info.file) {
            // Calculate pixel coordinates from normalized tex_coords
            let x = (info.left_tex_coord * tex_w).round() as u32;
            let y = (info.top_tex_coord * tex_h).round() as u32;
            let w = ((info.right_tex_coord - info.left_tex_coord) * tex_w).round() as u32;
            let h = ((info.bottom_tex_coord - info.top_tex_coord) * tex_h).round() as u32;

            self.get_or_load_texture_region(info.file, x, y, w, h)
        } else {
            None
        }
    }

    /// Load a texture and cache its ImageHandle.
    fn get_or_load_texture(&self, wow_path: &str) -> Option<ImageHandle> {
        // Check cache first
        {
            let cache = self.image_handles.borrow();
            if let Some(handle) = cache.get(wow_path) {
                return Some(handle.clone());
            }
        }

        // Load texture
        let mut tex_mgr = self.texture_manager.borrow_mut();
        if let Some(tex_data) = tex_mgr.load(wow_path) {
            let handle = ImageHandle::from_rgba(
                tex_data.width,
                tex_data.height,
                tex_data.pixels.clone(),
            );
            self.image_handles
                .borrow_mut()
                .insert(wow_path.to_string(), handle.clone());
            Some(handle)
        } else {
            None
        }
    }

    /// Load a sub-region of a texture (for atlas textures with tex_coords).
    fn get_or_load_texture_region(
        &self,
        wow_path: &str,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Option<ImageHandle> {
        // Create a cache key for this specific region
        let cache_key = format!("{}#{}_{}_{}_{}", wow_path, x, y, width, height);

        // Check cache first
        {
            let cache = self.image_handles.borrow();
            if let Some(handle) = cache.get(&cache_key) {
                return Some(handle.clone());
            }
        }

        // Load sub-region from texture manager
        let mut tex_mgr = self.texture_manager.borrow_mut();
        if let Some(tex_data) = tex_mgr.load_sub_region(wow_path, x, y, width, height) {
            let handle = ImageHandle::from_rgba(
                tex_data.width,
                tex_data.height,
                tex_data.pixels.clone(),
            );
            self.image_handles
                .borrow_mut()
                .insert(cache_key, handle.clone());
            Some(handle)
        } else {
            None
        }
    }

    /// Resolve texture path for a button state by checking both direct fields and child textures.
    fn resolve_button_texture(
        &self,
        button: &crate::widget::Frame,
        key: &str,
        registry: &crate::widget::WidgetRegistry,
    ) -> Option<String> {
        // First check child texture (may have atlas set via SetAtlas)
        if let Some(&tex_id) = button.children_keys.get(key) {
            if let Some(tex) = registry.get(tex_id) {
                if tex.texture.is_some() {
                    return tex.texture.clone();
                }
            }
        }

        // Fall back to button's direct fields
        match key {
            "NormalTexture" => button.normal_texture.clone(),
            "PushedTexture" => button.pushed_texture.clone(),
            "HighlightTexture" => button.highlight_texture.clone(),
            "DisabledTexture" => button.disabled_texture.clone(),
            _ => None,
        }
    }

    /// Resolve tex_coords for a button texture from child texture (set via atlas).
    fn resolve_button_tex_coords(
        &self,
        button: &crate::widget::Frame,
        key: &str,
        registry: &crate::widget::WidgetRegistry,
    ) -> Option<(f32, f32, f32, f32)> {
        if let Some(&tex_id) = button.children_keys.get(key) {
            if let Some(tex) = registry.get(tex_id) {
                return tex.tex_coords;
            }
        }
        None
    }

    fn draw_button_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
        frame_id: u64,
        normal_tex: Option<&str>,
        normal_coords: Option<(f32, f32, f32, f32)>,
        pushed_tex: Option<&str>,
        pushed_coords: Option<(f32, f32, f32, f32)>,
        highlight_tex: Option<&str>,
        highlight_coords: Option<(f32, f32, f32, f32)>,
    ) {
        let is_pressed = self.pressed_frame == Some(frame_id);
        let is_hovered = self.hovered_frame == Some(frame_id);

        // Try 3-slice button rendering first (gold button style)
        if let (Some(left_path), Some(middle_path), Some(right_path)) =
            (&f.left_texture, &f.middle_texture, &f.right_texture)
        {
            let left_handle = self.get_or_load_texture(left_path);
            let middle_handle = self.get_or_load_texture(middle_path);
            let right_handle = self.get_or_load_texture(right_path);

            if left_handle.is_some() && middle_handle.is_some() && right_handle.is_some() {
                // Gold button: left=64px, right=32px
                draw_horizontal_slice_texture(
                    frame,
                    bounds,
                    left_handle.as_ref().unwrap(), // fallback (unused when all 3 present)
                    left_handle.as_ref(),
                    middle_handle.as_ref(),
                    right_handle.as_ref(),
                    64.0, // left cap width
                    32.0, // right cap width
                    f.alpha,
                );

                // Draw button text on top
                if let Some(ref txt) = f.text {
                    TextRenderer::draw_centered_text(
                        frame,
                        txt,
                        bounds,
                        f.font_size,
                        Color::from_rgba(
                            f.text_color.r,
                            f.text_color.g,
                            f.text_color.b,
                            f.text_color.a * f.alpha,
                        ),
                        wow_font_to_iced(f.font.as_deref()),
                    );
                }
                return;
            }
        }

        // Select texture and tex_coords based on button state: pressed > normal
        let (button_texture, button_coords) = if is_pressed {
            (pushed_tex.or(normal_tex), pushed_coords.or(normal_coords))
        } else {
            (normal_tex, normal_coords)
        };

        // Try single texture (pushed or normal)
        let mut drew_background = false;
        if let Some(tex_path) = button_texture {
            // Load texture region based on tex_coords (for atlas textures) or special cases
            let handle = if let Some((left, right, top, bottom)) = button_coords {
                // Atlas texture - extract sub-region using tex_coords
                // First load the texture to get its size
                let _ = self.get_or_load_texture(tex_path);
                if let Some((tex_w, tex_h)) = self.get_texture_size(tex_path) {
                    let x = (left * tex_w).round() as u32;
                    let y = (top * tex_h).round() as u32;
                    let w = ((right - left) * tex_w).round() as u32;
                    let h = ((bottom - top) * tex_h).round() as u32;
                    self.get_or_load_texture_region(tex_path, x, y, w, h)
                } else {
                    self.get_or_load_texture(tex_path)
                }
            } else if tex_path.to_lowercase().contains("ui-panel-button") {
                // Panel button textures use TexCoords: 0.625 width, 0.6875 height
                // The actual content is 80x22 within a 128x32 texture
                self.get_or_load_texture_region(tex_path, 0, 0, 80, 22)
            } else {
                self.get_or_load_texture(tex_path)
            };

            if let Some(handle) = handle {
                // Draw the texture scaled to button bounds
                frame.draw_image(bounds, canvas::Image::new(handle));
                drew_background = true;
            }
        }

        // Fallback: default button styling (dark red gradient-like)
        if !drew_background {
            // Use different color when pressed for visual feedback
            let bg_color = if is_pressed {
                Color::from_rgba(0.20, 0.08, 0.08, 0.95 * f.alpha)
            } else if is_hovered {
                Color::from_rgba(0.18, 0.07, 0.07, 0.95 * f.alpha)
            } else {
                Color::from_rgba(0.15, 0.05, 0.05, 0.95 * f.alpha)
            };

            frame.fill_rectangle(bounds.position(), bounds.size(), bg_color);

            // Border - brighter when hovered
            let border_color = if is_hovered || is_pressed {
                Color::from_rgba(0.8, 0.6, 0.2, f.alpha)
            } else {
                Color::from_rgba(0.6, 0.45, 0.15, f.alpha)
            };

            frame.stroke(
                &Path::rectangle(bounds.position(), bounds.size()),
                Stroke::default().with_color(border_color).with_width(1.5),
            );
        }

        // Draw highlight texture overlay when hovered (and not pressed)
        // WoW uses additive blending for highlight - we approximate with semi-transparent overlay
        if is_hovered && !is_pressed {
            if let Some(highlight_path) = highlight_tex {
                // Load texture region based on tex_coords (for atlas textures) or special cases
                let handle = if let Some((left, right, top, bottom)) = highlight_coords {
                    // Atlas texture - extract sub-region using tex_coords
                    let _ = self.get_or_load_texture(highlight_path);
                    if let Some((tex_w, tex_h)) = self.get_texture_size(highlight_path) {
                        let x = (left * tex_w).round() as u32;
                        let y = (top * tex_h).round() as u32;
                        let w = ((right - left) * tex_w).round() as u32;
                        let h = ((bottom - top) * tex_h).round() as u32;
                        self.get_or_load_texture_region(highlight_path, x, y, w, h)
                    } else {
                        self.get_or_load_texture(highlight_path)
                    }
                } else if highlight_path.to_lowercase().contains("ui-panel-button") {
                    self.get_or_load_texture_region(highlight_path, 0, 0, 80, 22)
                } else {
                    self.get_or_load_texture(highlight_path)
                };

                if let Some(handle) = handle {
                    // Draw highlight with reduced opacity to simulate additive effect
                    let mut img = canvas::Image::new(handle);
                    img = img.opacity(0.5 * f.alpha);
                    frame.draw_image(bounds, img);
                }
            } else if drew_background {
                // No highlight texture, draw a subtle highlight overlay
                frame.fill_rectangle(
                    bounds.position(),
                    bounds.size(),
                    Color::from_rgba(1.0, 0.9, 0.6, 0.15 * f.alpha),
                );
            }
        }

        // Draw button text (centered)
        if let Some(ref txt) = f.text {
            TextRenderer::draw_centered_text(
                frame,
                txt,
                bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                wow_font_to_iced(f.font.as_deref()),
            );
        }
    }

    fn draw_texture_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        // Try color texture first
        if let Some(color) = f.color_texture {
            frame.fill_rectangle(
                bounds.position(),
                bounds.size(),
                Color::from_rgba(color.r, color.g, color.b, color.a * f.alpha),
            );
            return;
        }

        // Try to load and render the texture file
        if let Some(ref tex_path) = f.texture {
            // If we have tex_coords, we need to extract a sub-region
            let handle_opt = if let Some((left, right, top, bottom)) = f.tex_coords {
                // Ensure full texture is loaded first to get its size
                // (load it but don't use the handle - we want the sub-region instead)
                let _ = self.get_or_load_texture(tex_path);

                // Get full texture size to calculate pixel coords
                if let Some(tex_size) = self.get_texture_size(tex_path) {
                    let x = (left * tex_size.0).round() as u32;
                    let y = (top * tex_size.1).round() as u32;
                    let w = ((right - left) * tex_size.0).round() as u32;
                    let h = ((bottom - top) * tex_size.1).round() as u32;
                    self.get_or_load_texture_region(tex_path, x, y, w, h)
                } else {
                    // Fallback: use full texture if size not available
                    self.get_or_load_texture(tex_path)
                }
            } else {
                self.get_or_load_texture(tex_path)
            };

            if let Some(handle) = handle_opt {
                // Check if tiling is enabled
                if f.horiz_tile || f.vert_tile {
                    // Get texture dimensions from cache (use sub-region size if available)
                    let tex_size = if f.tex_coords.is_some() {
                        // For sub-regions, the tile size should be the region size
                        (bounds.width, bounds.height)
                    } else {
                        self.get_texture_size(tex_path).unwrap_or((256.0, 256.0))
                    };

                    // Draw tiled texture
                    self.draw_tiled_texture(
                        frame,
                        bounds,
                        &handle,
                        tex_size.0,
                        tex_size.1,
                        f.horiz_tile,
                        f.vert_tile,
                        f.alpha,
                    );
                } else {
                    // Draw stretched to bounds
                    frame.draw_image(bounds, canvas::Image::new(handle));
                }
                return;
            }
        }

        // No texture - don't draw anything (transparent)
    }

    /// Draw a texture tiled across the given bounds.
    fn draw_tiled_texture(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        handle: &ImageHandle,
        tex_width: f32,
        tex_height: f32,
        horiz_tile: bool,
        vert_tile: bool,
        alpha: f32,
    ) {
        let tile_width = if horiz_tile { tex_width } else { bounds.width };
        let tile_height = if vert_tile { tex_height } else { bounds.height };

        // Calculate how many tiles we need
        let cols = if horiz_tile {
            ((bounds.width / tile_width).ceil() as i32).max(1)
        } else {
            1
        };
        let rows = if vert_tile {
            ((bounds.height / tile_height).ceil() as i32).max(1)
        } else {
            1
        };

        // Clip all tile drawing to the bounds
        frame.with_clip(bounds, |frame| {
            for row in 0..rows {
                for col in 0..cols {
                    let x = bounds.x + col as f32 * tile_width;
                    let y = bounds.y + row as f32 * tile_height;

                    let full_tile =
                        Rectangle::new(Point::new(x, y), Size::new(tile_width, tile_height));
                    let mut img = canvas::Image::new(handle.clone());
                    if alpha < 1.0 {
                        img = img.opacity(alpha);
                    }
                    frame.draw_image(full_tile, img);
                }
            }
        });
    }

    /// Get the dimensions of a cached texture.
    fn get_texture_size(&self, path: &str) -> Option<(f32, f32)> {
        self.texture_manager
            .borrow()
            .get_texture_size(path)
            .map(|(w, h)| (w as f32, h as f32))
    }

    fn draw_fontstring_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        if let Some(ref txt) = f.text {
            let clean_text = strip_wow_markup(txt);

            TextRenderer::draw_justified_text(
                frame,
                &clean_text,
                bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                wow_font_to_iced(f.font.as_deref()),
                f.justify_h,
                f.justify_v,
            );
        }
    }

    fn draw_editbox_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        // Background
        frame.fill_rectangle(
            bounds.position(),
            bounds.size(),
            Color::from_rgba(0.08, 0.08, 0.1, 0.9 * f.alpha),
        );

        // Border
        frame.stroke(
            &Path::rectangle(bounds.position(), bounds.size()),
            Stroke::default()
                .with_color(Color::from_rgba(0.3, 0.3, 0.35, f.alpha))
                .with_width(1.0),
        );

        // Text
        if let Some(ref txt) = f.text {
            let padding = 4.0;
            let text_bounds = Rectangle::new(
                Point::new(bounds.x + padding, bounds.y),
                Size::new(bounds.width - padding * 2.0, bounds.height),
            );
            TextRenderer::draw_justified_text(
                frame,
                txt,
                text_bounds,
                f.font_size,
                Color::from_rgba(
                    f.text_color.r,
                    f.text_color.g,
                    f.text_color.b,
                    f.text_color.a * f.alpha,
                ),
                Font::DEFAULT,
                TextJustify::Left,
                TextJustify::Center,
            );
        }
    }

    fn draw_checkbutton_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
        checked: bool,
    ) {
        // WoW-style checkbox: ~16px square, positioned at left of bounds
        let box_size = 16.0_f32.min(bounds.height).min(bounds.width);
        let box_x = bounds.x + (bounds.width - box_size) / 2.0; // Centered in bounds
        let box_y = bounds.y + (bounds.height - box_size) / 2.0;

        // Checkbox background - dark brown
        frame.fill_rectangle(
            Point::new(box_x, box_y),
            Size::new(box_size, box_size),
            Color::from_rgba(0.15, 0.1, 0.05, 0.8 * f.alpha),
        );

        // Checkbox border - gold
        frame.stroke(
            &Path::rectangle(Point::new(box_x, box_y), Size::new(box_size, box_size)),
            Stroke::default()
                .with_color(Color::from_rgba(0.6, 0.45, 0.15, f.alpha))
                .with_width(1.5),
        );

        // Checkmark if checked - bright gold
        if checked {
            let margin = box_size * 0.2;
            let check_x1 = box_x + margin;
            let check_y1 = box_y + box_size * 0.55;
            let check_x2 = box_x + box_size * 0.4;
            let check_y2 = box_y + box_size - margin;
            let check_x3 = box_x + box_size - margin;
            let check_y3 = box_y + margin;

            let mut builder = canvas::path::Builder::new();
            builder.move_to(Point::new(check_x1, check_y1));
            builder.line_to(Point::new(check_x2, check_y2));
            builder.line_to(Point::new(check_x3, check_y3));
            let path = builder.build();

            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(Color::from_rgba(1.0, 0.85, 0.0, f.alpha)) // Bright gold
                    .with_width(2.0),
            );
        }
    }

    fn draw_scrollframe_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        if f.backdrop.enabled {
            let bg = &f.backdrop.bg_color;
            frame.fill_rectangle(
                bounds.position(),
                bounds.size(),
                Color::from_rgba(bg.r, bg.g, bg.b, bg.a * f.alpha),
            );

            let bc = &f.backdrop.border_color;
            frame.stroke(
                &Path::rectangle(bounds.position(), bounds.size()),
                Stroke::default()
                    .with_color(Color::from_rgba(bc.r, bc.g, bc.b, bc.a * f.alpha))
                    .with_width(f.backdrop.edge_size.max(1.0)),
            );
        }
    }

    fn draw_slider_widget(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        let track_height = 4.0;
        let track_y = bounds.y + (bounds.height - track_height) / 2.0;

        frame.fill_rectangle(
            Point::new(bounds.x, track_y),
            Size::new(bounds.width, track_height),
            Color::from_rgba(0.2, 0.2, 0.25, 0.9),
        );

        let thumb_width = 12.0;
        let thumb_height = 16.0;
        let thumb_x = bounds.x + (bounds.width - thumb_width) / 2.0;
        let thumb_y = bounds.y + (bounds.height - thumb_height) / 2.0;

        frame.fill_rectangle(
            Point::new(thumb_x, thumb_y),
            Size::new(thumb_width, thumb_height),
            Color::from_rgb(0.6, 0.5, 0.3),
        );

        frame.stroke(
            &Path::rectangle(
                Point::new(thumb_x, thumb_y),
                Size::new(thumb_width, thumb_height),
            ),
            Stroke::default()
                .with_color(Color::from_rgb(0.8, 0.7, 0.4))
                .with_width(1.0),
        );
    }

    /// Draw scroll bar for AddonList panel
    fn draw_addon_list_scrollbar(
        &self,
        frame: &mut canvas::Frame,
        addon_list_rect: &crate::LayoutRect,
        _screen_width: f32,
    ) {
        // Scroll bar positioning relative to AddonList
        // Based on WoW's MinimalScrollBar template
        let scrollbar_width = 22.0;
        let button_size = 16.0;
        let button_margin = 15.0; // Buttons are offset 15px from edges

        // Position scroll bar to the right of the content area
        // AddonList content area is roughly x+10 to x+width-34
        let list_x = addon_list_rect.x * UI_SCALE;
        let list_y = addon_list_rect.y * UI_SCALE;
        let list_width = addon_list_rect.width * UI_SCALE;
        let list_height = addon_list_rect.height * UI_SCALE;

        // Scroll bar is on the right side, inside the frame
        let scrollbar_x = list_x + list_width - scrollbar_width - 8.0;
        // Content area starts below title bar (~60px) and ends above buttons (~28px)
        let content_top = list_y + 65.0;
        let content_bottom = list_y + list_height - 32.0;
        let scrollbar_height = content_bottom - content_top;

        // Scroll parameters
        let max_scroll = 2600.0_f32;
        let scroll_ratio = (self.scroll_offset / max_scroll).clamp(0.0, 1.0);

        // Draw track background (dark)
        let track_x = scrollbar_x;
        let track_y = content_top + button_margin + button_size;
        let track_height = scrollbar_height - 2.0 * (button_margin + button_size);

        frame.fill_rectangle(
            Point::new(track_x, track_y),
            Size::new(scrollbar_width, track_height),
            Color::from_rgba(0.05, 0.05, 0.05, 0.9),
        );

        // Draw track border
        frame.stroke(
            &Path::rectangle(Point::new(track_x, track_y), Size::new(scrollbar_width, track_height)),
            Stroke::default()
                .with_color(Color::from_rgba(0.3, 0.25, 0.15, 0.8))
                .with_width(1.0),
        );

        // Draw up button
        let up_btn_x = scrollbar_x + (scrollbar_width - button_size) / 2.0;
        let up_btn_y = content_top + button_margin;
        self.draw_scroll_button(frame, up_btn_x, up_btn_y, button_size, true);

        // Draw down button
        let down_btn_y = content_bottom - button_margin - button_size;
        self.draw_scroll_button(frame, up_btn_x, down_btn_y, button_size, false);

        // Draw thumb
        let thumb_height = 40.0_f32.max(track_height * 0.1); // Min 40px or 10% of track
        let thumb_travel = track_height - thumb_height;
        let thumb_y = track_y + scroll_ratio * thumb_travel;
        let thumb_x = track_x + 2.0;
        let thumb_width = scrollbar_width - 4.0;

        // Thumb background
        frame.fill_rectangle(
            Point::new(thumb_x, thumb_y),
            Size::new(thumb_width, thumb_height),
            Color::from_rgba(0.4, 0.35, 0.25, 0.95),
        );

        // Thumb border (gold)
        frame.stroke(
            &Path::rectangle(Point::new(thumb_x, thumb_y), Size::new(thumb_width, thumb_height)),
            Stroke::default()
                .with_color(Color::from_rgba(0.7, 0.55, 0.25, 1.0))
                .with_width(1.0),
        );

        // Thumb grip lines (horizontal lines in middle)
        let grip_color = Color::from_rgba(0.6, 0.5, 0.3, 0.8);
        let grip_y_center = thumb_y + thumb_height / 2.0;
        for i in -1..=1 {
            let grip_y = grip_y_center + (i as f32) * 4.0;
            frame.stroke(
                &Path::line(
                    Point::new(thumb_x + 4.0, grip_y),
                    Point::new(thumb_x + thumb_width - 4.0, grip_y),
                ),
                Stroke::default().with_color(grip_color).with_width(1.0),
            );
        }
    }

    /// Draw a scroll button (up or down arrow)
    fn draw_scroll_button(
        &self,
        frame: &mut canvas::Frame,
        x: f32,
        y: f32,
        size: f32,
        is_up: bool,
    ) {
        // Button background
        frame.fill_rectangle(
            Point::new(x, y),
            Size::new(size, size),
            Color::from_rgba(0.25, 0.22, 0.18, 0.95),
        );

        // Button border
        frame.stroke(
            &Path::rectangle(Point::new(x, y), Size::new(size, size)),
            Stroke::default()
                .with_color(Color::from_rgba(0.5, 0.4, 0.25, 1.0))
                .with_width(1.0),
        );

        // Draw arrow
        let arrow_color = Color::from_rgba(0.8, 0.7, 0.5, 1.0);
        let cx = x + size / 2.0;
        let cy = y + size / 2.0;
        let arrow_size = size * 0.3;

        let (tip_y, base_y) = if is_up {
            (cy - arrow_size, cy + arrow_size * 0.5)
        } else {
            (cy + arrow_size, cy - arrow_size * 0.5)
        };

        // Draw triangle arrow
        let arrow = Path::new(|builder| {
            builder.move_to(Point::new(cx, tip_y));
            builder.line_to(Point::new(cx - arrow_size, base_y));
            builder.line_to(Point::new(cx + arrow_size, base_y));
            builder.close();
        });

        frame.fill(&arrow, arrow_color);
    }

    /// Draw addon list entries directly (fallback when ScrollBox/DataProvider doesn't render)
    fn draw_addon_list_entries(
        &self,
        frame: &mut canvas::Frame,
        addon_list_rect: &crate::LayoutRect,
        state: &crate::lua_api::SimState,
    ) {
        let addons = &state.addons;
        if addons.is_empty() {
            return;
        }

        // Content area bounds
        let list_x = addon_list_rect.x * UI_SCALE;
        let list_y = addon_list_rect.y * UI_SCALE;
        let list_width = addon_list_rect.width * UI_SCALE;
        let list_height = addon_list_rect.height * UI_SCALE;

        // Content area starts below title bar and ends above bottom buttons
        let content_left = list_x + 12.0;
        let content_right = list_x + list_width - 40.0; // Leave room for scroll bar
        let content_top = list_y + 65.0;
        let content_bottom = list_y + list_height - 32.0;
        let content_width = content_right - content_left;

        let entry_height = 20.0;
        let checkbox_size = 14.0;
        let checkbox_margin = 4.0;

        // Calculate visible entries based on scroll offset
        let visible_height = content_bottom - content_top;

        // Skip entries scrolled above visible area
        let first_visible = (self.scroll_offset / entry_height).floor() as usize;
        let visible_count = ((visible_height / entry_height).ceil() as usize) + 1;

        for (i, addon) in addons.iter().enumerate().skip(first_visible).take(visible_count) {
            let relative_y = (i as f32 * entry_height) - self.scroll_offset;
            let entry_y = content_top + relative_y;

            // Skip if outside content area
            if entry_y + entry_height < content_top || entry_y > content_bottom {
                continue;
            }

            // Draw checkbox
            let cb_x = content_left;
            let cb_y = entry_y + (entry_height - checkbox_size) / 2.0;

            // Checkbox background
            frame.fill_rectangle(
                Point::new(cb_x, cb_y),
                Size::new(checkbox_size, checkbox_size),
                Color::from_rgba(0.1, 0.1, 0.12, 0.9),
            );

            // Checkbox border
            frame.stroke(
                &Path::rectangle(Point::new(cb_x, cb_y), Size::new(checkbox_size, checkbox_size)),
                Stroke::default()
                    .with_color(Color::from_rgba(0.5, 0.4, 0.25, 1.0))
                    .with_width(1.0),
            );

            // Draw checkmark if enabled
            if addon.enabled {
                let check_color = Color::from_rgba(0.4, 0.8, 0.3, 1.0);
                let margin = 3.0;
                // Draw checkmark as two lines
                frame.stroke(
                    &Path::new(|builder| {
                        builder.move_to(Point::new(cb_x + margin, cb_y + checkbox_size / 2.0));
                        builder.line_to(Point::new(cb_x + checkbox_size / 2.5, cb_y + checkbox_size - margin));
                        builder.line_to(Point::new(cb_x + checkbox_size - margin, cb_y + margin));
                    }),
                    Stroke::default().with_color(check_color).with_width(2.0),
                );
            }

            // Draw addon title
            let text_x = cb_x + checkbox_size + checkbox_margin;
            let text_y = entry_y;
            let text_width = content_width - checkbox_size - checkbox_margin;

            // Text color based on load status
            let text_color = if addon.loaded {
                Color::from_rgba(1.0, 0.82, 0.0, 1.0) // Gold for loaded
            } else if addon.enabled {
                Color::from_rgba(1.0, 0.3, 0.3, 1.0) // Red for failed to load
            } else {
                Color::from_rgba(0.5, 0.5, 0.5, 1.0) // Gray for disabled
            };

            TextRenderer::draw_justified_text(
                frame,
                &addon.title,
                Rectangle::new(
                    Point::new(text_x, text_y),
                    Size::new(text_width, entry_height),
                ),
                12.0,
                text_color,
                Font::DEFAULT,
                crate::widget::TextJustify::Left,
                crate::widget::TextJustify::Center,
            );
        }
    }

    /// Handle click on addon list checkbox, returns true if a checkbox was clicked
    fn handle_addon_checkbox_click(&self, pos: Point) -> bool {
        let env = self.env.borrow();
        let state = env.state().borrow();

        // Find AddonList frame
        let addonlist_rect = state.widgets.all_ids().into_iter()
            .find(|&id| {
                state.widgets.get(id)
                    .map(|f| f.name.as_deref() == Some("AddonList"))
                    .unwrap_or(false)
            })
            .and_then(|id| {
                let screen_width = 1920.0; // TODO: get actual screen size
                let screen_height = 1080.0;
                Some(compute_frame_rect(&state.widgets, id, screen_width, screen_height))
            });

        let rect = match addonlist_rect {
            Some(r) => r,
            None => return false,
        };

        // Content area bounds (must match draw_addon_list_entries)
        let list_x = rect.x * UI_SCALE;
        let list_y = rect.y * UI_SCALE;
        let list_height = rect.height * UI_SCALE;

        let content_left = list_x + 12.0;
        let content_top = list_y + 65.0;
        let content_bottom = list_y + list_height - 32.0;

        let entry_height = 20.0;
        let checkbox_size = 14.0;

        // Check if click is in the checkbox column area
        let checkbox_right = content_left + checkbox_size + 10.0; // Some extra margin for easier clicking

        if pos.x < content_left || pos.x > checkbox_right {
            return false;
        }
        if pos.y < content_top || pos.y > content_bottom {
            return false;
        }

        // Calculate which addon was clicked
        let relative_y = pos.y - content_top + self.scroll_offset;
        let addon_index = (relative_y / entry_height).floor() as usize;

        // Need to drop state borrow before mutating
        drop(state);
        drop(env);

        // Toggle addon enabled state
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();

        if addon_index < state.addons.len() {
            state.addons[addon_index].enabled = !state.addons[addon_index].enabled;
            return true;
        }

        false
    }
}

// Button styles
fn event_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Active => (palette::BG_PANEL, palette::TEXT_SECONDARY),
        button::Status::Hovered => (palette::BORDER_HIGHLIGHT, palette::GOLD),
        button::Status::Pressed => (palette::GOLD_DIM, palette::TEXT_PRIMARY),
        button::Status::Disabled => (palette::BG_DARK, palette::TEXT_MUTED),
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color,
        border: Border {
            color: palette::BORDER,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

fn run_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color, border_color) = match status {
        button::Status::Active => (palette::GOLD_DIM, palette::TEXT_PRIMARY, palette::GOLD),
        button::Status::Hovered => (palette::GOLD, Color::BLACK, palette::GOLD),
        button::Status::Pressed => (palette::GOLD_DIM, Color::BLACK, palette::GOLD_DIM),
        button::Status::Disabled => (palette::BG_DARK, palette::TEXT_MUTED, palette::BORDER),
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Active => palette::BORDER,
        text_input::Status::Hovered => palette::BORDER_HIGHLIGHT,
        text_input::Status::Focused { is_hovered: _ } => palette::GOLD_DIM,
        text_input::Status::Disabled => palette::BG_DARK,
    };

    text_input::Style {
        background: iced::Background::Color(palette::BG_INPUT),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
        icon: palette::TEXT_MUTED,
        placeholder: palette::TEXT_MUTED,
        value: palette::TEXT_PRIMARY,
        selection: palette::GOLD_DIM,
    }
}

/// Compute frame rect with anchor resolution.
fn compute_frame_rect(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    screen_width: f32,
    screen_height: f32,
) -> LayoutRect {
    let frame = match registry.get(id) {
        Some(f) => f,
        None => return LayoutRect::default(),
    };

    // Special case: UIParent (id=1) fills the entire screen
    if frame.name.as_deref() == Some("UIParent") || (frame.parent_id.is_none() && id == 1) {
        return LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        };
    }

    let parent_rect = if let Some(parent_id) = frame.parent_id {
        compute_frame_rect(registry, parent_id, screen_width, screen_height)
    } else {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        }
    };

    if frame.anchors.is_empty() {
        return LayoutRect {
            x: parent_rect.x + (parent_rect.width - frame.width) / 2.0,
            y: parent_rect.y + (parent_rect.height - frame.height) / 2.0,
            width: frame.width,
            height: frame.height,
        };
    }

    if frame.anchors.len() >= 2 {
        let mut left_x: Option<f32> = None;
        let mut right_x: Option<f32> = None;
        let mut top_y: Option<f32> = None;
        let mut bottom_y: Option<f32> = None;

        for anchor in &frame.anchors {
            let relative_rect = if let Some(rel_id) = anchor.relative_to_id {
                compute_frame_rect(registry, rel_id as u64, screen_width, screen_height)
            } else {
                parent_rect
            };

            let (anchor_x, anchor_y) = anchor_position(
                anchor.relative_point,
                relative_rect.x,
                relative_rect.y,
                relative_rect.width,
                relative_rect.height,
            );
            let target_x = anchor_x + anchor.x_offset;
            let target_y = anchor_y - anchor.y_offset;

            match anchor.point {
                AnchorPoint::TopLeft => {
                    left_x = Some(target_x);
                    top_y = Some(target_y);
                }
                AnchorPoint::TopRight => {
                    right_x = Some(target_x);
                    top_y = Some(target_y);
                }
                AnchorPoint::BottomLeft => {
                    left_x = Some(target_x);
                    bottom_y = Some(target_y);
                }
                AnchorPoint::BottomRight => {
                    right_x = Some(target_x);
                    bottom_y = Some(target_y);
                }
                AnchorPoint::Top => {
                    top_y = Some(target_y);
                }
                AnchorPoint::Bottom => {
                    bottom_y = Some(target_y);
                }
                AnchorPoint::Left => {
                    left_x = Some(target_x);
                }
                AnchorPoint::Right => {
                    right_x = Some(target_x);
                }
                AnchorPoint::Center => {}
            }
        }

        let computed_width = if frame.width == 0.0 {
            if let (Some(lx), Some(rx)) = (left_x, right_x) {
                Some((rx - lx).max(0.0))
            } else {
                None
            }
        } else {
            None
        };

        let computed_height = if frame.height == 0.0 {
            if let (Some(ty), Some(by)) = (top_y, bottom_y) {
                Some((by - ty).max(0.0))
            } else {
                None
            }
        } else {
            None
        };

        let final_width = if frame.width > 0.0 {
            frame.width
        } else {
            computed_width.unwrap_or(0.0)
        };
        let final_height = if frame.height > 0.0 {
            frame.height
        } else {
            computed_height.unwrap_or(0.0)
        };

        let final_x =
            left_x.unwrap_or_else(|| right_x.map(|rx| rx - final_width).unwrap_or(parent_rect.x));
        let final_y =
            top_y.unwrap_or_else(|| bottom_y.map(|by| by - final_height).unwrap_or(parent_rect.y));

        return LayoutRect {
            x: final_x,
            y: final_y,
            width: final_width,
            height: final_height,
        };
    }

    let anchor = &frame.anchors[0];
    let width = frame.width;
    let height = frame.height;

    // For single anchor, check if it has a specific relativeTo frame
    let relative_rect = if let Some(rel_id) = anchor.relative_to_id {
        compute_frame_rect(registry, rel_id as u64, screen_width, screen_height)
    } else {
        parent_rect
    };

    let (anchor_x, anchor_y) = anchor_position(
        anchor.relative_point,
        relative_rect.x,
        relative_rect.y,
        relative_rect.width,
        relative_rect.height,
    );

    let target_x = anchor_x + anchor.x_offset;
    let target_y = anchor_y - anchor.y_offset;

    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect {
        x: frame_x,
        y: frame_y,
        width,
        height,
    }
}

fn anchor_position(point: AnchorPoint, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
    match point {
        AnchorPoint::TopLeft => (x, y),
        AnchorPoint::Top => (x + w / 2.0, y),
        AnchorPoint::TopRight => (x + w, y),
        AnchorPoint::Left => (x, y + h / 2.0),
        AnchorPoint::Center => (x + w / 2.0, y + h / 2.0),
        AnchorPoint::Right => (x + w, y + h / 2.0),
        AnchorPoint::BottomLeft => (x, y + h),
        AnchorPoint::Bottom => (x + w / 2.0, y + h),
        AnchorPoint::BottomRight => (x + w, y + h),
    }
}

fn frame_position_from_anchor(
    point: AnchorPoint,
    anchor_x: f32,
    anchor_y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    match point {
        AnchorPoint::TopLeft => (anchor_x, anchor_y),
        AnchorPoint::Top => (anchor_x - w / 2.0, anchor_y),
        AnchorPoint::TopRight => (anchor_x - w, anchor_y),
        AnchorPoint::Left => (anchor_x, anchor_y - h / 2.0),
        AnchorPoint::Center => (anchor_x - w / 2.0, anchor_y - h / 2.0),
        AnchorPoint::Right => (anchor_x - w, anchor_y - h / 2.0),
        AnchorPoint::BottomLeft => (anchor_x, anchor_y - h),
        AnchorPoint::Bottom => (anchor_x - w / 2.0, anchor_y - h),
        AnchorPoint::BottomRight => (anchor_x - w, anchor_y - h),
    }
}
