//! Iced-based UI for rendering WoW frames.

use super::nine_slice::{
    button_texture_path, draw_button, draw_nine_slice, preload_nine_slice_textures, ButtonState,
    NineSliceFrame,
};
use super::LayoutRect;
use crate::lua_api::WowLuaEnv;
use crate::texture::TextureManager;
use crate::widget::{Backdrop, WidgetType};
use iced::widget::canvas::{self, Cache, Canvas, Geometry, Image, Path, Stroke};
use iced::widget::image::Handle as ImageHandle;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Border, Color, Element, Length, Point, Rectangle, Shadow, Size, Theme};

// WoW-inspired color palette for the UI
mod palette {
    use iced::Color;

    // Backgrounds
    pub const BG_DARK: Color = Color::from_rgb(0.08, 0.08, 0.10);
    pub const BG_PANEL: Color = Color::from_rgb(0.12, 0.12, 0.14);
    pub const BG_INPUT: Color = Color::from_rgb(0.06, 0.06, 0.08);

    // Accents - WoW gold
    pub const GOLD: Color = Color::from_rgb(0.85, 0.65, 0.13);
    pub const GOLD_DIM: Color = Color::from_rgb(0.55, 0.42, 0.10);

    // Text
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.90, 0.85);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.60, 0.58, 0.55);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.43, 0.40);

    // Borders
    pub const BORDER: Color = Color::from_rgb(0.25, 0.23, 0.20);
    pub const BORDER_HIGHLIGHT: Color = Color::from_rgb(0.40, 0.35, 0.25);

    // Console
    pub const CONSOLE_BG: Color = Color::from_rgb(0.04, 0.04, 0.05);
    pub const CONSOLE_TEXT: Color = Color::from_rgb(0.70, 0.85, 0.70);
}

// Custom container style for panels
fn panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(palette::BG_PANEL)),
        border: Border {
            color: palette::BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        text_color: Some(palette::TEXT_PRIMARY),
        snap: false,
    }
}

// Console panel style (darker)
fn console_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(palette::CONSOLE_BG)),
        border: Border {
            color: palette::BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        text_color: Some(palette::CONSOLE_TEXT),
        snap: false,
    }
}

// Canvas container (no border, dark bg)
fn canvas_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(palette::BG_DARK)),
        border: Border {
            color: palette::BORDER_HIGHLIGHT,
            width: 2.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        text_color: None,
        snap: false,
    }
}

// Main app container
fn app_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(palette::BG_DARK)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(palette::TEXT_PRIMARY),
        snap: false,
    }
}

// Event button style
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
        shadow: Shadow::default(),
        snap: false,
    }
}

// Run button style (gold accent)
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
        shadow: Shadow::default(),
        snap: false,
    }
}

// Text input style
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
use iced_layout_inspector::server::{self, Command};
use iced_layout_inspector::{LayoutDump, LayoutDumper, Viewport};
use std::time::Duration;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

// WoW-style color palette
mod wow_colors {
    use iced::Color;

    // Texture placeholder tint (used when texture file is missing)
    pub const TEXTURE_TINT: Color = Color::from_rgba(0.4, 0.35, 0.3, 0.7);
}

/// Default path to wow-ui-textures repository.
const DEFAULT_TEXTURES_PATH: &str = "/home/osso/Repos/wow-ui-textures";

/// Run the iced UI with the given Lua environment.
pub fn run_ui(env: WowLuaEnv) -> iced::Result {
    run_ui_with_textures(env, PathBuf::from(DEFAULT_TEXTURES_PATH))
}

/// Run the iced UI with the given Lua environment and textures path.
pub fn run_ui_with_textures(env: WowLuaEnv, textures_path: PathBuf) -> iced::Result {
    // Store in thread-local for the boot function
    INIT_ENV.with(|cell| *cell.borrow_mut() = Some(env));
    INIT_TEXTURES.with(|cell| *cell.borrow_mut() = Some(textures_path));

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .window_size((1024.0, 768.0))
        .title("WoW UI Simulator")
        .run()
}

// Thread-local storage for init params (iced 0.14 boot function can't capture)
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

