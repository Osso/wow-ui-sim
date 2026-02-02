//! Iced-based UI for rendering WoW frames.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Event, Geometry, Path, Stroke};
use iced::widget::image::Handle as ImageHandle;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Border, Color, Element, Font, Length, Point, Rectangle, Size, Theme};
use iced::{Subscription, Task};

use iced_layout_inspector::server::{self as debug_server, Command as DebugCommand};
use tokio::sync::mpsc;

use crate::lua_api::WowLuaEnv;
use crate::render::text::{strip_wow_markup, wow_font_to_iced, TextRenderer};
use crate::render::texture::UI_SCALE;
use crate::texture::TextureManager;
use crate::widget::{AnchorPoint, TextJustify, WidgetType};
use crate::LayoutRect;

/// Default path to wow-ui-textures repository.
const DEFAULT_TEXTURES_PATH: &str = "/home/osso/Repos/wow-ui-textures";

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
    run_iced_ui_with_textures(env, PathBuf::from(DEFAULT_TEXTURES_PATH))
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
    image_handles: HashMap<String, ImageHandle>,
    frame_cache: Cache,
    hovered_frame: Option<u64>,
    pressed_frame: Option<u64>,
    mouse_down_frame: Option<u64>,
    scroll_offset: f32,
    screen_size: Size,
    debug_rx: Option<mpsc::Receiver<DebugCommand>>,
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
        let textures_path = INIT_TEXTURES
            .with(|cell| cell.borrow_mut().take())
            .unwrap_or_else(|| PathBuf::from(DEFAULT_TEXTURES_PATH));

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

        let app = App {
            env: env_rc,
            log_messages,
            command_input: String::new(),
            texture_manager,
            image_handles: HashMap::new(),
            frame_cache: Cache::new(),
            hovered_frame: None,
            pressed_frame: None,
            mouse_down_frame: None,
            scroll_offset: 0.0,
            screen_size: Size::new(600.0, 450.0),
            debug_rx: Some(cmd_rx),
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
                self.process_debug_commands();
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

    fn process_debug_commands(&mut self) {
        // Collect commands first to avoid borrow issues
        let commands: Vec<_> = if let Some(ref mut rx) = self.debug_rx {
            let mut cmds = Vec::new();
            while let Ok(cmd) = rx.try_recv() {
                cmds.push(cmd);
            }
            cmds
        } else {
            Vec::new()
        };

        // Then handle them
        for cmd in commands {
            Self::handle_debug_command(cmd);
        }
    }

    fn handle_debug_command(cmd: DebugCommand) {
        match cmd {
            DebugCommand::Dump { respond } => {
                let _ = respond.send("iced dump not yet implemented".to_string());
            }
            DebugCommand::Click { label, respond } => {
                let _ = respond.send(Err(format!("Click not implemented for '{}'", label)));
            }
            DebugCommand::Input {
                field,
                value: _,
                respond,
            } => {
                let _ = respond.send(Err(format!("Input not implemented for '{}'", field)));
            }
            DebugCommand::Submit { respond } => {
                let _ = respond.send(Err("Submit not implemented".to_string()));
            }
            DebugCommand::Screenshot { respond } => {
                let _ = respond.send(Err("Screenshot not implemented".into()));
            }
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
                let checked = if let Some(crate::widget::AttributeValue::Boolean(c)) =
                    f.attributes.get("__checked")
                {
                    *c
                } else {
                    false
                };
                Some((id, f, rect, checked))
            })
            .collect();

        frames.sort_by(|a, b| {
            a.1.frame_strata
                .cmp(&b.1.frame_strata)
                .then_with(|| a.1.frame_level.cmp(&b.1.frame_level))
                .then_with(|| {
                    let type_order = |t: &WidgetType| match t {
                        WidgetType::Texture => 0,
                        WidgetType::FontString => 1,
                        WidgetType::Frame => 2,
                        _ => 3,
                    };
                    type_order(&a.1.widget_type).cmp(&type_order(&b.1.widget_type))
                })
                .then_with(|| a.0.cmp(&b.0))
        });

        for (id, f, rect, checked) in frames {
            // Only show AddonList frame and children for now
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
            let mut y = rect.y * UI_SCALE;
            let w = rect.width * UI_SCALE;
            let h = rect.height * UI_SCALE;

            // Apply scroll offset for AddonList children
            let is_addonlist_root = addonlist_id == Some(id);
            if !is_addonlist_root && addonlist_ids.contains(&id) {
                y -= self.scroll_offset;
                let content_top = 260.0;
                let content_bottom = 700.0;
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
                    self.draw_button_widget(frame, bounds, f);
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

        // Special handling for AddonList frame title bar
        if f.name.as_deref() == Some("AddonList") {
            let title_height = 24.0;

            // Title bar background
            frame.fill_rectangle(
                bounds.position(),
                Size::new(bounds.width, title_height),
                Color::from_rgba(0.15, 0.12, 0.08, 0.95 * f.alpha),
            );

            // Title bar bottom border (gold)
            frame.stroke(
                &Path::line(
                    Point::new(bounds.x, bounds.y + title_height),
                    Point::new(bounds.x + bounds.width, bounds.y + title_height),
                ),
                Stroke::default()
                    .with_color(Color::from_rgba(0.8, 0.6, 0.2, f.alpha))
                    .with_width(2.0),
            );

            // Title text "Addons"
            TextRenderer::draw_justified_text(
                frame,
                "Addons",
                Rectangle::new(
                    Point::new(bounds.x + 10.0, bounds.y),
                    Size::new(bounds.width - 60.0, title_height),
                ),
                16.0,
                Color::from_rgba(1.0, 0.85, 0.4, f.alpha),
                Font::DEFAULT,
                TextJustify::Left,
                TextJustify::Center,
            );

            // Close button
            let close_size = 18.0;
            let close_x = bounds.x + bounds.width - close_size - 6.0;
            let close_y = bounds.y + (title_height - close_size) / 2.0;

            frame.fill_rectangle(
                Point::new(close_x, close_y),
                Size::new(close_size, close_size),
                Color::from_rgba(0.5, 0.2, 0.2, 0.8 * f.alpha),
            );

            // X mark
            let margin = 4.0;
            let x_color = Color::from_rgba(1.0, 0.9, 0.7, f.alpha);
            frame.stroke(
                &Path::line(
                    Point::new(close_x + margin, close_y + margin),
                    Point::new(close_x + close_size - margin, close_y + close_size - margin),
                ),
                Stroke::default().with_color(x_color).with_width(2.0),
            );
            frame.stroke(
                &Path::line(
                    Point::new(close_x + close_size - margin, close_y + margin),
                    Point::new(close_x + margin, close_y + close_size - margin),
                ),
                Stroke::default().with_color(x_color).with_width(2.0),
            );
        }
    }

    fn draw_button_widget(
        &self,
        frame: &mut canvas::Frame,
        bounds: Rectangle,
        f: &crate::widget::Frame,
    ) {
        // Default button styling (dark red gradient-like)
        frame.fill_rectangle(
            bounds.position(),
            bounds.size(),
            Color::from_rgba(0.15, 0.05, 0.05, 0.95 * f.alpha),
        );

        frame.stroke(
            &Path::rectangle(bounds.position(), bounds.size()),
            Stroke::default()
                .with_color(Color::from_rgba(0.6, 0.45, 0.15, f.alpha))
                .with_width(1.5),
        );

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

        // Fallback placeholder
        frame.fill_rectangle(
            bounds.position(),
            bounds.size(),
            Color::from_rgba(0.4, 0.35, 0.3, 0.7 * f.alpha),
        );

        // Diagonal lines
        let line_color = Color::from_rgba(1.0, 1.0, 1.0, 0.2 * f.alpha);
        frame.stroke(
            &Path::line(
                bounds.position(),
                Point::new(bounds.x + bounds.width, bounds.y + bounds.height),
            ),
            Stroke::default().with_color(line_color).with_width(1.0),
        );
        frame.stroke(
            &Path::line(
                Point::new(bounds.x + bounds.width, bounds.y),
                Point::new(bounds.x, bounds.y + bounds.height),
            ),
            Stroke::default().with_color(line_color).with_width(1.0),
        );
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
        let box_size = bounds.height.min(bounds.width).min(20.0);
        let box_x = bounds.x + (bounds.width - box_size) / 2.0;
        let box_y = bounds.y + (bounds.height - box_size) / 2.0;

        // Checkbox background
        frame.fill_rectangle(
            Point::new(box_x, box_y),
            Size::new(box_size, box_size),
            Color::from_rgba(0.12, 0.12, 0.15, 0.9 * f.alpha),
        );

        // Checkbox border (gold)
        frame.stroke(
            &Path::rectangle(Point::new(box_x, box_y), Size::new(box_size, box_size)),
            Stroke::default()
                .with_color(Color::from_rgba(0.8, 0.6, 0.2, f.alpha))
                .with_width(1.5),
        );

        // Checkmark if checked
        if checked {
            let margin = box_size * 0.2;
            let check_x1 = box_x + margin;
            let check_y1 = box_y + box_size * 0.5;
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
                    .with_color(Color::from_rgba(1.0, 0.8, 0.2, f.alpha))
                    .with_width(2.5),
            );
        }

        // Label text
        if let Some(ref txt) = f.text {
            let text_x = bounds.x + bounds.width + 6.0;
            let text_bounds =
                Rectangle::new(Point::new(text_x, bounds.y), Size::new(200.0, bounds.height));
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

    let (parent_anchor_x, parent_anchor_y) = anchor_position(
        anchor.relative_point,
        parent_rect.x,
        parent_rect.y,
        parent_rect.width,
        parent_rect.height,
    );

    let target_x = parent_anchor_x + anchor.x_offset;
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
