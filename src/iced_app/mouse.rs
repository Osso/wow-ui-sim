//! Mouse event handlers for the iced application.

use iced::Point;
use rilua::{LuaApiMut, Val};

use super::app::App;
use super::mouse_drag::{
    active_motion_drag_frame, apply_sizing, find_drag_script_target, find_slider_drag_target,
    frame_motion_scripts_allowed, moving_drag_anchor_update, reanchor_moving_drag_frame,
    sizing_drag_update,
};

/// Minimum distance (in pixels) the mouse must move while held to start a drag.
const DRAG_THRESHOLD: f32 = 5.0;

impl App {
    pub(super) fn handle_mouse_move(&mut self, pos: Point) {
        let previous_pos = self.mouse_position;
        self.sync_mouse_position(pos);
        self.maybe_start_drag(pos);
        self.maybe_move_active_drag_frame(previous_pos, pos);
        self.update_hovered_frame(pos);
    }

    pub(super) fn handle_mouse_leave(&mut self) {
        self.mouse_position = None;
        {
            let env = self.env.borrow();
            env.state().borrow_mut().set_mouse_position(None);
        }
        let cleared_pressed = self.pressed_frame.is_some();
        if cleared_pressed {
            self.clear_pressed_frame();
        }
        if self.hovered_frame.is_some() {
            self.fire_hover_transition(None);
            self.flush_mouse_move_visual_updates();
        } else if cleared_pressed {
            self.flush_mouse_move_visual_updates();
        }
    }

    fn sync_mouse_position(&mut self, pos: Point) {
        self.mouse_position = Some(pos);
        {
            let env = self.env.borrow();
            env.state()
                .borrow_mut()
                .set_mouse_position(Some((pos.x, pos.y)));
        }
    }

    fn maybe_start_drag(&mut self, pos: Point) {
        // Check drag threshold while mouse is held down.
        if let (Some(down_pos), Some(down_frame), false) =
            (self.mouse_down_pos, self.mouse_down_frame, self.dragging)
        {
            let dx = pos.x - down_pos.x;
            let dy = pos.y - down_pos.y;
            if (dx * dx + dy * dy).sqrt() >= DRAG_THRESHOLD {
                self.dragging = true;
                self.fire_drag_start(down_frame, "LeftButton");
                self.flush_post_script_updates();
            }
        }
    }

    fn maybe_move_active_drag_frame(&mut self, previous_pos: Option<Point>, pos: Point) {
        let Some(previous_pos) = previous_pos else {
            return;
        };

        let dx = pos.x - previous_pos.x;
        let dy = pos.y - previous_pos.y;
        if dx.abs() < f32::EPSILON && dy.abs() < f32::EPSILON {
            return;
        }

        if self.update_active_drag_motion(dx, dy) {
            self.mark_all_strata_dirty();
        }
    }

    fn update_active_drag_motion(&self, dx: f32, dy: f32) -> bool {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        let Some(active_drag_id) = state.active_drag_frame else {
            return false;
        };
        let drag_id = active_motion_drag_frame(&state, active_drag_id);
        let screen_size = self.screen_size.get();

        state.ensure_layout_rects();

        if let Some((new_width, new_height)) = sizing_drag_update(&state, drag_id, dx, dy) {
            apply_sizing(&mut state, drag_id, new_width, new_height);
            return true;
        }
        if let Some((parent_id, x_offset, y_offset)) = moving_drag_anchor_update(
            &state,
            drag_id,
            dx,
            dy,
            screen_size.width,
            screen_size.height,
        ) {
            reanchor_moving_drag_frame(&mut state, drag_id, parent_id, x_offset, y_offset);
            return true;
        }
        false
    }

    fn update_hovered_frame(&mut self, pos: Point) {
        let new_hovered = self.hit_test(pos);
        if new_hovered == self.hovered_frame {
            self.flush_mouse_move_visual_updates();
            return;
        }

        self.fire_hover_transition(new_hovered);
        // OnEnter/OnLeave scripts may show/hide tooltips or change widget
        // state; hit_test freshens the grid on entry, so the pending changes
        // coalesce until the next query instead of being applied eagerly here.
        self.flush_mouse_move_visual_updates();
    }