struct App {
    env: Rc<RefCell<WowLuaEnv>>,
    log_messages: Vec<String>,
    frame_cache: Cache,
    texture_manager: RefCell<TextureManager>,
    /// Cache of loaded texture image handles (wow_path -> Handle).
    image_handles: RefCell<HashMap<String, ImageHandle>>,
    /// Current text in the command input field.
    command_input: String,
    /// Debug server command receiver.
    debug_rx: server::CommandReceiver,
    /// Pending layout dump response.
    pending_dump_respond: Option<tokio::sync::oneshot::Sender<String>>,
    /// Debug server socket guard (keeps socket alive).
    _debug_guard: server::SocketGuard,
    /// Current window size for layout dumps.
    window_size: iced::Size,
}

/// Owned frame info for rendering.
#[derive(Debug, Clone)]
struct FrameInfo {
    id: u64,
    name: Option<String>,
    widget_type: WidgetType,
    rect: LayoutRect,
    visible: bool,
    frame_strata: crate::widget::FrameStrata,
    frame_level: i32,
    alpha: f32,
    text: Option<String>,
    text_color: crate::widget::Color,
    backdrop: Backdrop,
    vertex_color: Option<crate::widget::Color>,
    mouse_enabled: bool,
    /// Texture path for Texture widgets.
    texture: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    FireEvent(String),
    // Mouse interaction messages
    MouseTransition {
        leave: Option<u64>,
        enter: Option<u64>,
    },
    MouseDown(u64, String),
    MouseUp(u64, String),
    Click(u64, String),
    // Keyboard
    ReloadUI,
    // Command input
    CommandInputChanged(String),
    ExecuteCommand,
    // Debug server
    DebugPoll,
    DumpLayout,
    LayoutDumped(LayoutDump),
    // No-op for unhandled events
    NoOp,
}

/// State for canvas mouse interaction tracking.
#[derive(Debug, Default)]
struct CanvasState {
    /// Currently hovered frame ID (topmost mouse-enabled frame under cursor).
    hovered_frame: Option<u64>,
    /// Frame that received mouse down (for click detection).
    mouse_down_frame: Option<u64>,
    /// Which button is pressed (for click detection).
    mouse_down_button: Option<iced::mouse::Button>,
}

impl App {
    fn new() -> (Self, iced::Task<Message>) {
        let env = INIT_ENV.with(|cell| cell.borrow_mut().take().expect("INIT_ENV not set"));
        let textures_path = INIT_TEXTURES.with(|cell| cell.borrow_mut().take().expect("INIT_TEXTURES not set"));

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

        // Initialize debug server
        let (debug_rx, debug_guard) = server::init();
        eprintln!("[wow-ui-sim] Debug server at {}", server::socket_path().display());

        (
            App {
                env: env_rc,
                log_messages,
                frame_cache: Cache::new(),
                texture_manager: RefCell::new(TextureManager::new(textures_path)),
                image_handles: RefCell::new(HashMap::new()),
                command_input: String::new(),
                debug_rx,
                pending_dump_respond: None,
                _debug_guard: debug_guard,
                window_size: iced::Size::new(1024.0, 768.0),
            },
            iced::Task::none(),
        )
    }

