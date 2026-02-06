//! App::update() method and related logic.

use iced::{window, Task};
use iced_layout_inspector::server::{Command as DebugCommand, ScreenshotData};

use crate::lua_server::{LuaCommand, Response as LuaResponse};

use super::app::App;
use super::state::{CanvasMessage, InspectorState};
use super::Message;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
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
                        let errors = {
                            let env = self.env.borrow();
                            let mut errs = Vec::new();
                            if let Some(old_id) = self.hovered_frame {
                                if let Err(e) = env.fire_script_handler(old_id, "OnLeave", vec![]) {
                                    errs.push(format_script_error(&env, old_id, "OnLeave", &e));
                                }
                            }
                            if let Some(new_id) = new_hovered {
                                if let Err(e) = env.fire_script_handler(new_id, "OnEnter", vec![]) {
                                    errs.push(format_script_error(&env, new_id, "OnEnter", &e));
                                }
                            }
                            errs
                        };
                        for msg in errors {
                            eprintln!("{}", msg);
                            self.log_messages.push(msg);
                        }
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
                        let error = {
                            let env = self.env.borrow();
                            let button_val =
                                mlua::Value::String(env.lua().create_string("LeftButton").unwrap());
                            match env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val]) {
                                Err(e) => Some(format_script_error(&env, frame_id, "OnMouseDown", &e)),
                                Ok(_) => None,
                            }
                        };
                        if let Some(msg) = error {
                            eprintln!("{}", msg);
                            self.log_messages.push(msg);
                        }
                        self.drain_console();
                        self.frame_cache.clear();
                        self.quads_dirty.set(true);
                    }
                }
                CanvasMessage::MouseUp(pos) => {
                    let released_on = self.hit_test(pos);
                    if let Some(frame_id) = released_on {
                        let errors = {
                            let env = self.env.borrow();
                            let mut errs = Vec::new();
                            let button_val =
                                mlua::Value::String(env.lua().create_string("LeftButton").unwrap());

                            if self.mouse_down_frame == Some(frame_id) {
                                // CheckButton auto-toggles checked state before OnClick
                                {
                                    let mut state = env.state().borrow_mut();
                                    let is_checkbutton = state.widgets.get(frame_id)
                                        .map(|f| f.widget_type == crate::widget::WidgetType::CheckButton)
                                        .unwrap_or(false);
                                    if is_checkbutton {
                                        let old_checked = state.widgets.get(frame_id)
                                            .and_then(|f| f.attributes.get("__checked"))
                                            .and_then(|v| if let crate::widget::AttributeValue::Boolean(b) = v { Some(*b) } else { None })
                                            .unwrap_or(false);
                                        let new_checked = !old_checked;
                                        if let Some(frame) = state.widgets.get_mut(frame_id) {
                                            frame.attributes.insert("__checked".to_string(), crate::widget::AttributeValue::Boolean(new_checked));
                                        }
                                        if let Some(frame) = state.widgets.get(frame_id) {
                                            if let Some(&tex_id) = frame.children_keys.get("CheckedTexture") {
                                                if let Some(tex) = state.widgets.get_mut(tex_id) {
                                                    tex.visible = new_checked;
                                                }
                                            }
                                        }
                                    }
                                }

                                let down_val = mlua::Value::Boolean(false);
                                if let Err(e) = env.fire_script_handler(
                                    frame_id,
                                    "OnClick",
                                    vec![button_val.clone(), down_val],
                                ) {
                                    errs.push(format_script_error(&env, frame_id, "OnClick", &e));
                                }
                            }

                            if let Err(e) = env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val]) {
                                errs.push(format_script_error(&env, frame_id, "OnMouseUp", &e));
                            }
                            errs
                        };
                        for msg in errors {
                            eprintln!("{}", msg);
                            self.log_messages.push(msg);
                        }
                        self.drain_console();
                        self.frame_cache.clear();
                        self.quads_dirty.set(true);
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
                        self.inspector_position = iced::Point::new(pos.x + 10.0, pos.y + 10.0);
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
                {
                    let env = self.env.borrow();
                    match env.process_timers() {
                        Err(e) => eprintln!("Timer error: {}", e),
                        _ => {}
                    }
                }

                // Fire OnUpdate handlers
                let on_update_elapsed = now.duration_since(self.last_on_update_time);
                self.last_on_update_time = now;
                {
                    let env = self.env.borrow();
                    if let Err(e) = env.fire_on_update(on_update_elapsed.as_secs_f64()) {
                        eprintln!("OnUpdate error: {}", e);
                    }
                }

                self.drain_console();
                self.frame_cache.clear();
                self.quads_dirty.set(true);

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
                    self.quads_dirty.set(true);
                }
            }
            Message::ToggleFramesPanel => {
                self.frames_panel_collapsed = !self.frames_panel_collapsed;
            }
        }

        Task::none()
    }

    pub(crate) fn drain_console(&mut self) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        self.log_messages.append(&mut state.console_output);
    }

    pub(crate) fn process_debug_commands(&mut self) -> Task<Message> {
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

    pub(crate) fn process_lua_commands(&mut self) {
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

    pub(crate) fn handle_debug_command(&mut self, cmd: DebugCommand) -> Option<Task<Message>> {
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

    /// Populate inspector state from a frame's properties.
    pub(crate) fn populate_inspector(&mut self, frame_id: u64) {
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
    pub(crate) fn apply_inspector_changes(&mut self, frame_id: u64) {
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
}

/// Format a script handler error with frame context.
fn format_script_error(
    env: &crate::lua_api::WowLuaEnv,
    frame_id: u64,
    handler: &str,
    error: &crate::Error,
) -> String {
    let state = env.state().borrow();
    let name = state
        .widgets
        .get(frame_id)
        .and_then(|f| f.name.as_deref())
        .unwrap_or("(anon)");
    format!("[{}] {} error: {}", name, handler, error)
}