    fn fire_hover_transition(&mut self, new_hovered: Option<u64>) {
        // Update hovered_frame in both iced_app and SimState BEFORE firing events,
        // so IsMouseMotionFocus() / GetMouseFocus() return correct values in OnEnter.
        let old_hovered = self.hovered_frame;
        self.hovered_frame = new_hovered;
        {
            let env = self.env.borrow();
            {
                let mut state = env.state().borrow_mut();
                state.hovered_frame = new_hovered;
                mark_button_state_visuals_dirty(&mut state, old_hovered);
                mark_button_state_visuals_dirty(&mut state, new_hovered);
            }
            if let Some(old_id) = old_hovered.filter(|id| self.motion_scripts_allowed(*id)) {
                let _ = env.fire_script_handler(old_id, "OnLeave", vec![]);
            }
            if let Some(new_id) = new_hovered.filter(|id| self.motion_scripts_allowed(*id)) {
                let _ = env.fire_script_handler(new_id, "OnEnter", vec![]);
            }
        }
    }

    fn flush_mouse_move_visual_updates(&mut self) {
        let (dirty_mask, dirty_ids) = self
            .env
            .borrow()
            .state()
            .borrow()
            .widgets
            .take_render_dirty_with_ids();
        if dirty_mask != 0 {
            self.drain_console();
            self.mark_strata_dirty(dirty_mask);
            self.merge_pending_dirty_ids(dirty_ids);
        } else {
            self.drain_console();
        }
    }