    /// Drain console output from Lua and add to log messages.
    fn drain_console(&mut self) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        self.log_messages.append(&mut state.console_output);
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
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
            Message::MouseTransition { leave, enter } => {
                {
                    let env = self.env.borrow();
                    if let Some(frame_id) = leave {
                        if let Err(e) = env.fire_script_handler(frame_id, "OnLeave", vec![]) {
                            self.log_messages.push(format!("OnLeave error: {}", e));
                        }
                    }
                    if let Some(frame_id) = enter {
                        if let Err(e) = env.fire_script_handler(frame_id, "OnEnter", vec![]) {
                            self.log_messages.push(format!("OnEnter error: {}", e));
                        }
                    }
                }
                self.drain_console();
                self.frame_cache.clear();
            }
            Message::MouseDown(frame_id, button) => {
                {
                    let env = self.env.borrow();
                    let button_val =
                        mlua::Value::String(env.lua().create_string(&button).unwrap());
                    if let Err(e) =
                        env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val])
                    {
                        self.log_messages.push(format!("OnMouseDown error: {}", e));
                    }
                }
                self.drain_console();
                self.frame_cache.clear();
            }
            Message::MouseUp(frame_id, button) => {
                {
                    let env = self.env.borrow();
                    let button_val =
                        mlua::Value::String(env.lua().create_string(&button).unwrap());
                    if let Err(e) =
                        env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val])
                    {
                        self.log_messages.push(format!("OnMouseUp error: {}", e));
                    }
                }
                self.drain_console();
                self.frame_cache.clear();
            }
            Message::Click(frame_id, button) => {
                {
                    let env = self.env.borrow();
                    let button_val =
                        mlua::Value::String(env.lua().create_string(&button).unwrap());
                    let down_val = mlua::Value::Boolean(false); // Mouse click, not keyboard
                    if let Err(e) =
                        env.fire_script_handler(frame_id, "OnClick", vec![button_val, down_val])
                    {
                        self.log_messages.push(format!("OnClick error: {}", e));
                    }
                }
                self.drain_console();
                self.frame_cache.clear();
            }
            Message::ReloadUI => {
                self.log_messages.push("Reloading UI...".to_string());
                {
                    let env = self.env.borrow();

                    // Fire ADDON_LOADED
                    if let Ok(s) = env.lua().create_string("WoWUISim") {
                        let _ =
                            env.fire_event_with_args("ADDON_LOADED", &[mlua::Value::String(s)]);
                    }

                    // Fire PLAYER_LOGIN
                    let _ = env.fire_event("PLAYER_LOGIN");

                    // Fire PLAYER_ENTERING_WORLD with isReloadingUi = true
                    let _ = env.fire_event_with_args(
                        "PLAYER_ENTERING_WORLD",
                        &[
                            mlua::Value::Boolean(false), // isInitialLogin
                            mlua::Value::Boolean(true),  // isReloadingUi
                        ],
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
                    {
                        let env = self.env.borrow();
                        match env.dispatch_slash_command(&cmd) {
                            Ok(true) => {
                                // Command was handled
                            }
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
            Message::NoOp => {}
            Message::DebugPoll => {
                // Poll for debug commands
                while let Ok(cmd) = self.debug_rx.try_recv() {
                    match cmd {
                        Command::Dump { respond } => {
                            self.pending_dump_respond = Some(respond);
                            return iced::Task::done(Message::DumpLayout);
                        }
                        Command::Input { field, value, respond } => {
                            // Handle text input commands
                            let result = if field.to_lowercase().contains("command") || field.to_lowercase().contains("slash") {
                                self.command_input = value;
                                Ok(())
                            } else {
                                Err(format!("Unknown field: {}", field))
                            };
                            let _ = respond.send(result);
                        }
                        Command::Click { label, respond } => {
                            // Handle button click commands
                            let result = match label.to_lowercase().as_str() {
                                "run" => {
                                    let _ = respond.send(Ok(()));
                                    return iced::Task::done(Message::ExecuteCommand);
                                }
                                _ => Err(format!("Unknown button: {}", label)),
                            };
                            let _ = respond.send(result);
                        }
                        Command::Submit { respond } => {
                            let _ = respond.send(Ok(()));
                            return iced::Task::done(Message::ExecuteCommand);
                        }
                    }
                }
            }
            Message::DumpLayout => {
                let viewport = Viewport::new(self.window_size.width, self.window_size.height);
                return iced_runtime::task::widget(LayoutDumper::new(viewport))
                    .map(Message::LayoutDumped);
            }
            Message::LayoutDumped(dump) => {
                if let Some(respond) = self.pending_dump_respond.take() {
                    let _ = respond.send(dump.to_string());
                }
            }
        }
        iced::Task::none()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        use iced::keyboard;
        use iced::time;

        let keyboard_sub = keyboard::listen().map(|event| {
            if let keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            } = event
            {
                if modifiers.control() && c.as_str() == "r" {
                    return Message::ReloadUI;
                }
            }
            Message::NoOp
        });

        // Poll for debug commands every 50ms
        let debug_sub = time::every(Duration::from_millis(50)).map(|_| Message::DebugPoll);

        iced::Subscription::batch([keyboard_sub, debug_sub])
    }

    fn view(&self) -> Element<'_, Message> {
        // Collect frame info while holding the borrow
        let (frame_infos, frame_list_items): (Vec<FrameInfo>, Vec<String>) = {
            let env = self.env.borrow();
            let state = env.state().borrow();

            let mut infos = Vec::new();
            let mut list_items = Vec::new();

            for id in state.widgets.all_ids() {
                if let Some(frame) = state.widgets.get(id) {
                    let rect = compute_frame_rect_owned(&state.widgets, id, 500.0, 375.0);
                    infos.push(FrameInfo {
                        id,
                        name: frame.name.clone(),
                        widget_type: frame.widget_type,
                        rect,
                        visible: frame.visible,
                        frame_strata: frame.frame_strata,
                        frame_level: frame.frame_level,
                        alpha: frame.alpha,
                        text: frame.text.clone(),
                        text_color: frame.text_color,
                        backdrop: frame.backdrop.clone(),
                        vertex_color: frame.vertex_color,
                        mouse_enabled: frame.mouse_enabled,
                        texture: frame.texture.clone(),
                    });

                    let name = frame.name.as_deref().unwrap_or("(anonymous)");
                    let visible = if frame.visible { "visible" } else { "hidden" };
                    list_items.push(format!(
                        "{} [{}] {}x{} ({})",
                        name,
                        frame.widget_type.as_str(),
                        frame.width,
                        frame.height,
                        visible
                    ));
                }
            }

            // Sort by strata then level for proper z-ordering
            infos.sort_by(|a, b| {
                a.frame_strata
                    .cmp(&b.frame_strata)
                    .then_with(|| a.frame_level.cmp(&b.frame_level))
            });

            (infos, list_items)
        };

        // Load textures for any frames that need them
        {
            let mut tex_mgr = self.texture_manager.borrow_mut();
            let mut handles = self.image_handles.borrow_mut();
            let mut new_textures = false;

            // Preload 9-slice frame textures
            let nine_slice = NineSliceFrame::dialog_frame();
            let prev_count = handles.len();
            preload_nine_slice_textures(&mut tex_mgr, &mut handles, &nine_slice);
            if handles.len() > prev_count {
                new_textures = true;
            }

            // Preload button textures
            for state in [
                ButtonState::Normal,
                ButtonState::Hover,
                ButtonState::Pressed,
                ButtonState::Disabled,
            ] {
                let path = button_texture_path(state);
                if !handles.contains_key(path) {
                    if let Some(tex_data) = tex_mgr.load(path) {
                        let handle = ImageHandle::from_rgba(
                            tex_data.width,
                            tex_data.height,
                            tex_data.pixels.clone(),
                        );
                        handles.insert(path.to_string(), handle);
                        new_textures = true;
                    }
                }
            }

            // Load textures for individual frames
            for info in &frame_infos {
                if let Some(ref tex_path) = info.texture {
                    // Skip if already loaded
                    if handles.contains_key(tex_path) {
                        continue;
                    }

                    // Try to load the texture
                    if let Some(tex_data) = tex_mgr.load(tex_path) {
                        let handle = ImageHandle::from_rgba(
                            tex_data.width,
                            tex_data.height,
                            tex_data.pixels.clone(),
                        );
                        handles.insert(tex_path.clone(), handle);
                        new_textures = true;
                    }
                }
            }

            // Clear cache to force re-render with new textures
            if new_textures {
                self.frame_cache.clear();
            }
        }

        // Create canvas for rendering frames
        let canvas = Canvas::new(FrameRenderer {
            frames: frame_infos,
            cache: &self.frame_cache,
            image_handles: self.image_handles.borrow().clone(),
        })
        .width(Length::Fill)
        .height(Length::Fill);

        // Frame list sidebar - filter to only show meaningful frames
        let mut frame_list = Column::new().spacing(2).padding(6);
        frame_list = frame_list.push(
            text("Frames")
                .size(14)
                .color(palette::GOLD),
        );

        for item in frame_list_items
            .iter()
            .filter(|item| {
                // Only show frames with actual names and reasonable size
                !item.starts_with("(anonymous)")
                    && !item.starts_with("__")
                    && !item.contains("0x0")
                    && !item.starts_with("DBM")
                    && !item.starts_with("Details")
                    && !item.starts_with("Avatar")
                    && !item.starts_with("Plater")
                    && !item.starts_with("WeakAuras")
                    && !item.starts_with("UIWidget")
                    && !item.starts_with("GameMenu")
            })
            .take(15)
        {
            // Truncate long names
            let display = if item.len() > 25 {
                format!("{}...", &item[..22])
            } else {
                item.clone()
            };
            frame_list = frame_list.push(
                text(display)
                    .size(10)
                    .color(palette::TEXT_SECONDARY),
            );
        }

        // Log area - limit to 5 messages to save space
        let mut log_col = Column::new().spacing(2).padding(6);
        log_col = log_col.push(
            text("Console")
                .size(14)
                .color(palette::GOLD),
        );
        for msg in self.log_messages.iter().rev().take(5) {
            // Truncate long messages
            let display = if msg.len() > 80 {
                format!("{}...", &msg[..77])
            } else {
                msg.clone()
            };
            log_col = log_col.push(
                text(display)
                    .size(11)
                    .color(palette::CONSOLE_TEXT),
            );
        }

        // Event buttons
        let event_buttons = row![
            button("ADDON_LOADED")
                .on_press(Message::FireEvent("ADDON_LOADED".to_string()))
                .style(event_button_style),
            button("PLAYER_LOGIN")
                .on_press(Message::FireEvent("PLAYER_LOGIN".to_string()))
                .style(event_button_style),
            button("PLAYER_ENTERING_WORLD")
                .on_press(Message::FireEvent("PLAYER_ENTERING_WORLD".to_string()))
                .style(event_button_style),
        ]
        .spacing(8);

        // Command input row
        let command_row = row![
            text_input("/command", &self.command_input)
                .on_input(Message::CommandInputChanged)
                .on_submit(Message::ExecuteCommand)
                .width(Length::Fill)
                .style(input_style),
            button("Run")
                .on_press(Message::ExecuteCommand)
                .style(run_button_style),
        ]
        .spacing(8);

        // Main layout - use fixed heights to prevent overflow
        // Window: 768, padding: 16, title: 26, buttons: 31, command: 31, console: 100, spacing: 30
        // Available for canvas: 768 - 16 - 26 - 31 - 31 - 100 - 30 = 534px
        let content = column![
            text("WoW UI Simulator")
                .size(20)
                .color(palette::GOLD),
            row![
                container(canvas)
                    .width(Length::Fill)
                    .height(Length::Fixed(500.0))
                    .style(canvas_style),
                container(scrollable(frame_list).width(Length::Fixed(180.0)))
                    .height(Length::Fixed(500.0))
                    .style(panel_style),
            ],
            event_buttons,
            command_row,
            container(scrollable(log_col))
                .height(Length::Fixed(100.0))
                .width(Length::Fill)
                .style(console_style),
        ]
        .spacing(6)
        .padding(8);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(app_style)
            .into()
    }
}

