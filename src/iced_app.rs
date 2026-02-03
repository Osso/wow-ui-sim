//! Iced-based UI for rendering WoW frames.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Event, Geometry, Path, Stroke};
use iced::widget::image::Handle as ImageHandle;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::window::screenshot::Screenshot;
use iced::{window, Border, Color, Element, Font, Length, Point, Rectangle, Size, Theme};
use iced::{Subscription, Task};

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

    // Create test button with textures
    create_test_button(&env);
}

/// Create test buttons below AddonList to demonstrate texture rendering.
fn create_test_button(env: &WowLuaEnv) {
    let lua_code = r#"
        -- Create test button with WoW 3-slice gold button textures
        local btn = CreateFrame("Button", "TestTextureButton", UIParent)
        btn:SetSize(180, 32)
        btn:SetPoint("TOP", AddonList, "BOTTOM", 0, -20)
        btn:SetFrameStrata("HIGH")
        btn:SetFrameLevel(100)

        -- Set 3-slice button textures (gold button style)
        btn:SetLeftTexture("Interface\\Buttons\\UI-DialogBox-goldbutton-up-left")
        btn:SetMiddleTexture("Interface\\Buttons\\UI-DialogBox-goldbutton-up-middle")
        btn:SetRightTexture("Interface\\Buttons\\UI-DialogBox-goldbutton-up-right")

        -- Set button text directly
        btn:SetText("3-Slice Button")

        btn:Show()
        print("[Test] Created TestTextureButton with 3-slice gold button textures")

        -- Create second test button with standard normal/pushed/highlight textures
        -- Panel button textures are 80x22 visible content, so scale proportionally
        local btn2 = CreateFrame("Button", "TestHoverButton", UIParent)
        btn2:SetSize(120, 22)
        btn2:SetPoint("TOP", TestTextureButton, "BOTTOM", 0, -10)
        btn2:SetFrameStrata("HIGH")
        btn2:SetFrameLevel(100)

        -- Set standard button textures (panel button style)
        btn2:SetNormalTexture("Interface\\Buttons\\UI-Panel-Button-Up")
        btn2:SetPushedTexture("Interface\\Buttons\\UI-Panel-Button-Down")
        btn2:SetHighlightTexture("Interface\\Buttons\\UI-Panel-Button-Highlight")

        -- Button text
        btn2:SetText("Hover Me!")

        -- Enable mouse interaction
        btn2:EnableMouse(true)

        -- Add click handler for feedback
        btn2:SetScript("OnClick", function(self)
            print("[Test] TestHoverButton clicked!")
        end)
        btn2:SetScript("OnEnter", function(self)
            print("[Test] TestHoverButton: mouse entered")
        end)
        btn2:SetScript("OnLeave", function(self)
            print("[Test] TestHoverButton: mouse left")
        end)

        btn2:Show()
        print("[Test] Created TestHoverButton with normal/pushed/highlight textures")
    "#;

    if let Err(e) = env.lua().load(lua_code).exec() {
        eprintln!("[Test] Failed to create test button: {}", e);
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
}

/// Canvas-specific messages.
#[derive(Debug, Clone)]
pub enum CanvasMessage {
    MouseMove(Point),
    MouseDown(Point),
    MouseUp(Point),
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
    hovered_frame: Option<u64>,
    pressed_frame: Option<u64>,
    mouse_down_frame: Option<u64>,
    scroll_offset: f32,
    screen_size: Size,
    debug_rx: Option<mpsc::Receiver<DebugCommand>>,
    pending_screenshot: Option<oneshot::Sender<Result<ScreenshotData, String>>>,
    lua_rx: Option<std::sync::mpsc::Receiver<LuaCommand>>,
    /// Draw red debug borders around all frames when true.
    debug_borders: bool,
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

        let debug_borders = std::env::var("WOW_SIM_DEBUG_BORDERS").is_ok();

        let app = App {
            env: env_rc,
            log_messages,
            command_input: String::new(),
            texture_manager,
            image_handles: Rc::new(RefCell::new(HashMap::new())),
            frame_cache: Cache::new(),
            hovered_frame: None,
            pressed_frame: None,
            mouse_down_frame: None,
            scroll_offset: 0.0,
            screen_size: Size::new(600.0, 450.0),
            debug_rx: Some(cmd_rx),
            pending_screenshot: None,
            lua_rx: Some(lua_rx),
            debug_borders,
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
            }
            Message::CanvasEvent(canvas_msg) => match canvas_msg {
                CanvasMessage::MouseMove(pos) => {
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
                    }
                }
                CanvasMessage::MouseUp(pos) => {
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
                    }
                    self.mouse_down_frame = None;
                    self.pressed_frame = None;
                }
            },
            Message::Scroll(_dx, dy) => {
                let scroll_speed = 30.0;
                self.scroll_offset += dy * scroll_speed;
                let max_scroll = 2600.0;
                self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
                self.frame_cache.clear();
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
                }
            }
            Message::ProcessTimers => {
                // Process WoW timers
                let timer_result = {
                    let env = self.env.borrow();
                    env.process_timers()
                };
                match timer_result {
                    Ok(count) if count > 0 => {
                        self.drain_console();
                        self.frame_cache.clear();
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
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Title
        let title = text("WoW UI Simulator").size(20).color(palette::GOLD);

        // Canvas for WoW frames
        let canvas: Canvas<&App, Message> = Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill);

        let canvas_container = container(canvas)
            .width(Length::FillPortion(3))
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

        // Frames sidebar
        let frames_list = self.build_frames_sidebar();
        let sidebar = container(
            scrollable(frames_list)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(180)
        .height(Length::Fill)
        .padding(6)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(palette::BG_PANEL)),
            border: Border {
                color: palette::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

        // Main content row
        let content_row = row![canvas_container, sidebar].spacing(6);

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
            .take(10)
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
        .height(160)
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
        // Timer for processing WoW timers and debug commands (~30fps)
        iced::time::every(std::time::Duration::from_millis(33)).map(|_| Message::ProcessTimers)
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
        let screen_width = self.screen_size.width;
        let screen_height = self.screen_size.height;

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

    fn build_frames_sidebar(&self) -> Column<'_, Message> {
        let mut col = Column::new().spacing(2);

        col = col.push(text("Frames").size(14).color(palette::TEXT_PRIMARY));

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

                col = col.push(text(display).size(10).color(palette::TEXT_MUTED));

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
                    self.screen_size.width,
                    self.screen_size.height,
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
}

/// Canvas program implementation for rendering WoW frames.
impl canvas::Program<Message> for &App {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { position } => {
                    if bounds.contains(*position) {
                        let local = Point::new(position.x - bounds.x, position.y - bounds.y);
                        return Some(canvas::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseMove(local),
                        )));
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(canvas::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseDown(pos),
                        )));
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(canvas::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseUp(pos),
                        )));
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    let dy = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => *y,
                        mouse::ScrollDelta::Pixels { y, .. } => *y / 30.0,
                    };
                    return Some(canvas::Action::publish(Message::Scroll(0.0, dy)));
                }
                _ => {}
            },
            Event::Keyboard(keyboard_event) => {
                use iced::keyboard;
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = keyboard_event {
                    if modifiers.control() && *key == keyboard::Key::Character("r".into()) {
                        return Some(canvas::Action::publish(Message::ReloadUI));
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
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.frame_cache.draw(renderer, bounds.size(), |frame| {
            // Dark background
            frame.fill_rectangle(
                Point::ORIGIN,
                frame.size(),
                Color::from_rgb(0.05, 0.05, 0.08),
            );

            self.draw_wow_frames(frame, bounds.size());
        });

        vec![geometry]
    }
}

impl App {
    fn draw_wow_frames(&self, frame: &mut canvas::Frame, size: Size) {

        let env = self.env.borrow();
        let state = env.state().borrow();

        let screen_width = size.width;
        let screen_height = size.height;

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

        // Collect and sort frames
        let mut frames: Vec<_> = state
            .widgets
            .all_ids()
            .into_iter()
            .filter_map(|id| {
                let f = state.widgets.get(id)?;
                let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
                // Check for __checked attribute, default to true for addon list checkboxes
                let checked = if let Some(crate::widget::AttributeValue::Boolean(c)) =
                    f.attributes.get("__checked")
                {
                    *c
                } else {
                    // Default to checked for CheckButton widgets (most are addon enable checkboxes)
                    f.widget_type == WidgetType::CheckButton
                };
                Some((id, f, rect, checked))
            })
            .collect();

        frames.sort_by(|a, b| {
            a.1.frame_strata
                .cmp(&b.1.frame_strata)
                .then_with(|| a.1.frame_level.cmp(&b.1.frame_level))
                .then_with(|| {
                    // Regions (Texture, FontString) render within their frame
                    // sorted by draw_layer then draw_sub_layer.
                    // Non-regions (Frame, Button, etc.) render before their child regions.
                    let is_region = |t: &WidgetType| {
                        matches!(t, WidgetType::Texture | WidgetType::FontString)
                    };
                    match (is_region(&a.1.widget_type), is_region(&b.1.widget_type)) {
                        (true, true) => {
                            // Both regions: sort by draw_layer, then draw_sub_layer
                            a.1.draw_layer
                                .cmp(&b.1.draw_layer)
                                .then_with(|| a.1.draw_sub_layer.cmp(&b.1.draw_sub_layer))
                        }
                        (false, true) => std::cmp::Ordering::Less, // Frame before region
                        (true, false) => std::cmp::Ordering::Greater, // Region after frame
                        (false, false) => std::cmp::Ordering::Equal, // Both frames: use id
                    }
                })
                .then_with(|| a.0.cmp(&b.0))
        });

        // Find test buttons (TestTextureButton and TestHoverButton) and their children
        let mut test_button_ids = std::collections::HashSet::new();
        for test_name in ["TestTextureButton", "TestHoverButton"] {
            let test_button_id = state.widgets.all_ids().into_iter().find(|&id| {
                state
                    .widgets
                    .get(id)
                    .map(|f| f.name.as_deref() == Some(test_name))
                    .unwrap_or(false)
            });
            if let Some(root_id) = test_button_id {
                let mut queue = vec![root_id];
                while let Some(id) = queue.pop() {
                    test_button_ids.insert(id);
                    if let Some(f) = state.widgets.get(id) {
                        queue.extend(f.children.iter().copied());
                    }
                }
            }
        }

        // Capture AddonList rect before consuming frames
        let addonlist_rect = addonlist_id.and_then(|root_id| {
            frames.iter().find(|(id, _, _, _)| *id == root_id).map(|(_, _, r, _)| r.clone())
        });

        for (id, f, rect, checked) in frames {
            // Only show AddonList frame and children, plus test buttons and their children
            if !addonlist_ids.contains(&id) && !test_button_ids.contains(&id) {
                continue;
            }
            if !f.visible {
                continue;
            }
            if rect.width <= 0.0 || rect.height <= 0.0 {
                continue;
            }

            let x = rect.x * UI_SCALE;
            let mut y = rect.y * UI_SCALE;
            let w = rect.width * UI_SCALE;
            let h = rect.height * UI_SCALE;

            // Apply scroll offset for AddonList children
            let is_addonlist_root = addonlist_id == Some(id);
            if !is_addonlist_root && addonlist_ids.contains(&id) {
                y -= self.scroll_offset;
                // Cull elements that scroll outside the visible content area
                // AddonList content starts around y=70 (after title bar and header)
                // and ends around y=530 (before the bottom buttons)
                let content_top = 70.0;
                let content_bottom = 530.0;
                if y + h < content_top || y > content_bottom {
                    continue;
                }
            }

            let bounds = Rectangle::new(Point::new(x, y), Size::new(w, h));

            match f.widget_type {
                WidgetType::Frame => {
                    self.draw_frame_widget(frame, bounds, f);
                }
                WidgetType::Button => {
                    // Resolve textures from both button fields and child textures
                    let normal_tex = self.resolve_button_texture(f, "NormalTexture", &state.widgets);
                    let pushed_tex = self.resolve_button_texture(f, "PushedTexture", &state.widgets);
                    let highlight_tex = self.resolve_button_texture(f, "HighlightTexture", &state.widgets);
                    self.draw_button_widget(frame, bounds, f, id, normal_tex.as_deref(), pushed_tex.as_deref(), highlight_tex.as_deref());
                }
                WidgetType::Texture => {
                    self.draw_texture_widget(frame, bounds, f);
                }
                WidgetType::FontString => {
                    self.draw_fontstring_widget(frame, bounds, f);
                }
                WidgetType::EditBox => {
                    self.draw_editbox_widget(frame, bounds, f);
                }
                WidgetType::CheckButton => {
                    self.draw_checkbutton_widget(frame, bounds, f, checked);
                }
                WidgetType::ScrollFrame => {
                    self.draw_scrollframe_widget(frame, bounds, f);
                }
                WidgetType::Slider => {
                    self.draw_slider_widget(frame, bounds);
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
        }

        // Draw scroll bar for AddonList if it exists
        if let Some(ref rect) = addonlist_rect {
            self.draw_addon_list_scrollbar(frame, rect, screen_width);
        }

        // Draw center crosshair
        let cx = screen_width / 2.0;
        let cy = screen_height / 2.0;
        let crosshair_color = Color::from_rgba(1.0, 1.0, 1.0, 0.3);

        frame.stroke(
            &Path::line(Point::new(cx - 20.0, cy), Point::new(cx + 20.0, cy)),
            Stroke::default()
                .with_color(crosshair_color)
                .with_width(1.0),
        );
        frame.stroke(
            &Path::line(Point::new(cx, cy - 20.0), Point::new(cx, cy + 20.0)),
            Stroke::default()
                .with_color(crosshair_color)
                .with_width(1.0),
        );
    }

    fn draw_frame_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
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

    fn draw_button_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
        frame_id: u64,
        normal_tex: Option<&str>,
        pushed_tex: Option<&str>,
        highlight_tex: Option<&str>,
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

        // Select texture based on button state: pressed > normal
        let button_texture = if is_pressed {
            pushed_tex.or(normal_tex)
        } else {
            normal_tex
        };

        // Try single texture (pushed or normal)
        let mut drew_background = false;
        if let Some(tex_path) = button_texture {
            // Panel button textures use TexCoords: 0.625 width, 0.6875 height
            // The actual content is 80x22 within a 128x32 texture
            let handle = if tex_path.to_lowercase().contains("ui-panel-button") {
                // Load just the visible portion using TexCoords
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
                // Panel button highlight uses same TexCoords: 0.625 width, 0.6875 height
                let handle = if highlight_path.to_lowercase().contains("ui-panel-button") {
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
