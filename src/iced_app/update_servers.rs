//! Debug server, Lua REPL, and inspector update handlers.

use std::sync::mpsc;

#[cfg(not(target_os = "linux"))]
use crate::inspector_server_stub::Command as DebugCommand;
use iced::{Task, window};
#[cfg(target_os = "linux")]
use iced_layout_inspector::server::Command as DebugCommand;

use crate::lua_api::WowLuaEnv;
use crate::lua_server::{LuaCommand, Response as LuaResponse};

use super::Message;
use super::app::App;
use super::state::InspectorState;

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
        install_repl_print_capture(&env);

        let result = env.exec(code);

        restore_repl_print(&env);

        match result {
            Ok(()) => LuaResponse::Output(collect_lua_command_output(&env)),
            Err(e) => LuaResponse::Error(e.to_string()),
        }
    }

    pub(crate) fn process_lua_commands(&mut self) {
        let commands = drain_lua_commands(self.lua_rx.as_ref());

        for cmd in commands {
            self.handle_lua_command(cmd);
        }
    }

    fn handle_lua_command(&mut self, cmd: LuaCommand) {
        match cmd {
            LuaCommand::Exec { code, respond } => self.handle_lua_exec(code, respond),
            LuaCommand::DumpTree {
                filter,
                filter_key,
                visible_only,
                verbose,
                respond,
            } => self.handle_lua_dump_tree(filter, filter_key, visible_only, verbose, respond),
            LuaCommand::DumpQuads {
                filter,
                verbose,
                respond,
            } => self.handle_lua_dump_quads(filter, verbose, respond),
            LuaCommand::Screenshot {
                output,
                width,
                height,
                filter,
                crop,
                respond,
            } => self.handle_lua_screenshot(output, width, height, filter, crop, respond),
            LuaCommand::MouseMove { x, y, respond } => self.handle_lua_mouse_move(x, y, respond),
            LuaCommand::MouseClick { x, y, respond } => self.handle_lua_mouse_click(x, y, respond),
        }
    }

    fn handle_lua_exec(&mut self, code: String, respond: mpsc::Sender<LuaResponse>) {
        let response = self.exec_lua_command(&code);
        self.flush_post_script_updates();
        let _ = respond.send(response);
    }

    fn handle_lua_mouse_move(&mut self, x: f32, y: f32, respond: mpsc::Sender<LuaResponse>) {
        self.handle_mouse_move(iced::Point::new(x, y));
        let target = self.hovered_frame_name();
        let _ = respond.send(LuaResponse::Output(target));
    }

    fn handle_lua_mouse_click(&mut self, x: f32, y: f32, respond: mpsc::Sender<LuaResponse>) {
        let point = iced::Point::new(x, y);
        self.handle_mouse_move(point);
        self.handle_mouse_down(point);
        self.handle_mouse_up(point);
        let target = self.hovered_frame_name();
        let _ = respond.send(LuaResponse::Output(target));
    }

    fn hovered_frame_name(&self) -> String {
        let Some(id) = self.hovered_frame else {
            return "nil".to_string();
        };
        let env = self.env.borrow();
        let state = env.state().borrow();
        state
            .widgets
            .get(id)
            .and_then(|frame| frame.name.clone())
            .unwrap_or_else(|| format!("#{id}"))
    }

    fn handle_lua_dump_tree(
        &self,
        filter: Option<String>,
        filter_key: Option<String>,
        visible_only: bool,
        verbose: bool,
        respond: mpsc::Sender<LuaResponse>,
    ) {
        let tree = self.build_frame_tree_dump(
            filter.as_deref(),
            filter_key.as_deref(),
            visible_only,
            verbose,
        );
        let _ = respond.send(LuaResponse::Tree(tree));
    }

    fn handle_lua_dump_quads(
        &mut self,
        filter: Option<String>,
        verbose: bool,
        respond: mpsc::Sender<LuaResponse>,
    ) {
        let dump = self.build_cached_quad_dump(filter.as_deref(), verbose);
        let _ = respond.send(LuaResponse::Quads(dump));
    }

    fn handle_lua_screenshot(
        &mut self,
        output: String,
        width: u32,
        height: u32,
        filter: Option<String>,
        crop: Option<String>,
        respond: mpsc::Sender<LuaResponse>,
    ) {
        let result =
            self.render_screenshot(&output, width, height, filter.as_deref(), crop.as_deref());
        let _ = respond.send(result);
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
                let _ = self.handle_key_press(&key, None, std::time::Instant::now());
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
        if let Some(frame) = state.widgets.get_mut_visual(frame_id) {
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
            frame.set_mouse_enabled(self.inspector_state.mouse_enabled);
        }
    }
}

fn install_repl_print_capture(env: &WowLuaEnv) {
    // Blizzard_PrintHandler overwrites the Rust `print` during addon load,
    // so console_output is never populated. This wrapper captures print
    // calls at the Lua level regardless of which print is active.
    let _ = env.exec(
        r##"
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
    "##,
    );
}

fn restore_repl_print(env: &WowLuaEnv) {
    let _ = env.exec("print = __repl_prev_print");
}