/// Canvas renderer for WoW frames.
struct FrameRenderer<'a> {
    frames: Vec<FrameInfo>,
    cache: &'a Cache,
    /// Image handles for textures (keyed by wow path).
    image_handles: HashMap<String, ImageHandle>,
}

impl canvas::Program<Message> for FrameRenderer<'_> {
    type State = CanvasState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        // Extract hover/press state for button rendering
        let hovered_id = state.hovered_frame;
        let pressed_id = if state.mouse_down_button.is_some() {
            state.mouse_down_frame
        } else {
            None
        };

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Draw WoW-style dark background
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgb(0.05, 0.05, 0.08),
            );

            // Recompute layout for actual canvas size
            let scale_x = bounds.width / 500.0;
            let scale_y = bounds.height / 375.0;

            // Draw each visible frame (already sorted by strata/level)
            for info in self.frames.iter() {
                if !info.visible {
                    continue;
                }

                // Skip frames with no size
                if info.rect.width <= 0.0 || info.rect.height <= 0.0 {
                    continue;
                }

                // Skip built-in and addon internal frames
                if let Some(name) = &info.name {
                    if matches!(
                        name.as_str(),
                        "UIParent" | "Minimap" | "AddonCompartmentFrame"
                    ) {
                        continue;
                    }
                    // Skip addon internal frames
                    if name.starts_with("DBM")
                        || name.starts_with("Details")
                        || name.starts_with("Avatar")
                        || name.starts_with("Plater")
                        || name.starts_with("WeakAuras")
                        || name.starts_with("UIWidget")
                        || name.starts_with("GameMenu")
                        || name.starts_with("__")
                    {
                        continue;
                    }
                }

                let rect = LayoutRect {
                    x: info.rect.x * scale_x,
                    y: info.rect.y * scale_y,
                    width: info.rect.width * scale_x,
                    height: info.rect.height * scale_y,
                };

                // Draw based on widget type
                match info.widget_type {
                    WidgetType::Frame => {
                        draw_wow_frame(frame, &rect, info, &self.image_handles);
                    }
                    WidgetType::Button => {
                        // Determine button visual state
                        let button_state = if pressed_id == Some(info.id) {
                            ButtonState::Pressed
                        } else if hovered_id == Some(info.id) {
                            ButtonState::Hover
                        } else {
                            ButtonState::Normal
                        };
                        draw_wow_button(frame, &rect, info, &self.image_handles, button_state);
                    }
                    WidgetType::Texture => {
                        draw_wow_texture(frame, &rect, info, &self.image_handles);
                    }
                    WidgetType::FontString => {
                        draw_wow_fontstring(frame, &rect, info);
                    }
                }
            }

            // Draw coordinate guides (crosshair at center)
            let center_x = bounds.width / 2.0;
            let center_y = bounds.height / 2.0;

            let h_line = Path::line(
                Point::new(center_x - 20.0, center_y),
                Point::new(center_x + 20.0, center_y),
            );
            let v_line = Path::line(
                Point::new(center_x, center_y - 20.0),
                Point::new(center_x, center_y + 20.0),
            );
            let guide_stroke = Stroke::default()
                .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.3))
                .with_width(1.0);
            frame.stroke(&h_line, guide_stroke.clone());
            frame.stroke(&v_line, guide_stroke);
        });

        vec![geometry]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        use iced::mouse::Event as MouseEvent;

        let cursor_position = match cursor.position_in(bounds) {
            Some(pos) => pos,
            None => {
                // Cursor left canvas - fire OnLeave if we had a hovered frame
                if let Some(old_frame) = state.hovered_frame.take() {
                    return Some(canvas::Action::publish(Message::MouseTransition {
                        leave: Some(old_frame),
                        enter: None,
                    }));
                }
                return None;
            }
        };

        match event {
            iced::Event::Mouse(MouseEvent::CursorMoved { .. }) => {
                let new_hovered = hit_test_frames(&self.frames, cursor_position, bounds);

                // Handle hover state changes
                if new_hovered != state.hovered_frame {
                    let leave = state.hovered_frame;
                    state.hovered_frame = new_hovered;

                    if leave.is_some() || new_hovered.is_some() {
                        return Some(canvas::Action::publish(Message::MouseTransition {
                            leave,
                            enter: new_hovered,
                        }));
                    }
                }
                None
            }

            iced::Event::Mouse(MouseEvent::ButtonPressed(button)) => {
                if let Some(frame_id) = state.hovered_frame {
                    state.mouse_down_frame = Some(frame_id);
                    state.mouse_down_button = Some(*button);
                    let button_name = mouse_button_name(*button);
                    return Some(canvas::Action::publish(Message::MouseDown(
                        frame_id,
                        button_name,
                    )));
                }
                None
            }

            iced::Event::Mouse(MouseEvent::ButtonReleased(button)) => {
                let message = if let Some(frame_id) = state.hovered_frame {
                    let button_name = mouse_button_name(*button);

                    // Check if this is a click (same frame as mouse down)
                    if state.mouse_down_frame == Some(frame_id)
                        && state.mouse_down_button == Some(*button)
                    {
                        // Fire Click instead of just MouseUp
                        Some(Message::Click(frame_id, button_name))
                    } else {
                        Some(Message::MouseUp(frame_id, button_name))
                    }
                } else {
                    None
                };

                // Clear mouse down state
                state.mouse_down_frame = None;
                state.mouse_down_button = None;

                message.map(canvas::Action::publish)
            }

            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if state.hovered_frame.is_some() {
            iced::mouse::Interaction::Pointer
        } else {
            iced::mouse::Interaction::default()
        }
    }
}

