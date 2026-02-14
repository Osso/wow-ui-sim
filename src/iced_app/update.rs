//! App::update() method and related logic.

use iced::Task;
use iced_layout_inspector::server::ScreenshotData;

use crate::lua_api::WowLuaEnv;

use super::app::App;
use super::state::CanvasMessage;
use super::Message;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        // Always drain IPC so commands from the inspector/REPL are processed
        // even when idle (no timer subscription active).
        let ipc_task = self.process_ipc();

        let task = match message {
            Message::FireEvent(event) => { self.handle_fire_event(&event); Task::none() }
            Message::CanvasEvent(canvas_msg) => self.handle_canvas_event(canvas_msg),
            Message::Scroll(dx, dy) => { self.handle_scroll(dx, dy); Task::none() }
            Message::ReloadUI => { self.handle_reload_ui(); Task::none() }
            Message::CommandInputChanged(input) => { self.command_input = input; Task::none() }
            Message::ExecuteCommand => { self.handle_execute_command(); Task::none() }
            Message::ProcessTimers => self.handle_process_timers(),
            Message::ScreenshotTaken(screenshot) => { self.handle_screenshot_taken(screenshot); Task::none() }
            Message::FpsTick => Task::none(),
            Message::InspectorClose => { self.handle_inspector_close(); Task::none() }
            Message::InspectorWidthChanged(val) => { self.inspector_state.width = val; Task::none() }
            Message::InspectorHeightChanged(val) => { self.inspector_state.height = val; Task::none() }
            Message::InspectorAlphaChanged(val) => { self.inspector_state.alpha = val; Task::none() }
            Message::InspectorLevelChanged(val) => { self.inspector_state.frame_level = val; Task::none() }
            Message::InspectorVisibleToggled(val) => { self.inspector_state.visible = val; Task::none() }
            Message::InspectorMouseEnabledToggled(val) => { self.inspector_state.mouse_enabled = val; Task::none() }
            Message::InspectorApply => { self.handle_inspector_apply(); Task::none() }
            Message::ToggleFramesPanel => { self.frames_panel_collapsed = !self.frames_panel_collapsed; Task::none() }
            Message::XpLevelChanged(ref label) => { self.handle_xp_level_changed(label); Task::none() }
            Message::KeyPress(ref key, ref text) => {
                if key == "ESCAPE" && self.options_modal_visible {
                    self.options_modal_visible = false;
                } else {
                    self.handle_key_press(key, text.as_deref());
                }
                Task::none()
            }
            Message::PlayerClassChanged(ref name) => { self.handle_player_class_changed(name); Task::none() }
            Message::PlayerRaceChanged(ref name) => { self.handle_player_race_changed(name); Task::none() }
            Message::RotDamageLevelChanged(ref label) => { self.handle_rot_damage_level_changed(label); Task::none() }
            Message::ToggleOptionsModal => { self.options_modal_visible = !self.options_modal_visible; Task::none() }
            Message::CloseOptionsModal => { self.options_modal_visible = false; Task::none() }
        };

        Task::batch([task, ipc_task])
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
            CanvasMessage::RightMouseDown(pos) => self.handle_right_mouse_down(pos),
            CanvasMessage::RightMouseUp(pos) => self.handle_right_mouse_up(pos),
            CanvasMessage::MiddleClick(pos) => self.handle_middle_click(pos),
        }
        Task::none()
    }

    pub(super) fn handle_key_press(&mut self, key: &str, text: Option<&str>) {
        let env = self.env.borrow();
        if let Err(e) = env.send_key_press(key, text) {
            self.log_messages
                .push(format!("KeyPress({}) error: {}", key, e));
        }
        drop(env);
        self.invalidate();
    }

    fn handle_xp_level_changed(&mut self, label: &str) {
        use crate::lua_api::state::XP_LEVELS;
        self.selected_xp_level = label.to_string();
        let fraction = XP_LEVELS.iter()
            .find(|(l, _)| *l == label)
            .map(|(_, f)| *f)
            .unwrap_or(0.0);
        let at_max = fraction == 0.0;
        let event = if at_max { "DISABLE_XP_GAIN" } else { "ENABLE_XP_GAIN" };
        {
            let env = self.env.borrow();
            let xp_max = 89_750i32;
            let xp_current = (xp_max as f64 * fraction) as i32;
            let lua_code = format!(
                "IsPlayerAtEffectiveMaxLevel = function() return {} end; \
                 UnitXP = function() return {} end; \
                 UnitXPMax = function() return {} end",
                at_max, xp_current, xp_max
            );
            if let Err(e) = env.exec(&lua_code) {
                self.log_messages.push(format!("XP level error: {}", e));
            }
            if let Err(e) = env.fire_event(event) {
                self.log_messages.push(format!("XP event error: {}", e));
            }
        }
        self.save_config();
        self.invalidate();
    }

    fn handle_player_class_changed(&mut self, class_name: &str) {
        use crate::lua_api::state::CLASS_LABELS;
        let index = CLASS_LABELS.iter().position(|&n| n == class_name)
            .map(|i| (i + 1) as i32)
            .unwrap_or(1);
        self.selected_class = class_name.to_string();
        {
            let env = self.env.borrow();
            env.state().borrow_mut().player_class_index = index;
            self.fire_portrait_update(&env);
        }
        self.save_config();
        self.invalidate();
    }

    fn handle_player_race_changed(&mut self, race_name: &str) {
        use crate::lua_api::state::RACE_DATA;
        let index = RACE_DATA.iter().position(|(name, _, _)| *name == race_name)
            .unwrap_or(0);
        self.selected_race = race_name.to_string();
        {
            let env = self.env.borrow();
            env.state().borrow_mut().player_race_index = index;
            self.fire_portrait_update(&env);
        }
        self.save_config();
        self.invalidate();
    }

    fn handle_rot_damage_level_changed(&mut self, label: &str) {
        use crate::lua_api::state::ROT_DAMAGE_LEVELS;
        let index = ROT_DAMAGE_LEVELS.iter().position(|(l, _)| *l == label)
            .unwrap_or(0);
        self.selected_rot_level = label.to_string();
        self.env.borrow().state().borrow_mut().rot_damage_level = index;
        self.save_config();
    }

    /// Fire UNIT_PORTRAIT_UPDATE + PLAYER_ENTERING_WORLD to refresh unit frames.
    fn fire_portrait_update(&self, env: &WowLuaEnv) {
        let _ = env.fire_event_with_args(
            "UNIT_PORTRAIT_UPDATE",
            &[mlua::Value::String(env.lua().create_string("player").unwrap())],
        );
        let _ = env.fire_event_with_args(
            "PLAYER_ENTERING_WORLD",
            &[mlua::Value::Boolean(false), mlua::Value::Boolean(false)],
        );
    }

    fn handle_reload_ui(&mut self) {
        self.log_messages.push("Reloading UI...".to_string());
        {
            let env = self.env.borrow();
            if let Ok(s) = env.lua().create_string("WoWUISim") {
                let _ = env.fire_event_with_args("ADDON_LOADED", &[mlua::Value::String(s)]);
            }
            let _ = env.fire_event("VARIABLES_LOADED");
            let _ = env.fire_event_with_args(
                "PLAYER_ENTERING_WORLD",
                &[mlua::Value::Boolean(false), mlua::Value::Boolean(true)],
            );
            let _ = env.fire_event("UPDATE_BINDINGS");
            let _ = env.fire_event("DISPLAY_SIZE_CHANGED");
            let _ = env.fire_event("UI_SCALE_CHANGED");
        }
        self.drain_console();
        self.log_messages.push("UI reloaded.".to_string());
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
        self.run_pending_exec_lua();

        // Track timer dirty separately — timer callbacks can legitimately change widgets.
        self.env.borrow().state().borrow().widgets.take_render_dirty();
        self.run_wow_timers();
        let timers_dirty = self.env.borrow().state().borrow().widgets.take_render_dirty();

        // Resolve pending layout before OnUpdate so IsRectValid() returns true
        // for frames whose rects have been computed.  Without this, Blizzard code
        // that iterates buttonsWithDirtyEdges during OnUpdate can trigger
        // MarkEdgesDirty mid-iteration (via arrow-edge UpdatePosition calling
        // IsRectValid), corrupting the table and producing "invalid key to next".
        self.env.borrow().state().borrow_mut().ensure_layout_rects();

        self.fire_on_update();
        let on_update_dirty = self.env.borrow().state().borrow().widgets.take_render_dirty();

        self.tick_party_health();
        self.tick_casting();

        let health_dirty = self.env.borrow().state().borrow().widgets.take_render_dirty();
        if timers_dirty || on_update_dirty || health_dirty {
            static CNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let c = CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if c % 60 == 0 {
                eprintln!("[idle-debug] invalidate #{c}: timers={timers_dirty} on_update={on_update_dirty} health={health_dirty}");
            }
            self.invalidate();
        } else {
            self.drain_console();
        }

        Task::none()
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
            let env = self.env.borrow();
            env.state().borrow_mut().fps = self.fps;
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
            eprintln!("[OnUpdate] error: {}", e);
        }
    }


    fn tick_party_health(&mut self) {
        if self.selected_rot_level == "Off" {
            return;
        }
        let now = std::time::Instant::now();
        if now.duration_since(self.last_party_health_tick) < std::time::Duration::from_secs(2) {
            return;
        }
        self.last_party_health_tick = now;
        let env = self.env.borrow();
        let changed = {
            let mut state = env.state().borrow_mut();
            let (_, pct) = crate::lua_api::state::ROT_DAMAGE_LEVELS
                .get(state.rot_damage_level)
                .copied()
                .unwrap_or(("Light (1%)", 0.01));
            crate::lua_api::tick_party_health(&mut state.party_members, pct)
        };
        for idx in changed {
            let unit_id = format!("party{idx}");
            let _ = env.fire_event_with_args(
                "UNIT_HEALTH",
                &[mlua::Value::String(env.lua().create_string(&unit_id).unwrap())],
            );
        }
    }

    fn tick_casting(&mut self) {
        let env = self.env.borrow();
        let completed = extract_completed_cast(env.state());
        if let Some((cast_id, spell_id)) = completed {
            fire_cast_complete_events(&env, cast_id, spell_id);
            apply_heal_effect(env.state(), &env, spell_id);
        }
    }

    fn run_pending_exec_lua(&mut self) {
        if let Some(code) = self.pending_exec_lua.take() {
            eprintln!("[exec-lua] Running: {}", code);
            let env = self.env.borrow();
            if let Err(e) = env.exec(&code) {
                eprintln!("[exec-lua] Error: {}", e);
            }
            drop(env);
            self.invalidate();
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
            self.quads_dirty.set(true);
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    /// Save current UI settings to config file.
    fn save_config(&self) {
        let mut config = crate::config::SimConfig::load();
        config.player_class = self.selected_class.clone();
        config.player_race = self.selected_race.clone();
        config.rot_damage_level = self.selected_rot_level.clone();
        config.xp_level = self.selected_xp_level.clone();
        config.save();
    }

    /// Drain console, clear frame cache, and mark quads dirty.
    pub(super) fn invalidate(&mut self) {
        self.drain_console();
        self.quads_dirty.set(true);
    }

    /// Apply pending HitGrid changes from `set_frame_visible` calls.
    ///
    /// Walks the subtree of each changed root and inserts/removes hittable
    /// frames from the grid. Called after Lua handlers fire.
    pub(super) fn apply_hit_grid_changes(&self) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        let changes = std::mem::take(&mut state.pending_hit_grid_changes);
        if changes.is_empty() {
            return;
        }
        drop(state);

        let mut grid_ref = self.cached_hittable.borrow_mut();
        let Some(grid) = grid_ref.as_mut() else { return };

        let state = env.state().borrow();
        let registry = &state.widgets;

        for (root_id, became_visible) in changes {
            // Walk subtree and update each frame in the grid.
            let mut stack = vec![root_id];
            while let Some(id) = stack.pop() {
                let Some(f) = registry.get(id) else { continue };
                if became_visible {
                    if f.visible && f.effective_alpha > 0.0 && f.mouse_enabled
                        && !f.name.as_deref().is_some_and(|n| {
                            super::frame_collect::HIT_TEST_EXCLUDED.contains(&n)
                        })
                    {
                        if let Some(rect) = f.layout_rect {
                            let (il, ir, it, ib) = f.hit_rect_insets;
                            let scaled = iced::Rectangle::new(
                                iced::Point::new(
                                    (rect.x + il) * crate::render::texture::UI_SCALE,
                                    (rect.y + it) * crate::render::texture::UI_SCALE,
                                ),
                                iced::Size::new(
                                    (rect.width - il - ir).max(0.0) * crate::render::texture::UI_SCALE,
                                    (rect.height - it - ib).max(0.0) * crate::render::texture::UI_SCALE,
                                ),
                            );
                            grid.insert(id, scaled);
                        }
                    }
                } else {
                    grid.remove(id);
                }
                stack.extend_from_slice(&f.children);
            }
        }
    }

    /// Check whether a frame's `__enabled` attribute is true (default: true).
    pub(super) fn is_frame_enabled(&self, frame_id: u64) -> bool {
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

    /// Focus an EditBox on click, or clear focus when clicking elsewhere.
    pub(super) fn update_editbox_focus(&self, clicked_frame: Option<u64>) {
        let env = self.env.borrow();
        let is_editbox = clicked_frame.is_some_and(|fid| {
            env.state()
                .borrow()
                .widgets
                .get(fid)
                .map(|f| f.widget_type == crate::widget::WidgetType::EditBox)
                .unwrap_or(false)
        });
        let old_focus = env.state().borrow().focused_frame_id;

        if is_editbox {
            let fid = clicked_frame.unwrap();
            if old_focus != Some(fid) {
                // Focus the clicked EditBox via Lua SetFocus logic
                {
                    let mut state = env.state().borrow_mut();
                    state.focused_frame_id = Some(fid);
                }
                if let Some(old_id) = old_focus {
                    let _ = env.fire_script_handler(old_id, "OnEditFocusLost", vec![]);
                }
                let _ = env.fire_script_handler(fid, "OnEditFocusGained", vec![]);
            }
        } else if let Some(old_id) = old_focus {
            // Clicked on non-EditBox: clear focus
            {
                let mut state = env.state().borrow_mut();
                state.focused_frame_id = None;
            }
            let _ = env.fire_script_handler(old_id, "OnEditFocusLost", vec![]);
        }
    }

    /// Toggle CheckButton checked state before OnClick (WoW behavior).
    /// Skip action bar buttons — they manage checked state via UpdateState().
    pub(super) fn toggle_checkbutton_if_needed(&self, frame_id: u64, env: &WowLuaEnv) {
        let mut state = env.state().borrow_mut();
        let is_checkbutton = state
            .widgets
            .get(frame_id)
            .map(|f| f.widget_type == crate::widget::WidgetType::CheckButton)
            .unwrap_or(false);
        if !is_checkbutton {
            return;
        }
        // Action bar buttons registered via SetActionUIButton manage their own
        // checked state through UpdateState() — don't auto-toggle them.
        let is_action_button = state.action_ui_buttons.iter().any(|(id, _)| *id == frame_id);
        if is_action_button {
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
        let tex_id = state.widgets.get(frame_id)
            .and_then(|f| f.children_keys.get("CheckedTexture").copied());
        if let Some(tex_id) = tex_id {
            state.set_frame_visible(tex_id, new_checked);
        }
    }

    /// Sync the iced canvas size to SimState and UIParent/WorldFrame dimensions.
    /// Called from the render path when the window is resized by the window manager.
    pub(crate) fn sync_screen_size_to_state(&self, size: iced::Size) {
        let env = self.env.borrow();
        let state = env.state().borrow();
        if (state.screen_width - size.width).abs() > 0.5
            || (state.screen_height - size.height).abs() > 0.5
        {
            println!("Window size: {}x{} (was {}x{})",
                size.width as i32, size.height as i32,
                state.screen_width as i32, state.screen_height as i32);
            drop(state);
            env.set_screen_size(size.width, size.height);
        }
    }

    pub(crate) fn drain_console(&mut self) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        self.log_messages.append(&mut state.console_output);
    }

}

/// Check if a cast has completed and extract its info, clearing state.
fn extract_completed_cast(
    state: &std::rc::Rc<std::cell::RefCell<crate::lua_api::SimState>>,
) -> Option<(u32, u32)> {
    let mut s = state.borrow_mut();
    let c = s.casting.as_ref()?;
    let now = s.start_time.elapsed().as_secs_f64();
    if now < c.end_time {
        return None;
    }
    let cast_id = c.cast_id;
    let spell_id = c.spell_id;
    s.casting = None;
    Some((cast_id, spell_id))
}

/// Fire UNIT_SPELLCAST_STOP and UNIT_SPELLCAST_SUCCEEDED events.
fn fire_cast_complete_events(
    env: &crate::lua_api::WowLuaEnv,
    cast_id: u32,
    spell_id: u32,
) {
    let lua = env.lua();
    let Ok(player) = lua.create_string("player") else { return };
    let args = &[
        mlua::Value::String(player.clone()),
        mlua::Value::Integer(cast_id as i64),
        mlua::Value::Integer(spell_id as i64),
    ];
    let _ = env.fire_event_with_args("UNIT_SPELLCAST_STOP", args);
    let _ = env.fire_event_with_args("UNIT_SPELLCAST_SUCCEEDED", args);
    // Push state update to registered action buttons (casting is now None).
    let _ = crate::lua_api::globals::action_bar_api::push_action_button_state_update(
        &env.state(), env.lua(),
    );
}

/// Apply healing from a completed cast spell to the target or self.
fn apply_heal_effect(
    state: &std::rc::Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    env: &crate::lua_api::WowLuaEnv,
    spell_id: u32,
) {
    const HEAL_AMOUNT: i32 = 20_000;
    // Only healing spells apply an effect
    let is_heal = matches!(spell_id, 19750 | 82326 | 85673);
    if !is_heal {
        return;
    }
    let unit_event = {
        let mut s = state.borrow_mut();
        if let Some(ref mut t) = s.current_target {
            if !t.is_enemy {
                t.health = (t.health + HEAL_AMOUNT).min(t.health_max);
                Some(t.unit_id.clone())
            } else {
                // Heal self when targeting enemy
                s.player_health = (s.player_health + HEAL_AMOUNT).min(s.player_health_max);
                Some("player".to_string())
            }
        } else {
            s.player_health = (s.player_health + HEAL_AMOUNT).min(s.player_health_max);
            Some("player".to_string())
        }
    };
    if let Some(unit) = unit_event {
        let lua = env.lua();
        if let Ok(unit_str) = lua.create_string(&unit) {
            let _ = env.fire_event_with_args(
                "UNIT_HEALTH",
                &[mlua::Value::String(unit_str)],
            );
        }
    }
}

