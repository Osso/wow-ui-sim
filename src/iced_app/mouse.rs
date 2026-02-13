//! Mouse event handlers for the iced application.

use iced::Point;

use super::app::App;

impl App {
    pub(super) fn handle_mouse_move(&mut self, pos: Point) {
        self.mouse_position = Some(pos);
        {
            let env = self.env.borrow();
            env.state().borrow_mut().mouse_position = Some((pos.x, pos.y));
        }
        let new_hovered = self.hit_test(pos);
        if new_hovered == self.hovered_frame {
            return;
        }

        // Update hovered_frame in both iced_app and SimState BEFORE firing events,
        // so IsMouseMotionFocus() / GetMouseFocus() return correct values in OnEnter.
        let old_hovered = self.hovered_frame;
        self.hovered_frame = new_hovered;
        {
            let env = self.env.borrow();
            env.state().borrow_mut().hovered_frame = new_hovered;
            if let Some(old_id) = old_hovered {
                let _ = env.fire_script_handler(old_id, "OnLeave", vec![]);
            }
            if let Some(new_id) = new_hovered {
                let _ = env.fire_script_handler(new_id, "OnEnter", vec![]);
            }
        }
        // OnEnter/OnLeave scripts may show/hide tooltips or change widget state.
        // Check if Lua mutated any widget and invalidate the quad cache if so.
        if self.env.borrow().state().borrow().widgets.take_render_dirty() {
            self.invalidate();
        } else {
            self.drain_console();
        }
    }

    pub(super) fn handle_mouse_down(&mut self, pos: Point) {
        let hit_frame = self.hit_test(pos);

        // Focus/unfocus EditBox on click
        self.update_editbox_focus(hit_frame);

        let Some(frame_id) = hit_frame else {
            return;
        };

        if !self.is_frame_enabled(frame_id) {
            return;
        }

        self.mouse_down_frame = Some(frame_id);
        self.pressed_frame = Some(frame_id);

        {
            let env = self.env.borrow();
            let button_val = mlua::Value::String(env.lua().create_string("LeftButton").unwrap());
            let _ = env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val]);
        }
        self.invalidate();
    }

    pub(super) fn handle_mouse_up(&mut self, pos: Point) {
        let released_on = self.hit_test(pos);
        if let Some(frame_id) = released_on {
            {
                let env = self.env.borrow();
                let button_val =
                    mlua::Value::String(env.lua().create_string("LeftButton").unwrap());

                if self.mouse_down_frame == Some(frame_id) {
                    self.toggle_checkbutton_if_needed(frame_id, &env);

                    let down_val = mlua::Value::Boolean(false);
                    let _ = env.fire_script_handler(
                        frame_id,
                        "OnClick",
                        vec![button_val.clone(), down_val.clone()],
                    );

                    // PostClick fires after OnClick (WoW secure button sequence).
                    // ActionBar buttons use PostClick to call UpdateState().
                    let _ = env.fire_script_handler(
                        frame_id,
                        "PostClick",
                        vec![button_val.clone(), down_val],
                    );
                }

                let _ = env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val]);
            }
            self.invalidate();
        }
        self.mouse_down_frame = None;
        self.pressed_frame = None;
    }

    pub(super) fn handle_right_mouse_down(&mut self, pos: Point) {
        let Some(frame_id) = self.hit_test(pos) else { return };
        if !self.is_frame_enabled(frame_id) { return }
        self.right_mouse_down_frame = Some(frame_id);
        {
            let env = self.env.borrow();
            let button_val = mlua::Value::String(env.lua().create_string("RightButton").unwrap());
            let _ = env.fire_script_handler(frame_id, "OnMouseDown", vec![button_val]);
        }
        self.invalidate();
    }

    pub(super) fn handle_right_mouse_up(&mut self, pos: Point) {
        let released_on = self.hit_test(pos);
        if let Some(frame_id) = released_on {
            {
                let env = self.env.borrow();
                let button_val =
                    mlua::Value::String(env.lua().create_string("RightButton").unwrap());

                if self.right_mouse_down_frame == Some(frame_id) {
                    let down_val = mlua::Value::Boolean(false);
                    let _ = env.fire_script_handler(
                        frame_id, "OnClick",
                        vec![button_val.clone(), down_val.clone()],
                    );
                    let _ = env.fire_script_handler(
                        frame_id, "PostClick",
                        vec![button_val.clone(), down_val],
                    );
                }

                let _ = env.fire_script_handler(frame_id, "OnMouseUp", vec![button_val]);
            }
            self.invalidate();
        }
        self.right_mouse_down_frame = None;
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
            self.invalidate();
        } else {
            let scroll_speed = 30.0;
            self.scroll_offset -= dy * scroll_speed;
            let max_scroll = 2600.0;
            self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
            self.invalidate_layout();
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
                let _ = env.fire_script_handler(frame_id, "OnMouseWheel", vec![delta_val]);
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
}