/// Find the topmost mouse-enabled frame at the given canvas position.
/// Frames are checked in reverse z-order (topmost first).
fn hit_test_frames(frames: &[FrameInfo], point: Point, bounds: Rectangle) -> Option<u64> {
    // Must match the reference size used in draw() for coordinate scaling
    let scale_x = bounds.width / 500.0;
    let scale_y = bounds.height / 375.0;

    // Iterate in reverse (topmost frames are at the end after sorting)
    for info in frames.iter().rev() {
        if !info.visible || !info.mouse_enabled {
            continue;
        }

        // Skip built-in frames
        if matches!(info.name.as_deref(), Some("UIParent") | Some("Minimap")) {
            continue;
        }

        // Scale rect to canvas size
        let rect = Rectangle {
            x: info.rect.x * scale_x,
            y: info.rect.y * scale_y,
            width: info.rect.width * scale_x,
            height: info.rect.height * scale_y,
        };

        if rect.contains(point) {
            return Some(info.id);
        }
    }
    None
}

/// Convert iced mouse button to WoW button name.
fn mouse_button_name(button: iced::mouse::Button) -> String {
    match button {
        iced::mouse::Button::Left => "LeftButton".to_string(),
        iced::mouse::Button::Right => "RightButton".to_string(),
        iced::mouse::Button::Middle => "MiddleButton".to_string(),
        iced::mouse::Button::Other(n) => format!("Button{}", n),
        _ => "Unknown".to_string(),
    }
}

