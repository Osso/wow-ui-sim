//! Debug server, Lua REPL, and inspector update handlers.

use iced::{window, Task};
use iced_layout_inspector::server::Command as DebugCommand;

use crate::lua_server::{LuaCommand, Response as LuaResponse};

use super::app::App;
use super::state::InspectorState;
use super::Message;

impl App {
    /// Drain both IPC channels (debug inspector + Lua REPL).
    pub(crate) fn process_ipc(&mut self) -> Task<Message> {
        let commands: Vec<_> = if let Some(ref mut rx) = self.debug_rx {
            let mut cmds = Vec::new();
            while let Ok(cmd) = rx.try_recv() {
                cmds.push(cmd);
            }
            cmds
        } else {
            Vec::new()
        };

        let mut tasks = Vec::new();
        for cmd in commands {
            if let Some(task) = self.handle_debug_command(cmd) {
                tasks.push(task);
            }
        }

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

        // Install print interceptor that captures output to a Lua table.
        // Blizzard_PrintHandler overwrites the Rust `print` during addon load,
        // so console_output is never populated. This wrapper captures print
        // calls at the Lua level regardless of which print is active.
        let _ = env.exec(r##"
            __repl_prev_print = print
            __repl_captured = {}
            print = function(...)
                __repl_prev_print(...)
                local parts = {}
                for i = 1, select("#", ...) do
                    parts[#parts + 1] = tostring(select(i, ...))
                end
                __repl_captured[#__repl_captured + 1] = table.concat(parts, "\t")
            end
        "##);

        let result = env.exec(code);

        // Restore original print
        let _ = env.exec("print = __repl_prev_print");

        match result {
            Ok(()) => {
                // Read captured print output from Lua table
                let captured: String = env
                    .eval(r#"return table.concat(__repl_captured or {}, "\n")"#)
                    .unwrap_or_default();

                let mut state = env.state().borrow_mut();
                let console = state.console_output.join("\n");
                state.console_output.clear();

                // Combine console output and captured print output
                let output = match (console.is_empty(), captured.is_empty()) {
                    (true, true) => String::new(),
                    (false, true) => console,
                    (true, false) => captured,
                    (false, false) => format!("{}\n{}", console, captured),
                };
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
                    self.invalidate_layout();
                }
                LuaCommand::DumpTree {
                    filter,
                    visible_only,
                    respond,
                } => {
                    let tree = self.build_frame_tree_dump(filter.as_deref(), visible_only);
                    let _ = respond.send(LuaResponse::Tree(tree));
                }
                LuaCommand::Screenshot {
                    output,
                    width,
                    height,
                    filter,
                    crop,
                    respond,
                } => {
                    let result = self.render_screenshot(&output, width, height, filter.as_deref(), crop.as_deref());
                    let _ = respond.send(result);
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
            DebugCommand::Key { key, respond } => {
                self.handle_key_press(&key, None);
                let _ = respond.send(Ok(()));
                None
            }
            DebugCommand::Screenshot { respond } => {
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
