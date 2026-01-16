//! GTK4/relm4-based UI for rendering WoW frames.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use gtk::gdk;
use gtk::glib;
use glib::ControlFlow;
use relm4::prelude::*;

use crate::lua_api::WowLuaEnv;
use crate::render::LayoutRect;
use crate::texture::TextureManager;
use crate::widget::WidgetType;
use gtk_layout_inspector::server::{self as debug_server, Command as DebugCommand};
use gtk_layout_inspector::{dump_widget_tree, find_button_by_label, find_entry_by_placeholder};

/// Custom CSS for WoW-style theming.
const STYLE_CSS: &str = include_str!("style.css");

/// Default path to wow-ui-textures repository.
const DEFAULT_TEXTURES_PATH: &str = "/home/osso/Repos/wow-ui-textures";

/// Run the GTK UI with the given Lua environment.
pub fn run_gtk_ui(env: WowLuaEnv) -> Result<(), Box<dyn std::error::Error>> {
    run_gtk_ui_with_textures(env, PathBuf::from(DEFAULT_TEXTURES_PATH))
}

/// Run the GTK UI with the given Lua environment and textures path.
pub fn run_gtk_ui_with_textures(
    env: WowLuaEnv,
    textures_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = RelmApp::new("com.osso.wow-ui-sim");
    app.run::<App>((env, textures_path));
    Ok(())
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

// Note: FrameInfo will be used in Phase 2 for texture rendering

/// App state
struct App {
    env: Rc<RefCell<WowLuaEnv>>,
    log_messages: Vec<String>,
    // Note: texture_manager and texture_cache will be used in Phase 2 for texture rendering
    #[allow(dead_code)]
    texture_manager: RefCell<TextureManager>,
    #[allow(dead_code)]
    texture_cache: RefCell<HashMap<String, TextureData>>,
    /// Current text in the command input field.
    command_input: String,
    /// Drawing area for WoW frames canvas.
    drawing_area: gtk::DrawingArea,
    /// Frames list sidebar content.
    frames_box: gtk::Box,
    /// Console output label.
    console_label: gtk::Label,
    /// Currently hovered frame ID.
    hovered_frame: Option<u64>,
    /// Frame that received mouse down (for click detection).
    mouse_down_frame: Option<u64>,
}

/// Loaded texture data (will be used in Phase 2)
#[derive(Clone)]
#[allow(dead_code)]
struct TextureData {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

#[derive(Debug)]
enum Msg {
    FireEvent(String),
    MouseMove(f64, f64),
    MousePress(f64, f64),
    MouseRelease(f64, f64),
    ReloadUI,
    CommandInputChanged(String),
    ExecuteCommand,
    #[allow(dead_code)]
    Redraw,
}

#[relm4::component]
impl Component for App {
    type Init = (WowLuaEnv, PathBuf);
    type Input = Msg;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("WoW UI Simulator"),
            set_default_size: (1024, 768),

            add_controller = gtk::EventControllerKey {
                set_propagation_phase: gtk::PropagationPhase::Capture,
                connect_key_pressed[sender] => move |_, key, _, modifiers| {
                    if modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                        if key == gdk::Key::r {
                            sender.input(Msg::ReloadUI);
                            return glib::Propagation::Stop;
                        }
                    }
                    glib::Propagation::Proceed
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 7,

                // Title
                gtk::Label {
                    set_label: "WoW UI Simulator",
                    set_halign: gtk::Align::Start,
                    add_css_class: "title-2",
                },

                // Main content area: Canvas + Sidebar
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,
                    set_vexpand: true,

                    // WoW Canvas
                    gtk::Frame {
                        set_hexpand: true,
                        add_css_class: "view",

                        #[local_ref]
                        drawing_area -> gtk::DrawingArea {
                            set_hexpand: true,
                            set_vexpand: true,
                            set_content_width: 600,
                            set_content_height: 450,
                        },
                    },

                    // Frames sidebar
                    gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_min_content_width: 180,

                        #[local_ref]
                        frames_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 2,
                            set_margin_all: 6,
                        },
                    },
                },

                // Event buttons
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,

                    gtk::Button {
                        set_label: "ADDON_LOADED",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::FireEvent("ADDON_LOADED".to_string()));
                        },
                    },

                    gtk::Button {
                        set_label: "PLAYER_LOGIN",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::FireEvent("PLAYER_LOGIN".to_string()));
                        },
                    },

                    gtk::Button {
                        set_label: "PLAYER_ENTERING_WORLD",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::FireEvent("PLAYER_ENTERING_WORLD".to_string()));
                        },
                    },
                },

                // Command input row
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,

                    gtk::Entry {
                        set_hexpand: true,
                        set_placeholder_text: Some("/command"),
                        #[watch]
                        set_text: &model.command_input,
                        connect_changed[sender] => move |entry| {
                            sender.input(Msg::CommandInputChanged(entry.text().to_string()));
                        },
                        connect_activate[sender] => move |_| {
                            sender.input(Msg::ExecuteCommand);
                        },
                    },

                    gtk::Button {
                        set_label: "Run",
                        add_css_class: "suggested-action",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::ExecuteCommand);
                        },
                    },
                },

                // Console output
                gtk::ScrolledWindow {
                    set_min_content_height: 100,
                    set_max_content_height: 140,
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                    #[local_ref]
                    console_label -> gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Start,
                        set_wrap: true,
                        set_selectable: true,
                        add_css_class: "monospace",
                    },
                },
            },
        }
    }

    fn init(
        (env, textures_path): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Load custom CSS with USER priority to override theme
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data(STYLE_CSS);
        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        // Force dark color scheme
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);

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

        // Create widgets
        let drawing_area = gtk::DrawingArea::new();
        let frames_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let console_label = gtk::Label::new(None);

        let model = App {
            env: env_rc.clone(),
            log_messages,
            texture_manager: RefCell::new(TextureManager::new(textures_path)),
            texture_cache: RefCell::new(HashMap::new()),
            command_input: String::new(),
            drawing_area: drawing_area.clone(),
            frames_box: frames_box.clone(),
            console_label: console_label.clone(),
            hovered_frame: None,
            mouse_down_frame: None,
        };

        // Update console label
        model.update_console_label();

        // Set up drawing area
        let env_for_draw = env_rc.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            draw_wow_frames(&env_for_draw, cr, width, height);
        });

        // Mouse motion controller
        let motion_controller = gtk::EventControllerMotion::new();
        let sender_motion = sender.clone();
        motion_controller.connect_motion(move |_, x, y| {
            sender_motion.input(Msg::MouseMove(x, y));
        });
        drawing_area.add_controller(motion_controller);

        // Mouse click controller
        let click_controller = gtk::GestureClick::new();
        let sender_press = sender.clone();
        let sender_release = sender.clone();
        click_controller.connect_pressed(move |_, _n, x, y| {
            sender_press.input(Msg::MousePress(x, y));
        });
        click_controller.connect_released(move |_, _n, x, y| {
            sender_release.input(Msg::MouseRelease(x, y));
        });
        drawing_area.add_controller(click_controller);

        let widgets = view_output!();

        // Update frames sidebar
        model.update_frames_sidebar();

        // Initialize debug server
        let (mut cmd_rx, _guard) = debug_server::init();
        eprintln!("[wow-ui-sim] Debug server at {}", debug_server::socket_path().display());

        // Keep guard alive
        let guard = Rc::new(RefCell::new(Some(_guard)));
        let guard_clone = guard.clone();

        // Store window reference and sender for debug commands
        let window_weak = root.downgrade();
        let debug_sender = sender.clone();

        // Poll for debug commands
        glib::timeout_add_local(Duration::from_millis(50), move || {
            let _guard = guard_clone.borrow();

            while let Ok(cmd) = cmd_rx.try_recv() {
                let Some(window) = window_weak.upgrade() else {
                    continue;
                };

                match cmd {
                    DebugCommand::Dump { respond } => {
                        let dump = dump_widget_tree(&window);
                        let _ = respond.send(dump.to_string());
                    }
                    DebugCommand::DumpJson { respond } => {
                        let dump = dump_widget_tree(&window);
                        let _ = respond.send(dump.to_json());
                    }
                    DebugCommand::Click { label, respond } => {
                        if let Some(button) = find_button_by_label(&window, &label) {
                            button.emit_clicked();
                            let _ = respond.send(Ok(()));
                        } else {
                            let _ = respond.send(Err(format!("Button '{}' not found", label)));
                        }
                    }
                    DebugCommand::Input { field, value, respond } => {
                        if let Some(entry) = find_entry_by_placeholder(&window, &field) {
                            entry.set_text(&value);
                            let _ = respond.send(Ok(()));
                        } else {
                            let _ = respond.send(Err(format!("Entry '{}' not found", field)));
                        }
                    }
                    DebugCommand::Submit { respond } => {
                        use gtk::prelude::GtkWindowExt;
                        if let Some(focus) = GtkWindowExt::focus(&window) {
                            focus.activate();
                            let _ = respond.send(Ok(()));
                        } else {
                            let _ = respond.send(Err("No focused widget".to_string()));
                        }
                    }
                    DebugCommand::KeyPress { key, respond } => {
                        let msg = match key.as_str() {
                            "ctrl+r" | "Ctrl+r" => Some(Msg::ReloadUI),
                            "Return" => Some(Msg::ExecuteCommand),
                            _ => None,
                        };
                        if let Some(m) = msg {
                            debug_sender.input(m);
                            let _ = respond.send(Ok(()));
                        } else {
                            let _ = respond.send(Err(format!("Unknown key '{}'", key)));
                        }
                    }
                    DebugCommand::Screenshot { respond } => {
                        // Take screenshot of the drawing area
                        let _ = respond.send(Err("Screenshot not yet implemented for GTK".to_string()));
                    }
                }
            }
            ControlFlow::Continue
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Msg, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            Msg::FireEvent(event) => {
                {
                    let env = self.env.borrow();
                    if let Err(e) = env.fire_event(&event) {
                        self.log_messages.push(format!("Event error: {}", e));
                    } else {
                        self.log_messages.push(format!("Fired: {}", event));
                    }
                }
                self.drain_console();
                self.drawing_area.queue_draw();
                self.update_frames_sidebar();
                self.update_console_label();
            }
            Msg::MouseMove(x, y) => {
                let new_hovered = self.hit_test(x, y);
                if new_hovered != self.hovered_frame {
                    // Fire OnLeave/OnEnter
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
                    self.drawing_area.queue_draw();
                    self.update_console_label();
                }
            }
            Msg::MousePress(x, y) => {
                if let Some(frame_id) = self.hit_test(x, y) {
                    self.mouse_down_frame = Some(frame_id);
                    let env = self.env.borrow();
                    let button_val = mlua::Value::String(env.lua().create_string("LeftButton").unwrap());
                    let _ = env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val]);
                    drop(env);
                    self.drain_console();
                    self.drawing_area.queue_draw();
                    self.update_console_label();
                }
            }
            Msg::MouseRelease(x, y) => {
                let released_on = self.hit_test(x, y);
                if let Some(frame_id) = released_on {
                    let env = self.env.borrow();
                    let button_val = mlua::Value::String(env.lua().create_string("LeftButton").unwrap());

                    // Fire OnClick if released on same frame as pressed
                    if self.mouse_down_frame == Some(frame_id) {
                        let down_val = mlua::Value::Boolean(false);
                        let _ = env.fire_script_handler(frame_id, "OnClick", vec![button_val.clone(), down_val]);
                    }

                    let _ = env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val]);
                    drop(env);
                    self.drain_console();
                    self.drawing_area.queue_draw();
                    self.update_console_label();
                }
                self.mouse_down_frame = None;
            }
            Msg::ReloadUI => {
                self.log_messages.push("Reloading UI...".to_string());
                {
                    let env = self.env.borrow();

                    if let Ok(s) = env.lua().create_string("WoWUISim") {
                        let _ = env.fire_event_with_args("ADDON_LOADED", &[mlua::Value::String(s)]);
                    }

                    let _ = env.fire_event("PLAYER_LOGIN");

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
                self.drawing_area.queue_draw();
                self.update_frames_sidebar();
                self.update_console_label();
            }
            Msg::CommandInputChanged(input) => {
                self.command_input = input;
            }
            Msg::ExecuteCommand => {
                let cmd = self.command_input.clone();
                if !cmd.is_empty() {
                    self.log_messages.push(format!("> {}", cmd));

                    // Handle /frames specially
                    let cmd_lower = cmd.to_lowercase();
                    if cmd_lower == "/frames" || cmd_lower == "/f" {
                        let env = self.env.borrow();
                        let dump = env.dump_frames();
                        eprintln!("{}", dump);
                        let line_count = dump.lines().count();
                        self.log_messages.push(format!("Dumped {} frames to stderr", line_count / 2));
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
                    self.drawing_area.queue_draw();
                    self.update_frames_sidebar();
                    self.update_console_label();
                }
            }
            Msg::Redraw => {
                self.drawing_area.queue_draw();
            }
        }
    }
}