fn collect_lua_command_output(env: &WowLuaEnv) -> String {
    let captured: String = env
        .eval(r#"return table.concat(__repl_captured or {}, "\n")"#)
        .unwrap_or_default();

    let mut state = env.state().borrow_mut();
    let console = state.console_output.join("\n");
    state.console_output.clear();
    combine_console_and_captured(console, captured)
}

fn combine_console_and_captured(console: String, captured: String) -> String {
    match (console.is_empty(), captured.is_empty()) {
        (true, true) => String::new(),
        (false, true) => console,
        (true, false) => captured,
        (false, false) => format!("{}\n{}", console, captured),
    }
}

fn drain_lua_commands(rx: Option<&mpsc::Receiver<LuaCommand>>) -> Vec<LuaCommand> {
    rx.map(|rx| std::iter::from_fn(|| rx.try_recv().ok()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{App, combine_console_and_captured, drain_lua_commands};
    use crate::iced_app::app::AppInit;
    use crate::lua_api::WowLuaEnv;
    use crate::lua_server::LuaCommand;
    use crate::render::{GlyphAtlas, WowFontSystem};
    use crate::screen::ScreenKind;
    use crate::texture::TextureManager;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::mpsc;

    fn build_test_app_with_lua_rx(lua_rx: mpsc::Receiver<LuaCommand>) -> App {
        let env = Rc::new(RefCell::new(
            WowLuaEnv::new().expect("failed to create Lua environment"),
        ));
        env.borrow().set_screen_mode(ScreenKind::Game);
        env.borrow().set_screen_size(800.0, 600.0);

        let (_cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
        App::build_app(AppInit {
            env,
            log_messages: Vec::new(),
            texture_manager: Rc::new(RefCell::new(TextureManager::new())),
            font_system: Rc::new(RefCell::new(WowFontSystem::new())),
            glyph_atlas: Rc::new(RefCell::new(GlyphAtlas::new())),
            cmd_rx,
            lua_rx,
            debug_borders: false,
            debug_anchors: false,
            saved_vars: None,
            config: crate::config::SimConfig::default(),
        })
    }

    #[test]
    fn combine_console_and_captured_prefers_newline_when_both_present() {
        assert_eq!(
            combine_console_and_captured("console".to_string(), "captured".to_string()),
            "console\ncaptured"
        );
        assert_eq!(
            combine_console_and_captured(String::new(), "captured".to_string()),
            "captured"
        );
        assert_eq!(
            combine_console_and_captured("console".to_string(), String::new()),
            "console"
        );
    }

    #[test]
    fn drain_lua_commands_collects_all_pending_messages() {
        let (tx, rx) = mpsc::channel();
        let (respond, _recv) = mpsc::channel();
        tx.send(LuaCommand::Exec {
            code: "print('one')".to_string(),
            respond: respond.clone(),
        })
        .unwrap();
        tx.send(LuaCommand::DumpTree {
            filter: Some("Foo".to_string()),
            filter_key: Some("Bar".to_string()),
            visible_only: true,
            verbose: false,
            respond,
        })
        .unwrap();

        let commands = drain_lua_commands(Some(&rx));

        assert_eq!(commands.len(), 2);
        assert!(matches!(commands[0], LuaCommand::Exec { .. }));
        assert!(matches!(commands[1], LuaCommand::DumpTree { .. }));
        assert!(drain_lua_commands(Some(&rx)).is_empty());
    }

    #[test]
    fn drain_lua_commands_preserves_dump_quads_messages() {
        let (tx, rx) = mpsc::channel();
        let (respond, _recv) = mpsc::channel();
        tx.send(LuaCommand::DumpQuads {
            filter: Some("uigroupmanager".to_string()),
            verbose: true,
            respond,
        })
        .unwrap();

        let commands = drain_lua_commands(Some(&rx));

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            &commands[0],
            LuaCommand::DumpQuads {
                filter: Some(filter),
                verbose: true,
                ..
            } if filter == "uigroupmanager"
        ));
    }

    #[test]
    fn lua_exec_settles_immediate_layout_on_update_before_next_command() {
        let (tx, rx) = mpsc::channel();
        let mut app = build_test_app_with_lua_rx(rx);
        app.last_on_update_time = std::time::Instant::now() - std::time::Duration::from_millis(20);
        let (respond, response_rx) = mpsc::channel();

        tx.send(LuaCommand::Exec {
            code: r#"
                __layout_tick_count = 0
                DeferredLayoutFrame = CreateFrame("Frame", "DeferredLayoutFrame", UIParent)
                DeferredLayoutFrame:Show()
                DeferredLayoutFrame:SetScript("OnUpdate", function(self)
                    __layout_tick_count = __layout_tick_count + 1
                    self:SetScript("OnUpdate", nil)
                end)
            "#
            .to_string(),
            respond,
        })
        .expect("Lua exec command should queue");

        app.process_lua_commands();
        let _ = response_rx
            .recv()
            .expect("Lua exec command should respond after settling updates");

        let ticks: f64 = app
            .env
            .borrow()
            .eval("return __layout_tick_count")
            .expect("layout tick count should be readable");
        assert_eq!(
            ticks, 1.0,
            "Lua IPC mutations should settle immediate layout OnUpdate work before the next command"
        );
    }
}