    pub(super) fn handle_mouse_down(&mut self, pos: Point) {
        self.sync_mouse_position(pos);
        {
            let env = self.env.borrow();
            let _ = env
                .state()
                .borrow_mut()
                .set_mouse_button_down("LeftButton", true);
        }
        let hit_frame = self.hit_test_mouse_button(pos, "LeftButton", true);

        // Focus/unfocus EditBox on click
        self.update_editbox_focus(hit_frame);

        let Some(frame_id) = hit_frame else {
            return;
        };

        if !self.is_frame_enabled(frame_id) {
            return;
        }

        self.mouse_down_frame = Some(frame_id);
        self.mouse_down_pos = Some(pos);
        self.dragging = false;
        self.pressed_frame = Some(frame_id);
        self.mark_pressed_frame_visuals_dirty(Some(frame_id));
        {
            let env = self.env.borrow();
            let mut state = env.state().borrow_mut();
            let slider_drag_target = find_slider_drag_target(&state, frame_id);
            state.set_active_drag_frame(None);
            state.set_active_slider_thumb_drag_frame(slider_drag_target);
        }

        let clicks_on_down = self.frame_clicks_on_edge(frame_id, "LeftButton", true);
        {
            let env = self.env.borrow();
            let button_val = env.lua_string("LeftButton");
            if self.frame_mouse_on_edge(frame_id, "LeftButton", true) {
                let _ = env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val.clone()]);
            }
            if clicks_on_down {
                self.fire_left_click_sequence(frame_id, &env, &env.lua_string("LeftButton"), true);
            }
            self.fire_propagated_mouse_script(
                frame_id,
                &env,
                "OnMouseDown",
                "LeftButton",
                &button_val,
                true,
            );
        }
        self.flush_post_script_updates();
    }

    pub(super) fn handle_mouse_up(&mut self, pos: Point) {
        self.sync_mouse_position(pos);
        {
            let env = self.env.borrow();
            let _ = env
                .state()
                .borrow_mut()
                .set_mouse_button_down("LeftButton", false);
        }
        let (was_dragging, drag_source) = self.take_left_drag_state();
        let released_on = self.hit_test_mouse_button(pos, "LeftButton", false);

        if was_dragging {
            self.finish_drag(drag_source, released_on);
        } else {
            self.dispatch_left_mouse_release(released_on);
        }
        self.mouse_down_frame = None;
        self.clear_pressed_frame();
        self.flush_post_script_updates();
        self.update_hovered_frame(pos);
    }

    fn clear_pressed_frame(&mut self) {
        let old_pressed = self.pressed_frame.take();
        self.mark_pressed_frame_visuals_dirty(old_pressed);
    }

    fn mark_pressed_frame_visuals_dirty(&self, frame_id: Option<u64>) {
        let env = self.env.borrow();
        mark_button_state_visuals_dirty(&mut env.state().borrow_mut(), frame_id);
    }

    fn take_left_drag_state(&mut self) -> (bool, Option<u64>) {
        let was_dragging = self.dragging;
        let drag_source = self.env.borrow().state().borrow().active_drag_frame;
        self.mouse_down_pos = None;
        self.dragging = false;
        {
            let env = self.env.borrow();
            let mut state = env.state().borrow_mut();
            state.set_active_drag_frame(None);
            state.set_active_slider_thumb_drag_frame(None);
        }
        (was_dragging, drag_source)
    }

    fn dispatch_left_mouse_release(&mut self, released_on: Option<u64>) {
        let Some(frame_id) = released_on else {
            return;
        };

        if self.cursor_holds_item() {
            self.fire_receive_drag(frame_id);
            return;
        }

        let clicks_on_up = self.frame_clicks_on_edge(frame_id, "LeftButton", false);
        self.log_click_dispatch_if_enabled(frame_id, clicks_on_up);
        let env = self.env.borrow();
        let button_val = env.lua_string("LeftButton");

        if self.frame_mouse_on_edge(frame_id, "LeftButton", false) {
            let _ = env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val.clone()]);
        }

        if self.mouse_down_frame == Some(frame_id) && clicks_on_up {
            self.fire_left_click_sequence(frame_id, &env, &button_val, false);
        }

        self.fire_propagated_mouse_script(
            frame_id,
            &env,
            "OnMouseUp",
            "LeftButton",
            &button_val,
            false,
        );
    }

    fn cursor_holds_item(&self) -> bool {
        self.env.borrow().state().borrow().cursor_item.is_some()
    }

    /// Diagnostic trace for why a left-click did or didn't fire OnClick.
    /// Enable with WOW_SIM_DEBUG_CLICK_DISPATCH=1.
    fn log_click_dispatch_if_enabled(&self, frame_id: u64, clicks_on_up: bool) {
        if std::env::var_os("WOW_SIM_DEBUG_CLICK_DISPATCH").is_none() {
            return;
        }
        let env = self.env.borrow();
        let state = env.state().borrow();
        let describe = |id: u64| {
            state
                .widgets
                .get(id)
                .map(|frame| {
                    format!(
                        "{}(rect={:?})",
                        frame.name.as_deref().unwrap_or("<anon>"),
                        frame.layout_rect
                    )
                })
                .unwrap_or_else(|| "<missing>".to_string())
        };
        let registered = state
            .widgets
            .get(frame_id)
            .map(|frame| frame.registered_click_buttons.clone());
        let down_label = self.mouse_down_frame.map(describe);
        eprintln!(
            "[click-dispatch] release frame={frame_id} {} mouse_down_frame={:?} {down_label:?} clicks_on_up={clicks_on_up} registered={registered:?}",
            describe(frame_id),
            self.mouse_down_frame
        );
    }

    fn fire_left_click_sequence(
        &self,
        frame_id: u64,
        env: &crate::lua_api::WowLuaEnv,
        button_val: &Val,
        down: bool,
    ) {
        self.toggle_checkbutton_if_needed(frame_id, env);

        let down_val = Val::Bool(down);
        let _ = env.fire_script_handler(
            frame_id,
            "OnClick",
            vec![button_val.clone(), down_val.clone()],
        );

        // PostClick fires after OnClick (WoW secure button sequence).
        // ActionBar buttons use PostClick to call UpdateState().
        let _ = env.fire_script_handler(frame_id, "PostClick", vec![button_val.clone(), down_val]);
    }

    pub(super) fn handle_right_mouse_down(&mut self, pos: Point) {
        self.sync_mouse_position(pos);
        {
            let env = self.env.borrow();
            let _ = env
                .state()
                .borrow_mut()
                .set_mouse_button_down("RightButton", true);
        }
        let Some(frame_id) = self.hit_test_mouse_button(pos, "RightButton", true) else {
            return;
        };
        if !self.is_frame_enabled(frame_id) {
            return;
        }
        self.right_mouse_down_frame = Some(frame_id);
        let clicks_on_down = self.frame_clicks_on_edge(frame_id, "RightButton", true);
        {
            let env = self.env.borrow();
            let button_val = env.lua_string("RightButton");
            if self.frame_mouse_on_edge(frame_id, "RightButton", true) {
                let _ = env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val.clone()]);
            }
            if clicks_on_down {
                let down_val = Val::Bool(true);
                let _ = env.fire_script_handler(
                    frame_id,
                    "OnClick",
                    vec![env.lua_string("RightButton"), down_val.clone()],
                );
                let _ = env.fire_script_handler(
                    frame_id,
                    "PostClick",
                    vec![env.lua_string("RightButton"), down_val],
                );
            }
            self.fire_propagated_mouse_script(
                frame_id,
                &env,
                "OnMouseDown",
                "RightButton",
                &button_val,
                true,
            );
        }
        self.flush_post_script_updates();
    }

    pub(super) fn handle_right_mouse_up(&mut self, pos: Point) {
        self.sync_mouse_position(pos);
        {
            let env = self.env.borrow();
            let _ = env
                .state()
                .borrow_mut()
                .set_mouse_button_down("RightButton", false);
        }
        // Right-click clears the cursor (drops held spell/action) in WoW.
        // Use the Lua ClearCursor() function so events fire properly.
        let had_cursor_item = self.env.borrow().state().borrow().cursor_item.is_some();
        if had_cursor_item {
            eprintln!("[cursor] Right-click ClearCursor");
            {
                let env = self.env.borrow();
                let _ = env.call_global("ClearCursor", &[]);
            }
            self.right_mouse_down_frame = None;
            self.flush_post_script_updates();
            return;
        }

        let released_on = self.hit_test_mouse_button(pos, "RightButton", false);
        if let Some(frame_id) = released_on {
            let clicks_on_up = self.frame_clicks_on_edge(frame_id, "RightButton", false);
            {
                let env = self.env.borrow();
                let button_val = env.lua_string("RightButton");

                if self.right_mouse_down_frame == Some(frame_id) && clicks_on_up {
                    let down_val = Val::Bool(false);
                    let _ = env.fire_script_handler(
                        frame_id,
                        "OnClick",
                        vec![button_val.clone(), down_val.clone()],
                    );
                    let _ = env.fire_script_handler(
                        frame_id,
                        "PostClick",
                        vec![button_val.clone(), down_val],
                    );
                }

                if self.frame_mouse_on_edge(frame_id, "RightButton", false) {
                    let _ =
                        env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val.clone()]);
                }
                self.fire_propagated_mouse_script(
                    frame_id,
                    &env,
                    "OnMouseUp",
                    "RightButton",
                    &button_val,
                    false,
                );
            }
            self.flush_post_script_updates();
        }
        self.right_mouse_down_frame = None;
        self.update_hovered_frame(pos);
    }

    fn frame_clicks_on_edge(&self, frame_id: u64, button_name: &str, down: bool) -> bool {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let Some(frame) = state.widgets.get(frame_id) else {
            return false;
        };
        frame_click_registration_matches(frame, button_name, down)
    }

    fn frame_mouse_on_edge(&self, frame_id: u64, button_name: &str, down: bool) -> bool {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let Some(frame) = state.widgets.get(frame_id) else {
            return false;
        };
        crate::iced_app::frame_collect::frame_mouse_registration_matches(frame, button_name, down)
    }

    fn fire_propagated_mouse_script(
        &self,
        frame_id: u64,
        env: &crate::lua_api::WowLuaEnv,
        script_name: &str,
        button_name: &str,
        button_val: &Val,
        down: bool,
    ) {
        for parent_id in self.propagated_mouse_targets(frame_id, button_name, down) {
            let _ = env.fire_script_handler(parent_id, script_name, vec![button_val.clone()]);
        }
    }

    fn propagated_mouse_targets(&self, frame_id: u64, button_name: &str, down: bool) -> Vec<u64> {
        let env = self.env.borrow();
        let mut targets = Vec::new();
        let mut current_id = frame_id;

        loop {
            let forced = mouse_propagation_forced(&env, current_id);
            let state = env.state().borrow();
            let Some(current) = state.widgets.get(current_id) else {
                break;
            };
            if !current.propagate_mouse_clicks && !forced {
                break;
            }
            let Some(parent_id) = current.parent_id else {
                break;
            };
            let Some(parent) = state.widgets.get(parent_id) else {
                break;
            };
            let parent_accepts_button =
                crate::iced_app::frame_collect::frame_accepts_mouse_button(parent, button_name);
            let parent_accepts_edge =
                crate::iced_app::frame_collect::frame_mouse_registration_matches(
                    parent,
                    button_name,
                    down,
                );
            if parent_accepts_button && parent_accepts_edge {
                targets.push(parent_id);
            }
            current_id = parent_id;
        }

        targets
    }

    pub(super) fn handle_middle_click(&mut self, pos: Point) {
        if let Some(frame_id) = self.hit_test(pos) {
            self.populate_inspector(frame_id);
            self.inspected_frame = Some(frame_id);
            self.inspector_visible = true;
            self.inspector_position = Point::new(pos.x + 10.0, pos.y + 10.0);
        }
    }

    pub(super) fn handle_scroll(&mut self, _dx: f32, dy: f32) {
        if self.fire_mouse_wheel(dy) {
            self.invalidate_after_lua_mutation();
        } else {
            let scroll_speed = 30.0;
            self.scroll_offset -= dy * scroll_speed;
            let max_scroll = 2600.0;
            self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
            self.mark_all_strata_dirty();
        }
    }

    /// Fire OnDragStart on the source frame (walks up parent chain).
    fn fire_drag_start(&mut self, frame_id: u64, button_name: &str) {
        let drag_target = {
            let env = self.env.borrow();
            find_drag_script_target(&env, frame_id, "OnDragStart")
        };
        let Some(drag_target) = drag_target else {
            return;
        };

        let drag_button_registered = {
            let env = self.env.borrow();
            env.state()
                .borrow()
                .widgets
                .get(drag_target)
                .map(|frame| frame.registered_drag_buttons.contains(button_name))
                .unwrap_or(false)
        };
        if !drag_button_registered {
            return;
        }

        {
            let env = self.env.borrow();
            env.state()
                .borrow_mut()
                .set_active_drag_frame(Some(drag_target));
        }

        let env = self.env.borrow();
        let button_val = env.lua_string(button_name);
        eprintln!("[drag] OnDragStart fired on frame {}", drag_target);
        let _ = env.fire_script_handler(drag_target, "OnDragStart", vec![button_val]);
    }

    /// Fire OnDragStop on source and OnReceiveDrag on target.
    fn finish_drag(&mut self, source: Option<u64>, target: Option<u64>) {
        // Fire OnDragStop on the source frame (walk up parent chain).
        if let Some(src_id) = source {
            let env = self.env.borrow();
            let button_val = env.lua_string("LeftButton");
            if let Some(drag_stop_target) = find_drag_script_target(&env, src_id, "OnDragStop") {
                eprintln!("[drag] OnDragStop fired on frame {}", drag_stop_target);
                let _ = env.fire_script_handler(drag_stop_target, "OnDragStop", vec![button_val]);
            }
        }

        if let Some(tgt_id) = target {
            self.fire_receive_drag(tgt_id);
        }
    }

    /// Fire OnReceiveDrag on a frame (walks up parent chain).
    /// Used both at end of drag and on click when cursor holds an item.
    fn fire_receive_drag(&mut self, frame_id: u64) {
        let env = self.env.borrow();
        let button_val = env.lua_string("LeftButton");
        if let Some(receive_drag_target) = find_drag_script_target(&env, frame_id, "OnReceiveDrag")
        {
            eprintln!(
                "[drag] OnReceiveDrag fired on frame {}",
                receive_drag_target
            );
            let _ = env.fire_script_handler(receive_drag_target, "OnReceiveDrag", vec![button_val]);
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
            let (wheel_enabled, motion_allowed, parent_id) = {
                let state = env.state().borrow();
                state
                    .widgets
                    .get(frame_id)
                    .map(|frame| {
                        (
                            frame.mouse_wheel_enabled,
                            frame_motion_scripts_allowed(frame),
                            frame.parent_id,
                        )
                    })
                    .unwrap_or((false, false, None))
            };
            if motion_allowed && wheel_enabled && env.has_script_handler(frame_id, "OnMouseWheel") {
                let delta_val = Val::Num(dy as f64);
                let _ = env.fire_script_handler(frame_id, "OnMouseWheel", vec![delta_val]);
                return true;
            }
            current = parent_id;
        }
        false
    }

    /// Replay accepted `SimulateMouse*` input at the current cursor through the same
    /// handlers as physical input. Input queued by these handlers waits for the next tick.
    #[cfg(feature = "retail-12-0-7")]
    pub(super) fn dispatch_simulated_mouse_inputs(&mut self) {
        use crate::lua_api::globals::real::simulate_mouse::{
            SimulatedMouseButton::{Left, Right},
            SimulatedMouseInput::{Down, Up, Wheel},
        };
        let inputs = std::mem::take(
            &mut self
                .env
                .borrow()
                .state()
                .borrow_mut()
                .simulated_mouse_inputs,
        );
        // Without a cursor there is no mouse focus to receive the input.
        let Some(pos) = self.mouse_position else {
            return;
        };
        for input in inputs {
            match input {
                Down(Left) => self.handle_mouse_down(pos),
                Up(Left) => self.handle_mouse_up(pos),
                Down(Right) => self.handle_right_mouse_down(pos),
                Up(Right) => self.handle_right_mouse_up(pos),
                Wheel(delta) => {
                    if self.fire_mouse_wheel(delta) {
                        self.invalidate_after_lua_mutation();
                    }
                }
            }
        }
    }

    fn motion_scripts_allowed(&self, frame_id: u64) -> bool {
        let env = self.env.borrow();
        let state = env.state().borrow();
        state
            .widgets
            .get(frame_id)
            .map(frame_motion_scripts_allowed)
            .unwrap_or(false)
    }
}