impl App {
    /// Drain console output from Lua and add to log messages.
    fn drain_console(&mut self) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        self.log_messages.append(&mut state.console_output);
    }

    /// Update the console label with recent messages.
    fn update_console_label(&self) {
        let text: String = self.log_messages
            .iter()
            .rev()
            .take(10)
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        self.console_label.set_label(&text);
    }

    /// Update the frames sidebar.
    fn update_frames_sidebar(&self) {
        // Clear existing children
        while let Some(child) = self.frames_box.first_child() {
            self.frames_box.remove(&child);
        }

        // Add "Frames" header
        let header = gtk::Label::new(Some("Frames"));
        header.set_halign(gtk::Align::Start);
        header.add_css_class("heading");
        self.frames_box.append(&header);

        // Get frame list
        let env = self.env.borrow();
        let state = env.state().borrow();

        let mut count = 0;
        for id in state.widgets.all_ids() {
            if let Some(frame) = state.widgets.get(id) {
                // Filter out anonymous and internal frames
                let name = match &frame.name {
                    Some(n) => n.as_str(),
                    None => continue,
                };
                if name.starts_with("__") || name.starts_with("DBM") || name.starts_with("Details")
                    || name.starts_with("Avatar") || name.starts_with("Plater")
                    || name.starts_with("WeakAuras") || name.starts_with("UIWidget")
                    || name.starts_with("GameMenu")
                {
                    continue;
                }
                if frame.width <= 0.0 || frame.height <= 0.0 {
                    continue;
                }

                let visible = if frame.visible { "visible" } else { "hidden" };
                let text = format!(
                    "{} [{}] {}x{} ({})",
                    name,
                    frame.widget_type.as_str(),
                    frame.width as i32,
                    frame.height as i32,
                    visible
                );

                // Truncate long names
                let display = if text.len() > 30 {
                    format!("{}...", &text[..27])
                } else {
                    text
                };

                let label = gtk::Label::new(Some(&display));
                label.set_halign(gtk::Align::Start);
                label.add_css_class("dim-label");
                self.frames_box.append(&label);

                count += 1;
                if count >= 15 {
                    break;
                }
            }
        }
    }

    /// Hit test to find frame at given canvas coordinates.
    fn hit_test(&self, x: f64, y: f64) -> Option<u64> {
        let env = self.env.borrow();
        let state = env.state().borrow();

        let width = self.drawing_area.width() as f32;
        let height = self.drawing_area.height() as f32;
        let scale_x = width / 500.0;
        let scale_y = height / 375.0;

        // Collect frames and sort by z-order
        let mut frames: Vec<_> = state.widgets.all_ids()
            .into_iter()
            .filter_map(|id| {
                let frame = state.widgets.get(id)?;
                if !frame.visible || !frame.mouse_enabled {
                    return None;
                }
                if matches!(frame.name.as_deref(), Some("UIParent") | Some("Minimap")) {
                    return None;
                }
                let rect = compute_frame_rect(&state.widgets, id, 500.0, 375.0);
                Some((id, frame.frame_strata, frame.frame_level, rect))
            })
            .collect();

        frames.sort_by(|a, b| {
            a.1.cmp(&b.1)
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| a.0.cmp(&b.0))
        });

        // Check in reverse (topmost first)
        for (id, _, _, rect) in frames.iter().rev() {
            let scaled_x = rect.x * scale_x;
            let scaled_y = rect.y * scale_y;
            let scaled_w = rect.width * scale_x;
            let scaled_h = rect.height * scale_y;

            if x >= scaled_x as f64 && x <= (scaled_x + scaled_w) as f64
                && y >= scaled_y as f64 && y <= (scaled_y + scaled_h) as f64
            {
                return Some(*id);
            }
        }
        None
    }
}