/// Compute frame rect - owned version that doesn't borrow.
fn compute_frame_rect_owned(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    screen_width: f32,
    screen_height: f32,
) -> LayoutRect {

    let frame = match registry.get(id) {
        Some(f) => f,
        None => return LayoutRect::default(),
    };

    let width = frame.width;
    let height = frame.height;

    // If no anchors, default to center of parent
    if frame.anchors.is_empty() {
        let parent_rect = if let Some(parent_id) = frame.parent_id {
            compute_frame_rect_owned(registry, parent_id, screen_width, screen_height)
        } else {
            LayoutRect {
                x: 0.0,
                y: 0.0,
                width: screen_width,
                height: screen_height,
            }
        };

        return LayoutRect {
            x: parent_rect.x + (parent_rect.width - width) / 2.0,
            y: parent_rect.y + (parent_rect.height - height) / 2.0,
            width,
            height,
        };
    }

    let anchor = &frame.anchors[0];

    let parent_rect = if let Some(parent_id) = frame.parent_id {
        compute_frame_rect_owned(registry, parent_id, screen_width, screen_height)
    } else {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        }
    };

    let (parent_anchor_x, parent_anchor_y) = anchor_position(
        anchor.relative_point,
        parent_rect.x,
        parent_rect.y,
        parent_rect.width,
        parent_rect.height,
    );

    let target_x = parent_anchor_x + anchor.x_offset;
    // WoW uses Y-up coordinate system, screen uses Y-down, so negate Y offset
    let target_y = parent_anchor_y - anchor.y_offset;

    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect {
        x: frame_x,
        y: frame_y,
        width,
        height,
    }
}

