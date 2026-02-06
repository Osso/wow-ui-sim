//! App::update() method and related logic.

use iced::{window, Point, Task};
use iced_layout_inspector::server::{Command as DebugCommand, ScreenshotData};

use crate::lua_api::WowLuaEnv;
use crate::lua_server::{LuaCommand, Response as LuaResponse};

use super::app::App;
use super::state::{CanvasMessage, InspectorState};
use super::Message;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FireEvent(event) => self.handle_fire_event(&event),
            Message::CanvasEvent(canvas_msg) => return self.handle_canvas_event(canvas_msg),
            Message::Scroll(dx, dy) => self.handle_scroll(dx, dy),
            Message::ReloadUI => self.handle_reload_ui(),
            Message::CommandInputChanged(input) => self.command_input = input,
            Message::ExecuteCommand => self.handle_execute_command(),
            Message::ProcessTimers => return self.handle_process_timers(),
            Message::ScreenshotTaken(screenshot) => self.handle_screenshot_taken(screenshot),
            Message::FpsTick => {}
            Message::InspectorClose => self.handle_inspector_close(),
            Message::InspectorWidthChanged(val) => self.inspector_state.width = val,
            Message::InspectorHeightChanged(val) => self.inspector_state.height = val,
            Message::InspectorAlphaChanged(val) => self.inspector_state.alpha = val,
            Message::InspectorLevelChanged(val) => self.inspector_state.frame_level = val,
            Message::InspectorVisibleToggled(val) => self.inspector_state.visible = val,
            Message::InspectorMouseEnabledToggled(val) => self.inspector_state.mouse_enabled = val,
            Message::InspectorApply => self.handle_inspector_apply(),
            Message::ToggleFramesPanel => self.frames_panel_collapsed = !self.frames_panel_collapsed,
        }

        Task::none()
    }

    // ── Event handlers ──────────────────────────────────────────────────

    fn handle_fire_event(&mut self, event: &str) {
        {
            let env = self.env.borrow();
            if let Err(e) = env.fire_event(event) {
                self.log_messages.push(format!("Event error: {}", e));
            } else {
                self.log_messages.push(format!("Fired: {}", event));
            }
        }
        self.invalidate();
    }

    fn handle_canvas_event(&mut self, canvas_msg: CanvasMessage) -> Task<Message> {
        match canvas_msg {
            CanvasMessage::MouseMove(pos) => self.handle_mouse_move(pos),
            CanvasMessage::MouseDown(pos) => self.handle_mouse_down(pos),
            CanvasMessage::MouseUp(pos) => self.handle_mouse_up(pos),
            CanvasMessage::MiddleClick(pos) => self.handle_middle_click(pos),
        }
        Task::none()
    }

    fn handle_mouse_move(&mut self, pos: Point) {
        self.mouse_position = Some(pos);
        let new_hovered = self.hit_test(pos);
        if new_hovered == self.hovered_frame {
            return;
        }

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
        self.push_errors(errors);
        self.hovered_frame = new_hovered;
        self.invalidate();
    }

    fn handle_mouse_down(&mut self, pos: Point) {
        let Some(frame_id) = self.hit_test(pos) else {
            return;
        };

        if !self.is_frame_enabled(frame_id) {
            return;
        }

        self.mouse_down_frame = Some(frame_id);
        self.pressed_frame = Some(frame_id);

        let error = {
            let env = self.env.borrow();
            let button_val = mlua::Value::String(env.lua().create_string("LeftButton").unwrap());
            env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val])
                .err()
                .map(|e| format_script_error(&env, frame_id, "OnMouseDown", &e))
        };
        if let Some(msg) = error {
            self.push_errors(vec![msg]);
        }
        self.invalidate();
    }

    fn handle_mouse_up(&mut self, pos: Point) {
        let released_on = self.hit_test(pos);
        if let Some(frame_id) = released_on {
            let errors = {
                let env = self.env.borrow();
                let mut errs = Vec::new();
                let button_val =
                    mlua::Value::String(env.lua().create_string("LeftButton").unwrap());

                if self.mouse_down_frame == Some(frame_id) {
                    self.toggle_checkbutton_if_needed(frame_id, &env);

                    let down_val = mlua::Value::Boolean(false);
                    if let Err(e) = env.fire_script_handler(
                        frame_id,
                        "OnClick",
                        vec![button_val.clone(), down_val],
                    ) {
                        errs.push(format_script_error(&env, frame_id, "OnClick", &e));
                    }
                }

                if let Err(e) =
                    env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val])
                {
                    errs.push(format_script_error(&env, frame_id, "OnMouseUp", &e));
                }
                errs
            };
            self.push_errors(errors);
            self.invalidate();
        }
        self.mouse_down_frame = None;
        self.pressed_frame = None;
    }

    fn handle_middle_click(&mut self, pos: Point) {
        if let Some(frame_id) = self.hit_test(pos) {
            self.populate_inspector(frame_id);
            self.inspected_frame = Some(frame_id);
            self.inspector_visible = true;
            self.inspector_position = Point::new(pos.x + 10.0, pos.y + 10.0);
        }
    }

    fn handle_scroll(&mut self, _dx: f32, dy: f32) {
        if self.fire_mouse_wheel(dy) {
            self.invalidate();
        } else {
            let scroll_speed = 30.0;
            self.scroll_offset -= dy * scroll_speed;
            let max_scroll = 2600.0;
            self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
            self.frame_cache.clear();
            self.quads_dirty.set(true);
        }
    }

    /// Propagate OnMouseWheel up the parent chain. Returns true if handled.
    fn fire_mouse_wheel(&mut self, dy: f32) -> bool {
        let pos = match self.mouse_position {
            Some(p) => p,
            None => return false,
        };
        let start_frame = match self.hit_test(pos) {
            Some(f) => f,
            None => return false,
        };

        let env = self.env.borrow();
        let mut current = Some(start_frame);
        while let Some(frame_id) = current {
            if env.has_script_handler(frame_id, "OnMouseWheel") {
                let delta_val = mlua::Value::Number(dy as f64);
                if let Err(e) =
                    env.fire_script_handler(frame_id, "OnMouseWheel", vec![delta_val])
                {
                    let msg = format_script_error(&env, frame_id, "OnMouseWheel", &e);
                    eprintln!("{}", msg);
                    self.log_messages.push(msg);
                }
                return true;
            }
            current = env
                .state()
                .borrow()
                .widgets
                .get(frame_id)
                .and_then(|f| f.parent_id);
        }
        false
    }

    fn handle_reload_ui(&mut self) {
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

    fn handle_execute_command(&mut self) {
        let cmd = self.command_input.clone();
        if cmd.is_empty() {
            return;
        }

        self.log_messages.push(format!("> {}", cmd));
        self.execute_command_inner(&cmd);
        self.drain_console();
        self.command_input.clear();
        self.frame_cache.clear();
        self.quads_dirty.set(true);
    }

    fn execute_command_inner(&mut self, cmd: &str) {
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
            match env.dispatch_slash_command(cmd) {
                Ok(true) => {}
                Ok(false) => {
                    self.log_messages.push(format!("Unknown command: {}", cmd));
                }
                Err(e) => {
                    self.log_messages.push(format!("Command error: {}", e));
                }
            }
        }
    }

    fn handle_process_timers(&mut self) -> Task<Message> {
        self.update_fps_counter();
        self.run_wow_timers();
        self.fire_on_update();
        self.invalidate();
        self.process_debug_commands()
    }

    fn update_fps_counter(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.fps_last_time);
        if elapsed >= std::time::Duration::from_secs(1) {
            let frames = self.frame_count.get();
            self.fps = frames as f32 / elapsed.as_secs_f32();
            self.frame_time_display = self.frame_time_avg.get();
            self.frame_count.set(0);
            self.fps_last_time = now;
        }
    }

    fn run_wow_timers(&self) {
        let env = self.env.borrow();
        if let Err(e) = env.process_timers() {
            eprintln!("Timer error: {}", e);
        }
    }

    fn fire_on_update(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_on_update_time);
        self.last_on_update_time = now;
        let env = self.env.borrow();
        if let Err(e) = env.fire_on_update(elapsed.as_secs_f64()) {
            eprintln!("OnUpdate error: {}", e);
        }
    }

    fn handle_screenshot_taken(&mut self, screenshot: iced::window::screenshot::Screenshot) {
        if let Some(respond) = self.pending_screenshot.take() {
            let data = ScreenshotData {
                width: screenshot.size.width,
                height: screenshot.size.height,
                pixels: screenshot.rgba.to_vec(),
            };
            let _ = respond.send(Ok(data));
        }
    }

    fn handle_inspector_close(&mut self) {
        self.inspector_visible = false;
        self.inspected_frame = None;
    }

    fn handle_inspector_apply(&mut self) {
        if let Some(frame_id) = self.inspected_frame {
            self.apply_inspector_changes(frame_id);
            self.frame_cache.clear();
            self.quads_dirty.set(true);
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    /// Drain console, clear frame cache, and mark quads dirty.
    fn invalidate(&mut self) {
        self.drain_console();
        self.frame_cache.clear();
        self.quads_dirty.set(true);
    }

    /// Log errors to both stderr and the in-app log.
    fn push_errors(&mut self, errors: Vec<String>) {
        for msg in errors {
            eprintln!("{}", msg);
            self.log_messages.push(msg);
        }
    }

    /// Check whether a frame's `__enabled` attribute is true (default: true).
    fn is_frame_enabled(&self, frame_id: u64) -> bool {
        let env = self.env.borrow();
        let state = env.state().borrow();
        state
            .widgets
            .get(frame_id)
            .and_then(|f| f.attributes.get("__enabled"))
            .and_then(|v| {
                if let crate::widget::AttributeValue::Boolean(b) = v {
                    Some(*b)
                } else {
                    None
                }
            })
            .unwrap_or(true)
    }

    /// Toggle CheckButton checked state before OnClick (WoW behavior).
    fn toggle_checkbutton_if_needed(&self, frame_id: u64, env: &WowLuaEnv) {
        let mut state = env.state().borrow_mut();
        let is_checkbutton = state
            .widgets
            .get(frame_id)
            .map(|f| f.widget_type == crate::widget::WidgetType::CheckButton)
            .unwrap_or(false);
        if !is_checkbutton {
            return;
        }

        let old_checked = state
            .widgets
            .get(frame_id)
            .and_then(|f| f.attributes.get("__checked"))
            .and_then(|v| {
                if let crate::widget::AttributeValue::Boolean(b) = v {
                    Some(*b)
                } else {
                    None
                }
            })
            .unwrap_or(false);
        let new_checked = !old_checked;

        if let Some(frame) = state.widgets.get_mut(frame_id) {
            frame.attributes.insert(
                "__checked".to_string(),
                crate::widget::AttributeValue::Boolean(new_checked),
            );
        }
        if let Some(frame) = state.widgets.get(frame_id) {
            if let Some(&tex_id) = frame.children_keys.get("CheckedTexture") {
                if let Some(tex) = state.widgets.get_mut(tex_id) {
                    tex.visible = new_checked;
                }
            }
        }
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

    /// Execute Lua code from the REPL server and return the response.
    fn exec_lua_command(&self, code: &str) -> LuaResponse {
        let env = self.env.borrow();
        env.state().borrow_mut().console_output.clear();
        let result = env.exec(code);
        match result {
            Ok(()) => {
                let mut state = env.state().borrow_mut();
                let output = state.console_output.join("\n");
                state.console_output.clear();
                LuaResponse::Output(output)
            }
            Err(e) => LuaResponse::Error(e.to_string()),
        }
    }

    pub(crate) fn process_lua_commands(&mut self) {
        let commands: Vec<_> = self
            .lua_rx
            .as_ref()
            .map(|rx| std::iter::from_fn(|| rx.try_recv().ok()).collect())
            .unwrap_or_default();

        for cmd in commands {
            match cmd {
                LuaCommand::Exec { code, respond } => {
                    let response = self.exec_lua_command(&code);
                    let _ = respond.send(response);
                    self.drain_console();
                    self.frame_cache.clear();
                    self.quads_dirty.set(true);
                }
                LuaCommand::DumpTree {
                    filter,
                    visible_only,
                    respond,
                } => {
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