/// Draw WoW frames using Cairo.
fn draw_wow_frames(env: &Rc<RefCell<WowLuaEnv>>, cr: &gtk::cairo::Context, width: i32, height: i32) {
    let env = env.borrow();
    let state = env.state().borrow();

    // Dark background
    cr.set_source_rgb(0.05, 0.05, 0.08);
    cr.paint().ok();

    let scale_x = width as f64 / 500.0;
    let scale_y = height as f64 / 375.0;

    // Collect and sort frames
    let mut frames: Vec<_> = state.widgets.all_ids()
        .into_iter()
        .filter_map(|id| {
            let frame = state.widgets.get(id)?;
            let rect = compute_frame_rect(&state.widgets, id, 500.0, 375.0);
            Some((
                id,
                frame.frame_strata,
                frame.frame_level,
                frame.widget_type,
                frame.visible,
                frame.alpha,
                frame.text.clone(),
                frame.text_color,
                frame.backdrop.clone(),
                frame.name.clone(),
                rect,
            ))
        })
        .collect();

    // Sort by strata, level, type, id
    frames.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| {
                let type_order = |t: &WidgetType| match t {
                    WidgetType::Texture => 0,
                    WidgetType::FontString => 1,
                    WidgetType::Frame => 2,
                    WidgetType::Button => 3,
                };
                type_order(&a.3).cmp(&type_order(&b.3))
            })
            .then_with(|| a.0.cmp(&b.0))
    });

    for (_id, _strata, _level, widget_type, visible, alpha, text, text_color, backdrop, name, rect) in frames {
        if !visible {
            continue;
        }
        if rect.width <= 0.0 || rect.height <= 0.0 {
            continue;
        }

        // Skip internal frames
        if let Some(ref n) = name {
            if matches!(n.as_str(), "UIParent" | "Minimap" | "AddonCompartmentFrame") {
                continue;
            }
            if n.starts_with("DBM") || n.starts_with("Details") || n.starts_with("Avatar")
                || n.starts_with("Plater") || n.starts_with("WeakAuras") || n.starts_with("UIWidget")
                || n.starts_with("GameMenu") || n.starts_with("__")
            {
                continue;
            }
        }

        let x = (rect.x as f64) * scale_x;
        let y = (rect.y as f64) * scale_y;
        let w = (rect.width as f64) * scale_x;
        let h = (rect.height as f64) * scale_y;

        match widget_type {
            WidgetType::Frame => {
                if backdrop.enabled {
                    // Draw frame background
                    cr.set_source_rgba(0.1, 0.1, 0.15, 0.9 * alpha as f64);
                    cr.rectangle(x, y, w, h);
                    cr.fill().ok();

                    // Draw border
                    cr.set_source_rgba(0.7, 0.55, 0.2, alpha as f64);
                    cr.set_line_width(2.0);
                    cr.rectangle(x, y, w, h);
                    cr.stroke().ok();
                }
            }
            WidgetType::Button => {
                // Draw button background
                cr.set_source_rgba(0.15, 0.12, 0.1, 0.9 * alpha as f64);
                cr.rectangle(x, y, w, h);
                cr.fill().ok();

                // Draw button border
                cr.set_source_rgba(0.6, 0.45, 0.15, alpha as f64);
                cr.set_line_width(1.5);
                cr.rectangle(x, y, w, h);
                cr.stroke().ok();

                // Draw button text (centered)
                if let Some(ref txt) = text {
                    cr.set_source_rgba(
                        text_color.r as f64,
                        text_color.g as f64,
                        text_color.b as f64,
                        text_color.a as f64 * alpha as f64,
                    );
                    cr.select_font_face("Sans", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Normal);
                    cr.set_font_size(12.0);

                    let extents = cr.text_extents(txt).unwrap();
                    let text_x = x + (w - extents.width()) / 2.0;
                    let text_y = y + (h + extents.height()) / 2.0;
                    cr.move_to(text_x, text_y);
                    cr.show_text(txt).ok();
                }
            }
            WidgetType::Texture => {
                // Draw texture placeholder
                cr.set_source_rgba(0.4, 0.35, 0.3, 0.7 * alpha as f64);
                cr.rectangle(x, y, w, h);
                cr.fill().ok();

                // Diagonal lines to indicate placeholder
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.2 * alpha as f64);
                cr.set_line_width(1.0);
                cr.move_to(x, y);
                cr.line_to(x + w, y + h);
                cr.stroke().ok();
                cr.move_to(x + w, y);
                cr.line_to(x, y + h);
                cr.stroke().ok();
            }
            WidgetType::FontString => {
                if let Some(ref txt) = text {
                    cr.set_source_rgba(
                        text_color.r as f64,
                        text_color.g as f64,
                        text_color.b as f64,
                        text_color.a as f64 * alpha as f64,
                    );
                    cr.select_font_face("Sans", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Normal);
                    cr.set_font_size(12.0);

                    let extents = cr.text_extents(txt).unwrap();
                    let text_x = x + (w - extents.width()) / 2.0;
                    let text_y = y + (h + extents.height()) / 2.0;
                    cr.move_to(text_x, text_y);
                    cr.show_text(txt).ok();
                }
            }
        }
    }

    // Draw center crosshair
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.3);
    cr.set_line_width(1.0);
    cr.move_to(cx - 20.0, cy);
    cr.line_to(cx + 20.0, cy);
    cr.stroke().ok();
    cr.move_to(cx, cy - 20.0);
    cr.line_to(cx, cy + 20.0);
    cr.stroke().ok();
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

    let width = frame.width;
    let height = frame.height;

    // If no anchors, default to center of parent
    if frame.anchors.is_empty() {
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

        return LayoutRect {
            x: parent_rect.x + (parent_rect.width - width) / 2.0,
            y: parent_rect.y + (parent_rect.height - height) / 2.0,
            width,
            height,
        };
    }

    let anchor = &frame.anchors[0];

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

/// Check if a frame is effectively visible (itself and all ancestors are visible).
#[allow(dead_code)]
fn is_effectively_visible(registry: &crate::widget::WidgetRegistry, id: u64) -> bool {
    let mut current_id = Some(id);
    while let Some(cid) = current_id {
        match registry.get(cid) {
            Some(frame) => {
                if !frame.visible {
                    return false;
                }
                current_id = frame.parent_id;
            }
            None => break,
        }
    }
    true
}