pub(crate) fn frame_click_registration_matches(
    frame: &crate::widget::Frame,
    button_name: &str,
    down: bool,
) -> bool {
    if frame.registered_click_buttons.is_empty() {
        return matches!(frame.widget_type, crate::widget::WidgetType::Button)
            && !down
            && button_name == "LeftButton";
    }

    let edge = if down { "Down" } else { "Up" };
    registration_set_matches(
        &frame.registered_click_buttons,
        &format!("{button_name}{edge}"),
    ) || registration_set_matches(&frame.registered_click_buttons, &format!("Any{edge}"))
}

pub(crate) fn frame_click_registration_accepts_button(
    frame: &crate::widget::Frame,
    button_name: &str,
) -> bool {
    if frame.registered_click_buttons.is_empty() {
        return matches!(frame.widget_type, crate::widget::WidgetType::Button)
            && button_name == "LeftButton";
    }

    registration_set_matches(
        &frame.registered_click_buttons,
        &format!("{button_name}Down"),
    ) || registration_set_matches(&frame.registered_click_buttons, &format!("{button_name}Up"))
        || registration_set_matches(&frame.registered_click_buttons, "AnyDown")
        || registration_set_matches(&frame.registered_click_buttons, "AnyUp")
}

fn registration_set_matches(
    registrations: &std::collections::HashSet<String>,
    target: &str,
) -> bool {
    registrations
        .iter()
        .any(|registered| registered.eq_ignore_ascii_case(target))
}

fn mark_button_state_visuals_dirty(state: &mut crate::lua_api::SimState, frame_id: Option<u64>) {
    let Some(frame_id) = frame_id else {
        return;
    };
    state.widgets.mark_visual_dirty(frame_id);
    let Some(frame) = state.widgets.get(frame_id) else {
        return;
    };
    for parent_key in [
        "NormalTexture",
        "PushedTexture",
        "HighlightTexture",
        "DisabledTexture",
    ] {
        if let Some(child_id) = frame.children_keys.get(parent_key) {
            state.widgets.mark_visual_dirty(*child_id);
        }
    }
}

/// AlwaysPropagateInput forces click propagation regardless of SetPropagateMouseClicks.
fn mouse_propagation_forced(env: &crate::lua_api::WowLuaEnv, frame_id: u64) -> bool {
    let mut lua = env.rilua_mut();
    crate::lua_api::frame::methods::forbidden_aspects::input_propagation_forced(
        lua.state_mut(),
        frame_id,
    )
    .unwrap_or_else(|error| {
        eprintln!("forbidden-aspect propagation check failed for frame {frame_id}: {error}");
        false
    })
}

#[cfg(test)]
#[path = "mouse_test_modules.rs"]
mod mouse_test_modules;