fn anchor_position(
    point: crate::widget::AnchorPoint,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
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
    point: crate::widget::AnchorPoint,
    anchor_x: f32,
    anchor_y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
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

/// Draw a WoW-style frame with optional backdrop.
fn draw_wow_frame(
    frame: &mut canvas::Frame,
    rect: &LayoutRect,
    info: &FrameInfo,
    image_handles: &HashMap<String, ImageHandle>,
) {
    // Only draw frames that have a backdrop explicitly enabled
    if !info.backdrop.enabled {
        return;
    }

    let alpha = info.alpha;
    let bounds = Rectangle {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    };

    let nine_slice = NineSliceFrame::dialog_frame();
    draw_nine_slice(frame, bounds, &nine_slice, image_handles, alpha);
}

/// Draw a WoW-style button.
fn draw_wow_button(
    frame: &mut canvas::Frame,
    rect: &LayoutRect,
    info: &FrameInfo,
    image_handles: &HashMap<String, ImageHandle>,
    button_state: ButtonState,
) {
    let alpha = info.alpha;
    let bounds = Rectangle {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    };

    // Draw button texture with appropriate state
    draw_button(frame, bounds, button_state, image_handles, alpha);

    // Draw button text if present
    if let Some(text_content) = &info.text {
        let tc = &info.text_color;
        frame.fill_text(canvas::Text {
            content: text_content.clone(),
            position: Point::new(
                rect.x + rect.width / 2.0,
                rect.y + rect.height / 2.0 - 6.0,
            ),
            color: Color::from_rgba(tc.r, tc.g, tc.b, tc.a * alpha),
            size: iced::Pixels(12.0),
            align_x: iced::alignment::Alignment::Center.into(),
            ..Default::default()
        });
    }
}

/// Draw a WoW texture.
fn draw_wow_texture(
    frame: &mut canvas::Frame,
    rect: &LayoutRect,
    info: &FrameInfo,
    image_handles: &HashMap<String, ImageHandle>,
) {
    let alpha = info.alpha;

    // Try to render the actual texture if available
    if let Some(ref tex_path) = info.texture {
        if let Some(handle) = image_handles.get(tex_path) {
            let bounds = Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            };

            // Create image with optional vertex color tinting
            let image = if let Some(vc) = &info.vertex_color {
                Image::new(handle.clone())
                    .opacity(vc.a * alpha)
                    .filter_method(iced::widget::image::FilterMethod::Linear)
            } else {
                Image::new(handle.clone())
                    .opacity(alpha)
                    .filter_method(iced::widget::image::FilterMethod::Linear)
            };

            frame.draw_image(bounds, image);
            return;
        }
    }

    // Fallback: draw placeholder if no texture loaded
    let color = if let Some(vc) = &info.vertex_color {
        Color::from_rgba(vc.r, vc.g, vc.b, vc.a * alpha)
    } else {
        Color::from_rgba(
            wow_colors::TEXTURE_TINT.r,
            wow_colors::TEXTURE_TINT.g,
            wow_colors::TEXTURE_TINT.b,
            wow_colors::TEXTURE_TINT.a * alpha,
        )
    };

    // Fill texture area
    frame.fill_rectangle(
        Point::new(rect.x, rect.y),
        Size::new(rect.width, rect.height),
        color,
    );

    // Draw diagonal lines to indicate it's a texture placeholder
    let line1 = Path::line(
        Point::new(rect.x, rect.y),
        Point::new(rect.x + rect.width, rect.y + rect.height),
    );
    let line2 = Path::line(
        Point::new(rect.x + rect.width, rect.y),
        Point::new(rect.x, rect.y + rect.height),
    );
    let stroke = Stroke::default()
        .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.2 * alpha))
        .with_width(1.0);
    frame.stroke(&line1, stroke.clone());
    frame.stroke(&line2, stroke);
}

/// Draw a WoW FontString (text).
fn draw_wow_fontstring(frame: &mut canvas::Frame, rect: &LayoutRect, info: &FrameInfo) {
    let alpha = info.alpha;

    if let Some(text_content) = &info.text {
        let tc = &info.text_color;
        let color = Color::from_rgba(tc.r, tc.g, tc.b, tc.a * alpha);

        // Draw text centered in the rect
        frame.fill_text(canvas::Text {
            content: text_content.clone(),
            position: Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0 - 6.0),
            color,
            size: iced::Pixels(12.0),
            align_x: iced::alignment::Alignment::Center.into(),
            ..Default::default()
        });
    }
}
