//! App::view() and subscription methods plus UI building helpers.

use iced::widget::shader::Shader;
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, space, stack, text,
    text_input, Column, Container,
};
use iced::{Border, Color, Element, Font, Length, Padding, Subscription};

use crate::LayoutRect;

use super::app::App;
use super::layout::compute_frame_rect;
use super::styles::{event_button_style, input_style, palette, pick_list_style, run_button_style};
use super::Message;

impl App {
    /// Build the title bar with FPS counter, frame time, canvas size, and mouse coords.
    fn build_title_bar(&self) -> Element<'_, Message> {
        let mouse_str = match self.mouse_position {
            Some(pos) => format!(" | mouse:({:.0},{:.0})", pos.x, pos.y),
            None => String::new(),
        };
        let screen = self.screen_size.get();
        let screen_str = format!(" | screen:{}x{}", screen.width as i32, screen.height as i32);
        let title_text = format!(
            "WoW UI Simulator  [{:.1} FPS | {:.2}ms{}{}]",
            self.fps, self.frame_time_display, screen_str, mouse_str
        );
        text(title_text).size(20).color(palette::GOLD).into()
    }

    /// Build the canvas area with optional inspector panel overlay.
    fn build_canvas_area(&self) -> Container<'_, Message> {
        let shader: Shader<Message, &App> = Shader::new(self)
            .width(Length::Fill)
            .height(Length::Fill);

        let stacked: Element<'_, Message> = if self.inspector_visible {
            let inspector = self.build_inspector_panel();
            stack![shader, inspector].into()
        } else {
            shader.into()
        };

        container(stacked)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_DARK)),
                border: Border {
                    color: palette::BORDER_HIGHLIGHT,
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
    }

    /// Build the collapsible frames sidebar panel.
    fn build_sidebar_panel(&self) -> Container<'_, Message> {
        let toggle_label = if self.frames_panel_collapsed { ">> Frames" } else { "<< Frames" };
        let toggle_btn = button(text(toggle_label).size(12))
            .on_press(Message::ToggleFramesPanel)
            .padding([2, 6])
            .style(|_, _| button::Style {
                background: None,
                text_color: palette::TEXT_PRIMARY,
                ..Default::default()
            });

        let panel_style = |_: &_| container::Style {
            background: Some(iced::Background::Color(palette::BG_PANEL)),
            border: Border { color: palette::BORDER, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        };

        if self.frames_panel_collapsed {
            container(toggle_btn).padding(6).style(panel_style)
        } else {
            let frames_list = self.build_frames_sidebar();
            container(
                column![toggle_btn, scrollable(frames_list).width(Length::Fill).height(600)]
                    .spacing(4),
            )
            .width(240)
            .padding(6)
            .style(panel_style)
        }
    }

    /// Build the player configuration row (class, race, damage level dropdowns).
    fn build_player_config_row(&self) -> Element<'_, Message> {
        use crate::lua_api::state::{CLASS_LABELS, RACE_DATA, ROT_DAMAGE_LEVELS};

        let class_options: Vec<String> = CLASS_LABELS.iter().map(|s| s.to_string()).collect();
        let race_options: Vec<String> = RACE_DATA.iter().map(|(n, _, _)| n.to_string()).collect();
        let rot_options: Vec<String> = ROT_DAMAGE_LEVELS.iter().map(|(l, _)| l.to_string()).collect();

        row![
            text("Class:").size(12).color(palette::TEXT_SECONDARY),
            pick_list(class_options, Some(self.selected_class.clone()), Message::PlayerClassChanged)
                .text_size(12).width(130).style(pick_list_style),
            text("Race:").size(12).color(palette::TEXT_SECONDARY),
            pick_list(race_options, Some(self.selected_race.clone()), Message::PlayerRaceChanged)
                .text_size(12).width(130).style(pick_list_style),
            space::horizontal(),
            checkbox(self.xp_bar_visible)
                .label("XP Bar").on_toggle(Message::ToggleXpBar).size(14).text_size(12),
            checkbox(self.rot_damage_enabled)
                .label("Rot Damage").on_toggle(Message::ToggleRotDamage).size(14).text_size(12),
            pick_list(rot_options, Some(self.selected_rot_level.clone()), Message::RotDamageLevelChanged)
                .text_size(12).width(120).style(pick_list_style),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Build the event trigger buttons row.
    fn build_event_buttons(&self) -> Element<'_, Message> {
        row![
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
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Build the command input row.
    fn build_command_row(&self) -> Element<'_, Message> {
        row![
            text_input("/command", &self.command_input)
                .on_input(Message::CommandInputChanged)
                .on_submit(Message::ExecuteCommand)
                .width(Length::Fill)
                .style(input_style),
            button(text("Run").size(12))
                .on_press(Message::ExecuteCommand)
                .style(run_button_style),
        ]
        .spacing(6)
        .into()
    }

    /// Build the console output area showing recent log messages.
    fn build_console(&self) -> Container<'_, Message> {
        let console_text: String = self
            .log_messages
            .iter()
            .rev()
            .take(5)
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        container(
            scrollable(
                text(console_text).size(12).color(palette::CONSOLE_TEXT).font(Font::MONOSPACE),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(80)
        .padding(6)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(palette::BG_INPUT)),
            border: Border { color: palette::BORDER, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = self.build_title_bar();
        let render_container = self.build_canvas_area();

        // Position sidebar at top-right corner, stack over canvas
        let sidebar_positioned = container(self.build_sidebar_panel())
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right);
        let content_row = stack![render_container, sidebar_positioned];

        let main_column = column![
            title,
            content_row,
            self.build_player_config_row(),
            self.build_event_buttons(),
            self.build_command_row(),
            self.build_console(),
        ]
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

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard = iced::event::listen_with(|event, status, _window| {
            use iced::keyboard;
            if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key, modifiers, text, ..
            }) = &event
            {
                // Ctrl+R is a simulator-only shortcut
                if modifiers.control() && *key == keyboard::Key::Character("r".into()) {
                    return Some(Message::ReloadUI);
                }
                // Only dispatch to Lua when no iced widget captured the event
                // (i.e., when the command input is not focused)
                if matches!(status, iced::event::Status::Ignored)
                    && let Some(wow_key) = super::keybinds::iced_key_to_wow(key) {
                        // Include raw text for character input into focused EditBox.
                        // Skip text when Ctrl/Alt modifiers are held (shortcuts, not typing).
                        let raw_text = if modifiers.control() || modifiers.alt() {
                            None
                        } else {
                            text.as_ref().map(|t| t.to_string())
                        };
                        return Some(Message::KeyPress(wow_key, raw_text));
                    }
            }
            None
        });

        if let Some(interval) = self.compute_tick_interval() {
            let timer = iced::time::every(interval).map(|_| Message::ProcessTimers);
            Subscription::batch([timer, keyboard])
        } else {
            keyboard
        }
    }

    pub(crate) fn build_frames_sidebar(&self) -> Column<'_, Message> {
        let mut col = Column::new().spacing(2);

        let env = self.env.borrow();
        let state = env.state().borrow();

        let mut count = 0;
        for id in state.widgets.iter_ids() {
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

                col = col.push(text(display).size(14).color(palette::TEXT_MUTED));

                count += 1;
                if count >= 15 {
                    break;
                }
            }
        }

        col
    }

    /// Build the inspector panel widget.
    pub(crate) fn build_inspector_panel(&self) -> Element<'_, Message> {
        let env = self.env.borrow();
        let state = env.state().borrow();

        let frame_id = self.inspected_frame.unwrap_or(0);
        let frame = state.widgets.get(frame_id);

        let (name, widget_type, computed_rect) = Self::inspector_frame_info(
            frame, frame_id, &state.widgets,
            self.screen_size.get().width, self.screen_size.get().height,
        );

        let title = Self::inspector_title_bar(&name, &widget_type);
        let id_row = text(format!("ID: {}  Pos: ({:.0}, {:.0})", frame_id, computed_rect.x, computed_rect.y))
            .size(11)
            .color(palette::TEXT_SECONDARY);
        let size_row = self.inspector_size_row();
        let alpha_level_row = self.inspector_alpha_level_row();
        let checkbox_row = self.inspector_checkbox_row();
        let anchors_display = Self::inspector_anchors_display(frame);

        let apply_btn = button(text("Apply").size(12))
            .on_press(Message::InspectorApply)
            .padding(Padding::from([4, 12]));

        let parent_chain = Self::inspector_parent_chain(&state.widgets, frame_id);

        let content = column![
            title,
            id_row,
            size_row,
            alpha_level_row,
            checkbox_row,
            parent_chain,
            text("Anchors:").size(11).color(palette::TEXT_SECONDARY),
            anchors_display,
            apply_btn,
        ]
        .spacing(6)
        .padding(8);

        Self::position_inspector_panel(content, self.inspector_position)
    }

    /// Extract name, type, and computed rect for the inspected frame.
    fn inspector_frame_info(
        frame: Option<&crate::widget::Frame>,
        frame_id: u64,
        widgets: &crate::widget::WidgetRegistry,
        screen_width: f32,
        screen_height: f32,
    ) -> (String, String, LayoutRect) {
        match frame {
            Some(f) => {
                let rect = compute_frame_rect(widgets, frame_id, screen_width, screen_height);
                (
                    f.name.clone().unwrap_or_else(|| "(anon)".to_string()),
                    f.widget_type.as_str().to_string(),
                    rect,
                )
            }
            None => ("(none)".to_string(), "".to_string(), LayoutRect::default()),
        }
    }

    /// Build the inspector title bar with close button.
    fn inspector_title_bar<'a>(name: &str, widget_type: &str) -> Element<'a, Message> {
        row![
            text(format!("{} [{}]", name, widget_type))
                .size(14)
                .color(palette::GOLD),
            space::horizontal(),
            button(text("x").size(14))
                .on_press(Message::InspectorClose)
                .padding(2)
                .style(|_, _| button::Style {
                    background: Some(iced::Background::Color(Color::TRANSPARENT)),
                    text_color: palette::TEXT_SECONDARY,
                    ..Default::default()
                }),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Build the width/height input row.
    fn inspector_size_row(&self) -> Element<'_, Message> {
        row![
            text("W:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.width)
                .on_input(Message::InspectorWidthChanged)
                .size(11)
                .width(50),
            text("H:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.height)
                .on_input(Message::InspectorHeightChanged)
                .size(11)
                .width(50),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Build the alpha/level input row.
    fn inspector_alpha_level_row(&self) -> Element<'_, Message> {
        row![
            text("Alpha:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.alpha)
                .on_input(Message::InspectorAlphaChanged)
                .size(11)
                .width(40),
            text("Level:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.frame_level)
                .on_input(Message::InspectorLevelChanged)
                .size(11)
                .width(40),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Build the visible/mouse checkboxes row.
    fn inspector_checkbox_row(&self) -> Element<'_, Message> {
        row![
            checkbox(self.inspector_state.visible)
                .label("Visible")
                .on_toggle(Message::InspectorVisibleToggled)
                .size(14)
                .text_size(11),
            checkbox(self.inspector_state.mouse_enabled)
                .label("Mouse")
                .on_toggle(Message::InspectorMouseEnabledToggled)
                .size(14)
                .text_size(11),
        ]
        .spacing(10)
        .into()
    }

    /// Format anchor info for the inspected frame.
    fn inspector_anchors_display<'a>(frame: Option<&crate::widget::Frame>) -> Element<'a, Message> {
        let anchors_text = match frame {
            Some(f) if !f.anchors.is_empty() => {
                let anchor_strs: Vec<String> = f
                    .anchors
                    .iter()
                    .map(|a| {
                        let rel = a.relative_to.as_deref().unwrap_or("$parent");
                        format!(
                            "{:?}->{} {:?} ({:.0},{:.0})",
                            a.point, rel, a.relative_point, a.x_offset, a.y_offset
                        )
                    })
                    .collect();
                anchor_strs.join("\n")
            }
            _ => "No anchors".to_string(),
        };
        text(anchors_text).size(10).color(palette::TEXT_MUTED).into()
    }

    /// Build parent chain display for the inspector.
    fn inspector_parent_chain<'a>(
        widgets: &crate::widget::WidgetRegistry,
        frame_id: u64,
    ) -> Element<'a, Message> {
        let mut ancestors = Vec::new();
        let mut current = widgets.get(frame_id).and_then(|f| f.parent_id);
        while let Some(pid) = current {
            let Some(parent) = widgets.get(pid) else { break };
            let name = parent.name.as_deref().unwrap_or("(anon)");
            ancestors.push(name.to_string());
            if ancestors.len() >= 6 {
                ancestors.push("...".to_string());
                break;
            }
            current = parent.parent_id;
        }
        ancestors.reverse();
        let self_name = widgets.get(frame_id)
            .and_then(|f| f.name.as_deref())
            .unwrap_or("(anon)");
        ancestors.push(self_name.to_string());
        let chain_text = ancestors.join(" > ");
        text(chain_text).size(10).color(palette::TEXT_MUTED).into()
    }

    /// Wrap inspector content in a positioned panel container.
    fn position_inspector_panel<'a>(
        content: Column<'a, Message>,
        position: iced::Point,
    ) -> Element<'a, Message> {
        let panel: Container<'a, Message> = container(content)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_PANEL)),
                border: Border {
                    color: palette::BORDER_HIGHLIGHT,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .width(220);

        let x_pad = position.x.max(0.0);
        let y_pad = position.y.max(0.0);

        container(panel)
            .padding(Padding::new(0.0).top(y_pad).left(x_pad))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Hit test to find frame under cursor (uses cached rects from render pass).
    ///
    /// After finding the topmost frame at the cursor position (highest strata/level),
    /// drills down through child frames to find the deepest mouse-enabled descendant.
    /// This matches WoW behavior where child frames always receive clicks over parents,
    /// regardless of the parent's frame level.
    pub(crate) fn hit_test(&self, pos: iced::Point) -> Option<u64> {
        let cache = self.cached_hittable.borrow();
        let list = cache.as_ref()?;

        // Find initial hit (highest strata/level frame containing the point).
        let initial_id = list.iter().rev().find_map(|(id, rect)| {
            if rect.contains(pos) { Some(*id) } else { None }
        })?;

        // Build lookup of hittable frame IDs → rects for fast child checks.
        let hittable: std::collections::HashMap<u64, &iced::Rectangle> =
            list.iter().map(|(id, rect)| (*id, rect)).collect();

        // Drill down through children: prefer the deepest mouse-enabled descendant.
        let env = self.env.borrow();
        let state = env.state().borrow();
        let mut current = initial_id;
        loop {
            let Some(frame) = state.widgets.get(current) else { break };
            let child_hit = frame.children.iter().rev().find(|&&cid| {
                hittable.get(&cid).is_some_and(|r| r.contains(pos))
            });
            match child_hit {
                Some(&cid) => current = cid,
                None => break,
            }
        }

        Some(current)
    }

}
